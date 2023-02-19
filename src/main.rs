#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::arch::asm;
use core::panic::PanicInfo;

use rusty_os::{gdt, interrupts, println, vga};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::set_text_color(vga::Color::Red);
    println!("\n{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let welcome = "Welcome to RustyOS! There isn't much at the moment, 
    but I hope to be able to add more to this OS in the future!";
    init();
    vga::init();

    vga::set_bgcolor(vga::Color::White);

    vga::set_text_color(vga::Color::Blue);
    println!("{}", welcome);
    panic!("Hello, Panic!");
}

#[allow(dead_code)]
unsafe fn hlt() {
    asm!("hlt");
}

/// Initializes the GDT, IDT, and enables interrupts
fn init() {
    gdt::init();
    interrupts::init();
    unsafe {
        interrupts::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}
