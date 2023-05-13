use core::{fmt::{Write, self}, ffi::c_void};

use crate::sys::io_sys;



pub struct WriteFd(io_sys::Fd);

impl WriteFd {
    pub fn new(fd: io_sys::Fd) -> Self {
        return Self(fd)
    }
}

impl Write for WriteFd {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        match unsafe {
            io_sys::write(self.0, s.as_ptr() as *const c_void, s.len())
        } {
            Ok(_) => Ok(()),
            Err(_) => Err(fmt::Error),
        }
    }
}
