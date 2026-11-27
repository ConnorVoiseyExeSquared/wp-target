use clap::CommandFactory;
use clap_complete::CompleteEnv;
use wg_target::cli::Cli;
use wg_target::run;

fn main() -> color_eyre::Result<()> {
    // answers shell completion requests (COMPLETE=<shell>) and exits, before any real work starts.
    // this runs outside the tokio runtime because the target completer blocks on the api itself
    CompleteEnv::with_factory(Cli::command).complete();

    color_eyre::install()?;
    tokio::runtime::Runtime::new()?.block_on(run())
}
