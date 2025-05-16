use chrono::{FixedOffset, Utc};

use super::webhook::webhook;
use crate::structs::structs::GAME;

pub async fn sign() -> Result<(), Box<dyn std::error::Error>> {
    let games: Vec<GAME> = vec![
        GAME {
            name: String::from("原神"),
            domain: String::from("hk4e"),
            biz: String::from("sol"),
            act_id: String::from("e202102251931481"),
            signgame: String::from("hk4e"),
        },
        GAME {
            name: String::from("崩壞三"),
            domain: String::from("public"),
            biz: String::from("mani"),
            act_id: String::from("e202110291205111"),
            signgame: String::from("bh3"),
        },
        GAME {
            name: String::from("崩壞：星穹鐵道"),
            domain: String::from("public"),
            biz: String::from("luna/os"),
            act_id: String::from("e202303301540311"),
            signgame: String::from("hkrpg"),
        },
        GAME {
            name: String::from("絕區零"),
            domain: String::from("public"),
            biz: String::from("luna/zzz/os"),
            act_id: String::from("e202406031448091"),
            signgame: String::from("zzz"),
        },
    ];

    let mut result: Vec<serde_json::Value> = vec![];
    for game in games {
        result.extend(game.sign().await?);
    }

    let offset = FixedOffset::east_opt(8 * 60 * 60).unwrap();
    let time = Utc::now()
        .with_timezone(&offset)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    webhook(&result, &time).await?;

    Ok(())
}
