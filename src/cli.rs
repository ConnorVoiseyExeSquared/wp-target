use clap::{ArgAction, Parser, Subcommand};
use clap_complete::{ArgValueCandidates, CompletionCandidate};

use crate::APP_NAME;

/// target labels for shell completion, the same labels `list` prints.
/// any failure (no config, network, bad response) gives no candidates rather than an error,
/// since completion output must not contain anything but candidates
fn target_candidates() -> Vec<CompletionCandidate> {
    let Ok(config) = crate::config::load_config() else {
        return Vec::new();
    };
    let Ok(runtime) = tokio::runtime::Runtime::new() else {
        return Vec::new();
    };
    match runtime.block_on(crate::api::get_nested_targets(&config)) {
        Ok(targets) => targets
            .into_iter()
            .map(|(label, _)| CompletionCandidate::new(label))
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[derive(Debug, Parser)]
#[command(
    name = APP_NAME,
    version,
    about = "A really basic CLI to select a Warpgate target and ssh to it",
    subcommand_required = true,
    arg_required_else_help = true
)]
pub struct Cli {
    /// log verbosity: -v info, -vv debug, -vvv trace (overrides RUST_LOG)
    #[arg(short, long, action = ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// write the config file, prompting for each value
    Init,
    /// select a target and open an ssh session to it
    Ssh {
        /// target name to connect to directly, otherwise the picker opens
        #[arg(add = ArgValueCandidates::new(target_candidates))]
        target: Option<String>,
    },
    /// select a target and open an sftp session to it
    Sftp {
        /// target name to connect to directly, otherwise the picker opens
        #[arg(add = ArgValueCandidates::new(target_candidates))]
        target: Option<String>,
    },
    /// print the target labels, one per line
    List,
}
