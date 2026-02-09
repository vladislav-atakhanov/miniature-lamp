use std::str::FromStr;

#[rustfmt::skip]
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Key {
    KeyFn(u8),    KeyF13,      KeyF14,     KeyF15,   KeyF16, KeyF17, KeyF18, KeyF19, KeyF20,   KeyF21, KeyF22,       KeyF23,        KeyF24,
    KeyEsc,       KeyF1,       KeyF2,      KeyF3,    KeyF4,  KeyF5,  KeyF6,  KeyF7,  KeyF8,    KeyF9,  KeyF10,       KeyF11,        KeyF12,                      KeyPrint,  KeyScrollLock, KeyPause,
    KeyGrave,     Key1,        Key2,       Key3,     Key4,   Key5,   Key6,   Key7,   Key8,     Key9,   Key0,         KeyMinus,      KeyEqual,      KeyBackspace, KeyInsert, KeyHome,       KeyPageUp,   KeyNumlock, KeyKpSlash, KeyKpAsterisk, KeyKpMinus,
    KeyTab,       KeyQ,        KeyW,       KeyE,     KeyR,   KeyT,   KeyY,   KeyU,   KeyI,     KeyO,   KeyP,         KeyLeftBrace,  KeyRightBrace, KeyBackslash, KeyDelete, KeyEnd,        KeyPageDown, KeyKp7,     KeyKp8,     KeyKp9,        KeyKpPlus,
    KeyCapsLock,  KeyA,        KeyS,       KeyD,     KeyF,   KeyG,   KeyH,   KeyJ,   KeyK,     KeyL,   KeySemicolon, KeyApostrophe,                    KeyEnter,                                        KeyKp4,     KeyKp5,     KeyKp6,
    KeyLeftShift, KeyZ,        KeyX,       KeyC,     KeyV,   KeyB,   KeyN,   KeyM,   KeyComma, KeyDot, KeySlash,                                  KeyRightShift,            KeyUp,                      KeyKp1,     KeyKp2,     KeyKp3,        KeyKpEnter,
    KeyLeftCtrl,  KeyLeftMeta, KeyLeftAlt, KeySpace,                                                     KeyRightAlt,   KeyRightMeta, KeyMenu, KeyRightCtrl, KeyLeft,   KeyDown,       KeyRight,    KeyKp0,                 KeyKpDot,
}

impl FromStr for Key {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(d) = s.strip_prefix("fn") {
            let num: u8 = d.parse().map_err(|_| ())?;
            return Ok(Self::KeyFn(num));
        }
        if let Some(d) = s.strip_prefix("KeyFn") {
            let num: u8 = d.parse().map_err(|_| ())?;
            return Ok(Self::KeyFn(num));
        }
        if let Some(d) = s.strip_prefix("KeyF") {
            let num: u8 = d.parse().map_err(|_| ())?;
            return Ok(Self::KeyFn(num));
        }
        Ok(match s {
            "KeyEsc" | "esc" => Self::KeyEsc,
            "KeyF1" | "f1" => Self::KeyF1,
            "KeyF2" | "f2" => Self::KeyF2,
            "KeyF3" | "f3" => Self::KeyF3,
            "KeyF4" | "f4" => Self::KeyF4,
            "KeyF5" | "f5" => Self::KeyF5,
            "KeyF6" | "f6" => Self::KeyF6,
            "KeyF7" | "f7" => Self::KeyF7,
            "KeyF8" | "f8" => Self::KeyF8,
            "KeyF9" | "f9" => Self::KeyF9,
            "KeyF10" | "f10" => Self::KeyF10,
            "KeyF11" | "f11" => Self::KeyF11,
            "KeyF12" | "f12" => Self::KeyF12,
            "KeyF13" | "f13" => Self::KeyF13,
            "KeyF14" | "f14" => Self::KeyF14,
            "KeyF15" | "f15" => Self::KeyF15,
            "KeyF16" | "f16" => Self::KeyF16,
            "KeyF17" | "f17" => Self::KeyF17,
            "KeyF18" | "f18" => Self::KeyF18,
            "KeyF19" | "f19" => Self::KeyF19,
            "KeyF20" | "f20" => Self::KeyF20,
            "KeyF21" | "f21" => Self::KeyF21,
            "KeyF22" | "f22" => Self::KeyF22,
            "KeyF23" | "f23" => Self::KeyF23,
            "KeyF24" | "f24" => Self::KeyF24,
            "PrintScreen" => Self::KeyPrint,
            "ScrollLock" => Self::KeyScrollLock,
            "Pause" => Self::KeyPause,

            "Backquote" | "`" => Self::KeyGrave,
            "Digit1" | "1" => Self::Key1,
            "Digit2" | "2" => Self::Key2,
            "Digit3" | "3" => Self::Key3,
            "Digit4" | "4" => Self::Key4,
            "Digit5" | "5" => Self::Key5,
            "Digit6" | "6" => Self::Key6,
            "Digit7" | "7" => Self::Key7,
            "Digit8" | "8" => Self::Key8,
            "Digit9" | "9" => Self::Key9,
            "Digit0" | "0" => Self::Key0,
            "Minus" | "-" => Self::KeyMinus,
            "Equal" | "=" => Self::KeyEqual,
            "Backspace" | "bks" => Self::KeyBackspace,
            "Insert" | "ins" => Self::KeyInsert,
            "Home" | "home" => Self::KeyHome,
            "PageUp" | "pgup" => Self::KeyPageUp,
            "Numlock" => Self::KeyNumlock,

            "Tab" | "tab" => Self::KeyTab,
            "KeyQ" | "q" => Self::KeyQ,
            "KeyW" | "w" => Self::KeyW,
            "KeyE" | "e" => Self::KeyE,
            "KeyR" | "r" => Self::KeyR,
            "KeyT" | "t" => Self::KeyT,
            "KeyY" | "y" => Self::KeyY,
            "KeyU" | "u" => Self::KeyU,
            "KeyI" | "i" => Self::KeyI,
            "KeyO" | "o" => Self::KeyO,
            "KeyP" | "p" => Self::KeyP,
            "BracketLeft" | "[" => Self::KeyLeftBrace,
            "BracketRight" | "]" => Self::KeyRightBrace,
            "Backslash" | "\\" => Self::KeyBackslash,
            "Delete" | "del" => Self::KeyDelete,
            "End" | "end" => Self::KeyEnd,
            "PageDown" | "pgdn" => Self::KeyPageDown,

            "CapsLock" | "caps" => Self::KeyCapsLock,
            "KeyA" | "a" => Self::KeyA,
            "KeyS" | "s" => Self::KeyS,
            "KeyD" | "d" => Self::KeyD,
            "KeyF" | "f" => Self::KeyF,
            "KeyG" | "g" => Self::KeyG,
            "KeyH" | "h" => Self::KeyH,
            "KeyJ" | "j" => Self::KeyJ,
            "KeyK" | "k" => Self::KeyK,
            "KeyL" | "l" => Self::KeyL,
            "Semicolon" | ";" => Self::KeySemicolon,
            "Quote" | "'" => Self::KeyApostrophe,
            "Enter" | "ent" | "enter" => Self::KeyEnter,

            "LeftShift" | "sft" | "lsft" => Self::KeyLeftShift,
            "KeyZ" | "z" => Self::KeyZ,
            "KeyX" | "x" => Self::KeyX,
            "KeyC" | "c" => Self::KeyC,
            "KeyV" | "v" => Self::KeyV,
            "KeyB" | "b" => Self::KeyB,
            "KeyN" | "n" => Self::KeyN,
            "KeyM" | "m" => Self::KeyM,
            "Comma" | "," => Self::KeyComma,
            "Period" | "." => Self::KeyDot,
            "Slash" | "/" => Self::KeySlash,
            "RightShift" | "rsft" => Self::KeyRightShift,

            "Numpad0" | "kp0" => Self::KeyKp0,
            "Numpad1" | "kp1" => Self::KeyKp1,
            "Numpad2" | "kp2" => Self::KeyKp2,
            "Numpad3" | "kp3" => Self::KeyKp3,
            "Numpad4" | "kp4" => Self::KeyKp4,
            "Numpad5" | "kp5" => Self::KeyKp5,
            "Numpad6" | "kp6" => Self::KeyKp6,
            "Numpad7" | "kp7" => Self::KeyKp7,
            "Numpad8" | "kp8" => Self::KeyKp8,
            "Numpad9" | "kp9" => Self::KeyKp9,
            "NumpadPlus" | "kp+" => Self::KeyKpPlus,
            "NumpadEnter" | "kprt" => Self::KeyKpEnter,
            "NumpadDecimal" | "kp." => Self::KeyKpDot,
            "NumpadSlash" | "kp/" => Self::KeyKpSlash,
            "NumpadAsterisk" | "kp*" => Self::KeyKpAsterisk,
            "NumpadMinus" | "kp-" => Self::KeyKpMinus,

            "LeftCtrl" | "lctl" | "ctl" => Self::KeyLeftCtrl,
            "LeftMeta" | "lmeta" | "meta" => Self::KeyLeftMeta,
            "LeftAlt" | "lalt" | "alt" => Self::KeyLeftAlt,
            "Space" | "spc" => Self::KeySpace,
            "RightAlt" | "ralt" => Self::KeyRightAlt,
            "RightMeta" | "rmeta" => Self::KeyRightMeta,
            "Menu" | "menu" => Self::KeyMenu,
            "RightCtrl" | "rctl" => Self::KeyRightCtrl,

            "ArrowLeft" | "lt" => Self::KeyLeft,
            "ArrowDown" | "dn" => Self::KeyDown,
            "ArrowUp" | "up" => Self::KeyUp,
            "ArrowRight" | "rt" => Self::KeyRight,

            _ => return Err(()),
        })
    }
}
