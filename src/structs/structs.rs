pub struct SIGN_METHOD {
    pub name: String,
    pub method: String,
    pub cookie: i32,
}

pub struct SIGN_GAME {
    pub name: String,
    pub domain: String,
    pub biz: String,
    pub act_id: String,
    pub signgame: String,
}

pub struct REDEEM_GAME {
    pub name: String,
    pub domain: String,
    pub method: String,
}

pub struct REDEEM_DATA {
    pub code: String,
    pub reward: String,
    pub status: Option<String>,
}

impl REDEEM_DATA {
    pub fn new(code: String, reward: String) -> Self {
        Self {
            code,
            reward,
            status: None,
        }
    }
}
