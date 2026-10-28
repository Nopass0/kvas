//! Interface languages. English strings in the code are the lookup keys: `t("Save")` returns the
//! text in the active language and falls back to the key itself, so a missing translation never
//! breaks the UI.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Debug)]
pub enum Lang {
    #[default]
    En,
    Ru,
}

pub const LANGUAGES: &[(Lang, &str)] = &[(Lang::En, "English"), (Lang::Ru, "Русский")];

impl Lang {
    pub fn name(self) -> &'static str {
        LANGUAGES.iter().find(|(l, _)| *l == self).map_or("English", |(_, n)| n)
    }

    /// Best guess from the OS environment, used on the very first start.
    pub fn detect() -> Lang {
        let locale = ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"].iter().find_map(|k| std::env::var(k).ok()).unwrap_or_default();
        if locale.to_ascii_lowercase().starts_with("ru") || system_is_russian() {
            Lang::Ru
        } else {
            Lang::En
        }
    }
}

#[cfg(windows)]
fn system_is_russian() -> bool {
    // The user profile locale shows up in the registry as e.g. "ru-RU".
    std::process::Command::new("reg")
        .args(["query", r"HKCU\Control Panel\International", "/v", "LocaleName"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_ascii_lowercase().contains("ru-"))
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn system_is_russian() -> bool {
    false
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn set(lang: Lang) {
    CURRENT.store(lang as u8, Ordering::Relaxed);
}

pub fn current() -> Lang {
    if CURRENT.load(Ordering::Relaxed) == Lang::Ru as u8 {
        Lang::Ru
    } else {
        Lang::En
    }
}

fn ru() -> &'static HashMap<&'static str, &'static str> {
    static MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| crate::lang_ru_v4::RU_V4.iter().chain(crate::lang_ru::RU.iter()).chain(crate::lang_ru_nodes::RU_NODES.iter()).copied().collect())
}

/// Translates a UI string.
pub fn t(key: &'static str) -> &'static str {
    match current() {
        Lang::En => key,
        Lang::Ru => ru().get(key).copied().unwrap_or(key),
    }
}

/// Translates a string that is not known at compile time (component and field names).
pub fn td(key: &str) -> String {
    match current() {
        Lang::En => key.to_string(),
        Lang::Ru => ru().get(key).map_or_else(|| key.to_string(), |s| s.to_string()),
    }
}

/// Translates a template and fills its `{}` placeholders in order.
pub fn tf(template: &'static str, args: &[&str]) -> String {
    let mut out = String::new();
    let mut rest = t(template);
    for a in args {
        match rest.find("{}") {
            Some(i) => {
                out.push_str(&rest[..i]);
                out.push_str(a);
                rest = &rest[i + 2..];
            }
            None => break,
        }
    }
    out.push_str(rest);
    out
}
