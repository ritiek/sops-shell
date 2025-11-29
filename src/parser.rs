use anyhow::{anyhow, Result};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct CommandMapping {
    pub key: String,
    pub command: String,
}

pub fn parse_commands(decrypted_content: &str) -> Result<Vec<CommandMapping>> {
    let lines: Vec<&str> = decrypted_content.split('\n').collect();
    let mut mappings = Vec::new();

    let shell_comment_regex = Regex::new(r"^\s*#\s*shell:\s*(.+)$")?;

    for (i, line) in lines.iter().enumerate() {
        let stripped = line.trim();

        if let Some(captures) = shell_comment_regex.captures(stripped) {
            let command = captures.get(1).ok_or_else(|| anyhow!("Failed to capture command"))?.as_str().trim();
            if command.is_empty() {
                continue;
            }

            if let Some(key) = find_next_key(&lines, i + 1) {
                mappings.push(CommandMapping {
                    key: key.to_string(),
                    command: command.to_string(),
                });
            }
        }
    }

    Ok(mappings)
}

fn find_next_key<'a>(lines: &'a [&'a str], start_idx: usize) -> Option<&'a str> {
    let key_regex = Regex::new(r"^\s*([^:=\s]+)\s*[:=]").ok()?;

    for line in lines.iter().skip(start_idx) {
        let stripped = line.trim();

        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }

        if let Some(captures) = key_regex.captures(stripped) {
            return Some(captures.get(1)?.as_str());
        }
    }

    None
}