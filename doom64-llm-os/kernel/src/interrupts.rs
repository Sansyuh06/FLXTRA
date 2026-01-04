// IDT & PIC setup

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(0);
        }
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_fault_handler);
        idt.general_protection_fault.set_handler_fn(gp_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(fpu_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_handler);
        idt
    };
}

static mut CUSTOM_HANDLERS: [Option<extern "x86-interrupt" fn(InterruptStackFrame)>; 256] =
    [None; 256];

pub fn init() {
    IDT.load();
    
    // Initialize PIC (8259A)
    // ICW1: Start initialization
    crate::io::outb(0x20, 0x11);
    crate::io::outb(0xA0, 0x11);
    
    // ICW2: Set vector base (master=32, slave=40)
    crate::io::outb(0x21, 32);
    crate::io::outb(0xA1, 40);
    
    // ICW3: Cascade (master pin 2 = slave)
    crate::io::outb(0x21, 4);
    crate::io::outb(0xA1, 2);
    
    // ICW4: 8086 mode
    crate::io::outb(0x21, 1);
    crate::io::outb(0xA1, 1);
    
    // OCW1: Unmask all IRQs (will enable keyboard IRQ1 later)
    crate::io::outb(0x21, 0xFF);
    crate::io::outb(0xA1, 0xFF);
}

pub fn register_handler(irq: u8, handler: extern "x86-interrupt" fn(InterruptStackFrame)) {
    unsafe {
        CUSTOM_HANDLERS[(32 + irq) as usize] = Some(handler);
    }
}

// Exception handlers
extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    panic!("DIVIDE BY ZERO\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    crate::vga::write_string("[!] DEBUG exception\n");
}

extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    crate::vga::write_string("[!] NMI\n");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::vga::write_string("[!] BREAKPOINT\n");
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    panic!("OVERFLOW\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn bound_exceeded_handler(stack_frame: InterruptStackFrame) {
    panic!("BOUND EXCEEDED\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("INVALID OPCODE\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    panic!("DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _code: u64) -> ! {
    panic!("DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, _code: u64) {
    panic!("INVALID TSS\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, _code: u64) {
    panic!("SEGMENT NOT PRESENT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn stack_fault_handler(stack_frame: InterruptStackFrame, _code: u64) {
    panic!("STACK FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn gp_fault_handler(stack_frame: InterruptStackFrame, code: u64) {
    panic!("GENERAL PROTECTION FAULT: {:#x}\n{:#?}", code, stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!("PAGE FAULT: {:#?}\n{:#?}", error_code, stack_frame);
}

extern "x86-interrupt" fn fpu_handler(stack_frame: InterruptStackFrame) {
    panic!("X87 FPU\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, _code: u64) {
    panic!("ALIGNMENT CHECK\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("MACHINE CHECK\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn simd_handler(stack_frame: InterruptStackFrame) {
    panic!("SIMD FLOATING POINT\n{:#?}", stack_frame);
}
