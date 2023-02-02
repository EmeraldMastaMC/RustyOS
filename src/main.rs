#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::arch::asm;
use core::panic::PanicInfo;

use rusty_os::{vga, rand, gdt, interrupts, rdrand, println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::set_text_color(vga::Color::Red);
    println!("\n{}", info);
    loop {
        unsafe {
            hlt();
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let welcome = "Welcome to RustyOS! There isn't much at the moment, but I hope to be able to add more to this OS in the future!";

    init();

    vga::set_bgcolor(vga::Color::White);

    vga::set_text_color(vga::Color::Blue);
    println!("{}", welcome);

    vga::set_text_color(vga::Color::Green);
    println!("Random  8 bit: {}", rdrand!(u8));
    println!("Random 16 bit: {}", rdrand!(u16));
    println!("Random 32 bit: {}", rdrand!(u32));
    println!("Random 64 bit: {}", rdrand!(u64));
    println!(
        "Random 64 bit decimal number from 0 to 1 exclusive: {}",
        rand::rand_float()
    );


    unsafe {
        *(0xDEADBEEF as *mut usize) = 42;
    }
    // x86_64::instructions::interrupts::int3();
    panic!("Hello, Panic!");
}

#[allow(dead_code)]
unsafe fn hlt() {
    asm!("hlt");
}

fn vga_init() {
    vga::disable_cursor();
    vga::toggle_blinking();
}

fn init() {
    vga_init();
    interrupts::init();
    gdt::init();
}
