use super::{ExplainEntry, ExplainResult, RiskLevel, RiskResult, Rule, Rules, SubcommandTrace};
use glob::Pattern;

pub struct RulesEngine {
    rules: Rules,
}

impl RulesEngine {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }

    pub fn evaluate(&self, command: &str) -> RiskResult {
        // Split compound commands and evaluate each part
        // Return highest risk level found
        let subcommands = self.split_compound_command(command);

        let mut highest_result: Option<RiskResult> = None;

        for subcmd in &subcommands {
            let result = self.evaluate_single(subcmd);

            // Keep track of highest risk level
            match &highest_result {
                None => highest_result = Some(result),
                Some(current) => {
                    if self.risk_level_priority(&result.level)
                        > self.risk_level_priority(&current.level)
                    {
                        highest_result = Some(result);
                    }
                }
            }
        }

        highest_result.unwrap_or(RiskResult {
            level: RiskLevel::Allow,
            category: None,
            reason: Some("No matching rules".to_string()),
            matched_pattern: None,
            challenge: false,
        })
    }

    /// Split compound commands by &&, ||, ; while respecting quotes
    fn split_compound_command(&self, command: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = command.chars().peekable();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escape_next = false;

        while let Some(c) = chars.next() {
            if escape_next {
                current.push(c);
                escape_next = false;
                continue;
            }

            if c == '\\' {
                escape_next = true;
                current.push(c);
                continue;
            }

            if c == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                current.push(c);
                continue;
            }

            if c == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                current.push(c);
                continue;
            }

            // Only split if not inside quotes
            if !in_single_quote && !in_double_quote {
                // Check for && or ||
                if (c == '&' || c == '|') && chars.peek() == Some(&c) {
                    chars.next(); // consume second char
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        parts.push(trimmed);
                    }
                    current = String::new();
                    continue;
                }

                // Check for ;
                if c == ';' {
                    let trimmed = current.trim().to_string();
                    if !trimmed.is_empty() {
                        parts.push(trimmed);
                    }
                    current = String::new();
                    continue;
                }
            }

            current.push(c);
        }

        let trimmed = current.trim().to_string();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }

        if parts.is_empty() {
            parts.push(command.to_string());
        }

        parts
    }

    fn risk_level_priority(&self, level: &RiskLevel) -> u8 {
        match level {
            RiskLevel::Allow => 0,
            RiskLevel::Low => 1,
            RiskLevel::Medium => 2,
            RiskLevel::High => 3,
            RiskLevel::Critical => 4,
        }
    }

    fn evaluate_single(&self, command: &str) -> RiskResult {
        // Check whitelist first
        if self.matches_whitelist(command) {
            return RiskResult {
                level: RiskLevel::Allow,
                category: Some("whitelist".to_string()),
                reason: Some("Command is whitelisted".to_string()),
                matched_pattern: None,
                challenge: false,
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
            challenge: false,
        }
    }

    pub fn evaluate_with_trace(&self, command: &str) -> ExplainResult {
        let subcommands = self.split_compound_command(command);
        let mut traces = Vec::new();
        let mut highest_result: Option<RiskResult> = None;

        for subcmd in &subcommands {
            let trace = self.evaluate_single_with_trace(subcmd);

            match &highest_result {
                None => highest_result = Some(trace.result.clone()),
                Some(current) => {
                    if self.risk_level_priority(&trace.result.level)
                        > self.risk_level_priority(&current.level)
                    {
                        highest_result = Some(trace.result.clone());
                    }
                }
            }
            traces.push(trace);
        }

        let final_result = highest_result.unwrap_or(RiskResult {
            level: RiskLevel::Allow,
            category: None,
            reason: Some("No matching rules".to_string()),
            matched_pattern: None,
            challenge: false,
        });

        ExplainResult {
            subcommands: traces,
            final_result,
        }
    }

    fn evaluate_single_with_trace(&self, command: &str) -> SubcommandTrace {
        let mut entries = Vec::new();

        // Check whitelist
        let whitelist_match = self.find_whitelist_match(command);
        if whitelist_match.is_some() {
            return SubcommandTrace {
                command: command.to_string(),
                whitelist_match,
                entries,
                result: RiskResult {
                    level: RiskLevel::Allow,
                    category: Some("whitelist".to_string()),
                    reason: Some("Command is whitelisted".to_string()),
                    matched_pattern: None,
                    challenge: false,
                },
            };
        }

        // Check all levels, collecting trace
        let levels: &[(RiskLevel, &[Rule])] = &[
            (RiskLevel::Critical, &self.rules.critical),
            (RiskLevel::High, &self.rules.high),
            (RiskLevel::Medium, &self.rules.medium),
            (RiskLevel::Low, &self.rules.low),
        ];

        let mut matched_result: Option<RiskResult> = None;

        for (level, rules) in levels {
            for rule in *rules {
                for pattern in &rule.patterns {
                    let matched = self.glob_match(pattern, command);
                    entries.push(ExplainEntry {
                        level: *level,
                        category: rule.category.clone(),
                        pattern: pattern.clone(),
                        matched,
                        reason: rule.reason.clone(),
                    });
                    if matched && matched_result.is_none() {
                        matched_result = Some(RiskResult {
                            level: *level,
                            category: Some(rule.category.clone()),
                            reason: rule.reason.clone(),
                            matched_pattern: Some(pattern.clone()),
                            challenge: rule.challenge.unwrap_or(false),
                        });
                    }
                }
            }
        }

        let result = matched_result.unwrap_or(RiskResult {
            level: RiskLevel::Allow,
            category: None,
            reason: Some("No matching rules".to_string()),
            matched_pattern: None,
            challenge: false,
        });

        SubcommandTrace {
            command: command.to_string(),
            whitelist_match: None,
            entries,
            result,
        }
    }

    fn find_whitelist_match(&self, command: &str) -> Option<String> {
        for pattern in &self.rules.whitelist.commands {
            if self.glob_match(pattern, command) {
                return Some(pattern.clone());
            }
        }
        None
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
                        challenge: rule.challenge.unwrap_or(false),
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
            critical: vec![Rule {
                category: "destructive".to_string(),
                patterns: vec!["rm -rf /".to_string(), "rm -rf ~".to_string()],
                paths: vec![],
                reason: Some("Destructive command".to_string()),
                challenge: Some(true),
            }],
            high: vec![Rule {
                category: "secrets".to_string(),
                patterns: vec!["cat *.env*".to_string()],
                paths: vec![],
                reason: Some("Secrets access".to_string()),
                challenge: None,
            }],
            medium: vec![Rule {
                category: "git".to_string(),
                patterns: vec!["git push*".to_string()],
                paths: vec![],
                reason: None,
                challenge: None,
            }],
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

    #[test]
    fn test_challenge_field_propagation() {
        let engine = RulesEngine::new(create_test_rules());
        // Critical rule has challenge = true
        let result = engine.evaluate("rm -rf /");
        assert!(result.challenge, "Critical rule should have challenge=true");

        // High rule has challenge = None (defaults to false)
        let result = engine.evaluate("cat .env");
        assert!(
            !result.challenge,
            "High rule without challenge should default to false"
        );
    }

    #[test]
    fn test_compound_command_and() {
        let engine = RulesEngine::new(create_test_rules());
        // cd && rm -rf / should detect critical
        let result = engine.evaluate("cd /tmp && rm -rf /");
        assert_eq!(result.level, RiskLevel::Critical);
    }

    #[test]
    fn test_compound_command_semicolon() {
        let engine = RulesEngine::new(create_test_rules());
        // echo hello; rm -rf ~ should detect critical
        let result = engine.evaluate("echo hello; rm -rf ~");
        assert_eq!(result.level, RiskLevel::Critical);
    }

    #[test]
    fn test_compound_command_or() {
        let engine = RulesEngine::new(create_test_rules());
        // false || rm -rf / should detect critical
        let result = engine.evaluate("false || rm -rf /");
        assert_eq!(result.level, RiskLevel::Critical);
    }

    #[test]
    fn test_quoted_separators_not_split() {
        let engine = RulesEngine::new(create_test_rules());
        // echo "a && b" should not be split, should allow
        let result = engine.evaluate("echo \"a && rm -rf /\"");
        assert_eq!(result.level, RiskLevel::Allow);
    }

    #[test]
    fn test_explain_trace_whitelist() {
        let engine = RulesEngine::new(create_test_rules());
        let result = engine.evaluate_with_trace("ls");
        assert_eq!(result.subcommands.len(), 1);
        assert!(result.subcommands[0].whitelist_match.is_some());
        assert_eq!(result.final_result.level, RiskLevel::Allow);
    }

    #[test]
    fn test_explain_trace_match() {
        let engine = RulesEngine::new(create_test_rules());
        let result = engine.evaluate_with_trace("rm -rf /");
        assert_eq!(result.subcommands.len(), 1);
        let trace = &result.subcommands[0];
        assert!(trace.whitelist_match.is_none());
        assert!(!trace.entries.is_empty());
        assert!(trace.entries.iter().any(|e| e.matched));
        assert_eq!(result.final_result.level, RiskLevel::Critical);
    }

    #[test]
    fn test_explain_trace_compound() {
        let engine = RulesEngine::new(create_test_rules());
        let result = engine.evaluate_with_trace("echo hello && rm -rf /");
        assert_eq!(result.subcommands.len(), 2);
        assert_eq!(result.subcommands[0].result.level, RiskLevel::Allow);
        assert_eq!(result.subcommands[1].result.level, RiskLevel::Critical);
        assert_eq!(result.final_result.level, RiskLevel::Critical);
    }

    #[test]
    fn test_explain_trace_whitelisted_command() {
        let engine = RulesEngine::new(create_test_rules());
        let result = engine.evaluate_with_trace("pwd");
        assert_eq!(result.subcommands.len(), 1);
        let trace = &result.subcommands[0];
        assert!(trace.whitelist_match.is_some());
        assert_eq!(result.final_result.level, RiskLevel::Allow);
    }

    #[test]
    fn test_explain_trace_unknown_command() {
        let engine = RulesEngine::new(create_test_rules());
        let result = engine.evaluate_with_trace("foo bar");
        let trace = &result.subcommands[0];
        assert!(trace.whitelist_match.is_none());
        assert!(trace.entries.iter().all(|e| !e.matched));
        assert_eq!(result.final_result.level, RiskLevel::Allow);
    }

    #[test]
    fn test_split_compound_command() {
        let engine = RulesEngine::new(create_test_rules());
        let parts = engine.split_compound_command("cd /tmp && rm -rf test");
        assert_eq!(parts, vec!["cd /tmp", "rm -rf test"]);

        let parts = engine.split_compound_command("a; b; c");
        assert_eq!(parts, vec!["a", "b", "c"]);

        let parts = engine.split_compound_command("echo \"a && b\"");
        assert_eq!(parts, vec!["echo \"a && b\""]);
    }
}
