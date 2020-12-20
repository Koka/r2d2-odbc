//! ODBC support for the `r2d2` connection pool.
extern crate r2d2;
extern crate odbc;
extern crate odbc_safe as safe;

#[macro_use]
extern crate lazy_static;

use std::error::Error;
use std::fmt;
use odbc::*;

#[derive(Debug)]
pub struct ODBCConnectionManager {
    connection_string: String
}

#[derive(Debug)]
pub struct ODBCConnectionManagerTx {
    connection_string: String
}

pub struct ODBCConnection<'a, AC: safe::AutocommitMode>(Connection<'a, AC>);

unsafe impl Send for ODBCConnection<'static, safe::AutocommitOn> {}
unsafe impl Send for ODBCConnection<'static, safe::AutocommitOff> {}

impl <'a, AC: safe::AutocommitMode> ODBCConnection<'a, AC> {
    pub fn raw(&self) -> &Connection<'a, AC> {
        &self.0
    }
}

pub struct ODBCEnv(Environment<Version3>);

unsafe impl Sync for ODBCEnv {}

unsafe impl Send for ODBCEnv {}

#[derive(Debug)]
pub struct ODBCError(Box<dyn Error>);

lazy_static! {
    static ref ENV: ODBCEnv = ODBCEnv(create_environment_v3().unwrap());
}

impl Error for ODBCError {
    fn description(&self) -> &str {
        "Error connecting DB"
    }
}

impl fmt::Display for ODBCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<DiagnosticRecord> for ODBCError {
    fn from(err: DiagnosticRecord) -> Self {
        println!("ODBC ERROR {}", err);
        ODBCError(Box::new(err))
    }
}

impl <E: 'static> From<std::sync::PoisonError<E>> for ODBCError {
    fn from(err: std::sync::PoisonError<E>) -> Self {
        ODBCError(Box::new(err))
    }
}

impl ODBCConnectionManager {
    /// Creates a new `ODBCConnectionManager`.
    pub fn new<S: Into<String>>(connection_string: S) -> ODBCConnectionManager
    {
        ODBCConnectionManager {
            connection_string: connection_string.into()
        }
    }
}

impl ODBCConnectionManagerTx {
    /// Creates a new `ODBCConnectionManagerTx`.
    pub fn new<S: Into<String>>(connection_string: S) -> ODBCConnectionManagerTx
    {
        ODBCConnectionManagerTx {
            connection_string: connection_string.into()
        }
    }
}

/// An `r2d2::ManageConnection` for ODBC connections.
///
/// ## Example
///
/// ```rust,no_run
/// extern crate r2d2;
/// extern crate r2d2_odbc;
/// extern crate odbc;
///
/// use std::thread;
/// use r2d2_odbc::ODBCConnectionManager;
/// use odbc::*;
///
/// fn main() {
///     let manager = ODBCConnectionManager::new("DSN=PostgreSQL");
///     let pool = r2d2::Pool::new(manager).unwrap();
///
///     let mut children = vec![];
///     for i in 0..10i32 {
///         let pool = pool.clone();
///         children.push(thread::spawn(move || {
///             let pool_conn = pool.get().unwrap();
///             let conn = pool_conn.raw();
///             let stmt = Statement::with_parent(&conn).unwrap();
///             if let Data(mut stmt) = stmt.exec_direct("SELECT version()").unwrap() {
///                 while let Some(mut cursor) = stmt.fetch().unwrap() {
///                     if let Some(val) = cursor.get_data::<&str>(0).unwrap() {
///                         print!("THREAD {} {}", i, val);
///                     }
///                 }
///             }
///         }));
///     }
///     for child in children {
///         let _ = child.join();
///     }
/// }
/// ```
impl r2d2::ManageConnection for ODBCConnectionManager {
    type Connection = ODBCConnection<'static, safe::AutocommitOn>;
    type Error = ODBCError;

    fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        let env = &ENV.0;
        Ok(ODBCConnection(env.connect_with_connection_string(&self.connection_string)?))
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        let stmt = Statement::with_parent(conn.raw())?;
        stmt.exec_direct("SELECT 1")?;
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        //TODO
        false
    }
}

impl r2d2::ManageConnection for ODBCConnectionManagerTx {
    type Connection = ODBCConnection<'static, safe::AutocommitOff>;
    type Error = ODBCError;

    fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        let env = &ENV.0;
        let conn = env.connect_with_connection_string(&self.connection_string)?;
        let conn_result = conn.disable_autocommit();
        match conn_result {
            Ok(conn) => Ok(ODBCConnection(conn)),
            _ => Err(ODBCError("Unable to use transactions".into()))
        }
    }

    fn is_valid(&self, _conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        //TODO
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        //TODO
        false
    }
}
