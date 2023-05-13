#![no_std]
#![feature(lang_items)]
#![no_main]

use core::{
    ffi::{
        c_int,
        c_void,
    },
    slice,
    time::Duration,
    arch::global_asm,
};

use mini_winclient::{
    time::Point,
    winclient::{
        Format,
        Client,
    },
    write::WriteFd,
    STDOUT,
    socket::LOCALHOST,
};

#[cfg(target_arch = "x86")]
global_asm!(include_str!("x86/start.s"));

#[cfg(target_arch = "x86_64")]
global_asm!(include_str!("x86_64/start.s"));

struct Canvas<'p> {
    w: usize,
    h: usize,
    pixels: &'p mut [u8]
}

impl<'p> Canvas<'p> {
    #[inline]
    fn pixel_unchecked(&mut self, x: usize, y: usize) -> &mut u8 {
        &mut self.pixels[x + y * self.w]
    }

    #[inline]
    fn pixel(&mut self, x: isize, y: isize) -> Option<&mut u8> {
        if x > 0 && y > 0 && x < self.w as isize && y < self.h as isize {
            Some(self.pixel_unchecked(x as usize, y as usize))
        } else {
            None
        }
    }

    fn draw_rect(&mut self, x0: isize, y0: isize, x1: isize, y1: isize, color: u8) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        if dx >= 0 {
            for i in 0..=dx {
                *self.pixel(x0 + i, y0).unwrap() = color;
                *self.pixel(x0 + i, y0 + dy).unwrap() = color;
            }
        } else if dx < 0 {
            for i in dx..0 {
                *self.pixel(x0 + i, y0).unwrap() = color;
                *self.pixel(x0 + i, y0 + dy).unwrap() = color;
            }
        }
        if dy >= 0 {
            for i in 0..=dy {
                *self.pixel(x0, y0 + i).unwrap() = color;
                *self.pixel(x0 + dx, y0 + i).unwrap() = color;
            }
        } else if dy < 0 {
            for i in dy..0 {
                *self.pixel(x0, y0 + i).unwrap() = color;
                *self.pixel(x0 + dx, y0 + i).unwrap() = color;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    use core::fmt::Write;

    let mut stdout = WriteFd::new(STDOUT);

    writeln!(stdout, "start").unwrap();
    let mut client = Client::connect(LOCALHOST, 33315).unwrap();

    const W: usize = 1024;
    const H: usize = 1024;
    let mut pixels: [u8; W * H] = [0; W * H];

    let mut i = 0;
    let mut j: usize = 0;
    loop {
        pixels[i] = 255;
        i += 1;
        j = (j + 2) % 100;
        {
            let mut canvas = Canvas { w: W, h: H, pixels: &mut pixels };

            canvas.draw_rect(
                j as isize + 3,
                j as isize + 3,
                W as isize - j as isize - 3,
                H as isize - j as isize - 3,
                255
            );
        }

        client.present(Format::GS, W as u16, H as u16, &pixels).unwrap();

        writeln!(stdout, "i: {}, (id: {})", i, client.id()).unwrap();
        Point::now().loop_for(Duration::from_millis(16));
    }
}

#[cfg(not(test))]
mod no_test {
    use core::panic::PanicInfo;

    use mini_winclient::STDERR;
    use mini_winclient::write::WriteFd;

    extern "C" {
        pub fn __exit__(number: usize) -> !;
    }

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        use core::fmt::Write;
        let mut stderr = WriteFd::new(STDERR);
        writeln!(stderr, "{}", info).ok();
        unsafe { __exit__(1); }
    }

    #[lang = "eh_personality"]
    #[no_mangle]
    extern fn eh_personality() {}
}

#[no_mangle]
fn memset(s: *mut c_void, c: c_int, len: usize) -> *mut c_void
{
    let sl = unsafe { slice::from_raw_parts_mut(s as *mut u8, len) };
    for p in sl {
        *p = c as u8
    }
    s
}

#[no_mangle]
fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void
{
    unsafe {
        let dest_s = slice::from_raw_parts_mut(dest as *mut u8, n);
        let src_s = slice::from_raw_parts(src as *const u8, n);

        for i in 0..n {
            dest_s[i] = src_s[i]
        }
        dest
    }
}
