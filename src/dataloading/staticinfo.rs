use iced::Color;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StaticInfo {
    pub name: String,
    pub is_favorite: bool,
    #[serde(with = "color_format", default)]
    pub color: Option<Color>,
}

impl StaticInfo {
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        StaticInfo {
            name,
            is_favorite: false,
            color: None,
        }
    }
}

mod color_format {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(color: &Option<Color>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match color {
            Some(c) => {
                let r = (c.r * 255.0) as u8;
                let g = (c.g * 255.0) as u8;
                let b = (c.b * 255.0) as u8;
                let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
                serializer.serialize_some(&hex)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(hex) => {
                let hex = hex.trim_start_matches('#');
                if hex.len() != 6 {
                    return Ok(None);
                }
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                Ok(Some(Color::from_rgb8(r, g, b)))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_serialization() {
        let static_info = StaticInfo {
            name: "Waltz".to_string(),
            is_favorite: true,
            color: Some(Color::from_rgb8(255, 0, 128)),
        };

        let json = serde_json::to_string(&static_info).unwrap();
        assert!(json.contains("\"color\":\"#FF0080\""));
    }

    #[test]
    fn test_color_deserialization() {
        let json = r##"{"name":"Tango","is_favorite":false,"color":"#00FF00"}"##;
        let static_info: StaticInfo = serde_json::from_str(json).unwrap();

        assert_eq!(static_info.name, "Tango");
        assert_eq!(static_info.color, Some(Color::from_rgb8(0, 255, 0)));
    }

    #[test]
    fn test_color_deserialization_invalid() {
        let json = r##"{"name":"Tango","is_favorite":false,"color":"invalid"}"##;
        let static_info: StaticInfo = serde_json::from_str(json).unwrap();

        assert_eq!(static_info.color, None);
    }
}
