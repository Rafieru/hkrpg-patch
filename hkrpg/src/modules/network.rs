use ilhook::x64::Registers;

use crate::{addr::rva_config, il2cpp_string::Il2cppString};

use super::{HkrpgModule, HkrpgModuleContext};

pub struct Network;

impl HkrpgModule for HkrpgModuleContext<Network> {
    unsafe fn init(&mut self) -> Result<(), ilhook::HookError> {
        let config = rva_config();
        if config.make_initial_url != 0 && config.ptr_to_string_ansi != 0 {
            self.interceptor.attach(
                self.base.wrapping_add(config.make_initial_url),
                Network::on_make_initial_url,
            )?;
        } else {
            println!("[network::init] pattern is outdated! disabling http redirection")
        }
        Ok(())
    }
}

impl Network {
    const SDK_URL: &str = "http://127.0.0.1:21000";
    // const DISPATCH_URL: &str = "http://127.0.0.1:21000";
    const REDIRECT_SDK: bool = true;
    const REDIRECT_DISPATCH: bool = true;

    extern "win64" fn on_make_initial_url(reg: *mut Registers, _: usize) {
        let url = unsafe { Il2cppString::from((*reg).rcx).to_string() };

        let mut new_url = match &url {
            s if (s.contains("mihoyo.com") || s.contains("hoyoverse.com"))
                && Self::REDIRECT_SDK =>
            {
                Self::SDK_URL.to_string()
            }
            s if (s.contains("bhsr.com") || s.contains("starrails.com"))
                && Self::REDIRECT_DISPATCH =>
            {
                Self::SDK_URL.to_string()
            }
            s => {
                println!("Leaving request as-is: {s}");
                return;
            }
        };

        url.split('/').skip(3).for_each(|s| {
            new_url.push('/');
            new_url.push_str(s);
        });

        println!("UnityWebRequest: \"{url}\", replacing with \"{new_url}\"");
        unsafe {
            (*reg).rcx = Il2cppString::new(&new_url).raw() as u64;
        }
    }
}
