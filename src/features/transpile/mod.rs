// Transpile Feature
use super::common::{self,Options};

use std::convert::TryFrom;

static TYPESCRIPT: &str = include_str!(r"typescript.js");

pub fn transpile(text: String, options: &Options) -> Option<String> {
    
    common::init();
    
    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    macro_rules! v8_str {
        ($expression:expr) => {
            v8::String::new(scope, $expression)?.into()
        };
    }

    macro_rules! v8_set {
        ($obj:ident . $prop:ident = $value:expr) => {
            {
                let prop_name = v8_str!(stringify!($prop));
                let prop_value = $value; 
                $obj.set(scope,prop_name,prop_value)
            }
        };
    }

    macro_rules! v8_get {
        ($obj:ident . $prop:ident) => {
            {
                let prop_name = v8_str!(stringify!($prop));
                $obj.get(scope,prop_name)
            }
        };
    }

    let script = v8_str!(TYPESCRIPT);
    let script = v8::Script::compile(scope, script, None)?;
    script.run(scope)?;

    let global_this = context.global(scope);
    let ts = v8_get!(global_this.ts)?.to_object(scope)?;
    let transpile = v8::Local::<v8::Function>::try_from(v8_get!(ts.transpile)?.to_object(scope)?).ok()?;

    let text = v8_str!(text.as_str());

    let args = v8::Object::new(scope);
    v8_set!(args.target = v8_str!(options.target.as_str()));
    v8_set!(args.module = v8_str!("esnext"));

    if options.use_jsx {
        v8_set!(args.jsx = v8_str!(if options.jsx_factory.is_some() {"react"} else {"preserve"}));
        
        if options.jsx_factory.is_some() {
            v8_set!(args.jsxFactory = v8_str!(options.jsx_factory.clone().unwrap().as_str()));
            v8_set!(args.jsxFragmentFactory = v8_str!(options.jsx_fragment.clone().unwrap().as_str()));
        }
    }

    return Some(transpile.call(scope, ts.into(), &[text, args.into()])?.to_string(scope)?
        .to_rust_string_lossy(scope))
}