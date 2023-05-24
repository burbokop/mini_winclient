use crate::bufsocket::BufSocket;

mod event_types {
    pub const CLOSE: u8 = 0;
    pub const RESIZE: u8 = 1;
}

#[derive(Debug)]
pub enum Event {
    Close,
    Resize{w: u16, h: u16}
}

#[derive(Debug)]
pub enum Error {
    UnknownType(u8)
}

impl Event {
    pub fn pull<const CAPACITY: usize, const CHUNK_LEN: usize>(s: &mut BufSocket<CAPACITY, CHUNK_LEN>) -> Result<Event, Error> {
        let mut _type: u8 = 0;
        assert!(s.read_transmuted(&mut _type));

        match _type {
            event_types::CLOSE => Ok(Event::Close),
            event_types::RESIZE => {
                let mut w: u16 = 0;
                let mut h: u16 = 0;
                assert!(s.read_transmuted(&mut w));
                assert!(s.read_transmuted(&mut h));
                Ok(Event::Resize { w, h })
            }
            t => Err(Error::UnknownType(t))
        }
    }
}
