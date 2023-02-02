//! # VGA
//! This module allows interactions with the screen while in VGA text mode

use core::fmt;
use crate::io::*;
/// # Color
/// In a VGA character cell in VGA text mode, the foreground and background colors are represented with one byte.
///
/// Here are what the bits represent:
///
///|   7                                                       |   6                     |   5                     |   4                     |   3                     |   2                     |   1                   |   0                     |
///| --------------------------------------------------------- | ----------------------- | ----------------------- | ----------------------- | ----------------------- | ----------------------- | --------------------- | ----------------------- |
///|   blink bit (if blinking disabled, 3rd bit of bg color)   |   2nd bit of bg color   |   1st bit of bg color   |   0th bit of bg color   |   3rd bit of fg color   |   2nd bit of fg color   |  1st bit of fg color  |   0th bit of fg color   |
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Color {
    /// Represented as 0x00
    Black = 0x0,
    /// Represented as 0x01
    Blue = 0x1,
    /// Represented as 0x02
    Green = 0x2,
    /// Represented as 0x03
    Cyan = 0x3,
    /// Represented as 0x04
    Red = 0x4,
    /// Represented as 0x05
    Magenta = 0x5,
    /// Represented as 0x06
    Brown = 0x6,
    /// Represented as 0x07
    LightGray = 0x7,

    /// Represented as 0x08
    DarkGray = 0x8,
    /// Represented as 0x09
    LightBlue = 0x9,
    /// Represented as 0x0A
    LightGreen = 0xA,
    /// Represented as 0x0B
    LightCyan = 0xB,
    /// Represented as 0x0C
    LightRed = 0xC,
    /// Represented as 0x0D
    Pink = 0xD,
    /// Represented as 0x0E
    Yellow = 0xE,
    /// Represented as 0x0F
    White = 0xF,
}

/// # VGA Height
/// Default VGA text mode height.
const VGA_HEIGHT: u32 = 25;

/// # VGA Width
/// Default VGA text mode width.
const VGA_WIDTH: u32 = 80;

/// # Color Offset
/// Offset from the character part of a cell to the color part of a cell (1 byte).
const COLOR_OFFSET: u32 = 1;

/// # Cell Width
/// How many bytes wide a character cell is. (2 bytes)
const CELL_WIDTH: u32 = 2;

/// # Last Cursor Position
/// The last character cell index, exclusive.
const LAST_CURSOR_POSITION: u32 = 2000;

/// # Back Buffer
/// Where changes are staged before they are displayed to the screen. The back buffer is 4000 bytes long.
static mut BACK_BUFFER: [u8; 4_000] = [0; 4_000];

/// # Front Buffer
/// The address of the VGA text mode memory mapped IO.
static mut FRONT_BUFFER: *mut u8 = 0xB8000 as *mut u8;

/// # Background Color
/// The current color of the background. Default is black.
static mut BG_COLOR: Color = Color::Black;

/// # Cursor Offset
/// Where the cursor is on the screen. This cursor is not displayed, it is an internal tracker of where to put the next
/// character.
static mut CURSOR_OFFSET: u32 = 0;

/// # VGA Writer
/// A writer instance for the _print method to use, because write_str takes `&self` as an argument, so it needs a struct
/// instance to write to the screen.
static mut VGA_WRITER: VGAWriter = VGAWriter {};

/// # Current Color
/// The current foreground color. Default is white.
static mut CURRENT_COLOR: Color = Color::White;

#[allow(dead_code)]
/// Public API for manipulating the VGA screen
#[doc(hidden)]
pub struct VGAWriter;

/// Puts a character on the screen.
/// # Example
/// ```rust
/// putc('X', vga::Color::Red);
/// VGA::swap_buf();
/// ```
/// This will print a red 'X' character on the screen.
pub fn putc(character: char, color: Color) {
    let cursor_offset = get_cursor_offset();
    let background_color = get_bg_color();

    if cursor_offset >= LAST_CURSOR_POSITION {
        nextln();
        return;
    }
    let offset = (cursor_offset * CELL_WIDTH) as usize;
    let char_byte = character as u8;

    // Bytes 0-3 are foreground color, and bytes 4-7 are background color
    let color_byte = color as u8 | ((background_color as u8) << 4);

    if character == '\n' {
        nextln();
        return;
    }

    // Character byte
    write_back_buffer(offset, char_byte);

    // Color byte
    write_back_buffer(offset + COLOR_OFFSET as usize, color_byte);

    // Offset cursor for next character print
    next_char();
}
/// Puts a character on the screen at a given position. Supports newlines.
/// # Example
/// ```rust
/// putcpos('X', vga::Color::Red, 0, 5);
/// VGA::swap_buf();
/// ```
/// This will print a red 'X' character on the screen at column 0, and row 5 (starting from 0).
pub fn putcpos(character: char, color: Color, column: usize, row: usize) {
    let background_color = get_bg_color();
    let pos = get_pos(column, row);
    let char_byte = character as u8;

    // Bytes 0-3 are foreground color, and bytes 4-7 are background color
    let color_byte = color as u8 | ((background_color as u8) << 4);

    write_back_buffer(pos, char_byte);
    write_back_buffer(pos + COLOR_OFFSET as usize, color_byte);
}
/// Puts a &str on the screen.
/// # Example
/// ```rust
/// VGA::puts("Hello, World!\n", vga::Color::White);
/// VGA::swap_buf();
/// ```
/// This will print "Hello, World" on the screen, and go down a line.
pub fn puts(outstr: &str, color: Color) {
    for character in outstr.as_bytes().iter() {
        let character = *character as char;
        putc(character, color);
    }
}
/// Changes the background color to a specified color.
/// # Example
/// ```rust
/// VGA::set_bgcolor(vga::Color::LightBLue);
/// VGA::swap_buf();
/// ```
/// This will change the background color to Light Blue.
pub fn set_bgcolor(color: Color) {
    set_bg_color(color);

    let background_color = get_bg_color();

    for char_cell in 0..(VGA_HEIGHT * VGA_WIDTH) {
        let char_cell = (char_cell * CELL_WIDTH + COLOR_OFFSET) as usize;
        write_back_buffer(
            char_cell,
            // Deletes background bits, keeps foreground bits, and changes the background bits
            // to our new background
            (read_back_buffer(char_cell) & 0b0000_1111) | ((background_color as u8) << 4),
        );
    }
}
/// Changes the default color used to print text (mainly used for selecting what color the `print!()` macro will
/// use).
/// # Example
/// ```rust
/// VGA::set_text_color(vga::Color::LightBLue);
/// println!("{}", 5.234);
/// ```
/// This will print the float 5.234 on the screen and the text color will be light blue.
pub fn set_text_color(color: Color) {
    set_current_color(color);
}

/// Displays the changes you've made to the screen.
/// # Example
/// ```
/// VGA::puts("Hello", vga::Color::White);
/// VGA::swap_buf();
/// ```
/// This will write the contents of the string to the back buffer, then `VGA::swap_buf()` will display the back
/// buffer to the screen.
pub fn swap_buf() {
    for i in 0..(VGA_HEIGHT * VGA_WIDTH * CELL_WIDTH) {
        let i = i as usize;
        write_front_buffer(i, read_back_buffer(i));
    }
}

/// Disables the VGA visual cursor.
pub fn disable_cursor() {
    outb(0x3D4, 0x0A);
    outb(0x3D5, 0x20);
}

// Big thanks to u/FredFredricson for easy to understand information on toggling text blinking
// https://www.reddit.com/r/osdev/comments/70fcig/blinking_text/
/// Toggles whether the seventh bit in the color byte will represent text blinking or whether it is part of the
/// color.
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
/// Resets the cursor to column 0, row 24.
fn reset_cursor() {
    set_cursor_offset((get_pos(0, (VGA_HEIGHT - 1) as usize) / CELL_WIDTH as usize) as u32);
}
/// Moves the cursor to the next character cell.
fn next_char() {
    let cursor_offset = get_cursor_offset();
    if cursor_offset >= LAST_CURSOR_POSITION {
        nextln();
        return;
    }
    set_cursor_offset(cursor_offset + 1);
}
/// Moves the cursor down a line.
fn nextln() {
    let cursor_offset = get_cursor_offset();

    if cursor_offset >= LAST_CURSOR_POSITION - VGA_WIDTH {
        shift_up();
        reset_cursor();
        return;
    }

    let line_offset = cursor_offset % VGA_WIDTH;
    set_cursor_offset(cursor_offset + (VGA_WIDTH - line_offset));
}
/// Moves the contests from each row to the row above it, and clears the bottom row.
fn shift_up() {
    for column in 0..VGA_WIDTH {
        let column = column as usize;
        for row in 0..(VGA_HEIGHT - 1) {
            let row = row as usize;
            // Write to each line what is below it, for this line of code it copies the below
            // character
            write_back_buffer(
                get_pos(column, row),
                read_back_buffer(get_pos(column, row + 1)),
            );

            // Write to each line what is below it, for this line of code is copies the below
            // color
            write_back_buffer(
                get_pos(column, row) + COLOR_OFFSET as usize,
                read_back_buffer(get_pos(column, row + 1) + COLOR_OFFSET as usize),
            );
        }
    }
    for column in 0..VGA_WIDTH {
        let column = column as usize;
        putcpos('\0', Color::White, column, (VGA_HEIGHT - 1) as usize);
    }
}
/// Write a byte at a given offset in the back buffer.
fn write_back_buffer(index: usize, data: u8) {
    unsafe {
        BACK_BUFFER[index] = data;
    }
}
/// Read a byte at a given offset in the back buffer.
fn read_back_buffer(index: usize) -> u8 {
    unsafe { BACK_BUFFER[index] }
}
/// Update the screen at a given offset.
fn write_front_buffer(index: usize, data: u8) {
    unsafe {
        *FRONT_BUFFER.offset(index as isize) = data;
    }
}
/// Get the memory offset for a certain column and row.
fn get_pos(column: usize, row: usize) -> usize {
    let pos = ((column + (row * VGA_WIDTH as usize)) * CELL_WIDTH as usize) as usize;
    pos
}

/// Get the value of the `CURRENT_COLOR` static variable.
fn get_current_color() -> Color {
    unsafe { CURRENT_COLOR }
}

/// Set the value of the `CURRENT_COLOR` static variable.
fn set_current_color(color: Color) {
    unsafe {
        CURRENT_COLOR = color;
    }
}

/// Get the value of the `BG_COLOR` static variable.
fn get_bg_color() -> Color {
    unsafe { BG_COLOR }
}
/// Set the value of the `BG_COLOR` static variable.
fn set_bg_color(color: Color) {
    unsafe {
        BG_COLOR = color;
    }
}
/// Get the value of the `CURSOR_OFFSET` static variable.
fn get_cursor_offset() -> u32 {
    unsafe { CURSOR_OFFSET }
}
/// Set the value of the `CURSOR_OFFSET` static variable.
fn set_cursor_offset(offset: u32) {
    unsafe {
        CURSOR_OFFSET = offset;
    }
}
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    unsafe {
        VGA_WRITER.write_fmt(args).unwrap();
    }
}
impl fmt::Write for VGAWriter {
    fn write_str(&mut self, outstr: &str) -> fmt::Result {
        puts(outstr, get_current_color());
        swap_buf();
        Ok(())
    }
}

/// # print
/// Print a formatted string to the screen.
/// # Example
/// ```
/// let username = "Bob";
/// let age = 16;
/// print!("Hi, {}, you're {} years old!", username, age);
/// ```
/// This will print "Hi, Bob, you're 16 years old!" to the screen. Works identically to the standard library `print!`
/// macro.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

/// Print a formatted string to the screen with a new line.
/// # Example
/// ```
/// let username = "Bob";
/// let age = 16;
/// println!("Hi, {}, you're {} years old!", username, age);
/// ```
/// This will print "Hi, Bob, you're 16 years old!\n" to the screen. Works identically to the standard library
/// `println!` macro.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
