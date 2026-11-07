use core::cell::RefCell;

use defmt::error;
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};

pub const MAX_URLS: usize = 10;
pub const MAX_URL_LEN: usize = 128;

pub static RSS_FEEDS: Mutex<CriticalSectionRawMutex, RefCell<RssFeedList>> =
    Mutex::new(RefCell::new(RssFeedList::new()));

pub fn save_rss_feeds(input: &str) -> bool {
    RSS_FEEDS.lock(|feeds| match feeds.borrow_mut().replace(input) {
        Ok(()) => true,
        Err(err) => {
            error!("Failed to save RSS feeds: {:?}", err);
            false
        }
    })
}

pub fn get_rss_feeds() -> RssFeedList {
    RSS_FEEDS.lock(|feeds| feeds.borrow().clone())
}

#[derive(Clone, defmt::Format)]
pub struct RssFeedList {
    urls: [[u8; MAX_URL_LEN]; MAX_URLS],
    lengths: [usize; MAX_URLS],
    count: usize,
}

#[derive(Debug, defmt::Format)]
pub enum RssFeedError {
    TooManyUrls,
    UrlTooLong,
}

impl RssFeedList {
    pub const fn new() -> Self {
        Self {
            urls: [[0; MAX_URL_LEN]; MAX_URLS],
            lengths: [0; MAX_URLS],
            count: 0,
        }
    }

    pub fn replace(&mut self, input: &str) -> Result<(), RssFeedError> {
        let mut new_list = Self::new();

        for line in input.lines() {
            let url = line.trim();

            if url.is_empty() {
                continue;
            }

            new_list.add(url)?;
        }

        *self = new_list;
        Ok(())
    }

    pub fn to_string(&self) -> heapless::String<{ MAX_URLS * (MAX_URL_LEN + 1) }> {
        let mut result = heapless::String::new();

        for (i, url) in self.iter().enumerate() {
            if i > 0 {
                result.push('\n').unwrap();
            }

            result.push_str(url).unwrap();
        }

        result
    }

    fn add(&mut self, url: &str) -> Result<(), RssFeedError> {
        if self.count >= MAX_URLS {
            return Err(RssFeedError::TooManyUrls);
        }

        if url.len() > MAX_URL_LEN {
            return Err(RssFeedError::UrlTooLong);
        }

        let index = self.count;

        self.urls[index][..url.len()].copy_from_slice(url.as_bytes());
        self.lengths[index] = url.len();
        self.count += 1;

        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.urls[..self.count]
            .iter()
            .zip(self.lengths[..self.count].iter())
            .map(|(url, len)| {
                core::str::from_utf8(&url[..*len]).expect("stored URLs are valid UTF-8")
            })
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}
