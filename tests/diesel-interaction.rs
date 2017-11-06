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

table! {
    petri_dishes {
        id -> Integer,
        mycologist_id -> Integer,
        size -> Integer,
    }
}

table! {
    mikes {
        id -> Integer,
        rust_count -> Integer,
    }
}

mod items {
    use super::{mycologists, rusts, mikes};

    #[derive(DieselIntermediate)]
    #[derive(Debug, Clone, PartialEq, Identifiable, Insertable, Queryable)]
    #[intermediate_derive(Debug, PartialEq, Insertable)]
    #[table_name = "mycologists"]
    pub struct Mycologist {
        #[intermediate_exclude]
        pub id: i32,
        pub rust_count: i32,
    }

    #[derive(DieselIntermediate)]
    #[derive(Debug, Clone, PartialEq, Identifiable, Insertable, Queryable)]
    #[intermediate_derive(Debug, PartialEq, Insertable)]
    #[intermediate_table_name = "mikes"]
    #[table_name = "mycologists"]
    pub struct Scientist {
        #[intermediate_exclude]
        pub id: i32,
        pub rust_count: i32,
    }

    #[derive(DieselIntermediate)]
    #[derive(Debug, Clone, PartialEq, Identifiable, Insertable, Queryable, Associations)]
    #[intermediate_derive(Debug, PartialEq, Insertable)]
    #[table_name = "rusts"]
    #[belongs_to(Mycologist)]
    pub struct Rust {
        #[intermediate_exclude]
        pub id: i32,
        #[intermediate_exclude(Captured)]
        pub mycologist_id: i32,
        pub life_cycle_stage: i32,
    }
}

use items::*;

#[cfg(test)]
fn setup() -> SqliteConnection {
    let conn = SqliteConnection::establish(":memory:").unwrap();
    let setup = sql::<diesel::types::Bool>(
        "
        CREATE TABLE mycologists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            rust_count INTEGER NOT NULL
        )",
    );
    setup.execute(&conn).expect("Can't create table: mycologists");
    let setup = sql::<diesel::types::Bool>(
        "
        CREATE TABLE rusts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mycologist_id INTEGER NOT NULL,
            life_cycle_stage INTEGER,
            FOREIGN KEY(mycologist_id) REFERENCES mycologists(id)
        )",
    );
    setup.execute(&conn).expect("Can't create table: rusts");
    let setup = sql::<diesel::types::Bool>(
        "
        CREATE TABLE mikes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            rust_count INTEGER
        )",
    );
    setup.execute(&conn).expect("Can't create table: mikes");
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
    assert_eq!(
        found,
        vec![
            Mycologist {
                id: 1,
                rust_count: 156,
            },
        ]
    );
}

#[test]
fn can_insert_intermediate() {
    let conn = setup();

    let rust = NewRust {
        life_cycle_stage: 0,
    };
    let mike = NewMycologist { rust_count: 0 };

    diesel::insert(&mike)
        .into(mycologists::table)
        .execute(&conn)
        .expect("Couldn't insert struct into mycologists");

    let created_mike: Mycologist = mycologists::table
        .order(mycologists::id.desc())
        .first(&conn)
        .unwrap();

    let captured_rust = CapturedRust {
        mycologist_id: created_mike.id,
        life_cycle_stage: rust.life_cycle_stage,
    };

    diesel::insert(&captured_rust)
        .into(rusts::table)
        .execute(&conn)
        .expect("Couldn't insert captured_rust into table");

    let created = rusts::table.load::<Rust>(&conn).unwrap();

    assert_eq!(
        created,
        vec![
            Rust {
                id: 1,
                mycologist_id: 1,
                life_cycle_stage: 0,
            },
        ]
    );

    let rusts = Rust::belonging_to(&created_mike)
        .load::<Rust>(&conn)
        .expect("couldn't load rusts belonging to mike");

    assert_eq!(
        rusts,
        vec![
            Rust {
                id: 1,
                mycologist_id: 1,
                life_cycle_stage: 0,
            },
        ]
    );
}


#[test]
fn can_insert_into_intermediate_table() {
    let conn = setup();
    let mike = NewScientist { rust_count: 12 };

    diesel::insert(&mike)
        .into(mikes::table)
        .execute(&conn)
        .expect("Couldn't insert mike into scientists table");

    let fetched_mike = mikes::table.load::<Scientist>(&conn).unwrap();

    diesel::insert(&fetched_mike)
        .into(mycologists::table)
        .execute(&conn)
        .expect("Couldn't insert mike into mycologists table");
}
