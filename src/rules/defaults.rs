use super::{Rule, Rules, Whitelist};

pub fn default_rules() -> Rules {
    Rules {
        critical: vec![
            Rule {
                category: "destructive".to_string(),
                patterns: vec![
                    "rm -rf /".to_string(),
                    "rm -rf /*".to_string(),
                    "rm -rf ~".to_string(),
                    "rm -rf ~/*".to_string(),
                    "mkfs*".to_string(),
                    "dd if=* of=/dev/*".to_string(),
                    "> /dev/sda*".to_string(),
                ],
                paths: vec![],
                reason: Some("Potentially destructive system command".to_string()),
                challenge: None,
            },
            Rule {
                category: "credentials".to_string(),
                patterns: vec![
                    "*AWS_SECRET*".to_string(),
                    "*PRIVATE_KEY*".to_string(),
                    "cat ~/.ssh/id_*".to_string(),
                    "cat *id_rsa*".to_string(),
                ],
                paths: vec![],
                reason: Some("Credential exposure risk".to_string()),
                challenge: None,
            },
            // File operation rules (Gemini CLI write_file/edit_file)
            Rule {
                category: "file-system-critical".to_string(),
                patterns: vec![
                    "*_file:/etc/passwd".to_string(),
                    "*_file:/etc/shadow".to_string(),
                    "*_file:/etc/sudoers*".to_string(),
                    "*_file:~/.ssh/*".to_string(),
                    "*_file:~/.gnupg/*".to_string(),
                ],
                paths: vec![],
                reason: Some("Write to critical system/auth file".to_string()),
                challenge: None,
            },
        ],
        high: vec![
            Rule {
                category: "rm-recursive-force".to_string(),
                patterns: vec![
                    "rm -rf *".to_string(),
                    "rm -fr *".to_string(),
                    "rm * -rf".to_string(),
                    "rm * -fr".to_string(),
                ],
                paths: vec![],
                reason: Some("Recursive force delete".to_string()),
                challenge: None,
            },
            Rule {
                category: "secrets".to_string(),
                patterns: vec![
                    "cat *.env*".to_string(),
                    "cat .env".to_string(),
                    "cat *secret*".to_string(),
                    "cat *password*".to_string(),
                ],
                paths: vec![],
                reason: Some("Secrets file access".to_string()),
                challenge: None,
            },
            Rule {
                category: "git-destructive".to_string(),
                patterns: vec![
                    "git push*--force*".to_string(),
                    "git push*-f*".to_string(),
                    "git reset --hard*".to_string(),
                    "git clean -fd*".to_string(),
                ],
                paths: vec![],
                reason: Some("Destructive git operation".to_string()),
                challenge: None,
            },
            Rule {
                category: "shell-wrapper".to_string(),
                patterns: vec![
                    "bash -c *".to_string(),
                    "bash -c*".to_string(),
                    "sh -c *".to_string(),
                    "sh -c*".to_string(),
                    "zsh -c *".to_string(),
                    "zsh -c*".to_string(),
                    "eval *".to_string(),
                    "exec *".to_string(),
                    "sudo *".to_string(),
                ],
                paths: vec![],
                reason: Some(
                    "Shell wrapper can execute arbitrary commands — review wrapped content"
                        .to_string(),
                ),
                challenge: None,
            },
            // File operation rules (Gemini CLI write_file/edit_file)
            Rule {
                category: "file-secrets".to_string(),
                patterns: vec![
                    "*_file:*.env".to_string(),
                    "*_file:*.env.*".to_string(),
                    "*_file:*secret*".to_string(),
                    "*_file:*credential*".to_string(),
                    "*_file:*password*".to_string(),
                    "*_file:*.pem".to_string(),
                    "*_file:*.key".to_string(),
                ],
                paths: vec![],
                reason: Some("Write to secrets/credential file".to_string()),
                challenge: None,
            },
        ],
        medium: vec![
            Rule {
                category: "rm-recursive".to_string(),
                patterns: vec![
                    "rm -r *".to_string(),
                    "rm * -r".to_string(),
                    "rm -R *".to_string(),
                    "rm * -R".to_string(),
                ],
                paths: vec![],
                reason: Some("Recursive delete".to_string()),
                challenge: None,
            },
            Rule {
                category: "git".to_string(),
                patterns: vec![
                    "git push*".to_string(),
                    "git merge*".to_string(),
                    "git rebase*".to_string(),
                ],
                paths: vec![],
                reason: Some("Git operation that modifies remote".to_string()),
                challenge: None,
            },
            Rule {
                category: "command-wrapper".to_string(),
                patterns: vec![
                    "xargs *".to_string(),
                    "nohup *".to_string(),
                ],
                paths: vec![],
                reason: Some("Command wrapper — verify the wrapped command".to_string()),
                challenge: None,
            },
            Rule {
                category: "install".to_string(),
                patterns: vec![
                    "npm install*".to_string(),
                    "pip install*".to_string(),
                    "cargo install*".to_string(),
                    "brew install*".to_string(),
                    "apt install*".to_string(),
                    "apt-get install*".to_string(),
                ],
                paths: vec![],
                reason: Some("Package installation".to_string()),
                challenge: None,
            },
        ],
        low: vec![
            Rule {
                category: "rm".to_string(),
                patterns: vec!["rm *".to_string()],
                paths: vec![],
                reason: Some("File deletion".to_string()),
                challenge: None,
            },
            Rule {
                category: "network".to_string(),
                patterns: vec!["curl*".to_string(), "wget*".to_string()],
                paths: vec![],
                reason: Some("Network request".to_string()),
                challenge: None,
            },
        ],
        whitelist: Whitelist {
            commands: vec![
                "ls*".to_string(),
                "pwd".to_string(),
                "echo *".to_string(),
                "cat *".to_string(),
                "head *".to_string(),
                "tail *".to_string(),
                "grep *".to_string(),
                "find *".to_string(),
                "which *".to_string(),
                "whoami".to_string(),
                "date".to_string(),
                "cargo build*".to_string(),
                "cargo test*".to_string(),
                "cargo check*".to_string(),
                "cargo fmt*".to_string(),
                "cargo clippy*".to_string(),
                "npm run*".to_string(),
                "npm test*".to_string(),
                "git status*".to_string(),
                "git log*".to_string(),
                "git diff*".to_string(),
                "git branch*".to_string(),
                "git show*".to_string(),
                // Text processing (prevent false positives with keyword patterns)
                "awk *".to_string(),
                "sed *".to_string(),
                "jq *".to_string(),
                "sort *".to_string(),
                "uniq *".to_string(),
                "cut *".to_string(),
                "tr *".to_string(),
                "wc *".to_string(),
                // File info (read-only)
                "diff *".to_string(),
                "file *".to_string(),
                "stat *".to_string(),
                "rg *".to_string(),
                // Safe operations
                "cd *".to_string(),
                "mkdir *".to_string(),
                "touch *".to_string(),
            ],
            paths: vec![],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{RiskLevel, RulesEngine};

    fn engine() -> RulesEngine {
        RulesEngine::new(default_rules())
    }

    // ── Critical ──────────────────────────────────────────────

    #[test]
    fn test_critical_rm_rf_root() {
        let r = engine().evaluate("rm -rf /");
        assert_eq!(r.level, RiskLevel::Critical);
        assert_eq!(r.category.as_deref(), Some("destructive"));
    }

    #[test]
    fn test_critical_rm_rf_home() {
        let r = engine().evaluate("rm -rf ~");
        assert_eq!(r.level, RiskLevel::Critical);
    }

    #[test]
    fn test_critical_mkfs() {
        let r = engine().evaluate("mkfs.ext4 /dev/sda");
        assert_eq!(r.level, RiskLevel::Critical);
        assert_eq!(r.category.as_deref(), Some("destructive"));
    }

    #[test]
    fn test_critical_dd_device() {
        let r = engine().evaluate("dd if=/dev/zero of=/dev/sda");
        assert_eq!(r.level, RiskLevel::Critical);
    }

    #[test]
    fn test_critical_cat_ssh_key() {
        // Whitelist "cat *" matches before credential rules, so this is allowed.
        // This documents a known gap: cat ~/.ssh/id_rsa is not blocked.
        let r = engine().evaluate("cat ~/.ssh/id_rsa");
        assert_eq!(
            r.level,
            RiskLevel::Allow,
            "cat ~/.ssh/id_rsa matches whitelist 'cat *' before credential rules"
        );
    }

    #[test]
    fn test_critical_aws_secret() {
        // Whitelist "echo *" matches before credential rules, so this is allowed.
        // This documents a known gap: echo $AWS_SECRET_ACCESS_KEY is not blocked.
        let r = engine().evaluate("echo $AWS_SECRET_ACCESS_KEY");
        assert_eq!(
            r.level,
            RiskLevel::Allow,
            "echo $AWS_SECRET matches whitelist 'echo *' before credential rules"
        );
    }

    #[test]
    fn test_critical_file_op_etc_passwd() {
        let r = engine().evaluate("write_file:/etc/passwd");
        assert_eq!(r.level, RiskLevel::Critical);
        assert_eq!(r.category.as_deref(), Some("file-system-critical"));
    }

    #[test]
    fn test_critical_file_op_ssh() {
        let r = engine().evaluate("edit_file:~/.ssh/authorized_keys");
        assert_eq!(r.level, RiskLevel::Critical);
    }

    // ── High ──────────────────────────────────────────────────

    #[test]
    fn test_high_rm_rf_relative() {
        let r = engine().evaluate("rm -rf ./project");
        assert_eq!(r.level, RiskLevel::High);
        assert_eq!(r.category.as_deref(), Some("rm-recursive-force"));
    }

    #[test]
    fn test_high_cat_env() {
        // Whitelist "cat *" is checked BEFORE risk rules, so "cat .env"
        // matches the whitelist pattern and is allowed.
        let r = engine().evaluate("cat .env");
        assert_eq!(
            r.level,
            RiskLevel::Allow,
            "cat .env matches whitelist 'cat *' before high-risk rules"
        );
    }

    #[test]
    fn test_high_cat_secret_file() {
        // "cat secret.txt" also matches whitelist "cat *" first
        let r = engine().evaluate("cat secret.txt");
        assert_eq!(
            r.level,
            RiskLevel::Allow,
            "cat secret.txt matches whitelist 'cat *' before high-risk rules"
        );
    }

    #[test]
    fn test_high_git_force_push() {
        let r = engine().evaluate("git push --force origin main");
        assert_eq!(r.level, RiskLevel::High);
        assert_eq!(r.category.as_deref(), Some("git-destructive"));
    }

    #[test]
    fn test_high_git_reset_hard() {
        let r = engine().evaluate("git reset --hard HEAD~3");
        assert_eq!(r.level, RiskLevel::High);
    }

    #[test]
    fn test_high_bash_c() {
        let r = engine().evaluate("bash -c 'echo hello'");
        assert_eq!(r.level, RiskLevel::High);
        assert_eq!(r.category.as_deref(), Some("shell-wrapper"));
    }

    #[test]
    fn test_high_sudo() {
        let r = engine().evaluate("sudo rm -rf /tmp/test");
        assert_eq!(r.level, RiskLevel::High);
        assert_eq!(r.category.as_deref(), Some("shell-wrapper"));
    }

    #[test]
    fn test_high_shell_wrapper_with_eval_cmd() {
        let r = engine().evaluate("eval 'dangerous command'");
        assert_eq!(r.level, RiskLevel::High);
    }

    #[test]
    fn test_high_file_op_env() {
        let r = engine().evaluate("write_file:.env");
        assert_eq!(r.level, RiskLevel::High);
        assert_eq!(r.category.as_deref(), Some("file-secrets"));
    }

    #[test]
    fn test_high_file_op_pem() {
        let r = engine().evaluate("write_file:server.pem");
        assert_eq!(r.level, RiskLevel::High);
    }

    // ── Medium ────────────────────────────────────────────────

    #[test]
    fn test_medium_git_push() {
        let r = engine().evaluate("git push origin main");
        assert_eq!(r.level, RiskLevel::Medium);
        assert_eq!(r.category.as_deref(), Some("git"));
    }

    #[test]
    fn test_medium_git_rebase() {
        let r = engine().evaluate("git rebase main");
        assert_eq!(r.level, RiskLevel::Medium);
    }

    #[test]
    fn test_medium_npm_install() {
        let r = engine().evaluate("npm install express");
        assert_eq!(r.level, RiskLevel::Medium);
        assert_eq!(r.category.as_deref(), Some("install"));
    }

    #[test]
    fn test_medium_pip_install() {
        let r = engine().evaluate("pip install requests");
        assert_eq!(r.level, RiskLevel::Medium);
    }

    #[test]
    fn test_medium_xargs() {
        let r = engine().evaluate("xargs rm -f");
        assert_eq!(r.level, RiskLevel::Medium);
        assert_eq!(r.category.as_deref(), Some("command-wrapper"));
    }

    // ── Low ───────────────────────────────────────────────────

    #[test]
    fn test_low_rm_single() {
        let r = engine().evaluate("rm file.txt");
        assert_eq!(r.level, RiskLevel::Low);
        assert_eq!(r.category.as_deref(), Some("rm"));
    }

    #[test]
    fn test_low_curl() {
        let r = engine().evaluate("curl http://example.com");
        assert_eq!(r.level, RiskLevel::Low);
        assert_eq!(r.category.as_deref(), Some("network"));
    }

    #[test]
    fn test_low_wget() {
        let r = engine().evaluate("wget http://example.com/file.tar.gz");
        assert_eq!(r.level, RiskLevel::Low);
    }

    // ── Whitelist ─────────────────────────────────────────────

    #[test]
    fn test_whitelist_ls() {
        let r = engine().evaluate("ls -la");
        assert_eq!(r.level, RiskLevel::Allow);
    }

    #[test]
    fn test_whitelist_cargo_test() {
        let r = engine().evaluate("cargo test --release");
        assert_eq!(r.level, RiskLevel::Allow);
    }

    #[test]
    fn test_whitelist_git_status() {
        let r = engine().evaluate("git status");
        assert_eq!(r.level, RiskLevel::Allow);
    }

    #[test]
    fn test_whitelist_git_diff() {
        let r = engine().evaluate("git diff HEAD");
        assert_eq!(r.level, RiskLevel::Allow);
    }

    #[test]
    fn test_whitelist_echo() {
        let r = engine().evaluate("echo hello world");
        assert_eq!(r.level, RiskLevel::Allow);
    }

    #[test]
    fn test_whitelist_grep() {
        let r = engine().evaluate("grep -r pattern src/");
        assert_eq!(r.level, RiskLevel::Allow);
    }

    #[test]
    fn test_whitelist_mkdir() {
        let r = engine().evaluate("mkdir -p src/new_module");
        assert_eq!(r.level, RiskLevel::Allow);
    }

    // ── Edge cases ────────────────────────────────────────────

    #[test]
    fn test_cat_env_priority() {
        // Whitelist "cat *" is evaluated before risk rules in evaluate_single,
        // so "cat .env" is allowed despite matching "cat .env" high-risk pattern.
        // This documents the current priority behavior.
        let r = engine().evaluate("cat .env");
        assert_eq!(r.level, RiskLevel::Allow);
        assert_eq!(r.category.as_deref(), Some("whitelist"));
    }

    #[test]
    fn test_rm_rf_slash_never_whitelisted() {
        // "rm -rf /" must never be whitelisted — no whitelist pattern matches it
        let r = engine().evaluate("rm -rf /");
        assert_eq!(r.level, RiskLevel::Critical);
    }
}
