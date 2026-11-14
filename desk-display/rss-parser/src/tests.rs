use crate::parser::{RssParser, parse};
use crate::types::ParseError;

const SIMPLE_FEED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <link>https://example.com</link>
    <language>en-us</language>
    <item>
      <title>First Post</title>
      <link>https://example.com/post/1</link>
      <description>&lt;p&gt;Hello, world!&lt;/p&gt;</description>
      <author>alice@example.com</author>
      <pubDate>Mon, 01 Jan 2024 00:00:00 GMT</pubDate>
    </item>
    <item>
      <title>Second Post</title>
      <link>https://example.com/post/2</link>
      <description>Plain text description</description>
      <author>bob@example.com</author>
      <pubDate>Tue, 02 Jan 2024 00:00:00 GMT</pubDate>
    </item>
  </channel>
</rss>"#;

#[test]
fn parses_simple_feed() {
    let feed = parse(SIMPLE_FEED).expect("should parse");
    assert_eq!(feed.title.as_str(), "Test Feed");
    assert_eq!(feed.link.as_str(), "https://example.com");
    assert_eq!(feed.language.as_str(), "en-us");
    assert_eq!(feed.items.len(), 2);
}

#[test]
fn parses_first_item() {
    let feed = parse(SIMPLE_FEED).expect("should parse");
    let item = &feed.items[0];
    assert_eq!(item.title.as_str(), "First Post");
    assert_eq!(item.link.as_str(), "https://example.com/post/1");
    assert_eq!(item.author.as_str(), "alice@example.com");
    assert_eq!(item.pub_date.as_str(), "Mon, 01 Jan 2024 00:00:00 GMT");
    assert_eq!(item.description.as_str(), "\nHello, world!\n");
}

#[test]
fn parses_second_item() {
    let feed = parse(SIMPLE_FEED).expect("should parse");
    let item = &feed.items[1];
    assert_eq!(item.title.as_str(), "Second Post");
    assert_eq!(item.link.as_str(), "https://example.com/post/2");
    assert_eq!(item.description.as_str(), "Plain text description");
    assert_eq!(item.author.as_str(), "bob@example.com");
}

#[test]
fn strips_html_from_description() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Feed</title>
    <link>https://example.com</link>
    <item>
      <title>Post</title>
      <link>https://example.com/1</link>
      <description><p>Paragraph one</p><p><b>Bold</b> and <i>italic</i></p></description>
    </item>
  </channel>
</rss>"#;

    let feed = parse(xml).expect("should parse");
    let desc = feed.items[0].description.as_str();
    assert!(desc.contains("Paragraph one"));
    assert!(desc.contains("Bold"));
    assert!(desc.contains("italic"));
    assert!(!desc.contains("<p>"));
    assert!(!desc.contains("</p>"));
    assert!(!desc.contains("<b>"));
}

#[test]
fn handles_cdata_description() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Feed</title>
    <link>https://example.com</link>
    <item>
      <title>Post</title>
      <link>https://example.com/1</link>
      <description><![CDATA[<p>Raw HTML in CDATA</p>]]></description>
    </item>
  </channel>
</rss>"#;

    let feed = parse(xml).expect("should parse");
    let desc = feed.items[0].description.as_str();
    assert!(desc.contains("Raw HTML in CDATA"));
    assert!(!desc.contains("<p>"));
}

#[test]
fn handles_empty_feed() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Empty Feed</title>
    <link>https://example.com</link>
  </channel>
</rss>"#;

    let feed = parse(xml).expect("should parse");
    assert_eq!(feed.title.as_str(), "Empty Feed");
    assert_eq!(feed.items.len(), 0);
}

#[test]
fn handles_missing_optional_fields() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Feed</title>
    <link>https://example.com</link>
    <item>
      <title>Bare Post</title>
    </item>
  </channel>
</rss>"#;

    let feed = parse(xml).expect("should parse");
    let item = &feed.items[0];
    assert_eq!(item.title.as_str(), "Bare Post");
    assert_eq!(item.link.as_str(), "");
    assert_eq!(item.description.as_str(), "");
    assert_eq!(item.author.as_str(), "");
    assert_eq!(item.pub_date.as_str(), "");
}

#[test]
fn rejects_non_rss_xml() {
    let xml = r#"<?xml version="1.0"?>
<root>
  <not_rss>true</not_rss>
</root>"#;

    let result = parse(xml);
    assert!(matches!(result, Err(ParseError::Malformed)));
}

#[test]
fn rejects_empty_string() {
    let result = parse("");
    assert!(matches!(result, Err(ParseError::Malformed)));
}

#[test]
fn rejects_malformed_xml() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Unclosed tag
  </channel>
</rss>"#;

    let _ = parse(xml);
}

#[test]
fn handles_html_entities_in_description() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Feed</title>
    <link>https://example.com</link>
    <item>
      <title>Post</title>
      <link>https://example.com/1</link>
      <description>&lt;p&gt;Text with &amp; entities &lt;/p&gt;</description>
    </item>
  </channel>
</rss>"#;

    let feed = parse(xml).expect("should parse");
    let desc = feed.items[0].description.as_str();
    assert!(desc.contains("Text with"));
    assert!(desc.contains("entities"));
}

#[test]
fn handles_utf8_content() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>国际新闻</title>
    <link>https://example.com</link>
    <item>
      <title>東京で会議</title>
      <link>https://example.com/1</link>
      <description>日本語のテスト — café résumé</description>
    </item>
  </channel>
</rss>"#;

    let feed = parse(xml).expect("should parse");
    assert_eq!(feed.title.as_str(), "国际新闻");
    assert_eq!(feed.items[0].title.as_str(), "東京で会議");
    assert!(feed.items[0].description.as_str().contains("日本語"));
    assert!(feed.items[0].description.as_str().contains("café"));
}

#[test]
fn handles_block_tags_in_description() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Feed</title>
    <link>https://example.com</link>
    <item>
      <title>Post</title>
      <link>https://example.com/1</link>
      <description><h1>Heading</h1><p>Body text</p><br/>End</description>
    </item>
  </channel>
</rss>"#;

    let feed = parse(xml).expect("should parse");
    let desc = feed.items[0].description.as_str();
    assert!(desc.contains("Heading"));
    assert!(desc.contains("Body text"));
    assert!(desc.contains("End"));
    assert!(desc.contains('\n'));
}

#[test]
fn handles_multiple_items() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Feed</title>
    <link>https://example.com</link>
    <item>
      <title>Item 1</title>
      <link>https://example.com/1</link>
    </item>
    <item>
      <title>Item 2</title>
      <link>https://example.com/2</link>
    </item>
    <item>
      <title>Item 3</title>
      <link>https://example.com/3</link>
    </item>
  </channel>
</rss>"#;

    let feed = parse(xml).expect("should parse");
    assert_eq!(feed.items.len(), 3);
    assert_eq!(feed.items[0].title.as_str(), "Item 1");
    assert_eq!(feed.items[1].title.as_str(), "Item 2");
    assert_eq!(feed.items[2].title.as_str(), "Item 3");
}

#[test]
fn description_truncation() {
    let long_text = "a".repeat(500);
    let xml = format!(
        r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Feed</title>
    <link>https://example.com</link>
    <item>
      <title>Post</title>
      <link>https://example.com/1</link>
      <description>{}</description>
    </item>
  </channel>
</rss>"#,
        long_text
    );

    let feed = parse(&xml).expect("should parse");
    let desc = feed.items[0].description.as_str();
    assert!(desc.len() <= 256);
    assert!(desc.starts_with("aaaa"));
}

#[test]
fn handles_whitespace_only_text() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>  Feed  </title>
    <link>  https://example.com  </link>
    <item>
      <title>  Post  </title>
      <link>  https://example.com/1  </link>
    </item>
  </channel>
</rss>"#;

    let feed = parse(xml).expect("should parse");
    assert_eq!(feed.title.as_str(), "Feed");
    assert_eq!(feed.items[0].title.as_str(), "Post");
}

#[test]
fn handles_nested_tags_in_description() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Feed</title>
    <link>https://example.com</link>
    <item>
      <title>Post</title>
      <link>https://example.com/1</link>
      <description><div class="entry"><p>Text with <a href="https://example.com">link</a> inside</p></div></description>
    </item>
  </channel>
</rss>"#;

    let feed = parse(xml).expect("should parse");
    let desc = feed.items[0].description.as_str();
    assert!(desc.contains("Text with"));
    assert!(desc.contains("link"));
    assert!(desc.contains("inside"));
    assert!(!desc.contains("href"));
    assert!(!desc.contains("<div"));
}

#[test]
fn handles_item_with_no_description() {
    let xml = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Feed</title>
    <link>https://example.com</link>
    <item>
      <title>No Desc</title>
      <link>https://example.com/1</link>
      <author>test@example.com</author>
    </item>
  </channel>
</rss>"#;

    let feed = parse(xml).expect("should parse");
    let item = &feed.items[0];
    assert_eq!(item.title.as_str(), "No Desc");
    assert_eq!(item.description.as_str(), "");
    assert_eq!(item.author.as_str(), "test@example.com");
}

#[test]
fn parse_error_display() {
    assert_eq!(ParseError::Malformed.to_string(), "malformed RSS feed");
    assert_eq!(ParseError::TooLong.to_string(), "field too long");
    assert_eq!(ParseError::XmlError.to_string(), "XML parse error");
}

#[test]
fn rss_feed_and_item_are_clone() {
    let feed = parse(SIMPLE_FEED).expect("should parse");
    let _clone = feed.clone();
    let _item_clone = feed.items[0].clone();
}

#[test]
fn parses_feed_in_chunks() {
    let mut parser = RssParser::new();
    let chunk_size = 32;
    for chunk in SIMPLE_FEED.as_bytes().chunks(chunk_size) {
        let text = core::str::from_utf8(chunk).expect("valid UTF-8");
        parser.push(text);
    }
    let feed = parser.finish().expect("should parse");
    assert_eq!(feed.title.as_str(), "Test Feed");
    assert_eq!(feed.items.len(), 2);
    assert_eq!(feed.items[0].title.as_str(), "First Post");
    assert_eq!(feed.items[1].title.as_str(), "Second Post");
}

#[test]
fn parses_feed_with_tag_split_across_chunks() {
    let mut parser = RssParser::new();
    parser.push("<?xml version=\"1.0\"?>\n<rss version=\"2.0\">\n  <chann");
    parser.push("el>\n    <title>Split</title>\n  </channel>\n</rss>");
    let feed = parser.finish().expect("should parse");
    assert_eq!(feed.title.as_str(), "Split");
}

#[test]
fn parses_feed_with_entity_split_across_chunks() {
    let mut parser = RssParser::new();
    parser.push("<?xml version=\"1.0\"?>\n<rss version=\"2.0\">\n  <channel>");
    parser.push("\n    <title>A &am");
    parser.push("p; B</title>\n  </channel>\n</rss>");
    let feed = parser.finish().expect("should parse");
    assert_eq!(feed.title.as_str(), "A & B");
}

#[test]
fn parses_feed_with_description_split_across_chunks() {
    let mut parser = RssParser::new();
    parser.push("<?xml version=\"1.0\"?>\n<rss version=\"2.0\">\n  <channel>\n    <item>\n      <title>T</title>\n      <description><p>Hel");
    parser.push("lo</p></description>\n    </item>\n  </channel>\n</rss>");
    let feed = parser.finish().expect("should parse");
    assert_eq!(feed.items.len(), 1);
    let desc = feed.items[0].description.as_str();
    assert!(desc.contains("Hello"));
}

#[test]
fn parses_feed_with_cdata_split_across_chunks() {
    let mut parser = RssParser::new();
    parser.push("<?xml version=\"1.0\"?>\n<rss version=\"2.0\">\n  <channel>\n    <item>\n      <title>T</title>\n      <description><![CDATA[<p>Raw");
    parser.push(" text</p>]]></description>\n    </item>\n  </channel>\n</rss>");
    let feed = parser.finish().expect("should parse");
    assert_eq!(feed.items.len(), 1);
    let desc = feed.items[0].description.as_str();
    assert!(desc.contains("Raw text"));
}

#[test]
fn streaming_parser_rejects_empty() {
    let mut parser = RssParser::new();
    parser.push("");
    let result = parser.finish();
    assert!(matches!(result, Err(ParseError::Malformed)));
}

#[test]
fn streaming_parser_rejects_non_rss() {
    let mut parser = RssParser::new();
    parser.push("<root>not rss</root>");
    let result = parser.finish();
    assert!(matches!(result, Err(ParseError::Malformed)));
}
