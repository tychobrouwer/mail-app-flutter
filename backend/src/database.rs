use rusqlite::{params, vtab, Connection, OpenFlags};

use crate::my_error::MyError;

pub mod connections;
pub mod mailbox;
pub mod message;
pub mod messages;

pub async fn new(database_path: &str) -> Result<Connection, MyError> {
    let conn = match Connection::open_with_flags(
        database_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    ) {
        Ok(conn) => conn,
        Err(e) => {
            
            let err = MyError::Sqlite(e, String::from("Error opening database"));
            err.log_error();

            return Err(err);
        }
    };

    match vtab::array::load_module(&conn) {
        Ok(_) => {}
        Err(e) => {
            
            let err = MyError::Sqlite(e, String::from("Error loading database array module"));
            err.log_error();

            return Err(err);
        }
    }

    return Ok(conn);
}

pub async fn initialise(conn: &Connection) -> Result<(), MyError> {
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS connections (
                username VARCHAR(500) NOT NULL,
                password VARCHAR(500) NOT NULL,
                address VARCHAR(500) NOT NULL,
                port INTEGER NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY(username, address)
            )",
        params![],
    ) {
        Ok(_) => {}
        Err(e) => {
            

            let err = MyError::Sqlite(e, String::from("Error creating connections table"));
            err.log_error();
            return Err(err);
        }
    }

    match conn.execute(
            "CREATE TABLE IF NOT EXISTS mailboxes (
                c_username VARCHAR(500) NOT NULL,
                c_address VARCHAR(500) NOT NULL,
                path VARCHAR(500) NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY(c_username, c_address, path),
                FOREIGN KEY(c_username, c_address) REFERENCES connections(username, address) ON DELETE CASCADE
            )",
            params![],
        ) {
            Ok(_) => {}
            Err(e) => {
                

                let err = MyError::Sqlite(e, String::from("Error creating mailboxes table"));
                err.log_error();

                return Err(err);
            }
        }

    match conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                message_uid INTEGER NOT NULL,
                c_username VARCHAR(500) NOT NULL,
                c_address VARCHAR(500) NOT NULL,
                m_path VARCHAR(500) NOT NULL,
                sequence_id INTEGER NULL,
                message_id VARCHAR(500) NOT NULL,
                subject VARCHAR(500) NOT NULL,
                from_ VARCHAR(500) NOT NULL,
                sender VARCHAR(500) NOT NULL,
                to_ VARCHAR(500) NOT NULL,
                cc VARCHAR(500) NOT NULL,
                bcc VARCHAR(500) NOT NULL,
                reply_to VARCHAR(500) NOT NULL,
                in_reply_to VARCHAR(500) NOT NULL,
                delivered_to VARCHAR(500) NOT NULL,
                date_ TIMESTAMP NOT NULL,
                received TIMESTAMP NOT NULL,
                flags VARCHAR(500) NOT NULL,
                html TEXT NOT NULL,
                text TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY(c_username, c_address, m_path, message_uid),
                FOREIGN KEY(c_username, c_address) REFERENCES connections(username, address) ON DELETE CASCADE,
                FOREIGN KEY(c_username, c_address, m_path) REFERENCES mailboxes(c_username, c_address, path) ON DELETE CASCADE
            )",
            params![],
        ) {
            Ok(_) => {}
            Err(e) => {
                

                let err = MyError::Sqlite(e, String::from("Error creating mailboxes table"));
                err.log_error();

                return Err(err);
            }
        }

    return Ok(());
}
