use anyhow::{bail, Result};
use clap::Parser;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_docker_run::DockerRunPreprocessor;
use std::{io, path::PathBuf};

/// Plugin for mdBook which runs a script using Docker and emits the result.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Options {
    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Check if the renderer is supported.
    Supports(SupportsCommand),
    /// Process a parsed book (default).
    Process,
    /// Install support for mdbook-docker-run into the current mdbook project.
    Install(InstallCommand),
}

#[derive(Parser, Debug)]
pub struct SupportsCommand {
    pub renderer: String,
}

#[derive(Parser, Debug)]
pub struct InstallCommand {
    #[clap(long)]
    pub assets: Option<PathBuf>,
}

impl Options {
    fn run(&self, preprocessor: &dyn Preprocessor) -> Result<()> {
        match &self.command {
            Some(Command::Supports(command)) => {
                if preprocessor.supports_renderer(&command.renderer) {
                    Ok(())
                } else {
                    bail!("unsupported renderer {}", command.renderer);
                }
            }
            None | Some(Command::Process) => {
                let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;
                let output = preprocessor.run(&ctx, book)?;
                serde_json::to_writer(io::stdout(), &output)?;
                Ok(())
            }
            Some(Command::Install(_command)) => Ok(()),
        }
    }
}

#[tokio::main(flavor = "current_thread")]
#[tracing::instrument(name = "mdbook_docker_run")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .map_writer(move |_| std::io::stderr)
        .init();
    let options = Options::parse();
    let renderer = DockerRunPreprocessor::new_current();
    tokio::task::spawn_blocking(move || options.run(&renderer)).await?
}
