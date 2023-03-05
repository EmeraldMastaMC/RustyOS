//! # VGA
//! This module allows interactions with the screen while in VGA text mode

/// # Color
/// In a VGA character cell in VGA text mode, the foreground and background colors are represented with one byte.
///
/// Here are what the bits represent:
///
///|   7                                                       |   6                     |   5                     |   4                     |   3                     |   2                     |   1                   |   0                     |
///| --------------------------------------------------------- | ----------------------- | ----------------------- | ----------------------- | ----------------------- | ----------------------- | --------------------- | ----------------------- |
///|   blink bit (if blinking disabled, 3rd bit of bg color)   |   2nd bit of bg color   |   1st bit of bg color   |   0th bit of bg color   |   3rd bit of fg color   |   2nd bit of fg color   |  1st bit of fg color  |   0th bit of fg color   |
use crate::io::{inb, outb};
#[derive(Clone, Copy)]
#[allow(dead_code)]
#[repr(u8)]
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

type ColorByte = u8;

trait FromColors {
    fn from_colors(background: Color, foreground: Color) -> ColorByte;
}

impl FromColors for ColorByte {
    fn from_colors(background: Color, foreground: Color) -> ColorByte {
        ((background as u8) << 4) | foreground as u8
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CharCell {
    character: u8,
    color: ColorByte,
}

impl CharCell {
    fn new(character: u8, color: u8) -> Self {
        Self { character, color }
    }
}

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const START_POS: usize = 1920;
const CELL_WIDTH: usize = 2;
const START_OF_LAST_LINE: usize = 1920;
const VIDEO_MEMORY_SIZE: usize = VGA_WIDTH * VGA_HEIGHT;
const LAST_POS: usize = VIDEO_MEMORY_SIZE - 1;
const VIDEO_MEMORY: usize = 0xB8000;

static mut FRONT_MIRROR: [CharCell; VIDEO_MEMORY_SIZE] = [CharCell {
    character: b' ',
    color: 0x0F,
}; VIDEO_MEMORY_SIZE];

static mut BACK_BUFFER: [Option<CharCell>; VIDEO_MEMORY_SIZE] = [None; VIDEO_MEMORY_SIZE];
static mut CURSOR: usize = START_POS;
static mut BACKGROUND_COLOR: Color = Color::Black;
static mut FOREGROUND_COLOR: Color = Color::White;
static mut WRITER: spin::Mutex<Writer> = spin::Mutex::new(Writer {});

struct Writer();
impl core::fmt::Write for Writer {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        puts(string.as_bytes(), get_fgcolor());
        Ok(())
    }
}
pub fn putc(character: u8, color: Color) {
    if get_cursor() == VIDEO_MEMORY_SIZE {
        scroll();
    }
    if character == b'\n' {
        newline();
        return;
    }
    write_backbuf(
        Some(CharCell::new(
            character,
            ColorByte::from_colors(get_bgcolor(), color),
        )),
        get_cursor(),
    );
    next();
}

pub fn puts(string: &[u8], color: Color) {
    for character in string {
        putc(*character, color);
    }
}

pub fn update() {
    for offset in 0..VIDEO_MEMORY_SIZE {
        match get_backbuf(offset) {
            Some(character) => write_frontbuf(character, offset),
            None => continue,
        }
    }
    wipe_backbuf();
}

fn next() {
    if get_cursor() > LAST_POS {
        scroll();
    }
    next_cell();
}

fn next_cell() {
    set_cursor(get_cursor() + 1);
}

fn scroll() {
    mirror_to_backbuf();
    for offset in VGA_WIDTH..VIDEO_MEMORY_SIZE {
        write_backbuf(get_backbuf(offset), offset - VGA_WIDTH);
    }
    for offset in START_OF_LAST_LINE..VIDEO_MEMORY_SIZE {
        write_backbuf(
            Some(CharCell::new(
                b' ',
                ColorByte::from_colors(get_bgcolor(), Color::White),
            )),
            offset,
        );
    }
    set_cursor(START_OF_LAST_LINE);
}

fn newline() {
    let cursor = get_cursor();
    if START_OF_LAST_LINE <= cursor && cursor <= LAST_POS {
        scroll();
        return;
    }
    set_cursor(cursor - (cursor % VGA_WIDTH) + VGA_WIDTH);
}
fn mirror_to_backbuf() {
    for offset in 0..LAST_POS {
        match get_backbuf(offset) {
            Some(_) => {}
            None => write_backbuf(Some(get_frontbuf(offset)), offset),
        }
    }
}
fn get_cursor() -> usize {
    unsafe { CURSOR }
}
fn set_cursor(new_cursor: usize) {
    unsafe { CURSOR = new_cursor }
}
fn wipe_backbuf() {
    for offset in 0..VIDEO_MEMORY_SIZE {
        unsafe {
            BACK_BUFFER[offset] = None;
        }
    }
}

fn write_backbuf(character: Option<CharCell>, offset: usize) {
    unsafe {
        BACK_BUFFER[offset] = character;
    }
}

fn get_backbuf(offset: usize) -> Option<CharCell> {
    unsafe { BACK_BUFFER[offset] }
}

fn write_frontbuf(character: CharCell, offset: usize) {
    unsafe {
        let print_location = (VIDEO_MEMORY + offset * CELL_WIDTH) as *mut CharCell;
        FRONT_MIRROR[offset] = character;
        *print_location = character;
    }
}

fn get_frontbuf(offset: usize) -> CharCell {
    unsafe { FRONT_MIRROR[offset] }
}

pub fn set_bgcolor(color: Color) {
    unsafe {
        BACKGROUND_COLOR = color;
    }
}

fn get_bgcolor() -> Color {
    unsafe { BACKGROUND_COLOR }
}

pub fn set_fgcolor(color: Color) {
    unsafe {
        FOREGROUND_COLOR = color;
    }
}

fn get_fgcolor() -> Color {
    unsafe { FOREGROUND_COLOR }
}

pub fn clear() {
    for offset in 0..VIDEO_MEMORY_SIZE {
        write_backbuf(
            Some(CharCell::new(
                b' ',
                ColorByte::from_colors(get_bgcolor(), get_fgcolor()),
            )),
            offset,
        )
    }
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
    attribute_mode ^= 0b0000_1000;

    // Send attribute mode to port 0x03C0 to toggle the blinking
    outb(0x03C0, attribute_mode);
}

pub fn disable_cursor() {
    outb(0x3D4, 0x0A);
    outb(0x3D5, 0x20);
}

pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    unsafe {
        WRITER.lock().write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::display::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };

    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
