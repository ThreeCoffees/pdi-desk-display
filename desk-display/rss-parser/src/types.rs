use core::fmt;
use heapless::{String, Vec};

pub const MAX_ITEMS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RssFeed {
    pub title: String<128>,
    pub link: String<256>,
    pub language: String<16>,
    pub items: Vec<RssItem, MAX_ITEMS>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RssItem {
    pub title: String<128>,
    pub link: String<256>,
    pub description: String<256>,
    pub author: String<128>,
    pub pub_date: String<64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    Malformed,
    TooLong,
    XmlError,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Malformed => write!(f, "malformed RSS feed"),
            ParseError::TooLong => write!(f, "field too long"),
            ParseError::XmlError => write!(f, "XML parse error"),
        }
    }
}
