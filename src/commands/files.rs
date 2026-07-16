use std::sync::Arc;

use clap::{Args, Subcommand};

use crate::client::ScientexClient;
use crate::config::Config;
use crate::output::{print_result, OutputFormat};

#[derive(Args)]
pub struct FilesArgs {
    #[command(subcommand)]
    pub command: FilesCommand,
}

#[derive(Subcommand)]
pub enum FilesCommand {
    /// Upload a file and return a reference for task input_data.
    Upload { file: String },
}

pub async fn run(
    args: &FilesArgs,
    config: &Arc<Config>,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let client = ScientexClient::new(Arc::clone(config))?;
    match &args.command {
        FilesCommand::Upload { file } => {
            let reference = client.upload_file_reference(file).await?;
            print_result(&reference, format);
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
        Files(FilesArgs),
    }

    #[test]
    fn parses_file_upload() {
        let cli = TestCli::try_parse_from(["scitex", "files", "upload", "input.bin"])
            .expect("files upload should parse");
        assert!(matches!(
            cli.command,
            TestCommand::Files(FilesArgs {
                command: FilesCommand::Upload { .. }
            })
        ));
    }
}
