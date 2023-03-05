use core::arch::asm;

#[allow(dead_code)]
pub fn outb(port: u16, val: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") val
        );
    }
}

#[allow(dead_code)]
pub fn inb(port: u16) -> u8 {
    unsafe {
        let info: u8;
        asm!(
            "in al, dx",
            in("dx") port,
            out("al") info
        );
        info
    }
}

pub fn outw(port: u16, val: u16) {
    unsafe {
        asm!(
            "out dx, ax",
            in("dx") port,
            in("ax") val
        );
    }
}

pub fn outl(port: u16, val: u32) {
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") val
        );
    }
}
