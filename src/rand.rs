#[macro_export]
macro_rules! rdseed {
    () => (rdseed!(usize));
    (u8) => {{
        let outnum: u16;
        unsafe {
            core::arch::asm!(
                "rdseed ax",
                out("ax") outnum
            );
        }
        (outnum >> 8) as u8
    }};
    (u16) => {{
        let outnum: u16;
        unsafe {
            core::arch::asm!(
                "rdseed ax",
                out("ax") outnum
            );
        }
        outnum
    }};
    (u32) => {{
        let outnum: u32;
        unsafe {
            core::arch::asm!(
                "rdseed eax",
                out("eax") outnum
            );
        }
        outnum
    }};
    (u64) => {{
        let outnum: u64;
        unsafe {
            core::arch::asm!(
                "rdseed rax",
                out("rax") outnum
            );
        }
        outnum
    }};
    (usize) => {{
        let outnum: usize;
        unsafe {
            core::arch::asm!(
                "rdseed rax",
                out("rax") outnum
            );
        }
        outnum
    }};
}

#[macro_export]
macro_rules! rdrand {
    () => (rdrand!(usize));
    (u8) => {{
        let outnum: u16;
        unsafe {
            core::arch::asm!(
                "rdrand ax",
                out("ax") outnum
            );
        }
        (outnum >> 8) as u8
    }};
    (u16) => {{
        let outnum: u16;
        unsafe {
            core::arch::asm!(
                "rdrand ax",
                out("ax") outnum
            );
        }
        outnum
    }};
    (u32) => {{
        let outnum: u32;
        unsafe {
            core::arch::asm!(
                "rdrand eax",
                out("eax") outnum
            );
        }
        outnum
    }};
    (u64) => {{
        let outnum: u64;
        unsafe {
            core::arch::asm!(
                "rdrand rax",
                out("rax") outnum
            );
        }
        outnum
    }};
    (usize) => {{
        let outnum: usize;
        unsafe {
            core::arch::asm!(
                "rdrand rax",
                out("rax") outnum
            );
        }
        outnum
    }};
}
pub fn rand_float() -> f64 {
    const MAX_64_BIT_VALUE: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    let random_int = rdseed!(u64);
    if random_int == 0 {
        return 0.0;
    } else {
        return (random_int - 1) as f64 / MAX_64_BIT_VALUE as f64;
    }
}
