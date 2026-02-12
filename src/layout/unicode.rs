use super::{Action, Keymap};
use keys::keys::Key;
use log::warn;
use s_expression::Expr::*;
use std::{collections::HashMap, str::FromStr};

pub fn unicode(
    ch: &char,
    keymap: &Keymap,
    en_hotkey: &Action,
    lang_hotkey: &Action,
) -> Result<Action, String> {
    let content = format!("({})", include_str!("unicode.txt"));
    let expr = s_expression::from_str(content.as_str()).map_err(|_| "Parse error")?;
    let list = expr.list()?;

    let keymaps = list.iter().try_fold(
        HashMap::<Keymap, HashMap<char, Action>>::with_capacity(list.len()),
        |mut acc, l| {
            let list = l.list()?;
            let [Atom(name), params @ ..] = list.as_slice() else {
                return Err(format!("Name of {} not found", l));
            };
            if *name != "defunicode" {
                return Err(format!("Unknown {:?}", name));
            }
            let [Atom(keymap), params @ ..] = params else {
                return Err(format!("Expected atom, found {:?}", params));
            };
            let keymap = keymap
                .parse::<Keymap>()
                .map_err(|_| format!("Keymap {:?} not found", keymap))?;
            if params.len() % 2 != 0 {
                return Err("Syntax error".to_string());
            }

            acc.insert(
                keymap.clone(),
                params.chunks(2).try_fold(
                    HashMap::with_capacity(params.len() / 2),
                    |mut acc, c| {
                        let [Atom(ch), action] = c else {
                            unreachable!()
                        };
                        let ch = match *ch {
                            "lb" => '(',
                            "rb" => ')',
                            _ => ch
                                .chars()
                                .next()
                                .ok_or(format!("Expected char, found {:?}", ch))?,
                        };
                        let action = Action::from_expr(action)?;
                        acc.insert(ch, action);
                        Ok::<_, String>(acc)
                    },
                )?,
            );
            Ok(acc)
        },
    )?;

    if let Some(chars) = keymaps.get(keymap) {
        if let Some(a) = chars.get(ch) {
            return Ok(a.clone());
        }
    };
    let chars = keymaps
        .get(&Keymap::En)
        .ok_or(format!("Symbols for {:?} not found", keymap))?;
    if let Some(a) = chars.get(ch) {
        return Ok(Action::Sequence(
            [en_hotkey.clone(), a.clone(), lang_hotkey.clone()].to_vec(),
        ));
    }
    if let Ok(key) = Key::from_str(format!("{}", ch).as_str()) {
        return Ok(Action::Tap(key));
    }
    warn!("raw unicode {:?}", ch);
    Ok(if ch.is_ascii() && !ch.is_control() {
        let digits = (*ch as u8).to_string();
        let mut res = Vec::with_capacity(digits.len() + 2);
        res.push(Action::Hold(Key::LeftAlt));

        digits.chars().try_for_each(|c| match c {
            '0'..='9' => {
                res.push(Action::Tap(Key::from_digit(c)));
                Ok(())
            }
            _ => Err(format!("Expected digit, found {:?}", c)),
        })?;
        res.push(Action::Release(Key::LeftAlt));
        Action::Sequence(res)
    } else {
        Action::Unicode(*ch)
    })
}
