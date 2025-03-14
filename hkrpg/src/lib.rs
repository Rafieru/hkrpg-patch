#![feature(str_from_utf16_endian, once_cell_get_mut)]

use std::{thread, time::Duration};

use modules::{
    HkrpgModuleManager, censorship_patch::CensorshipPatch, hk_check::HkCheck, network::Network,
};
use windows::{
    Win32::System::{Console, LibraryLoader::GetModuleHandleA},
    core::s,
};

mod addr;
mod il2cpp_string;
mod interceptor;
mod modules;
mod util;

pub fn main() {
    unsafe {
        let _ = Console::AllocConsole();

        println!("[hkrpg::main] init");

        while GetModuleHandleA(s!("GameAssembly.dll")).is_err() {
            thread::sleep(Duration::from_millis(200));
        }

        addr::init_rvas();

        let mut module_manager = HkrpgModuleManager::default();
        module_manager.add::<HkCheck>();
        module_manager.add::<Network>();
        module_manager.add::<CensorshipPatch>();
        module_manager
            .init()
            .expect("[hkrpg::main] failed to initialize modules");

        thread::sleep(Duration::from_secs(u64::MAX));
    }
}
