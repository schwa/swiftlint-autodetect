use anyhow::Result;
use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;

mod swiftlint;
use swiftlint::Swiftlint;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Count the number of SwiftLint rule violations in a project.
    Count {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Generate a SwiftLint configuration, by disabling rules with a minimum number of violations.
    Generate {
        /// Path to the project.
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Include violation counts in the generated configuration.
        #[clap(long = "counts", short = 'c')]
        include_counts: bool,

        /// Output path for the generated configuration.
        #[clap(long, short)]
        output: Option<PathBuf>,

        /// Minimum number of violations required to disable a rule.
        #[clap(long, short, default_value = "1")]
        minimum_violations: u32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Count { path }) => {
            let swiftlint = Swiftlint::new(path)?;
            swiftlint.count()?;
        }
        Some(Commands::Generate {
            path,
            include_counts,
            output,
            minimum_violations,
        }) => {
            let swiftlint = Swiftlint::new(path)?;
            swiftlint.generate(output, include_counts, minimum_violations)?;
        }
        None => {
            println!("No command provided");
        }
    }
    Ok(())
}
