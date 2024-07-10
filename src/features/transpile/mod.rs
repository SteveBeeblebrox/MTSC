
// Transpile Feature
use super::common::{with_v8,include_script,SHARED_RUNTIME};
use crate::Options;

use std::convert::TryFrom;

pub fn transpile(text: String, options: &Options) -> Option<String> {
    include_script!(SHARED_RUNTIME, r"typescript.js");

    return with_v8! {
        use runtime(scope,context) = SHARED_RUNTIME;
        
        let global_this = global_this!();
        let ts = v8_get!(global_this.ts)?.to_object(scope)?;
        let transpile = v8::Local::<v8::Function>::try_from(v8_get!(ts.transpile)?.to_object(scope)?).ok()?;
    
        let text = v8_str!(text.as_str());
    
        let args: v8::Local<v8::Object> = v8_object!({
            target: v8_str!(options.target.as_str()),
            module: v8_str!("esnext")
        });
    
        if options.use_jsx {
            v8_set!(args.jsx = v8_str!(if options.jsx_factory.is_some() {"react"} else {"preserve"}));
            
            if options.jsx_factory.is_some() {
                v8_set!(args.jsxFactory = v8_str!(options.jsx_factory.clone().unwrap().as_str()));
                v8_set!(args.jsxFragmentFactory = v8_str!(options.jsx_fragment.clone().unwrap().as_str()));
            }
        }
    
        return Some(transpile.call(scope, ts.into(), &[text, args.into()])?.to_rust_string_lossy(scope))
    }
}