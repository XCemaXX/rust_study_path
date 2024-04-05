pub fn luhn(cc_number: &str) -> bool {
    let mut sum = 0;
    let mut need_to_double = false;
    let mut counter = 0_usize;
    
    for c in cc_number.chars().filter(|c| *c != ' ').rev() {
        if let Some(digit) = c.to_digit(10) {
            if need_to_double {
                let double_digit = digit * 2;
                sum += if double_digit > 9 { 1 + (double_digit % 10)} else { double_digit };
            } else {
                sum += digit;
            }
            need_to_double = !need_to_double;
            counter += 1;
        } else {
            return false;
        }
    }
    sum % 10 == 0 && counter >= 2
}

fn main() {
    let cc_number = "1234 5678 1234 5670";
    println!(
        "{cc_number} is ok card number? {}",
        if luhn(cc_number) { "Yes" } else { "No" }
    );
}

// Test module. Runs with cmd `cargo test --bin luhn`
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_cc_number() {
        assert!(luhn("4263 9826 4026 9299"));
        assert!(luhn("4539 3195 0343 6467"));
        assert!(luhn("7992 7398 713"));
    }

    #[test]
    fn test_invalid_cc_number() {
        assert!(!luhn("4223 9826 4026 9299"));
        assert!(!luhn("4539 3195 0343 6476"));
        assert!(!luhn("8273 1232 7352 0569"));
    }

    #[test]
    fn test_non_digit_cc_number() {
        assert!(!luhn("foo"));
        assert!(!luhn("foo 0 0"));
    }

    #[test]
    fn test_empty_cc_number() {
        assert!(!luhn(""));
        assert!(!luhn(" "));
        assert!(!luhn("  "));
        assert!(!luhn("    "));
    }

    #[test]
    fn test_single_digit_cc_number() {
        assert!(!luhn("0"));
    }

    #[test]
    fn test_two_digit_cc_number() {
        assert!(luhn(" 0 0 "));
    }
}