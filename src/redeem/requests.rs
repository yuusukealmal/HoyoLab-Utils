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

async fn build_cookie_header() -> Result<HeaderMap, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    let account_mid_v2 = env::var("account_mid_v2")?;
    let cookie_token_v2 = match env::var("cookie_token_v2") {
        Ok(token) => token,
        Err(_) => {
            let (account, password) = get_account()?;
            utils::refresh::refresh_token(&account, &password).await?;
            env::var("cookie_token_v2")?
        }
    };

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

impl RedeemData {
    pub async fn redeem(
        &self,
        game: &RedeemGame,
        game_info: &HashMap<String, Option<String>>,
        headers: &mut HeaderMap,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut retry = true;
        loop {
            let response = self.send_redeem_request(game, game_info, headers).await?;

            if response["retcode"].as_i64() == Some(-1071) && retry {
                retry = false;
                tokio::time::sleep(Duration::from_secs(5)).await;
                let (account, password) = get_account()?;
                utils::refresh::refresh_token(&account, &password).await?;
                *headers = build_cookie_header().await?;
                continue;
            }

            return Ok(response["message"].to_string());
        }
    }

    async fn send_redeem_request(
        &self,
        game: &RedeemGame,
        game_info: &HashMap<String, Option<String>>,
        headers: &HeaderMap,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!(
            "https://{}.hoyoverse.com/common/apicdkey/api/webExchangeCdkey{}",
            game.domain,
            if game.method == "POST" { "Risk" } else { "" }
        );

        let json = json!({
            "lang": "zh-tw",
            "cdkey": self.cdkey,
            "uid": game_info.get("uid"),
            "game_biz": game_info.get("game_biz"),
            "region": game_info.get("region"),
        });

        let client = reqwest::Client::new();
        let response = client
            .request(game.method.parse()?, &url)
            .headers(headers.clone())
            .json(&json)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            Ok(serde_json::from_str(&body)?)
        } else {
            Err(format!("Redeem failed ({}): {}", status, body).into())
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
        let redeemed = json[self.name.as_str()]
            .as_array()
            .unwrap()
            .iter()
            .map(|code| code["cdkey"].as_str().unwrap_or_default().to_string())
            .collect::<Vec<String>>();
        for code in body["codes"].as_array().unwrap_or(&vec![]) {
            let (cdkey, reward) = (
                code["code"].as_str().unwrap_or_default().to_string(),
                code["rewards"].as_str().unwrap_or_default().to_string(),
            );
            if !redeemed.contains(&cdkey) {
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
            let result = code.redeem(self, game_info, &mut headers).await?;
            code.status = Some(result);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }

        Ok(())
    }
}
