use keys::keys::{Key, KeyIndex};
use s_expression::{Expr, Expr::*};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Item {
    KeyCode(u8, u8),
    Encoder(u8, u8),
}

#[derive(Debug, Default)]
pub struct Vial(HashMap<KeyIndex, Item>);

impl Vial {
    fn add(&mut self, item: Item) -> Option<Item> {
        self.0.insert(self.0.len().try_into().ok()?, item)
    }
    pub fn ok_or<E>(&self, e: E) -> Result<&HashMap<KeyIndex, Item>, E> {
        if self.0.len() > 0 {
            Ok(&self.0)
        } else {
            Err(e)
        }
    }
}

pub fn parse<'a>(items: &[Expr<'a>]) -> Result<Vial, String> {
    let mut vial = Vial(HashMap::new());
    items.iter().enumerate().try_for_each(|(i, x)| {
        let row = x.list()?.iter().filter_map(|e| match e {
            Atom(s) => Some(*s),
            _ => None,
        });
        let row: Vec<&str> = row.collect();
        let first = row.first().ok_or("Key not found".to_string())?;
        let key: Key = first
            .parse()
            .map_err(|_| format!("Unknown key {}", first))?;
        let item = match row.as_slice() {
            [a, b] | [a, b, _] => {
                let a = a.parse().map_err(|_| format!("Unknown value {}", a))?;
                let b = b.parse().map_err(|_| format!("Unknown value {}", b))?;

                Some(match row.get(2) {
                    Some(&"e") => Item::Encoder(a, b),
                    _ => Item::KeyCode(a, b),
                })
            }
            _ => None,
        };

        match vial.add(item.ok_or(format!("Unexpected {:?}", row))?) {
            None => {}
            _ => return Err(format!("Key {:?} already in map", key)),
        }
        Ok(())
    })?;
    Ok(vial)
}
