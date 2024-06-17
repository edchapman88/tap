use clap::{Parser, Subcommand};

use tap::{tap_in, tap_out};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    direction: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// You're starting work.
    In,
    /// Home time.
    Out,
}
fn main() {
    let cli = Cli::parse();

    match cli.direction {
        Commands::In => tap_in(),
        Commands::Out => tap_out(),
    }
}
