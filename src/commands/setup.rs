use colored::Colorize;
use std::fs;
use std::path::PathBuf;

/// Get Claude Code settings path
fn get_claude_settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("settings.json"))
}

/// Check if veto hooks are already configured in Claude Code
pub fn is_claude_configured() -> bool {
    let Some(path) = get_claude_settings_path() else {
        return false;
    };

    if !path.exists() {
        return false;
    }

    let Ok(content) = fs::read_to_string(&path) else {
        return false;
    };

    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };

    // Check if veto hook exists in PreToolUse
    json["hooks"]["PreToolUse"]
        .as_array()
        .map(|hooks| {
            hooks.iter().any(|h| {
                h["hooks"]
                    .as_array()
                    .map(|inner| {
                        inner.iter().any(|ih| {
                            ih["command"]
                                .as_str()
                                .map(|c| c.contains("veto gate"))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Setup Claude Code hooks integration
pub fn run_setup_claude(uninstall: bool) -> Result<(), Box<dyn std::error::Error>> {
    let settings_path = get_claude_settings_path()
        .ok_or("Cannot find home directory")?;

    println!("{}", "Setting up Claude Code integration...".bold());
    println!();

    if uninstall {
        remove_veto_hooks(&settings_path)?;
        println!("  {} Removed veto hooks from Claude Code", "✓".green());
        println!();
        println!("Restart Claude Code for changes to take effect.");
    } else {
        add_veto_hooks(&settings_path)?;
        println!("  {} Added veto hooks to PreToolUse", "✓".green());
        println!();
        println!("Done! Restart Claude Code for changes to take effect.");
    }

    Ok(())
}

/// Add veto hooks to Claude Code settings
fn add_veto_hooks(settings_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Read existing settings or create new
    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(settings_path)?;
        serde_json::from_str(&content)?
    } else {
        // Create parent directory if needed
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)?;
        }
        serde_json::json!({})
    };

    // Create veto hook configuration
    let veto_hook = serde_json::json!({
        "matcher": "Bash",
        "hooks": [{
            "type": "command",
            "command": "veto gate --claude"
        }]
    });

    // Get or create hooks object
    if settings.get("hooks").is_none() {
        settings["hooks"] = serde_json::json!({});
    }

    // Get or create PreToolUse array
    let pre_tool_use = settings["hooks"]
        .get_mut("PreToolUse")
        .and_then(|v| v.as_array_mut());

    if let Some(hooks) = pre_tool_use {
        // Check if veto hook already exists
        let already_exists = hooks.iter().any(|h| {
            h["hooks"]
                .as_array()
                .map(|inner| {
                    inner.iter().any(|ih| {
                        ih["command"]
                            .as_str()
                            .map(|c| c.contains("veto gate"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        });

        if already_exists {
            println!("  {} veto hooks already configured", "○".yellow());
            return Ok(());
        }

        hooks.push(veto_hook);
    } else {
        settings["hooks"]["PreToolUse"] = serde_json::json!([veto_hook]);
    }

    // Write settings back
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(settings_path, content)?;

    Ok(())
}

/// Remove veto hooks from Claude Code settings
fn remove_veto_hooks(settings_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !settings_path.exists() {
        println!("  {} No Claude Code settings found", "○".yellow());
        return Ok(());
    }

    let content = fs::read_to_string(settings_path)?;
    let mut settings: serde_json::Value = serde_json::from_str(&content)?;

    // Remove veto hooks from PreToolUse
    if let Some(hooks) = settings["hooks"]["PreToolUse"].as_array_mut() {
        hooks.retain(|h| {
            !h["hooks"]
                .as_array()
                .map(|inner| {
                    inner.iter().any(|ih| {
                        ih["command"]
                            .as_str()
                            .map(|c| c.contains("veto gate"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        });

        // Clean up empty PreToolUse array
        if hooks.is_empty() {
            if let Some(hooks_obj) = settings["hooks"].as_object_mut() {
                hooks_obj.remove("PreToolUse");
            }
        }
    }

    // Clean up empty hooks object
    if let Some(hooks_obj) = settings["hooks"].as_object() {
        if hooks_obj.is_empty() {
            if let Some(root) = settings.as_object_mut() {
                root.remove("hooks");
            }
        }
    }

    // Write settings back
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(settings_path, content)?;

    Ok(())
}
