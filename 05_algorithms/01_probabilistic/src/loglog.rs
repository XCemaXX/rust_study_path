
// HyperLogLog
// https://habr.com/ru/articles/119852/

use std::fs::File;
use std::io::{Read, Result};
pub struct HyperLogLog {
    words: Vec<String>,
    m: u32,
    k_comp: u32,
    alpha_m: f64,
    seed: u32,
    ranks: Vec<u32>,
}

impl HyperLogLog {
    pub fn from_file(file_name: &str, std_err: f64) -> Result<Self> {
        let data = Self::get_data_from_file(file_name)?;
        let words = Self::split_into_words(data);

        let mf = 1.04 / std_err;
        let k = (mf * mf).log2() as u32;
        let m = 2_u32.pow(k);
        let alpha_m = match m {
            16 => 0.673,
            32 => 0.697,
            64 => 0.709,
            _ => 0.7213 / (1.0 + 1.079 / m as f64),
        };

        let seed = rand::random::<u32>();
        Ok(Self { words: words, 
            m: m, k_comp: 32 - k, alpha_m: alpha_m, seed: seed, 
            ranks: vec![0; m as usize]})
    }

    pub fn get_words_len(&self) -> usize {
        self.words.len()
    }

    pub fn count(&mut self) -> usize {
        self.count_words();
        let mut c = 0.0;
        for i in &self.ranks {
            c += 1.0 / (2_u32.pow(*i) as f64);
        }
        let m = self.m as f64;
        let mut e = self.alpha_m * m * m / c;
        // -- make corrections
        let pow_2_32 = (0xFFFFFFFF_u64 + 1) as f64;
        if e <= 5.0/2.0 * m {
            let mut v = 0;
            for i in &self.ranks {
                v += (*i == 0) as i32;
            }
            if v > 0 {
                e = m * (m / v as f64).ln();
            }
        } else if e > 1.0/30.0 * pow_2_32 {
            e = -pow_2_32 * (1.0 - e / pow_2_32).ln();
        } 
        e as usize
    }

    fn count_words(&mut self) {
        for w in &self.words {
            let hash = Self::hash_fnv1a(w) ^ self.seed;
            let j = (hash >> self.k_comp) as usize;
            self.ranks[j] = std::cmp::max(self.ranks[j], Self::get_rank(hash, self.k_comp));
        }
    }

    fn get_data_from_file(file_name: &str) -> Result<String> {
        let mut file = File::open(file_name)?;
        let mut buf = String::new();
        let len = file.read_to_string(&mut buf)?;
        Ok(buf)
    }

    fn split_into_words(data: String) -> Vec<String> {
        let parts = data.split(", ");
        parts.map(|v| v.to_string()).collect()
    }

    fn get_rank(hash: u32, max: u32) -> u32 {
        // pos of first set bit
        let mut hash = hash;
        let mut r = 1_u32;
        while (hash & 1) == 0 && r <= max { 
            r += 1;
            hash >>= 1; 
        }
        r
    }

    fn hash_fnv1a(text: &str) -> u32 {
        let mut hash  = 2166136261_u64;
        for c in text.chars() {
            hash ^= c as u64;
            hash += ((hash << 1) + (hash << 4) + (hash << 7) + (hash << 8) + (hash << 24)) & 0xFFFFFFFF;
        }
        (hash & 0xFFFFFFFF) as u32
    }
}

/*fn get_cur_path() {
    use std::env;
    let path = env::current_dir();
    if path {
        println!("The current directory is {}", path.unwrap().display());
    } else {
        println!("Cannot get path", path.unwrap().display());
    }
    
}*/