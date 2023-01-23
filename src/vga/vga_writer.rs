#[path = "../ports/mod.rs"]
mod ports;
use crate::ports::io::{inb, outb};
use core::fmt;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,

    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    LightCyan = 0xB,
    LightRed = 0xC,
    Pink = 0xD,
    Yellow = 0xE,
    White = 0xF,
}

const VGA_HEIGHT: u32 = 25;
const VGA_WIDTH: u32 = 80;
const COLOR_OFFSET: u32 = 1;
const CELL_WIDTH: u32 = 2;
const LAST_CURSOR_POSITION: u32 = 2000;
static mut BACK_BUFFER: [u8; 4_000] = [0; 4_000];
static mut FRONT_BUFFER: *mut u8 = 0xB8000 as *mut u8;
static mut BG_COLOR: Color = Color::White;
static mut CURSOR_OFFSET: u32 = 0;
static mut VGA_WRITER: VGAWriter = VGAWriter {};
static mut CURRENT_COLOR: Color = Color::White;

pub struct VGAWriter {}

#[allow(dead_code)]
impl<'a> VGAWriter {
    pub fn putc(character: char, color: Color) {
        unsafe {
            if CURSOR_OFFSET >= LAST_CURSOR_POSITION {
                VGAWriter::nextln();
                return;
            }
            let offset = (CURSOR_OFFSET * CELL_WIDTH) as usize;
            let char_byte = character as u8;

            // Bytes 0-3 are foreground color, and bytes 4-7 are background color
            let color_byte = color as u8 | ((BG_COLOR as u8) << 4);

            if character == '\n' {
                VGAWriter::nextln();
                return;
            }
            // Character byte
            VGAWriter::write_back_buffer(offset, char_byte);

            // Color byte
            VGAWriter::write_back_buffer(offset + COLOR_OFFSET as usize, color_byte);

            // Offset cursor for next character print
            VGAWriter::next_char();
        }
    }
    pub fn putcpos(character: char, color: Color, column: usize, row: usize) {
        unsafe {
            let pos = VGAWriter::get_pos(column, row);
            let char_byte = character as u8;

            // Bytes 0-3 are foreground color, and bytes 4-7 are background color
            let color_byte = color as u8 | ((BG_COLOR as u8) << 4);

            VGAWriter::write_back_buffer(pos, char_byte);
            VGAWriter::write_back_buffer(pos + COLOR_OFFSET as usize, color_byte);
        }
    }
    pub fn puts(outstr: &'a str, color: Color) {
        for character in outstr.as_bytes().iter() {
            let character = *character as char;
            VGAWriter::putc(character, color);
        }
    }
    pub fn set_bgcolor(color: Color) {
        unsafe {
            BG_COLOR = color;
            for i in 0..(VGA_HEIGHT * VGA_WIDTH) {
                let i = (i * CELL_WIDTH + COLOR_OFFSET) as usize;
                VGAWriter::write_back_buffer(
                    i,
                    // Deletes background bits, keeps foreground bits, and changes the background bits
                    // to our new background
                    (VGAWriter::read_back_buffer(i) & 0b0000_1111) | ((BG_COLOR as u8) << 4),
                );
            }
        }
    }
    pub fn set_text_color(color: Color) {
        unsafe {
            CURRENT_COLOR = color;
        }
    }
    pub fn swap_buf() {
        for i in 0..(VGA_HEIGHT * VGA_WIDTH * CELL_WIDTH) {
            let i = i as usize;
            VGAWriter::write_front_buffer(i, VGAWriter::read_back_buffer(i));
        }
    }
    fn reset_cursor() {
        unsafe {
            CURSOR_OFFSET =
                (VGAWriter::get_pos(0, (VGA_HEIGHT - 1) as usize) / CELL_WIDTH as usize) as u32;
        }
    }
    fn next_char() {
        unsafe {
            if CURSOR_OFFSET >= LAST_CURSOR_POSITION {
                VGAWriter::nextln();
                return;
            }
            CURSOR_OFFSET += 1;
        }
    }
    fn nextln() {
        unsafe {
            if CURSOR_OFFSET >= LAST_CURSOR_POSITION - VGA_WIDTH {
                VGAWriter::shift_up();
                VGAWriter::reset_cursor();
                return;
            }
            CURSOR_OFFSET = CURSOR_OFFSET + (VGA_WIDTH - (CURSOR_OFFSET % VGA_WIDTH));
        }
    }
    fn shift_up() {
        for column in 0..VGA_WIDTH {
            let column = column as usize;
            for row in 0..(VGA_HEIGHT - 1) {
                let row = row as usize;
                // Write to each line what is below it, for this line of code it copies the below
                // character
                VGAWriter::write_back_buffer(
                    VGAWriter::get_pos(column, row),
                    VGAWriter::read_back_buffer(VGAWriter::get_pos(column, row + 1)),
                );

                // Write to each line what is below it, for this line of code is copies the below
                // color
                VGAWriter::write_back_buffer(
                    VGAWriter::get_pos(column, row) + COLOR_OFFSET as usize,
                    VGAWriter::read_back_buffer(
                        VGAWriter::get_pos(column, row + 1) + COLOR_OFFSET as usize,
                    ),
                );
            }
        }
        for column in 0..VGA_WIDTH {
            let column = column as usize;
            VGAWriter::putcpos('\0', Color::White, column, (VGA_HEIGHT - 1) as usize);
        }
    }
    pub fn new() -> Self {
        VGAWriter {}
    }
    pub fn disable_cursor() {
        outb(0x3D4, 0x0A);
        outb(0x3D5, 0x20);
    }

    // Big thanks to u/FredFredricson for easy to understand information on toggling text blinking
    // https://www.reddit.com/r/osdev/comments/70fcig/blinking_text/
    #[allow(unused_assignments)]
    pub fn toggle_blinking() {
        // Read from port 0x03DA to enable adress mode, ignore value, but keep value anyway
        let mut attribute_mode = inb(0x03DA);

        // Write 0x30 to port 0x03C0 to enable "Attribute Mode Control Register"
        outb(0x03C0, 0x30);

        // Now that "Attribute Mode Control Register" is selected, we now read from port 0x03C1
        attribute_mode = inb(0x03C1);

        // Toggle bit 3 (starting from 0) which is the blinking bit
        attribute_mode = attribute_mode ^ 0b0000_1000;

        // Send attribute mode to port 0x03C0 to toggle the blinking
        outb(0x03C0, attribute_mode);
    }
    fn write_back_buffer(index: usize, data: u8) {
        unsafe {
            BACK_BUFFER[index] = data;
        }
    }
    fn read_back_buffer(index: usize) -> u8 {
        unsafe { BACK_BUFFER[index] }
    }
    fn write_front_buffer(index: usize, data: u8) {
        unsafe {
            *FRONT_BUFFER.offset(index as isize) = data;
        }
    }
    fn get_pos(column: usize, row: usize) -> usize {
        let pos = ((column + (row * VGA_WIDTH as usize)) * CELL_WIDTH as usize) as usize;
        pos
    }
    pub fn _print(args: fmt::Arguments) {
        use core::fmt::Write;
        unsafe {
            VGA_WRITER.write_fmt(args).unwrap();
        }
    }
}

impl fmt::Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            VGAWriter::puts(s, CURRENT_COLOR);
            VGAWriter::swap_buf();
            Ok(())
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::vga_writer::VGAWriter::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
