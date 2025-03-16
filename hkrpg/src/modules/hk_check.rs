use ilhook::x64::Registers;

use crate::addr::rva_config;

use super::{HkrpgModule, HkrpgModuleContext};

pub struct HkCheck;

impl HkrpgModule for HkrpgModuleContext<HkCheck> {
    unsafe fn init(&mut self) -> Result<(), ilhook::HookError> {
        let config = rva_config();
        if config.hk_check1 != 0 && config.hk_check2 != 0 {
            self.interceptor.replace(
                self.base.wrapping_add(config.hk_check1),
                HkCheck::replacement,
            )?;
            self.interceptor.replace(
                self.base.wrapping_add(config.hk_check2),
                HkCheck::replacement,
            )?;
            println!("[hk_check::init] hk_check bypassed")
        }
        Ok(())
    }
}

impl HkCheck {
    extern "win64" fn replacement(_: *mut Registers, _: usize, _: usize) -> usize {
        0
    }
}
