use once_cell::sync::Lazy;
use redis::Client;

pub const TRON_GRID: &str = ""; // TronGrid api
pub const SUPPORT_ID: i64 = 0; // telegram support chat id

pub static REDIS_CLIENT: Lazy<Client> =
    Lazy::new(|| Client::open("redis://127.0.0.1:6379").expect("Invalid Redis URL"));
