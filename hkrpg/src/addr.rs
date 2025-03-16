use std::sync::{LazyLock, OnceLock};

use windows::{Win32::System::LibraryLoader::GetModuleHandleA, core::s};

use crate::util::scan_il2cpp_section;

const PTR_TO_STRING_ANSI: &str = "E8 ? ? ? ? 48 ? ? 48 85 C0 75 ? 48 8D 4C 24";
const MAKE_INITIAL_URL: &str = "55 41 56 56 57 53 48 83 EC ? 48 8D 6C 24 ? 48 C7 45 ? ? ? ? ? 48 89 D6 48 89 CF E8 ? ? ? ? 84 C0"; // TODO
const SET_ELEVATION_DITHER: &str = "E9 ? ? ? ? 0F 28 74 24 ? 48 83 C4 ? 5B 5F 5E 41 5E 41 5F C3 31 F6"; // TODO
const SET_DISTANCE_DITHER: &str = "E8 ? ? ? ? 49 8B 46 ? 48 85 C0 0F 84 ? ? ? ? 48 8B 4D"; // TODO
const SET_DITHER_ALPHA: &str = "56 57 48 83 EC ? 0F 29 74 24 ? 44 89 C6 0F 28 F1 48 89 CF 80 3D ? ? ? ? ? 75 ? 80 7F"; // TODO
const SET_DITHER_ALPHA_ANIM: &str = "56 57 55 53 48 83 EC ? 44 0F 29 44 24 ? 0F 29 7C 24 ? 0F 29 74 24 ? 44 0F 28 C3 0F 28 F2 0F 28 F9"; // TODO
const SDK_PUBLIC_KEY_LITERAL: &str = "48 8B 0D ? ? ? ? 4C 89 FA E8 ? ? ? ? 48 89 C1 E8 ? ? ? ? 48 8B 15 ? ? ? ? 48 89 F1";
// const HK_CHECK1: &str = "55 41 56 56 57 53 48 81 EC 00 01 00 00 48 8D AC 24 80 00 00 00 C7 45 7C 00 00 00 00";
// const HK_CHECK2: &str = "55 41 57 41 56 41 55 41 54 56 57 53 48 81 EC B8 02 00 00";

#[derive(Default)]
pub struct RVAConfig {
    pub ptr_to_string_ansi: usize,
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

    // ptr_to_string_ansi
    set_rva!(
        GAME_ASSEMBLY_BASE,
        config,
        ptr_to_string_ansi,
        scan_il2cpp_section,
        PTR_TO_STRING_ANSI,
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
