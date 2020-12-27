extern crate gemtext;
use cursive::utils::lines::simple::{make_lines, LinesIterator};
use url::Url;
// https://gemini.circumlunar.space/docs/spec-spec.txt

#[derive(Clone, Debug, PartialEq)]
pub enum GeminiType {
    Text,
    Gemini,
}

pub fn parse(text: &str, base_url: &Url, viewport_width: usize) -> Vec<(String, Option<Url>)> {
    let mut nodes = gemtext::parse(text);
    nodes
        .drain(..)
        .map(|node: gemtext::Node| -> Vec<(String, Option<Url>)> {
            use gemtext::Node;

            // Helper function to wrap lines if necessary while indicating that they are continuations like this
            // ```text
            //   ###  Heading that
            //     |  goes over
            //     \  multiple lines
            // ```
            let continuation_lines = |first_prefix, text: &str, url: Option<Url>| {
                let lines = make_lines(if text.is_empty() { " " } else { &text }, viewport_width);
                lines
                    .iter()
                    .enumerate()
                    .map(|(i, row)| {
                        let prefix = match i {
                            0 => first_prefix,
                            x if x == lines.len() - 1 => "\\",
                            _ => "|",
                        };

                        (
                            format!("{:>5}  {}", prefix, &text[row.start..row.end]),
                            url.clone(),
                        )
                    })
                    .collect()
            };

            match node {
                Node::Text(text) => {
                    let text = if text.is_empty() { " " } else { &text };
                    LinesIterator::new(text, viewport_width)
                        .map(|row| (format!("       {}", &text[row.start..row.end]), None))
                        .collect()
                }
                Node::Link { to, name } => {
                    let url = base_url.join(&to).expect("could not parse link url");
                    let prefix = match url.scheme() {
                        "https" | "http" => "[WWW]".to_string(),
                        "gemini" => "[GEM]".to_string(),
                        "gopher" => "[GPH]".to_string(),
                        "mailto" => "[ \u{2709} ]".to_string(),
                        // show first three letters of scheme, lower case to differentiate
                        other => format!("[{}]", other.chars().take(3).collect::<String>()),
                    };

                    let name = name.unwrap_or_else(|| url.to_string());
                    continuation_lines(&prefix, &name, Some(url))
                }
                Node::Heading { level, body } => {
                    let text = if body.is_empty() { " " } else { &body };
                    continuation_lines(&"#".repeat(level as usize), &text, None)
                }
                Node::Quote(text) => {
                    let text = if text.is_empty() { " " } else { &text };
                    LinesIterator::new(text, viewport_width)
                        .map(|row| (format!("    >  {}", &text[row.start..row.end]), None))
                        .collect()
                }
                Node::ListItem(text) => continuation_lines("*", &text, None),
                Node::Preformatted(lines) => {
                    // preformatted lines should not be wrapped
                    lines
                        .lines()
                        .map(|line| (format!("    @  {}", line), None))
                        .collect()
                }
            }
        })
        .flatten()
        .collect::<Vec<_>>()
}
