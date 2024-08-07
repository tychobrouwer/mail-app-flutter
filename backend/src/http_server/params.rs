use std::collections::HashMap;
use url_escape::decode;

pub fn get_usize(uri_param: Option<&String>) -> Option<usize> {
    match uri_param {
        Some(param) => Some(param.parse::<usize>().unwrap()),
        None => None,
    }
}

pub fn get_u16(uri_param: Option<&String>) -> Option<u16> {
    match uri_param {
        Some(param) => Some(param.parse::<u16>().unwrap()),
        None => None,
    }
}

pub fn get_u32(uri_param: Option<&String>) -> Option<u32> {
    match uri_param {
        Some(param) => Some(param.parse::<u32>().unwrap()),
        None => None,
    }
}

pub fn get_bool(uri_param: Option<&String>) -> Option<bool> {
    match uri_param {
        Some(param) => Some(param.parse::<bool>().unwrap()),
        None => None,
    }
}

pub fn parse_params(uri: String) -> HashMap<String, String> {
    let uri_parts: Vec<&str> = uri.split("&").collect();

    let mut result: HashMap<String, String> = HashMap::new();

    uri_parts.iter().for_each(|part| {
        let parts: Vec<&str> = part.split("=").collect();

        if parts.len() != 2 {
            return;
        }

        let key = parts[0].to_owned();
        let value = decode(parts[1]).to_string();

        result.insert(key, value);
    });

    return result;
}
