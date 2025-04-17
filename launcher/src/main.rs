use std::process::Child;
use std::fs;
use serde::{Deserialize, Serialize};
use std::io::Write;

use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx,
};
use windows::Win32::System::Threading::{
    CREATE_SUSPENDED, CreateProcessW, CreateRemoteThread, PROCESS_INFORMATION, ResumeThread,
    STARTUPINFOW, WaitForSingleObject,
};
use windows::core::{PCWSTR, PWSTR, w, s};
use windows::Win32::UI::Controls::Dialogs::{GetOpenFileNameW, OPENFILENAMEW};
use std::path::PathBuf;
use std::process::Command;
use std::os::windows::ffi::OsStrExt;

// Add resource handling
#[cfg(windows)]
extern crate winres;

// Embedded binaries - only included if they exist
#[cfg(has_robinsr_dll)]
const ROBINSR_DLL: &[u8] = include_bytes!("../robinsr.dll");
#[cfg(has_gameserver_exe)]
const GAMESERVER_EXE: &[u8] = include_bytes!("../gameserver.exe");
#[cfg(has_sdkserver_exe)]
const SDKSERVER_EXE: &[u8] = include_bytes!("../sdkserver.exe");

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
            .join("launcher.toml")
    }

    fn load() -> Self {
        match fs::read_to_string(Self::get_config_path()) {
            Ok(contents) => toml::from_str(&contents).unwrap_or(Config { starrail_path: None }),
            Err(_) => Config { starrail_path: None },
        }
    }

    fn save(&self) -> std::io::Result<()> {
        let toml = toml::to_string_pretty(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(Self::get_config_path(), toml)
    }
}

fn get_starrail_path() -> Option<PathBuf> {
    unsafe {
        let mut buffer = [0u16; 260];
        let mut ofn = OPENFILENAMEW::default();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.lpstrFile = PWSTR(buffer.as_mut_ptr());
        ofn.nMaxFile = buffer.len() as u32;
        ofn.lpstrFilter = w!("Executable\0StarRail.exe\0All Files\0*.*\0\0");
        ofn.lpstrTitle = w!("Select StarRail.exe location");
        ofn.Flags = windows::Win32::UI::Controls::Dialogs::OPEN_FILENAME_FLAGS(0x00000800); // OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST

        if GetOpenFileNameW(&mut ofn).as_bool() {
            let path = String::from_utf16_lossy(&buffer)
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
            GetModuleHandleW(w!("kernel32.dll")).unwrap(),
            s!("LoadLibraryW"),
        )
        .unwrap();

        // Convert DLL path to wide string
        let dll_path_wide: Vec<u16> = std::ffi::OsStr::new(dll_path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let dll_path_addr = VirtualAllocEx(
            h_target,
            None,
            dll_path_wide.len() * std::mem::size_of::<u16>(),
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
            dll_path_wide.as_ptr() as _,
            dll_path_wide.len() * std::mem::size_of::<u16>(),
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

fn extract_embedded_binary(name: &str, data: &[u8]) -> std::io::Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join("RobinSR");
    fs::create_dir_all(&temp_dir)?;
    
    let path = temp_dir.join(name);
    if !path.exists() {
        let mut file = fs::File::create(&path)?;
        file.write_all(data)?;
    }
    Ok(path)
}

fn run_server(exe_name: &str) -> Option<Child> {
    #[cfg(has_gameserver_exe)]
    if exe_name == "gameserver.exe" {
        if let Ok(path) = extract_embedded_binary("gameserver.exe", GAMESERVER_EXE) {
            if let Ok(child) = Command::new(path).spawn() {
                println!("Started gameserver.exe from embedded binary");
                return Some(child);
            }
        }
    }

    #[cfg(has_sdkserver_exe)]
    if exe_name == "sdkserver.exe" {
        if let Ok(path) = extract_embedded_binary("sdkserver.exe", SDKSERVER_EXE) {
            if let Ok(child) = Command::new(path).spawn() {
                println!("Started sdkserver.exe from embedded binary");
                return Some(child);
            }
        }
    }

    // Try running from current directory if embedded version failed or doesn't exist
    let current_dir = std::env::current_dir().unwrap();
    let server_path = current_dir.join(exe_name);
    if server_path.exists() {
        match Command::new(exe_name).spawn() {
            Ok(child) => {
                println!("Started {} from current directory", exe_name);
                return Some(child);
            }
            Err(e) => {
                println!("Failed to start {}: {}", exe_name, e);
            }
        }
    }

    println!("{} not found, skipping...", exe_name);
    None
}

fn main() {
    let current_dir = std::env::current_dir().unwrap();
    let mut config = Config::load();

    // Start servers first and store their processes
    let server_processes = ServerProcesses {
        game_server: run_server("gameserver.exe"),
        sdk_server: run_server("sdkserver.exe"),
    };

    // Try to extract robinsr.dll if it's embedded
    let hkrpg_path = {
        #[cfg(has_robinsr_dll)]
        {
            match extract_embedded_binary("robinsr.dll", ROBINSR_DLL) {
                Ok(path) => Some(path),
                Err(e) => {
                    println!("Failed to extract robinsr.dll: {}", e);
                    None
                }
            }
        }
        #[cfg(not(has_robinsr_dll))]
        {
            println!("robinsr.dll not found in embedded resources");
            None
        }
    };

    let mut proc_info = PROCESS_INFORMATION::default();
    let startup_info = STARTUPINFOW::default();

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

        // Convert path to wide string for CreateProcessW
        let path_wide: Vec<u16> = starrail_path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        CreateProcessW(
            PCWSTR(path_wide.as_ptr()),
            None,
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

        // Inject robinsr.dll if available
        if let Some(dll_path) = hkrpg_path {
            if inject_standard(proc_info.hProcess, dll_path.to_str().unwrap()) {
                println!("Injected robinsr.dll successfully");
            } else {
                println!("Failed to inject robinsr.dll");
            }
        }
        
        // Try to inject optional veritas.dll if it exists in the current directory
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

        CloseHandle(proc_info.hThread).unwrap();
        CloseHandle(proc_info.hProcess).unwrap();
    }
}
