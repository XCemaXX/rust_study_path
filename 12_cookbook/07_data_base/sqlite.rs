#![allow(dead_code)]
use std::collections::HashMap;

use rusqlite::{Connection, Result, params};

#[derive(Debug)]
struct Cat {
    name: String,
    color: String,
}

fn create_db() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;

    conn.execute(
        "create table if not exists cat_colors (
             id integer primary key,
             name text not null unique
         )",
        (),
    )?;
    conn.execute(
        "create table if not exists cats (
             id integer primary key,
             name text not null,
             color_id integer not null references cat_colors(id)
         )",
        (),
    )?;

    Ok(conn)
}

fn insert_data(conn: &Connection) -> Result<()> {
    let mut cats = HashMap::new();
    cats.insert("Blue", vec!["Tiger", "Sammy"]);
    cats.insert("Black", vec!["Oreo", "Biscuit"]);

    for (color, names) in &cats {
        conn.execute("INSERT INTO cat_colors (name) VALUES (?1)", [color])?;
        let last_id = conn.last_insert_rowid();

        for cat in names {
            conn.execute(
                "INSERT INTO cats (name, color_id) values (?1, ?2)",
                params![cat, last_id],
            )?;
        }
    }

    Ok(())
}

fn select_data(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT c.name, cc.name FROM cats c
         INNER JOIN cat_colors cc
         ON cc.id = c.color_id;",
    )?;

    let cats = stmt.query_map([], |row| {
        Ok(Cat {
            name: row.get(0)?,
            color: row.get(1)?,
        })
    })?;

    cats.for_each(|cat| {
        println!("Cat: {:?}", cat);
    });
    Ok(())
}

fn successful_tx(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute("insert into cat_colors (name) values (?1)", ["Purple"])?;
    let color_id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO cats (name, color_id) values (?1, ?2)",
        params!["Bob", color_id],
    )?;
    tx.commit()
}

fn rolled_back_tx(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute("delete from cat_colors", ())?;
    tx.execute("insert into cat_colors (name) values (?1)", ["lavender"])?;
    tx.execute("insert into cat_colors (name) values (?1)", ["blue"])?;
    tx.execute("insert into cat_colors (name) values (?1)", ["lavender"])?;
    tx.commit()
}

fn main() {
    let mut conn = create_db().expect("Should be created");
    insert_data(&conn).expect("Should be inserted");
    select_data(&conn).expect("Should be selected");
    let res = successful_tx(&mut conn);
    assert!(res.is_ok());
    let res = successful_tx(&mut conn);
    assert!(res.is_err());
    println!("After transactions");
    select_data(&conn).expect("Should be selected");
}
