// Input handling

pub struct Input {
    pub move_x: i32,
    pub move_y: i32,
    pub exit_requested: bool,
}

pub fn poll_keyboard() -> Input {
    // Stub: would read from kernel keyboard driver
    Input {
        move_x: 0,
        move_y: 0,
        exit_requested: false,
    }
}
