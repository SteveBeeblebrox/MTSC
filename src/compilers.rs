
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{
    CharacterTokens, EndTag, NullCharacterToken, StartTag, TagToken, DoctypeToken, CommentToken, EOFToken,
    ParseError, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts, BufferQueue
};

use std::convert::TryFrom;
use std::default::Default;

use v8;

static TYPESCRIPT_SERVICES: &str = include_str!(r"typescriptServices.js");

#[derive(Clone)]
pub struct CompileOptions {
    pub target: String,
    pub module: String,
    pub use_jsx: bool,
    pub jsx_factory: Option<String>,
    pub jsx_fragment: Option<String>
}

#[allow(dead_code)]
pub fn compile_typescript(text: &str, options: CompileOptions) -> Option<String> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
    
    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    let ts_compiler = v8::String::new(scope, TYPESCRIPT_SERVICES)?;
    
    let script = v8::Script::compile(scope, ts_compiler, None)?;
    script.run(scope)?;

    let ts_obj_name = v8::String::new(scope, "ts")?.into();
    let ts_obj = context.global(scope).get(scope, ts_obj_name)?;
    
    let transpile_func_name = v8::String::new(scope, "transpile")?.into();
    let transpile_function = ts_obj.to_object(scope)?.get(scope, transpile_func_name)?.to_object(scope)?;
    let transpile_function = v8::Local::<v8::Function>::try_from(transpile_function).ok()?;

    let text = v8::String::new(scope, text)?.into();

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
                let mut attrs = String::new();
                    
                for attr in tag.attrs.iter() {
                    if attr.value.len() > 0 {
                        attrs.push_str(&format!(" {}=\"{}\"", attr.name.local, attr.value));
                    } else {
                        attrs.push_str(&format!(" {}", attr.name.local));
                    }
                }
                
                let tag_str = match tag.kind {
                    _ if tag.self_closing => format!("<{}{}/>", tag.name, attrs),
                    StartTag => format!("<{}{}>", tag.name, attrs),
                    EndTag => format!("</{}>", tag.name),
                };

                if tag.name.to_lowercase() == "script" {
                    match tag.kind {
                        StartTag => {
                            if let Some(attr) = tag.attrs.iter_mut().find(|attr| attr.name.local.as_ref() == "type") {
                                match attr.value.as_ref() {
                                    "text/typescript" | "application/typescript" => {
                                        tag.attrs.retain(|attr| attr.name.local.as_ref() != "type");
                                        self.write_text(tag_str.clone());
                                        self.typescript_mode = TargetType::Classic
                                    },
                                    "module/typescript" | "tsmodule" => {
                                        attr.value = StrTendril::from("module");
                                        self.write_text(tag_str.clone());
                                        self.typescript_mode = TargetType::Module
                                    },
                                    _ => self.write_text(tag_str.clone())
                                }
                            } else {
                                self.write_text(tag_str.clone());
                            }
                        },
                        EndTag => {
                            if self.typescript_mode != TargetType::None {
                                self.typescript_mode = TargetType::None;
                                
                                let mut options = self.options.clone();
                                
                                if self.typescript_mode != TargetType::Module {
                                    options.module = "none".to_string();
                                }

                                self.write_text("\n");
                                let script_buffer = self.script_buffer.clone();
                                self.write_text(compile_typescript(&script_buffer, options).expect("Error compiling TypeScript within HTML"));
                                self.write_text(script_buffer.lines().last().unwrap_or(""));
                                self.script_buffer = String::new();
                            }
                            self.write_text(tag_str.clone());
                        }
                    }
                }
                else {
                    self.write_text(tag_str);
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