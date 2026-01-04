// 80x25 text mode driver
// VGA memory: 0xB8000 (80*25*2 bytes)
// Each cell: 1 byte ASCII + 1 byte color (bg:4 | fg:4)

pub fn init() {
    set_cursor(0, 0);
}

pub fn clear_screen() {
    unsafe {
        let buffer = 0xB8000 as *mut u16;
        for i in 0..(80 * 25) {
            *buffer.add(i) = 0x0F20; // white on black, space
        }
    }
    set_cursor(0, 0);
}

pub fn write_string(s: &str) {
    for ch in s.chars() {
        write_char(ch);
    }
}

pub fn write_char(ch: char) {
    static mut ROW: usize = 0;
    static mut COL: usize = 0;

    unsafe {
        match ch {
            '\n' => {
                ROW += 1;
                COL = 0;
                if ROW >= 25 {
                    ROW = 24;
                    scroll_up();
                }
            }
            '\x08' => {
                // Backspace
                if COL > 0 {
                    COL -= 1;
                } else if ROW > 0 {
                    ROW -= 1;
                    COL = 79;
                }
                let pos = ROW * 80 + COL;
                let buffer = 0xB8000 as *mut u16;
                *buffer.add(pos) = 0x0F20;
            }
            '\t' => {
                // Tab (4 spaces)
                for _ in 0..4 {
                    write_char(' ');
                }
            }
            c if c.is_ascii() && c != '\r' => {
                let pos = ROW * 80 + COL;
                let buffer = 0xB8000 as *mut u16;
                *buffer.add(pos) = 0x0F00 | (c as u16);
                COL += 1;
                if COL >= 80 {
                    COL = 0;
                    ROW += 1;
                    if ROW >= 25 {
                        ROW = 24;
                        scroll_up();
                    }
                }
            }
            _ => {
                // Non-ASCII: print ?
                let pos = ROW * 80 + COL;
                let buffer = 0xB8000 as *mut u16;
                *buffer.add(pos) = 0x0F3F; // ?
                COL += 1;
                if COL >= 80 {
                    COL = 0;
                    ROW += 1;
                    if ROW >= 25 {
                        ROW = 24;
                        scroll_up();
                    }
                }
            }
        }
        set_cursor(ROW, COL);
    }
}

fn scroll_up() {
    unsafe {
        let buffer = 0xB8000 as *mut u16;
        for i in 0..(24 * 80) {
            *buffer.add(i) = *buffer.add(i + 80);
        }
        for i in (24 * 80)..(25 * 80) {
            *buffer.add(i) = 0x0F20;
        }
    }
}

fn set_cursor(row: usize, col: usize) {
    let pos = (row * 80 + col) as u16;
    crate::io::outb(0x3D4, 0x0F);
    crate::io::outb(0x3D5, (pos & 0xFF) as u8);
    crate::io::outb(0x3D4, 0x0E);
    crate::io::outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
}
