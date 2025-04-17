use winres::WindowsResource;
use std::path::Path;

fn main() {
    // Declare our custom cfg flags
    println!("cargo:rustc-check-cfg=cfg(has_robinsr_dll)");
    println!("cargo:rustc-check-cfg=cfg(has_gameserver_exe)");
    println!("cargo:rustc-check-cfg=cfg(has_sdkserver_exe)");

    // Create Windows resource file
    let mut res = WindowsResource::new();
    res.set_icon("robin.ico");
    res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
            </requestedPrivileges>
        </security>
    </trustInfo>
</assembly>
"#);

    // Check for optional binaries and set build flags
    let binaries = ["robinsr.dll", "gameserver.exe", "sdkserver.exe"];
    for binary in binaries.iter() {
        if Path::new(binary).exists() {
            println!("cargo:rustc-cfg=has_{}", binary.replace(".", "_"));
        }
    }

    res.compile().unwrap();
}