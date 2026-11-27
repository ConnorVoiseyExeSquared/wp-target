use crate::api::get_nested_targets;
use crate::cli::{Cli, Command};
use crate::config::{init_config, load_config};
use crate::ssh::{Mode, find_exact, run_session, select_target};
use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;

pub mod api;
pub mod cli;
pub mod config;
pub mod ssh;

pub const APP_NAME: &str = "wg-target";

/// logs go to stderr. -v flags win over RUST_LOG, and with neither the default level is warn
fn init_logging(verbose: u8) -> color_eyre::Result<()> {
    let filter = match verbose {
        0 => EnvFilter::builder()
            .with_default_directive(LevelFilter::WARN.into())
            .from_env_lossy(),
        1 => EnvFilter::new("info"),
        2 => EnvFilter::new("debug"),
        _ => EnvFilter::new("trace"),
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init()
        .map_err(|e| color_eyre::eyre::eyre!("failed to start logging: {e}"))?;
    if verbose > 2 {
        info!(v_count = %verbose, "verbose only goes to 3 like -vvv")
    }
    Ok(())
}

pub async fn run() -> color_eyre::Result<()> {
    let cli = Cli::parse();
    init_logging(cli.verbose)?;

    let theme = ColorfulTheme::default();
    let (mode, target_arg) = match cli.command {
        Command::Init => {
            return init_config(&theme);
        }
        Command::Ssh { target } => (Mode::Ssh, target),
        Command::Sftp { target } => (Mode::Sftp, target),
        Command::List => {
            let config = load_config()?;
            for (label, _) in get_nested_targets(&config).await? {
                println!("{label}");
            }
            return Ok(());
        }
    };

    let config = load_config()?;
    tracing::debug!(
        wg_url = %config.wg_url,
        wg_username = %config.wg_username,
        wg_ssh_key = %config.wg_ssh_key,
        wg_ssh_port = config.wg_ssh_port,
        "config loaded"
    );

    let targets = get_nested_targets(&config).await?;
    tracing::info!(count = targets.len(), "fetched targets");
    if targets.is_empty() {
        color_eyre::eyre::bail!("warpgate returned no targets that belong to a group");
    }

    // a typed name connects directly when it matches exactly, otherwise the picker opens with it as the search
    let typed = target_arg.as_deref().map(str::trim).filter(|name| !name.is_empty());
    let exact = typed.and_then(|name| find_exact(&targets, name));
    let selected_index = match exact {
        Some(index) => index,
        None => select_target(&theme, &targets, typed)?,
    };
    let selected = &targets[selected_index].1;
    tracing::info!(target = %selected.name, ?mode, "opening session");
    let exit_code = run_session(&config, selected, mode)?;
    tracing::info!(exit_code, "session ended");
    if exit_code != 0 {
        // pass the session's exit code through to the caller
        std::process::exit(exit_code);
    }

    Ok(())
}
