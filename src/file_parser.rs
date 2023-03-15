use itertools::Itertools;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Lines, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Debug)]
pub struct Location(u64, u64);
#[derive(Debug)]
pub struct Page(u64);

#[derive(Debug)]
pub enum EntryType {
    Highlight,
    Note(String),
}

#[derive(Debug)]
pub struct Entry {
    title: String,
    author: String,
    kind: EntryType,
    page: Page,
    location: Location,
    creation_date: i64,
    text: String,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error during reading the file")]
    FileReadError(#[from] io::Error),
    #[error("Invalid entry type {0}, must be Highlight or Note")]
    InvalidKind(String),
    #[error("Invalid page number {0}")]
    InvalidPage(String),
    #[error("Invalid location {0}")]
    InvalidLocation(String),
    #[error("Invalid date {0}")]
    InvalidDate(String),
    #[error("Unknown error during parsing the file")]
    UnknownError,
}

pub fn parse_file<P>(filename: P) -> Result<Vec<Entry>, ParseError>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).map_err(ParseError::FileReadError)?;
    parse_lines(BufReader::new(file).lines())
}

fn parse_lines(lines: Lines<BufReader<File>>) -> Result<Vec<Entry>, ParseError> {
    let separator = "==========";
    lines
        .flatten()
        .group_by(|line| line != separator)
        .into_iter()
        .filter(|(id, _)| *id)
        .map(|(_, group)| parse_entry(group.collect()))
        .collect()
}

fn parse_entry(lines: Vec<String>) -> Result<Entry, ParseError> {
    Ok(Entry {
        title: "".to_string(),
        author: "".to_string(),
        kind: EntryType::Highlight,
        page: Page(0),
        location: Location(1, 2),
        creation_date: 0,
        text: "".to_string(),
    })
}
