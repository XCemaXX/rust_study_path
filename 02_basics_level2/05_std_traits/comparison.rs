use std::cmp::Ordering;

#[derive(Debug)]
struct Citation {
    author: String,
    year: u32,
}

// full compare: Eq
impl PartialEq for Citation {
    fn eq(&self, other: &Self) -> bool {
        self.author == other.author
    }
}
// with other type
impl PartialEq<&str> for Citation {
    fn eq(&self, other: &&str) -> bool {
        self.author == *other
    }
}

// for <, <=, >= и >
impl PartialOrd for Citation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.author.partial_cmp(&other.author) {
            Some(Ordering::Equal) => self.year.partial_cmp(&other.year),
            author_ord => author_ord,
        }
    }
}

fn main() {
    let k1 = Citation{year: 1, author: "1".to_string()};
    let k2 = Citation{year: 2, author: "2".to_string()};
    let k3 = Citation{year: 3, author: "1".to_string()};
    assert_eq!(k1, k3);
    assert_ne!(k1, k2);
    assert_eq!(k2, "2");
    assert_ne!(k2, "10");

    assert!(k1 < k2);
    assert!(k1 < k3);
}