use core::{ffi::c_void, mem::{transmute, size_of}};

use super::syscall::{
    nums::{
        SYS_CLOSE,
        SYS_WRITE,
        SYS_READ,
        SYS_SOCKET,
        SYS_CONNECT, SYS_FCNTL
    }, __mini_wc_syscall3__, __mini_wc_syscall1__,
};

pub type ErrNo = usize;
pub type Fd = usize;

pub mod errno {
    use super::ErrNo;

    pub const EPERM: ErrNo = 1;

    pub const EINTR: ErrNo = 4;
    pub const EIO: ErrNo = 5;

    pub const EBADF: ErrNo = 9;

    pub const EAGAIN: ErrNo = 11;

    pub const EACCES: ErrNo = 13;
    pub const EFAULT: ErrNo = 14;

    pub const EISDIR: ErrNo = 21;
    pub const EINVAL: ErrNo = 22;
    pub const ENFILE: ErrNo = 23;
    pub const EMFILE: ErrNo = 24;

    pub const EFBIG: ErrNo = 27;
    pub const ENOSPC: ErrNo = 28;

    pub const EPIPE: ErrNo = 32;

    pub const ENOTSOCK: ErrNo = 88;
    pub const EDESTADDRREQ: ErrNo = 89;

    pub const EPROTOTYPE: ErrNo = 91;

    pub const EPROTONOSUPPORT: ErrNo = 93;

    pub const EAFNOSUPPORT: ErrNo = 97;
    pub const EADDRINUSE: ErrNo = 98;
    pub const EADDRNOTAVAIL: ErrNo = 99;

    pub const ENETUNREACH: ErrNo = 101;

    pub const ENOBUFS: ErrNo = 105;
    pub const EISCONN: ErrNo = 106;

    pub const ETIMEDOUT: ErrNo = 110;
    pub const ECONNREFUSED: ErrNo = 111;

    pub const EALREADY: ErrNo = 114;
    pub const EINPROGRESS: ErrNo = 115;

    pub const EDQUOT: ErrNo = 122;
}

unsafe fn separate_unit(val: *mut c_void) -> Result<(), ErrNo> {
    if transmute::<_, isize>(val) >= 0 {
        Ok(())
    } else {
        Err((-transmute::<_, isize>(val)) as ErrNo)
    }
}

unsafe fn separate_usize(val: *mut c_void) -> Result<usize, ErrNo> {
    if transmute::<_, isize>(val) >= 0 {
        Ok(transmute::<_, usize>(val))
    } else {
        Err((-transmute::<_, isize>(val)) as ErrNo)
    }
}

unsafe fn separate_fd(val: *mut c_void) -> Result<Fd, ErrNo> {
    if transmute::<_, isize>(val) >= 0 {
        Ok(transmute::<_, Fd>(val))
    } else {
        Err((-transmute::<_, isize>(val)) as ErrNo)
    }
}

/// Close file or socket by descriptor
pub unsafe fn close(fd: usize) {
    __mini_wc_syscall1__(SYS_CLOSE, transmute(fd));
}

pub unsafe fn write(fd: usize, data: * const c_void, nbytes: usize) -> Result<usize, ErrNo> {
    separate_usize(__mini_wc_syscall3__(
        SYS_WRITE,
        transmute(fd),
        data as *mut c_void,
        transmute(nbytes)
    ))
}

pub unsafe fn read(fd: usize, data: *mut c_void, nbytes: usize) -> Result<usize, ErrNo> {
    separate_usize(__mini_wc_syscall3__(
        SYS_READ,
        transmute(fd),
        data,
        transmute(nbytes)
    ))
}

pub unsafe fn fcntl(fd: usize, cmd: usize, arg: usize) -> Result<usize, ErrNo> {
    separate_usize(__mini_wc_syscall3__(
        SYS_FCNTL,
        transmute(fd),
        transmute(cmd),
        transmute(arg)
    ))
}

pub static STDOUT: usize = 1;
pub static STDERR: usize = 2;

pub static AF_INET: usize = 2;
pub static SOCK_STREAM: usize = 1;
pub static IPPROTO_TCP: usize = 6;

/// Duplicate file descriptor.
pub static F_DUPFD: usize = 0;
/// Get file descriptor flags.
pub static F_GETFD: usize = 1;
/// Set file descriptor flags.
pub static F_SETFD: usize = 2;
/// Get file status flags.
pub static F_GETFL: usize = 3;
/// Set file status flags.
pub static F_SETFL: usize = 4;

pub mod bits {
    pub static O_NONBLOCK: usize = 04000;
}

fn flip16(v: u16) -> u16 {
    return (v << 8) | (v >> 8);
}

#[repr(C)]
pub struct SockAddr {
    family: u16,
    /// NOTE: this is big endian
    port: u16,
    /// NOTE: this is big endian
    addr: u32,
    zero: [u8; 8],
}

impl SockAddr {
    pub fn new(family: usize, port: u16, ip: [u8; 4]) -> Self {
        Self {
            family: family as u16,
            port: flip16(port),
            addr: unsafe { transmute(ip) },
            zero: [0; 8]
        }
    }

}

#[cfg(target_arch = "x86")]
mod socketcall {
    use core::mem::transmute;

    static SYS_SOCKET: usize = 1;
    static SYS_CONNECT: usize  = 3;

    fn socketcall(call: u32, args: *mut void) -> usize {
        return syscall2(
            SYS_socketcall,
            transmute(call as usize),
            args
        ) as usize;
    }
}

pub unsafe fn socket(family: usize, _type: usize, protocol: usize) -> Result<Fd, ErrNo> {
    #[cfg(target_arch = "x86")]
    {
        void* args[3];
        args[0] = transmute(family);
        args[1] = transmute(_type);
        args[2] = transmute(protocol);

        separate_fd(socketcall::socketcall(SYS_SOCKET, args))
    }
    #[cfg(target_arch = "x86_64")]
    {
        separate_fd(__mini_wc_syscall3__(
            SYS_SOCKET,
            transmute(family),
            transmute(_type),
            transmute(protocol),
        ))
    }
}

pub unsafe fn connect(sockfd: usize, addr: *const SockAddr) -> Result<(), ErrNo> {
    #[cfg(target_arch = "x86")]
    {
        void* args[3];
        args[0] = transmute(sockfd as usize);
        args[1] = addr as *mut c_void;
        args[2] = transmute(sizeof(sockaddr_in));

        socketcall(SYS_CONNECT, args);
    }
    #[cfg(target_arch = "x86_64")]
    {
        separate_unit(__mini_wc_syscall3__(
            SYS_CONNECT,
            transmute(sockfd),
            addr as *mut c_void,
            transmute(size_of::<SockAddr>())
        ))
    }
}
