pub struct SignMethod {
    pub name: String,
    pub method: String,
    pub cookie: i32,
}

pub struct SignGame {
    pub name: String,
    pub domain: String,
    pub biz: String,
    pub act_id: String,
    pub signgame: String,
}

pub struct RedeemGame {
    pub name: String,
    pub domain: String,
    pub method: String,
}

#[derive(serde::Serialize, Debug)]
pub struct RedeemData {
    pub cdkey: String,
    pub reward: String,
    pub status: Option<String>,
}

impl RedeemData {
    pub fn new(code: String, reward: String) -> Self {
        Self {
            cdkey: code,
            reward,
            status: None,
        }
    }
}
