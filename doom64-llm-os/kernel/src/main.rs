#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]

use core::panic::PanicInfo;
use x86_64::instructions::hlt;
use ai::run_inference;

// Internal modules
mod vga;
mod io;
mod power;
mod scheduler;
mod memory;
mod gdt;
mod interrupts;
mod keyboard;

// Multiboot2 Header
#[link_section = ".multiboot_header"]
static HEADER: [u8; 24] = [
    0xd6, 0xe8, 0x03, 0xe8, // Magic: 0xe85250d6 (little endian)
    0x00, 0x00, 0x00, 0x00, // Arch: 0 (i386) protected mode
    0x18, 0x00, 0x00, 0x00, // Header length: 24
    // Checksum: -(magic + arch + length)
    0x12, 0x17, 0xfc, 0x17, // Checksum (0x100000000 - (0xe85250d6 + 0 + 24)) -> 0x17fc1712
    0x08, 0x00, 0x00, 0x00, // End tag type
    0x00, 0x00, 0x00, 0x00, // End tag flags
];

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. Initialize Global Descriptor Table
    gdt::init();

    // 2. Initialize Interrupts
    interrupts::init();
    
    // 3. Initialize Memory (Heap)
    memory::init_paging();
    memory::init_heap();

    // 4. Initialize Peripherals
    vga::init();
    vga::clear_screen();
    vga::write_string("DoomLLM OS Kernel Initialized.\n");
    
    keyboard::init();
    scheduler::init();
    
    // Enable Interrupts
    x86_64::instructions::interrupts::enable();
    
    vga::write_string("System Ready.\n");
    
    // Main CLI Loop
    cli_main_loop();
    
    // Should not reach here
    power::shutdown();
}

fn cli_main_loop() {
    vga::write_string("\nDOOM OS v0.1\nCommands: /ai <prompt>, /game, /off, help\n\n> ");
    
    let mut buffer = [0u8; 128];
    let mut cursor = 0;
    
    loop {
        // Blocks until key press
        let char = keyboard::read_char();
        
        if char == '\n' {
            vga::write_char('\n');
            // Process command
            if cursor > 0 {
                let cmd_str = core::str::from_utf8(&buffer[0..cursor]).unwrap_or("");
                process_command(cmd_str);
            }
            cursor = 0;
            vga::write_string("> ");
        } else if char == '\x08' { // Backspace
            if cursor > 0 {
                cursor -= 1;
                vga::write_string("\x08 \x08"); // Visual backspace
            }
        } else if cursor < buffer.len() {
            buffer[cursor] = char as u8;
            cursor += 1;
            vga::write_char(char);
        }
    }
}

fn process_command(cmd: &str) {
    if cmd == "/off" {
        vga::write_string("Shutting down...\n");
        power::shutdown();
    } else if cmd == "/game" {
        vga::write_string("Starting Doom64...\n");
        game::doom64_run();
        vga::clear_screen(); // Restore text mode cleanup
        vga::write_string("Exited Game.\n> ");
    } else if cmd.starts_with("/ai ") {
        let prompt = &cmd[4..];
        vga::write_string("Thinking...\n");
        
        // Call AI Module
        match run_inference(prompt, 128) {
            Ok(response) => {
                vga::write_string("AI: ");
                vga::write_string(&response);
                vga::write_char('\n');
            },
            Err(e) => {
                vga::write_string("AI Error: ");
                vga::write_string(e);
                vga::write_char('\n');
            }
        }
    } else if cmd == "help" {
        vga::write_string("Available commands:\n  /ai <prompt>\n  /game\n  /off\n");
    } else {
        vga::write_string("Unknown command.\n");
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::write_string("\n\nKERNEL PANIC:\n");
    let _msg = info.message();
    vga::write_string("Critical Error Occurred.\n");
    
    loop {
        hlt();
    }
}
