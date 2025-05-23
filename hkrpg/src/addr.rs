use std::sync::{LazyLock, OnceLock};

use windows::{Win32::System::LibraryLoader::GetModuleHandleA, core::s};

use crate::util::scan_il2cpp_section;

const IL2CPP_STRING_NEW_LEN: &str = "E8 ? ? ? ? EB ? 31 C0 48 89 06 48 8B 47 ? 48 89 46 ? F2 0F 10 47";
const MAKE_INITIAL_URL: &str = "E8 ? ? ? ? 48 89 D9 48 89 C2 E8 ? ? ? ? 48 89 D9 4C 89 FA E8 ? ? ? ? 49 89 5D"; // TODO
const SET_ELEVATION_DITHER: &str = "56 48 83 EC ? 0F 29 74 24 ? 0F 28 F1 48 89 CE 80 3D ? ? ? ? ? 75 ? 80 7E ? ? 74 ? 0F 57 C0 F3 0F 5F C6 F3 0F 10 0D ? ? ? ? F3 0F 5D C8 F3 0F 11 4E ? F3 0F 59 4E ? 48 89 F1 41 B8 ? ? ? ? E8 ? ? ? ? 84 C0 75 ? C7 46 ? ? ? ? ? 0F 28 74 24 ? 48 83 C4 ? 5E C3 B9 ? ? ? ? E8 ? ? ? ? 48 85 C0 74 ? 48 89 C1 48 89 F2 0F 28 D6 0F 28 74 24 ? 48 83 C4 ? 5E E9 ? ? ? ? E8 ? ? ? ? CC 0F 1F 00 56 57 53"; // TODO
const SET_DISTANCE_DITHER: &str = "E8 ? ? ? ? 49 8B 46 ? 48 85 C0 0F 84 ? ? ? ? 48 8B 4D"; // TODO
const SET_DITHER_ALPHA: &str = "E8 ? ? ? ? 0F 28 74 24 ? 48 83 C4 ? 5B 5F 5E C3 B9 ? ? ? ? E8 ? ? ? ? 48 85 C0 0F 84 ? ? ? ? 48 89 C1 48 89 F2 E8"; // TODO
const SET_DITHER_ALPHA_ANIM: &str = "E8 ? ? ? ? 8B 46 ? 0F 57 C0 0F 2E C6 0F 82"; // TODO
const SDK_PUBLIC_KEY_LITERAL: &str = "48 8B 0D ? ? ? ? 4C 89 E2 E8 ? ? ? ? 48 89 C6 48 8B 0D ? ? ? ? E8 ? ? ? ? 48 89 C7 48 8B 0D";
// const HK_CHECK1: &str = "55 41 56 56 57 53 48 81 EC 00 01 00 00 48 8D AC 24 80 00 00 00 C7 45 7C 00 00 00 00";
// const HK_CHECK2: &str = "55 41 57 41 56 41 55 41 54 56 57 53 48 81 EC B8 02 00 00";

#[derive(Default)]
pub struct RVAConfig {
    pub il2cpp_string_new_len: usize,
    pub make_initial_url: usize,
    pub set_elevation_dither: usize,
    pub set_distance_dither: usize,
    pub set_dither_alpha: usize,
    pub set_dither_alpha_anim: usize,
    pub hk_check1: usize,
    pub hk_check2: usize,
    pub sdk_public_key: usize,
}

#[allow(static_mut_refs)]
pub fn rva_config() -> &'static mut RVAConfig {
    static mut RVA_CONFIG: OnceLock<RVAConfig> = OnceLock::new();
    unsafe { RVA_CONFIG.get_mut_or_init(RVAConfig::default) }
}

pub static GAME_ASSEMBLY_BASE: LazyLock<usize> =
    LazyLock::new(|| unsafe { GetModuleHandleA(s!("GameAssembly.dll")).unwrap().0 as usize });

// pub static UNITY_PLAYER_BASE: LazyLock<usize> =
//     LazyLock::new(|| unsafe { GetModuleHandleA(s!("UnityPlayer.dll")).unwrap().0 as usize });

macro_rules! set_rva {
    ($base:ident, $config:ident, $field:ident, $scan_fn:ident, $rva_pat:expr, $fallback:expr) => {
        if let Some(addr) = unsafe { $scan_fn($rva_pat) } {
            $config.$field = addr - *$base;
            println!(
                "[hkrpg::addr::set_rva] Found relative address for {} [{}] -> 0x{:X}",
                stringify!($field),
                stringify!($base),
                $config.$field
            );
        } else {
            eprintln!(
                "[hkrpg::addr::set_rva] Failed to find pattern for {} [{}] using {}",
                stringify!($field),
                stringify!($base),
                stringify!($scan_fn)
            );

            $config.$field = $fallback
        }
    };
}

pub unsafe fn init_rvas() {
    let config = rva_config();

    // il2cpp_string_new_len
    set_rva!(
        GAME_ASSEMBLY_BASE,
        config,
        il2cpp_string_new_len,
        scan_il2cpp_section,
        IL2CPP_STRING_NEW_LEN,
        0x0
    );

    // make_initial_url
    set_rva!(
        GAME_ASSEMBLY_BASE,
        config,
        make_initial_url,
        scan_il2cpp_section,
        MAKE_INITIAL_URL,
        0x0
    );

    // set_elevation_dither
    set_rva!(
        GAME_ASSEMBLY_BASE,
        config,
        set_elevation_dither,
        scan_il2cpp_section,
        SET_ELEVATION_DITHER,
        0x0
    );
    // set_distance_dither
    set_rva!(
        GAME_ASSEMBLY_BASE,
        config,
        set_distance_dither,
        scan_il2cpp_section,
        SET_DISTANCE_DITHER,
        0x0
    );
    // set_dither_alpha
    set_rva!(
        GAME_ASSEMBLY_BASE,
        config,
        set_dither_alpha,
        scan_il2cpp_section,
        SET_DITHER_ALPHA,
        0x0
    );
    // set_dither_alpha_anim
    set_rva!(
        GAME_ASSEMBLY_BASE,
        config,
        set_dither_alpha_anim,
        scan_il2cpp_section,
        SET_DITHER_ALPHA_ANIM,
        0x0
    );

    // sdk_public_key_literal
    set_rva!(
        GAME_ASSEMBLY_BASE,
        config,
        sdk_public_key,
        scan_il2cpp_section,
        SDK_PUBLIC_KEY_LITERAL,
        0x0
    )

    // set_rva!(
    //     UNITY_PLAYER_BASE,
    //     config,
    //     hk_check1,
    //     scan_unity_player_section,
    //     HK_CHECK1,
    //     0x0
    // );
    // set_rva!(
    //     UNITY_PLAYER_BASE,
    //     config,
    //     hk_check2,
    //     scan_unity_player_section,
    //     HK_CHECK2,
    //     0x0
    // );
}
