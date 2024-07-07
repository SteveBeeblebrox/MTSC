// Transpile Feature
use super::common::{self,Options};

use std::convert::TryFrom;

static TYPESCRIPT: &str = include_str!(r"typescript.js");

use lazy_static::lazy_static;
use std::sync::{Mutex,Once,Arc};
use std::cell::RefCell;
use std::rc::Rc;

macro_rules! timer {
    ($msg:literal,$expression:expr) => {
        {
            let timer_instant = std::time::Instant::now();
            let rvalue = $expression;
            eprintln!($msg, format!("{:.2?}", timer_instant.elapsed()));
            rvalue
        }
    };
    ($expression:expr) => {
        timer!("Time Elapsed: {:.2?}",$expr)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time() {
        timer!("Run 0: {}", assert!(transpile(String::from("let x: number = 1;"), &Default::default()) == Some("let x = 1;\n".to_string()), "Unexpected compile result 0!"));
        timer!("Run 1: {}", assert!(transpile(String::from("let x: number = 1;"), &Default::default()) == Some("let x = 1;\n".to_string()), "Unexpected compile result 1!"));
        timer!("Run 2: {}", assert!(transpile(String::from("let x: number = 1;"), &Default::default()) == Some("let x = 1;\n".to_string()), "Unexpected compile result 2!"));
    }
}

static TS_INIT: Once = Once::new();

// Based on https://github.com/Valerioageno/ssr-rs/blob/main/src/ssr.rs
struct Runtime<'s,'i> {
    isolate: *mut v8::OwnedIsolate,
    handle: *mut v8::HandleScope<'s,()>,
    context: *mut v8::Local<'s, v8::Context>,
    scope: *mut v8::ContextScope<'i, v8::HandleScope<'s,v8::Context>>,
}

impl Drop for Runtime<'_, '_> {
    fn drop(&mut self) {
        use std::mem::drop;
        unsafe {
            drop(Box::from_raw(self.scope));
            drop(Box::from_raw(self.context));
            drop(Box::from_raw(self.handle));
            drop(Box::from_raw(self.isolate));
        };
    }
}

#[allow(unused)]
impl<'s, 'i> Runtime<'s, 'i> where 's: 'i, {
    fn new() -> Self {
        common::init_v8();

        let isolate = Box::into_raw(Box::new(v8::Isolate::new(v8::CreateParams::default())));
        let handle = unsafe { Box::into_raw(Box::new(v8::HandleScope::new(&mut *isolate))) };
        let context = unsafe { Box::into_raw(Box::new(v8::Context::new(&mut *handle))) };
        let scope = unsafe { Box::into_raw(Box::new(v8::ContextScope::new(&mut *handle, *context))) };

        return Runtime {isolate, handle, context, scope};
    }

    fn get_isolate(&self) -> &mut v8::OwnedIsolate {
        return unsafe {&mut *self.isolate};
    }

    fn get_handle(&self) -> &mut v8::HandleScope<'s,()> {
        return unsafe {&mut *self.handle};
    }

    fn get_context(&self) -> &mut v8::Local<'s, v8::Context> {
        return unsafe {&mut *self.context};
    }

    fn get_scope(&self) -> &mut v8::ContextScope<'i, v8::HandleScope<'s,v8::Context>> {
        return unsafe {&mut *self.scope};
    }
}

thread_local! {
    static RUNTIME: Runtime<'static,'static> = Runtime::new();
}

pub fn transpile(text: String, options: &Options) -> Option<String> {
    common::init_v8();
    
    return RUNTIME.with(|runtime| {
        let context = runtime.get_context();
        let scope = runtime.get_scope();
    
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
    
        TS_INIT.call_once(|| {
            let script = v8::String::new(scope, TYPESCRIPT).expect("init err").into();
            let script = v8::Script::compile(scope, script, None).expect("init err");
            script.run(scope).expect("init err");
        });

    
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
    });
}