use dialoguer::FuzzySelect;
use dialoguer::theme::ColorfulTheme;

use crate::api::Target;
use crate::config::Config;

/// show the labelled targets in a fuzzy finder and return the index the user picked.
/// `search` pre-fills the finder's search text
pub fn select_target(
    theme: &ColorfulTheme,
    targets: &[(String, Target)],
    search: Option<&str>,
) -> color_eyre::Result<usize> {
    tracing::debug!(count = targets.len(), ?search, "showing target picker");
    let labels: Vec<&str> = targets.iter().map(|(label, _)| label.as_str()).collect();

    Ok(FuzzySelect::with_theme(theme)
        .with_prompt("target")
        .with_initial_text(search.unwrap_or_default())
        .default(0)
        .items(&labels)
        .max_length(20)
        .interact()?)
}

/// index of the one target whose displayed label ("group - name") matches `text`,
/// ignoring case and surrounding spaces. returns None when nothing matches or when
/// more than one label matches
pub fn find_exact(targets: &[(String, Target)], text: &str) -> Option<usize> {
    let wanted = text.trim().to_lowercase();
    let mut matches = targets
        .iter()
        .enumerate()
        .filter(|(_, (label, _))| label.trim().to_lowercase() == wanted)
        .map(|(index, _)| index);
    let first = matches.next()?;
    match matches.next() {
        None => Some(first),
        Some(_) => {
            tracing::warn!(text, "more than one target matches this label, opening the picker");
            None
        }
    }
}

/// which client to open against the selected target
#[derive(Debug, Clone, Copy)]
pub enum Mode {
    Ssh,
    Sftp,
}

/// open an ssh or sftp session to the selected target through warpgate
/// (warpgate routes `user:target@host` to the target).
/// returns the client's exit code, which is the remote shell's code for a normal session
pub fn run_session(config: &Config, target: &Target, mode: Mode) -> color_eyre::Result<i32> {
    let host = reqwest::Url::parse(&config.wg_url)?
        .host_str()
        .ok_or_else(|| color_eyre::eyre::eyre!("wg_url has no host: {}", config.wg_url))?
        .to_string();
    let login = format!("{}:{}@{}", config.wg_username, target.name, host);

    tracing::debug!(
        ?mode,
        %host,
        %login,
        key = %config.wg_ssh_key,
        port = config.wg_ssh_port,
        "spawning client"
    );
    // ssh takes the port as -p, sftp takes it as -P
    let (program, port_flag) = match mode {
        Mode::Ssh => ("ssh", "-p"),
        Mode::Sftp => ("sftp", "-P"),
    };

    let mut command = std::process::Command::new(program);
    command
        .arg("-i")
        .arg(&config.wg_ssh_key)
        .arg(port_flag)
        .arg(config.wg_ssh_port.to_string());
    match mode {
        Mode::Ssh => {
            command.arg(&login);
        }
        // sftp reads `user:target@host` as `host:path`, so the login goes in as the user option
        Mode::Sftp => {
            command
                .arg("-o")
                // ssh's config parser splits on spaces, so the value needs its own quotes
                .arg(format!("User=\"{}:{}\"", config.wg_username, target.name))
                .arg(&host);
        }
    }
    let status = command.status()?;
    // None means the process was killed by a signal, treat that as a failure
    Ok(status.code().unwrap_or(1))
}
