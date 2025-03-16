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

use crate::addr::GAME_ASSEMBLY_BASE;

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

pub unsafe fn scan_il2cpp_section(pat: &str) -> Option<usize> {
    let mut slice = unsafe { game_assembly_slice() };
    scan_first_match(&mut slice, pat).unwrap().map(|address| {
        let slice = unsafe { game_assembly_slice() };
        match slice.get(address) {
            // jmp sub_xxxxxxx
            Some(&0xE8) => {
                let offset = i32::from_le_bytes(slice[address + 1..address + 5].try_into().unwrap());
                GAME_ASSEMBLY_BASE.wrapping_add(address + 5 + offset as usize)
            }
            // mov rcx, [rip + offset] (0x48 0x8B 0x0D XXXXXXXX)
            Some(&0x48)
                if slice.get(address + 1) == Some(&0x8B)
                    && slice.get(address + 2) == Some(&0x0D) =>
            {
                let offset = i32::from_le_bytes(slice[address + 3..address + 7].try_into().unwrap());
                GAME_ASSEMBLY_BASE.wrapping_add(address + 7 + offset as usize)
            }
            _ => GAME_ASSEMBLY_BASE.wrapping_add(address),
        }
    })
}

// #[allow(static_mut_refs)]
// unsafe fn unity_player_slice() -> &'static [u8] {
//     static mut SLICE: OnceCell<&[u8]> = OnceCell::new();
//     unsafe {
//         SLICE.get_or_init(|| {
//             let module = GetModuleHandleA(s!("UnityPlayer.dll")).unwrap();
//             let mut module_info = MODULEINFO {
//                 lpBaseOfDll: std::ptr::null_mut(),
//                 SizeOfImage: 0,
//                 EntryPoint: std::ptr::null_mut(),
//             };
//             GetModuleInformation(
//                 GetCurrentProcess(),
//                 module,
//                 &mut module_info,
//                 std::mem::size_of::<MODULEINFO>() as u32,
//             )
//             .unwrap();
//             std::slice::from_raw_parts(
//                 module.0 as *const u8,
//                 module_info.SizeOfImage.try_into().unwrap(),
//             )
//         })
//     }
// }

// pub unsafe fn scan_unity_player_section(pat: &str) -> Option<usize> {
//     let mut slice = unsafe { unity_player_slice() };
//     scan_first_match(&mut slice, pat)
//         .unwrap()
//         .map(|loc| UNITY_PLAYER_BASE.wrapping_add(loc))
// }
