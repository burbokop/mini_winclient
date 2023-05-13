use core::{mem::{size_of}, slice};

use crate::{socket::{Socket, ConnectError, WriteError, ReadError}, sys::io_sys::STDOUT, write::WriteFd};


pub enum Format {
    GS = 0,
    ARGB = 1,
}

const PROTO_VERSION: u8 = 0;

pub struct Client {
    s: Socket,
    id: u8
}

#[repr(C)]
pub struct Pull {
    pub flags: u32,
    pub w: u16,
    pub h: u16,
}

impl Client {
    pub fn connect(ip: [u8; 4], port: u16) -> Result<Self, ConnectError> {
        Socket::connect(ip, port).and_then(|mut s|
            match Self::wait_for_init(&mut s) {
                Ok(id) => Ok(Self { s, id }),
                Err(err) => Err(ConnectError::ReadError(err)),
            }
        )
    }

    #[inline]
    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn proto_version() -> u8 {
        PROTO_VERSION
    }

    fn wait_for_init(s: &mut Socket) -> Result<u8, ReadError> {

        let mut package_size: u32 = 0;

        let mut btr: usize = 0;
        while btr < size_of::<u32>() {
            btr = s.read_transmuted(&mut package_size)?;
        }
        assert_eq!(btr, size_of::<u32>());

        assert_eq!(package_size as usize, size_of::<u8>() + size_of::<u8>());

        let mut package_type: u8 = 0;
        let mut client_id: u8 = 0;
        assert_eq!(s.read_transmuted(&mut package_type)?, size_of::<u8>());
        assert_eq!(s.read_transmuted(&mut client_id)?, size_of::<u8>());

        assert_eq!(package_type, 0);
        Ok(client_id)
    }

    pub fn present<P: Sized>(&mut self, format: Format, w: u16, h: u16, pixels: &[P]) -> Result<(), WriteError> {
        const PACKAGE_TYPE: u8 = 0;

        let format = format as u8;
        let pixel_size = size_of::<P>() as u8;


        let package_size
            =(size_of::<u8>()  // Self::proto_version()
            + size_of::<u8>()  // client id
            + size_of::<u8>()  // package type
            + size_of::<u8>()  // format
            + size_of::<u8>()  // pixel_size
            + size_of::<u16>() // w
            + size_of::<u16>() // h
            + size_of::<P>() * pixels.len()
        ) as u32;

        use core::fmt::Write;
        let mut stdout = WriteFd::new(STDOUT);
        writeln!(stdout, "ps: {} -> {}", pixels.len(), package_size).unwrap();

        self.s.write_transmuted(package_size)?;

        self.s.write_transmuted(Self::proto_version())?;
        self.s.write_transmuted(self.id)?;
        self.s.write_transmuted(PACKAGE_TYPE)?;
        self.s.write_transmuted(format)?;
        self.s.write_transmuted(pixel_size)?;
        self.s.write_transmuted(w)?;
        self.s.write_transmuted(h)?;
        self.s.write_bytes(unsafe {
            slice::from_raw_parts(pixels.as_ptr() as *const u8, pixels.len() * size_of::<P>())
        })?;
        Ok(())
    }

    pub fn pull(&mut self) -> Result<Pull, ReadError> {
        let mut p: Pull = Pull { flags: 0, w: 0, h: 0 };
        self.s.read_bytes(unsafe {
            slice::from_raw_parts_mut((&mut p) as *mut Pull as *mut u8, size_of::<Pull>())
        })?;
        Ok(p)
    }
}
