use keys::keys::{Key, KeyIndex};
use log::warn;
use s_expression::Expr::{self, *};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

mod action;
mod layer;
mod template;
mod unwrap;
use layer::Layer;

use crate::layout::action::Action;

fn preprocess<'a>(expr: &Expr<'a>) -> Result<Expr<'a>, String> {
    let mut templates = template::Templates::new();
    let root = expr.list()?;

    root.iter().try_for_each(|item| -> Result<(), String> {
        let lst = item.list()?;
        let name = lst.first().ok_or("Not found".to_string())?;
        if name.atom()? == "deftemplate" {
            templates.extend(template::deftemplate(lst[1..].to_vec())?)
        }
        Ok(())
    })?;
    let root = template::expand(expr, &templates);
    let root = unwrap::unwrap(&root, Some(&HashSet::from(["deftemplate"])));
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert<'a, F>(input: &'a str, output: &'a str, f: F)
    where
        F: Fn(&Expr<'a>) -> Expr<'a>,
    {
        let input = s_expression::from_str(input).unwrap();
        let output = s_expression::from_str(output).unwrap();
        assert_eq!(f(&input).to_string(), output.to_string())
    }

    #[test]
    fn preprocess_aliases() {
        assert(
            r#"(
                (deftemplate app (x) (multi meta x))
                (defalias
                    a0 (app 0)
                    a1 (app 1)
                    a2 (app 2)
                )
            )"#,
            r#"(
                (defalias
                    a0 (multi meta 0)
                    a1 (multi meta 1)
                    a2 (multi meta 2)
                )
            )"#,
            |e| preprocess(e).unwrap(),
        );
    }
}

#[derive(Debug, Default)]
pub struct Layout {
    pub keys: HashMap<Key, KeyIndex>,
    pub layers: HashMap<String, Layer>,
}
impl Layout {
    fn new() -> Self {
        Self::default()
    }
    fn layer_from(&self, parent: String, name: String) -> Result<Layer, String> {
        let Some(parent) = self
            .layers
            .get(&name)
            .or_else(|| self.layers.get(&parent))
            .or_else(|| self.layers.get(&"src".to_string()))
        else {
            return Err(format!("Layer {:?} not defined", parent));
        };
        if parent.name == name {
            Ok(parent.clone())
        } else {
            Ok(Layer {
                name: name,
                parent: parent.name.clone(),
                keys: parent.keys.clone(),
            })
        }
    }
}

impl FromStr for Layout {
    type Err = String;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let content = format!("({})", content);
        let expr = s_expression::from_str(content.as_str()).map_err(|_| "Parse error")?;
        let mut layout = Self::new();

        let root = preprocess(&expr)?;
        let mut aliases: HashMap<String, Action> = HashMap::new();
        root.list()?
            .iter()
            .try_for_each(|r| -> Result<(), String> {
                let [name, params @ ..] = r.list()?.as_slice() else {
                    return Err("Exprected name".to_string());
                };
                match name.atom()? {
                    "keyboard" => {
                        let [Atom(id)] = params else {
                            return Err("Syntax error".to_string());
                        };
                        let content = std::fs::read_to_string(format!("./keyboards/{}.txt", id))
                            .map_err(|e| e.to_string())?;
                        let keyboard = parser::parse(content.as_str())?;
                        layout.keys = keyboard.source;
                        let src = Layer::from_keyboard(&layout.keys);
                        layout.layers.insert(src.name.to_string(), src);
                    }
                    "deflayer" => {
                        let layer = Layer::from_def(params)?;
                        if layer.keys.len() != layout.keys.len() {
                            return Err(format!(
                                "Syntax error: expected {}, found {} ({})",
                                layout.keys.len(),
                                layer.keys.len(),
                                name
                            ));
                        }
                        layout.layers.insert(layer.name.to_string(), layer);
                    }
                    "deflayermap" => {
                        let layer = Layer::from_map(params, &layout.keys)?;
                        let mut l = layout.layer_from(layer.parent, layer.name)?;
                        l.keys.extend(layer.keys);
                        layout.layers.insert(l.name.to_string(), l);
                    }
                    "defalias" => {
                        aliases.extend(
                            params
                                .chunks(2)
                                .map(|x| {
                                    let [Atom(name), expr] = x else {
                                        println!("{:?}", x);
                                        return Err(format!("Syntax error: {:?}", x));
                                    };
                                    let action = Action::from_expr(expr)?;
                                    Ok((name.to_string(), action))
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                    }
                    "defoverride" => {}
                    _ => return Err(format!("Unexpected {}", name)),
                }
                Ok(())
            })?;
        Ok(layout)
    }
}
