#![no_std]
#![feature(abi_x86_interrupt)]
pub mod io;
pub mod rand;
pub mod display {
    pub mod vga;
}
