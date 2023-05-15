use core::{mem::{MaybeUninit, size_of}, slice};

use crate::{socket::{Socket, WriteError, ReadError, self, FlagsChangeError, ConnectError}, STDOUT, write::WriteFd};

struct CyclicBuf<T, const CAPACITY: usize> {
    buf: [MaybeUninit<T>; CAPACITY],
    begin: usize,
    end: usize
}

impl<T, const CAPACITY: usize> Default for CyclicBuf<T, CAPACITY> {
    #[inline]
    fn default() -> Self {
        Self { buf: MaybeUninit::uninit_array(), begin: 0, end: 0 }
    }
}

impl<T, const CAPACITY: usize> CyclicBuf<T, CAPACITY> {
    #[inline]
    pub fn push(&mut self, v: T) -> bool {
        if self.is_full() { false } else {
            self.buf[self.end].write(v);
            self.end = (self.end + 1) % CAPACITY;
            true
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() { None } else {
            let v = unsafe { self.buf[self.begin].assume_init_read() };
            self.begin = (self.begin + 1) % CAPACITY;
            Some(v)
        }
    }

    #[inline]
    pub fn top(&mut self) -> Option<T>
    where
        MaybeUninit<T>: Clone
    {
        if self.is_empty() { None } else {
            Some(unsafe { self.buf[self.begin].clone().assume_init() })
        }
    }

    #[inline]
    pub fn peek(&mut self, output: &mut [T]) -> usize
    where
        MaybeUninit<T>: Clone
    {
        let mut fb = self.begin;
        let len = output.len().min(self.len());
        for i in 0..len {
            output[i] = unsafe { self.buf[fb].clone().assume_init() };
            fb = (fb + 1) % CAPACITY;
        }
        len
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.push_ability() == 0
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.begin == self.end
    }

    #[inline]
    pub fn len(&self) -> usize {
        CAPACITY - self.push_ability() - 1
    }

    //1 - 0 = 1

    //-1

    //0 1 4

    #[inline]
    pub fn push_ability(&self) -> usize {

            (self.begin as isize - self.end as isize - 1).rem_euclid(CAPACITY as isize)
         as usize
    }
}

#[cfg(test)]
mod tests {
    use super::CyclicBuf;

    #[test]
    fn cyclic_buf_until_full_test() {
        let mut buf: CyclicBuf<u8, 4> = CyclicBuf::default();

        assert_eq!(buf.len(), 0);
        assert_eq!(buf.push_ability(), 3);
        assert_eq!(buf.is_empty(), true);
        assert_eq!(buf.is_full(), false);

        assert_eq!(buf.push(0), true);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.push_ability(), 2);
        assert_eq!(buf.is_empty(), false);
        assert_eq!(buf.is_full(), false);

        assert_eq!(buf.push(0), true);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.push_ability(), 1);
        assert_eq!(buf.is_empty(), false);
        assert_eq!(buf.is_full(), false);

        assert_eq!(buf.push(0), true);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.push_ability(), 0);
        assert_eq!(buf.is_empty(), false);
        assert_eq!(buf.is_full(), true);

        assert_eq!(buf.push(0), false);
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.push_ability(), 0);
        assert_eq!(buf.is_empty(), false);
        assert_eq!(buf.is_full(), true);
    }

    #[test]
    fn cyclic_buf_push_pop_test() {
        let mut buf: CyclicBuf<u8, 4> = CyclicBuf::default();
        assert_eq!(buf.push(0), true);
        assert_eq!(buf.push(1), true);
        assert_eq!(buf.push_ability(), 1);

        assert_eq!(buf.pop(), Some(0));
        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.push_ability(), 3);

        assert_eq!(buf.push(2), true);
        assert_eq!(buf.push(3), true);
        assert_eq!(buf.push_ability(), 1);

        assert_eq!(buf.pop(), Some(2));
        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.push_ability(), 3);

        assert_eq!(buf.push(4), true);
        assert_eq!(buf.push(5), true);
        assert_eq!(buf.push_ability(), 1);

        assert_eq!(buf.push(6), true);
        assert_eq!(buf.push(7), false);
        assert_eq!(buf.push_ability(), 0);

        assert_eq!(buf.pop(), Some(4));
        assert_eq!(buf.pop(), Some(5));
        assert_eq!(buf.push_ability(), 2);

        assert_eq!(buf.push(8), true);
        assert_eq!(buf.push(9), true);

        assert_eq!(buf.push_ability(), 0);

        assert_eq!(buf.pop(), Some(6));
        assert_eq!(buf.pop(), Some(8));
        assert_eq!(buf.pop(), Some(9));

        assert_eq!(buf.pop(), None);
        assert_eq!(buf.push_ability(), 3);
    }
}

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
            self.read_bytes(unsafe {
                slice::from_raw_parts_mut(dst
                    as *mut T
                    as *mut u8,
                size_of::<T>())
            });
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn peek_transmuted<T: Sized>(&mut self, dst: &mut T) -> bool {
        if self.buf.len() >= size_of::<T>() {
            self.peek(unsafe {
                slice::from_raw_parts_mut(dst
                    as *mut T
                    as *mut u8,
                size_of::<T>())
            });
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn bufferize_chunk(&mut self) -> Result<usize, ReadError> {
        let can_read_bytes = self.buf.push_ability();
        use core::fmt::Write;

        let mut stdout = WriteFd::new(STDOUT);

        writeln!(stdout, "bufferize_chunk:can_read_bytes: {}", can_read_bytes).unwrap();

        Ok(if can_read_bytes > 0 {
            let bytes_read = self.s.read_bytes(&mut self.chunk[0..can_read_bytes.min(CHUNK_LEN)])?;
            writeln!(stdout, "bufferize_chunk:bytes_read: {}", bytes_read).unwrap();
            for i in 0..bytes_read {
                writeln!(stdout, "bufferize_chunk:i: {}, {:x}", i, self.chunk[i]).unwrap();
                assert!(self.buf.push(self.chunk[i]));
            }
            bytes_read
        } else { 0 })
    }

    #[inline]
    pub fn bufferize(&mut self) -> Result<usize, ReadError> {
        let mut bufferized = 0;
        loop {
            let b = self.bufferize_chunk()?;
            if b == 0 { break; }
            bufferized += b;
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
