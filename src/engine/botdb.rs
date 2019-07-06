use crate::engine::errors::StringError;
use mongodb::{bson, db::ThreadedDatabase, doc, Client, ThreadedClient};

use std::str::FromStr;

impl From<mongodb::error::Error> for StringError {
    fn from(e: mongodb::error::Error) -> StringError {
        StringError::new(format!("MongoDB error: {}", e.to_string()).as_str())
    }
}

impl From<serde_json::Error> for StringError {
    fn from(e: serde_json::Error) -> StringError {
        StringError::new(format!("Error parsing JSON: {}", e.to_string()).as_str())
    }
}

impl From<bson::oid::Error> for StringError {
    fn from(e: bson::oid::Error) -> StringError {
        StringError::new(format!("BSON error: {}", e.to_string()).as_str())
    }
}

pub struct BotDB {
    client: Client,
}

impl Default for BotDB {
    fn default() -> Self {
        BotDB::new()
    }
}

impl BotDB {
    pub fn new() -> Self {
        BotDB {
            client: Client::connect("localhost", 27017).expect("Failed to connect to MongoDB"),
        }
    }

    pub fn save_bot(
        &mut self,
        bot_name: &str,
        recipe: &serde_json::Value,
        score: f32,
    ) -> Result<String, StringError> {
        let coll = self.client.db("naughts").collection("bots");
        let insert_result = coll
            .insert_one(
                doc! { "name": bot_name, "recipe": format!("{}", recipe), "score": score},
                None,
            )
            .unwrap();

        match insert_result.inserted_id {
            Some(x) => Ok(x.to_string()),
            None => match insert_result.write_exception {
                Some(x) => Err(StringError::new(
                    format!("Error saving bot: {}", x.message).as_str(),
                )),
                None => Err(StringError::new("Error saving bot")),
            },
        }
    }

    pub fn load_bot(&mut self, id: &str) -> Result<serde_json::Value, StringError> {
        let coll = self.client.db("naughts").collection("bots");
        let rec = coll.find_one(
            Some(doc! {"_id": bson::oid::ObjectId::with_string(id)?}),
            None,
        )?;
        let recipe = match rec {
            Some(x) => x
                .get("recipe")
                .ok_or_else(|| StringError::new("Bot not found"))?
                .to_string(),
            None => return Err(StringError::new("Bot not found")),
        };

        // recipes are enclosed in double quotes.
        Ok(serde_json::Value::from_str(recipe.trim_matches('"'))?)
    }
}