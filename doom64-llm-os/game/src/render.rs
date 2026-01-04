// Rendering (VGA mode 13h for MVP)

pub fn init_graphics() {
    set_vga_mode_13h();
    clear_screen();
}

pub fn draw_frame() {
    // Stub: render test pattern
    unsafe {
        let vram = 0xA0000 as *mut u8;
        for y in 0..200 {
            for x in 0..320 {
                let idx = (y * 320 + x) as usize;
                *vram.add(idx) = ((x + y) % 256) as u8;
            }
        }
    }
}

pub fn shutdown_graphics() {
    set_vga_text_mode();
}

fn set_vga_mode_13h() {
    unsafe {
        core::arch::asm!(
            "int 0x10",
            in("ax") 0x0013u16,
        );
    }
}

fn set_vga_text_mode() {
    unsafe {
        core::arch::asm!(
            "int 0x10",
            in("ax") 0x0003u16,
        );
    }
}

fn clear_screen() {
    unsafe {
        let vram = 0xA0000 as *mut u8;
        for i in 0..(320 * 200) {
            *vram.add(i) = 0;
        }
    }
}
