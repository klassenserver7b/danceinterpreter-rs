use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StaticInfo {
    pub name: String,
    pub is_favorite: bool,
}

impl StaticInfo {
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        StaticInfo {
            name,
            is_favorite: false,
        }
    }
}
