use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{
    CharacterTokens, EndTag, NullCharacterToken, StartTag, TagToken, DoctypeToken, CommentToken, EOFToken,
    ParseError, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts, BufferQueue, Tag
};

use crate::wave;
use wave::{preprocess_text};

use std::convert::TryFrom;
use std::default::Default;
use std::sync::Once;

use v8;

static TYPESCRIPT: &str = include_str!(r"typescript.js");
static TERSER: &str = include_str!(r"terser.js");

#[derive(Clone)]
pub struct CompileOptions {
    pub target: String,
    pub module: String,
    pub use_jsx: bool,
    pub jsx_factory: Option<String>,
    pub jsx_fragment: Option<String>,

    pub use_preprocessor: bool,
    pub macros: Vec<String>,
    pub filename: Option<String>,
    pub include_paths: Vec<String>
}

#[derive(Clone)]
pub struct MinifyOptions {
    pub target: String,
    pub module: String
}

impl From<CompileOptions> for MinifyOptions {
    fn from(options: CompileOptions) -> Self {
        MinifyOptions {
            target: options.target,
            module: options.module
        }
    }
}

static V8_INIT: Once = Once::new();

#[allow(dead_code)]
pub fn compile_typescript(text: &str, options: CompileOptions) -> Option<String> {
    let text: String = if options.use_preprocessor {
        preprocess_text(String::from(text), options.filename.unwrap_or(String::from("-")), options.macros, options.include_paths).expect("error running preprocessor")
    } else {
        text.to_string()
    };

    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
    
    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    let ts_compiler = v8::String::new(scope, TYPESCRIPT)?;
    
    let script = v8::Script::compile(scope, ts_compiler, None)?;
    script.run(scope)?;

    let ts_obj_name = v8::String::new(scope, "ts")?.into();
    let ts_obj = context.global(scope).get(scope, ts_obj_name)?;
    
    let transpile_func_name = v8::String::new(scope, "transpile")?.into();
    let transpile_function = ts_obj.to_object(scope)?.get(scope, transpile_func_name)?.to_object(scope)?;
    let transpile_function = v8::Local::<v8::Function>::try_from(transpile_function).ok()?;

    let text = v8::String::new(scope, text.as_str())?.into();

    let args = v8::Object::new(scope);

    let target_prop_name = v8::String::new(scope, "target")?.into();
    let target_prop_value = v8::String::new(scope, options.target.as_str())?.into();
    args.set(scope, target_prop_name, target_prop_value);

    let module_prop_name = v8::String::new(scope, "module")?.into();
    let module_prop_value = v8::String::new(scope, options.module.as_str())?.into();
    args.set(scope, module_prop_name, module_prop_value);

    if options.use_jsx {
        let jsx_factory_prop_name = v8::String::new(scope, "jsx")?.into();
        let jsx_factory_prop_value = v8::String::new(scope, if options.jsx_factory.is_some() {"react"} else {"preserve"})?.into();
        args.set(scope, jsx_factory_prop_name, jsx_factory_prop_value);
        
        if options.jsx_factory.is_some() {
             let jsx_factory_prop_name = v8::String::new(scope, "jsxFactory")?.into();
            let jsx_factory_prop_value = v8::String::new(scope, options.jsx_factory.unwrap().as_str())?.into();
            args.set(scope, jsx_factory_prop_name, jsx_factory_prop_value);

            let jsx_fragment_prop_name = v8::String::new(scope, "jsxFragmentFactory")?.into();
            let jsx_fragment_prop_value = v8::String::new(scope, options.jsx_fragment.unwrap().as_str())?.into();
            args.set(scope, jsx_fragment_prop_name, jsx_fragment_prop_value);
        }
    }

    return Some(transpile_function.call(scope, ts_obj, &[text, args.into()])?.to_string(scope)?
        .to_rust_string_lossy(scope))
}


#[derive(PartialEq)]
enum TargetType {
    None, Classic, Module
}

struct Document {
    options: CompileOptions,
    typescript_mode: TargetType,
    inner_html: String,
    script_buffer: String
}

impl Document {
    fn write_text<S: AsRef<str>>(&mut self, html: S) {
        if self.typescript_mode == TargetType::None {
            self.inner_html.push_str(html.as_ref());
        } else {
            self.script_buffer.push_str(html.as_ref());
        }
    }
    fn new(options: CompileOptions) -> Self {
        Document {
            options,
            typescript_mode: TargetType::None,
            inner_html: String::new(),
            script_buffer: String::new()
        }
    }
}

impl TokenSink for &mut Document {
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
                                    "text/typescript" | "application/typescript" => {
                                        tag.attrs.retain(|attr| attr.name.local.as_ref() != "type");
                                        self.write_text(get_tag_str(tag));
                                        self.typescript_mode = TargetType::Classic
                                    },
                                    "module/typescript" | "tsmodule" => {
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
                                self.typescript_mode = TargetType::None;
                                
                                let mut options = self.options.clone();
                                
                                if self.typescript_mode != TargetType::Module {
                                    options.module = "none".to_string();
                                }

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

                                self.write_text(format!("\n{}",
                                    compile_typescript(
                                            &script_buffer.lines().map(|line| line.strip_prefix(indentation.as_str()).unwrap_or(line).to_string()).collect::<Vec<String>>().join("\n"),
                                            options
                                        ).expect("error compiling TypeScript within HTML")
                                    .lines().map(|line| format!("{}{}", indentation, line)).collect::<Vec<String>>().join("\n")
                                ));
                                
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

#[allow(dead_code)]
pub fn compile_html(text: &str, options: CompileOptions) -> Option<String> {
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

#[allow(dead_code)]
pub fn minify_javascript(text: &str, options: MinifyOptions) -> Option<String> {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
    
    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    let terser = v8::String::new(scope, TERSER)?;
    
    let script = v8::Script::compile(scope, terser, None)?;
    script.run(scope)?;

    let terser_obj_name = v8::String::new(scope, "Terser")?.into();
    let terser_obj = context.global(scope).get(scope, terser_obj_name)?;
    
    let minify_func_name = v8::String::new(scope, "minify")?.into();
    let minify_function = terser_obj.to_object(scope)?.get(scope, minify_func_name)?.to_object(scope)?;
    let minify_function = v8::Local::<v8::Function>::try_from(minify_function).ok()?;

    let text = v8::String::new(scope, text)?.into();

    let args = v8::Object::new(scope);

    // Global Options
    let module_prop_name = v8::String::new(scope, "module")?.into();
    let module_prop_value = v8::Boolean::new(scope, options.module != "none").into();
    args.set(scope, module_prop_name, module_prop_value);

    // Compress Options
    let compress_prop_name = v8::String::new(scope, "compress")?.into();
    let compress_prop_value = v8::Object::new(scope);

    let ecma_prop_name = v8::String::new(scope, "ecma")?.into();
    let ecma_prop_value = v8::String::new(scope, options.target.to_lowercase().strip_prefix("es")?)?.into();
    compress_prop_value.set(scope, ecma_prop_name, ecma_prop_value);

    let keep_classnames_prop_name = v8::String::new(scope, "keep_classnames")?.into();
    let keep_classnames_prop_value = v8::Boolean::new(scope, true).into();
    compress_prop_value.set(scope, keep_classnames_prop_name, keep_classnames_prop_value);

    args.set(scope, compress_prop_name, compress_prop_value.into());
    
    // Mangle Options
    let mangle_prop_name = v8::String::new(scope, "mangle")?.into();
    let mangle_prop_value = v8::Object::new(scope);

    let keep_classnames_prop_name = v8::String::new(scope, "keep_classnames")?.into();
    let keep_classnames_prop_value = v8::Boolean::new(scope, true).into();
    mangle_prop_value.set(scope, keep_classnames_prop_name, keep_classnames_prop_value);

    args.set(scope, mangle_prop_name, mangle_prop_value.into());

    // Format Options
    let format_prop_name = v8::String::new(scope, "format")?.into();
    let format_prop_value = v8::Object::new(scope);

    let ecma_prop_name = v8::String::new(scope, "ecma")?.into();
    let ecma_prop_value = v8::String::new(scope, options.target.to_lowercase().strip_prefix("es")?)?.into();
    format_prop_value.set(scope, ecma_prop_name, ecma_prop_value);

    let comment_prop_name = v8::String::new(scope, "comments")?.into();
    let comment_prop_value = v8::String::new(scope, "/^!/")?.into();
    format_prop_value.set(scope, comment_prop_name, comment_prop_value);

    args.set(scope, format_prop_name, format_prop_value.into());

    let result = minify_function.call(scope, terser_obj, &[text, args.into()])?;

    if result.is_promise() {
        let promise = v8::Local::<v8::Promise>::try_from(result).ok()?;

        while promise.state() == v8::PromiseState::Pending {
            scope.perform_microtask_checkpoint();
        }
        if promise.state() == v8::PromiseState::Rejected {
            panic!("Promise rejected");
        } else {
            let code_name = v8::String::new(scope, "code")?.into();
            let resolved = promise.result(scope).to_object(scope)?.get(scope, code_name)?;
            return Some(resolved.to_string(scope)?.to_rust_string_lossy(scope))
        }
    } else {
        panic!("Value is not a promise");
    }
}
