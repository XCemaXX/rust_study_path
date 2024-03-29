// Jaccard coef = interseption of set / union of sets
// https://habr.com/ru/articles/115147/
#![allow(dead_code)]
pub struct MinHash {
    functions: Vec<Box<dyn Fn(&str) -> u32>>,
}

fn gen_hash_fun(size: u64) -> impl Fn(&str) -> u32 {
    let seed = ((rand::random::<u32>() as u64) * size + 32) & 0xFFFFFFFF;
    move |s| { 
        let mut res = 1;
        for c in s.chars() {
            res = (seed * res + c as u64) & 0xFFFFFFFF;
        }
        res as u32
    }
}

impl MinHash {
    pub fn new(hashes_count : u32) -> Self {
        let mut t: Vec<Box<dyn Fn(&str) -> u32>> = vec![];
        for _i in 0..hashes_count {
            t.push(Box::new(gen_hash_fun(hashes_count as u64)));
        }
        Self {functions: t}
    }

    fn find_min(set: impl IntoIterator<Item = impl AsRef<str>>, hash_func: &Box<dyn Fn(&str) -> u32>) -> u32
    {
        let mut min: u32 = 0xFFFFFFFF;
        for i in set {
            let hash = hash_func(i.as_ref());
            min = std::cmp::min(hash, min);
        }
        min
    }

    fn get_signature(&self, set: impl IntoIterator<Item = impl AsRef<str>> + std::marker::Copy) -> Vec<u32>
    {
        let mut res = vec![0; self.functions.len()];
        for (i, f) in self.functions.iter().enumerate()  {
            res[i] = Self::find_min(set, &f);
        }
        res
    }

    pub fn similarity<T: IntoIterator<Item = impl AsRef<str>> + std::marker::Copy>(&self, set_a: T, set_b: T) -> f64
    {
        let mut equal_count = 0_u32;
        let sig_a = self.get_signature(set_a);
        let sig_b = self.get_signature(set_b);
        let function_count = self.functions.len();
        for i in 0..function_count {
            equal_count += (sig_a[i] == sig_b[i]) as u32;
        }
        equal_count as f64 / function_count as f64
    }
}