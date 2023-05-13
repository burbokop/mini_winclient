use core::{ptr::null_mut, time::Duration, ops::Sub};

use crate::sys::time_sys::{
    CTimeval,
    vdso_fallback_gtod
};

#[derive(Debug, Default, Clone)]
pub struct Point(CTimeval);

impl Point {

    #[inline]
    pub fn now() -> Self {
        let mut tv = CTimeval::default();
        unsafe { vdso_fallback_gtod(&mut tv, null_mut()) };
        Self(tv)
    }

    #[inline]
    pub fn epoch() -> Self {
        Self(CTimeval { tv_sec: 0, tv_usec: 0 })
    }

    #[inline]
    pub fn elapsed(&self) -> Option<Duration> {
        Self::now() - self.clone()
    }

    #[inline]
    pub fn loop_for(self, for_dur: Duration) -> bool {
        loop {
            if let Some(e) = self.elapsed() {
                if e >= for_dur { return true }
            } else {
                return false;
            }
        }
    }
}

impl Sub for Point {
    type Output = Option<Duration>;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}
