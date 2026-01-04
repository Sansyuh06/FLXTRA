// Global Descriptor Table setup

use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use lazy_static::lazy_static;

lazy_static! {
    static ref GDT: GlobalDescriptorTable = {
        let mut gdt = GlobalDescriptorTable::new();
        gdt.add_entry(Descriptor::user_data_segment());
        gdt.add_entry(Descriptor::kernel_code_segment());
        gdt
    };
}

pub fn init() {
    GDT.load();
}
