//! module `loader::help`

use std::{fs, path::Path};

use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};

/// Represents a single command in the help system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpCommand {
    pub command: String,
    pub description: String,
}

/// Wrapper for the TOML file containing help commands
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpCommandFile {
    pub commands: Vec<HelpCommand>,
}

/// Complete help data including basic text and commands
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpData {
    pub basic_text: String,
    pub commands: Vec<HelpCommand>,
}

/// Loads help commands from a TOML file
/// # Errors
/// - on file IO error or TOML parsing error
pub fn load_help_commands(toml_path: &Path) -> Result<Vec<HelpCommand>> {
    let help_file = fs::read_to_string(toml_path)
        .with_context(|| format!("reading help commands from '{}'", toml_path.display()))?;
    let wrapper: HelpCommandFile =
        toml::from_str(&help_file).with_context(|| format!("parsing help commands from '{}'", toml_path.display()))?;

    info!(
        "{} help commands loaded from '{}'",
        wrapper.commands.len(),
        toml_path.display()
    );

    Ok(wrapper.commands)
}

/// Loads basic help text from a text file
/// # Errors
/// - on file IO error
pub fn load_help_basic_text(text_path: &Path) -> Result<String> {
    let basic_text = fs::read_to_string(text_path)
        .with_context(|| format!("reading basic help text from '{}'", text_path.display()))?;

    info!("Basic help text loaded from '{}'", text_path.display());

    Ok(basic_text.trim().to_string())
}

/// Loads complete help data from both text and TOML files
/// # Errors
/// - on file IO error or TOML parsing error
pub fn load_help_data(basic_text_path: &Path, commands_toml_path: &Path) -> Result<HelpData> {
    let basic_text = load_help_basic_text(basic_text_path).context("while loading basic help text")?;
    let commands = load_help_commands(commands_toml_path).context("while loading help commands")?;

    Ok(HelpData { basic_text, commands })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    #[test]
    fn test_help_system_integration() {
        let basic_text_path = Path::new("data/help_basic.txt");
        let commands_toml_path = Path::new("data/help_commands.toml");

        // Test that both files exist
        assert!(basic_text_path.exists(), "help_basic.txt should exist");
        assert!(commands_toml_path.exists(), "help_commands.toml should exist");

        // Test loading help data
        let help_data =
            load_help_data(basic_text_path, commands_toml_path).expect("Should successfully load help data");

        // Verify basic text is loaded
        assert!(!help_data.basic_text.is_empty(), "Basic help text should not be empty");
        assert!(
            help_data.basic_text.contains("commands listed below"),
            "Should contain expected content"
        );

        // Verify commands are loaded
        assert!(!help_data.commands.is_empty(), "Commands should not be empty");

        // Check for essential commands
        let command_names: Vec<&str> = help_data.commands.iter().map(|cmd| cmd.command.as_str()).collect();

        assert!(command_names.contains(&"help"), "Should contain 'help' command");
        assert!(command_names.contains(&"look"), "Should contain 'look' command");
        assert!(
            command_names.contains(&"inventory"),
            "Should contain 'inventory' command"
        );
        assert!(command_names.contains(&"quit"), "Should contain 'quit' command");

        // Verify each command has a description
        for command in &help_data.commands {
            assert!(
                !command.description.is_empty(),
                "Command '{}' should have a non-empty description",
                command.command
            );
        }
    }
}
