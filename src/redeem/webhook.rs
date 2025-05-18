use std::env;

use rand::Rng;
use reqwest::header;

use crate::structs::structs::{RedeemData, RedeemGame};

impl RedeemGame {
    pub async fn webhook(
        &self,
        codes: &Vec<RedeemData>,
        time: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let webhook_url = env::var("webhook_url").expect("webhook not set in environment");
        let data = serde_json::json!({
            "content": format!("<@{}> Finish Redeem For {} At `{}`", env::var("userid").expect("user_id not set in environment"), self.name, time),
            "embeds":
            [
                {
                    "title": format!("{} 兌換碼兌換", self.name),
                    "color": rand::thread_rng().gen::<u32>() & 0xFFFFFF,
                    "fields": codes.iter().map(|code| {
                        serde_json::json!({
                            "name": format!("**{}**", code.cdkey),
                            "value": format!("`{}`", if code.status.as_ref().unwrap().contains("❌ Redeem failed") {
                                code.status.as_ref().unwrap().clone()
                            } else if code.reward.is_empty() {
                                "Reward Not Displayed".to_string()
                            } else {
                                code.reward.clone()
                            }),
                            "inline": false
                        })
                    }).collect::<Vec<_>>()
                }
            ],
            "username": "アストローギスト・モナ・メギストス",
            "attachments": []
        });

        let client = reqwest::Client::new();
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        client
            .post(webhook_url)
            .headers(headers)
            .body(data.to_string())
            .send()
            .await?;

        Ok(())
    }
}
