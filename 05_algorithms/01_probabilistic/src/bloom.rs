//pub mod bloom {
    use bitvec::prelude as bv;

    #[derive(Debug)]
    pub struct Bloom {
        filter: bv::BitVec<u8, bv::Msb0>,
        hash_functions: u64,
    }

    fn djb2_hash(s: &String, modn: u64) -> u64 {
        let mut hash = 5381;
        for c in s.chars() {
            hash = ((hash << 5) + hash) + (c as u64);
            hash = hash % modn;
        }
        hash
    }

    fn hash_with_index(s: &str, n: u64, modn: u64) -> u64 {
        let ns = String::from(s) + &n.to_string();
        djb2_hash(&ns, modn)
    }
    pub trait Interface {
        fn new(expected_elements: u64, size: usize) -> Self;
        fn add_to_filter(&mut self, s: &str);
        fn is_not_in_filter(&self, s: &str) -> bool;
    }
    impl Interface for Bloom {
        fn new(expected_elements: u64, size: usize) -> Self {
            let t = bv::bitvec![u8, bv::Msb0; 0; size]; // bv::Lsb0
            let hash_functions = (((size as f64) / (expected_elements as f64)) * (2_f64).ln()) as u64;
            println!("{hash_functions}");
            Self{filter: t, hash_functions: hash_functions + 1}
        }
        fn add_to_filter(&mut self, s: &str) {
            let fl = self.filter.len() as u64;
            for i in 0..self.hash_functions {
                let h = hash_with_index(s, i, fl) as usize;
                self.filter.set(h, true);
            }
        }
        fn is_not_in_filter(&self, s: &str) -> bool {
            let fl = self.filter.len() as u64;
            for i in 0..self.hash_functions {
                let h = (hash_with_index(s, i, fl) as usize) % self.filter.len();
                if self.filter[h] == false {
                    return true;
                }
            }
            false
        }
    }
//}
