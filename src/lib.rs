#![no_std]
#![feature(maybe_uninit_uninit_array)]

mod sys;
pub mod socket;
pub mod write;
pub mod winclient;
pub mod time;

pub use sys::io_sys::{
    STDOUT,
    STDERR
};
