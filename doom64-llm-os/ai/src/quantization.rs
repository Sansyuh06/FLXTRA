// Q4_K_M quantization format

pub struct Q4KMTensor {
    data: alloc::vec::Vec<u8>,
    shape: (usize, usize),
}

impl Q4KMTensor {
    pub fn dequantize(&self) -> Result<alloc::vec::Vec<f32>, &'static str> {
        // Stub: would implement int4 dequantization
        Ok(alloc::vec::Vec::new())
    }
}
