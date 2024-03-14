
// https://habr.com/ru/articles/116236/
// find diff between max and min on any segments with length k

struct Segments {
    k: usize,
    b: Vec<usize>,
    c: Vec<usize>,
    cmp_func: Box<dyn Fn(usize, usize) -> usize>,
}

impl Segments {
    fn gen(v: &Vec<usize>, k: usize, cmp_func: impl Fn(usize, usize) -> usize + 'static) -> Self {
        let n = v.len();
        let mut b = vec![0_usize; n];
        let mut c = vec![0_usize; n];
        let k = k - 1;
        b[0] = v[0];
        for i in 1..n {
            b[i] = if i % k != 0 {
                cmp_func(v[i], b[i-1])
            } else {
                v[i]
            };
        }
        c[n-1] = v[n-1];
        let mut i: usize = n - 2;
        while i > 0 {
            c[i] = if (i + 1) % k != 0 {
                cmp_func(v[i], c[i+1])
            } else {
                v[i]
            };
            i -= 1;
        }
        c[0] = cmp_func(v[0], c[1]);
        Self { k: k + 1, b, c, cmp_func: Box::new(cmp_func) }
    }

    fn new_maxs(v: &Vec<usize>, k: usize) -> Self {
        Self::gen(v, k, std::cmp::max::<usize>)
    }

    fn new_mins(v: &Vec<usize>, k: usize) -> Self {
        Self::gen(v, k, std::cmp::min::<usize>)
    }

    fn get(&self, l: usize) -> usize {
        (self.cmp_func)(self.c[l], self.b[l + self.k - 1])
    }
}

fn main() {
    let (n, k) = (22, 5);
    let v: Vec<usize> = vec![3, 7, 2, 4, 9, 2, 1, 10, 3, 8, 11, 10, 4, 17, 9, 20, 22, 8, 20, 4, 3, 9];
    if k == 1 || n == 1 {
        println!("0");
        return;
    } else if n <= k {
        let min = v.iter().min().unwrap();
        let max = v.iter().max().unwrap();
        println!("{}", max - min);
        return;
    }
    //println!("{v:?}");
    let maxs = Segments::new_maxs(&v, k);
    let mins  = Segments::new_mins(&v, k);

    let mut diff = 0;
    for i in 0..(n - k) {
        let min = mins.get(i);
        let max = maxs.get(i);
        diff = diff.max(max - min);
    }
    println!("{diff}");
}