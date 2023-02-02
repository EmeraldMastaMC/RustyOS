use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use lazy_static::lazy_static;

pub const PAGE_SIZE: usize = 4096;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = PAGE_SIZE * 10;

            // The actual stack
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // Where the stack starts: the bottom of the stack
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });

            // Where the stack ends: the top of the stack
            let stack_end = stack_start + STACK_SIZE;

            // The stack starts at the top and goes to the bottom, so we return the top
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector})
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    // Load the GDT
    GDT.0.load();
    unsafe {
        // Load the code selector
        CS::set_reg(GDT.1.code_selector);
        // Load the tss selector
        load_tss(GDT.1.tss_selector);
    }
}