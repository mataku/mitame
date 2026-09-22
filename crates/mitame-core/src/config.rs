use std::path::Path;

use globset::{Glob, GlobMatcher};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Paths {
    pub root: String,
}

impl Default for Paths {
    fn default() -> Self {
        Paths {
            root: ".mitame".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CompareConfig {
    pub algorithm: String,
    pub threshold: f64,
    pub max_diff_pixels: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_tolerance: Option<f64>,
    pub anti_aliasing: bool,
}

impl Default for CompareConfig {
    fn default() -> Self {
        CompareConfig {
            algorithm: "pixel".to_string(),
            threshold: 0.0,
            max_diff_pixels: 0,
            pixel_tolerance: None,
            anti_aliasing: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CaptureConfig {
    pub command: Vec<String>,
    pub fonts: String,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        CaptureConfig {
            command: Vec::new(),
            fonts: "real".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Policy {
    pub added: Severity,
    pub removed: Severity,
    pub mismatch: Severity,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            added: Severity::Warn,
            removed: Severity::Warn,
            mismatch: Severity::Fail,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "match")]
    pub pattern: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_diff_pixels: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pixel_tolerance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anti_aliasing: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub paths: Paths,
    pub capture: CaptureConfig,
    pub compare: CompareConfig,
    pub policy: Policy,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

pub const CONFIG_TEMPLATE: &str = r#"# mitame configuration. Every key is optional; the values below are the defaults.
# See docs/configuration.md in https://github.com/mataku/mitame for details.

[paths]
root = ".mitame"

[capture]
command = []               # the test command `mitame capture` and `mitame run` execute, e.g. ["flutter", "test"]
fonts = "real"             # Flutter only: "ahem" renders text as boxes so one baseline serves macOS and Linux

[compare]
threshold = 0.0            # allowed diff ratio (differing pixels / total pixels)
max_diff_pixels = 0        # allowed differing pixels; the larger allowance applies
anti_aliasing = true       # ignore pixels that only differ by anti-aliasing

[policy]
added = "warn"             # pass | warn | fail
removed = "warn"
mismatch = "fail"

# Per-identity overrides; the last matching rule wins.
# [[rules]]
# match = "flutter/**/*__*theme=dark*"
# max_diff_pixels = 40
"#;

#[derive(Debug, Clone, Copy)]
pub struct EffectiveCompare {
    pub threshold: f64,
    pub max_diff_pixels: u64,
    pub pixel_tolerance: f64,
    pub anti_aliasing: bool,
}

impl EffectiveCompare {
    pub fn allowance(&self, total_pixels: u64) -> f64 {
        (self.max_diff_pixels as f64).max(self.threshold * total_pixels as f64)
    }

    pub fn is_changed(&self, diff_pixels: u64, total_pixels: u64) -> bool {
        diff_pixels as f64 > self.allowance(total_pixels)
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
        toml::from_str(&text).map_err(|e| Error::Config(format!("{}: {e}", path.display())))
    }

    pub fn load_or_default(path: &Path) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            Ok(Self::default())
        }
    }

    pub fn matchers(&self) -> Result<Vec<(GlobMatcher, &Rule)>> {
        self.rules
            .iter()
            .map(|rule| {
                Glob::new(&rule.pattern)
                    .map(|g| (g.compile_matcher(), rule))
                    .map_err(|e| Error::Config(format!("rule `{}`: {e}", rule.pattern)))
            })
            .collect()
    }

    pub fn pixel_tolerance(&self) -> f64 {
        self.compare
            .pixel_tolerance
            .unwrap_or(if self.capture.fonts == "ahem" {
                0.2
            } else {
                0.1
            })
    }

    pub fn effective(&self, matchers: &[(GlobMatcher, &Rule)], id: &str) -> EffectiveCompare {
        let mut out = EffectiveCompare {
            threshold: self.compare.threshold,
            max_diff_pixels: self.compare.max_diff_pixels,
            pixel_tolerance: self.pixel_tolerance(),
            anti_aliasing: self.compare.anti_aliasing,
        };
        for (matcher, rule) in matchers {
            if matcher.is_match(id) {
                if let Some(t) = rule.threshold {
                    out.threshold = t;
                }
                if let Some(m) = rule.max_diff_pixels {
                    out.max_diff_pixels = m;
                }
                if let Some(p) = rule.pixel_tolerance {
                    out.pixel_tolerance = p;
                }
                if let Some(a) = rule.anti_aliasing {
                    out.anti_aliasing = a;
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_matching_rule_wins() {
        let config: Config = toml::from_str(
            r#"
            [compare]
            threshold = 0.001

            [[rules]]
            match = "flutter/**"
            threshold = 0.005

            [[rules]]
            match = "flutter/**/*__*theme=dark*"
            threshold = 0.01
            "#,
        )
        .unwrap();
        let matchers = config.matchers().unwrap();
        assert_eq!(config.effective(&matchers, "ios/a/b").threshold, 0.001);
        assert_eq!(config.effective(&matchers, "flutter/a/b").threshold, 0.005);
        assert_eq!(
            config
                .effective(&matchers, "flutter/a/b__locale=ja,theme=dark")
                .threshold,
            0.01
        );
    }

    #[test]
    fn defaults_apply_when_sections_are_missing() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.paths.root, ".mitame");
        assert_eq!(config.policy.mismatch, Severity::Fail);
        assert!(config.compare.anti_aliasing);
        assert_eq!(config.compare.threshold, 0.0);
        assert_eq!(config.compare.max_diff_pixels, 0);
    }

    #[test]
    fn allowance_is_the_larger_of_pixels_and_ratio() {
        let strict = EffectiveCompare {
            threshold: 0.0,
            max_diff_pixels: 0,
            pixel_tolerance: 0.1,
            anti_aliasing: true,
        };
        assert!(strict.is_changed(1, 1000));
        assert!(!strict.is_changed(0, 1000));
        let pixels = EffectiveCompare {
            max_diff_pixels: 20,
            ..strict
        };
        assert!(!pixels.is_changed(20, 1000));
        assert!(pixels.is_changed(21, 1000));
        let ratio = EffectiveCompare {
            threshold: 0.05,
            ..strict
        };
        assert!(!ratio.is_changed(50, 1000));
        assert!(ratio.is_changed(51, 1000));
        let both = EffectiveCompare {
            threshold: 0.001,
            max_diff_pixels: 20,
            ..strict
        };
        assert!(!both.is_changed(20, 1000));
        assert!(both.is_changed(21, 1000));
    }
}

#[cfg(test)]
mod template_tests {
    use super::*;

    #[test]
    fn template_parses_to_the_defaults() {
        let parsed: Config = toml::from_str(CONFIG_TEMPLATE).unwrap();
        assert_eq!(
            toml::to_string(&parsed).unwrap(),
            toml::to_string(&Config::default()).unwrap()
        );
    }
}

#[cfg(test)]
mod tolerance_tests {
    use super::*;

    #[test]
    fn ahem_widens_the_pixel_tolerance_unless_set_explicitly() {
        let mut config = Config::default();
        assert_eq!(config.pixel_tolerance(), 0.1);
        config.capture.fonts = "ahem".to_string();
        assert_eq!(config.pixel_tolerance(), 0.2);
        config.compare.pixel_tolerance = Some(0.05);
        assert_eq!(config.pixel_tolerance(), 0.05);
    }
}
