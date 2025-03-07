use adview::{show_fields, show_head, show_less, show_shapes};
use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use hdf5::File;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "adview")]
#[command(about = "Adata Viewer: Head/Less/Shape h5ad file in terminal")]
#[command(author, version)]
#[command(
    help_template = "{name} -- {about}\n\nVersion: {version}\n\nAuthors: {author}\
    \n\n{usage-heading} {usage}\n\n{all-args}"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args)]
struct FileArg {
    /// HDF5 file path
    #[arg(value_name = "FILE")]
    file: PathBuf,
}

#[derive(Args)]
struct HeadArg {
    #[clap(flatten)]
    file_arg: FileArg,

    /// Number of lines to show
    #[arg(short = 'n', long = "lines", default_value = "10")]
    lines: usize,
}

#[derive(Subcommand)]
enum Commands {
    /// Show first n obs
    #[command(visible_alias = "oh")]
    ObsHead(#[clap(flatten)] HeadArg),
    /// Show all obs
    #[command(visible_alias = "oa")]
    ObsAll(#[clap(flatten)] FileArg),
    /// Show first n var
    #[command(visible_alias = "vh")]
    VarHead(#[clap(flatten)] HeadArg),
    /// Show all var
    #[command(visible_alias = "va")]
    VarAll(#[clap(flatten)] FileArg),
    /// Show shapes of obs and var
    #[command(visible_alias = "s")]
    Shape(#[clap(flatten)] FileArg),
    /// Show fields in obs and var
    #[command(visible_alias = "f")]
    Field(#[clap(flatten)] FileArg),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // open file
    let open_file = |path: &PathBuf| -> Result<File> {
        File::open(path).with_context(|| format!("Failed to open file: {:?}", path))
    };

    match cli.command {
        Commands::ObsHead(args) => show_head(&open_file(&args.file_arg.file)?, "obs", args.lines)?,
        Commands::ObsAll(args) => show_less(&open_file(&args.file)?, "obs")?,
        Commands::VarHead(args) => show_head(&open_file(&args.file_arg.file)?, "var", args.lines)?,
        Commands::VarAll(args) => show_less(&open_file(&args.file)?, "var")?,
        Commands::Shape(args) => show_shapes(&open_file(&args.file)?)?,
        Commands::Field(args) => show_fields(&open_file(&args.file)?)?,
    }

    Ok(())
}
