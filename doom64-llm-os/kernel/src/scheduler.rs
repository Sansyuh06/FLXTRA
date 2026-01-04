// Simple cooperative scheduler

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Task {
    CLI,
    AIorGame,
}

static mut CURRENT_TASK: Task = Task::CLI;

pub fn init() {
    // Initialize scheduler state
}

pub fn switch_task(to: Task) {
    unsafe {
        CURRENT_TASK = to;
    }
}

pub fn current_task() -> Task {
    unsafe { CURRENT_TASK }
}
