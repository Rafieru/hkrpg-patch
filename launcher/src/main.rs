// Source: https://git.xeondev.com/NewEriduPubSec/JaneDoe-Patch/src/branch/master/launcher/src/main.rs

use std::ffi::CString;
use std::process::Child;
use std::fs;
use serde::{Deserialize, Serialize};

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
use windows::core::{PCSTR, PSTR, s};
use windows::Win32::UI::Controls::Dialogs::{GetOpenFileNameA, OPENFILENAMEA};
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize, Deserialize)]
struct Config {
    starrail_path: Option<String>,
}

impl Config {
    fn get_config_path() -> PathBuf {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
            .join("launcher-config.json")
    }

    fn load() -> Self {
        match fs::read_to_string(Self::get_config_path()) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or(Config { starrail_path: None }),
            Err(_) => Config { starrail_path: None },
        }
    }

    fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(Self::get_config_path(), json)
    }
}

fn get_starrail_path() -> Option<PathBuf> {
    unsafe {
        let mut buffer = [0u8; 260];
        let mut ofn = OPENFILENAMEA::default();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEA>() as u32;
        ofn.lpstrFile = PSTR(buffer.as_mut_ptr());
        ofn.nMaxFile = buffer.len() as u32;
        ofn.lpstrFilter = s!("Executable\0StarRail.exe\0All Files\0*.*\0\0");
        ofn.lpstrTitle = s!("Select StarRail.exe location");
        ofn.Flags = windows::Win32::UI::Controls::Dialogs::OPEN_FILENAME_FLAGS(0x00000800); // OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST

        if GetOpenFileNameA(&mut ofn).as_bool() {
            let path = String::from_utf8_lossy(&buffer)
                .trim_matches(char::from(0))
                .to_string();
            Some(PathBuf::from(path))
        } else {
            None
        }
    }
}

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

// Add a struct to store server processes
struct ServerProcesses {
    game_server: Option<Child>,
    sdk_server: Option<Child>,
}

fn run_server(exe_name: &str) -> Option<Child> {
    let current_dir = std::env::current_dir().unwrap();
    let server_path = current_dir.join(exe_name);
    
    if !server_path.exists() {
        // If looking for game/sdk servers and they don't exist, try server.exe
        if exe_name == "gameserver.exe" || exe_name == "sdkserver.exe" {
            static mut SERVER_STARTED: bool = false;
            unsafe {
                if !SERVER_STARTED {
                    let alt_server = current_dir.join("server.exe");
                    if alt_server.exists() {
                        println!("Using server.exe instead of gameserver.exe and sdkserver.exe");
                        match Command::new("server.exe").spawn() {
                            Ok(child) => {
                                println!("Starting RobinSR Server");
                                SERVER_STARTED = true;
                                return Some(child);
                            }
                            Err(e) => {
                                println!("Failed to start server.exe: {}", e);
                                return None;
                            }
                        }
                    } else {
                        println!("No server executables found (tried server.exe, gameserver.exe and sdkserver.exe)");
                        return None;
                    }
                }
                return None;
            }
        }
        println!("{} not found", exe_name);
        return None;
    }

    match Command::new(exe_name).spawn() {
        Ok(child) => {
            println!("Started {}", exe_name);
            Some(child)
        }
        Err(e) => {
            println!("Failed to start {}: {}", exe_name, e);
            None
        }
    }
}

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let mut config = Config::load();

    // Start servers first and store their processes
    let server_processes = ServerProcesses {
        game_server: run_server("gameserver.exe"),
        sdk_server: run_server("sdkserver.exe"),
    };

    if server_processes.game_server.is_none() && server_processes.sdk_server.is_none() {
        println!("Required server executables are missing");
        println!("Press Enter to exit...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        return;
    }
    
    // Check for required hkrpg.dll
    let hkrpg_path = current_dir.join("robinsr.dll");
    if !hkrpg_path.is_file() {
        println!("robinsr.dll not found");
        return;
    }

    let mut proc_info = PROCESS_INFORMATION::default();
    let startup_info = STARTUPINFOA::default();

    unsafe {
        let starrail_path = if let Some(saved_path) = &config.starrail_path {
            let path = PathBuf::from(saved_path);
            if path.exists() {
                path
            } else {
                match get_starrail_path() {
                    Some(path) => {
                        config.starrail_path = Some(path.to_string_lossy().into_owned());
                        let _ = config.save(); // Save the new path
                        path
                    }
                    None => {
                        println!("StarRail.exe location not selected");
                        return;
                    }
                }
            }
        } else {
            match get_starrail_path() {
                Some(path) => {
                    config.starrail_path = Some(path.to_string_lossy().into_owned());
                    let _ = config.save(); // Save the new path
                    path
                }
                None => {
                    println!("StarRail.exe location not selected");
                    return;
                }
            }
        };

        let path_str = CString::new(starrail_path.to_str().unwrap()).unwrap();
        CreateProcessA(
            PCSTR(path_str.as_ptr() as *const u8),
            Some(PSTR(null_mut())),  // Wrapped in Some
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

        // Inject required hkrpg.dll
        if inject_standard(proc_info.hProcess, hkrpg_path.to_str().unwrap()) {
            println!("Injected robinsr.dll successfully");
            
            // Try to inject optional veritas.dll if it exists
            let veritas_path = current_dir.join("veritas.dll");
            if veritas_path.is_file() {
                if inject_standard(proc_info.hProcess, veritas_path.to_str().unwrap()) {
                    println!("Injected veritas.dll successfully");
                } else {
                    println!("Failed to inject veritas.dll");
                }
            }

            ResumeThread(proc_info.hThread);
            
            // Wait for game process to exit
            WaitForSingleObject(proc_info.hProcess, 0xFFFFFFFF);

            // Kill server processes
            if let Some(mut game_server) = server_processes.game_server {
                let _ = game_server.kill();
            }
            if let Some(mut sdk_server) = server_processes.sdk_server {
                let _ = sdk_server.kill();
            }
        }

        CloseHandle(proc_info.hThread).unwrap();
        CloseHandle(proc_info.hProcess).unwrap();
    }
}
