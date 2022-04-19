use std::error::Error;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::PathBuf;

use thalweg::bathymetry::Bathymetry;
use thalweg::format::{self, OutputFormat};
use thalweg::generator::ThalwegGenerator;
use thalweg::read;

use clap::{Args, Parser, Subcommand};

/// Generate a thalweg of an inlet
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[clap(flatten)]
    args: CommonArgs,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generates a new thalweg
    Generate,

    /// Apply a path to bathymetry, effectively creating a thalweg
    FromPath,
}

// common arguments
#[derive(Args, Debug)]
struct CommonArgs {
    /// File containing the relevant points along the inlet
    points: OsString,

    /// Directory containing NONNA-10 bathymetry data
    data: OsString,

    /// Directory to write resulting path to
    #[clap(short, long, default_value = ".")]
    prefix: OsString,

    /// Format of output file.
    /// Has no effect on section info output
    #[clap(short, long, default_value_t = OutputFormat::default())]
    format: OutputFormat,

    /// Resolution of desired thalweg in metres
    #[clap(short, long, default_value_t = 1000)]
    resolution: usize,

    /// Skip adding resolution to final thalweg
    #[clap(short, long)]
    sparse: bool,

    /// Attempt a best first guess by using depths in bathymetry as weights
    #[clap(short, long)]
    weighted: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut data = vec![];
    for entry in fs::read_dir(cli.args.data)? {
        let file_name = entry?.path();
        let file = File::open(file_name)?;
        let mut reader = BufReader::new(file);
        data.extend(read::bathymetry::from_nonna(&mut reader)?);
    }
    println!("{} data values", data.len());
    let data = data.into_iter().filter(|bath| bath.depth() > 0.0).collect();

    let points = {
        let point_path = PathBuf::from(cli.args.points);
        let points = File::open(&point_path)?;
        let mut reader = BufReader::new(points);
        if let Some(ext) = point_path.extension() {
            match ext.to_str() {
                Some("txt") => read::point::from_nonna(&mut reader)?,
                Some("csv") => read::point::from_csv(&mut reader)?,
                Some(..) => vec![],
                None => vec![],
            }
        } else {
            read::point::from_nonna(&mut reader)?
        }
    };

    let generator = ThalwegGenerator::new(data, cli.args.resolution, cli.args.weighted);

    let path = match &cli.command {
        Commands::Generate => {
            // points represents points of interest along the inlet
            let mut full_path = vec![];
            for ends in points.windows(2) {
                let source = *ends.first().expect("no source");
                let sink = *ends.last().expect("no sink");
                if let Some(mut path) = generator.thalweg(source, sink) {
                    full_path.append(&mut path);
                } else {
                    return Err(Box::<dyn Error>::from(format!(
                        "No path found between {:?} and {:?}",
                        source, sink
                    )));
                }
            }
            println!("path contains {} points", full_path.len());
            improve(&full_path, &generator)
        }
        Commands::FromPath => {
            // points represents a full path along the inlet
            generator.from_path(&points)
        }
    };

    let path = if !cli.args.sparse {
        println!("Increasing density of path");
        generator.populate(&path)
    } else {
        path
    };

    let path_vec = format::convert(cli.args.format, &path);

    let output_path = PathBuf::from(cli.args.prefix);
    let output_file = output_path
        .join("path.txt")
        .with_extension(format::extension(cli.args.format));

    let mut file = File::create(output_file)?;
    file.write_all(path_vec.as_bytes())?;

    Ok(())
}

fn improve(path: &[Bathymetry], generator: &ThalwegGenerator) -> Vec<Bathymetry> {
    let mut current_path = generator.add_midpoints(&path);
    loop {
        // find fixed-point thalweg - mostly in an attempt to ensure the thalweg does not pass over land
        let new_path = generator.sink(&current_path);
        if new_path == current_path {
            break generator.simplify(&current_path);
        }
        // combine points that are too close and may produce strange paths on further sink steps
        current_path = generator.shrink(&new_path);
    }
}
