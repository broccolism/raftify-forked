include!(concat!(env!("OUT_DIR"), "/built.rs"));

use crate::commands::debug::debug_persisted;
use clap::{App, Arg, SubCommand};
use raftify::raft::default_logger;
use raftify::Result;

mod commands;

fn main() -> Result<()> {
    let logger = default_logger();

    let matches = App::new("raftify")
        .version(PKG_VERSION)
        .author(PKG_AUTHORS)
        .about(PKG_DESCRIPTION)
        .subcommand(
            SubCommand::with_name("debug")
                .about("Debug tools")
                .subcommand(
                    SubCommand::with_name("persisted")
                        .about("List persisted log entries and metadata")
                        .arg(
                            Arg::with_name("path")
                                .help("The log dir path")
                                .required(true)
                                .index(1),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("entries")
                        .about("List all log entries")
                        .arg(
                            Arg::with_name("address")
                                .help("The address of the RaftNode")
                                .required(true)
                                .index(1),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("node").about("Inspect RaftNode").arg(
                        Arg::with_name("address")
                            .help("The address of the RaftNode")
                            .required(true)
                            .index(1),
                    ),
                ),
        )
        // .subcommand(SubCommand::with_name("health").about("Check health"))
        .get_matches();

    match matches.subcommand() {
        Some(("debug", debug_matches)) => {
            match debug_matches.subcommand() {
                Some(("entries", entries_matches)) => {
                    if let Some(address) = entries_matches.value_of("address") {
                        // 'entries' 서브커맨드 로직 구현
                    }
                }
                Some(("node", node_matches)) => {
                    if let Some(address) = node_matches.value_of("address") {
                        // 'node' 서브커맨드 로직 구현
                    }
                }
                Some(("persisted", node_matches)) => {
                    if let Some(path) = node_matches.value_of("path") {
                        debug_persisted(path, logger)?;
                    }
                }
                _ => {}
            }
        }
        _ => {}
    };

    Ok(())
}
