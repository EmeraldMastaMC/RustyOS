#![no_std]
#![no_main]

use core::arch::asm;

mod ports;
mod vga;
use core::panic::PanicInfo;
use vga::vga_writer::Color;
use vga::vga_writer::VGAWriter;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    VGAWriter::set_text_color(Color::Red);
    println!("\n{}", info);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let welcome = "Welcome to RustyOS! There isn't much at the moment, but I hope to be able to add more to this OS in the future!";
    let cool_string = "cool string";

    VGAWriter::disable_cursor();
    VGAWriter::toggle_blinking();
    VGAWriter::set_bgcolor(Color::White);

    VGAWriter::set_text_color(Color::Blue);
    println!("{}", welcome);

    VGAWriter::set_text_color(Color::Green);
    println!("println! macro: {}, {}, {}", 42, 2.0 / 3.0, cool_string);
    panic!("Exception: testing panic! macro");
}

#[allow(dead_code)]
unsafe fn hlt() {
    asm!("hlt");
}
