use core::mem::size_of;


macro_rules! read_impl {
    ($name: ident, $from_bytes: ident, $_type: ty) => {
        pub fn $name(input: &mut &[u8]) -> Option<$_type> {
            if input.len() <= size_of::<$_type>() {
                let (int_bytes, rest) = input.split_at(size_of::<$_type>());
                *input = rest;
                Some(<$_type>::$from_bytes(int_bytes.try_into().unwrap()))
            } else {
                None
            }
        }
    };
}

read_impl!(read_le_u8, from_le_bytes, u8);
read_impl!(read_le_u16, from_le_bytes, u16);
read_impl!(read_le_u32, from_le_bytes, u32);
read_impl!(read_le_u64, from_le_bytes, u64);

read_impl!(read_be_u8, from_be_bytes, u8);
read_impl!(read_be_u16, from_be_bytes, u16);
read_impl!(read_be_u32, from_be_bytes, u32);
read_impl!(read_be_u64, from_be_bytes, u64);
