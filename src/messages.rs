use chrono::serde::ts_milliseconds;
use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OutgoingMessageDTO {
    TextMessage(TextMessageDTO),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextMessageDTO {
    message: String,
    name: Option<String>,
    dice_results: Option<Vec<i32>>,
    #[serde(with = "ts_milliseconds")]
    time: DateTime<Utc>,
}

impl OutgoingMessageDTO {
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl TextMessageDTO {
    pub fn dice_result(message: &str, dice_results: &Vec<i32>, sender: &str) -> Self {
        TextMessageDTO {
            message: message.to_owned(),
            name: Some(sender.to_owned()),
            dice_results: Some(dice_results.clone()),
            time: Utc::now(),
        }
    }

    pub fn chat(message: &str, sender: &str) -> Self {
        TextMessageDTO {
            message: message.to_owned(),
            name: Some(sender.to_owned()),
            dice_results: None,
            time: Utc::now(),
        }
    }

    pub fn system(message: &str) -> Self {
        TextMessageDTO {
            message: message.to_owned(),
            name: None,
            dice_results: None,
            time: Utc::now(),
        }
    }
}
