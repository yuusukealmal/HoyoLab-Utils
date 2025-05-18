use std::{fs::File, path::Path};

use serde_json::Value;

use crate::{structs::structs::RedeemGame, utils::time::get_time};

fn is_missing(value: Option<&Option<String>>) -> bool {
    value
        .and_then(|v| v.as_ref())
        .map_or(true, |v| v.trim().is_empty())
}

pub async fn redeem() -> Result<(), Box<dyn std::error::Error>> {
    let games = vec![
        RedeemGame {
            name: "genshin".to_string(),
            domain: "sg-hk4e-api".to_string(),
            method: "GET".to_string(),
        },
        RedeemGame {
            name: "hkrpg".to_string(),
            domain: "public-operation-hkrpg".to_string(),
            method: "POST".to_string(),
        },
        RedeemGame {
            name: "nap".to_string(),
            domain: "public-operation-nap".to_string(),
            method: "POST".to_string(),
        },
    ];

    let map = ini!("config.ini");
    for game in games {
        let game_info = map.get(game.name.as_str()).unwrap();
        let uid = game_info.get("uid");
        let region = game_info.get("region");
        let game_biz = game_info.get("game_biz");

        if is_missing(uid) || is_missing(region) || is_missing(game_biz) {
            println!("uid, region, or game_biz is missing for {}", game.name);
            continue;
        }

        let mut codes = game.get_codes().await?;

        if !codes.is_empty() {
            game.redeem_codes(&mut codes).await?;

            let path = Path::new("codes.json");
            let mut json: Value = serde_json::from_reader(File::open(path)?)?;

            let arr = if let Some(arr) = json
                .get_mut(game.name.as_str())
                .and_then(Value::as_array_mut)
            {
                arr
            } else {
                json[game.name.as_str()] = Value::Array(vec![]);
                json[game.name.as_str()].as_array_mut().unwrap()
            };

            for code in &codes {
                let code_value = serde_json::to_value(code).unwrap();
                let cdkey = &code.cdkey;

                if let Some(existing) = arr.iter_mut().find(|v| v["cdkey"].as_str() == Some(cdkey))
                {
                    *existing = code_value;
                } else {
                    arr.push(code_value);
                }
            }

            serde_json::to_writer(File::create(path)?, &json)?;

            let time = get_time();
            game.webhook(&codes, &time).await?;
        }
    }

    Ok(())
}
