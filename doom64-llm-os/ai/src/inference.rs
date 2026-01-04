// Tokenization & inference stub

use alloc::vec::Vec;
use alloc::string::String;

pub fn tokenize(prompt: &str) -> Result<Vec<u32>, &'static str> {
    // Stub: simple tokenizer (split by space)
    let tokens: Vec<u32> = prompt
        .split_whitespace()
        .map(|word| {
            let mut hash = 0u32;
            for byte in word.bytes() {
                hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
            }
            hash % 32000 // vocab_size
        })
        .collect();
    
    Ok(tokens)
}

pub fn detokenize(tokens: &[u32]) -> Result<String, &'static str> {
    // Stub: just concat token IDs
    use alloc::string::ToString;
    let result = tokens
        .iter()
        .map(|t| t.to_string())
        .collect::<alloc::vec::Vec<_>>()
        .join(" ");
    Ok(result)
}
