mod bloom;
mod min_hash;
mod loglog;

fn main() {

}

// run example: cargo test minhash -p probabilistic -- --show-output

#[cfg(test)]
mod tests {
    #[test]
    fn bloom() {
        use crate::bloom::*;
        let mut bloom_filter = Bloom::new(1000000, 100000);
        let base_ip = String::from("192.168.1.");
        bloom_filter.add_to_filter(&(base_ip.clone() + "1"));
        let mut counter_check = 0;
        for i in 1..100000 {
            let ip = base_ip.clone() + &i.to_string();
            if !bloom_filter.is_not_in_filter(&ip) {
                counter_check += 1;
                println!("{ip}");
            }
        }
        assert!(counter_check > 0);
    }
    #[test]
    fn bloom2() {
        use crate::bloom::*;
        let mut bloom_filter = Bloom::new(256*256*256*256, 10000000);
        let base_ip = String::from("192.168.1.");
        bloom_filter.add_to_filter(&(base_ip.clone() + "1"));
        let mut counter_check = 0;
        //for i1 in 1..256 { //too match
            for i2 in 1..64 {
                for i3 in 1..256 {
                    for i4 in 1..256 {
                        let ip = format!("1.{i2}.{i3}.{i4}");
                        if !bloom_filter.is_not_in_filter(&ip) {
                            counter_check += 1;
                        }
                    }
                }
        //    }
        }
        assert!(counter_check > 0);
    }

    #[test]
    fn minhash() {
        use crate::min_hash::*;
        let min_hash = MinHash::new(100);
        let set_a = vec!["apple", "orange", "lol", "god", "1", "2"];
        let set_sim = vec!["apple", "peach", "lol", "god"];
        let set_not_sim = vec!["1", "45", "34", "54"];
        let sim_more = min_hash.similarity(&set_a, &set_sim);
        let sim_less = min_hash.similarity(&set_a, &set_not_sim);
        println!("{sim_more}");
        println!("{sim_less}");
        assert!(sim_more>sim_less);
    }

    #[test]
    fn loglog() {
        use crate::loglog::*;
        
        let path = "loglog_data.txt";
        let hp_res = HyperLogLog::from_file(path, 0.065);
        assert!(!hp_res.is_err());
        let mut hp = hp_res.unwrap();
        let count = hp.count() as i64;
        let words_len = hp.get_words_len() as i64;
        let error = (count - words_len) as f64 / (words_len as f64 / 100.0);
        println!("Unique elems {count} from {words_len}, error {error:.2}%");
        assert!(error.abs() < 13.0_f64)
    }
}