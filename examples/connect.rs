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