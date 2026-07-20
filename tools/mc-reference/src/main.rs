use anyhow::Result;
use clap::{Parser, Subcommand};
use mc_reference::{Command, Context, ExperimentCommand, ProtocolCommand};

#[derive(Debug, Parser)]
#[command(
    name = "mc-ref",
    about = "Version-locked Minecraft behavior reference tooling"
)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    Fetch {
        #[arg(long)]
        version: String,
    },
    Reports,
    Query {
        kind: String,
        id: String,
    },
    Symbols,
    Coverage,
    Readiness,
    Protocol {
        #[command(subcommand)]
        command: CliProtocolCommand,
    },
    Experiment {
        #[command(subcommand)]
        command: CliExperimentCommand,
    },
    Verify {
        #[arg(long)]
        offline: bool,
    },
}

#[derive(Debug, Subcommand)]
enum CliProtocolCommand {
    Inventory,
    Coverage,
    Readiness,
    Verify,
}

#[derive(Debug, Subcommand)]
enum CliExperimentCommand {
    List,
    Run { id: String },
    Verify,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let context = Context::discover()?;
    let command = match cli.command {
        CliCommand::Fetch { version } => Command::Fetch { version },
        CliCommand::Reports => Command::Reports,
        CliCommand::Query { kind, id } => Command::Query { kind, id },
        CliCommand::Symbols => Command::Symbols,
        CliCommand::Coverage => Command::Coverage,
        CliCommand::Readiness => Command::Readiness,
        CliCommand::Protocol { command } => Command::Protocol(match command {
            CliProtocolCommand::Inventory => ProtocolCommand::Inventory,
            CliProtocolCommand::Coverage => ProtocolCommand::Coverage,
            CliProtocolCommand::Readiness => ProtocolCommand::Readiness,
            CliProtocolCommand::Verify => ProtocolCommand::Verify,
        }),
        CliCommand::Experiment { command } => Command::Experiment(match command {
            CliExperimentCommand::List => ExperimentCommand::List,
            CliExperimentCommand::Run { id } => ExperimentCommand::Run { id },
            CliExperimentCommand::Verify => ExperimentCommand::Verify,
        }),
        CliCommand::Verify { offline } => Command::Verify { offline },
    };
    mc_reference::run(&context, command)
}
