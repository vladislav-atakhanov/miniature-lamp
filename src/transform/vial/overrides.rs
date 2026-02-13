use keys::keys::Key;
use vitaly::protocol::KeyOverride;

use crate::transform::vial::keycode::key_to_mod;

use super::keycode::Keycode;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Override {
    pub source: Keycode,
    pub target: Keycode,
    pub source_mods: Vec<Key>,
    pub target_mods: Vec<Key>,
}

impl Override {
    pub fn to_key_override(&self, layers_mask: u16, i: usize) -> KeyOverride {
        let mut target = self.target_mods.clone();
        target.extend(self.source_mods.clone());
        KeyOverride {
            index: i as u8,
            ko_enabled: true,
            trigger: self.source.0,
            replacement: self.target.0,
            layers: layers_mask,
            trigger_mods: mods_to_mask(&self.source_mods).unwrap(),
            negative_mod_mask: 0,
            suppressed_mods: mods_to_mask(&target).unwrap(),
            ko_option_activation_trigger_down: true,
            ko_option_activation_required_mod_down: true,
            ko_option_activation_negative_mod_up: true,
            ko_option_one_mod: false,
            ko_option_no_reregister_trigger: false,
            ko_option_no_unregister_on_other_key_down: false,
        }
    }
}
fn mods_to_mask(mods: &Vec<Key>) -> Result<u8, String> {
    if mods.len() == 0 {
        return Ok(0);
    }
    vitaly::keycodes::name_to_bitmod(
        mods.iter()
            .filter_map(|m| key_to_mod(m))
            .collect::<Vec<_>>()
            .join("|")
            .as_str(),
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use vitaly::keycodes::bitmod_to_name;

    use super::*;

    #[test]
    fn mods_to_mask_test() {
        assert_eq!(
            bitmod_to_name(mods_to_mask(&[Key::LeftCtrl].to_vec()).unwrap()),
            "MOD_BIT_LCTRL"
        );
        assert_eq!(
            bitmod_to_name(mods_to_mask(&[Key::LeftCtrl, Key::RightMeta].to_vec()).unwrap()),
            "MOD_BIT_LCTRL|MOD_BIT_RGUI"
        );
    }
}
