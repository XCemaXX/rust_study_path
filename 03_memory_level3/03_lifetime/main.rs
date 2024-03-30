#![allow(dead_code)]

#[derive(Debug)]
struct Point(i32, i32);

fn left_most<'a>(p1: &'a Point, p2: &'a Point) -> &'a Point {
    if p1.0 < p2.0 {
        &p1
    } else {
        &p2
    }
}

fn cab_distance(p1: &Point, p2: &Point) -> i32 {
    (p1.0 - p2.0).abs() + (p1.1 - p2.1).abs()
}

//not necessary to put 'q here. Only for example
fn nearest<'a, 'q>(points: &'a [Point], query: &'q Point) -> Option<&'a Point> {
    let mut nearest_p_dist = None;
    for p in points {
        if let Some((_, nearest_dist)) = nearest_p_dist {
            let dist = cab_distance(p, query);
            if dist < nearest_dist {
                nearest_p_dist = Some((p, dist));
            }
        } else {
            nearest_p_dist = Some((p, cab_distance(p, query)));
        }
    }
    nearest_p_dist.map(|(p, _)| p)
}

#[derive(Debug)]
struct Highlight<'lifetime>(&'lifetime str);

fn main() {
    let mut a = [1, 2, 3, 4,5,6];
    let s = &a[2..4];

    let p1 = Point(1, 2);
    let p2 = Point(5, 6);
    let p3 = left_most(&p1, &p2);
    
    println!("Slice: {:?}; Point {:?}", s, p3);
    a[3] = 20; // can change only here

    println!("{:?}",
        nearest(
            &[Point(1, 0), Point(1, 0), Point(-1, 0), Point(0, -1),],
            &Point(0, 2)
        ).unwrap()
    );

    let text = String::from("The quick brown fox jumps over the lazy dog.");
    let h1 = Highlight(&text[4..19]);
    let h2 = Highlight(&text[35..43]);
    let _swallow = |t: String| {println!("Bye {t}!");};
    //_swallow(text); //cannot call because of h1, h2
    println!("{:?}", h1);
    println!("{:?}", h2);
}