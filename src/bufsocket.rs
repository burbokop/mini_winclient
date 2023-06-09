use core::{mem::{MaybeUninit, size_of}, slice};

use crate::{
    socket::{
        Socket,
        WriteError,
        ReadError,
        FlagsChangeError,
        ConnectError
    },
    utils::CyclicBuf
};

pub struct BufSocket<const CAPACITY: usize, const CHUNK_LEN: usize> {
    s: Socket,
    buf: CyclicBuf<u8, CAPACITY>,
    chunk: [u8; CHUNK_LEN]
}

impl<const CAPACITY: usize, const CHUNK_LEN: usize> BufSocket<CAPACITY, CHUNK_LEN> {
    #[inline]
    #[deprecated]
    pub fn soc(&mut self) -> &mut Socket {
        &mut self.s
    }

    #[inline]
    pub fn connect(ip: [u8; 4], port: u16) -> Result<Self, ConnectError> {
        match Socket::connect(ip, port) {
            Ok(s) => Ok(Self {
                s,
                buf: Default::default(),
                chunk: unsafe { MaybeUninit::uninit().assume_init() }
            }),
            Err(err) => Err(err),
        }
    }

    #[inline]
    pub fn set_non_blocking_mode(&mut self, nbm: bool) -> Result<(), FlagsChangeError> {
        self.s.set_non_blocking_mode(nbm)
    }

    #[inline]
    pub fn bytes_available(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn write_bytes(&mut self, b: &[u8]) -> Result<usize, WriteError> {
        self.s.write_bytes(b)
    }

    #[inline]
    pub fn write_transmuted<T: Sized>(&mut self, val: T) -> Result<usize, WriteError> {
        self.s.write_transmuted(val)
    }

    #[inline]
    pub fn read_transmuted<T: Sized>(&mut self, dst: &mut T) -> bool {
        if self.buf.len() >= size_of::<T>() {
            let sl = unsafe {
                slice::from_raw_parts_mut(dst as *mut T as *mut u8, size_of::<T>())
            };
            assert_eq!(self.read_bytes(sl), sl.len());
            #[cfg(target_endian = "little")]
            sl.reverse();
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn peek_transmuted<T: Sized>(&mut self, dst: &mut T) -> bool {
        if self.buf.len() >= size_of::<T>() {
            let sl = unsafe {
                slice::from_raw_parts_mut(dst as *mut T as *mut u8, size_of::<T>())
            };
            assert_eq!(self.peek(sl), sl.len());
            #[cfg(target_endian = "little")]
            sl.reverse();
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn bufferize_chunk(&mut self) -> Result<usize, ReadError> {
        let can_read_bytes = self.buf.push_ability();

        Ok(if can_read_bytes > 0 {
            let bytes_read = self.s.read_bytes(&mut self.chunk[0..can_read_bytes.min(CHUNK_LEN)])?;
            for i in 0..bytes_read {
                assert!(self.buf.push(self.chunk[i]));
            }
            bytes_read
        } else { 0 })
    }

    #[inline]
    pub fn bufferize(&mut self) -> Result<usize, ReadError> {
        let mut bufferized = 0;
        loop {
            match self.bufferize_chunk() {
                Ok(count) => {
                    if count == 0 { break; }
                    bufferized += count;
                },
                Err(err) => match err {
                    ReadError::Again => break,
                    err => Err(err)?,
                },
            }
        }
        Ok(bufferized)
    }

    #[inline]
    pub fn peek(&mut self, b: &mut[u8]) -> usize {
        self.buf.peek(b)
    }

    #[inline]
    pub fn read_bytes(&mut self, b: &mut[u8]) -> usize {
        let bytes_to_read = self.buf.len().min(b.len());
        for i in 0..bytes_to_read {
            b[i] = self.buf.pop().unwrap();
        }
        bytes_to_read
    }
}
