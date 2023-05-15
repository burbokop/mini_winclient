#![no_std]
#![feature(maybe_uninit_uninit_array)]
#![feature(generic_const_exprs)]
#![feature(inherent_associated_types)]

mod sys;
mod read;
pub mod event;
pub mod socket;
pub mod bufsocket;
pub mod write;
pub mod winclient;
pub mod time;

pub use sys::io_sys::{
    STDOUT,
    STDERR
};
