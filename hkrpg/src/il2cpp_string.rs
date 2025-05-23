use std::{ffi::CString, fmt::Display};

use crate::addr::{GAME_ASSEMBLY_BASE, rva_config};

#[repr(transparent)]
pub struct Il2cppString(usize);

impl From<u64> for Il2cppString {
    fn from(value: u64) -> Self {
        Self(value as usize)
    }
}

impl Il2cppString {
    pub fn new(string: &str) -> Self {
        let func = unsafe {
            std::mem::transmute::<usize, fn(*const u8, usize) -> usize>(
                *GAME_ASSEMBLY_BASE + rva_config().il2cpp_string_new_len,
            )
        };
        let len = string.len();
        let string = CString::new(string).unwrap();
        let string = string.as_c_str();
        Self(func(string.to_bytes_with_nul().as_ptr(), len))
    }

    pub fn raw(&self) -> u64 {
        self.0 as u64
    }
}

impl Display for Il2cppString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str_length = unsafe { *(self.0.wrapping_add(16) as *const u32) };
        let str_ptr = self.0.wrapping_add(20) as *const u8;
        let slice = unsafe { std::slice::from_raw_parts(str_ptr, (str_length * 2) as usize) };
        write!(
            f,
            "{}",
            String::from_utf16le(slice).map_err(|_| std::fmt::Error)?
        )
    }
}
