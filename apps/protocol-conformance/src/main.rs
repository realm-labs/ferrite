#![forbid(unsafe_code)]

//! Minecraft Java 26.2 C0/C1 protocol conformance runner.

mod fixture;
mod headless;
mod smoke;
mod vanilla;

use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

pub(crate) type DynError = Box<dyn Error + Send + Sync>;

fn main() -> Result<(), DynError> {
    match Arguments::parse(std::env::args().skip(1))? {
        Arguments::Run => {
            let report = headless::run()?;
            println!("{}", report.summary());
        }
        Arguments::TcpSmoke => {
            let report = smoke::run_loopback()?;
            println!("{}", report.summary());
        }
        Arguments::C2Smoke => {
            let report = smoke::run_playable_loopback()?;
            println!("{}", report.summary());
        }
        Arguments::ConnectC2 { address } => {
            let report = smoke::run_playable_client(address)?;
            println!("{}", report.summary());
        }
        Arguments::VanillaProbe {
            playable,
            client_jar,
            registry_report,
            bind,
            timeout,
            evidence,
        } => {
            let probe = vanilla::VanillaProbe {
                client_jar,
                registry_report,
                bind,
                timeout,
                evidence,
            };
            if playable {
                println!("{}", vanilla::run_playable(probe)?.summary());
            } else {
                println!("{}", vanilla::run(probe)?.summary());
            }
        }
        Arguments::VerifyVanillaFixture {
            client_jar,
            registry_report,
        } => println!(
            "{}",
            vanilla::verify_fixture(&client_jar, &registry_report)?
        ),
        Arguments::Help => print_help(),
    }
    Ok(())
}

enum Arguments {
    Run,
    TcpSmoke,
    C2Smoke,
    ConnectC2 {
        address: SocketAddr,
    },
    VanillaProbe {
        playable: bool,
        client_jar: PathBuf,
        registry_report: PathBuf,
        bind: String,
        timeout: Duration,
        evidence: PathBuf,
    },
    VerifyVanillaFixture {
        client_jar: PathBuf,
        registry_report: PathBuf,
    },
    Help,
}

impl Arguments {
    fn parse(mut arguments: impl Iterator<Item = String>) -> Result<Self, DynError> {
        match arguments.next().as_deref() {
            None | Some("run") => Ok(Self::Run),
            Some("tcp-smoke") => Ok(Self::TcpSmoke),
            Some("c2-smoke") => Ok(Self::C2Smoke),
            Some("connect-c2") => {
                let address = arguments
                    .next()
                    .ok_or("connect-c2 requires an IP:PORT address")?
                    .parse()?;
                if arguments.next().is_some() {
                    return Err("connect-c2 accepts exactly one IP:PORT address".into());
                }
                Ok(Self::ConnectC2 { address })
            }
            Some("vanilla-probe") => Self::parse_vanilla(arguments, false),
            Some("vanilla-c2-probe") => Self::parse_vanilla(arguments, true),
            Some("verify-vanilla-fixture") => Self::parse_fixture(arguments),
            Some("--help" | "-h") => Ok(Self::Help),
            Some(command) => Err(format!("unknown protocol-conformance command: {command}").into()),
        }
    }

    fn parse_vanilla(
        mut arguments: impl Iterator<Item = String>,
        playable: bool,
    ) -> Result<Self, DynError> {
        let mut client_jar = None;
        let mut registry_report =
            PathBuf::from("target/mc-reference/26.2/generated/reports/registries.json");
        let mut bind = "127.0.0.1:25565".to_owned();
        let mut timeout_seconds = 120u64;
        let mut evidence = PathBuf::from(if playable {
            "target/protocol-conformance/vanilla-c2.toml"
        } else {
            "target/protocol-conformance/vanilla-c0-c1.toml"
        });
        while let Some(argument) = arguments.next() {
            let value = arguments
                .next()
                .ok_or_else(|| format!("{argument} requires a value"))?;
            match argument.as_str() {
                "--client-jar" => client_jar = Some(PathBuf::from(value)),
                "--registry-report" => registry_report = PathBuf::from(value),
                "--bind" => bind = value,
                "--timeout-seconds" => timeout_seconds = value.parse()?,
                "--evidence" => evidence = PathBuf::from(value),
                _ => return Err(format!("unknown vanilla-probe option: {argument}").into()),
            }
        }
        Ok(Self::VanillaProbe {
            playable,
            client_jar: client_jar.ok_or("--client-jar is required")?,
            registry_report,
            bind,
            timeout: Duration::from_secs(timeout_seconds),
            evidence,
        })
    }

    fn parse_fixture(mut arguments: impl Iterator<Item = String>) -> Result<Self, DynError> {
        let mut client_jar = None;
        let mut registry_report =
            PathBuf::from("target/mc-reference/26.2/generated/reports/registries.json");
        while let Some(argument) = arguments.next() {
            let value = arguments
                .next()
                .ok_or_else(|| format!("{argument} requires a value"))?;
            match argument.as_str() {
                "--client-jar" => client_jar = Some(PathBuf::from(value)),
                "--registry-report" => registry_report = PathBuf::from(value),
                _ => {
                    return Err(format!("unknown verify-vanilla-fixture option: {argument}").into());
                }
            }
        }
        Ok(Self::VerifyVanillaFixture {
            client_jar: client_jar.ok_or("--client-jar is required")?,
            registry_report,
        })
    }
}

fn print_help() {
    println!(
        "Usage: protocol-conformance [run|tcp-smoke|c2-smoke]\n\
         protocol-conformance connect-c2 <IP:PORT>\n\
         \n\
         protocol-conformance vanilla-probe --client-jar <PATH> [options]\n\
         protocol-conformance vanilla-c2-probe --client-jar <PATH> [options]\n\
         protocol-conformance verify-vanilla-fixture --client-jar <PATH> [options]\n\
         Options: --registry-report <PATH> --bind <IP:PORT> \
         --timeout-seconds <N> --evidence <PATH>"
    );
}
