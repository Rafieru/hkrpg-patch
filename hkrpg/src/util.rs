use std::cell::OnceCell;

use patternscan::scan_first_match;
use windows::{
    Win32::System::{
        LibraryLoader::GetModuleHandleA,
        ProcessStatus::{GetModuleInformation, MODULEINFO},
        Threading::GetCurrentProcess,
    },
    core::s,
};

use crate::addr::{GAME_ASSEMBLY_BASE, UNITY_PLAYER_BASE};

#[allow(static_mut_refs)]
unsafe fn game_assembly_slice() -> &'static [u8] {
    static mut SLICE: OnceCell<&[u8]> = OnceCell::new();
    unsafe {
        SLICE.get_or_init(|| {
            let module = GetModuleHandleA(s!("GameAssembly.dll")).unwrap();
            let mut module_info = MODULEINFO {
                lpBaseOfDll: std::ptr::null_mut(),
                SizeOfImage: 0,
                EntryPoint: std::ptr::null_mut(),
            };
            GetModuleInformation(
                GetCurrentProcess(),
                module,
                &mut module_info,
                std::mem::size_of::<MODULEINFO>() as u32,
            )
            .unwrap();
            std::slice::from_raw_parts(
                module.0 as *const u8,
                module_info.SizeOfImage.try_into().unwrap(),
            )
        })
    }
}

#[allow(static_mut_refs)]
unsafe fn unity_player_slice() -> &'static [u8] {
    static mut SLICE: OnceCell<&[u8]> = OnceCell::new();
    unsafe {
        SLICE.get_or_init(|| {
            let module = GetModuleHandleA(s!("UnityPlayer.dll")).unwrap();
            let mut module_info = MODULEINFO {
                lpBaseOfDll: std::ptr::null_mut(),
                SizeOfImage: 0,
                EntryPoint: std::ptr::null_mut(),
            };
            GetModuleInformation(
                GetCurrentProcess(),
                module,
                &mut module_info,
                std::mem::size_of::<MODULEINFO>() as u32,
            )
            .unwrap();
            std::slice::from_raw_parts(
                module.0 as *const u8,
                module_info.SizeOfImage.try_into().unwrap(),
            )
        })
    }
}

pub unsafe fn scan_il2cpp_section(pat: &str) -> Option<usize> {
    let mut slice = unsafe { game_assembly_slice() };
    scan_first_match(&mut slice, pat).unwrap().map(|address| {
        let slice = unsafe { game_assembly_slice() };
        let instruction = &slice[address..address + 1][0];
        if *instruction == 0xE8 {
            let offset = i32::from_le_bytes((&slice[address + 1..address + 1 + 4]).try_into().unwrap());
            let pointer = offset as usize + 5 + address;
            return GAME_ASSEMBLY_BASE.wrapping_add(pointer);
        }
        GAME_ASSEMBLY_BASE.wrapping_add(address)
    })
}

pub unsafe fn scan_unity_player_section(pat: &str) -> Option<usize> {
    let mut slice = unsafe { unity_player_slice() };
    scan_first_match(&mut slice, pat)
        .unwrap()
        .map(|loc| UNITY_PLAYER_BASE.wrapping_add(loc))
}
