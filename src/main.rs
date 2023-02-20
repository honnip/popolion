use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use globset::Glob;
use indicatif::{ProgressBar, ProgressStyle};
use libtos::IpfArchive;
use std::path::{self, PathBuf};

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
        /// path to ipf archives
        #[arg(required = true)]
        files: Vec<PathBuf>,

        /// extract to Sub-directory (with same name as filename)
        #[arg(short, long)]
        sub: bool,

        /// eXclude files with glob pattern
        #[arg(short = 'x', long)]
        exclude: Option<Vec<String>>,

        /// Never overwrite files (overwrite files by default)
        #[arg(short = 'n', long)]
        never_overwrite: bool,

        /// Destination directory
        #[arg(short, long)]
        dest: Option<PathBuf>,
    },
    /// List files in ipf archive
    List {
        /// path to ipf archive
        #[arg(required = true)]
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        SubCommands::Extract {
            files,
            sub,
            exclude,
            never_overwrite,
            dest,
        } => {
            for file in files.iter() {
                // open ipf archive
                let mut ipf = IpfArchive::open(file)
                    .with_context(|| format!("could not read file: {}", file.display()))?;

                // init progress bar
                let pb = ProgressBar::new(ipf.len() as u64);
                pb.set_style(
                    ProgressStyle::with_template(
                        "{spinner:.green} [{elapsed_precise}] [{msg}] [{wide_bar:.cyan/blue}] [{human_pos}/{human_len:^}]",
                    )
                    .unwrap()
                    .progress_chars("#>-"),
                );
                let filename = file.file_name().unwrap().to_str().unwrap_or("?");
                pb.set_message(filename.to_owned());

                'entry: for i in 0..ipf.len() {
                    let mut entry = ipf.by_index(i)?;

                    // --exclude
                    if let Some(exclude) = &exclude {
                        for pattern in exclude {
                            if let Some(whitelist) = pattern.strip_prefix('!') {
                                let glob = Glob::new(whitelist)
                                    .with_context(|| format!("invalid glob pattern: {}", pattern))?
                                    .compile_matcher();
                                if !glob.is_match(entry.full_path()) {
                                    continue 'entry;
                                }
                            } else {
                                let glob = Glob::new(pattern)
                                    .with_context(|| format!("invalid glob pattern: {}", pattern))?
                                    .compile_matcher();
                                if glob.is_match(entry.full_path()) {
                                    continue 'entry;
                                }
                            }
                        }
                    }

                    let mut path = PathBuf::new();
                    if let Some(dest) = &dest {
                        path.push(dest);
                    }

                    if sub {
                        path.push(file.file_stem().with_context(|| {
                            format!("input file has no filename: {}", file.display())
                        })?);
                    }

                    path.push(sanitize(entry.full_path()));

                    if path.exists() {
                        if never_overwrite {
                            continue;
                        }
                        pb.println(format!("{} already exists, overwriting", path.display()))
                    }

                    if let Some(p) = path.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p).with_context(|| {
                                format!("failed to create directory: {}", p.display())
                            })?;
                        }
                    }

                    let mut file = std::fs::File::create(path.clone())
                        .with_context(|| format!("failed to create file: {}", path.display()))?;
                    std::io::copy(&mut entry, &mut file)
                        .with_context(|| format!("failed to write file: {}", path.display()))?;

                    // update progress bar
                    pb.inc(1);
                }
                pb.finish_and_clear();
            }
        }
        SubCommands::List { file } => {
            let mut ipf = IpfArchive::open(file)?;
            for i in 0..ipf.len() {
                let entry = ipf.by_index(i).unwrap();
                println!("{}", entry.full_path().display());
            }
        }
    }

    Ok(())
}

fn sanitize(path: PathBuf) -> PathBuf {
    let mut sanitized = PathBuf::new();
    for component in path.components() {
        match component {
            path::Component::Normal(s) => {
                sanitized.push(sanitize_filename::sanitize(s.to_str().unwrap()));
            }
            _ => sanitized.push(component),
        }
    }
    sanitized
}
