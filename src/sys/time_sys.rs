use core::{
    ffi::c_void,
    ops::Sub,
    time::Duration
};

use super::syscall::{
    nums::SYS_GETTIMEOFDAY,
    __mini_wc_syscall2__
};

#[cfg(target_arch = "x86_64")]
pub mod clib_types {
    use core::ffi::c_long;

    pub type CTimeT = c_long;
    pub type CSusecondsT = c_long;
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct CTimeval {
    /// seconds
    pub tv_sec: clib_types::CTimeT,
    /// microseconds
    pub tv_usec: clib_types::CSusecondsT,
}

impl Sub for CTimeval {
    type Output = Option<Duration>;

    #[inline]
    fn sub(self, mut rhs: Self) -> Self::Output {
        let mut result = CTimeval::default();
        if self.tv_usec < rhs.tv_usec {
            let nsec = (rhs.tv_usec - self.tv_usec) / 1000000 + 1;
            rhs.tv_usec -= 1000000 * nsec;
            rhs.tv_sec += nsec;
        }
        if self.tv_usec - rhs.tv_usec > 1000000 {
            let nsec = (self.tv_usec - rhs.tv_usec) / 1000000;
            rhs.tv_usec += 1000000 * nsec;
            rhs.tv_sec -= nsec;
        }

        result.tv_sec = self.tv_sec - rhs.tv_sec;
        result.tv_usec = self.tv_usec - rhs.tv_usec;

        if self.tv_sec > rhs.tv_sec || (self.tv_sec == rhs.tv_sec && self.tv_usec >= rhs.tv_usec) {
            Some(Duration::from_secs(result.tv_sec as u64) + Duration::from_micros(result.tv_usec as u64))
        } else {
            None
        }
    }
}

pub unsafe fn vdso_fallback_gtod(tv: *mut CTimeval, tz: *mut c_void) -> *mut c_void {
    __mini_wc_syscall2__(SYS_GETTIMEOFDAY, tv as *mut c_void, tz)
}
