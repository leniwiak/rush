#![allow(dead_code)]
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct RushConfig {
    pub prompt: String,
    pub aliases: HashMap<String, String>,
}
// `Default` settings for `MyConfig`
impl ::std::default::Default for RushConfig {
    fn default() -> Self {
        Self {
            prompt: "> ".into(),
            aliases: HashMap::new(),
        }
    }
}
