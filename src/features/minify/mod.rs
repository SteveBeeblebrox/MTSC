// Minify Feature
use super::common::{with_v8,include_script,SHARED_RUNTIME};
use crate::Options;

use std::convert::TryFrom;
use std::ops::Deref;

fn format_ecma_version_string<S: Deref<Target = str>>(target: S) -> String {
    String::from(match target.to_lowercase().as_str() {
        "esnext" => "2023",
        v @ _ => v.strip_prefix("es").unwrap_or("esnext")
    })
}

pub fn minify(text: String, options: &Options) -> Option<String> {
    include_script!(SHARED_RUNTIME,r"terser.js");

    return with_v8! {
        use runtime(scope,context) = SHARED_RUNTIME;

        let global_this = context.global(scope);
        let terser = v8_get!(global_this.Terser)?.to_object(scope)?;
        let minify = v8::Local::<v8::Function>::try_from(v8_get!(terser.minify)?.to_object(scope)?).ok()?;
    
        let text = v8_str!(text.as_str());

        // See https://github.com/terser/terser/blob/master/tools/terser.d.ts
        // https://terser.org/docs/options/
        let args: v8::Local<v8::Object> = v8_object!({
            module: v8_bool!(options.module),
            keep_classnames: v8_bool!(true),
            compress: v8_object!({
                ecma: v8_str!(&format_ecma_version_string(options.target.clone()))
            }),
            mangle: v8_object!({

            }),
            format: v8_object!({
                ecma: v8_str!(&format_ecma_version_string(options.target.clone())),
                comments: v8_str!("/^!/")
            })
        });

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
}
