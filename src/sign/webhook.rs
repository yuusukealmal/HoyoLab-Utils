use std::env;

use rand::Rng;
use reqwest::header;

fn parse_res(res: &Vec<serde_json::Value>) -> Vec<serde_json::Value> {
    let mut res = res.clone();
    let blank = serde_json::json!({"name": "** **", "value": "** **"});

    for i in (1..res.len()).rev() {
        res.insert(i, blank.clone());
    }

    res
}

pub async fn webhook(
    res: &Vec<serde_json::Value>,
    time: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let webhook_url = env::var("webhook_url").expect("webhook not set in environment");
    let data = serde_json::json!({
        "content": format!("<@{}> Finish check-in at `{}`", env::var("userid").expect("user_id not set in environment"), time),
        "embeds":
        [
            {
                "title": "HoyoLab 簽到",
                "color": rand::thread_rng().gen::<u32>() & 0xFFFFFF,
                "fields": parse_res(res)
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
