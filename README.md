# r2d2-odbc

# This crate is not maintained anymore

[ODBC](https://github.com/Koka/odbc-rs) adapter for [r2d2](https://github.com/sfackler/r2d2) connection pool

[![https://travis-ci.org/Koka/odbc-rs](https://travis-ci.org/Koka/r2d2-odbc.svg?branch=master)](https://travis-ci.org/Koka/r2d2-odbc)
[![Appveyor Build status](https://ci.appveyor.com/api/projects/status/kyhokonmstsplla6?svg=true)](https://ci.appveyor.com/project/Koka/r2d2-odbc)
[![https://crates.io/crates/r2d2_odbc](https://meritbadge.herokuapp.com/r2d2_odbc#nocache8)](https://crates.io/crates/r2d2_odbc)
[![Coverage Status](https://coveralls.io/repos/github/Koka/r2d2-odbc/badge.svg)](https://coveralls.io/github/Koka/r2d2-odbc)
[![Docs](https://docs.rs/r2d2_odbc/badge.svg)](https://docs.rs/r2d2_odbc)

Example:

```rust

extern crate r2d2;
extern crate r2d2_odbc;
extern crate odbc;

use std::thread;
use r2d2_odbc::ODBCConnectionManager;
use odbc::*;

fn main() {
    let manager = ODBCConnectionManager::new("DSN=PostgreSQL");
    let pool = r2d2::Pool::new(manager).unwrap();

    let mut children = vec![];
    for i in 0..10i32 {
        let pool = pool.clone();
        children.push(thread::spawn(move || {
            let pool_conn = pool.get().unwrap();
            let conn = pool_conn.raw();
            let stmt = Statement::with_parent(&conn).unwrap();
            if let Data(mut stmt) = stmt.exec_direct("SELECT version()").unwrap() {
                while let Some(mut cursor) = stmt.fetch().unwrap() {
                    if let Some(val) = cursor.get_data::<&str>(1).unwrap() {
                        println!("THREAD {} {}", i, val);
                    }
                }
            }
        }));
    }

    for child in children {
        let _ = child.join();
    }
}

```
