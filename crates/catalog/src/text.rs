//! A localization key or a literal text, and the stringtable's languages.

use serde::Serialize;

/// What a `name`/`description` attribute declares: in the DLC files a key
/// (`#THE_SAD_ONION_NAME`), in the base game a literal name. Kept as-is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Text {
    Key { key: String },
    Literal { text: String },
}

impl Text {
    pub fn from_attr(value: &str) -> Text {
        match value.strip_prefix('#') {
            Some(key) => Text::Key {
                key: key.to_string(),
            },
            None => Text::Literal {
                text: value.to_string(),
            },
        }
    }
}

/// The languages the game's stringtable declares. No Italian: it isn't there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Language {
    English,
    Japanese,
    Korean,
    ChineseSimplified,
    Russian,
    German,
    Spanish,
    French,
}

impl Language {
    /// From the `name` of `<language>`. A new language returns `None` and the reader diagnoses it.
    pub fn from_name(name: &str) -> Option<Language> {
        match name {
            "English" => Some(Language::English),
            "Japanese" => Some(Language::Japanese),
            "Korean" => Some(Language::Korean),
            "Chinese (Simple)" => Some(Language::ChineseSimplified),
            "Russian" => Some(Language::Russian),
            "German" => Some(Language::German),
            "Spanish" => Some(Language::Spanish),
            "French" => Some(Language::French),
            _ => None, // allowed: the input is an open string, not an enum of ours
        }
    }
}
