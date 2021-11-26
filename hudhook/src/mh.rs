#![allow(dead_code, non_snake_case, non_camel_case_types)]

use std::{ffi::c_void, ptr::null_mut};

use log::*;

pub use winapi::shared::{
    minwindef::LPVOID,
    ntdef::{LPCSTR, LPCWSTR},
};

#[must_use]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MH_STATUS {
    /// Unknown error. Should not be returned.
    MH_UNKNOWN = -1,
    /// Successful.
    MH_OK = 0,
    /// MinHook is already initialized.
    MH_ERROR_ALREADY_INITIALIZED,
    /// MinHook is not initialized yet, or already uninitialized.
    MH_ERROR_NOT_INITIALIZED,
    /// The hook for the specified target function is already created.
    MH_ERROR_ALREADY_CREATED,
    /// The hook for the specified target function is not created yet.
    MH_ERROR_NOT_CREATED,
    /// The hook for the specified target function is already enabled.
    MH_ERROR_ENABLED,
    /// The hook for the specified target function is not enabled yet, or
    /// already disabled.
    MH_ERROR_DISABLED,
    /// The specified pointer is invalid. It points the address of non-allocated
    /// and/or non-executable region.
    MH_ERROR_NOT_EXECUTABLE,
    /// The specified target function cannot be hooked.
    MH_ERROR_UNSUPPORTED_FUNCTION,
    /// Failed to allocate memory.
    MH_ERROR_MEMORY_ALLOC,
    /// Failed to change the memory protection.
    MH_ERROR_MEMORY_PROTECT,
    /// The specified module is not loaded.
    MH_ERROR_MODULE_NOT_FOUND,
    /// The specified function is not found.
    MH_ERROR_FUNCTION_NOT_FOUND,
}

extern "system" {
    pub fn MH_Initialize() -> MH_STATUS;
    pub fn MH_Uninitialize() -> MH_STATUS;
    pub fn MH_CreateHook(pTarget: LPVOID, pDetour: LPVOID, ppOriginal: *mut LPVOID) -> MH_STATUS;
    pub fn MH_EnableHook(pTarget: LPVOID) -> MH_STATUS;
    pub fn MH_QueueEnableHook(pTarget: LPVOID) -> MH_STATUS;
    pub fn MH_DisableHook(pTarget: LPVOID) -> MH_STATUS;
    pub fn MH_QueueDisableHook(pTarget: LPVOID) -> MH_STATUS;
    pub fn MH_ApplyQueued() -> MH_STATUS;
}

pub struct Hook {
    addr: *mut c_void,
    hook_impl: *mut c_void,
    trampoline: *mut c_void,
}

impl Hook {
    pub unsafe fn new(addr: *mut c_void, hook_impl: *mut c_void) -> Hook {
        Hook {
            addr,
            hook_impl,
            trampoline: null_mut(),
        }
    }

    pub fn trampoline(&self) -> *mut c_void {
        self.trampoline
    }

    unsafe fn queue_enable(&self) {
        let status = MH_QueueEnableHook(self.hook_impl);
        debug!("MH_QueueEnableHook: {:?}", status);
    }

    unsafe fn queue_disable(&self) {
        let status = MH_QueueDisableHook(self.hook_impl);
        debug!("MH_QueueDisableHook: {:?}", status);
    }

    pub unsafe fn apply_queue(hooks: &[&Hook]) {
        for hook in hooks {
            debug!("MH_QueueEnable: {:?}", MH_QueueEnableHook(hook.addr));
        }
        debug!("MH_ApplyQueued: {:?}", MH_ApplyQueued());
    }
}
