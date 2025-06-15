use std::{borrow::Cow, collections::HashSet, fmt};

use regex::{Regex, RegexSetBuilder};

fn extract_login(input: &str) -> Option<&str> {
    let re = Regex::new(
        r"(?x)
        ^(?P<login>[^@\s]+)@
            ([[:word:]]+\.)*
            [[:word:]]+$
            ",
    )
    .unwrap();

    re.captures(input)
        .and_then(|cap| cap.name("login").map(|login| login.as_str()))
}

fn extract_hashtags(text: &str) -> HashSet<&str> {
    let re = Regex::new(r"\#[a-zA-Z][0-9a-zA-Z_]*").unwrap();
    re.find_iter(text).map(|mat| mat.as_str()).collect()
}

struct PhoneNumber<'a> {
    area: &'a str,
    exchange: &'a str,
    subscriber: &'a str,
}

impl<'a> fmt::Display for PhoneNumber<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "1 ({}) {}-{}", self.area, self.exchange, self.subscriber)
    }
}

fn extract_phones() {
    let phone_text = "
    +1 505 881 9292 (v) +1 505 778 2212 (c) +1 505 881 9297 (f)
    (202) 991 9534
    Alex 5553920011
    1 (800) 233-2010
    1.299.339.1020";

    let re = Regex::new(
        r#"(?x)
          (?:\+?1)?                       # Country Code Optional
          [\s\.]?
          (([2-9]\d{2})|\(([2-9]\d{2})\)) # Area Code
          [\s\.\-]?
          ([2-9]\d{2})                    # Exchange Code
          [\s\.\-]?
          (\d{4})                         # Subscriber Number"#,
    )
    .unwrap();

    let numbers = re.captures_iter(phone_text).filter_map(|cap| {
        let groups = (cap.get(2).or(cap.get(3)), cap.get(4), cap.get(5));
        if let (Some(area), Some(ext), Some(sub)) = groups {
            Some(PhoneNumber {
                area: area.as_str(),
                exchange: ext.as_str(),
                subscriber: sub.as_str(),
            })
        } else {
            None
        }
    });
    assert_eq!(
        numbers.map(|m| m.to_string()).collect::<Vec<_>>(),
        vec![
            "1 (505) 881-9292",
            "1 (505) 778-2212",
            "1 (505) 881-9297",
            "1 (202) 991-9534",
            "1 (555) 392-0011",
            "1 (800) 233-2010",
            "1 (299) 339-1020",
        ]
    );
}

fn match_several_regexes() {
    let text = "
        version \"1.2.3\" line1
        line2 with ip 192.168.0.1:443
        line3 with Warning: timeout expired
        line 4 fake
        line 5 version 1
        line 6 with warning";

    let set = RegexSetBuilder::new(&[
        r#"version "\d\.\d\.\d""#,
        r#"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:443"#,
        r#"warning.*timeout expired"#,
    ])
    .case_insensitive(true)
    .build()
    .unwrap();

    text.lines()
        .filter(|line| set.is_match(line))
        .for_each(|l| println!("{l}"));
}

fn replace(dates: &str) -> Cow<str> {
    let re = Regex::new(r"(?P<y>\d{4})-(?P<m>\d{2})-(?P<d>\d{2})").unwrap();
    re.replace_all(dates, "$m/$d/$y")
}

fn main() {
    assert_eq!(extract_login(r"my_email@domain.com"), Some(r"my_email"));
    assert_eq!(extract_login(r"fake@email@addr.com"), None);

    let tweet = "Hey #world, I just got my new #dog, say hello to Till. #dog #forever #2 #_ ";
    let tags = extract_hashtags(tweet);
    assert!(tags.contains("#dog") && tags.contains("#forever") && tags.contains("#world"));
    assert_eq!(tags.len(), 3);

    extract_phones();
    match_several_regexes();

    let before = "2012-03-14, 2013-01-15 and 2014-07-05";
    let after = replace(before);
    assert_eq!(after, "03/14/2012, 01/15/2013 and 07/05/2014");
}
