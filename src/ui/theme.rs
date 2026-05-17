use std::{env, fs, path::PathBuf};

use serde::Deserialize;

#[derive(Clone, Copy, Debug)]
pub enum BannerStyle {
    Plain,
    Alternating,
    Gradient,
}

impl BannerStyle {
    fn from_str(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "plain" => BannerStyle::Plain,
            "alternating" => BannerStyle::Alternating,
            "gradient" => BannerStyle::Gradient,
            _ => BannerStyle::Alternating,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ThemeFile {
    name: Option<String>,

    prompt: String,
    continuation_prompt: String,
    result: String,
    error: String,

    banner_primary: String,
    banner_secondary: String,
    banner_title: String,
    banner_style: String,

    reset: String,
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,

    pub prompt: String,
    pub continuation_prompt: String,
    pub result: String,
    pub error: String,

    pub banner_primary: String,
    pub banner_secondary: String,
    pub banner_title: String,
    pub banner_style: BannerStyle,

    pub reset: String,
}

pub fn load_theme() -> Theme {
    let theme_name = env::var("VOIDLAMBDA_THEME")
        .unwrap_or_else(|_| "default".to_string());

    load_theme_by_name(&theme_name).unwrap_or_else(|err| {
        eprintln!("theme load error: {err}");
        eprintln!("falling back to built-in default theme");
        builtin_default()
    })
}

pub fn load_theme_by_name(name: &str) -> Result<Theme, String> {
    let path = theme_path(name);

    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("could not read {}: {err}", path.display()))?;

    let parsed: ThemeFile = toml::from_str(&contents)
        .map_err(|err| format!("could not parse {}: {err}", path.display()))?;

    Ok(Theme {
        name: parsed.name.unwrap_or_else(|| name.to_string()),

        prompt: parsed.prompt,
        continuation_prompt: parsed.continuation_prompt,
        result: parsed.result,
        error: parsed.error,

        banner_primary: parsed.banner_primary,
        banner_secondary: parsed.banner_secondary,
        banner_title: parsed.banner_title,
        banner_style: BannerStyle::from_str(&parsed.banner_style),

        reset: parsed.reset,
    })
}

fn theme_path(name: &str) -> PathBuf {
    PathBuf::from("themes").join(format!("{name}.toml"))
}

fn builtin_default() -> Theme {
    Theme {
        name: "builtin-default".to_string(),

        prompt: "\x1b[32m".to_string(),
        continuation_prompt: "\x1b[36m".to_string(),
        result: "\x1b[0m".to_string(),
        error: "\x1b[31m".to_string(),

        banner_primary: "\x1b[35m".to_string(),
        banner_secondary: "\x1b[36m".to_string(),
        banner_title: "\x1b[92m".to_string(),
        banner_style: BannerStyle::Alternating,

        reset: "\x1b[0m".to_string(),
    }
}