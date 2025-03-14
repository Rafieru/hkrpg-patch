use std::marker::PhantomData;

use crate::{addr, interceptor::Interceptor};

pub mod censorship_patch;
pub mod hk_check;
pub mod network;

pub struct HkrpgModuleContext<T> {
    base: usize,
    interceptor: Interceptor,
    _module_type: PhantomData<T>,
}

impl<T> HkrpgModuleContext<T> {
    fn new(base: usize) -> Self {
        Self {
            base,
            interceptor: Interceptor::default(),
            _module_type: PhantomData,
        }
    }
}

pub trait HkrpgModule {
    unsafe fn init(&mut self) -> Result<(), ilhook::HookError>;
}

#[derive(Default)]
pub struct HkrpgModuleManager {
    modules: Vec<Box<dyn HkrpgModule>>,
}

impl HkrpgModuleManager {
    pub fn add<T: 'static>(&mut self)
    where
        HkrpgModuleContext<T>: HkrpgModule,
    {
        self.modules.push(Box::new(HkrpgModuleContext::<T>::new(
            *addr::GAME_ASSEMBLY_BASE,
        )));
    }

    pub unsafe fn init(&mut self) -> Result<(), ilhook::HookError> {
        for module in self.modules.iter_mut() {
            unsafe { module.init() }?;
        }

        Ok(())
    }
}
