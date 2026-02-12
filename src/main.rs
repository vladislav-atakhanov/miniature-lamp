mod layout;
mod transform;

use layout::Layout;

fn main() -> Result<(), String> {
    env_logger::init();
    let content = std::fs::read_to_string("./layout.txt").map_err(|e| e.to_string())?;
    let layout: Layout = content.parse()?;

    layout.vial(None)?;

    Ok(())
}
