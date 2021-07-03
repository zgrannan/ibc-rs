use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn batch_short_id(b: &Vec<prost_types::Any>) -> u64 {
    let mut hasher = DefaultHasher::new();

    let mut numbers = vec![b.len() as u8];  // lossy, but ok
    let mut others: Vec<u8> = b.iter().take(3).map(|a| a.value.clone()).flatten().collect();
    numbers.append(&mut others);

    Hash::hash_slice(&numbers, &mut hasher);

    hasher.finish()
}