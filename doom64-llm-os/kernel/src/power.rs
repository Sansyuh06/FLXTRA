// Power management (ACPI shutdown)

pub fn shutdown() -> ! {
    // Try ACPI S5 shutdown
    // PIIX3 chipset PM1a_CNT at 0x404
    // Write: SLP_TYP_S5 (0x1C00) | SLP_EN (0x2000)
    crate::io::outw(0x404, 0x2000 | 0x1C00);
    
    // Fallback: triple fault
    let idt: u64 = 0;
    unsafe {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) &idt,
        );
        core::arch::asm!("int3");
    }
    
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
