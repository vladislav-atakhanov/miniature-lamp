mod layout;
use std::collections::HashMap;

use keys::keys::{Key, KeyIndex};
use layout::Layout;

fn main() -> Result<(), String> {
    env_logger::init();
    let content = std::fs::read_to_string("./layout.txt").map_err(|e| e.to_string())?;
    let layout: Layout = content.parse()?;

    // layout
    //     .aliases
    //     .iter()
    //     .for_each(|(n, a)| println!("{} => {:?}", n, a));

    // let keys_by_index: HashMap<KeyIndex, Key> = layout.keys.iter().map(|(k, i)| (*i, *k)).collect();

    Ok(())
}
