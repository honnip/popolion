use clap::{Parser, Subcommand};
use globset::{Glob, GlobSetBuilder};
use libtos::IpfArchive;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "Popolion")]
#[command(author = "honnip @ github")]
#[command(about = "Popolion is a cli tool for ipf archives")]
struct Cli {
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    /// Extract files from ipf archive
    Extract {
        /// Path to ipf archives
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// Extract to sub-directory (with same name as filename)
        #[arg(short, long)]
        sub: bool,

        /// Exclude files with regex pattern
        #[arg(short, long)]
        exclude: Option<Vec<String>>,

        /// Never overwrite files (Overwrite files by default)
        #[arg(short = 'n', long)]
        never_overwrite: bool,

        /// Destination directory
        #[arg(short, long)]
        dest: Option<PathBuf>,
    },
    /// List files in ipf archive
    List {
        /// Path to ipf archive
        #[arg(required = true)]
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        SubCommands::Extract {
            files,
            sub,
            exclude,
            never_overwrite,
            dest,
        } => {
            for file in &files {
                // TODO error handling
                let mut ipf = IpfArchive::open(file).unwrap();
                for i in 0..ipf.len() {
                    let mut entry = ipf.by_index(i).unwrap();
                    if let Some(exclude) = &exclude {
                        let mut builder = GlobSetBuilder::new();
                        exclude.iter().for_each(|pattern| {
                            builder.add(Glob::new(pattern).unwrap());
                        });
                        let set = builder.build().unwrap();
                        if !set.matches(entry.full_path()).is_empty() {
                            continue;
                        }
                    }

                    let mut path = PathBuf::new();
                    if let Some(dest) = &dest {
                        path.push(dest);
                    }
                    if sub {
                        path.push(file.file_stem().unwrap());
                    }
                    path.push(entry.full_path());

                    if never_overwrite && path.exists() {
                        continue;
                    }

                    if let Some(p) = path.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p).unwrap();
                        }
                    }

                    let mut file = std::fs::File::create(path).unwrap();
                    std::io::copy(&mut entry, &mut file).unwrap();
                }
            }
        }
        SubCommands::List { file } => {
            let mut ipf = IpfArchive::open(file).unwrap();
            for i in 0..ipf.len() {
                let entry = ipf.by_index(i).unwrap();
                println!("{}", entry.full_path().display());
            }
        }
    }
}
