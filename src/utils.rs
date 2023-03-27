use std::time::{SystemTime, UNIX_EPOCH};

pub fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

//
//
//
pub fn add_tv(mut payload: json::JsonValue, token: &str, value: &str) -> json::JsonValue {
    payload[token.to_string()] = value.to_string().into();
    payload.clone()
}
pub fn add_utv(mut payload: json::JsonValue, token: &str, value: usize) -> json::JsonValue {
    payload[token.to_string()] = value.into();
    payload.clone()
}
pub fn add_ftv(mut payload: json::JsonValue, token: &str, value: f32) -> json::JsonValue {
    payload[token.to_string()] = value.into();
    payload.clone()
}

//
// Returns the unix_epoch in seconds
//
pub fn get_unix_epoch() -> usize {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(n) => match n.as_secs().to_string().parse::<usize>() {
            Ok(v) => v,
            _ => 0,
        },
        Err(_) => 0,
    }
}
