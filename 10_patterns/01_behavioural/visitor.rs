use std::cell::Cell;

trait Visitable {
    fn accept(&self, visitor: &dyn Visitor);
}

trait Visitor {
    fn visit_doc(&self, d: &Document);
    fn visit_table(&self, t: &Table);
    fn visit_text(&self, t: &Text);
}

struct Document {
    elements: Vec<Box<dyn Visitable>>,
}

impl Document {
    fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    fn add_element(&mut self, element: Box<dyn Visitable>) {
        self.elements.push(element);
    }

    fn print(&self, ident: usize) {
        if self.elements.is_empty() {
            println!("{}Empty document", " ".repeat(ident));
        } else {
            println!("{}Document:", " ".repeat(ident));
        }
    }
}

impl Visitable for Document {
    fn accept(&self, visitor: &dyn Visitor) {
        visitor.visit_doc(&self);
    }
}

struct Table {
    elements: Vec<Box<dyn Visitable>>,
}

impl Table {
    fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    fn add_element(&mut self, element: Box<dyn Visitable>) {
        self.elements.push(element);
    }

    fn print(&self, ident: usize) {
        if self.elements.is_empty() {
            println!("{}Empty table", " ".repeat(ident));
        } else {
            println!("{}Table:", " ".repeat(ident));
        }
    }
}

impl Visitable for Table {
    fn accept(&self, visitor: &dyn Visitor) {
        visitor.visit_table(&self);
    }
}

struct Text {
    data: String,
}

impl Text {
    fn print(&self, ident: usize) {
        println!("{}Text: {}", " ".repeat(ident), self.data);
    }
}

impl Visitable for Text {
    fn accept(&self, visitor: &dyn Visitor) {
        visitor.visit_text(&self);
    }
}

#[derive(Default)]
struct Printer {
    ident: Cell<usize>,
}

impl Visitor for Printer {
    fn visit_doc(&self, d: &Document) {
        let ident = self.ident.get();
        d.print(ident);
        self.ident.set(ident + 1);
        for element in &d.elements {
            element.accept(self);
        }
    }

    fn visit_table(&self, t: &Table) {
        let ident = self.ident.get();
        t.print(ident);
        self.ident.set(ident + 1);
        for element in &t.elements {
            element.accept(self);
        }
    }

    fn visit_text(&self, t: &Text) {
        let ident = self.ident.get();
        t.print(ident);
    }
}

#[derive(Default)]
struct Counter {
    count: Cell<usize>,
}

impl Counter {
    fn print_count(&self) {
        println!("{}", self.count.get());
    }
}

impl Visitor for Counter {
    fn visit_doc(&self, d: &Document) {
        self.count.set(self.count.get() + 1);
        for element in &d.elements {
            element.accept(self);
        }
    }

    fn visit_table(&self, t: &Table) {
        self.count.set(self.count.get() + 1);
        for element in &t.elements {
            element.accept(self);
        }
    }

    fn visit_text(&self, _: &Text) {
        self.count.set(self.count.get() + 1);
    }
}

fn main() {
    let mut doc = Document::new();
    let mut table_outer = Table::new();
    let mut table_inner = Table::new();
    table_inner.add_element(Box::new(Text {
        data: "inner_first".to_string(),
    }));
    table_inner.add_element(Box::new(Text {
        data: "inner_second".to_string(),
    }));

    table_outer.add_element(Box::new(Text {
        data: "outer".to_string(),
    }));
    table_outer.add_element(Box::new(table_inner));

    doc.add_element(Box::new(table_outer));
    doc.add_element(Box::new(Table::new()));

    let printer = Printer::default();
    doc.accept(&printer);

    let counter = Counter::default();
    doc.accept(&counter);
    counter.print_count();
}
