use std::cell::{Cell, OnceCell, Ref, RefCell};

struct Holder {
    data: Vec<i32>
}

fn mut_cells(cells: &[Cell<i32>]) {
    for c in cells {
        let temp = c.get(); // copy
        c.set(temp * 10);
    }
}

fn mut_cell(cell: &Cell<i32>) {
    cell.set(cell.get() + 100);
}

fn mut_vec(v: &RefCell<Vec<i32>>) {
    {
        let mut write_borrow = v.borrow_mut();
        // let mut b2 = v.borrow_mut(); // will panic in runtime not compile time
        // let read_borrow = v.borrow(); // will panic in runtime not compile time
        write_borrow.push(9999);
    }
    let read_borrow = v.borrow();
    let read_borrow2 = v.borrow();
    println!("1read: {read_borrow:?}");
    println!("2read: {read_borrow2:?}");
}


fn get_lazy_value(c: &OnceCell<String>) -> &String {
    c.get_or_init(|| {
        println!("INIT ONCE");
        "Hello, World!".to_string()
    })
}

fn main() {
    let cells: Vec<Cell<i32>> = vec![1.into(), 2.into(), 3.into(), 4.into(), 5.into()];
    println!("Init: {cells:?}");
    mut_cell(cells.get(1).unwrap());
    mut_cells(&cells);
    println!("Changed via cell: {cells:?}");

    let v = cells.iter().map(|cell| cell.get()).collect::<Vec<_>>();
    drop(cells);
    let ref_v = RefCell::new(v);
    mut_vec(&ref_v);
    println!("Changed via ref_cell: {:?}", ref_v.borrow());

    let rfcopy = RefCell::clone(&ref_v);
    ref_v.borrow_mut().push(1);
    rfcopy.borrow_mut().push(-1);
    println!("Origin ref_cell: {:?}", ref_v.borrow());
    println!("Copy ref_cell_copy: {:?}", rfcopy.borrow());

    let person = RefCell::new(
        Holder{data: vec![1, 2, 3, 4, ]}
    );
    let name = Ref::map(person.borrow(), |p| &p.data);
    println!("Ref on part: {name:?}");

    let cell = OnceCell::new();
    assert!(cell.get().is_none());
    let _  = get_lazy_value(&cell);
    let value  = get_lazy_value(&cell);
    println!("{value}");
    assert!(cell.set("value".to_string()).is_err());
}