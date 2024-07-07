// Transpile Feature
use super::common::{self,Options};

use std::convert::TryFrom;

static TYPESCRIPT: &str = include_str!(r"typescript.js");

use lazy_static::lazy_static;
use std::sync::{Mutex,Once,Arc};
use std::cell::RefCell;
use std::rc::Rc;


/*
thread_local! {
    static V8_ISOLATE: Rc<RefCell<Option<v8::OwnedIsolate>>> = Rc::new(RefCell::new(None));
    static V8_CONTEXT: Rc<RefCell<Option<v8::Local<'static, v8::Object>>>> = Rc::new(RefCell::new(None));
}

thread_local! {
    static ISOLATE: RefCell<Option<v8::OwnedIsolate>> = RefCell::new(None);
    static HANDLE: RefCell<Option<v8::HandleScope<'static, ()>>> = RefCell::new(None);
}

#[allow(unused)]
fn init_ts() {
    static TS_INIT: Once = Once::new();
    TS_INIT.call_once(|| {
        common::init_v8();

        ISOLATE.with_borrow_mut(move |isolate| {
            *isolate = Some(v8::Isolate::new(Default::default()));

            static isolate = isolate.as_mut().unwrap();

            HANDLE.with_borrow_mut(move |handle| {
                *handle = Some(v8::HandleScope::new(isolate));
            });

            // let isolate = isolate.as_mut().unwrap();

            // let scope = &mut v8::HandleScope::new(&mut *isolate);
            // let context = v8::Context::new(scope);
            // let scope = &mut v8::ContextScope::new(scope, context);

            // let script = v8::String::new(scope, TYPESCRIPT).expect("Error with TS").into();
            // let script = v8::Script::compile(scope, script, None).expect("Error with TS");
            // script.run(scope).expect("Error with TS");
        });

    });


fn get_isolate() -> Rc<RefCell<Option<v8::OwnedIsolate>>> {
    V8_ISOLATE.with(|isolate| {
        if isolate.borrow().is_none() {
            common::init_v8();
            let isolate_ref = v8::Isolate::new(Default::default());
            isolate.replace(Some(isolate_ref));
        }
        Rc::clone(isolate)
    })
}


// thread_local! {
//     static ISOLATE: RefCell<v8::OwnedIsolate> = RefCell::new(v8::Isolate::new(Default::default()));
// }
// thread_local! {
//     static HANDLE: v8::HandleScope<'static,()> = v8::HandleScope::new(&mut ISOLATE.get());
// }

// #[allow(unused)]
// fn init_ts() -> &'static TS {
//     common::init_v8();

//     static ONCE: OnceLock<TS> = OnceLock::new();
//     return ONCE.get_or_init(|| {
//         common::init_v8();

//         static isolate = v8::Isolate::new(Default::default());

//         let scope = &mut v8::HandleScope::new(&mut isolate);
//         let context = v8::Context::new(scope);
//         let scope = &mut v8::ContextScope::new(scope, context);

//         let script = v8::String::new(scope, TYPESCRIPT).expect("Error with TS").into();
//         let script = v8::Script::compile(scope, script, None).expect("Error with TS");
//         script.run(scope).expect("Error with TS");

//         return TS {
//             scope
//         };
//     });
// }

// TODO load snapshot?


fn get_context() -> Rc<RefCell<Option<v8::Local<'static, v8::Object>>>> {
    V8_CONTEXT.with_borrow_mut(|context| {
        if *context.is_none() {
            let isolate: &'static mut v8::OwnedIsolate = get_isolate().borrow_mut().as_mut().unwrap();
            let handle = &mut v8::HandleScope::new(&mut *isolate);
            let xcontext = v8::Context::new(handle);
            let scope = &mut v8::ContextScope::new(handle, xcontext);


            context.replace(Some(xcontext.global(scope)));
        }
        Rc::clone(context)
    })
}
}*/

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

struct Runtime<'a,'b> {
    isolate: *mut v8::OwnedIsolate,
    handle: *mut v8::HandleScope<'a,()>,
    context: *mut v8::Local<'a, v8::Context>,
    scope: *mut v8::ContextScope<'b, v8::HandleScope<'a,v8::Context>>,
}

// TODO impl drop
// https://github.dev/Valerioageno/ssr-rs

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

impl<'s, 'i> Runtime<'s, 'i> where 's: 'i, {
    fn new() -> Self {
        common::init_v8();

        let isolate = Box::into_raw(Box::new(v8::Isolate::new(v8::CreateParams::default())));

        let handle_scope = unsafe { Box::into_raw(Box::new(v8::HandleScope::new(&mut *isolate))) };

        let context = unsafe { Box::into_raw(Box::new(v8::Context::new(&mut *handle_scope))) };

        let scope_ptr =
            unsafe { Box::into_raw(Box::new(v8::ContextScope::new(&mut *handle_scope, *context))) };

        let scope = unsafe { &mut *scope_ptr };

        return Runtime {isolate, handle: handle_scope, context, scope};
    }
}

thread_local! {
    static RUNTIME: RefCell<Runtime<'static,'static>> = RefCell::new(Runtime::new());
}

thread_local! {
    static ISOLATE: RefCell<v8::OwnedIsolate> = RefCell::new(v8::Isolate::new(Default::default()));
}

pub fn transpile(text: String, options: &Options) -> Option<String> {
    common::init_v8();
    
    return RUNTIME.with_borrow_mut(|runtime| {
        let context = unsafe {&mut *runtime.context};
        let scope = unsafe {&mut *runtime.scope};
        // let scope = &mut v8::HandleScope::new(isolate);
        // let context = v8::Context::new(scope);
        // let scope = &mut v8::ContextScope::new(scope, context);
    
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