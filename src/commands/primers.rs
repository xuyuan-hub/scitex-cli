use std::sync::Arc;

use clap::{Args, Subcommand};

use crate::client::ScientexClient;
use crate::config::Config;
use crate::output::{print_result, OutputFormat};

#[derive(Args)]
pub struct PrimersArgs {
    #[command(subcommand)]
    pub command: PrimersCommand,
}

#[derive(Subcommand)]
pub enum PrimersCommand {
    /// List primers created by synthesis orders.
    List {
        #[arg(long, default_value_t = 0)]
        skip: u32,
        #[arg(long, default_value_t = 200)]
        limit: u32,
    },
}

pub async fn run(
    args: &PrimersArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = ScientexClient::new(Arc::clone(config))?;
    match &args.command {
        PrimersCommand::List { skip, limit } => {
            let primers = client.list_primers(*skip, *limit).await?;
            print_result(&primers, format);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct TestCli {
        #[command(subcommand)]
        command: TestCommand,
    }

    #[derive(Subcommand)]
    enum TestCommand {
        Primers(PrimersArgs),
    }

    #[test]
    fn parses_primer_list() {
        let cli =
            TestCli::try_parse_from(["scitex", "primers", "list", "--skip", "20", "--limit", "50"])
                .expect("primers list should parse");
        assert!(matches!(
            cli.command,
            TestCommand::Primers(PrimersArgs {
                command: PrimersCommand::List {
                    skip: 20,
                    limit: 50
                }
            })
        ));
    }
}
