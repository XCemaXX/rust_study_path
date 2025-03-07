use std::fmt::{self, Debug};

trait Transormable<T> {
    fn transform(&self) -> T;
}

enum DocPrimitive {
    Document { 
        name: String,
        content: Vec<DocPrimitive>,
    },
    Table { 
        number: usize,
        rows: Vec<DocPrimitive>,
    },
    Text(String),
}

enum JSON {
    Object(Vec<(String, JSON)>),
    Array(Vec<JSON>),
    String(String),
    Number(f64),
}

impl Debug for JSON {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JSON::Object(obj) => write!(
                f,
                "{{ {} }}",
                obj.iter()
                    .map(|(k, v)| format!("\"{}\": {:?}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            JSON::Array(arr) => write!(
                f,
                "[{}]",
                arr.iter()
                    .map(|v| format!("{:?}", v))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            JSON::String(s) => write!(f, "\"{}\"", s),
            JSON::Number(n) => write!(f, "{}", n),
        }
    }
}

impl Transormable<JSON> for DocPrimitive {
    fn transform(&self) -> JSON {
        match self {
            DocPrimitive::Document { name, content } => {
                let transformed: Vec<JSON> = content.iter().map(DocPrimitive::transform).collect();
                JSON::Object(vec![
                    ("type".to_string(), JSON::String("document".to_string())),
                    ("name".to_string(), JSON::String(name.clone())),
                    ("content".to_string(), JSON::Array(transformed)),
                ])
            }
            DocPrimitive::Table { number, rows } => {
                let transformed: Vec<JSON> = rows.iter().map(|child| child.transform()).collect();
                JSON::Object(vec![
                    ("type".to_string(), JSON::String("table".to_string())),
                    ("number".to_string(), JSON::Number(*number as f64)),
                    ("rows".to_string(), JSON::Array(transformed)),
                ])
            }
            DocPrimitive::Text(text) => {
                JSON::String(text.clone())
            }
        }
    }
}

fn main() {
    let doc = DocPrimitive::Document {
        name: "My Document".to_string(),
        content: vec![
            DocPrimitive::Text("Hello".to_string()),
            DocPrimitive::Table {
                number: 1,
                rows: vec![
                    DocPrimitive::Text("Row1".to_string()),
                    DocPrimitive::Text("Row2".to_string()),
                    DocPrimitive::Table { 
                        number: 2,
                        rows: Vec::new()
                    }
                ],
            },
            DocPrimitive::Text("World".to_string()),
        ],
    };

    let json = doc.transform();
    println!("JSON representation:\n{:?}", json);
}