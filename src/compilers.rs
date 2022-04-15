#[path = "rcdom.rs"]
mod rcdom;
use rcdom::{RcDom, SerializableHandle,Node,NodeData};

use html5ever::tendril::TendrilSink;
use html5ever::driver::ParseOpts;
use html5ever::LocalName;

use tendril::StrTendril;
use if_chain::if_chain;

use std::convert::TryFrom;
use std::default::Default;
use std::cell::RefCell;
use std::rc::Rc;

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


#[allow(dead_code)]
pub fn compile_html(text: &str, options: CompileOptions) -> Option<String> {
    let dom = html5ever::parse_document(RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(&mut text.as_bytes())
        .unwrap();

    let mut queue: Vec<std::rc::Rc<rcdom::Node>> = vec![dom.document.clone()];
    
     while let Some(node) = queue.pop() {
            if_chain! {
                if let NodeData::Element { ref name, ref attrs, .. } = node.data;
                if name.local == LocalName::from("script");
                then {
                    let mut attrs = attrs.borrow_mut();
                    if let Some(attr) = attrs.iter_mut().find(|attr| attr.name.local == LocalName::from("type")) {

                        fn get_text_content(node: &Rc<Node>) -> String {
                            let mut text_content = String::from("");
                            for child in node.children.borrow().iter() {
                                if let NodeData::Text { ref contents } = child.data {
                                    text_content.push_str(contents.borrow().as_ref());
                                }
                            }
                            return text_content;
                        }

                        fn set_text_content(node: &Rc<Node>, text_content: String) {
                            node.children.borrow_mut().clear();
                            node.children.borrow_mut().push(Node::new(NodeData::Text { contents:  RefCell::new(StrTendril::from(text_content)) }));
                        }

                        match attr.value.as_ref() {
                            value @ ("text/typescript" | "application/typescript") => {

                                let value = StrTendril::from(value);
                                attrs.retain(|it| it.name.local != LocalName::from("type") && it.value == value);

                                let mut options = options.clone();
                                options.module = "none".to_string();

                                set_text_content(&node, compile_typescript(get_text_content(&node).as_ref(), options)?);
                            },
                            "tsmodule" | "module/typescript" => {
                                attr.value = StrTendril::from("module");
                                set_text_content(&node, compile_typescript(get_text_content(&node).as_ref(), options.clone())?);
                            },
                            _ => {},
                        }
                    }
                } else {
                     queue.extend(node.children.borrow().clone());
                }
            }
     }

    let mut buffer = std::io::BufWriter::new(vec![]);     
    html5ever::serialize(&mut buffer, &SerializableHandle::from(dom.document.clone()), Default::default()).ok()?;

    return String::from_utf8(buffer.into_parts().1.ok()?).ok();
}