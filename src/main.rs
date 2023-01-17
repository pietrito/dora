mod device;
mod errors;
mod globals;

use crate::device::Device;
use crate::errors::Result;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, name = "BYTES")]
    sector_size: Option<usize>,

    #[arg(long, name = "OFFSET")]
    seek: Option<String>,

    #[arg(long)]
    status: Option<bool>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Find {
        #[arg(short, long)]
        path: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Find { path } => {
            let mut device =
                match Device::new(PathBuf::from(path), cli.sector_size, cli.seek, cli.status) {
                    Err(e) => {
                        eprintln!("{e:?}");
                        return Ok(());
                    }
                    Ok(d) => d,
                };
            println!("Device size: {}Gb", device.get_size()? / globals::GB_SIZE);
            let _ = device.find_first_non_zero()?;
        }
    }

    Ok(())
}
