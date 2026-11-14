use heapless::{String, Vec};

use crate::entity::decode_xml_entity;
use crate::html::HtmlToText;
use crate::types::{MAX_ITEMS, ParseError, RssFeed, RssItem};

const DESCRIPTION_MAX: usize = 256;
const TAG_NAME_MAX: usize = 32;
const ENTITY_BUF_MAX: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    Root,
    Channel,
    Item,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    None,
    FeedTitle,
    FeedLink,
    FeedLanguage,
    ItemTitle,
    ItemLink,
    ItemDescription,
    ItemAuthor,
    ItemPubDate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum State {
    Text,
    Tag {
        state: TagState,
        name: String<TAG_NAME_MAX>,
        attr_name: String<TAG_NAME_MAX>,
        closing: bool,
    },
    Entity(String<ENTITY_BUF_MAX>),
    Comment,
    CData { saw_close_bracket: bool },
    Pi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagState {
    Name,
    BeforeAttr,
    AttrName,
    AttrEquals,
    AttrValue { quote: char },
}

const FIELD_TAGS: &[(&str, Context, Field)] = &[
    ("title", Context::Channel, Field::FeedTitle),
    ("title", Context::Item, Field::ItemTitle),
    ("link", Context::Channel, Field::FeedLink),
    ("link", Context::Item, Field::ItemLink),
    ("language", Context::Channel, Field::FeedLanguage),
    ("author", Context::Item, Field::ItemAuthor),
    ("pubdate", Context::Item, Field::ItemPubDate),
];

fn lookup_field(name: &str, context: Context) -> Field {
    for (tag, ctx, field) in FIELD_TAGS.iter() {
        if *tag == name && *ctx == context {
            return *field;
        }
    }
    Field::None
}

struct ItemAccum {
    title: String<128>,
    link: String<256>,
    description_raw: String<1024>,
    author: String<128>,
    pub_date: String<64>,
    in_description: bool,
}

impl ItemAccum {
    const fn new() -> Self {
        Self {
            title: String::new(),
            link: String::new(),
            description_raw: String::new(),
            author: String::new(),
            pub_date: String::new(),
            in_description: false,
        }
    }

    fn clear(&mut self) {
        *self = Self::new();
    }
}

pub struct RssParser {
    state: State,
    context_stack: [Context; 8],
    depth: usize,
    current_field: Field,
    text_buf: String<512>,
    feed_title: String<128>,
    feed_link: String<256>,
    feed_language: String<16>,
    item: ItemAccum,
    items: Vec<RssItem, MAX_ITEMS>,
    root_found: bool,
    channel_found: bool,
    error: Option<ParseError>,
}

impl RssParser {
    pub fn new() -> Self {
        Self {
            state: State::Text,
            context_stack: [Context::Root; 8],
            depth: 0,
            current_field: Field::None,
            text_buf: String::new(),
            feed_title: String::new(),
            feed_link: String::new(),
            feed_language: String::new(),
            item: ItemAccum::new(),
            items: Vec::new(),
            root_found: false,
            channel_found: false,
            error: None,
        }
    }

    pub fn push(&mut self, input: &str) {
        if self.error.is_some() {
            return;
        }

        for ch in input.chars() {
            self.state = self.process_char(ch);
            if self.error.is_some() {
                return;
            }
        }
    }

    pub fn finish(&mut self) -> Result<RssFeed, ParseError> {
        if let Some(err) = self.error {
            return Err(err);
        }

        if !self.root_found {
            return Err(ParseError::Malformed);
        }

        if !self.channel_found {
            return Err(ParseError::Malformed);
        }

        Ok(RssFeed {
            title: self.feed_title.clone(),
            link: self.feed_link.clone(),
            language: self.feed_language.clone(),
            items: self.items.clone(),
        })
    }

    fn process_char(&mut self, ch: char) -> State {
        let state = core::mem::replace(&mut self.state, State::Text);
        match state {
            State::Text => self.process_text(ch),
            State::Tag { state, name, attr_name, closing } => {
                self.process_tag(state, name, attr_name, closing, ch)
            }
            State::Entity(buf) => self.process_entity(buf, ch),
            State::Comment => self.process_comment(ch),
            State::Pi => self.process_pi(ch),
            State::CData { saw_close_bracket } => {
                self.process_cdata(saw_close_bracket, ch)
            }
        }
    }

    fn process_text(&mut self, ch: char) -> State {
        match ch {
            '<' => {
                if self.item.in_description {
                    let _ = self.item.description_raw.push('<');
                }
                State::Tag {
                    state: TagState::Name,
                    name: String::new(),
                    attr_name: String::new(),
                    closing: false,
                }
            }
            '&' => State::Entity(String::new()),
            _ => {
                self.emit_text(ch);
                State::Text
            }
        }
    }

    fn process_tag(
        &mut self,
        state: TagState,
        mut name: String<TAG_NAME_MAX>,
        mut attr_name: String<TAG_NAME_MAX>,
        mut closing: bool,
        ch: char,
    ) -> State {
        if self.item.in_description {
            let _ = self.item.description_raw.push(ch);
        }

        let state = match state {
            TagState::Name => {
                if ch == '>' {
                    return self.finish_tag(name, closing);
                }
                if ch == '/' {
                    closing = true;
                    return State::Tag { state: TagState::Name, name, attr_name, closing };
                }
                if ch.is_ascii_whitespace() {
                    return State::Tag { state: TagState::BeforeAttr, name, attr_name, closing };
                }
                if ch == '!' || ch == '?' {
                    return self.handle_special_tag(ch);
                }
                let lower = ch.to_ascii_lowercase();
                if name.push(lower).is_err() {
                    self.error = Some(ParseError::XmlError);
                    return State::Text;
                }
                TagState::Name
            }
            TagState::BeforeAttr => {
                if ch == '>' {
                    return self.finish_tag(name, closing);
                }
                if ch == '/' {
                    return State::Tag { state: TagState::BeforeAttr, name, attr_name, closing: true };
                }
                if ch.is_ascii_whitespace() {
                    return State::Tag { state: TagState::BeforeAttr, name, attr_name, closing };
                }
                attr_name.clear();
                let _ = attr_name.push(ch.to_ascii_lowercase());
                TagState::AttrName
            }
            TagState::AttrName => {
                if ch == '=' {
                    return State::Tag { state: TagState::AttrEquals, name, attr_name, closing };
                }
                if ch.is_ascii_whitespace() {
                    attr_name.clear();
                    return State::Tag { state: TagState::BeforeAttr, name, attr_name, closing };
                }
                if ch == '>' {
                    return self.finish_tag(name, closing);
                }
                let _ = attr_name.push(ch.to_ascii_lowercase());
                TagState::AttrName
            }
            TagState::AttrEquals => {
                if ch.is_ascii_whitespace() {
                    return State::Tag { state: TagState::AttrEquals, name, attr_name, closing };
                }
                let quote = if ch == '"' || ch == '\'' {
                    ch
                } else {
                    let _ = attr_name.push(ch);
                    return State::Tag { state: TagState::AttrName, name, attr_name, closing };
                };
                TagState::AttrValue { quote }
            }
            TagState::AttrValue { quote } => {
                if ch == quote {
                    attr_name.clear();
                    return State::Tag { state: TagState::BeforeAttr, name, attr_name, closing };
                }
                TagState::AttrValue { quote }
            }
        };

        State::Tag { state, name, attr_name, closing }
    }

    fn finish_tag(&mut self, name: String<TAG_NAME_MAX>, closing: bool) -> State {
        let name_str = name.as_str();

        if self.item.in_description {
            if closing && name_str == "description" {
                self.handle_close_tag(name_str);
            }
        } else if closing {
            if self.depth > 0 {
                self.handle_close_tag(name_str);
            }
        } else {
            self.handle_open_tag(name_str);
        }
        State::Text
    }

    fn handle_open_tag(&mut self, name: &str) {
        if self.depth >= self.context_stack.len() {
            self.error = Some(ParseError::XmlError);
            return;
        }

        let prev_context = if self.depth > 0 {
            self.context_stack[self.depth - 1]
        } else {
            Context::Root
        };

        match name {
            "rss" | "feed" => {
                self.root_found = true;
                self.context_stack[self.depth] = Context::Root;
                self.current_field = Field::None;
            }
            "channel" => {
                if prev_context != Context::Root {
                    self.error = Some(ParseError::Malformed);
                    return;
                }
                self.channel_found = true;
                self.context_stack[self.depth] = Context::Channel;
                self.current_field = Field::None;
            }
            "item" => {
                if prev_context != Context::Channel {
                    self.error = Some(ParseError::Malformed);
                    return;
                }
                self.item.clear();
                self.context_stack[self.depth] = Context::Item;
                self.current_field = Field::None;
            }
            "description" => {
                if prev_context == Context::Item {
                    self.current_field = Field::ItemDescription;
                    self.item.in_description = true;
                    self.item.description_raw.clear();
                } else {
                    self.current_field = Field::None;
                }
            }
            _ => {
                self.current_field = lookup_field(name, prev_context);
            }
        }

        self.text_buf.clear();
        self.depth += 1;
    }

    fn handle_close_tag(&mut self, name: &str) {
        if self.depth == 0 {
            self.error = Some(ParseError::Malformed);
            return;
        }

        self.depth -= 1;
        let closing_context = self.context_stack[self.depth];
        let trimmed = self.text_buf.trim();

        match name {
            "description" => {
                if self.current_field == Field::ItemDescription && self.item.in_description {
                    self.item.in_description = false;
                    let mut stripped = String::<{ DESCRIPTION_MAX }>::new();
                    let mut html = HtmlToText::new();
                    html.push(self.item.description_raw.as_str(), &mut stripped);
                    html.finish(&mut stripped);
                    self.item.description_raw.clear();
                    let _ = self.item.description_raw.push_str(stripped.as_str());
                }
            }
            "item" => {
                if closing_context == Context::Item {
                    let rss_item = RssItem {
                        title: self.item.title.clone(),
                        link: self.item.link.clone(),
                        description: {
                            let mut desc = String::<{ DESCRIPTION_MAX }>::new();
                            let _ = desc.push_str(self.item.description_raw.as_str());
                            desc
                        },
                        author: self.item.author.clone(),
                        pub_date: self.item.pub_date.clone(),
                    };
                    let _ = self.items.push(rss_item);
                }
            }
            _ => {}
        }

        match self.current_field {
            Field::FeedTitle => { let _ = self.feed_title.push_str(trimmed); }
            Field::FeedLink => { let _ = self.feed_link.push_str(trimmed); }
            Field::FeedLanguage => { let _ = self.feed_language.push_str(trimmed); }
            Field::ItemTitle => { let _ = self.item.title.push_str(trimmed); }
            Field::ItemLink => { let _ = self.item.link.push_str(trimmed); }
            Field::ItemAuthor => { let _ = self.item.author.push_str(trimmed); }
            Field::ItemPubDate => { let _ = self.item.pub_date.push_str(trimmed); }
            Field::ItemDescription | Field::None => {}
        }

        self.current_field = Field::None;
        self.text_buf.clear();
    }

    fn handle_special_tag(&mut self, ch: char) -> State {
        match ch {
            '!' => State::Comment,
            '?' => State::Pi,
            _ => State::Text,
        }
    }

    fn process_comment(&mut self, ch: char) -> State {
        if ch == '-' {
            return State::Comment;
        }
        if ch == '[' {
            return State::CData { saw_close_bracket: false };
        }
        if ch == '>' {
            return State::Text;
        }
        State::Comment
    }

    fn process_pi(&mut self, ch: char) -> State {
        if ch == '>' {
            return State::Text;
        }
        State::Pi
    }

    fn process_cdata(&mut self, mut saw_close_bracket: bool, ch: char) -> State {
        if ch == ']' {
            saw_close_bracket = true;
            return State::CData { saw_close_bracket };
        }
        if ch == '>' && saw_close_bracket {
            return State::Text;
        }
        saw_close_bracket = false;
        if self.item.in_description {
            let _ = self.item.description_raw.push(ch);
        }
        State::CData { saw_close_bracket }
    }

    fn process_entity(&mut self, mut buf: String<ENTITY_BUF_MAX>, ch: char) -> State {
        match ch {
            ';' => {
                self.emit_entity(&buf, true);
                State::Text
            }
            '&' => {
                let prev = core::mem::replace(&mut buf, String::new());
                self.emit_entity(&prev, false);
                self.emit_text('&');
                State::Entity(buf)
            }
            '<' => {
                self.emit_entity(&buf, false);
                State::Tag {
                    state: TagState::Name,
                    name: String::new(),
                    attr_name: String::new(),
                    closing: false,
                }
            }
            _ => {
                if buf.push(ch).is_err() {
                    self.emit_entity(&buf, false);
                    self.emit_text(ch);
                    return State::Text;
                }
                State::Entity(buf)
            }
        }
    }

    fn emit_text(&mut self, ch: char) {
        if self.item.in_description {
            let _ = self.item.description_raw.push(ch);
        } else if self.current_field != Field::None {
            let _ = self.text_buf.push(ch);
        }
    }

    fn emit_entity(&mut self, entity: &String<ENTITY_BUF_MAX>, terminated: bool) {
        if let Some(c) = decode_xml_entity(entity.as_str()) {
            self.emit_text(c);
            return;
        }

        self.emit_text('&');
        for ch in entity.chars() {
            self.emit_text(ch);
        }
        if terminated {
            self.emit_text(';');
        }
    }
}

impl Default for RssParser {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse(xml: &str) -> Result<RssFeed, ParseError> {
    let mut parser = RssParser::new();
    parser.push(xml);
    parser.finish()
}
