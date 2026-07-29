#![forbid(unsafe_code)]

mod architecture;
mod cache;
mod content;
mod deployment;
mod source_policy;
mod task;
mod workspace;

use anyhow::{Result, bail};
use std::env;

fn main() -> Result<()> {
    let workspace = workspace::discover()?;
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [command] if command == "help" || command == "--help" || command == "-h" => {
            print_help();
            Ok(())
        }
        [group, command] if group == "architecture" && command == "verify" => {
            architecture::verify(&workspace)
        }
        [group, command] if group == "source" && command == "verify" => {
            source_policy::verify(&workspace)
        }
        [group, command] if group == "deployment" && command == "verify" => {
            deployment::verify(&workspace)
        }
        [group, command] if group == "cache" && command == "inspect" => {
            cache::inspect_command(&workspace)
        }
        [group, command, options @ ..] if group == "cache" && command == "prune" => {
            cache::prune(&workspace, cache::parse_apply_mode(options)?)
        }
        [group, command, options @ ..] if group == "cache" && command == "maintain" => {
            cache::maintain(&workspace, cache::parse_apply_mode(options)?)
        }
        [group, command, options @ ..] if group == "content" => {
            content::run(&workspace, command, options)
        }
        [group, namespace, cargo_arguments @ ..] if group == "cargo" => {
            cache::run_isolated_cargo(&workspace, namespace, cargo_arguments)
        }
        [group, command] if group == "task" && command == "check" => task::check(&workspace),
        [] => {
            print_help();
            Ok(())
        }
        _ => bail!("unknown ferrite-tooling command; run `cargo ferrite help`"),
    }
}

fn print_help() {
    println!(
        "\
Ferrite repository tooling

Usage:
  cargo ferrite architecture verify
  cargo ferrite source verify
  cargo ferrite deployment verify
  cargo ferrite cache inspect
  cargo ferrite cache prune [--apply]
  cargo ferrite cache maintain [--apply]
  cargo ferrite content import [--source <cache>] [--output <bundle>]
  cargo ferrite content verify [--bundle <bundle>]
  cargo ferrite cargo <debugging|coverage|fuzz|bench|ci> <cargo arguments...>
  cargo ferrite-check

Cache mutation is dry-run unless --apply is explicit. The repository check task performs the
rate-limited policy maintenance before running architecture, format, Clippy, test, and offline
reference gates."
    );
}
