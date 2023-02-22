use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use globset::Glob;
use indicatif::{ProgressBar, ProgressStyle};
use libtos::IpfArchive;
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::{self, PathBuf},
    sync::{Arc, Mutex},
};

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
                let ipf_file = File::open(file)
                    .with_context(|| format!("could not open file: {}", file.display()))
                    .unwrap();
                let ipf_file = CloneableSeekableReader::new(ipf_file);
                let ipf = IpfArchive::new(ipf_file)
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

                (0..ipf.len()).into_par_iter().for_each(|i| {
                    let mut ipf = ipf.clone();
                    let mut entry = ipf.by_index(i).unwrap();

                    // sorry
                    let mut status = true;

                    // --exclude
                    if let Some(exclude) = &exclude {
                        for exclude in exclude.iter() {
                            if match_glob(exclude, &entry.full_path()).unwrap() {
                                status = false;
                                break;
                            }
                        }
                    }

                    let mut path = PathBuf::new();
                    if let Some(dest) = &dest {
                        path.push(dest);
                    }

                    if sub {
                        path.push(
                            file.file_stem()
                                .with_context(|| {
                                    format!("input file has no filename: {}", file.display())
                                })
                                .unwrap(),
                        );
                    }

                    path.push(sanitize(entry.full_path()));

                    if path.exists() {
                        if never_overwrite {
                            status = false;
                        } else {
                            // pb.println(format!("{} already exists, overwriting", path.display()))
                        }
                    }

                    // sorry
                    if status {
                        if let Some(p) = path.parent() {
                            if !p.exists() {
                                std::fs::create_dir_all(p)
                                    .with_context(|| {
                                        format!("failed to create directory: {}", p.display())
                                    })
                                    .unwrap();
                            }
                        }

                        let mut file = std::fs::File::create(path.clone())
                            .with_context(|| format!("failed to create file: {}", path.display()))
                            .unwrap();
                        std::io::copy(&mut entry, &mut file)
                            .with_context(|| format!("failed to write file: {}", path.display()))
                            .unwrap();
                    }

                    // update progress bar
                    pb.inc(1);
                });
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

fn match_glob(pattern: &str, path: &PathBuf) -> Result<bool> {
    if let Some(whitelist) = pattern.strip_prefix('!') {
        let glob = Glob::new(whitelist)
            .with_context(|| format!("invalid glob pattern: {}", pattern))?
            .compile_matcher();
        if !glob.is_match(path) {
            return Ok(true);
        }
    } else {
        let glob = Glob::new(pattern)
            .with_context(|| format!("invalid glob pattern: {}", pattern))?
            .compile_matcher();
        if glob.is_match(path) {
            return Ok(true);
        }
    }

    Ok(false)
}

// Thanks to adetaylor
// https://github.com/google/ripunzip/blob/main/src/unzip/cloneable_seekable_reader.rs

trait HasLength {
    /// return the current total length of this stream
    fn len(&self) -> u64;
}

struct CloneableSeekableReader<R: Read + Seek + HasLength> {
    file: Arc<Mutex<R>>,
    pos: u64,
    // TODO determine and store this once instead of per cloneable file
    file_length: Option<u64>,
}

impl<R: Read + Seek + HasLength> Clone for CloneableSeekableReader<R> {
    fn clone(&self) -> Self {
        Self {
            file: self.file.clone(),
            pos: self.pos,
            file_length: self.file_length,
        }
    }
}

impl<R: Read + Seek + HasLength> CloneableSeekableReader<R> {
    fn new(file: R) -> Self {
        Self {
            file: Arc::new(Mutex::new(file)),
            pos: 0,
            file_length: None,
        }
    }

    fn ascertain_file_length(&mut self) -> u64 {
        match self.file_length {
            Some(file_length) => file_length,
            None => {
                let len = self.file.lock().unwrap().len();
                self.file_length = Some(len);
                len
            }
        }
    }
}

impl<R: Read + Seek + HasLength> Read for CloneableSeekableReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut underlying_file = self.file.lock().expect("Unable to get underlying file");
        // TODO share an object which knows current position to avoid unnecessary

        underlying_file.seek(SeekFrom::Start(self.pos))?;
        let read_result = underlying_file.read(buf);
        if let Ok(bytes_read) = read_result {
            // TODO, once stabilised, use checked_add_signed
            self.pos += bytes_read as u64;
        }
        read_result
    }
}

impl<R: Read + Seek + HasLength> Seek for CloneableSeekableReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(pos) => pos,
            SeekFrom::End(offset_from_end) => {
                let file_len = self.ascertain_file_length();
                if -offset_from_end as u64 > file_len {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Seek too far backwards",
                    ));
                }
                // TODO, once stabilised, use checked_add_signed
                file_len - (-offset_from_end as u64)
            }
            SeekFrom::Current(offset_from_pos) => {
                if offset_from_pos > 0 {
                    self.pos + (offset_from_pos as u64)
                } else {
                    self.pos - ((-offset_from_pos) as u64)
                }
            }
        };
        self.pos = new_pos;
        Ok(new_pos)
    }
}

impl<R: HasLength> HasLength for BufReader<R> {
    fn len(&self) -> u64 {
        self.get_ref().len()
    }
}

impl HasLength for File {
    fn len(&self) -> u64 {
        self.metadata().unwrap().len()
    }
}
