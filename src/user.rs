use crate::database;
use redis::Commands;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub userid: String,
    pub balance: f64,
    pub invoice: u8,
}

impl User {
    pub async fn save_to_redis(&self) {
        let mut conn = database::REDIS_CLIENT.get_connection().unwrap();
        let key = format!("user:{}", self.userid);
        let value = serde_json::to_string(self).unwrap();
        let _: () = conn.set(key, value).expect("setting key to value");
    }

    pub async fn load_from_redis(userid: &str) -> Self {
        let mut conn = database::REDIS_CLIENT.get_connection().unwrap();
        let key = format!("user:{}", userid);
        if let Ok(value) = conn.get::<_, String>(key.clone()) {
            let user: User = serde_json::from_str(&value).unwrap();
            user
        } else {
            let user = User::new(userid);
            let value = serde_json::to_string(&user).unwrap();
            let _: () = conn.set(key.clone(), value).expect("setting key to value");
            user
        }
    }
    fn new(userid: &str) -> Self {
        User {
            userid: userid.to_string(),
            balance: 1.0,
            invoice: 1,
        }
    }

    pub fn get_invoice_address(&self) -> String {
        let inv_nb = self.invoice;
        let userid = self.userid.clone();
        let passphrase = format!("/|tgrustpanels|/<Tron-Address>?!_{}_{}", userid, inv_nb);
        let private_key_hex = rucpanels::passphrase_to_private_key(&passphrase);
        rucpanels::generate_tron_address(&private_key_hex)
    }
}
