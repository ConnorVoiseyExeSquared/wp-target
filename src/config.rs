use color_eyre::Section;
use color_eyre::eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use dialoguer::{Input, Password, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use crate::APP_NAME;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// api token used for every request to warpgate api
    pub token: String,

    /// url for the warpgate api
    pub wg_url: String,

    /// warpgate username
    pub wg_username: String,

    /// ssh key path
    pub wg_ssh_key: String,

    /// warpgate ssh port
    pub wg_ssh_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token: String::new(),
            wg_url: String::new(),
            wg_username: String::new(),
            wg_ssh_key: String::new(),
            wg_ssh_port: 2222,
        }
    }
}

/// load the config from confy's file, creating a default one if it doesn't exist,
/// and fail if any required value was left empty
pub fn load_config() -> color_eyre::Result<Config> {
    let config: Config = confy::load(APP_NAME, None)?;
    let path = confy::get_configuration_file_path(APP_NAME, None)?;
    tracing::debug!(path = %path.display(), "loading config");

    let required = [
        ("token", &config.token),
        ("wg_url", &config.wg_url),
        ("wg_username", &config.wg_username),
        ("wg_ssh_key", &config.wg_ssh_key),
    ];
    let missing: Vec<&str> = required
        .into_iter()
        .filter(|(_, value)| value.is_empty())
        .map(|(field, _)| field)
        .collect();
    if !missing.is_empty() {
        // tracing shows the list in a readable form, the eyre report below carries the stack trace
        tracing::error!(
            path = %path.display(),
            missing = %missing.join(", "),
            "config is incomplete"
        );
        return Err(eyre!(
            "config is incomplete, {} value(s) are empty",
            missing.len()
        ))
        .note(format!("empty values: {}", missing.join(", ")))
        .note(format!("config file: {}", path.display()))
        .suggestion(format!(
            "run `{APP_NAME} init` to fill in the missing values"
        ));
    }

    Ok(config)
}

/// prompt for each value, using the existing config as defaults, then write the file
pub fn init_config(theme: &ColorfulTheme) -> color_eyre::Result<()> {
    let existing: Config = confy::load(APP_NAME, None)?;

    let token = Password::with_theme(theme)
        .with_prompt("warpgate api token (empty keeps the current one)")
        .allow_empty_password(!existing.token.is_empty())
        .interact()?;
    let token = if token.is_empty() {
        existing.token
    } else {
        token
    };

    let wg_url = prompt_required(theme, "warpgate url", &existing.wg_url)?;
    let wg_username = prompt_required(theme, "warpgate username", &existing.wg_username)?;
    let wg_ssh_key = prompt_required(theme, "ssh key path", &existing.wg_ssh_key)?;
    let wg_ssh_port = Input::<u16>::new()
        .with_prompt("warpgate ssh port")
        .default(existing.wg_ssh_port)
        .interact_text()?;

    let config = Config {
        token,
        wg_url,
        wg_username,
        wg_ssh_key,
        wg_ssh_port,
    };
    let path = confy::get_configuration_file_path(APP_NAME, None)?;
    create_private_file(&path)?;
    tracing::info!(path = %path.display(), "writing config");
    confy::store(APP_NAME, None, &config)?;

    println!("Wrote config to: {}", path.display().to_string().cyan());
    Ok(())
}

/// make sure the config file exists and is only readable by its owner, since it holds the api token.
/// confy's store truncates the existing file rather than replacing it, so the mode survives the write
fn create_private_file(path: &Path) -> color_eyre::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .mode(0o600)
        .open(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

/// text prompt that refuses empty input, with the existing value as the default
fn prompt_required(
    theme: &ColorfulTheme,
    prompt: &str,
    existing: &str,
) -> color_eyre::Result<String> {
    let mut input = Input::<String>::with_theme(theme)
        .with_prompt(prompt)
        .validate_with(|value: &String| -> Result<(), &str> {
            if value.is_empty() {
                Err("value is required")
            } else {
                Ok(())
            }
        });
    if !existing.is_empty() {
        input = input.default(existing.to_string());
    }
    Ok(input.interact_text()?)
}
