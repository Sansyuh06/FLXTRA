// GPU/CPU compute kernels (stub)
use alloc::vec::Vec;

pub fn matrix_multiply_q4(_a: &[u8], _b: &[u8]) -> Result<Vec<f32>, &'static str> {
    // Stub: would implement Q4 dequant + matmul
    Ok(alloc::vec::Vec::new())
}
