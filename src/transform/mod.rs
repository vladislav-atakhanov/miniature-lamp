use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    fmt::Debug,
    io::Write,
};

use hidapi::{HidApi, HidDevice};
use keys::keys::{Key, KeyIndex};
use parser::VialItem;
use serde_json::Value;
use vitaly::{
    keycodes::{name_to_qid, qid_to_name},
    protocol,
};

use crate::layout::{Action, Layer, Layout};

pub fn load_meta(dev: &HidDevice) -> Result<Value, String> {
    let meta_data = match protocol::load_vial_meta(dev) {
        Ok(meta_data) => meta_data,
        Err(e) => {
            return Err(format!("failed to load vial meta {:?}", e));
        }
    };
    Ok(meta_data)
}

fn unlock_device(dev: &HidDevice, meta: &Value, unlock: bool) -> Result<(), String> {
    let mut status = protocol::get_locked_status(&dev).map_err(|e| e.to_string())?;
    if status.locked && unlock {
        println!("Starting unlock process... ");
        println!("Push marked buttons and keep then pushed to unlock...");
        let layout_options = &meta["layouts"]["labels"];
        let state = protocol::load_layout_options(&dev).map_err(|e| e.to_string())?;
        let options =
            protocol::LayoutOptions::from_json(state, layout_options).map_err(|e| e.to_string())?;
        let mut buttons = vitaly::keymap::keymap_to_buttons(&meta["layouts"]["keymap"], &options)
            .map_err(|e| e.to_string())?;
        let mut button_labels = HashMap::new();
        for (row, col) in &status.unlock_buttons {
            button_labels.insert((*row, *col), "☆☆,☆☆".to_string());
        }
        for button in &mut buttons {
            button.color = if status
                .unlock_buttons
                .contains(&(button.wire_x, button.wire_y))
            {
                Some((255, 255, 255))
            } else {
                None
            };
        }
        vitaly::keymap::render_and_dump(&buttons, Some(button_labels));
        if !status.unlock_in_progress {
            protocol::start_unlock(&dev).map_err(|e| e.to_string())?;
        }
        let sleep_duration = std::time::Duration::from_millis(100);
        let mut unlocked = false;
        let mut polls_remaining: u8;
        while !unlocked {
            std::thread::sleep(sleep_duration);
            (unlocked, polls_remaining) = protocol::unlock_poll(&dev).map_err(|e| e.to_string())?;
            print!("\r");
            print!(
                "Seconds remaining: {} keep pushing...",
                (polls_remaining as f64) / 10.0
            );
            std::io::stdout().flush().map_err(|e| e.to_string())?;
        }
        status = protocol::get_locked_status(&dev).map_err(|e| e.to_string())?;
        println!("\nDevice is locked: {}", status.locked);
    } else if !status.locked {
        println!("Locking keyboard...");
        protocol::set_locked(&dev).map_err(|e| e.to_string())?;
        status = protocol::get_locked_status(&dev).map_err(|e| e.to_string())?;
        println!("Device is locked: {}", status.locked);
    }

    Ok(())
}

#[derive(Debug)]
struct Node<'a> {
    weight: usize,
    deps: Vec<&'a str>,
}
fn priority_topo_sort<'a>(graph: &HashMap<&'a str, Node<'a>>) -> Result<Vec<&'a str>, String> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut reverse_graph: HashMap<&str, Vec<&str>> = HashMap::new();

    // Инициализация
    for (&name, node) in graph {
        in_degree.entry(name).or_insert(0);

        for &dep in &node.deps {
            reverse_graph.entry(dep).or_default().push(name);
            *in_degree.entry(name).or_insert(0) += 1;
        }
    }

    // Min-heap по weight
    let mut heap = BinaryHeap::new();

    for (&name, &deg) in &in_degree {
        if deg == 0 {
            let weight = graph[name].weight;
            heap.push((weight, name));
        }
    }

    let mut result = Vec::new();

    while let Some((_, node)) = heap.pop() {
        result.push(node);

        if let Some(dependents) = reverse_graph.get(node) {
            for &dep in dependents {
                let deg = in_degree.get_mut(dep).unwrap();
                *deg -= 1;

                if *deg == 0 {
                    let weight = graph[dep].weight;
                    heap.push((weight, dep));
                }
            }
        }
    }

    if result.len() != graph.len() {
        return Err("Cycle detected".into());
    }

    Ok(result)
}

impl Layout {
    fn sorted_layers(&self) -> Result<Vec<&Layer>, String> {
        let mut order = priority_topo_sort(
            &self
                .layers
                .iter()
                .map(|(_, l)| {
                    (
                        l.name.as_str(),
                        Node {
                            deps: l.get_dependencies(),
                            weight: l.index,
                        },
                    )
                })
                .collect(),
        )?;
        order.reverse();
        println!("{:?}", order);
        Ok(order
            .into_iter()
            .map(|n| self.layers.get(n).unwrap())
            .collect())
    }
    pub fn vial(&self, device_id: Option<u16>) -> Result<(), String> {
        let sorted = self.sorted_layers()?;
        let vial = self
            .keyboard
            .vial
            .ok_or("Vial is not defined".to_string())?;
        let api = HidApi::new().map_err(|e| e.to_string())?;

        let Some((device, capabilities, meta)) = get_device(&api, device_id) else {
            return Err("Device not found".to_string());
        };
        if capabilities.vial_version > 0 {
            unlock_device(&device, &meta, false)?;
            unlock_device(&device, &meta, true)?;
        }

        let version = capabilities.vial_version;
        let layers_by_name: HashMap<&str, usize> = sorted
            .iter()
            .enumerate()
            .map(|(i, l)| (l.name.as_str(), i))
            .collect();

        let mut layers: Vec<_> = sorted
            .iter()
            .map(|layer| {
                let keys: HashMap<&KeyIndex, VialAction> = layer
                    .keys
                    .iter()
                    .map(|(key_index, action)| {
                        let action = VialAction::from_action(
                            action,
                            &|name| layers_by_name.get(name).map(|x| *x),
                            version,
                        )?;
                        Ok::<_, String>((key_index, action))
                    })
                    .collect::<Result<_, _>>()?;
                let layer_index = layers_by_name
                    .get(layer.name.as_str())
                    .ok_or(format!("Layer {:?} not found", layer.name))?;
                Ok::<_, String>((*layer_index, keys))
            })
            .collect::<Result<_, _>>()?;

        layers.sort_by_key(|(n, _)| *n);
        let layers: Vec<_> = layers.into_iter().map(|(_, v)| v).collect();

        let mut macros: HashMap<Macro, u8> = Default::default();
        let mut tap_dances: HashMap<TapDance, u8> = Default::default();

        layers
            .iter()
            .enumerate()
            .try_for_each(|(layer_index, keys)| {
                let mut keys: Vec<_> = keys.iter().map(|(k, v)| (*k, v.clone())).collect();
                keys.sort_by_key(|(k, _)| *k);
                for (k, a) in keys.iter_mut() {
                    match a {
                        VialAction::Keycode(_, _) => {}
                        VialAction::TapDance(td) => {
                            let id = if let Some(i) = tap_dances.get(td) {
                                *i
                            } else {
                                let id = tap_dances.len() as u8;
                                tap_dances.insert(td.clone(), id);
                                id
                            };
                            *a = VialAction::keycode(format!("TD({})", id).to_string(), version)?;
                        }
                        VialAction::Macro(macro_actions) => {
                            let id = if let Some(id) = macros.get(macro_actions) {
                                *id
                            } else {
                                let id = macros.len() as u8;
                                macros.insert(macro_actions.clone(), id);
                                id
                            };
                            *a = VialAction::keycode(format!("M{}", id).to_string(), version)?;
                        }
                    }
                    match vial.get(k).ok_or(format!("Vial for {:?} not defined", k))? {
                        VialItem::KeyCode(row, col) => {
                            protocol::set_keycode(
                                &device,
                                layer_index as u8,
                                *row,
                                *col,
                                a.unwrap(),
                            )
                            .map_err(|e| e.to_string())?;
                        }
                        VialItem::Encoder(index, direction) => protocol::set_encoder(
                            &device,
                            layer_index as u8,
                            *index,
                            *direction,
                            a.unwrap(),
                        )
                        .map_err(|e| e.to_string())?,
                    };
                }
                println!("Layer {}", sorted.get(layer_index).unwrap().name);
                Ok::<_, String>(())
            })?;

        let mut macros: Vec<_> = macros.iter().collect();
        macros.sort_by_key(|(_, i)| *i);
        let macros: Vec<_> = macros.into_iter().map(|(x, _)| x).collect();

        protocol::set_macros(
            &device,
            &capabilities,
            &macros
                .iter()
                .enumerate()
                .map(|(i, m)| protocol::Macro {
                    index: i as u8,
                    steps: m
                        .0
                        .iter()
                        .map(|s| match s {
                            MacroAction::Down(action) => protocol::MacroStep::Down(action.unwrap()),
                            MacroAction::Up(action) => protocol::MacroStep::Up(action.unwrap()),
                            MacroAction::Tap(action) => protocol::MacroStep::Tap(action.unwrap()),
                            MacroAction::Delay(d) => protocol::MacroStep::Delay(*d),
                        })
                        .collect(),
                })
                .collect(),
        )
        .map_err(|e| e.to_string())?;
        println!("Macros");

        tap_dances.iter().try_for_each(|(td, i)| {
            protocol::set_tap_dance(
                &device,
                &protocol::TapDance {
                    index: *i,
                    tap: td.tap.unwrap(),
                    hold: td.hold.unwrap(),
                    double_tap: td.double_tap.unwrap(),
                    tap_hold: td.tap_hold.unwrap(),
                    tapping_term: td.tapping_term,
                },
            )
            .map_err(|e| e.to_string())
        })?;
        println!("Tap dance");
        if capabilities.vial_version > 0 {
            unlock_device(&device, &meta, false)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct TapDance {
    tap: Box<VialAction>,
    hold: Box<VialAction>,
    double_tap: Box<VialAction>,
    tap_hold: Box<VialAction>,
    tapping_term: u16,
}
impl PartialEq for TapDance {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}
impl Eq for TapDance {}
impl std::hash::Hash for TapDance {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{:?}", self).hash(state);
    }
}
impl Debug for TapDance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "tap-dance({:?}, {:?}, {:?}, {:?})",
            self.tap, self.hold, self.double_tap, self.tap_hold,
        )
    }
}

#[derive(Clone)]
struct Macro(Vec<MacroAction>);

impl Debug for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "macro(")?;
        self.0.iter().try_for_each(|a| write!(f, "{:?},", a))?;
        write!(f, ")")
    }
}
impl PartialEq for Macro {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}
impl Eq for Macro {}
impl std::hash::Hash for Macro {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{:?}", self).hash(state);
    }
}

#[derive(Clone)]
enum VialAction {
    Keycode(u16, u32),
    TapDance(TapDance),
    Macro(Macro),
}

#[derive(Debug, Clone)]
enum MacroAction {
    Down(VialAction),
    Up(VialAction),
    Tap(VialAction),
    Delay(u16),
}

impl Debug for VialAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VialAction::Keycode(x, version) => write!(f, "{}", qid_to_name(*x, *version)),
            VialAction::TapDance(td) => td.fmt(f),
            VialAction::Macro(v) => v.fmt(f),
        }
    }
}

impl VialAction {
    fn keycode(name: String, version: u32) -> Result<Self, String> {
        name_to_qid(name.as_str(), version)
            .map(|x| Self::Keycode(x, version))
            .map_err(|e| e.to_string())
    }
    fn unwrap(&self) -> u16 {
        match self {
            Self::Keycode(v, _) => *v,
            _ => panic!("Unwrap {:?}", self),
        }
    }
    fn tap_hold(tap: Self, hold: Self) -> Self {
        Self::TapDance(TapDance {
            tap: Box::new(tap.clone()),
            hold: Box::new(hold.clone()),
            double_tap: Box::new(Self::Keycode(0, 6)),
            tap_hold: Box::new(Self::Keycode(0, 6)),
            tapping_term: 200,
        })
    }
    fn from_action<'a, L>(
        action: &'a Action,
        layer_by_name: &'a L,
        version: u32,
    ) -> Result<Self, String>
    where
        L: Fn(&str) -> Option<usize>,
    {
        let s = match action {
            Action::NoAction => "KC_NO".to_string(),
            Action::Tap(k) => key_to_string(k).to_string(),
            Action::TapHold(tap, hold) => {
                if let Action::Tap(tap) = tap.as_ref() {
                    match hold.as_ref() {
                        Action::Tap(k) if k.is_modifier() => {
                            return Self::keycode(
                                format!(
                                    "{}_T({})",
                                    key_to_mod(k).ok_or(format!("Unreachable {:?}", k))?,
                                    key_to_string(tap)
                                ),
                                version,
                            );
                        }
                        Action::LayerSwitch(x) | Action::LayerWhileHeld(x) => {
                            if let Some(l) = layer_by_name(x) {
                                return Self::keycode(
                                    format!("LT({},{})", l, key_to_string(tap)),
                                    version,
                                );
                            } else {
                                return Err(format!("Layer {} not found", x));
                            }
                        }
                        Action::Multi(actions) => {
                            let mods: Vec<_> = actions
                                .iter()
                                .filter_map(|a| match a {
                                    Action::Tap(k) => key_to_mod(k),
                                    _ => None,
                                })
                                .collect();
                            if mods.len() == actions.len() {
                                return Self::keycode(
                                    format!("MT({},{})", mods.join("|"), key_to_string(tap)),
                                    version,
                                );
                            }
                        }
                        _ => {}
                    }
                }
                let tap = Self::from_action(tap, layer_by_name, version)?;
                let hold = Self::from_action(hold, layer_by_name, version)?;
                return Ok(Self::tap_hold(tap, hold));
            }
            Action::Alias(_) | Action::Unicode(_) => {
                return Err(format!("Action {:?} not implemented", action));
            }
            Action::LayerSwitch(x) => {
                if let Some(l) = layer_by_name(x) {
                    format!("DF({})", l)
                } else {
                    return Err(format!("Layer {} not found", x));
                }
            }
            Action::LayerWhileHeld(x) => {
                if let Some(l) = layer_by_name(x) {
                    format!("MO({})", l)
                } else {
                    return Err(format!("Layer {} not found", x));
                }
            }
            Action::Multi(elems) => {
                let taps: Vec<_> = elems
                    .iter()
                    .map_while(|a| {
                        if let Action::Tap(k) = a {
                            Some(k)
                        } else {
                            None
                        }
                    })
                    .collect();
                if taps.len() == elems.len() {
                    let (mods, keys): (Vec<&Key>, Vec<&Key>) =
                        taps.iter().partition(|k| k.is_modifier());
                    if let [tap] = keys.as_slice() {
                        if let Some(mods) = format_mods(mods.as_slice()) {
                            return Self::keycode(
                                format!("{}({})", mods, key_to_string(tap)),
                                version,
                            );
                        }
                    }
                }
                let actions: Vec<_> = elems
                    .iter()
                    .map(|a| Self::from_action(a, layer_by_name, version).map(MacroAction::Tap))
                    .collect::<Result<_, _>>()?;

                return Ok(Self::Macro(Macro(actions)));
            }
            Action::Transparent => "KC_TRANSPARENT".to_string(),
            Action::Sequence(actions) => {
                return Ok(Self::Macro(Macro(
                    actions
                        .iter()
                        .map(|a| match a {
                            Action::Hold(key) => {
                                Self::keycode(key_to_string(key).to_string(), version)
                                    .map(|a| MacroAction::Down(a))
                            }
                            Action::Release(key) => {
                                Self::keycode(key_to_string(key).to_string(), version)
                                    .map(|a| MacroAction::Up(a))
                            }
                            a => Self::from_action(a, layer_by_name, version)
                                .map(|a| MacroAction::Tap(a)),
                        })
                        .collect::<Result<_, _>>()?,
                )));
            }
            Action::Hold(_) | Action::Release(_) => {
                return Err(format!("Action {:?} not in sequence", action));
            }
        };
        Self::keycode(s, version)
    }
}

fn format_mods(mods: &[&Key]) -> Option<&'static str> {
    let set: HashSet<_> = mods.iter().copied().map(|k| k.clone()).collect();
    if set == HashSet::from([Key::LeftCtrl, Key::LeftShift, Key::LeftAlt, Key::LeftMeta]) {
        Some("HYPR")
    } else if set == HashSet::from([Key::LeftCtrl, Key::LeftShift, Key::LeftAlt]) {
        Some("MEH")
    } else if set == HashSet::from([Key::LeftCtrl, Key::LeftAlt, Key::LeftMeta]) {
        Some("LCAG")
    } else if set == HashSet::from([Key::LeftCtrl, Key::LeftShift]) {
        Some("LCS")
    } else if set == HashSet::from([Key::LeftCtrl, Key::LeftAlt]) {
        Some("LCA")
    } else if set == HashSet::from([Key::LeftCtrl, Key::LeftMeta]) {
        Some("LCG")
    } else if set == HashSet::from([Key::RightCtrl, Key::RightMeta]) {
        Some("RCG")
    } else if set == HashSet::from([Key::LeftShift, Key::LeftAlt]) {
        Some("LSA")
    } else if set == HashSet::from([Key::LeftShift, Key::LeftMeta]) {
        Some("LSG")
    } else if set.len() == 1 {
        Some(match mods[0] {
            Key::LeftCtrl => "LCTL",
            Key::RightCtrl => "RCTL",
            Key::LeftShift => "LSFT",
            Key::RightShift => "RSFT",
            Key::LeftAlt => "LALT",
            Key::RightAlt => "RALT",
            Key::LeftMeta => "LGUI",
            Key::RightMeta => "RGUI",
            _ => return None,
        })
    } else {
        None
    }
}

fn key_to_mod(key: &Key) -> Option<&str> {
    Some(match key {
        Key::LeftAlt => "LALT",
        Key::RightAlt => "RALT",
        Key::LeftCtrl => "LCTL",
        Key::RightCtrl => "RCTL",
        Key::LeftShift => "LSFT",
        Key::RightShift => "RSFT",
        Key::LeftMeta => "LGUI",
        Key::RightMeta => "RGUI",
        _ => return None,
    })
}

fn key_to_string(key: &Key) -> &str {
    match key {
        Key::Q => "KC_Q",
        Key::W => "KC_W",
        Key::E => "KC_E",
        Key::R => "KC_R",
        Key::T => "KC_T",
        Key::Y => "KC_Y",
        Key::U => "KC_U",
        Key::I => "KC_I",
        Key::O => "KC_O",
        Key::P => "KC_P",
        Key::A => "KC_A",
        Key::S => "KC_S",
        Key::D => "KC_D",
        Key::F => "KC_F",
        Key::G => "KC_G",
        Key::H => "KC_H",
        Key::J => "KC_J",
        Key::K => "KC_K",
        Key::L => "KC_L",
        Key::Z => "KC_Z",
        Key::X => "KC_X",
        Key::C => "KC_C",
        Key::V => "KC_V",
        Key::B => "KC_B",
        Key::N => "KC_N",
        Key::M => "KC_M",

        Key::Zero => "KC_0",
        Key::One => "KC_1",
        Key::Two => "KC_2",
        Key::Three => "KC_3",
        Key::Four => "KC_4",
        Key::Five => "KC_5",
        Key::Six => "KC_6",
        Key::Seven => "KC_7",
        Key::Eight => "KC_8",
        Key::Nine => "KC_9",

        Key::Fn(_) => "KC_NO",
        Key::F1 => "KC_F1",
        Key::F2 => "KC_F2",
        Key::F3 => "KC_F3",
        Key::F4 => "KC_F4",
        Key::F5 => "KC_F5",
        Key::F6 => "KC_F6",
        Key::F7 => "KC_F7",
        Key::F8 => "KC_F8",
        Key::F9 => "KC_F9",
        Key::F10 => "KC_F10",
        Key::F11 => "KC_F11",
        Key::F12 => "KC_F12",
        Key::F13 => "KC_F13",
        Key::F14 => "KC_F14",
        Key::F15 => "KC_F15",
        Key::F16 => "KC_F16",
        Key::F17 => "KC_F17",
        Key::F18 => "KC_F18",
        Key::F19 => "KC_F19",
        Key::F20 => "KC_F20",
        Key::F21 => "KC_F21",
        Key::F22 => "KC_F22",
        Key::F23 => "KC_F23",
        Key::F24 => "KC_F24",

        Key::VolumeUp => "KC_AUDIO_VOL_UP",
        Key::VolumeDown => "KC_AUDIO_VOL_DOWN",
        Key::VolumeMute => "KC_AUDIO_MUTE",
        Key::Esc => "KC_ESCAPE",

        Key::Print => "KC_PRINT_SCREEN",
        Key::ScrollLock => "KC_SCROLL_LOCK",
        Key::Pause => "KC_PAUSE",
        Key::Grave => "KC_GRAVE",

        Key::Minus => "KC_MINUS",
        Key::Equal => "KC_EQUAL",
        Key::Backspace => "KC_BACKSPACE",
        Key::Insert => "KC_INSERT",
        Key::Home => "KC_HOME",
        Key::PageUp => "KC_PAGE_UP",
        Key::Numlock => "KC_NUM_LOCK",
        Key::KpSlash => "KC_KP_SLASH",
        Key::KpAsterisk => "KC_KP_ASTERISK",
        Key::KpMinus => "KC_KP_MINUS",
        Key::Tab => "KC_TAB",

        Key::LeftBracket => "KC_LEFT_BRACKET",
        Key::RightBracket => "KC_RIGHT_BRACKET",
        Key::Backslash => "KC_BACKSLASH",
        Key::Delete => "KC_DELETE",
        Key::End => "KC_END",
        Key::PageDown => "KC_PAGE_DOWN",

        Key::KpPlus => "KC_KP_PLUS",
        Key::CapsLock => "KC_CAPS_LOCK",

        Key::Semicolon => "KC_SEMICOLON",
        Key::Apostrophe => "KC_QUOTE",
        Key::Enter => "KC_ENTER",

        Key::LeftShift => "KC_LEFT_SHIFT",

        Key::Comma => "KC_COMMA",
        Key::Dot => "KC_DOT",
        Key::Slash => "KC_SLASH",
        Key::RightShift => "KC_RIGHT_SHIFT",
        Key::Up => "KC_UP",

        Key::Kp0 => "KC_KP_0",
        Key::Kp1 => "KC_KP_1",
        Key::Kp2 => "KC_KP_2",
        Key::Kp3 => "KC_KP_3",
        Key::Kp4 => "KC_KP_4",
        Key::Kp5 => "KC_KP_5",
        Key::Kp6 => "KC_KP_6",
        Key::Kp7 => "KC_KP_7",
        Key::Kp8 => "KC_KP_8",
        Key::Kp9 => "KC_KP_9",

        Key::KpEqual => "KC_KP_EQUAL",
        Key::LeftCtrl => "KC_LEFT_CTRL",
        Key::LeftMeta => "KC_LEFT_GUI",
        Key::LeftAlt => "KC_LEFT_ALT",
        Key::Space => "KC_SPACE",
        Key::RightAlt => "KC_RIGHT_ALT",
        Key::RightMeta => "KC_RIGHT_GUI",
        Key::Menu => "KC_APPLICATION",
        Key::RightCtrl => "KC_RIGHT_CTRL",
        Key::Left => "KC_LEFT",
        Key::Down => "KC_DOWN",
        Key::Right => "KC_RIGHT",
        Key::KpDot => "KC_KP_DOT",
        Key::KpEnter => "KC_KP_ENTER",

        Key::MediaPlayPause => "KC_MEDIA_PLAY_PAUSE",

        Key::MouseCursorUp => "KC_MS_UP",
        Key::MouseCursorDown => "KC_MS_DOWN",
        Key::MouseCursorLeft => "KC_MS_LEFT",
        Key::MouseCursorRight => "KC_MS_RIGHT",
        Key::MouseWheelUp => "KC_WH_U",
        Key::MouseWheelDown => "KC_WH_D",
        Key::MouseWheelLeft => "KC_WH_L",
        Key::MouseWheelRight => "KC_WH_R",
        Key::MouseButton1 => "KC_MS_BTN1",
        Key::MouseButton2 => "KC_MS_BTN2",
        Key::MouseButton3 => "KC_MS_BTN3",
        Key::MouseButton4 => "KC_MS_BTN4",
        Key::MouseButton5 => "KC_MS_BTN5",
        Key::MouseAcceleration0 => "KC_MS_ACCEL0",
        Key::MouseAcceleration1 => "KC_MS_ACCEL1",
        Key::MouseAcceleration2 => "KC_MS_ACCEL2",
    }
}

fn get_device(
    api: &HidApi,
    device_id: Option<u16>,
) -> Option<(HidDevice, protocol::Capabilities, Value)> {
    api.device_list().find_map(|device| {
        if let Some(id) = device_id
            && device.product_id() != id
        {
            return None;
        }

        if device.usage_page() == protocol::USAGE_PAGE && device.usage() == protocol::USAGE_ID {
            let device_path = device.path();
            let dev = api.open_path(device_path).ok()?;
            let capabilities = protocol::scan_capabilities(&dev).ok()?;
            let meta = load_meta(&dev).ok()?;
            meta["matrix"]["cols"].as_u64()?;
            meta["matrix"]["rows"].as_u64()?;
            Some((dev, capabilities, meta))
        } else {
            None
        }
    })
}
