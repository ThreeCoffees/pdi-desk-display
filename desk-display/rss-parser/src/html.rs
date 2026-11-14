use heapless::String;

use crate::entity::decode_html_entity;

pub struct HtmlToText {
    state: ParseState,
    last_was_newline: bool,
}

enum TagState {
    Name,
    Attributes,
}

enum ParseState {
    Text,
    Tag {
        state: TagState,
        name: String<10>,
        overflow: bool,
    },
    Entity(String<12>),
}

impl HtmlToText {
    pub const fn new() -> Self {
        Self {
            state: ParseState::Text,
            last_was_newline: false,
        }
    }

    pub fn push<const N: usize>(&mut self, input: &str, output: &mut String<N>) {
        for ch in input.chars() {
            let state = core::mem::replace(&mut self.state, ParseState::Text);
            self.state = self.process_char(state, ch, output);
        }
    }

    pub fn finish<const N: usize>(&mut self, output: &mut String<N>) {
        let state = core::mem::replace(&mut self.state, ParseState::Text);
        self.finish_state(state, output);
    }

    fn process_char<const N: usize>(
        &mut self,
        state: ParseState,
        ch: char,
        output: &mut String<N>,
    ) -> ParseState {
        match state {
            ParseState::Text => self.process_text(ch, output),
            ParseState::Tag {
                state,
                name,
                overflow,
            } => self.process_tag(state, name, overflow, ch, output),
            ParseState::Entity(entity) => self.process_entity(entity, ch, output),
        }
    }

    fn process_text<const N: usize>(&mut self, ch: char, output: &mut String<N>) -> ParseState {
        match ch {
            '<' => Self::new_tag(),
            '&' => ParseState::Entity(String::new()),
            _ => {
                self.emit_char(ch, output);
                ParseState::Text
            }
        }
    }

    fn process_tag<const N: usize>(
        &mut self,
        state: TagState,
        mut name: String<10>,
        mut overflow: bool,
        ch: char,
        output: &mut String<N>,
    ) -> ParseState {
        if ch == '>' {
            if !overflow && is_block_tag(name.as_str()) {
                self.emit_newline(output);
            }
            return ParseState::Text;
        }

        let state = match state {
            TagState::Name if ch.is_ascii_whitespace() => TagState::Attributes,
            TagState::Name if ch == '/' => TagState::Name,
            TagState::Name => {
                if name.push(ch.to_ascii_lowercase()).is_err() {
                    overflow = true;
                }
                TagState::Name
            }
            TagState::Attributes => TagState::Attributes,
        };

        ParseState::Tag {
            state,
            name,
            overflow,
        }
    }

    fn process_entity<const N: usize>(
        &mut self,
        mut entity: String<12>,
        ch: char,
        output: &mut String<N>,
    ) -> ParseState {
        match ch {
            ';' => {
                self.emit_entity(&entity, true, output);
                ParseState::Text
            }
            '&' => {
                self.emit_entity(&entity, false, output);
                ParseState::Entity(String::new())
            }
            '<' => {
                self.emit_entity(&entity, false, output);
                Self::new_tag()
            }
            ch if ch.is_ascii_whitespace() => {
                self.emit_entity(&entity, false, output);
                self.emit_char(ch, output);
                ParseState::Text
            }
            _ if entity.push(ch).is_ok() => ParseState::Entity(entity),
            _ => {
                self.emit_entity(&entity, false, output);
                self.emit_char(ch, output);
                ParseState::Text
            }
        }
    }

    fn emit_entity<const N: usize>(
        &mut self,
        entity: &String<12>,
        terminated: bool,
        output: &mut String<N>,
    ) {
        if terminated && let Some(ch) = decode_html_entity(entity.as_str()) {
            self.emit_char(ch, output);
            return;
        }

        self.emit_char('&', output);
        for ch in entity.chars() {
            self.emit_char(ch, output);
        }
        if terminated {
            self.emit_char(';', output);
        }
    }

    fn finish_state<const N: usize>(&mut self, state: ParseState, output: &mut String<N>) {
        if let ParseState::Entity(entity) = state {
            self.emit_entity(&entity, false, output);
        }
    }

    fn new_tag() -> ParseState {
        ParseState::Tag {
            state: TagState::Name,
            name: String::new(),
            overflow: false,
        }
    }

    fn emit_char<const N: usize>(&mut self, ch: char, output: &mut String<N>) {
        if output.push(ch).is_ok() {
            self.last_was_newline = ch == '\n';
        }
    }

    fn emit_newline<const N: usize>(&mut self, output: &mut String<N>) {
        if !self.last_was_newline {
            self.emit_char('\n', output);
        }
    }
}

impl Default for HtmlToText {
    fn default() -> Self {
        Self::new()
    }
}

fn is_block_tag(name: &str) -> bool {
    matches!(
        name,
        "br" | "p"
            | "div"
            | "li"
            | "tr"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "blockquote"
            | "pre"
            | "hr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::String;

    fn convert(input: &str) -> String<512> {
        let mut filter = HtmlToText::new();
        let mut output = String::new();

        filter.push(input, &mut output);
        filter.finish(&mut output);

        output
    }

    #[test]
    fn strips_inline_tags() {
        assert_eq!(convert("Hello <b>world</b>!").as_str(), "Hello world!");
    }

    #[test]
    fn converts_block_tags_to_newlines() {
        assert_eq!(
            convert("<p>First</p><p>Second</p>").as_str(),
            "\nFirst\nSecond\n"
        );
    }

    #[test]
    fn converts_br_tags_to_newlines() {
        assert_eq!(
            convert("First<br>Second<br/>Third").as_str(),
            "First\nSecond\nThird"
        );
    }

    #[test]
    fn handles_attributes_and_uppercase_tags() {
        assert_eq!(
            convert(r#"<DIV class="news"><A href="/story">Story</A></DIV>"#).as_str(),
            "\nStory\n"
        );
    }

    #[test]
    fn decodes_named_entities() {
        assert_eq!(
            convert("&amp; &lt; &gt; &quot; &apos; &nbsp; &copy;").as_str(),
            "& < > \" '   ©"
        );
    }

    #[test]
    fn decodes_numeric_entities() {
        assert_eq!(convert("&#65; &#x42; &#x1F600;").as_str(), "A B 😀");
    }

    #[test]
    fn preserves_unknown_entities() {
        assert_eq!(convert("A &unknown; B").as_str(), "A &unknown; B");
    }

    #[test]
    fn preserves_utf8_text() {
        assert_eq!(
            convert("Zażółć gęślą jaźń — café").as_str(),
            "Zażółć gęślą jaźń — café"
        );
    }

    #[test]
    fn handles_entities_split_across_chunks() {
        let mut filter = HtmlToText::new();
        let mut output = String::<128>::new();

        filter.push("Fish &am", &mut output);
        filter.push("p; chips", &mut output);
        filter.finish(&mut output);

        assert_eq!(output.as_str(), "Fish & chips");
    }

    #[test]
    fn handles_tags_split_across_chunks() {
        let mut filter = HtmlToText::new();
        let mut output = String::<128>::new();

        filter.push("First<di", &mut output);
        filter.push("v>Second</div>", &mut output);
        filter.finish(&mut output);

        assert_eq!(output.as_str(), "First\nSecond\n");
    }

    #[test]
    fn handles_utf8_split_between_chunks() {
        let mut filter = HtmlToText::new();
        let mut output = String::<128>::new();

        filter.push("Café ", &mut output);
        filter.push("東京", &mut output);
        filter.finish(&mut output);

        assert_eq!(output.as_str(), "Café 東京");
    }

    #[test]
    fn truncates_output_but_continues_processing() {
        let mut filter = HtmlToText::new();
        let mut output = String::<5>::new();

        filter.push("123456<b>789</b>", &mut output);
        filter.finish(&mut output);

        assert_eq!(output.as_str(), "12345");
    }

    #[test]
    fn collapses_consecutive_block_newlines() {
        assert_eq!(convert("<p>A</p><p></p><div>B</div>").as_str(), "\nA\nB\n");
    }

    #[test]
    fn emits_incomplete_entity_on_finish() {
        let mut filter = HtmlToText::new();
        let mut output = String::<64>::new();

        filter.push("Value: &amp", &mut output);
        filter.finish(&mut output);

        assert_eq!(output.as_str(), "Value: &amp");
    }

    #[test]
    fn ignores_incomplete_tag_on_finish() {
        let mut filter = HtmlToText::new();
        let mut output = String::<64>::new();

        filter.push("Before <b", &mut output);
        filter.finish(&mut output);

        assert_eq!(output.as_str(), "Before ");
    }
}
