
#[derive(Debug)]
pub struct Todo {
    pub message: String,
    pub done: bool,
}

pub struct Todos {
    pub list: Vec<Todo>,
}
 
struct TodosIteratorByRef<'a> {
    todos: &'a Todos,
    index: usize,
}

impl <'a>Iterator for TodosIteratorByRef<'a> {
    type Item = &'a Todo;

    fn next(&mut self) -> Option<Self::Item> {
        let list = &self.todos.list;
        if self.index < list.len() {
            let result = Some(&list[self.index]);
            self.index += 1;
            result
        } else {
            None
        }
    }

}

impl Todos {
    fn iter(&self) -> TodosIteratorByRef {
        TodosIteratorByRef { todos: self, index: 0, }
    }
}
//####################
pub struct TodosIntoIteratorByValue {
    todos: Todos
}

impl IntoIterator for Todos {
    type Item = Todo;
    type IntoIter = TodosIntoIteratorByValue;

    fn into_iter(self) -> TodosIntoIteratorByValue {
        TodosIntoIteratorByValue { todos: self }
    }
}

impl Iterator for TodosIntoIteratorByValue {
    type Item = Todo;
    fn next(&mut self) -> Option<Self::Item> {
        if self.todos.list.len() == 0 {
            return None;
        }
        let result = self.todos.list.remove(0);
        Some(result)
    }
}

fn main() {
    let todo_list = Todos{list: 
        vec![Todo{message: "first".to_string(), done: true},
        Todo{message: "second".to_string(), done: true},
        Todo{message: "third".to_string(), done: true}]};
    for todo in todo_list.iter() {
        println!("{:?}", todo);// todo is a &Todo
    }
    // we can reuse todo_list, since it's not consumed
    for todo in todo_list {
        println!("{:?}", todo);// todo is a &Todo
    }
    // we cannot reuse todo_list, since the for loop consumes it
}