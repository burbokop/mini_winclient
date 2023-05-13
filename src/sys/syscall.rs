use core::{arch::global_asm, ffi::c_void};


#[cfg(target_arch = "x86")]
pub mod nums {
    pub static SYS_READ         : usize = 3;
    pub static SYS_WRITE        : usize = 4;
    pub static SYS_CLOSE        : usize = 6;
    pub static SYS_EXIT         : usize = 1;
    pub static SYS_SOCKETCALL   : usize = 102;
}

#[cfg(target_arch = "x86_64")]
pub mod nums {
    pub static SYS_READ         : usize = 0;
    pub static SYS_WRITE        : usize = 1;
    pub static SYS_CLOSE        : usize = 3;
    pub static SYS_SOCKET       : usize = 41;
    pub static SYS_CONNECT      : usize = 42;
    pub static SYS_GETTIMEOFDAY : usize = 96;
}

#[cfg(target_arch = "x86")]
global_asm!(include_str!("x86/syscall.s"));

#[cfg(target_arch = "x86_64")]
global_asm!(include_str!("x86_64/syscall.s"));

extern "C" {
    pub fn __mini_wc_syscall1__(
        number: usize,
        arg1: *mut c_void
    ) -> *mut c_void;

    pub fn __mini_wc_syscall2__(
        number: usize,
        arg1: *mut c_void,
        arg2: *mut c_void
    ) -> *mut c_void;

    pub fn __mini_wc_syscall3__(
        number: usize,
        arg1: *mut c_void,
        arg2: *mut c_void,
        arg3: *mut c_void
    ) -> *mut c_void;
}
