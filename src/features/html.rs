// HTML Feature
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{
    CharacterTokens, EndTag, NullCharacterToken, StartTag, TagToken, DoctypeToken, CommentToken, EOFToken,
    ParseError, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts, BufferQueue, Tag
};

use super::common::Options;

#[derive(PartialEq)]
enum TargetType {
    None, Classic, Module
}

struct Document<'a> {
    options: &'a Options,
    typescript_mode: TargetType,
    inner_html: String,
    script_buffer: String
}

impl<'a> Document<'a> {
    fn write_text<S: AsRef<str>>(&mut self, html: S) {
        if self.typescript_mode == TargetType::None {
            self.inner_html.push_str(html.as_ref());
        } else {
            self.script_buffer.push_str(html.as_ref());
        }
    }
    fn new(options: &'a Options) -> Self {
        Document {
            options,
            typescript_mode: TargetType::None,
            inner_html: String::new(),
            script_buffer: String::new()
        }
    }
}

impl<'a> TokenSink for &mut Document<'a> {
    type Handle = ();
    fn adjusted_current_node_present_but_not_in_html_namespace(&self) -> bool {
        true
    }
    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        match token {
            CharacterTokens(str_tendril) => {
                self.write_text(str_tendril);
            },
            DoctypeToken(doctype) => {
                self.write_text("<!DOCTYPE ");
                if let Some(name) = doctype.name {
                    self.write_text(name.as_ref());
                }
                if let Some(public_id) = doctype.public_id {
                    self.write_text(format!(" PUBLIC \"{}\"", public_id));
                }
                if let Some(system_id) = doctype.system_id {
                    self.write_text(format!(" \"{}\"", system_id));
                }
                self.write_text(">");
            },
            TagToken(mut tag) => {
                fn get_tag_str(tag: Tag) -> String {
                    let mut attrs = String::new();
                    
                    for attr in tag.attrs.iter() {
                        if attr.value.len() > 0 {
                            let value = attr.value.to_string();
                            
                            if value.contains("\"") && !value.contains("'") {
                                attrs.push_str(&format!(" {}='{}'", attr.name.local, value));
                            } else if value.contains("'") && !value.contains("\"") {
                                attrs.push_str(&format!(r#" {}="{}""#, attr.name.local, value));
                            }
                            else {
                                attrs.push_str(&format!(r#" {}="{}""#, attr.name.local, attr.value.as_ref().replace("\"", "&quot;")));
                            }
                        } else {
                            attrs.push_str(&format!(" {}", attr.name.local));
                        }
                    }
                    
                    return match tag.kind {
                        _ if tag.self_closing => format!("<{}{}/>", tag.name, attrs),
                        StartTag => format!("<{}{}>", tag.name, attrs),
                        EndTag => format!("</{}>", tag.name),
                    }
                }

                if tag.name.to_lowercase() == "script" {
                    match tag.kind {
                        StartTag => {
                            if let Some(attr) = tag.attrs.iter_mut().find(|attr| attr.name.local.as_ref() == "type") {
                                match attr.value.as_ref() {
                                    "text/typescript" => {
                                        tag.attrs.retain(|attr| attr.name.local.as_ref() != "type");
                                        self.write_text(get_tag_str(tag));
                                        self.typescript_mode = TargetType::Classic
                                    },
                                    "tsmodule" => {
                                        attr.value = StrTendril::from("module");
                                        self.write_text(get_tag_str(tag));
                                        self.typescript_mode = TargetType::Module
                                    },
                                    _ => self.write_text(get_tag_str(tag))
                                }
                            } else {
                                self.write_text(get_tag_str(tag));
                            }
                            return TokenSinkResult::RawData(html5ever::tokenizer::states::RawKind::ScriptData);
                        },
                        EndTag => {
                            if self.typescript_mode != TargetType::None {
                                let mut options = self.options.clone();
                                
                                options.module = self.typescript_mode == TargetType::Module;
                                self.typescript_mode = TargetType::None;

                                let script_buffer = self.script_buffer.clone();
                                
                                let mut lines: Vec<&str> = script_buffer.lines().collect::<Vec<&str>>();
                                lines.retain(|line| !line.trim().is_empty());
                                let mut indentation = String::new();

                                if lines.len() > 0 {
                                    for i in 0..lines[0].len() {
                                        if let Some(char) = lines[0].chars().nth(i) {
                                            if char.is_whitespace() && lines.iter().all(move |line| line.chars().nth(i) == Some(char)) {
                                                indentation.push(char);
                                            } else {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                }

                                let mut text = crate::compile_script(
                                        &script_buffer.lines().map(|line| line.strip_prefix(indentation.as_str()).unwrap_or(line).to_string()).collect::<Vec<String>>().join("\n"),
                                        &options
                                    ).expect("error compiling TypeScript within HTML")
                                    .lines().map(|line| format!("{}{}", indentation, line)).collect::<Vec<String>>().join("\n")
                                ;

                                #[cfg(feature = "minify")]
                                if options.minify {
                                    text = super::minify(text,&options).expect("error minifying within HTML").lines().map(|line| format!("{}{}", indentation, line)).collect::<Vec<String>>().join("\n");
                                }

                                self.write_text(format!("\n{}",text));
                                
                                let last = script_buffer.lines().last().unwrap_or("");
                                if last.chars().all(|char| char.is_whitespace()) {
                                    self.write_text(format!("\n{}", last));
                                }

                                self.script_buffer = String::new();
                            }
                            self.write_text(get_tag_str(tag));
                            return TokenSinkResult::Continue
                        }
                    }
                }
                else {
                    self.write_text(get_tag_str(tag));
                }
            },
            CommentToken(comment) => {
                self.write_text(format!("<!--{}-->", comment));
            },
            NullCharacterToken => self.write_text("\0"),
            ParseError(_error) => (),
            EOFToken => ()
        }
        TokenSinkResult::Continue
    }
}

pub fn compile_html(text: String, options: &Options) -> Option<String> {
    let mut document = Document::new(options);
    
    let mut input = BufferQueue::new();
    input.push_back(StrTendril::from(text));

    let mut tokenizer = Tokenizer::new(&mut document, TokenizerOpts {
        ..Default::default()
    });

    let _ = tokenizer.feed(&mut input);
    tokenizer.end();

    return Some(document.inner_html);
}