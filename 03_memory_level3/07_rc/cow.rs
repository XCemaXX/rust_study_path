use std::{borrow::Cow, str::FromStr};

fn to_lowercase<'a>(s: &'a str) -> Cow<'a, str> {
    if s.chars().all(char::is_lowercase) {
        println!{"return without allocation link to existing string: {s}"};
        Cow::Borrowed(s)
    } else {
        let low = s.to_lowercase();
        println!{"return new owned allocation: {low}"};
        Cow::Owned(low)
    }
}

fn main() {
    let low = "lowercase".to_string();
    let high = "UPPERcase".to_string();
    let low_res = to_lowercase(&low);
    println!("{low_res}");
    let high_res = to_lowercase(&high);
    println!("{high_res}");
}


