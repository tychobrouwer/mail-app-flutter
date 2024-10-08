use async_std::sync::{Arc, Mutex};

use crate::http_server::{params, to_display};
use crate::inbox_client;
use crate::types::session::{Client, Session};

pub async fn login(
    uri: &str,
    sessions: Arc<Mutex<Vec<Session>>>,
    database_conn: Arc<Mutex<rusqlite::Connection>>,
    clients: Arc<Mutex<Vec<Client>>>,
) -> String {
    let uri_params = params::parse_params(String::from(uri));

    let username = uri_params.get("username");
    let password = uri_params.get("password");
    let address = uri_params.get("address");
    let port = match params::get_u16(uri_params.get("port")) {
        Ok(port) => port,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };

    if username.is_none() || password.is_none() || address.is_none() || port.is_none() {
        eprintln!(
            "Provide username, password, address, and port GET parameters: {}",
            uri
        );
        return String::from("{\"success\": false, \"message\": \"Provide all GET parameters\"}");
    }

    let username = username.unwrap();
    let password = password.unwrap();
    let address = address.unwrap();
    let port = port.unwrap();

    let locked_clients = clients.lock().await;
    match locked_clients
        .iter()
        .position(|x| x.username == username.to_string() && x.address == address.to_string())
    {
        Some(idx) => {
            return format!("{{\"success\": true, \"message\": \"Allready connected to IMAP server\", \"data\": {{ \"session_id\": {}}}}}", idx);
        }
        None => {}
    };

    drop(locked_clients);

    let client_add = Client {
        username: username.to_string(),
        password: password.to_string(),
        address: address.to_string(),
        port,
    };

    let clients_2 = Arc::clone(&clients);
    match inbox_client::connect::connect(sessions, database_conn, clients_2, &client_add).await {
        Ok(idx) => {
            return format!("{{\"success\": true, \"message\": \"Connected to IMAP server\", \"data\": {{ \"session_id\": {}}}}}", idx);
        }
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    }
}

pub async fn logout(
    uri: &str,
    sessions: Arc<Mutex<Vec<Session>>>,
    database_conn: Arc<Mutex<rusqlite::Connection>>,
    clients: Arc<Mutex<Vec<Client>>>,
) -> String {
    let uri_params = params::parse_params(String::from(uri));

    let session_id = match params::get_usize(uri_params.get("session_id")) {
        Ok(session_id) => session_id,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };

    if session_id.is_none() {
        eprintln!("Provide session_id GET parameter: {}", uri);
        return String::from(
            "{\"success\": false, \"message\": \"Provide session_id GET parameter\"}",
        );
    }

    let session_id = session_id.unwrap();

    let locked_clients = clients.lock().await;

    if session_id + 1 > locked_clients.len() {
        return String::from("{\"success\": false, \"message\": \"Invalid session_id\"}");
    }

    let client = &locked_clients[session_id].clone();
    drop(locked_clients);

    match inbox_client::logout::logout(sessions, database_conn, client, session_id).await {
        Ok(_) => {
            return format!(
                "{{\"success\": true, \"message\": \"Logged out\", \"data\": {}}}",
                session_id
            );
        }
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    }
}

pub async fn get_sessions(clients: Arc<Mutex<Vec<Client>>>) -> String {
    let mut response =
        String::from("{\"success\": true, \"message\": \"Sessions retrieved\", \"data\": [");

    let locked_clients = clients.lock().await;

    for (i, client) in locked_clients.iter().enumerate() {
        response.push_str(&format!(
            "{{\"session_id\": {}, \"username\": \"{}\", \"address\": \"{}\", \"port\": {}}}",
            i, client.username, client.address, client.port
        ));

        if i < locked_clients.len() - 1 {
            response.push_str(",");
        }
    }
    response.push_str("]}");

    return response;
}

pub async fn get_mailboxes(
    uri: &str,
    database_conn: Arc<Mutex<rusqlite::Connection>>,
    clients: Arc<Mutex<Vec<Client>>>,
) -> String {
    let uri_params = params::parse_params(String::from(uri));

    let session_id = match params::get_usize(uri_params.get("session_id")) {
        Ok(session_id) => session_id,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };

    if session_id.is_none() {
        eprintln!("Provide session_id GET parameter: {}", uri);
        return String::from(
            "{\"success\": false, \"message\": \"Provide session_id GET parameter\"}",
        );
    }

    let session_id = session_id.unwrap();

    let locked_clients = clients.lock().await;

    if session_id + 1 > locked_clients.len() {
        return String::from("{\"success\": false, \"message\": \"Invalid session_id\"}");
    }

    let client = &locked_clients[session_id].clone();
    drop(locked_clients);

    match inbox_client::mailboxes::get_database(database_conn, client).await {
        Ok(mailboxes) => {
            let mailboxes_str = to_display::string_vec_to_display(&mailboxes);

            return format!(
                "{{\"success\": true, \"message\": \"Mailboxes retrieved\", \"data\": {}}}",
                mailboxes_str
            );
        }
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    }
}

pub async fn update_mailboxes(
    uri: &str,
    sessions: Arc<Mutex<Vec<Session>>>,
    database_conn: Arc<Mutex<rusqlite::Connection>>,
    clients: Arc<Mutex<Vec<Client>>>,
) -> String {
    let uri_params = params::parse_params(String::from(uri));

    let session_id = match params::get_usize(uri_params.get("session_id")) {
        Ok(session_id) => session_id,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };

    if session_id.is_none() {
        eprintln!("Provide session_id GET parameter: {}", uri);
        return String::from(
            "{\"success\": false, \"message\": \"Provide session_id GET parameter\"}",
        );
    }

    let session_id = session_id.unwrap();

    let locked_clients = clients.lock().await;

    if session_id + 1 > locked_clients.len() {
        return String::from("{\"success\": false, \"message\": \"Invalid session_id\"}");
    }

    let client = &locked_clients[session_id].clone();
    drop(locked_clients);

    match inbox_client::mailboxes::update(sessions, database_conn, session_id, client).await {
        Ok(mailboxes) => {
            let mailboxes_str = to_display::string_vec_to_display(&mailboxes);

            return format!(
                "{{\"success\": true, \"message\": \"Mailboxes updated\", \"data\": {}}}",
                mailboxes_str
            );
        }
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    }
}

pub async fn get_messages_with_uids(
    uri: &str,
    database_conn: Arc<Mutex<rusqlite::Connection>>,
    clients: Arc<Mutex<Vec<Client>>>,
) -> String {
    let uri_params = params::parse_params(String::from(uri));

    let session_id = match params::get_usize(uri_params.get("session_id")) {
        Ok(session_id) => session_id,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };
    let mailbox_path = uri_params.get("mailbox_path");
    let message_uids = uri_params.get("message_uids");

    if session_id.is_none() || mailbox_path.is_none() || message_uids.is_none() {
        eprintln!(
            "Provide session_id, mailbox_path, and message_uids GET parameters: {}",
            uri
        );
        return String::from("{\"success\": false, \"message\": \"Provide session_id, mailbox_path and message_uid GET parameters\"}");
    }

    let session_id = session_id.unwrap();
    let mailbox_path = mailbox_path.unwrap();
    let message_uids = message_uids.unwrap();

    let message_uids: Vec<u32> = message_uids
        .split(",")
        .map(|x| x.parse::<u32>().unwrap())
        .collect();

    let locked_clients = clients.lock().await;

    if session_id + 1 > locked_clients.len() {
        return String::from("{\"success\": false, \"message\": \"Invalid session_id\"}");
    }

    let client = &locked_clients[session_id].clone();
    drop(locked_clients);

    match inbox_client::messages::get_database_with_uids(
        database_conn,
        client,
        mailbox_path,
        &message_uids,
    )
    .await
    {
        Ok(messages) => {
            let messages_str = to_display::message_vec_to_display(&messages);

            return format!(
                "{{\"success\": true, \"message\": \"Messages retrieved\", \"data\": {}}}",
                messages_str
            );
        }
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    }
}

pub async fn get_messages_sorted(
    uri: &str,
    database_conn: Arc<Mutex<rusqlite::Connection>>,
    clients: Arc<Mutex<Vec<Client>>>,
) -> String {
    let uri_params = params::parse_params(String::from(uri));

    let session_id = match params::get_usize(uri_params.get("session_id")) {
        Ok(session_id) => session_id,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };
    let mailbox_path = uri_params.get("mailbox_path");

    let start = match params::get_u32(uri_params.get("start")) {
        Ok(start) => start,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };
    let end = match params::get_u32(uri_params.get("end")) {
        Ok(end) => end,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };

    if session_id.is_none() || mailbox_path.is_none() || start.is_none() || end.is_none() {
        eprintln!(
            "Provide session_id, mailbox_path, start, and end GET parameters: {}",
            uri
        );
        return String::from(
            "{\"success\": false, \"message\": \"Provide session_id, mailbox_path, start, and end GET parameters\"}",
        );
    }

    let session_id = session_id.unwrap();
    let mailbox_path = mailbox_path.unwrap();
    let start = start.unwrap();
    let end = end.unwrap();

    let locked_clients = clients.lock().await;

    if session_id + 1 > locked_clients.len() {
        return String::from("{\"success\": false, \"message\": \"Invalid session_id\"}");
    }

    let client = &locked_clients[session_id].clone();
    drop(locked_clients);

    match inbox_client::messages::get_database_sorted(
        database_conn,
        client,
        mailbox_path,
        start,
        end,
    )
    .await
    {
        Ok(messages) => {
            let messages_str = to_display::message_vec_to_display(&messages);
            
            return format!(
                "{{\"success\": true, \"message\": \"Messages retrieved\", \"data\": {}}}",
                messages_str
            );
        }
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    }
}

pub async fn get_messages_with_flag(
    uri: &str,
    database_conn: Arc<Mutex<rusqlite::Connection>>,
    clients: Arc<Mutex<Vec<Client>>>,
) -> String {
    let uri_params = params::parse_params(String::from(uri));

    let session_id = match params::get_usize(uri_params.get("session_id")) {
        Ok(session_id) => session_id,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };
    let mailbox_path = uri_params.get("mailbox_path");
    let flag = uri_params.get("flag");
    let not_flag = match params::get_bool(uri_params.get("not_flag")) {
        Ok(not_flag) => not_flag,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };

    if session_id.is_none() || mailbox_path.is_none() || flag.is_none() || not_flag.is_none() {
        eprintln!(
            "Provide session_id, mailbox_path, flag, and not_flag GET parameters: {}",
            uri
        );
        return String::from("{\"success\": false, \"message\": \"Provide session_id, mailbox_path and message_uid GET parameters\"}");
    }

    let session_id = session_id.unwrap();
    let mailbox_path = mailbox_path.unwrap();
    let flag = flag.unwrap();
    let not_flag = not_flag.unwrap();

    let locked_clients = clients.lock().await;

    if session_id + 1 > locked_clients.len() {
        return String::from("{\"success\": false, \"message\": \"Invalid session_id\"}");
    }

    let client = &locked_clients[session_id].clone();
    drop(locked_clients);

    match inbox_client::messages::get_database_with_flag(
        database_conn,
        client,
        mailbox_path,
        &flag,
        not_flag,
    )
    .await
    {
        Ok(messages) => {
            let messages_str = to_display::message_vec_to_display(&messages);

            return format!(
                "{{\"success\": true, \"message\": \"Messages retrieved\", \"data\": {}}}",
                messages_str
            );
        }
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    }
}

pub async fn update_mailbox(
    uri: &str,
    sessions: Arc<Mutex<Vec<Session>>>,
    database_conn: Arc<Mutex<rusqlite::Connection>>,
    clients: Arc<Mutex<Vec<Client>>>,
) -> String {
    let uri_params = params::parse_params(String::from(uri));

    let session_id = match params::get_usize(uri_params.get("session_id")) {
        Ok(session_id) => session_id,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };
    let mailbox_path = uri_params.get("mailbox_path");
    let mut quick = match params::get_bool(uri_params.get("quick")) {
        Ok(quick) => quick,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };

    if session_id.is_none() || mailbox_path.is_none() {
        eprintln!(
            "Provide session_id and mailbox_path GET parameters: {}",
            uri
        );
        return String::from("{\"success\": false, \"message\": \"Provide session_id and mailbox_path GET parameters\"}");
    }

    if quick.is_none() {
        quick = Some(false);
    }

    let session_id = session_id.unwrap();
    let mailbox_path = mailbox_path.unwrap();

    let locked_clients = clients.lock().await;

    if session_id + 1 > locked_clients.len() {
        return String::from("{\"success\": false, \"message\": \"Invalid session_id\"}");
    }

    let client = &locked_clients[session_id].clone();
    drop(locked_clients);

    match inbox_client::update_mailbox::update_mailbox(
        sessions,
        database_conn,
        session_id,
        client,
        mailbox_path,
        quick.unwrap(),
    )
    .await
    {
        Ok(updated) => {
            let removed_uids_str = to_display::u32_vec_to_display(&updated.removed);
            let new_uids_str = to_display::u32_vec_to_display(&updated.new);
            let changed_uids_str = to_display::u32_vec_to_display(&updated.changed);

            return format!(
                "{{\"success\": true, \"message\": \"Mailbox updated\", \"data\": {{\"new_uids\": {}, \"removed_uids\": {}, \"changed_uids\": {}}}}}",
                new_uids_str, removed_uids_str, changed_uids_str
            );
        }
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    }
}

pub async fn modify_flags(
    uri: &str,
    sessions: Arc<Mutex<Vec<Session>>>,
    database_conn: Arc<Mutex<rusqlite::Connection>>,
    clients: Arc<Mutex<Vec<Client>>>,
) -> String {
    let uri_params = params::parse_params(String::from(uri));

    let session_id = match params::get_usize(uri_params.get("session_id")) {
        Ok(session_id) => session_id,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };
    let mailbox_path = uri_params.get("mailbox_path");
    let message_uid = match params::get_u32(uri_params.get("message_uid")) {
        Ok(message_uid) => message_uid,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };
    let flags = uri_params.get("flags");
    let add = match params::get_bool(uri_params.get("add")) {
        Ok(add) => add,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };

    if session_id.is_none()
        || mailbox_path.is_none()
        || message_uid.is_none()
        || flags.is_none()
        || add.is_none()
    {
        eprintln!(
            "Provide session_id, mailbox_path, message_uid, flags, and add GET parameters: {}",
            uri
        );
        return String::from("{\"success\": false, \"message\": \"Provide session_id, mailbox_path, message_uid, flags, and add GET parameters\"}");
    }

    let session_id = session_id.unwrap();
    let mailbox_path = mailbox_path.unwrap();
    let message_uid = message_uid.unwrap();
    let flags = flags.unwrap();
    let add = add.unwrap();

    let locked_clients = clients.lock().await;

    if session_id + 1 > locked_clients.len() {
        return String::from("{\"success\": false, \"message\": \"Invalid session_id\"}");
    }

    let client = &locked_clients[session_id].clone();
    drop(locked_clients);

    let flags: Vec<String> = flags.split(",").map(|f| f.to_string()).collect();

    match inbox_client::message_flags::modify(
        database_conn,
        sessions,
        session_id,
        client,
        mailbox_path,
        message_uid,
        &flags,
        add,
    )
    .await
    {
        Ok(_) => {
            let flag_str = to_display::string_vec_to_display(&flags);

            return format!(
                "{{\"success\": true, \"message\": \"Flags successfully updated\", \"data\": {}}}",
                flag_str
            );
        }
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    }
}

pub async fn move_message(
    uri: &str,
    sessions: Arc<Mutex<Vec<Session>>>,
    database_conn: Arc<Mutex<rusqlite::Connection>>,
    clients: Arc<Mutex<Vec<Client>>>,
) -> String {
    let uri_params = params::parse_params(String::from(uri));

    let session_id = match params::get_usize(uri_params.get("session_id")) {
        Ok(session_id) => session_id,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };
    let mailbox_path = uri_params.get("mailbox_path");
    let message_uid = match params::get_u32(uri_params.get("message_uid")) {
        Ok(message_uid) => message_uid,
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    };
    let mailbox_path_dest = uri_params.get("mailbox_path_dest");

    if session_id.is_none()
        || mailbox_path.is_none()
        || message_uid.is_none()
        || mailbox_path_dest.is_none()
    {
        eprintln!(
            "Provide session_id, mailbox_path, message_uid, and mailbox_path_dest GET parameters: {}",
            uri
        );
        return String::from("{\"success\": false, \"message\": \"Provide session_id, mailbox_path, message_uid, and mailbox_path_dest GET parameters\"}");
    }

    let session_id = session_id.unwrap();
    let mailbox_path = mailbox_path.unwrap();
    let message_uid = message_uid.unwrap();
    let mailbox_path_dest = mailbox_path_dest.unwrap();

    let locked_clients = clients.lock().await;

    if session_id + 1 > locked_clients.len() {
        return String::from("{\"success\": false, \"message\": \"Invalid session_id\"}");
    }

    let client = &locked_clients[session_id].clone();
    drop(locked_clients);

    match inbox_client::message::mv(
        sessions,
        database_conn,
        session_id,
        client,
        mailbox_path,
        message_uid,
        mailbox_path_dest,
    )
    .await
    {
        Ok(_) => {
            return String::from("{\"success\": true, \"message\": \"Message successfully moved\"}")
        }
        Err(e) => {
            return format!("{{\"success\": false, \"message\": \"{}\"}}", e);
        }
    }
}
