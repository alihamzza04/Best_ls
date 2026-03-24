use chrono::{DateTime, Utc};
use clap::Parser;
use colored::Colorize;
use serde::Serialize;
use std::{
    fs::{self}, path::PathBuf, io
};
use strum::Display;
use tabled::{
    Table, Tabled,
    settings::{Color, Style, object::Columns},
};

#[derive(Debug, Display, Serialize)]
enum FileType {
    File,
    Dir,
}

#[derive(Debug, Tabled, Serialize)]
struct FileEntry {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    e_type: FileType,
    #[tabled(rename = "Size (bytes)")]
    len_bytes: u64,
    #[tabled(rename = "Modified")]
    modified: String,
}

const DATE_FORMAT: &str = "%a %b %e %Y";
const UNKNOWN_NAME: &str = "unknown name";

#[derive(Debug, Parser)]
#[command(version, about, long_about = "Best ls command ever")]
struct Cli {
    path: Option<PathBuf>,
    #[arg(short, long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();
    let path = cli.path.unwrap_or_else(|| PathBuf::from("."));

    match fs::exists(&path) {
        Ok(true) => {
            if cli.json {
                match get_files(&path) {
                    Ok(files) => {
                        match serde_json::to_string_pretty(&files) {
                            Ok(json) => println!("{}", json),
                            Err(_) => eprintln!("{}", "cannot parse json".red()),
                        }
                    }
                    Err(e) => eprintln!("{}", e.to_string().red()),
                }
            } else if let Err(e) = print_table(&path) {
                eprintln!("{}", e.to_string().red());
            }
        }
        Ok(false) => eprintln!("{}", "does not exist".red()),
        Err(_) => eprintln!("{}", "error reading directory".red()),
    }
}

fn get_files(path: &PathBuf) -> Result<Vec<FileEntry>, io::Error> {
    let read_dir = fs::read_dir(path)?;
    let mut data = Vec::with_capacity(8);

    for entry in read_dir {
        match entry {
            Ok(file) => {
                if let Ok(entry) = map_data(file) {
                    data.push(entry);
                }
            }
            Err(_) => continue,
        }
    }
    
    Ok(data)
}

fn map_data(file: fs::DirEntry) -> Result<FileEntry, io::Error> {
    let meta = fs::metadata(&file.path())?;
    
    let name = file.file_name().into_string().unwrap_or_else(|_| UNKNOWN_NAME.to_string());
    
    let modified = meta.modified()
        .map(|modi| {
            let date: DateTime<Utc> = modi.into();
            date.format(DATE_FORMAT).to_string()
        })
        .unwrap_or_default();
    
    Ok(FileEntry {
        name,
        e_type: if meta.is_dir() { FileType::Dir } else { FileType::File },
        len_bytes: meta.len(),
        modified,
    })
}


fn print_table(path: &PathBuf) -> Result<(), io::Error> {
    let files = get_files(path)?;

    let mut table = Table::new(&files);
    table
        .with(Style::rounded())
        .modify(Columns::first(), Color::FG_BRIGHT_CYAN)
        .modify(Columns::one(2), Color::FG_BRIGHT_MAGENTA)
        .modify(Columns::one(3), Color::FG_BRIGHT_YELLOW)
        .modify(Columns::one(1), Color::FG_BRIGHT_GREEN);

    println!("{}", table);
    Ok(())
}