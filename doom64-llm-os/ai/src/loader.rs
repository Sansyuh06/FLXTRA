// GGUF model loader stub

use alloc::vec::Vec;

pub struct Model {
    vocab_size: u32,
    hidden_size: u32,
    num_layers: u32,
}

static mut MODEL_CACHE: Option<Model> = None;

pub fn load_model_from_disk(_path: &str) -> Result<(), &'static str> {
    // Stub: simulate loading
    unsafe {
        MODEL_CACHE = Some(Model {
            vocab_size: 32000,
            hidden_size: 2048,
            num_layers: 32,
        });
    }
    Ok(())
}

pub fn get_model_cached() -> Result<&'static Model, &'static str> {
    unsafe {
        MODEL_CACHE.as_ref().ok_or("Model not loaded")
    }
}
