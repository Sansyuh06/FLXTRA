// PS/2 keyboard driver (Intel i8042 controller)
// IRQ1, PS/2 scancodes → ASCII

use core::sync::atomic::{AtomicUsize, Ordering};

static mut CHAR_BUFFER: [u8; 256] = [0u8; 256];
static mut KEY_STATE: [bool; 256] = [false; 256];
static BUF_HEAD: AtomicUsize = AtomicUsize::new(0);
static BUF_TAIL: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    // Register IRQ1 handler in PIC
    crate::interrupts::register_handler(1, keyboard_irq_handler);
    
    // Unmask IRQ1 in PIC (master, pin 1)
    let mut mask = crate::io::inb(0x21);
    mask &= !0x02; // Clear bit 1
    crate::io::outb(0x21, mask);
}

pub fn read_char() -> char {
    loop {
        let head = BUF_HEAD.load(Ordering::Acquire);
        let tail = BUF_TAIL.load(Ordering::Acquire);
        if head != tail {
            unsafe {
                let ch = CHAR_BUFFER[tail] as char;
                BUF_TAIL.store((tail + 1) % 256, Ordering::Release);
                return ch;
            }
        }
        // Halt CPU while waiting
        x86_64::instructions::hlt();
    }
}

pub fn is_key_pressed(scancode: u8) -> bool {
    unsafe { KEY_STATE[scancode as usize] }
}

pub fn is_key_held(scancode: u8) -> bool {
    unsafe { KEY_STATE[scancode as usize] }
}

extern "x86-interrupt" fn keyboard_irq_handler(_: x86_64::structures::idt::InterruptStackFrame) {
    let scancode = crate::io::inb(0x60);
    
    // Track key state (0x80 = key release)
    let is_pressed = (scancode & 0x80) == 0;
    let code = scancode & 0x7F;
    
    unsafe {
        KEY_STATE[code as usize] = is_pressed;
    }
    
    // Convert to ASCII for press events
    if is_pressed {
        if let Some(ch) = scancode_to_ascii(code) {
            let head = BUF_HEAD.load(Ordering::Acquire);
            unsafe {
                CHAR_BUFFER[head] = ch as u8;
                BUF_HEAD.store((head + 1) % 256, Ordering::Release);
            }
        }
    }
    
    // Send EOI to PIC
    crate::io::outb(0x20, 0x20);
}

fn scancode_to_ascii(code: u8) -> Option<char> {
    // US QWERTY scancodes (make codes only)
    match code {
        0x02 => Some('1'), 0x03 => Some('2'), 0x04 => Some('3'), 0x05 => Some('4'),
        0x06 => Some('5'), 0x07 => Some('6'), 0x08 => Some('7'), 0x09 => Some('8'),
        0x0A => Some('9'), 0x0B => Some('0'),
        0x0C => Some('-'), 0x0D => Some('='), 0x0E => Some('\x08'), // backspace
        0x0F => Some('\t'),
        0x10 => Some('q'), 0x11 => Some('w'), 0x12 => Some('e'), 0x13 => Some('r'),
        0x14 => Some('t'), 0x15 => Some('y'), 0x16 => Some('u'), 0x17 => Some('i'),
        0x18 => Some('o'), 0x19 => Some('p'), 0x1A => Some('['), 0x1B => Some(']'),
        0x1C => Some('\n'),
        0x1E => Some('a'), 0x1F => Some('s'), 0x20 => Some('d'), 0x21 => Some('f'),
        0x22 => Some('g'), 0x23 => Some('h'), 0x24 => Some('j'), 0x25 => Some('k'),
        0x26 => Some('l'), 0x27 => Some(';'), 0x28 => Some('\''), 0x29 => Some('`'),
        0x2B => Some('\\'),
        0x2C => Some('z'), 0x2D => Some('x'), 0x2E => Some('c'), 0x2F => Some('v'),
        0x30 => Some('b'), 0x31 => Some('n'), 0x32 => Some('m'), 0x33 => Some(','),
        0x34 => Some('.'), 0x35 => Some('/'),
        0x39 => Some(' '),
        _ => None,
    }
}
