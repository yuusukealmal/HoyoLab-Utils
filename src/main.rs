use dotenv::dotenv;
use std::env;
use reqwest::cookie::Jar;
use std::sync::Arc;
use url::Url;
use chrono::{Utc, FixedOffset};
use rand::Rng;
use reqwest::header;

#[warn(dead_code)]
struct Method{
    name: &'static str,
    method: &'static str,
    cookie: i32
}

struct GAME{
    name: &'static str,
    domain: &'static str,
    biz: &'static str,
    act_id: &'static str
}

const METHODS: &[Method] = &[
    Method{name:"sign", method:"POST", cookie: 1},
    Method{name:"info", method:"GET", cookie: 1},
    Method{name:"home", method:"GET", cookie: 0},
];

const GAMES: &[GAME] = &[
    GAME{name:"原神", domain:"hk4e", biz:"sol", act_id:"e202102251931481"},
    GAME{name:"崩壞三", domain:"public", biz:"mani", act_id:"e202110291205111"},
    GAME{name:"崩壞：星穹鐵道", domain:"public", biz:"luna/os", act_id:"e202303301540311"},
    GAME{name:"絕區零", domain:"public", biz:"luna/zzz/os", act_id:"e202406031448091"}
];

fn set_cookies(jar: &Arc<Jar>, url: &Url) {
    dotenv().ok();

    let ltuid_v2 = env::var("ltuid_v2").expect("ltuid_v2 not set in environment");
    let ltoken_v2 = env::var("ltoken_v2").expect("ltoken_v2 not set in environment");

    jar.add_cookie_str(&format!("ltuid_v2={}", ltuid_v2), url);
    jar.add_cookie_str(&format!("ltoken_v2={}", ltoken_v2), url);
}

fn req(method: &Method, game: &GAME) -> serde_json::Value  {
    let jar = Arc::new(Jar::default());
    let mut headers = header::HeaderMap::new();
    if game.name == "絕區零" {
        headers.insert("x-rpc-signgame", header::HeaderValue::from_static("zzz"));
    }
    let client = reqwest::blocking::Client::builder().cookie_provider(jar.clone()).build().unwrap();
    let url = Url::parse(&format!("https://sg-{}-api.hoyolab.com/event/{}/{}?act_id={}&lang=zh-tw",game.domain, game.biz, method.name, game.act_id)).unwrap();

    if method.cookie == 1 {
        set_cookies(&jar, &url);
    }

    let r = client.request(reqwest::Method::from_bytes(method.method.as_bytes()).unwrap(),url).headers(headers).send().unwrap();
    let json: serde_json::Value = serde_json::from_str(r.text().unwrap().as_str()).unwrap();

    json
}

fn parse_res(res: &Vec<serde_json::Value>) -> Vec<serde_json::Value> {
    let mut res = res.clone();
    let blank = serde_json::json!({"name": "** **", "value": "** **"});

    for i in (1..res.len()).rev() {
        res.insert(i, blank.clone());
    }

    res
}

fn webhook(res: &Vec<serde_json::Value>, time: String) {
    dotenv().ok();

    let webhook = Url::parse(&env::var("webhook_url").expect("webhook not set in environment")).unwrap();
    let client = reqwest::blocking::Client::builder().build().unwrap();

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

    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));

    let r = client.post(webhook).headers(headers).body(data.to_string()).send().unwrap();
    println!("{}", r.text().unwrap());
}

fn main(){
    let mut res = Vec::new();

    for game in GAMES {
        let r = req(&METHODS[0], game);

        if r["retcode"].as_i64().unwrap() == -5003 {
            res.push(serde_json::json!({"name": game.name,"value": r["message"]}));
        }

        if r["message"].as_str().unwrap() == "OK" {
            let is_sign = req(&METHODS[1], game)["data"]["total_sign_day"].as_i64().unwrap();
            let sign_rewards = req(&METHODS[2], game);
            let sign_data = sign_rewards["data"].as_object().unwrap();
            let rewards = sign_data["awards"][is_sign as usize-1].as_object().unwrap();
            let total_days = sign_rewards["awards"].as_array().unwrap().len();
            res.push(serde_json::json!({"name": format!("{} {} / {}", game.name, is_sign, total_days), "value": format!("`{}` x{}", rewards["name"], rewards["cnt"]) }));
        }
    }

    let offset = FixedOffset::east_opt(8 * 60 * 60).unwrap();
    let time = Utc::now().with_timezone(&offset).format("%Y-%m-%d %H:%M:%S").to_string();

    webhook(&res, time);
}