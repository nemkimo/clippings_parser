use crate::file_parser::EntryType::{Bookmark, Highlight, Note};
use chrono::NaiveDateTime;
use itertools::Itertools;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::Path;
use std::str::FromStr;
use std::{fmt, io};
use thiserror::Error;

#[derive(Debug)]
pub struct Location(u64, u64);

impl FromStr for Location {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let separated: Result<Vec<u64>, Self::Err> = s
            .split('-')
            .map(|part| {
                part.parse::<u64>()
                    .map_err(|_| ParseError::InvalidLocation(part.to_string()))
            })
            .collect();

        match separated {
            Ok(values) => match values.len() {
                2 => Ok(Location(values[0], values[1])),
                1 => Ok(Location(values[0], values[0])),
                _ => Err(ParseError::InvalidLocation(s.to_string())),
            },
            Err(_) => Err(ParseError::InvalidLocation(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct Page(u64);

impl FromStr for Page {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u64>() {
            Ok(value) => Ok(Page(value)),
            Err(_) => Err(ParseError::InvalidPage(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum EntryType {
    Highlight,
    Note,
    Bookmark,
}

impl FromStr for EntryType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Highlight" => Ok(Highlight),
            "Note" => Ok(Note),
            "Bookmark" => Ok(Bookmark),
            _ => Err(ParseError::InvalidKind(s.to_string())),
        }
    }
}

pub struct Entry {
    title: String,
    author: String,
    kind: EntryType,
    page: Option<Page>,
    location: Location,
    creation_date: NaiveDateTime,
    text: String,
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} ({}) in {}: {} - page: {:?} location: {:?}, text: {}",
            self.kind,
            self.creation_date,
            self.author,
            self.title,
            self.page,
            self.location,
            self.text
        )
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error during reading the file")]
    FileReadError(#[from] io::Error),
    #[error("Title is not found")]
    TitleNotFound,
    #[error("Author is not found")]
    AuthorNotFound,
    #[error("Entry type is not found")]
    KindNotFound,
    #[error("Location is not found")]
    LocationNotFound,
    #[error("Date is not found")]
    DateNotFound,
    #[error("Invalid entry type {0}, must be Highlight or Note")]
    InvalidKind(String),
    #[error("Invalid page number {0}")]
    InvalidPage(String),
    #[error("Invalid location {0}")]
    InvalidLocation(String),
    #[error("Invalid date {0}")]
    InvalidDate(String),
}

pub fn parse_file<P>(filename: P) -> Result<Vec<Entry>, ParseError>
where
    P: AsRef<Path>,
{
    let file = File::open(filename).map_err(ParseError::FileReadError)?;
    parse_lines(BufReader::new(file).lines())
}

fn parse_lines(lines: Lines<BufReader<File>>) -> Result<Vec<Entry>, ParseError> {
    const SEPARATOR: &str = "==========";
    lines
        .flatten()
        .group_by(|line| line != SEPARATOR)
        .into_iter()
        .filter(|(id, _)| *id)
        .map(|(_, group)| parse_entry(group.collect()))
        .collect()
}

fn parse_entry(lines: Vec<String>) -> Result<Entry, ParseError> {
    let title_author_regex = Regex::new(r"^(.*) \((.*)\)$").unwrap();
    let first_line_captures = title_author_regex.captures(lines[0].as_str()).unwrap();

    let kind_page_location_date_regex =
        Regex::new(r"^- Your (.*) on( page ([0-9]+) \|)? Location ([0-9\-]+) \| Added on (.*)$")
            .unwrap();
    let second_line_captures = kind_page_location_date_regex
        .captures(lines[1].as_str())
        .unwrap();

    let title = match first_line_captures.get(1) {
        Some(value) => Ok(value.as_str().to_string()),
        None => Err(ParseError::TitleNotFound),
    }?;

    let author = match first_line_captures.get(2) {
        Some(value) => Ok(value.as_str().to_string()),
        None => Err(ParseError::AuthorNotFound),
    }?;

    let kind = match second_line_captures.get(1) {
        Some(value) => EntryType::from_str(value.as_str()),
        None => Err(ParseError::KindNotFound),
    }?;

    let page: Option<Page> = match second_line_captures.get(3) {
        Some(value) => Some(Page::from_str(value.as_str())?),
        None => None,
    };

    let location = match second_line_captures.get(4) {
        Some(value) => Location::from_str(value.as_str()),
        None => Err(ParseError::LocationNotFound),
    }?;

    let date = match second_line_captures.get(5) {
        Some(value) => NaiveDateTime::parse_from_str(value.as_str(), "%A, %B %-e, %Y %-l:%M:%S %p")
            .map_err(|_| ParseError::InvalidDate(value.as_str().to_string())),
        None => Err(ParseError::DateNotFound),
    }?;

    let text = lines[3].to_string();

    Ok(Entry {
        title,
        author,
        kind,
        page,
        location,
        creation_date: date,
        text,
    })
}
