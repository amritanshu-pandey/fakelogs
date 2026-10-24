use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub patterns: Vec<String>,
    /// Maximum events per second; 0 means uncapped.
    #[serde(default = "default_eps")]
    pub eps: u128,
}

/// Low enough to watch in a terminal when the config doesn't set `eps`.
fn default_eps() -> u128 {
    10
}

pub fn load(path: &Path) -> Result<Config, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("Unable to read config file {}: {}", path.display(), e))?;
    parse(&text).map_err(|e| format!("Invalid config file {}: {}", path.display(), e))
}

fn parse(text: &str) -> Result<Config, String> {
    // serde_yaml's messages include the line and column of the problem.
    let config: Config = serde_yaml::from_str(text).map_err(|e| e.to_string())?;
    if config.patterns.is_empty() {
        return Err("`patterns` needs at least one pattern".to_string());
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let config = parse("patterns:\n  - '{ipv4}'\neps: 5\n").unwrap();
        assert_eq!(config.patterns, vec!["{ipv4}"]);
        assert_eq!(config.eps, 5);
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
