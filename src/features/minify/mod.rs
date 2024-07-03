// Minify Feature
use super::common::{self,Options};

use std::convert::TryFrom;
use std::ops::Deref;

static TERSER: &str = include_str!(r"terser.js");

fn format_ecma_version_string<S: Deref<Target = str>>(target: S) -> String {
    String::from(match target.to_lowercase().as_str() {
        "esnext" => "2023",
        v @ _ => v.strip_prefix("es").unwrap_or("esnext")
    })
}

pub fn minify(text: String, options: &Options) -> Option<String> {

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

    macro_rules! v8_bool {
        ($expression:expr) => {
            v8::Boolean::new(scope, $expression).into()
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

    let script = v8_str!(TERSER);
    let script = v8::Script::compile(scope, script, None)?;
    script.run(scope)?;

    let global_this = context.global(scope);
    let terser = v8_get!(global_this.Terser)?.to_object(scope)?;
    let minify = v8::Local::<v8::Function>::try_from(v8_get!(terser.minify)?.to_object(scope)?).ok()?;

    let text = v8_str!(text.as_str());

    // Global Options
    let args = v8::Object::new(scope);
    v8_set!(args.module = v8_bool!(options.module));


    // Compress Options
    let compress = v8::Object::new(scope);
    v8_set!(compress.ecma = v8_str!(&format_ecma_version_string(options.target.clone())));
    v8_set!(compress.keep_classnames = v8_bool!(true));

    v8_set!(args.compress = compress.into());

    // Mangle Options
    let mangle = v8::Object::new(scope);
    v8_set!(mangle.keep_classnames = v8_bool!(true));

    v8_set!(args.mangle = mangle.into());

    // Format Options
    let format = v8::Object::new(scope);
    v8_set!(format.ecma = v8_str!(&format_ecma_version_string(options.target.clone())));
    v8_set!(format.comments = v8_str!("/^!/"));

    v8_set!(args.format = format.into());

    let result = minify.call(scope, terser.into(), &[text, args.into()])?;

    if result.is_promise() {
        let promise = v8::Local::<v8::Promise>::try_from(result).ok()?;

        while promise.state() == v8::PromiseState::Pending {
            scope.perform_microtask_checkpoint();
        }
        if promise.state() == v8::PromiseState::Rejected {
            panic!("Promise rejected");
        } else {
            let resolved = promise.result(scope).to_object(scope)?;
            return Some(v8_get!(resolved.code)?.to_string(scope)?.to_rust_string_lossy(scope));
        }
    } else {
        panic!("Value is not a promise");
    }
}