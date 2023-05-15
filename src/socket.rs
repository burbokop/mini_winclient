use core::{
    mem::{
        MaybeUninit,
        size_of
    },
    ffi::c_void,
    slice,
    fmt::Write
};

use crate::sys::io_sys;

pub const LOCALHOST: [u8; 4] = [ 127, 0, 0, 1 ];

#[derive(Debug)]
pub enum ConnectError {
    NoAccess,
    AfNoSupport,
    Invalid,
    MFile,
    NFile,
    NoBufs,
    ProtoNoSupport,
    UnknownCreateErr(io_sys::ErrNo),

    Refused,
    UnknownConnectErr(io_sys::ErrNo),
}

#[derive(Debug)]
pub enum ReadError {
    Again,
    Unknown(io_sys::ErrNo),
}

#[derive(Debug)]
pub enum FlagsChangeError {
    BadFd,
    Interrupted,
    Unknown(io_sys::ErrNo),
}

impl From<io_sys::ErrNo> for FlagsChangeError {
    fn from(value: io_sys::ErrNo) -> Self {
        match value {
            io_sys::errno::EBADF => FlagsChangeError::BadFd,
            io_sys::errno::EINTR => FlagsChangeError::Interrupted,
            other => FlagsChangeError::Unknown(other)
        }
    }
}


#[derive(Debug)]
pub enum WriteError {
    Unknown(io_sys::ErrNo),
}

pub struct Socket {
    fd: io_sys::Fd
}

pub struct SocketIter<'s, const BUF_SIZE: usize> {
    s: &'s mut Socket,
    i: usize,
    count: usize,
    buf: [u8; BUF_SIZE]
}

impl<'s, const BUF_SIZE: usize> SocketIter<'s, BUF_SIZE> {
    fn new(s: &'s mut Socket) -> Self {
        Self {
            s,
            i: 0,
            count: 0,
            buf: unsafe { MaybeUninit::uninit().assume_init() }
        }
    }
}

impl Socket {
    pub fn connect(ip: [u8; 4], port: u16) -> Result<Self, ConnectError> {
        match unsafe {
            io_sys::socket(io_sys::AF_INET, io_sys::SOCK_STREAM, io_sys::IPPROTO_TCP)
        } {
            Ok(fd) => {
                let addr = io_sys::SockAddr::new(
                    io_sys::AF_INET,
                    port,
                    ip
                );

                match unsafe {
                    io_sys::connect(fd, &addr)
                } {
                    Ok(_) => Ok(Self { fd: fd }),
                    Err(errno) => {
                        unsafe { io_sys::close(fd) };
                        match errno {
                            io_sys::errno::EACCES => todo!(),
                            io_sys::errno::EADDRINUSE => todo!(),
                            io_sys::errno::EADDRNOTAVAIL => todo!(),
                            io_sys::errno::EAFNOSUPPORT => todo!(),
                            io_sys::errno::EAGAIN => todo!(),
                            io_sys::errno::EALREADY => todo!(),
                            io_sys::errno::EBADF => todo!(),
                            io_sys::errno::ECONNREFUSED => Err(ConnectError::Refused),
                            io_sys::errno::EFAULT => todo!(),
                            io_sys::errno::EINPROGRESS => todo!(),
                            io_sys::errno::EINTR => todo!(),
                            io_sys::errno::EISCONN => todo!(),
                            io_sys::errno::ENETUNREACH => todo!(),
                            io_sys::errno::ENOTSOCK => todo!(),
                            io_sys::errno::EPROTOTYPE => todo!(),
                            io_sys::errno::ETIMEDOUT => todo!(),
                            i => Err(ConnectError::UnknownConnectErr(i))
                        }
                    },
                }
            },
            Err(errno) => match errno {
                io_sys::errno::EACCES => Err(ConnectError::NoAccess),
                io_sys::errno::EAFNOSUPPORT => Err(ConnectError::AfNoSupport),
                io_sys::errno::EINVAL => Err(ConnectError::Invalid),
                io_sys::errno::EMFILE => Err(ConnectError::MFile),
                io_sys::errno::ENFILE => Err(ConnectError::NFile),
                io_sys::errno::ENOBUFS => Err(ConnectError::NoBufs),
                io_sys::errno::EPROTONOSUPPORT => Err(ConnectError::ProtoNoSupport),
                i => Err(ConnectError::UnknownCreateErr(i))
            },
        }
    }

    pub fn set_non_blocking_mode(&mut self, nbm: bool) -> Result<(), FlagsChangeError> {
        unsafe {
            let mut flags = io_sys::fcntl(self.fd, io_sys::F_GETFL, 0)
                .map_err(|e|FlagsChangeError::from(e))?;

            if nbm {
                flags = flags | io_sys::bits::O_NONBLOCK;
            } else {
                flags = flags & !io_sys::bits::O_NONBLOCK;
            }

            io_sys::fcntl(self.fd, io_sys::F_SETFL, flags)
                .map_err(|e|FlagsChangeError::from(e))?;
        }
        Ok(())
    }

    pub fn write_bytes(&mut self, b: &[u8]) -> Result<usize, WriteError> {
        match unsafe {
            io_sys::write(self.fd, b.as_ptr() as *const c_void, b.len())
        } {
            Ok(bytes) => Ok(bytes),
            Err(errno) => match errno {
                io_sys::errno::EAGAIN => todo!(),
                io_sys::errno::EBADF => todo!(),
                io_sys::errno::EDESTADDRREQ => todo!(),
                io_sys::errno::EDQUOT => todo!(),
                io_sys::errno::EFAULT => todo!(),
                io_sys::errno::EFBIG => todo!(),
                io_sys::errno::EINTR => todo!(),
                io_sys::errno::EINVAL => todo!(),
                io_sys::errno::EIO => todo!(),
                io_sys::errno::ENOSPC => todo!(),
                io_sys::errno::EPERM => todo!(),
                io_sys::errno::EPIPE => todo!(),
                i => Err(WriteError::Unknown(i))
            },
        }
    }


    pub fn write_transmuted<T: Sized>(&mut self, val: T) -> Result<usize, WriteError> {
        self.write_bytes(unsafe {
            slice::from_raw_parts((&val)
                as *const T
                as *const u8,
            size_of::<T>())
        })
    }

    pub fn read_transmuted<T: Sized>(&mut self, dst: &mut T) -> Result<usize, ReadError> {
        self.read_bytes(unsafe {
            slice::from_raw_parts_mut(dst
                as *mut T
                as *mut u8,
            size_of::<T>())
        })
    }

    pub fn read_bytes(&mut self, b: &mut[u8]) -> Result<usize, ReadError> {
        match unsafe {
            io_sys::read(self.fd, b.as_mut_ptr() as *mut c_void, b.len())
        } {
            Ok(bytes) => Ok(bytes),
            Err(errno) => match errno {
                io_sys::errno::EAGAIN => Err(ReadError::Again),
                io_sys::errno::EBADF => todo!(),
                io_sys::errno::EFAULT => todo!(),
                io_sys::errno::EINTR => todo!(),
                io_sys::errno::EINVAL => todo!(),
                io_sys::errno::EIO => todo!(),
                io_sys::errno::EISDIR => todo!(),
                i => Err(ReadError::Unknown(i))
            },
        }
    }

    pub fn iter<'s, const BUF_SIZE: usize>(&'s mut self) -> SocketIter<'s, BUF_SIZE> {
        SocketIter::new(self)
    }
}

impl Write for Socket {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match self.write_bytes(s.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(core::fmt::Error),
        }
    }
}

impl<'s, const BUF_SIZE: usize> Iterator for SocketIter<'s, BUF_SIZE> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.count {
            match self.s.read_bytes(&mut self.buf) {
                Ok(count) => {
                    self.count = count;
                    self.i = 0;
                }
                Err(_) => todo!(),
            }
        }

        if self.i < self.count {
            let byte = self.buf[self.i];
            self.i += 1;
            Some(byte)
        } else {
            None
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            io_sys::close(self.fd)
        }
    }
}
