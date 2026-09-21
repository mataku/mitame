use std::collections::BTreeMap;
use std::fmt;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IdentityError {
    #[error("identity has no platform segment: {0}")]
    Empty(String),
    #[error("identity component is not in [a-z0-9_.-] or is reserved: {0}")]
    UnsafeComponent(String),
    #[error("variant pair is not key=value: {0}")]
    BadVariant(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identity {
    pub platform: String,
    pub group: Vec<String>,
    pub name: String,
    pub variant: BTreeMap<String, String>,
}

impl Identity {
    pub fn parse(id: &str) -> Result<Self, IdentityError> {
        let mut segments: Vec<&str> = id.split('/').filter(|s| !s.is_empty()).collect();
        if segments.len() < 2 {
            return Err(IdentityError::Empty(id.to_string()));
        }
        let stem = segments.pop().expect("checked length");
        let platform = segments.remove(0);
        let (name, variant_str) = match stem.split_once("__") {
            Some((n, v)) => (n, Some(v)),
            None => (stem, None),
        };
        let mut variant = BTreeMap::new();
        if let Some(v) = variant_str {
            for pair in v.split(',') {
                let (k, val) = pair
                    .split_once('=')
                    .ok_or_else(|| IdentityError::BadVariant(pair.to_string()))?;
                check_component(k)?;
                check_component(val)?;
                variant.insert(k.to_string(), val.to_string());
            }
        }
        check_component(platform)?;
        check_component(name)?;
        for g in &segments {
            check_component(g)?;
        }
        Ok(Identity {
            platform: platform.to_string(),
            group: segments.iter().map(|s| s.to_string()).collect(),
            name: name.to_string(),
            variant,
        })
    }

    pub fn from_relative_png(path: &str) -> Result<Self, IdentityError> {
        let trimmed = path.strip_suffix(".png").unwrap_or(path);
        Self::parse(&trimmed.replace('\\', "/"))
    }

    pub fn encoded_variant(&self) -> String {
        self.variant
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn stem(&self) -> String {
        if self.variant.is_empty() {
            self.name.clone()
        } else {
            format!("{}__{}", self.name, self.encoded_variant())
        }
    }

    pub fn id(&self) -> String {
        let mut parts = Vec::with_capacity(self.group.len() + 2);
        parts.push(self.platform.as_str());
        parts.extend(self.group.iter().map(|s| s.as_str()));
        let stem = self.stem();
        parts.push(stem.as_str());
        parts.join("/")
    }

    pub fn png_path(&self) -> String {
        format!("{}.png", self.id())
    }

    pub fn sidecar_path(&self) -> String {
        format!("{}.json", self.id())
    }

    pub fn group_string(&self) -> String {
        self.group.join("/")
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.id())
    }
}

fn check_component(value: &str) -> Result<(), IdentityError> {
    let safe = !value.is_empty()
        && !value.starts_with('.')
        && value != "."
        && value != ".."
        && !value.contains("__")
        && value.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'.' || b == b'-'
        });
    if safe {
        Ok(())
    } else {
        Err(IdentityError::UnsafeComponent(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_identity() {
        let id = Identity::parse("flutter/login/form/login_form__locale=ja,theme=dark").unwrap();
        assert_eq!(id.platform, "flutter");
        assert_eq!(id.group, vec!["login", "form"]);
        assert_eq!(id.name, "login_form");
        assert_eq!(id.variant.get("theme").map(String::as_str), Some("dark"));
        assert_eq!(
            id.id(),
            "flutter/login/form/login_form__locale=ja,theme=dark"
        );
    }

    #[test]
    fn empty_group_is_allowed() {
        let id = Identity::parse("ios/button").unwrap();
        assert!(id.group.is_empty());
        assert_eq!(id.png_path(), "ios/button.png");
    }

    #[test]
    fn variant_is_canonicalized_on_output() {
        let id = Identity::parse("android/card__theme=dark,locale=en").unwrap();
        assert_eq!(id.id(), "android/card__locale=en,theme=dark");
    }

    #[test]
    fn single_underscore_in_name_is_not_a_separator() {
        let id = Identity::parse("flutter/a/b_c").unwrap();
        assert_eq!(id.name, "b_c");
        assert!(id.variant.is_empty());
    }

    #[test]
    fn rejects_unsafe_components() {
        assert!(matches!(
            Identity::parse("flutter/Login/form"),
            Err(IdentityError::UnsafeComponent(_))
        ));
        assert!(matches!(
            Identity::parse("flutter/../form"),
            Err(IdentityError::UnsafeComponent(_))
        ));
        assert!(matches!(
            Identity::parse("flutter/a/b__theme"),
            Err(IdentityError::BadVariant(_))
        ));
    }

    #[test]
    fn from_relative_png_strips_extension_and_normalizes_separators() {
        let id = Identity::from_relative_png("flutter\\login\\form.png").unwrap();
        assert_eq!(id.id(), "flutter/login/form");
    }
}
