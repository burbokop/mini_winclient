use core::mem::MaybeUninit;


pub struct CyclicBuf<T, const CAPACITY: usize> {
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
    pub fn top(&self) -> Option<T>
    where
        MaybeUninit<T>: Clone
    {
        if self.is_empty() { None } else {
            Some(unsafe { self.buf[self.begin].clone().assume_init() })
        }
    }

    #[inline]
    pub fn peek(&self, output: &mut [T]) -> usize
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
    fn until_full_test() {
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
    fn push_pop_test() {
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

    #[test]
    fn top_test() {
        let mut buf: CyclicBuf<u8, 4> = CyclicBuf::default();
        assert_eq!(buf.push(0), true);
        assert_eq!(buf.push(1), true);
        assert_eq!(buf.len(), 2);

        assert_eq!(buf.top(), Some(0));
        assert_eq!(buf.len(), 2);

        assert_eq!(buf.pop(), Some(0));
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.top(), None);
    }

    #[test]
    fn peek_test() {
        let mut buf: CyclicBuf<u8, 4> = CyclicBuf::default();

        assert_eq!(buf.push(0), true);
        assert_eq!(buf.push(1), true);
        assert_eq!(buf.len(), 2);

        let mut tmp: [u8; 2] = [0; 2];
        assert_eq!(buf.peek(&mut tmp), 2);
        assert_eq!(tmp, [0, 1]);
    }

    #[test]
    fn peek_not_all_test() {
        let mut buf: CyclicBuf<u8, 4> = CyclicBuf::default();

        assert_eq!(buf.push(1), true);
        assert_eq!(buf.push(2), true);
        assert_eq!(buf.len(), 2);

        let mut tmp: [u8; 4] = [0; 4];
        assert_eq!(buf.peek(&mut tmp), 2);
        assert_eq!(tmp, [1, 2, 0, 0]);
    }

}
