use core::{
    mem::size_of,
    slice
};

use crate::{
    socket::{
        Socket,
        WriteError,
        ReadError, self, FlagsChangeError
    },
    bufsocket::{BufSocket, self},
    sys::io_sys::STDOUT,
    write::WriteFd, event::{Event, self}
};

pub enum Format {
    GS = 0,
    ARGB = 1,
}

const PROTO_VERSION: u8 = 0;

pub type PackageType = u8;

mod in_package_types {
    pub const INIT: super::PackageType = 0;
    pub const EVENT: super::PackageType = 1;
}

mod out_package_types {
    pub const PRESENT: super::PackageType = 0;
}

#[derive(Debug)]
pub enum Package {
    Init(u8),
    Event(Event)
}

#[derive(Debug)]
pub enum PackageError {
    ReadError(ReadError),
    UnknownType(u8),
    EventErr(event::Error)
}

impl Package {
    pub fn pull<const CAPACITY: usize, const CHUNK_LEN: usize>(s: &mut BufSocket<CAPACITY, CHUNK_LEN>) -> Result<Package, PackageError> {
        let mut pakcage_type: PackageType = 0;
        assert!(s.read_transmuted(&mut pakcage_type));
        match pakcage_type {
            in_package_types::INIT => todo!(),
            in_package_types::EVENT => match Event::pull(s) {
                Ok(event) => Ok(Package::Event(event)),
                Err(err) => Err(PackageError::EventErr(err)),
            },
            p => Err(PackageError::UnknownType(p))
        }
    }
}

#[derive(Debug)]
pub enum ConnectError {
    ConnectError(socket::ConnectError),
    ReadError(ReadError)
}


pub struct Client<const CAPACITY: usize, const CHUNK_LEN: usize> {
    s: BufSocket<CAPACITY, CHUNK_LEN>,
    id: u8
}

#[repr(C)]
pub struct Pull {
    pub flags: u32,
    pub w: u16,
    pub h: u16,
}

impl<const CAPACITY: usize, const CHUNK_LEN: usize> Client<CAPACITY, CHUNK_LEN> {
    pub fn connect(ip: [u8; 4], port: u16) -> Result<Self, ConnectError> {
        match BufSocket::connect(ip, port) {
            Ok(mut s) => match Self::wait_for_init(&mut s.soc()) {
                Ok(id) => Ok(Self { s, id,  }),
                Err(err) => Err(ConnectError::ReadError(err)),
            },
            Err(err) => Err(ConnectError::ConnectError(err)),
        }
    }

    #[inline]
    pub fn set_non_blocking_mode(&mut self, nbm: bool) -> Result<(), FlagsChangeError> {
        self.s.set_non_blocking_mode(nbm)
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

    pub fn read_package(&mut self) -> Result<Option<Package>, PackageError> {
        use core::fmt::Write;

        let mut stdout = WriteFd::new(STDOUT);

        writeln!(stdout, "read_package:").unwrap();

        match self.s.bufferize() {
            Ok(_) => {
                writeln!(stdout, "read_package:bufferized").unwrap();

                let mut package_size: u32 = 0;
                if self.s.peek_transmuted(&mut package_size) {
                    writeln!(stdout, "read_package:peek_transmuted(psize)): {} => {} + {} = {}",
                        self.s.bytes_available(),
                        size_of::<u32>(),
                        package_size, self.s.bytes_available() >= size_of::<u32>() + package_size as usize
                    ).unwrap();

                    if self.s.bytes_available() >= size_of::<u32>() + package_size as usize {
                        self.s.read_transmuted(&mut package_size);

                        return Package::pull(&mut self.s).map(|p| Some(p));
                    }
                }
                Ok(None)
            },
            Err(err) => match err {
                ReadError::Again => {
                    writeln!(stdout, "read_package:Again").unwrap();

                    Ok(None)
                },
                err => Err(PackageError::ReadError(err)),
            },
        }
    }

    //pub fn pull_event(&mut self) -> Option<Event> {
    //    let mut package_size: u32 = 0;
    //    if self.s.peek_transmuted(&mut package_size)? {
    //        self.s.bytes_available() >= size_of::<u32>()
    //    }
//
    //}

    pub fn present<P: Sized>(&mut self, format: Format, w: u16, h: u16, pixels: &[P]) -> Result<(), WriteError> {
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
        self.s.write_transmuted(out_package_types::PRESENT)?;
        self.s.write_transmuted(format)?;
        self.s.write_transmuted(pixel_size)?;
        self.s.write_transmuted(w)?;
        self.s.write_transmuted(h)?;
        self.s.write_bytes(unsafe {
            slice::from_raw_parts(pixels.as_ptr() as *const u8, pixels.len() * size_of::<P>())
        })?;
        Ok(())
    }

    //pub fn pull(&mut self) -> Result<Pull, ReadError> {
    //    let mut p: Pull = Pull { flags: 0, w: 0, h: 0 };
    //    self.s.read_bytes(unsafe {
    //        slice::from_raw_parts_mut((&mut p) as *mut Pull as *mut u8, size_of::<Pull>())
    //    })?;
    //    Ok(p)
    //}
}
