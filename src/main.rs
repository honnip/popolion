use clap::{Parser, Subcommand};
use globset::Glob;
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
                // TODO do without label
                'entry: for i in 0..ipf.len() {
                    let mut entry = ipf.by_index(i).unwrap();
                    if let Some(exclude) = &exclude {
                        for pattern in exclude {
                            if let Some(whitelist) = pattern.strip_prefix('!') {
                                let glob = Glob::new(whitelist)
                                    .expect("Invalid glob pattern")
                                    .compile_matcher();
                                if !glob.is_match(entry.full_path()) {
                                    continue 'entry;
                                }
                            } else {
                                let glob = Glob::new(pattern)
                                    .expect("Invalid glob pattern")
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
                        path.push(file.file_stem().unwrap());
                    }

                    path.push(sanitize(entry.full_path()));

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
