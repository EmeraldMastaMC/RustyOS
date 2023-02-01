#![no_std]
#![no_main]
use core::arch::asm;

mod ports;
mod rand;
mod vga;
use core::panic::PanicInfo;
use vga::VGA;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    VGA::set_text_color(vga::Color::Red);
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
    let cool_string = "cool string";

    vga_init();
    VGA::set_bgcolor(vga::Color::White);

    VGA::set_text_color(vga::Color::Blue);
    println!("{}", welcome);

    VGA::set_text_color(vga::Color::Green);
    println!(
        "println! macro: {}, {}, {}",
        rdrand!(u8),
        rand::rand_float(),
        cool_string
    );
    panic!("Exception: testing panic! macro");
}

#[allow(dead_code)]
unsafe fn hlt() {
    asm!("hlt");
}

fn vga_init() {
    VGA::disable_cursor();
    VGA::toggle_blinking();
}
