use colored::Colorize;
use std::fs;
use std::path::PathBuf;

/// Get Claude Code settings path
fn get_claude_settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("settings.json"))
}

/// Get Gemini CLI settings path
fn get_gemini_settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".gemini").join("settings.json"))
}

/// Get Cursor CLI hooks path
fn get_cursor_hooks_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".cursor").join("hooks.json"))
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

/// Check if veto hooks are already configured in Gemini CLI
pub fn is_gemini_configured() -> bool {
    let Some(path) = get_gemini_settings_path() else {
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

    json["hooks"]["BeforeTool"]
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

/// Check if veto hooks are already configured in Cursor CLI
pub fn is_cursor_configured() -> bool {
    let Some(path) = get_cursor_hooks_path() else {
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

    json["hooks"]["beforeShellExecution"]
        .as_array()
        .map(|hooks| {
            hooks.iter().any(|h| {
                h["command"]
                    .as_str()
                    .map(|c| c.contains("veto gate"))
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

/// Setup Gemini CLI hooks integration
pub fn run_setup_gemini(uninstall: bool) -> Result<(), Box<dyn std::error::Error>> {
    let settings_path = get_gemini_settings_path()
        .ok_or("Cannot find home directory")?;

    println!("{}", "Setting up Gemini CLI integration...".bold());
    println!();

    if uninstall {
        remove_gemini_hooks(&settings_path)?;
        println!("  {} Removed veto hooks from Gemini CLI", "✓".green());
        println!();
        println!("Restart Gemini CLI for changes to take effect.");
    } else {
        add_gemini_hooks(&settings_path)?;
        println!("  {} Added veto hooks to BeforeTool", "✓".green());
        println!();
        println!("Done! Restart Gemini CLI for changes to take effect.");
    }

    Ok(())
}

/// Setup Cursor CLI hooks integration
pub fn run_setup_cursor(uninstall: bool) -> Result<(), Box<dyn std::error::Error>> {
    let hooks_path = get_cursor_hooks_path()
        .ok_or("Cannot find home directory")?;

    println!("{}", "Setting up Cursor CLI integration...".bold());
    println!();

    if uninstall {
        remove_cursor_hooks(&hooks_path)?;
        println!("  {} Removed veto hook from Cursor CLI", "✓".green());
        println!();
        println!("Restart Cursor CLI for changes to take effect.");
    } else {
        add_cursor_hooks(&hooks_path)?;
        println!("  {} Added veto hook to beforeShellExecution", "✓".green());
        println!();
        println!("Done! Restart Cursor CLI for changes to take effect.");
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

    // Create veto hook configuration for shell commands
    // Note: Claude Code uses seconds for timeout (not milliseconds)
    let bash_hook = serde_json::json!({
        "matcher": "Bash",
        "hooks": [{
            "type": "command",
            "command": "veto gate --claude",
            "timeout": 90
        }]
    });

    // Create veto hook configuration for file operations (Write|Edit)
    let file_hook = serde_json::json!({
        "matcher": "Write|Edit",
        "hooks": [{
            "type": "command",
            "command": "veto gate --claude --file-op",
            "timeout": 30
        }]
    });

    // Get or create hooks object
    if settings.get("hooks").is_none() {
        settings["hooks"] = serde_json::json!({});
    }

    // Helper to check if a veto hook with specific command exists
    let has_veto_hook = |hooks: &[serde_json::Value], cmd_contains: &str| {
        hooks.iter().any(|h| {
            h["hooks"]
                .as_array()
                .map(|inner| {
                    inner.iter().any(|ih| {
                        ih["command"]
                            .as_str()
                            .map(|c| c.contains(cmd_contains))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        })
    };

    // Get or create PreToolUse array
    let pre_tool_use = settings["hooks"]
        .get_mut("PreToolUse")
        .and_then(|v| v.as_array_mut());

    if let Some(hooks) = pre_tool_use {
        let mut updated = false;

        // Find existing bash hook index
        let bash_hook_idx = hooks.iter().position(|h| {
            h["matcher"].as_str() == Some("Bash") &&
            h["hooks"]
                .as_array()
                .map(|inner| {
                    inner.iter().any(|ih| {
                        ih["command"]
                            .as_str()
                            .map(|c| c.contains("veto gate") && !c.contains("--file-op"))
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        });

        if let Some(idx) = bash_hook_idx {
            // Update existing bash hook with timeout if missing
            if hooks[idx]["hooks"][0].get("timeout").is_none() {
                hooks[idx] = bash_hook;
                updated = true;
            }
        } else {
            // Add new bash hook
            hooks.push(bash_hook);
            updated = true;
        }

        // Add file hook if not exists
        if !has_veto_hook(hooks, "--file-op") {
            hooks.push(file_hook);
            updated = true;
        }

        if !updated {
            println!("  {} veto hooks already configured", "○".yellow());
            return Ok(());
        }
    } else {
        settings["hooks"]["PreToolUse"] = serde_json::json!([bash_hook, file_hook]);
    }

    // Write settings back
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(settings_path, content)?;

    Ok(())
}

/// Add veto hooks to Gemini CLI settings
fn add_gemini_hooks(settings_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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

    // Create veto hook configuration for shell commands
    let shell_hook = serde_json::json!({
        "matcher": "run_shell_command",
        "hooks": [{
            "name": "veto-gate-shell",
            "type": "command",
            "command": "veto gate --gemini",
            "timeout": 90000,
            "description": "Security gate for shell commands"
        }]
    });

    // Create veto hook configuration for file operations
    let file_hook = serde_json::json!({
        "matcher": "write_file|edit_file|replace_in_file",
        "hooks": [{
            "name": "veto-gate-file",
            "type": "command",
            "command": "veto gate --gemini --file-op",
            "timeout": 30000,
            "description": "Security gate for file write operations"
        }]
    });

    // Get or create hooks object
    if settings.get("hooks").is_none() {
        settings["hooks"] = serde_json::json!({});
    }

    // Helper to check if a veto hook with given name exists
    let has_veto_hook = |hooks: &[serde_json::Value], name: &str| {
        hooks.iter().any(|h| {
            h["hooks"]
                .as_array()
                .map(|inner| {
                    inner.iter().any(|ih| {
                        ih["name"].as_str() == Some(name)
                            || ih["command"]
                                .as_str()
                                .map(|c| c.contains("veto gate"))
                                .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        })
    };

    // Get or create BeforeTool array
    let before_tool = settings["hooks"]
        .get_mut("BeforeTool")
        .and_then(|v| v.as_array_mut());

    if let Some(hooks) = before_tool {
        let mut added = 0;

        // Add shell hook if not exists
        if !has_veto_hook(hooks, "veto-gate-shell") {
            hooks.push(shell_hook);
            added += 1;
        }

        // Add file hook if not exists
        if !has_veto_hook(hooks, "veto-gate-file") {
            hooks.push(file_hook);
            added += 1;
        }

        if added == 0 {
            println!("  {} veto hooks already configured", "○".yellow());
            return Ok(());
        }
    } else {
        settings["hooks"]["BeforeTool"] = serde_json::json!([shell_hook, file_hook]);
    }

    // Write settings back
    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(settings_path, content)?;

    Ok(())
}

/// Add veto hooks to Cursor CLI hooks.json
fn add_cursor_hooks(hooks_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Read existing hooks or create new
    let mut settings: serde_json::Value = if hooks_path.exists() {
        let content = fs::read_to_string(hooks_path)?;
        serde_json::from_str(&content)?
    } else {
        if let Some(parent) = hooks_path.parent() {
            fs::create_dir_all(parent)?;
        }
        serde_json::json!({})
    };

    // Ensure version exists
    if settings.get("version").is_none() {
        settings["version"] = serde_json::json!(1);
    }

    // Ensure hooks object exists
    if settings.get("hooks").is_none() {
        settings["hooks"] = serde_json::json!({});
    }

    let veto_hook = serde_json::json!({
        "command": "veto gate --cursor"
    });

    let before_shell = settings["hooks"]
        .get_mut("beforeShellExecution")
        .and_then(|v| v.as_array_mut());

    if let Some(hooks) = before_shell {
        let already_exists = hooks.iter().any(|h| {
            h["command"]
                .as_str()
                .map(|c| c.contains("veto gate"))
                .unwrap_or(false)
        });

        if already_exists {
            println!("  {} veto hook already configured", "○".yellow());
            return Ok(());
        }

        hooks.push(veto_hook);
    } else {
        settings["hooks"]["beforeShellExecution"] = serde_json::json!([veto_hook]);
    }

    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(hooks_path, content)?;

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

/// Remove veto hooks from Gemini CLI settings
fn remove_gemini_hooks(settings_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !settings_path.exists() {
        println!("  {} No Gemini CLI settings found", "○".yellow());
        return Ok(());
    }

    let content = fs::read_to_string(settings_path)?;
    let mut settings: serde_json::Value = serde_json::from_str(&content)?;

    // Remove veto hooks from BeforeTool
    if let Some(hooks) = settings["hooks"]["BeforeTool"].as_array_mut() {
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

        // Clean up empty BeforeTool array
        if hooks.is_empty() {
            if let Some(hooks_obj) = settings["hooks"].as_object_mut() {
                hooks_obj.remove("BeforeTool");
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

/// Remove veto hooks from Cursor CLI hooks.json
fn remove_cursor_hooks(hooks_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !hooks_path.exists() {
        println!("  {} No Cursor CLI hooks found", "○".yellow());
        return Ok(());
    }

    let content = fs::read_to_string(hooks_path)?;
    let mut settings: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(hooks) = settings["hooks"]["beforeShellExecution"].as_array_mut() {
        hooks.retain(|h| {
            !h["command"]
                .as_str()
                .map(|c| c.contains("veto gate"))
                .unwrap_or(false)
        });

        if hooks.is_empty() {
            if let Some(hooks_obj) = settings["hooks"].as_object_mut() {
                hooks_obj.remove("beforeShellExecution");
            }
        }
    }

    if let Some(hooks_obj) = settings["hooks"].as_object() {
        if hooks_obj.is_empty() {
            if let Some(root) = settings.as_object_mut() {
                root.remove("hooks");
            }
        }
    }

    let content = serde_json::to_string_pretty(&settings)?;
    fs::write(hooks_path, content)?;

    Ok(())
}

// ============================================================================
// OpenCode Integration
// ============================================================================

const OPENCODE_PLUGIN_FILENAME: &str = "veto-gate.js";

/// Get OpenCode plugins directory path
fn get_opencode_plugins_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".config").join("opencode").join("plugins"))
}

/// Get full path to veto plugin file
fn get_opencode_plugin_file_path() -> Option<PathBuf> {
    get_opencode_plugins_path().map(|p| p.join(OPENCODE_PLUGIN_FILENAME))
}

/// Check if veto plugin is already configured in OpenCode
pub fn is_opencode_configured() -> bool {
    get_opencode_plugin_file_path()
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// Setup OpenCode plugin integration
pub fn run_setup_opencode(uninstall: bool) -> Result<(), Box<dyn std::error::Error>> {
    let plugins_path = get_opencode_plugins_path()
        .ok_or("Cannot find home directory")?;

    println!("{}", "Setting up OpenCode integration...".bold());
    println!();

    if uninstall {
        remove_opencode_plugin(&plugins_path)?;
        println!("  {} Removed veto plugin from OpenCode", "✓".green());
        println!();
        println!("Restart OpenCode for changes to take effect.");
    } else {
        add_opencode_plugin(&plugins_path)?;
        println!("  {} Added veto plugin to OpenCode", "✓".green());
        println!();
        println!("Done! Restart OpenCode for changes to take effect.");
    }

    Ok(())
}

/// Add veto plugin to OpenCode plugins directory
fn add_opencode_plugin(plugins_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Create plugins directory if needed
    if !plugins_path.exists() {
        fs::create_dir_all(plugins_path)?;
    }

    let plugin_file = plugins_path.join(OPENCODE_PLUGIN_FILENAME);

    // Check if already exists
    if plugin_file.exists() {
        println!("  {} veto plugin already configured", "○".yellow());
        return Ok(());
    }

    // Plugin content - ES module format for OpenCode (uses Bun)
    // Uses thrown errors to block tool execution and inform the AI when auth is needed
    let plugin_content = r#"export const VetoGate = async ({ project, client, $, directory, worktree }) => {
  // Track denied commands across retries in the same session
  const deniedCommands = new Set()

  // Tools that veto should intercept
  const INTERCEPTED_TOOLS = ["bash", "write", "edit", "read"]

  const extractCommand = (input, output) => {
    return (
      output?.args?.command ??
      input?.args?.command ??
      output?.command ??
      input?.command ??
      null
    )
  }

  const extractFilePath = (input, output) => {
    return (
      output?.args?.file_path ??
      input?.args?.file_path ??
      output?.args?.path ??
      input?.args?.path ??
      null
    )
  }

  const normalizeCommand = (command) => (typeof command === "string" ? command.trim() : "")

  return {
    "tool.execute.before": async (input, output) => {
      const tool = input.tool?.toLowerCase()
      if (!INTERCEPTED_TOOLS.includes(tool)) return

      let command
      let isFileOp = false

      if (tool === "bash") {
        command = normalizeCommand(extractCommand(input, output))
      } else {
        // File operations: write, edit, read
        const path = extractFilePath(input, output)
        if (!path) return
        command = `${tool}_file:${path}`
        isFileOp = true
      }

      if (!command) return

      // Block permanently denied commands
      if (deniedCommands.has(command)) {
        throw new Error("[veto] BLOCKED. This command was rejected. DO NOT RETRY.")
      }

      const args = isFileOp
        ? ["gate", "--opencode", "--file-op", "--", command]
        : ["gate", "--opencode", "--", command]

      const result = await $`veto ${args}`.nothrow()

      if (result.exitCode === 0) {
        // Approved
        return
      }

      // Check stderr for auth instructions
      const stderr = result.stderr.toString()
      const stdout = result.stdout.toString()
      const message = (stderr || stdout).trim()

      // If user explicitly denied (dialog/touchid), block permanently
      if (message.includes("STOP_RETRY") || message.includes("User rejected")) {
        deniedCommands.add(command)
        throw new Error("[veto] BLOCKED. User rejected this command. DO NOT RETRY.")
      }

      // If auth code needed (PIN/TOTP), throw with instructions for AI
      if (
        message.includes("VETO_PIN=") ||
        message.includes("VETO_TOTP=") ||
        message.includes("VETO_CONFIRM=") ||
        message.includes("VETO_RESPONSE=")
      ) {
        throw new Error(message)
      }

      // Other errors
      throw new Error(`[veto] ${message || "Command blocked"}`)
    }
  }
}
"#;

    fs::write(&plugin_file, plugin_content)?;

    Ok(())
}

/// Remove veto plugin from OpenCode plugins directory
fn remove_opencode_plugin(plugins_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let plugin_file = plugins_path.join(OPENCODE_PLUGIN_FILENAME);

    if !plugin_file.exists() {
        println!("  {} No veto plugin found", "○".yellow());
        return Ok(());
    }

    fs::remove_file(&plugin_file)?;

    Ok(())
}
