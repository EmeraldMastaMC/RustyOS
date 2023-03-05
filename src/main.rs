#![no_std]
#![no_main]

use core::arch::asm;
use core::panic::PanicInfo;

use rusty_os::display::vga;
use rusty_os::{print, println};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n{}", info);
    vga::update();
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga::disable_cursor();
    vga::toggle_blinking();
    println!("Hello, World!");
    println!("{} {}", 5, "is a cool number");
    print!("Welcome to Rusty_OS!");
    vga::update();
    panic!("Hello, Panic!");
}

#[allow(dead_code)]
unsafe fn hlt() {
    asm!("hlt");
}
