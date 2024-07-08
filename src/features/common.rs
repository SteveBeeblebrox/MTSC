// Shared Code
use fancy_default::Default;

#[cfg(all(feature = "transpile", feature = "compile"))]
#[derive(Default,Debug,PartialEq,PartialOrd)]
pub enum TSMode {
    #[default]
    Preserve,
    Transpile,
    Compile,
}

#[derive(Clone,Default,Debug,std::hash::Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Options {
    #[cfg(feature = "common")]
    #[default(expr=String::from("es2022"))]
    pub target: String,
    #[cfg(feature = "common")]
    pub module: bool,

    // Compile and Transpile Features
    #[cfg(all(feature = "transpile", feature = "compile"))]
    pub ts: TSMode,
    #[cfg(all(not(feature = "transpile"), feature = "compile"))]
    #[default(expr=true)]
    pub compile: bool,
    #[cfg(all(feature = "transpile", not(feature = "compile")))]
    #[default(expr=true)]
    pub transpile: bool,

    #[cfg(any(feature = "transpile", feature = "compile"))]
    pub use_jsx: bool,
    #[cfg(any(feature = "transpile", feature = "compile"))]
    pub jsx_factory: Option<String>,
    #[cfg(any(feature = "transpile", feature = "compile"))]
    pub jsx_fragment: Option<String>,

    // Minify Feature
    #[cfg(feature = "minify")]
    pub minify: bool,

    // Preprocess Feature
    #[cfg(feature = "preprocess")]
    pub preprocess: bool,
    #[cfg(feature = "preprocess")]
    pub macros: Vec<String>,
    #[cfg(feature = "preprocess")]
    pub filename: Option<String>,
    #[cfg(feature = "preprocess")]
    pub include_paths: Vec<String>,

    // HTML Feature
    #[cfg(feature = "html")]
    pub html: bool,
}

#[cfg(feature = "common")]
thread_local! {
    pub(in crate::features) static RUNTIME: runtime::Runtime<'static,'static> = runtime::Runtime::new();
}

#[cfg(feature = "common")]
pub(in crate::features) mod runtime {
    use std::sync::Once;

    // Based on https://github.com/Valerioageno/ssr-rs/blob/main/src/ssr.rs
    pub(in crate::features) struct Runtime<'s,'i> {
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
        pub(in crate::features) fn new() -> Self {
            #[cfg(not(feature = "external-v8"))]
            static V8_INIT: Once = Once::new();

            #[cfg(not(feature = "external-v8"))]
            V8_INIT.call_once(|| {
                let platform = v8::new_default_platform(0, false).make_shared();
                v8::V8::initialize_platform(platform);
                v8::V8::initialize();
            });

            let isolate = Box::into_raw(Box::new(v8::Isolate::new(v8::CreateParams::default())));
            let handle = unsafe { Box::into_raw(Box::new(v8::HandleScope::new(&mut *isolate))) };
            let context = unsafe { Box::into_raw(Box::new(v8::Context::new(&mut *handle))) };
            let scope = unsafe { Box::into_raw(Box::new(v8::ContextScope::new(&mut *handle, *context))) };

            return Runtime {isolate, handle, context, scope};
        }

        pub(in crate::features) fn get_isolate(&self) -> &mut v8::OwnedIsolate {
            return unsafe {&mut *self.isolate};
        }

        pub(in crate::features) fn get_handle(&self) -> &mut v8::HandleScope<'s,()> {
            return unsafe {&mut *self.handle};
        }

        pub(in crate::features) fn get_context(&self) -> &mut v8::Local<'s, v8::Context> {
            return unsafe {&mut *self.context};
        }

        pub(in crate::features) fn get_scope(&self) -> &mut v8::ContextScope<'i, v8::HandleScope<'s,v8::Context>> {
            return unsafe {&mut *self.scope};
        }

        pub(in crate::features) fn run<S: AsRef<str>>(&self, s: S) -> Option<v8::Local<'s, v8::Value>> {
            let scope = self.get_scope();
            let script = v8::String::new(scope, s.as_ref())?.into();
            let script = v8::Script::compile(scope, script, None)?;
            return script.run(scope);
        }
    }

    pub(in crate::features) trait UsingRuntime<'x,'s,'i> {
        fn using_runtime<F, R>(&'x self, f: F) -> R where F: FnOnce(&Runtime<'s,'i>) -> R;
    }

    impl <'x,'s,'i> UsingRuntime<'x,'s,'i> for Runtime<'s,'i> where 's: 'i {
        fn using_runtime<F, R>(&'x self, f: F) -> R where F: FnOnce(&Runtime<'s,'i>) -> R {
            f(self)
        }
    }

    impl UsingRuntime<'static,'static, 'static> for std::thread::LocalKey<Runtime<'static,'static>> {
        fn using_runtime<F, R>(&'static self, f: F) -> R where F: FnOnce(&Runtime<'static,'static>) -> R {
            self.with(f)
        }
    }
}

#[macro_export]
macro_rules! with_v8 {
    (use $runtime:ident = $src:expr; $($body:tt)*) => {
        {
            use crate::features::common::runtime::UsingRuntime as _;
            $src.using_runtime(|$runtime| {
                #[allow(unused_macros)]
                macro_rules! v8_str {
                    ($expression:expr) => {
                        v8::String::new($runtime.get_scope(), $expression)?.into()
                    };
                }
                
                #[allow(unused_macros)]
                macro_rules! v8_set {
                    ($obj:ident . $prop:ident = $value:expr) => {
                        {
                            let prop_name = v8_str!(stringify!($prop));
                            let prop_value = $value; 
                            $obj.set($runtime.get_scope(),prop_name,prop_value)
                        }
                    };
                }
            
                #[allow(unused_macros)]
                macro_rules! v8_get {
                    ($obj:ident . $prop:ident) => {
                        {
                            let prop_name = v8_str!(stringify!($prop));
                            $obj.get($runtime.get_scope(),prop_name)
                        }
                    };
                }

                #[allow(unused_macros)]
                macro_rules! v8_bool {
                    ($expression:expr) => {
                        v8::Boolean::new($runtime.get_scope(), $expression).into()
                    };
                }

                #[allow(unused_macros)]
                macro_rules! v8_object {
                    ({$$($prop:ident : $value:expr),*}) => {
                        {
                            let object = v8::Object::new($runtime.get_scope());
                            $$(
                                let prop_name = v8_str!(stringify!($prop));
                                let prop_value = $value; 
                                object.set($runtime.get_scope(),prop_name,prop_value);
                            )*
                            object.into()
                        }
                    };
                }
        
                $($body)*
            })
        }
    };
}

pub use with_v8;