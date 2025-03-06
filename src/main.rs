use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use csv::Writer;
use hdf5::types::VarLenUnicode;
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
    /// Export obs data to CSV file
    #[command(name = "export-obs", visible_alias = "e")]
    ExportObs {
        #[clap(flatten)]
        file_arg: FileArg,
        /// Output CSV file path, None for stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
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
        Commands::ExportObs { file_arg, output } => {
            export_obs_to_csv(&open_file(&file_arg.file)?, output)?
        }
    }

    Ok(())
}

fn get_index_data(
    file: &File,
    group_name: &str,
    start: usize,
    count: Option<usize>,
) -> Result<Vec<String>> {
    let index_name = file
        .group(group_name)?
        .attr("_index")?
        .read_scalar::<VarLenUnicode>()
        .with_context(|| format!("Failed to read _index attribute from {} group", group_name))?;

    let dataset = file.dataset(&format!("{}/{}", group_name, index_name.as_str()))?;
    let total_len = dataset.shape()[0];

    // if count not set, all in
    let count = count.unwrap_or(total_len - start);
    // safe count
    let count = count.min(total_len - start);

    let data = dataset
        .read_slice_1d::<VarLenUnicode, _>(start..start + count)?
        .iter()
        .map(|s| s.to_string())
        .collect();

    Ok(data)
}

fn show_head(file: &File, group_name: &str, count: usize) -> Result<()> {
    let data = get_index_data(file, group_name, 0, Some(count))?;

    for (i, value) in data.iter().enumerate() {
        println!("{}: {}", i + 1, value);
    }

    Ok(())
}

fn show_less(file: &File, group_name: &str) -> Result<()> {
    let data = get_index_data(file, group_name, 0, None)?; // all in

    for (i, value) in data.iter().enumerate() {
        println!("{}: {}", i + 1, value);
    }

    Ok(())
}

fn show_shapes(file: &File) -> Result<()> {
    let obs_shape = file
        .dataset(&format!(
            "obs/{}",
            file.group("obs")?
                .attr("_index")?
                .read_scalar::<VarLenUnicode>()?
        ))?
        .shape()[0];

    let var_shape = file
        .dataset(&format!(
            "var/{}",
            file.group("var")?
                .attr("_index")?
                .read_scalar::<VarLenUnicode>()?
        ))?
        .shape()[0];

    println!("obs shape: {}", obs_shape);
    println!("var shape: {}", var_shape);

    Ok(())
}

fn export_obs_to_csv(file: &File, output: Option<PathBuf>) -> Result<()> {
    let obs_group = file.group("obs")?;

    // get members
    let member_names = obs_group.member_names()?;

    // init writer
    let writer: Box<dyn std::io::Write> = match output {
        Some(path) => Box::new(std::fs::File::create(path)?),
        None => Box::new(std::io::stdout()),
    };

    let mut writer = Writer::from_writer(writer);

    // write header
    writer.write_record(&member_names)?;

    // get col data and judge type
    let mut all_data: Vec<Vec<String>> = Vec::new();
    for name in &member_names {
        let encoding_type = match obs_group.dataset(name) {
            Ok(dataset) => dataset
                .attr("encoding-type")?
                .read_scalar::<VarLenUnicode>()?,
            Err(_) => {
                // as group
                let group = obs_group.group(name)?;
                group
                    .attr("encoding-type")?
                    .read_scalar::<VarLenUnicode>()?
            }
        };

        let data = match encoding_type.as_str() {
            "string-array" => obs_group
                .dataset(name)?
                .read_1d::<VarLenUnicode>()?
                .iter()
                .map(|s| s.to_string())
                .collect(),
            "categorical" => {
                let group = obs_group.group(name)?;
                let categories: Vec<String> = group
                    .dataset("categories")?
                    .read_1d::<VarLenUnicode>()?
                    .iter()
                    .map(|s| s.to_string())
                    .collect();

                let codes = group.dataset("codes")?.read_1d::<u32>()?;

                codes
                    .iter()
                    .map(|&code| categories[code as usize].clone())
                    .collect()
            }
            "array" => obs_group
                .dataset(name)?
                .read_1d::<i64>()?
                .iter()
                .map(|&n| n.to_string())
                .collect(),
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported encoding-type: {}",
                    encoding_type
                ))
            }
        };

        all_data.push(data);
    }

    // write all
    for row_idx in 0..all_data[0].len() {
        let row: Vec<String> = all_data.iter().map(|col| col[row_idx].clone()).collect();
        writer.write_record(&row)?;
    }

    Ok(())
}
