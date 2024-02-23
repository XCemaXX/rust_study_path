fn transpose(matrix: [[i32; 3]; 3]) -> [[i32; 3]; 3] {
    const SIZE : usize = 3; // matrix.len();
    let mut transposed = [[0; SIZE]; SIZE];
    for i in 0..SIZE {
        for j in 0..SIZE {
            transposed[j][i] = matrix[i][j];
        }
    }
    transposed
}

fn main() {
    let matrix = [
        [101, 102, 103], // <-- comment forbids format matrix in one line
        [201, 202, 203],
        [301, 302, 303],
    ];
    let transposed = transpose(matrix);
    println!("transposed matrix: {:#?}", transposed);
    assert_eq!(
        transposed,
        [
            [101, 201, 301], //
            [102, 202, 302],
            [103, 203, 303],
        ]
    );
    println!("Matrix by rows");
    for row in matrix {
        println!("{:#?}", row);
    }
}