use anyhow::Result;
use clap::{Parser, Subcommand};

use sawitdb::{cli::{Cli as SawitCli, run_cli}, engine::Engine, server};

#[derive(Parser)]
#[command(name="sawitdb", version, about="SawitDB-like server + CLI")]
struct App {
    #[command(subcommand)]
    cmd: AppCmd,
}

#[derive(Subcommand)]
enum AppCmd {
    Serve {
        #[arg(long, default_value="127.0.0.1:27017")]
        addr: String,

        #[arg(long, default_value="data")]
        data_dir: String,
    },

    Cli(SawitCli),
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = App::parse();
    match app.cmd {
        AppCmd::Serve { addr, data_dir } => {
            let engine = Engine::new(data_dir)?;
            server::serve(&addr, engine).await
        }
        AppCmd::Cli(cli) => run_cli(cli).await,
    }
}
