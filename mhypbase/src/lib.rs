use windows::Win32::{
    Foundation::HINSTANCE,
    System::{
        SystemServices::DLL_PROCESS_ATTACH,
        Threading::{GetCurrentThread, TerminateThread},
    },
};

#[unsafe(no_mangle)]
#[allow(non_snake_case, unused_variables)]
extern "cdecl" fn Initialize() -> bool {
    std::thread::sleep(std::time::Duration::from_secs(2));

    unsafe {
        TerminateThread(GetCurrentThread(), 0).unwrap();
    }

    false
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(_: HINSTANCE, call_reason: u32, _: *mut ()) {
    if call_reason == DLL_PROCESS_ATTACH {
        std::thread::spawn(hkrpg::main);
    }
}
