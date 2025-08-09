use std::str;

use base64::{engine::general_purpose, Engine};
use rand::thread_rng;
use regex::Regex;
use reqwest::Client;
use rsa::{pkcs1v15::Pkcs1v15Encrypt, pkcs8::DecodePublicKey, RsaPublicKey};
use serde_json::json;

use super::cookie_handle;
use super::random;

fn encrypt(message: &str) -> Result<String, Box<dyn std::error::Error>> {
    let public_key_pem = b"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA4PMS2JVMwBsOIrYWRluY
wEiFZL7Aphtm9z5Eu/anzJ09nB00uhW+ScrDWFECPwpQto/GlOJYCUwVM/raQpAj
/xvcjK5tNVzzK94mhk+j9RiQ+aWHaTXmOgurhxSp3YbwlRDvOgcq5yPiTz0+kSeK
ZJcGeJ95bvJ+hJ/UMP0Zx2qB5PElZmiKvfiNqVUk8A8oxLJdBB5eCpqWV6CUqDKQ
KSQP4sM0mZvQ1Sr4UcACVcYgYnCbTZMWhJTWkrNXqI8TMomekgny3y+d6NX/cFa6
6jozFIF4HCX5aW8bp8C8vq2tFvFbleQ/Q3CU56EWWKMrOcpmFtRmC18s9biZBVR/
8QIDAQAB
-----END PUBLIC KEY-----";

    let key = RsaPublicKey::from_public_key_pem(str::from_utf8(public_key_pem)?)?;
    let mut rng = thread_rng();
    let encrypted = key.encrypt(&mut rng, Pkcs1v15Encrypt, message.as_bytes())?;

    Ok(general_purpose::STANDARD.encode(encrypted))
}

pub async fn refresh_token(
    account: &str,
    password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let headers = json!(        {
        "x-rpc-app_id": "c9oqaq3s3gu8",
        "x-rpc-client_type": "4",
        "x-rpc-sdk_version": "2.14.1",
        "x-rpc-game_biz": "bbs_oversea",
        "x-rpc-source": "v2.webLogin",
        "x-rpc-referrer": "https://www.hoyolab.com",
        "Origin": "https://account.hoyolab.com",
        "Referer": "https://account.hoyolab.com/",
        "x-rpc-device_id": random::random_device_id(),
    });

    let json = json!({
        "account": encrypt(account)?,
        "password": encrypt(password)?,
        "token_type": 6,
    });

    let client = Client::new();
    let response = client
        .post("https://sg-public-api.hoyolab.com/account/ma-passport/api/webLoginByPassword")
        .headers(reqwest::header::HeaderMap::from_iter(
            headers.as_object().unwrap().iter().map(|(k, v)| {
                (
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                    reqwest::header::HeaderValue::from_str(v.as_str().unwrap()).unwrap(),
                )
            }),
        ))
        .json(&json)
        .send()
        .await?;

    let cookie_regex = Regex::new(r"cookie_token_v2=([^;]+)").unwrap();

    for value in response.headers().get_all("Set-Cookie") {
        let value_str = value.to_str()?;
        if value_str.contains("cookie_token_v2") {
            if let Some(caps) = cookie_regex.captures(value_str) {
                if let Some(token) = caps.get(1) {
                    let cookie_token_v2 = token.as_str().to_string();
                    cookie_handle::set_env("cookie_token_v2", &cookie_token_v2, ".env")?;
                    std::env::set_var("cookie_token_v2", &cookie_token_v2);
                    break;
                }
            }
        }
    }

    Ok(())
}
