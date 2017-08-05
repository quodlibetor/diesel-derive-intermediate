#[macro_use]
extern crate diesel_derive_intermediate;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;

use diesel::prelude::*;
use diesel::expression::sql;
use diesel::sqlite::SqliteConnection;

table! {
    mycologists {
        id -> Integer,
        rust_count -> Integer,
    }
}

table! {
    rusts {
        id -> Integer,
        mycologist_id -> Integer,
        life_cycle_stage -> Integer,
    }
}

#[derive(DieselIntermediate)]
#[derive(Debug, Clone, PartialEq, Identifiable, Insertable, Queryable)]
#[diesel_intermediate_derive(Debug, PartialEq, Insertable)]
#[intermediate_table_name(mycologists)]
#[table_name = "mycologists"]
pub struct Mycologist {
    #[diesel_intermediate_exclude]
    id: i32,
    rust_count: i32,
}

#[derive(DieselIntermediate)]
#[derive(Debug, Clone, PartialEq, Identifiable, Insertable, Queryable)]
#[diesel_intermediate_derive(Debug, PartialEq, Insertable)]
#[intermediate_table_name(rusts)]
#[table_name = "rusts"]
pub struct Rust {
    #[diesel_intermediate_exclude]
    id: i32,
    #[diesel_intermediate_exclude(Captured)]
    mycologist_id: i32,
    life_cycle_stage: i32,
}

#[cfg(test)]
fn setup() -> SqliteConnection {
    let conn = SqliteConnection::establish(":memory:").unwrap();
    let setup = sql::<diesel::types::Bool>("
        CREATE TABLE mycologists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            rust_count INTEGER NOT NULL
        )");
    setup.execute(&conn).expect("Can't create table");
    let setup = sql::<diesel::types::Bool>("
        CREATE TABLE rusts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mycologist_id INTEGER NOT NULL,
            life_cycle_stage INTEGER
        )");
    setup.execute(&conn).expect("Can't create table");
    conn
}

#[test]
fn can_insert_mycologist() {
    let conn = setup();
    let obj = NewMycologist { rust_count: 156 };

    diesel::insert(&obj)
        .into(mycologists::table)
        .execute(&conn)
        .expect("Couldn't insert struct into mycologists");

    let found: Vec<Mycologist> = mycologists::table.load(&conn).unwrap();
    assert_eq!(found, vec![ Mycologist { id: 1, rust_count: 156 } ]);
}

#[test]
fn can_insert_intermediate() {
    let conn = setup();

    let rust = NewRust { life_cycle_stage: 0 };
    let mike = NewMycologist { rust_count: 0 };

    diesel::insert(&mike)
        .into(mycologists::table)
        .execute(&conn)
        .expect("Couldn't insert struct into mycologists");

    let created: Mycologist = mycologists::table
        .order(mycologists::id.desc())
        .first(&conn)
        .unwrap();

    let captured_rust = CapturedRust {
        mycologist_id: created.id,
        life_cycle_stage: rust.life_cycle_stage
    };

    diesel::insert(&captured_rust)
        .into(rusts::table)
        .execute(&conn)
        .expect("Couldn't insert captured_rust into table");

    let created = rusts::table.load::<Rust>(&conn).unwrap();

    assert_eq!(created, vec![Rust {
        id: 1,
        mycologist_id: 1,
        life_cycle_stage: 0,
    }])
}