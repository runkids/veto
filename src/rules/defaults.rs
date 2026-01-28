use super::{Rules, Rule, Whitelist};

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
                patterns: vec![
                    "rm *".to_string(),
                ],
                paths: vec![],
                reason: Some("File deletion".to_string()),
                challenge: None,
            },
            Rule {
                category: "network".to_string(),
                patterns: vec![
                    "curl*".to_string(),
                    "wget*".to_string(),
                ],
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
            ],
            paths: vec![],
        },
    }
}
