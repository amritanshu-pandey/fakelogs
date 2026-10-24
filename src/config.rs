use serde::Deserialize;
use serde_yaml::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub patterns: Vec<PatternConfig>,
    /// Maximum events per second; 0 means uncapped.
    pub eps: u128,
}

#[derive(Debug, PartialEq)]
pub struct PatternConfig {
    pub pattern: String,
    /// How often this pattern is used relative to the others; 0 turns it off.
    pub weight: u32,
}

/// The config file as written. Each pattern is checked separately in
/// `PatternConfig::from_value`, so its errors can say which pattern is wrong.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    patterns: Vec<Value>,
    #[serde(default = "default_eps")]
    eps: u128,
}

/// Low enough to watch in a terminal when the config doesn't set `eps`.
fn default_eps() -> u128 {
    10
}

impl PatternConfig {
    /// Accepts either a plain string (weight 1) or `{pattern: ..., weight: ...}`.
    fn from_value(value: &Value) -> Result<Self, String> {
        let expected = "expected a string, or `pattern:` with an optional `weight:`";
        let mapping = match value {
            Value::String(pattern) => {
                return Ok(Self {
                    pattern: pattern.clone(),
                    weight: 1,
                })
            }
            Value::Mapping(mapping) => mapping,
            _ => return Err(expected.to_string()),
        };

        let mut pattern = None;
        let mut weight = 1;
        for (key, value) in mapping {
            match key.as_str() {
                Some("pattern") => match value {
                    Value::String(text) => pattern = Some(text.clone()),
                    _ => return Err("`pattern` must be a string".to_string()),
                },
                Some("weight") => {
                    weight = value
                        .as_u64()
                        .and_then(|w| u32::try_from(w).ok())
                        .ok_or("`weight` must be a whole number, 0 or more")?
                }
                _ => {
                    return Err(format!(
                        "unknown key {}, expected `pattern` or `weight`",
                        serde_yaml::to_string(key)
                            .map(|k| format!("`{}`", k.trim()))
                            .unwrap_or_default()
                    ))
                }
            }
        }
        let pattern = pattern.ok_or(format!("missing `pattern`; {}", expected))?;
        Ok(Self { pattern, weight })
    }
}

pub fn load(path: &Path) -> Result<Config, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("Unable to read config file {}: {}", path.display(), e))?;
    parse(&text).map_err(|e| format!("Invalid config file {}: {}", path.display(), e))
}

fn parse(text: &str) -> Result<Config, String> {
    // serde_yaml's messages include the line and column of the problem.
    let raw: RawConfig = serde_yaml::from_str(text).map_err(|e| e.to_string())?;
    if raw.patterns.is_empty() {
        return Err("`patterns` needs at least one pattern".to_string());
    }
    let patterns = raw
        .patterns
        .iter()
        .enumerate()
        .map(|(i, value)| {
            PatternConfig::from_value(value).map_err(|e| format!("pattern {}: {}", i + 1, e))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if patterns.iter().all(|p| p.weight == 0) {
        return Err("every pattern has weight 0, so nothing would be generated".to_string());
    }
    Ok(Config {
        patterns,
        eps: raw.eps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let config = parse("patterns:\n  - '{ipv4}'\neps: 5\n").unwrap();
        assert_eq!(
            config.patterns,
            vec![PatternConfig {
                pattern: "{ipv4}".to_string(),
                weight: 1
            }]
        );
        assert_eq!(config.eps, 5);
    }

    #[test]
    fn test_parse_weighted_patterns() {
        let text = "patterns:\n  - '{ipv4}'\n  - pattern: '{word}'\n    weight: 9\n  - pattern: x\n    weight: 0\n";
        let weights: Vec<u32> = parse(text)
            .unwrap()
            .patterns
            .iter()
            .map(|p| p.weight)
            .collect();
        assert_eq!(weights, vec![1, 9, 0]);
    }

    #[test]
    fn test_eps_defaults_to_10() {
        assert_eq!(parse("patterns: ['{ipv4}']").unwrap().eps, 10);
        assert_eq!(parse("patterns: ['{ipv4}']\neps: 0").unwrap().eps, 0);
    }

    #[test]
    fn test_parse_errors_are_descriptive() {
        let cases = [
            ("patterns: ['{ipv4}']\nepz: 5\n", "unknown field `epz`"),
            ("eps: 5\n", "missing field `patterns`"),
            ("patterns: []\n", "at least one pattern"),
            ("patterns: ['{ipv4}']\neps: -1\n", "eps"),
            ("patterns:\n  - '{ipv4}'\n  bad indent: x\n", "line"),
            (
                "patterns:\n  - '{ipv4}'\n  - pattern: x\n    wieght: 2\n",
                "pattern 2: unknown key `wieght`",
            ),
            ("patterns:\n  - weight: 2\n", "pattern 1: missing `pattern`"),
            (
                "patterns:\n  - pattern: x\n    weight: -1\n",
                "pattern 1: `weight` must be a whole number",
            ),
            (
                "patterns:\n  - pattern: x\n    weight: 1.5\n",
                "`weight` must be a whole number",
            ),
            ("patterns:\n  - 42\n", "pattern 1: expected a string"),
            (
                "patterns:\n  - pattern: x\n    weight: 0\n",
                "every pattern has weight 0",
            ),
        ];
        for (text, expected) in cases {
            let err = parse(text).unwrap_err();
            assert!(err.contains(expected), "{:?} gave {:?}", text, err);
        }
    }

    #[test]
    fn test_load_reports_path() {
        let err = load(Path::new("does/not/exist.yaml")).unwrap_err();
        assert!(err.contains("does/not/exist.yaml"), "{}", err);
    }
}
