use std::collections::HashMap;
use base64::{prelude::BASE64_STANDARD, Engine};
use regex::Regex;
use imap;
use imap_proto;

use chrono::{DateTime, FixedOffset};

fn message_str(string: Option<&[u8]>) -> String {
    match string {
        Some(s) => match std::str::from_utf8(s) {
            Ok(s) => String::from(s),
            Err(_) => String::from(""),
        },
        None => String::from(""),
    }
}

fn message_address(address: &Option<Vec<imap_proto::types::Address>>) -> String {
    match address {
        Some(a) => {
            let mut result = String::from("[");

            for (i, address) in a.iter().enumerate() {
                result.push_str("{");
                result.push_str(&format!("\"name\": \"{}\",", message_str(address.name)));
                result.push_str(&format!("\"mailbox\": \"{}\",", message_str(address.mailbox)));
                result.push_str(&format!("\"host\": \"{}\"", message_str(address.host)));
                result.push_str("}");

                if i < a.len() - 1 {
                    result.push_str(",");
                }
            }

            result.push_str("]");

            return result;
        },
        None => return String::from("[]"),
    }
}

struct MessageBody {
    date: String,
    received: String,
    to: String,
    delivered_to: String,
    from: String,
    subject: String,
    message_id: String,
    text: String,
    html: String,
}

enum State {
    HeaderKey,
    HeaderValue,
    TextHeader,
    Text,
    HtmlHeader,
    Html,
    BlankLine,
}

fn parse_time_rfc2822(time_str: Option<&String>) -> DateTime<FixedOffset> {
    let time_re = Regex::new(r"(\w{1,3}, \d{1,2} \w{1,3} \d{4} \d{2}:\d{2}:\d{2} ([+-]\d{4})?(\w{3})?)").unwrap();
    let binding = String::from("");

    let date = match time_re.captures(time_str.unwrap_or(&binding)) {
        Some(c) => c.get(1).unwrap().as_str(),
        None => {
            dbg!("Error: Could not parse date");
            "Thu, 1 Jan 1970 00:00:00 +0000"
        },
    };

    let date = match DateTime::parse_from_rfc2822(&date) {
        Ok(date) => date,
        Err(e) => {
            dbg!("Error: {}", e);
            DateTime::parse_from_rfc2822("Thu, 1 Jan 1970 00:00:00 +0000").unwrap()
        },
    };

    return date;
}

fn parse_message_body(body: &str) -> MessageBody {   
    let mut state = State::HeaderKey;

    let mut header_key = String::from("");
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut html = String::from("");
    let mut text = String::from("");

    let mut html_encoding = String::from("utf-8");
    let mut text_encoding = String::from("utf-8");

    let lines = body.lines();

    let re_boundary = Regex::new(r#"boundary="(.*)""#).unwrap();
    let boundary = match re_boundary.captures(body) {
        Some(c) => c.get(1).unwrap().as_str(),
        None => "",
    };

    let mut i = 0;
    while i < lines.clone().count() {
        let line = lines.clone().nth(i).unwrap();
        i += 1;

        match &state {
            State::HeaderKey => {
                if line.is_empty() {
                    state = State::BlankLine;

                    continue;
                }

                let split = match line.split_once(":") {
                    Some(s) => s,
                    None => {
                        dbg!("Error: Could not split header key and value");

                        ("", "")
                    },
                };

                header_key = split.0.to_string();

                let value_part = split.1.trim().replace("\r\n", " ");
                
                if headers.contains_key(&header_key) {
                    headers.insert(header_key.clone(), headers[&header_key].clone() + value_part.as_str());
                } else {
                    headers.insert(header_key.clone(), value_part);
                }

                state = State::HeaderValue;
            },
            State::HeaderValue => {
                if line.is_empty() {
                    state = State::BlankLine;

                    continue;
                } else if line.contains(":") && line.starts_with(char::is_alphabetic) {
                    state = State::HeaderKey;

                    i -= 1;
                    continue;
                }

                let value = line
                    .trim()
                    .replace("\r\n", " ");

                if headers.contains_key(&header_key) {
                    headers.insert(header_key.clone(), headers[&header_key].clone() + value.as_str());
                } else {
                    headers.insert(header_key.clone(), value);
                }
            },
            State::TextHeader => {
                if line.is_empty() || (!line.contains(":") && line.starts_with(char::is_alphabetic)) {
                    state = State::Text;

                    continue;
                }

                dbg!(line);

                let split = match line.split_once(":") {
                    Some(s) => s,
                    None => {
                        dbg!("Error: Could not split header key and value");

                        ("", "")
                    },
                };

                let key = split.0.trim();

                if key == "Content-Transfer-Encoding" {
                    text_encoding = split.1.trim().to_string();
                }
            },
            State::Text => {
                if line.starts_with(&(String::from("--") + boundary)) {
                    state = State::BlankLine;

                    continue;
                }

                text.push_str(line);
            },
            State::HtmlHeader => {
                if line.is_empty() || (!line.contains(":") && line.starts_with(char::is_alphabetic)) {
                    state = State::Html;
                }

                dbg!(line);

                let split = match line.split_once(":") {
                    Some(s) => s,
                    None => {
                        dbg!("Error: Could not split header key and value");

                        ("", "")
                    },
                };

                let key = split.0.trim();

                if key == "Content-Transfer-Encoding" {
                    html_encoding = split.1.trim().to_string();
                }
            },
            State::Html => {
                if line.starts_with(&(String::from("--") + boundary)) {
                    state = State::BlankLine;

                    continue;
                }

                html.push_str(line);
            },
            State::BlankLine => {

                if line.starts_with("Content-Type: text/plain") {
                    state = State::TextHeader;
                } else if line.starts_with("Content-Type: text/html") {
                    state = State::HtmlHeader;
                }
            }
        }
    }

    let re_encoding = Regex::new(r"=(..)").unwrap();
    html = re_encoding.replace_all(html.as_str(), |caps: &regex::Captures| {
        if caps.get(1).unwrap().as_str() == "3D" {
            String::from("=")
        } else {
            caps.get(1).unwrap().as_str().to_string()
        }
    }).to_string();

    html = html.replace("=3D", "=");
    html = html.replace("&#39;", "'");
    html = html.replace("&amp;", "&");
    html = html.replace("&copy;", "©");

    if text_encoding != "base64" {
        text = BASE64_STANDARD.encode(text.as_bytes());
    }

    if html_encoding != "base64" {
        html = BASE64_STANDARD.encode(html.as_bytes());
    }

    let date = parse_time_rfc2822(headers.get("Date"));
    let received = parse_time_rfc2822(headers.get("Received"));

    let binding = String::from("");
    let to = headers.get("To").unwrap_or(&binding);
    let delivered_to = headers.get("Delivered-To").unwrap_or(&binding);
    let from = headers.get("From").unwrap_or(&binding);
    let subject = headers.get("Subject").unwrap_or(&binding);
    let message_id = headers.get("Message-ID").unwrap_or(&binding);

    dbg!(&headers);

    return MessageBody {
        date: date.to_rfc3339(),
        received: received.to_rfc3339(),
        to: to.to_string(),
        delivered_to: delivered_to.to_string(),
        from: from.to_string(),
        subject: subject.to_string(),
        message_id: message_id.to_string(),
        text,
        html,
    };
}

pub fn envelope_to_string(fetch: &imap::types::Fetch, message_uid: u32) -> String {
    let envelope = match fetch.envelope() {
        Some(e) => e,
        None => return String::from(""),
    };

    let mut result = String::from("{");

    result.push_str(&format!("\"date\": \"{}\",", message_str(envelope.date)));
    result.push_str(&format!("\"subject\": \"{}\",", message_str(envelope.subject)));
    result.push_str(&format!("\"from\": {},", message_address(&envelope.from)));
    result.push_str(&format!("\"sender\": {},", message_address(&envelope.sender)));
    result.push_str(&format!("\"reply_to\": {},", message_address(&envelope.reply_to)));
    result.push_str(&format!("\"to\": {},", message_address(&envelope.to)));
    result.push_str(&format!("\"cc\": {},", message_address(&envelope.cc)));
    result.push_str(&format!("\"bcc\": {},", message_address(&envelope.bcc)));
    result.push_str(&format!("\"in_reply_to\": \"{}\",", message_str(envelope.in_reply_to)));
    result.push_str(&format!("\"message_id\": \"{}\",", message_str(envelope.message_id)));
    result.push_str(&format!("\"message_uid\": {}", message_uid));

    result.push_str("}");

    return result;
}

pub fn message_to_string(body_fetch: &imap::types::Fetch, message_uid: u32) -> String {
    let message = match body_fetch.body() {
        Some(m) => std::str::from_utf8(m).unwrap(),
        None => "",
    };

    let message_body: MessageBody = parse_message_body(message);

    let mut result = String::from("{");

    result.push_str(&format!("\"message_uid\": {},", message_uid));
    result.push_str(&format!("\"date\": \"{}\",", message_body.date));
    result.push_str(&format!("\"received\": \"{}\",", message_body.received));
    result.push_str(&format!("\"subject\": \"{}\",", message_body.subject));
    result.push_str(&format!("\"from\": \"{}\",", message_body.from));
    result.push_str(&format!("\"to\": \"{}\",", message_body.to));
    result.push_str(&format!("\"delivered_to\": \"{}\",", message_body.delivered_to));
    result.push_str(&format!("\"message_id\": \"{}\",", message_body.message_id));
    result.push_str(&format!("\"text\": \"{}\",", message_body.text));
    result.push_str(&format!("\"html\": \"{}\"", message_body.html));

    result.push_str("}");

    return result;
}
