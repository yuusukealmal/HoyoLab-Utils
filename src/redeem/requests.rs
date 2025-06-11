use std::{collections::HashMap, env, fs::File, time::Duration};

use reqwest::{
    self,
    header::{HeaderMap, COOKIE},
};
use serde_json::{json, Value};

use crate::{
    structs::structs::{RedeemData, RedeemGame},
    utils,
};

async fn get_or_refresh_token() -> Result<String, Box<dyn std::error::Error>> {
    match env::var("cookie_token_v2") {
        Ok(token) => Ok(token),
        Err(_) => {
            let (account, password) = get_account()?;
            utils::refresh::refresh_token(&account, &password).await?;
            Ok(env::var("cookie_token_v2")?)
        }
    }
}

async fn build_cookie_header() -> Result<HeaderMap, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    let account_mid_v2 = env::var("account_mid_v2")?;
    let cookie_token_v2 = get_or_refresh_token().await?;

    let cookie = format!(
        "account_mid_v2={};cookie_token_v2={}",
        account_mid_v2, cookie_token_v2
    );
    headers.insert(COOKIE, cookie.parse()?);
    Ok(headers)
}

fn get_account() -> Result<(String, String), Box<dyn std::error::Error>> {
    let account = env::var("account")?;
    let password = env::var("password")?;
    Ok((account, password))
}

const TOKEN_EXPIRED_RETCODE: i64 = -1071;
impl RedeemData {
    pub async fn redeem(
        &self,
        game: &RedeemGame,
        game_info: &HashMap<String, Option<String>>,
        headers: &mut HeaderMap,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut retried = false;

        loop {
            let response = self.send_redeem_request(game, game_info, headers).await?;

            match response["retcode"].as_i64() {
                Some(TOKEN_EXPIRED_RETCODE) if !retried => {
                    retried = true;
                    eprintln!("⚠️ Token expired. Attempting refresh...");
                    tokio::time::sleep(Duration::from_secs(5)).await;

                    let (account, password) = get_account()?;
                    utils::refresh::refresh_token(&account, &password).await?;
                    *headers = build_cookie_header().await?;
                    continue;
                }
                _ => {
                    println!("{}", response["message"]);
                    return Ok(response["message"]
                        .as_str()
                        .unwrap_or("未知錯誤")
                        .to_string());
                }
            }
        }
    }

    async fn send_redeem_request(
        &self,
        game: &RedeemGame,
        game_info: &HashMap<String, Option<String>>,
        headers: &HeaderMap,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let uid = game_info
            .get("uid")
            .and_then(|v| v.clone())
            .unwrap_or_default();
        let game_biz = game_info
            .get("game_biz")
            .and_then(|v| v.clone())
            .unwrap_or_default();
        let region = game_info
            .get("region")
            .and_then(|v| v.clone())
            .unwrap_or_default();
        let url = format!(
            "https://{}.hoyoverse.com/common/apicdkey/api/webExchangeCdkey{}",
            game.domain,
            if game.method == "POST" { "Risk" } else { "" }
        );

        let client = reqwest::Client::new();
        let request = client
            .request(game.method.parse()?, &url)
            .headers(headers.clone());

        let response = if game.method == "GET" {
            let query = [
                ("lang", "zh-tw"),
                ("cdkey", &self.cdkey),
                ("uid", &uid),
                ("game_biz", &game_biz),
                ("region", &region),
            ];
            request.query(&query).send().await?
        } else {
            let json = json!({
                "lang": "zh-tw",
                "cdkey": self.cdkey,
                "uid": uid,
                "game_biz": game_biz,
                "region": region,
            });
            request.json(&json).send().await?
        };

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(serde_json::from_str(&body)?)
        } else {
            Err(format!("❌ Redeem failed {}: {}", status, body).into())
        }
    }
}

impl RedeemGame {
    pub async fn get_codes(&self) -> Result<Vec<RedeemData>, Box<dyn std::error::Error>> {
        let mut codes = vec![];

        let client = reqwest::Client::new();
        let url = format!("https://hoyo-codes.seria.moe/codes?game={}", self.name);

        let response = client.get(url).send().await?;
        let body = response.json::<Value>().await?;

        let json: Value = serde_json::from_reader(File::open("codes.json")?)?;
        let redeemed: HashMap<String, String> = json[self.name.as_str()]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|code| {
                Some((
                    code["cdkey"].as_str()?.to_string(),
                    code["status"].as_str().unwrap_or("").to_string(),
                ))
            })
            .collect();

        for code in body["codes"].as_array().unwrap() {
            let cdkey = code["code"].as_str().unwrap_or_default().to_string();
            let reward = code["rewards"].as_str().unwrap_or_default().to_string();

            let judege = match redeemed.get(&cdkey) {
                None => true,
                Some(status) => status.contains("❌ Redeem failed"),
            };

            if judege {
                codes.push(RedeemData::new(cdkey, reward));
            }
        }
        Ok(codes)
    }

    pub async fn redeem_codes(
        &self,
        codes: &mut Vec<RedeemData>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = build_cookie_header().await?;

        let map = ini!("config.ini");
        let game_info = map
            .get(self.name.as_str())
            .ok_or("Game info not found in config.ini")?;

        for code in codes.iter_mut() {
            let result = match code.redeem(self, game_info, &mut headers).await {
                Ok(result) => result,
                Err(e) => e.to_string(),
            };
            code.status = Some(result);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        Ok(())
    }
}
