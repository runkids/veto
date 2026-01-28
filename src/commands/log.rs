//! Log command - view and manage audit log

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

use colored::Colorize;

use crate::audit::{clear_audit_log, get_audit_log_path, read_audit_log};
use crate::cli::LogArgs;

/// Run the log command
pub fn run_log(args: LogArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Handle clear first
    if args.clear {
        return handle_clear();
    }

    // Handle follow mode
    if args.follow {
        return follow_log(args.filter.as_deref());
    }

    // Read and display log entries
    let lines = read_audit_log()?;

    if lines.is_empty() {
        println!("{}", "No audit log entries found.".dimmed());
        return Ok(());
    }

    // Apply filter
    let filtered: Vec<&String> = if let Some(ref filter) = args.filter {
        let filter_upper = filter.to_uppercase();
        lines
            .iter()
            .filter(|line| line.contains(&format!(" {} ", filter_upper)))
            .collect()
    } else {
        lines.iter().collect()
    };

    // Apply tail
    let to_show: Vec<&String> = if let Some(n) = args.tail {
        filtered.iter().rev().take(n).rev().cloned().collect()
    } else {
        filtered
    };

    // Display entries
    for line in to_show {
        print_colored_line(line);
    }

    Ok(())
}

/// Print a log line with colors based on result
fn print_colored_line(line: &str) {
    if line.contains(" ALLOWED ") {
        println!("{}", line.green());
    } else if line.contains(" DENIED ") {
        println!("{}", line.red());
    } else if line.contains(" BLOCKED ") {
        println!("{}", line.yellow());
    } else {
        println!("{}", line);
    }
}

/// Follow log in real-time (like tail -f)
fn follow_log(filter: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = get_audit_log_path();
    let mut last_size = if log_path.exists() {
        std::fs::metadata(&log_path)?.len()
    } else {
        0
    };

    println!("{}", "Following audit log (Ctrl+C to stop)...".dimmed());

    // Show existing entries first
    if last_size > 0 {
        let lines = read_audit_log()?;
        let filtered = filter_lines(&lines, filter);
        // Show last 10 entries initially
        for line in filtered.iter().rev().take(10).rev() {
            print_colored_line(line);
        }
    }

    loop {
        thread::sleep(Duration::from_millis(500));

        if !log_path.exists() {
            continue;
        }

        let current_size = std::fs::metadata(&log_path)?.len();
        if current_size > last_size {
            // Read new content
            let content = std::fs::read_to_string(&log_path)?;
            let lines: Vec<&str> = content.lines().collect();

            // Calculate how many lines to show
            let old_content_approx_lines = if last_size > 0 {
                content[..last_size as usize].lines().count()
            } else {
                0
            };

            for line in lines.iter().skip(old_content_approx_lines) {
                if let Some(f) = filter {
                    if line.contains(&format!(" {} ", f.to_uppercase())) {
                        print_colored_line(line);
                        io::stdout().flush()?;
                    }
                } else {
                    print_colored_line(line);
                    io::stdout().flush()?;
                }
            }

            last_size = current_size;
        }
    }
}

/// Filter lines by result type
fn filter_lines<'a>(lines: &'a [String], filter: Option<&str>) -> Vec<&'a String> {
    if let Some(f) = filter {
        let filter_upper = f.to_uppercase();
        lines
            .iter()
            .filter(|line| line.contains(&format!(" {} ", filter_upper)))
            .collect()
    } else {
        lines.iter().collect()
    }
}

/// Handle clear with confirmation
fn handle_clear() -> Result<(), Box<dyn std::error::Error>> {
    let log_path = get_audit_log_path();
    if !log_path.exists() {
        println!("{}", "Audit log is already empty.".dimmed());
        return Ok(());
    }

    // Count entries
    let lines = read_audit_log()?;
    if lines.is_empty() {
        println!("{}", "Audit log is already empty.".dimmed());
        return Ok(());
    }

    // Prompt for confirmation
    print!(
        "{} {} entries will be deleted. Continue? [y/N] ",
        "Warning:".yellow().bold(),
        lines.len()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        clear_audit_log()?;
        println!("{}", "Audit log cleared.".green());
    } else {
        println!("{}", "Cancelled.".dimmed());
    }

    Ok(())
}
