#![no_std]
extern crate alloc;

pub mod render;
pub mod input;
pub mod wad_loader;
pub mod engine;

pub fn doom64_run() {
    // Initialize graphics
    render::init_graphics();
    
    // Load WAD
    let _wad = wad_loader::load_default();
    
    // Main game loop
    game_loop();
    
    // Shutdown
    render::shutdown_graphics();
}

fn game_loop() {
    let mut running = true;
    
    while running {
        // Poll input
        let input = input::poll_keyboard();
        
        if input.exit_requested {
            running = false;
        }
        
        // Update game state
        engine::update(&input);
        
        // Render
        render::draw_frame();
    }
}
