// Source: https://git.xeondev.com/NewEriduPubSec/JaneDoe-Patch/src/branch/master/launcher/src/main.rs

use std::ffi::CString;

use std::ptr::null_mut;
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx,
};
use windows::Win32::System::Threading::{
    CREATE_SUSPENDED, CreateProcessA, CreateRemoteThread, PROCESS_INFORMATION, ResumeThread,
    STARTUPINFOA, WaitForSingleObject,
};
use windows::core::{PSTR, s};

fn inject_standard(h_target: HANDLE, dll_path: &str) -> bool {
    unsafe {
        let loadlib = GetProcAddress(
            GetModuleHandleA(s!("kernel32.dll")).unwrap(),
            s!("LoadLibraryA"),
        )
        .unwrap();

        let dll_path_cstr = CString::new(dll_path).unwrap();
        let dll_path_addr = VirtualAllocEx(
            h_target,
            None,
            dll_path_cstr.to_bytes_with_nul().len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );
        if dll_path_addr.is_null() {
            println!(
                "Failed allocating memory in the target process. GetLastError(): {:?}",
                GetLastError()
            );
            return false;
        }

        WriteProcessMemory(
            h_target,
            dll_path_addr,
            dll_path_cstr.as_ptr() as _,
            dll_path_cstr.to_bytes_with_nul().len(),
            None,
        )
        .unwrap();

        let h_thread = CreateRemoteThread(
            h_target,
            None,
            0,
            Some(std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
            >(loadlib)),
            Some(dll_path_addr),
            0,
            None,
        )
        .unwrap();

        WaitForSingleObject(h_thread, 0xFFFFFFFF);

        VirtualFreeEx(h_target, dll_path_addr, 0, MEM_RELEASE).unwrap();
        CloseHandle(h_thread).unwrap();
        true
    }
}

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let dll_path = current_dir.join("hkrpg.dll");
    if !dll_path.is_file() {
        println!("hkrpg.dll not found");
        return;
    }

    let mut proc_info = PROCESS_INFORMATION::default();
    let startup_info = STARTUPINFOA::default();

    unsafe {
        CreateProcessA(
            s!("StarRail.exe"),
            PSTR(null_mut()),
            None,
            None,
            false,
            CREATE_SUSPENDED,
            None,
            None,
            &startup_info,
            &mut proc_info,
        )
        .unwrap();

        if inject_standard(proc_info.hProcess, dll_path.to_str().unwrap()) {
            ResumeThread(proc_info.hThread);
        }

        CloseHandle(proc_info.hThread).unwrap();
        CloseHandle(proc_info.hProcess).unwrap();
    }
}
