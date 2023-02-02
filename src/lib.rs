#![no_std]
#![feature(abi_x86_interrupt)]
pub mod gdt;
pub mod interrupts;
pub mod io;
pub mod rand;
pub mod vga;
