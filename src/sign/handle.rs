use super::webhook::webhook;
use crate::{structs::structs::SignGame, utils::time::get_time};

pub async fn sign() -> Result<(), Box<dyn std::error::Error>> {
    let games: Vec<SignGame> = vec![
        SignGame {
            name: String::from("原神"),
            domain: String::from("hk4e"),
            biz: String::from("sol"),
            act_id: String::from("e202102251931481"),
            signgame: String::from("hk4e"),
        },
        SignGame {
            name: String::from("崩壞三"),
            domain: String::from("public"),
            biz: String::from("mani"),
            act_id: String::from("e202110291205111"),
            signgame: String::from("bh3"),
        },
        SignGame {
            name: String::from("崩壞：星穹鐵道"),
            domain: String::from("public"),
            biz: String::from("luna/os"),
            act_id: String::from("e202303301540311"),
            signgame: String::from("hkrpg"),
        },
        SignGame {
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

    let time = get_time();
    webhook(&result, &time).await?;

    Ok(())
}
