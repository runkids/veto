use glob::Pattern;
use super::{RiskLevel, RiskResult, Rules, Rule};

pub struct RulesEngine {
    rules: Rules,
}

impl RulesEngine {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }

    pub fn evaluate(&self, command: &str) -> RiskResult {
        // Check whitelist first
        if self.matches_whitelist(command) {
            return RiskResult {
                level: RiskLevel::Allow,
                category: Some("whitelist".to_string()),
                reason: Some("Command is whitelisted".to_string()),
                matched_pattern: None,
            };
        }

        // Check by priority: critical > high > medium > low
        if let Some(result) = self.check_rules(&self.rules.critical, command, RiskLevel::Critical) {
            return result;
        }
        if let Some(result) = self.check_rules(&self.rules.high, command, RiskLevel::High) {
            return result;
        }
        if let Some(result) = self.check_rules(&self.rules.medium, command, RiskLevel::Medium) {
            return result;
        }
        if let Some(result) = self.check_rules(&self.rules.low, command, RiskLevel::Low) {
            return result;
        }

        // Default: allow
        RiskResult {
            level: RiskLevel::Allow,
            category: None,
            reason: Some("No matching rules".to_string()),
            matched_pattern: None,
        }
    }

    fn matches_whitelist(&self, command: &str) -> bool {
        for pattern in &self.rules.whitelist.commands {
            if self.glob_match(pattern, command) {
                return true;
            }
        }
        false
    }

    fn check_rules(&self, rules: &[Rule], command: &str, level: RiskLevel) -> Option<RiskResult> {
        for rule in rules {
            for pattern in &rule.patterns {
                if self.glob_match(pattern, command) {
                    return Some(RiskResult {
                        level,
                        category: Some(rule.category.clone()),
                        reason: rule.reason.clone(),
                        matched_pattern: Some(pattern.clone()),
                    });
                }
            }
        }
        None
    }

    fn glob_match(&self, pattern: &str, text: &str) -> bool {
        // Handle wildcards
        if pattern.contains('*') {
            if let Ok(pat) = Pattern::new(pattern) {
                return pat.matches(text);
            }
            // Fallback: simple contains
            let core = pattern.trim_matches('*');
            return text.contains(core);
        }
        text == pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::Whitelist;

    fn create_test_rules() -> Rules {
        Rules {
            critical: vec![
                Rule {
                    category: "destructive".to_string(),
                    patterns: vec!["rm -rf /".to_string(), "rm -rf ~".to_string()],
                    paths: vec![],
                    reason: Some("Destructive command".to_string()),
                },
            ],
            high: vec![
                Rule {
                    category: "secrets".to_string(),
                    patterns: vec!["cat *.env*".to_string()],
                    paths: vec![],
                    reason: Some("Secrets access".to_string()),
                },
            ],
            medium: vec![
                Rule {
                    category: "git".to_string(),
                    patterns: vec!["git push*".to_string()],
                    paths: vec![],
                    reason: None,
                },
            ],
            low: vec![],
            whitelist: Whitelist {
                commands: vec!["ls".to_string(), "pwd".to_string(), "echo *".to_string()],
                paths: vec![],
            },
        }
    }

    #[test]
    fn test_whitelist_allows() {
        let engine = RulesEngine::new(create_test_rules());
        let result = engine.evaluate("ls");
        assert_eq!(result.level, RiskLevel::Allow);
    }

    #[test]
    fn test_critical_detection() {
        let engine = RulesEngine::new(create_test_rules());
        let result = engine.evaluate("rm -rf /");
        assert_eq!(result.level, RiskLevel::Critical);
        assert_eq!(result.category, Some("destructive".to_string()));
    }

    #[test]
    fn test_medium_detection() {
        let engine = RulesEngine::new(create_test_rules());
        let result = engine.evaluate("git push origin main");
        assert_eq!(result.level, RiskLevel::Medium);
    }

    #[test]
    fn test_unknown_command_allows() {
        let engine = RulesEngine::new(create_test_rules());
        let result = engine.evaluate("cargo build");
        assert_eq!(result.level, RiskLevel::Allow);
    }
}
