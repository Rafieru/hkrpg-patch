use crate::{
    addr::{GAME_ASSEMBLY_BASE, rva_config},
    il2cpp_string::Il2cppString,
};

use super::{HkrpgModule, HkrpgModuleContext};

pub struct Crypto;

const ACCOUNT_RSA_KEY_REPLACEMENT: &str = include_str!("../../sdk_public_key.xml");

impl HkrpgModule for HkrpgModuleContext<Crypto> {
    unsafe fn init(&mut self) -> Result<(), ilhook::HookError> {
        let config = rva_config();
        if config.sdk_public_key != 0 {
            unsafe {
                *(GAME_ASSEMBLY_BASE.wrapping_add(config.sdk_public_key) as *mut Il2cppString) =
                    Il2cppString::new(ACCOUNT_RSA_KEY_REPLACEMENT)
            }
            println!("[crypto::init] AccountRSAKey replaced")
        } else {
            println!("[crypto::init] pattern is outdated! disabling AccountRSAKey replacement")
        }
        Ok(())
    }
}
