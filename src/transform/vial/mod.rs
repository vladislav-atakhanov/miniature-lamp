mod keycode;
use keycode::{Keycode, format_mods, key_to_mod, key_to_string};
mod actions;
mod device;

use super::graph::{Node, priority_topo_sort};
use crate::layout::{Action, Layer, Layout};
use actions::{Macro, MacroAction, TapDance, VialAction};
use device::{get_device, unlock_device};
use hidapi::{HidApi, HidDevice};
use keys::keys::{Key, KeyIndex};
use parser::VialItem;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    io::Write,
    str::FromStr,
};
use vitaly::protocol;
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
        Ok(order
            .into_iter()
            .map(|n| self.layers.get(n).unwrap())
            .collect())
    }
    pub fn vial(&self, device_id: Option<u16>) -> Result<(), String> {
        let sorted = self.sorted_layers()?;
        let vial_items = self
            .keyboard
            .vial
            .ok_or("Vial is not defined".to_string())?;
        let api = HidApi::new().map_err(|e| e.to_string())?;

        let layers_by_name: HashMap<&str, usize> = sorted
            .iter()
            .enumerate()
            .map(|(i, l)| (l.name.as_str(), i))
            .collect();

        let mut vial = Vial {
            layers: &layers_by_name,
            macros: Default::default(),
            tap_dances: Default::default(),
            version: 6,
        };

        let mut layers: Vec<_> = sorted
            .iter()
            .map(|layer| {
                let keys: HashMap<&KeyIndex, Keycode> = layer
                    .keys
                    .iter()
                    .map(|(key_index, action)| {
                        let action = vial.action_to_keycode(action)?;
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

        let Some((device, capabilities, meta)) = get_device(&api, device_id) else {
            return Err("Device not found".to_string());
        };
        let mut macros: Vec<_> = vial.macros.iter().collect();
        macros.sort_by_key(|(_, i)| *i);
        let macros: Vec<_> = macros
            .into_iter()
            .enumerate()
            .map(|(i, (m, _))| protocol::Macro {
                index: i as u8,
                steps: m
                    .0
                    .iter()
                    .map(|s| match s {
                        MacroAction::Down(Keycode(action)) => protocol::MacroStep::Down(*action),
                        MacroAction::Up(Keycode(action)) => protocol::MacroStep::Up(*action),
                        MacroAction::Tap(Keycode(action)) => protocol::MacroStep::Tap(*action),
                        MacroAction::Delay(d) => protocol::MacroStep::Delay(*d),
                    })
                    .collect(),
            })
            .collect();

        let tap_dances: Vec<_> = vial
            .tap_dances
            .iter()
            .map(|(td, i)| protocol::TapDance {
                index: *i,
                tap: td.tap.0,
                hold: td.hold.0,
                double_tap: td.double_tap.0,
                tap_hold: td.tap_hold.0,
                tapping_term: td.tapping_term,
            })
            .collect();

        if capabilities.vial_version > 0 {
            unlock_device(&device, &meta, false)?;
            unlock_device(&device, &meta, true)?;
        }
        layers
            .into_iter()
            .enumerate()
            .try_for_each(|(layer_index, (_, mut keys))| {
                for (k, a) in keys.iter_mut() {
                    match vial_items
                        .get(k)
                        .ok_or(format!("Vial for {:?} not defined", k))?
                    {
                        VialItem::KeyCode(row, col) => {
                            protocol::set_keycode(&device, layer_index as u8, *row, *col, a.0)
                                .map_err(|e| e.to_string())?;
                        }
                        VialItem::Encoder(index, direction) => protocol::set_encoder(
                            &device,
                            layer_index as u8,
                            *index,
                            *direction,
                            a.0,
                        )
                        .map_err(|e| e.to_string())?,
                    };
                }
                println!("Layer {}", sorted.get(layer_index).unwrap().name);
                Ok::<_, String>(())
            })?;

        protocol::set_macros(&device, &capabilities, &macros).map_err(|e| e.to_string())?;
        println!("Macros");

        tap_dances
            .iter()
            .try_for_each(|td| protocol::set_tap_dance(&device, td).map_err(|e| e.to_string()))?;
        println!("Tap dance");

        if capabilities.vial_version > 0 {
            unlock_device(&device, &meta, false)?;
        }
        Ok(())
    }
}

struct Vial<'a> {
    macros: HashMap<Macro, u8>,
    tap_dances: HashMap<TapDance, u8>,
    layers: &'a HashMap<&'a str, usize>,
    version: u32,
}
impl<'a> Vial<'a> {
    fn layer_by_name(&self, name: &str) -> Option<usize> {
        self.layers.get(name).map(|x| *x)
    }

    pub fn action_to_vial(&mut self, action: &Action) -> Result<VialAction, String> {
        Ok(VialAction::Keycode(match action {
            Action::NoAction => Keycode(0),
            Action::Tap(k) => Keycode::from_key(k, self.version)?,
            Action::TapHold(tap, hold) => {
                if let Action::Tap(tap) = tap.as_ref() {
                    match hold.as_ref() {
                        Action::Tap(k) if k.is_modifier() => {
                            return Keycode::from_name(
                                format!(
                                    "{}_T({})",
                                    key_to_mod(k).ok_or(format!("Unreachable {:?}", k))?,
                                    key_to_string(tap)
                                ),
                                self.version,
                            )
                            .map(VialAction::Keycode);
                        }
                        Action::LayerSwitch(x) | Action::LayerWhileHeld(x) => {
                            let l = self
                                .layer_by_name(x)
                                .ok_or(format!("Layer {} not found", x))?;
                            return Keycode::from_name(
                                format!("LT({},{})", l, key_to_string(tap)),
                                self.version,
                            )
                            .map(VialAction::Keycode);
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
                                return Keycode::from_name(
                                    format!("MT({},{})", mods.join("|"), key_to_string(tap)),
                                    self.version,
                                )
                                .map(VialAction::Keycode);
                            }
                        }
                        _ => {}
                    }
                }

                return Ok(VialAction::tap_hold(
                    self.action_to_keycode(tap)?,
                    self.action_to_keycode(hold)?,
                ));
            }
            Action::Alias(_) | Action::Unicode(_) => {
                return Err(format!("Action {:?} not implemented", action));
            }
            Action::LayerSwitch(x) => {
                let layer = self
                    .layer_by_name(x)
                    .ok_or(format!("Layer {} not found", x))?;
                Keycode::from_name(format!("DF({})", layer), self.version)?
            }
            Action::LayerWhileHeld(x) => {
                let layer = self
                    .layer_by_name(x)
                    .ok_or(format!("Layer {} not found", x))?;

                Keycode::from_name(format!("MO({})", layer), self.version)?
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
                            return Keycode::from_name(
                                format!("{}({})", mods, key_to_string(tap)),
                                self.version,
                            )
                            .map(VialAction::Keycode);
                        }
                    }
                }
                let actions: Vec<_> = elems
                    .iter()
                    .map(|a| self.action_to_keycode(a).map(MacroAction::Tap))
                    .collect::<Result<_, String>>()?;

                return Ok(VialAction::Macro(Macro(actions)));
            }
            Action::Transparent => Keycode(1),
            Action::Sequence(actions) => {
                let act: Vec<_> = actions
                    .iter()
                    .map(|a| {
                        Ok(match a {
                            Action::Hold(key) => {
                                MacroAction::Down(Keycode::from_key(key, self.version)?)
                            }
                            Action::Release(key) => {
                                MacroAction::Up(Keycode::from_key(key, self.version)?)
                            }
                            a => MacroAction::Tap(self.action_to_keycode(a)?),
                        })
                    })
                    .collect::<Result<_, String>>()?;

                let result = act.iter().skip(1).fold(vec![act[0].clone()], |mut acc, x| {
                    if matches!(*acc.last().unwrap(), MacroAction::Tap(_))
                        && matches!(x, MacroAction::Tap(_))
                    {
                        acc.push(MacroAction::Delay(0));
                    }
                    acc.push(x.clone());
                    acc
                });
                return Ok(VialAction::Macro(Macro(result)));
            }
            Action::Hold(_) | Action::Release(_) => {
                return Err(format!("Action {:?} not in sequence", action));
            }
        }))
    }

    fn tap_dance(&mut self, td: TapDance) -> u8 {
        if let Some(i) = self.tap_dances.get(&td) {
            *i
        } else {
            let id = self.tap_dances.len() as u8;
            self.tap_dances.insert(td.clone(), id);
            id
        }
    }
    fn macros(&mut self, m: Macro) -> u8 {
        if let Some(id) = self.macros.get(&m) {
            *id
        } else {
            let id = self.macros.len() as u8;
            self.macros.insert(m.clone(), id);
            id
        }
    }

    fn action_to_keycode(self: &mut Vial<'a>, action: &Action) -> Result<Keycode, String> {
        Ok(match self.action_to_vial(action)? {
            VialAction::Keycode(keycode) => keycode,
            VialAction::TapDance(td) => {
                Keycode::from_name(format!("TD({})", self.tap_dance(td)), self.version)?
            }
            VialAction::Macro(m) => {
                Keycode::from_name(format!("M{}", self.macros(m)), self.version)?
            }
        })
    }
}
