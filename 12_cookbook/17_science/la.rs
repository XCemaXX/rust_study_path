use approx::assert_abs_diff_eq;
use nalgebra::{DMatrix, Matrix3};
use ndarray::{Array, Array1, ArrayView1, arr1, arr2, array};

fn matrices() {
    let a = arr2(&[[1, 2, 3], [4, 5, 6]]);
    let b = arr2(&[[6, 5, 4], [3, 2, 100]]);
    let c = arr2(&[[6, 3], [5, 2], [4, 1]]);
    println!("Sum: {}", &a + &b);
    println!("Mul: {}", a.dot(&c));
}

fn scalar_mul() {
    let m = arr2(&[[1, 2, 3], [4, 5, 6]]);
    let v = arr1(&[4, 5, 6]);
    let v: Array1<_> = 4 * v;
    println!("scalar * vector: {}", v);
    println!("matrix * vector: {}", m.dot(&v))
}

fn vec_comparison() {
    let a = Array::from(vec![1., 2., 3.]);
    let b = Array::from(vec![3., 2., 1.]);
    let mut c = Array::from(vec![1., 2., 3.]);
    let d = Array::from(vec![3., 2., 1.]);

    let z = a + b;
    let w = &c + &d;
    assert_abs_diff_eq!(z, Array::from(vec![4., 4., 4.]));
    c[0] = 10.;
    assert_abs_diff_eq!(c, Array::from(vec![10., 2., 3.]));
    assert_abs_diff_eq!(w, Array::from(vec![4., 4., 4.]));
}

fn l1_norm(x: ArrayView1<f64>) -> f64 {
    x.fold(0., |acc, elem| acc + elem.abs())
}

fn l2_norm(x: ArrayView1<f64>) -> f64 {
    x.dot(&x).sqrt()
}

fn normalize(mut x: Array1<f64>) -> Array1<f64> {
    let norm = l2_norm(x.view());
    x.mapv_inplace(|e| e / norm);
    x
}

fn normals() {
    let x = array![1., 2., 3.];
    println!("||x||_2 = {}", l2_norm(x.view()));
    println!("||x||_1 = {}", l1_norm(x.view()));
    println!("Normalizing x yields {:?}", normalize(x));
}

fn inverse() {
    let m = Matrix3::new(2.0, 1.0, 1.0, 3.0, 2.0, 1.0, 2.0, 1.0, 2.0);
    print!("m = {}", m);
    if let Some(inv) = m.try_inverse() {
        print!("The inverse of matrix is: {}", inv);
    } else {
        println!("Matrix is not invertible!");
    }
}

fn to_json() {
    let row_slice: Vec<i32> = (1..5001).collect();
    let matrix = DMatrix::from_row_slice(50, 100, &row_slice);

    let ser = serde_json::to_string(&matrix).unwrap();
    let des: DMatrix<i32> = serde_json::from_str(&ser).unwrap();

    assert_eq!(matrix, des);
}

fn main() {
    matrices();
    scalar_mul();
    vec_comparison();
    normals();
    inverse();
    to_json();
}
