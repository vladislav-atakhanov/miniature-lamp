use std::str::FromStr;

#[rustfmt::skip]
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Key {
    Fn(u8),    F13,      F14,     F15,   F16, F17, F18, F19, F20,   F21, F22,       F23,         F24,
    Esc,       F1,       F2,      F3,    F4,  F5,  F6,  F7,  F8,    F9,  F10,       F11,         F12,                   Print,  ScrollLock, Pause,
    Grave,     N1,       N2,      N3,    N4,  N5,  N6,  N7,  N8,    N9,  N0,        Minus,       Equal,        Backspace, Insert, Home,       PageUp,   Numlock, KpSlash, KpAsterisk, KpMinus,
    Tab,       Q,        W,       E,     R,   T,   Y,   U,   I,     O,   P,         LeftBracket, RightBracket, Backslash, Delete, End,        PageDown, Kp7,     Kp8,     Kp9,        KpPlus,
    CapsLock,  A,        S,       D,     F,   G,   H,   J,   K,     L,   Semicolon, Apostrophe,                    Enter,                               Kp4,     Kp5,     Kp6,
    LeftShift, Z,        X,       C,     V,   B,   N,   M,   Comma, Dot, Slash,                               RightShift,         Up,                   Kp1,     Kp2,     Kp3,        KpEnter,
    LeftCtrl,  LeftMeta, LeftAlt, Space,                                 RightAlt,  RightMeta,   Menu,         RightCtrl, Left,   Down,       Right,    Kp0,              KpDot,
}

impl FromStr for Key {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(d) = s.strip_prefix("fn") {
            let num: u8 = d.parse().map_err(|_| ())?;
            return Ok(Self::Fn(num));
        }
        if let Some(d) = s.strip_prefix("KeyFn") {
            let num: u8 = d.parse().map_err(|_| ())?;
            return Ok(Self::Fn(num));
        }
        if let Some(d) = s.strip_prefix("KeyF") {
            let num: u8 = d.parse().map_err(|_| ())?;
            return Ok(Self::Fn(num));
        }
        Ok(match s {
            "KeyEsc" | "esc" => Self::Esc,
            "KeyF1" | "f1" => Self::F1,
            "KeyF2" | "f2" => Self::F2,
            "KeyF3" | "f3" => Self::F3,
            "KeyF4" | "f4" => Self::F4,
            "KeyF5" | "f5" => Self::F5,
            "KeyF6" | "f6" => Self::F6,
            "KeyF7" | "f7" => Self::F7,
            "KeyF8" | "f8" => Self::F8,
            "KeyF9" | "f9" => Self::F9,
            "KeyF10" | "f10" => Self::F10,
            "KeyF11" | "f11" => Self::F11,
            "KeyF12" | "f12" => Self::F12,
            "KeyF13" | "f13" => Self::F13,
            "KeyF14" | "f14" => Self::F14,
            "KeyF15" | "f15" => Self::F15,
            "KeyF16" | "f16" => Self::F16,
            "KeyF17" | "f17" => Self::F17,
            "KeyF18" | "f18" => Self::F18,
            "KeyF19" | "f19" => Self::F19,
            "KeyF20" | "f20" => Self::F20,
            "KeyF21" | "f21" => Self::F21,
            "KeyF22" | "f22" => Self::F22,
            "KeyF23" | "f23" => Self::F23,
            "KeyF24" | "f24" => Self::F24,
            "PrintScreen" => Self::Print,
            "ScrollLock" => Self::ScrollLock,
            "Pause" => Self::Pause,

            "Backquote" | "`" => Self::Grave,
            "Digit1" | "1" => Self::N1,
            "Digit2" | "2" => Self::N2,
            "Digit3" | "3" => Self::N3,
            "Digit4" | "4" => Self::N4,
            "Digit5" | "5" => Self::N5,
            "Digit6" | "6" => Self::N6,
            "Digit7" | "7" => Self::N7,
            "Digit8" | "8" => Self::N8,
            "Digit9" | "9" => Self::N9,
            "Digit0" | "0" => Self::N0,
            "Minus" | "-" => Self::Minus,
            "Equal" | "=" => Self::Equal,
            "Backspace" | "bks" => Self::Backspace,
            "Insert" | "ins" => Self::Insert,
            "Home" | "home" => Self::Home,
            "PageUp" | "pgup" => Self::PageUp,
            "Numlock" => Self::Numlock,

            "Tab" | "tab" => Self::Tab,
            "KeyQ" | "q" => Self::Q,
            "KeyW" | "w" => Self::W,
            "KeyE" | "e" => Self::E,
            "KeyR" | "r" => Self::R,
            "KeyT" | "t" => Self::T,
            "KeyY" | "y" => Self::Y,
            "KeyU" | "u" => Self::U,
            "KeyI" | "i" => Self::I,
            "KeyO" | "o" => Self::O,
            "KeyP" | "p" => Self::P,
            "BracketLeft" | "[" => Self::LeftBracket,
            "BracketRight" | "]" => Self::RightBracket,
            "Backslash" | "\\" => Self::Backslash,
            "Delete" | "del" => Self::Delete,
            "End" | "end" => Self::End,
            "PageDown" | "pgdn" => Self::PageDown,

            "CapsLock" | "caps" => Self::CapsLock,
            "KeyA" | "a" => Self::A,
            "KeyS" | "s" => Self::S,
            "KeyD" | "d" => Self::D,
            "KeyF" | "f" => Self::F,
            "KeyG" | "g" => Self::G,
            "KeyH" | "h" => Self::H,
            "KeyJ" | "j" => Self::J,
            "KeyK" | "k" => Self::K,
            "KeyL" | "l" => Self::L,
            "Semicolon" | ";" => Self::Semicolon,
            "Quote" | "'" => Self::Apostrophe,
            "Enter" | "ent" | "enter" => Self::Enter,

            "LeftShift" | "sft" | "lsft" => Self::LeftShift,
            "KeyZ" | "z" => Self::Z,
            "KeyX" | "x" => Self::X,
            "KeyC" | "c" => Self::C,
            "KeyV" | "v" => Self::V,
            "KeyB" | "b" => Self::B,
            "KeyN" | "n" => Self::N,
            "KeyM" | "m" => Self::M,
            "Comma" | "," => Self::Comma,
            "Period" | "." => Self::Dot,
            "Slash" | "/" => Self::Slash,
            "RightShift" | "rsft" => Self::RightShift,

            "Numpad0" | "kp0" => Self::Kp0,
            "Numpad1" | "kp1" => Self::Kp1,
            "Numpad2" | "kp2" => Self::Kp2,
            "Numpad3" | "kp3" => Self::Kp3,
            "Numpad4" | "kp4" => Self::Kp4,
            "Numpad5" | "kp5" => Self::Kp5,
            "Numpad6" | "kp6" => Self::Kp6,
            "Numpad7" | "kp7" => Self::Kp7,
            "Numpad8" | "kp8" => Self::Kp8,
            "Numpad9" | "kp9" => Self::Kp9,
            "NumpadPlus" | "kp+" => Self::KpPlus,
            "NumpadEnter" | "kprt" => Self::KpEnter,
            "NumpadDecimal" | "kp." => Self::KpDot,
            "NumpadSlash" | "kp/" => Self::KpSlash,
            "NumpadAsterisk" | "kp*" => Self::KpAsterisk,
            "NumpadMinus" | "kp-" => Self::KpMinus,

            "LeftCtrl" | "lctl" | "ctl" => Self::LeftCtrl,
            "LeftMeta" | "lmeta" | "meta" => Self::LeftMeta,
            "LeftAlt" | "lalt" | "alt" => Self::LeftAlt,
            "Space" | "spc" => Self::Space,
            "RightAlt" | "ralt" => Self::RightAlt,
            "RightMeta" | "rmeta" => Self::RightMeta,
            "Menu" | "menu" => Self::Menu,
            "RightCtrl" | "rctl" => Self::RightCtrl,

            "ArrowLeft" | "lt" => Self::Left,
            "ArrowDown" | "dn" => Self::Down,
            "ArrowUp" | "up" => Self::Up,
            "ArrowRight" | "rt" => Self::Right,
            _ => return Err(()),
        })
    }
}
