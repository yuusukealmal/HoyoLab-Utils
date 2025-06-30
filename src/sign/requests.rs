use std::env;

use reqwest::header::{self, COOKIE};

use crate::structs::structs::{SignGame, SignMethod};

impl SignMethod {
    async fn req(&self, game: &SignGame) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let url = format!(
            "https://sg-{}-api.hoyolab.com/event/{}/{}?act_id={}&lang=zh-tw",
            game.domain, game.biz, self.name, game.act_id
        );

        let mut headers = header::HeaderMap::new();
        headers.insert("x-rpc-signgame", game.signgame.parse()?);
        if self.cookie == 1 {
            let ltuid_v2 = env::var("ltuid_v2").expect("ltuid_v2 not set in environment");
            let ltoken_v2 = env::var("ltoken_v2").expect("ltoken_v2 not set in environment");
            headers.insert(
                COOKIE,
                format!("ltuid_v2={};ltoken_v2={}", ltuid_v2, ltoken_v2).parse()?,
            );
        }

        let client = reqwest::Client::new();
        let response = client
            .request(self.method.parse()?, url)
            .headers(headers)
            .send()
            .await?;

        Ok(response.json::<serde_json::Value>().await?)
    }
}

impl SignGame {
    pub async fn sign(&self) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let mut result: Vec<serde_json::Value> = vec![];

        let methods: Vec<SignMethod> = vec![
            SignMethod {
                name: String::from("sign"),
                method: String::from("POST"),
                cookie: 1,
            },
            SignMethod {
                name: String::from("info"),
                method: String::from("GET"),
                cookie: 1,
            },
            SignMethod {
                name: String::from("home"),
                method: String::from("GET"),
                cookie: 0,
            },
        ];

        let sign_result = methods[0].req(self).await?;
        if sign_result["retcode"] == -5003 {
            result.push(serde_json::json!({"name": self.name,"value": sign_result["message"]}));
        }

        if sign_result["message"] == "OK" {
            let is_sign: i64 = methods[1].req(self).await?["data"]["total_sign_day"]
                .as_i64()
                .unwrap();
            let sign_rewards = methods[2].req(self).await?;
            let sign_data = sign_rewards["data"].as_object().unwrap();
            let rewards = sign_data["awards"][is_sign as usize - 1]
                .as_object()
                .unwrap();
            let total_days = sign_rewards["data"]["awards"].as_array().unwrap().len();
            result.push(serde_json::json!({"name": format!("{} {} / {}", self.name, is_sign, total_days), "value": format!("`{}` x{}", rewards["name"].as_str().unwrap(), rewards["cnt"]) }));
        }

        Ok(result)
    }
}
