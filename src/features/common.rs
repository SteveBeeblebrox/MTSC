// Shared Code
use runtime::Runtime;
use std::sync::{Mutex,LazyLock};
use std::cell::RefCell;

// thread_local! {
//     pub(in crate::features) static TLS_RUNTIME: RefCell<runtime::Runtime> = RefCell::new(runtime::Runtime::new());
//     // with_v8!{ use _ = TLS_RUNTIME; };
// }

pub(in crate::features) static SHARED_RUNTIME: Mutex<LazyLock<RefCell<Runtime>>> = Mutex::new(LazyLock::new(|| RefCell::new(Runtime::new())));

pub fn init_v8(primary: bool) {
    runtime::init_v8(primary);
    LazyLock::force(&SHARED_RUNTIME.lock().unwrap());
}

pub(in crate::features) mod runtime {
    use std::sync::{Mutex,LazyLock};
    use std::cell::RefCell;
    use std::thread::LocalKey;

    extern "C" fn get_new_heap_size(_data: *mut std::ffi::c_void, current_heap_limit: usize, _initial_heap_limit: usize) -> usize {
        return 2*current_heap_limit;
    }
    
    pub(in crate::features::common) fn init_v8(primary: bool) {
        use std::sync::Once;
        static V8_INIT: Once = Once::new();
    
        V8_INIT.call_once(|| {
            if primary {
                let platform = v8::new_default_platform(0, false).make_shared();
                v8::V8::initialize_platform(platform);
                v8::V8::initialize();
            }
        });
    }

    // Based on https://github.com/abnud1/rust-ssr/blob/main/src/ssr.rs#L51-L57
    pub(in crate::features) struct Runtime {
        pub isolate: v8::OwnedIsolate,
        pub context: v8::Global<v8::Context>,
    }

    unsafe impl Send for Runtime {}

    #[allow(unused)]
    impl Runtime {
        pub fn new() -> Self {
            init_v8(true);

            let mut isolate = v8::Isolate::new(Default::default());
            isolate.add_near_heap_limit_callback(get_new_heap_size, std::ptr::null_mut());

            let context = {
                let handle_scope = &mut v8::HandleScope::new(&mut isolate);
                let context = v8::Context::new(handle_scope);
                v8::Global::new(handle_scope, context)
            };

            return Self {
                isolate,
                context,
            };
        }

        pub fn run<'s,S: AsRef<str>>(&'s mut self, s: S) -> Option<v8::Local<'s, v8::Value>> {
            let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);    
            let context = v8::Local::new(handle_scope, &self.context);
            let scope = &mut v8::ContextScope::new(handle_scope, context);
            
            let script = v8::String::new(scope, s.as_ref())?.into();
            let script = v8::Script::compile(scope, script, None)?;
            return script.run(scope);
        }

        fn using_runtime<F, R>(&mut self, f: F) -> R where F: FnOnce(&mut Runtime) -> R {
            return f(self);
        }
    }

    pub trait UsingRuntime<'x> {
        fn using_runtime<F, R>(&'x self, f: F) -> R where F: FnOnce(&mut Runtime) -> R;
    }

    impl UsingRuntime<'static> for LocalKey<RefCell<Runtime>> {
        fn using_runtime<F, R>(&'static self, f: F) -> R where F: FnOnce(&mut Runtime) -> R {
            return self.with_borrow_mut(f);
        }
    }
    
    impl UsingRuntime<'static> for Mutex<LazyLock<RefCell<Runtime>>> {
        fn using_runtime<F, R>(&'static self, f: F) -> R where F: FnOnce(&mut Runtime) -> R {
            return f(&mut (**self.lock().unwrap()).borrow_mut());
        }
    }
}

#[macro_export]
macro_rules! with_v8 {
    (use _ = $src:expr; $($body:tt)*) => {
        {
            use $crate::features::common::runtime::UsingRuntime as _;
            $src.using_runtime(|_| {});
        }
    };
    (use $runtime:ident = $src:expr; $($body:tt)*) => {
        {
            use $crate::features::common::runtime::UsingRuntime as _;
            $src.using_runtime(|$runtime| {
                $runtime.isolate.perform_microtask_checkpoint();
                $runtime.isolate.low_memory_notification();

                let handle_scope = &mut v8::HandleScope::new(&mut $runtime.isolate);
                let context = v8::Local::new(handle_scope, &$runtime.context);
                let scope = &mut v8::ContextScope::new(handle_scope, context);
        
                #[allow(unused_macros)]
                macro_rules! scope {
                    () => {
                        scope
                    }
                }

                #[allow(unused_macros)]
                macro_rules! global_this {
                    () => {
                        context.global(scope)
                    }
                }

                #[allow(unused_macros)]
                macro_rules! v8_str {
                    ($expression:expr) => {
                        v8::String::new(scope, $expression)?.into()
                    };
                }
                
                #[allow(unused_macros)]
                macro_rules! v8_set {
                    ($obj:ident . $prop:ident = $value:expr) => {
                        {
                            let prop_name = v8_str!(stringify!($prop));
                            let prop_value = $value; 
                            $obj.set(scope,prop_name,prop_value)
                        }
                    };
                }
            
                #[allow(unused_macros)]
                macro_rules! v8_get {
                    ($obj:ident . $prop:ident) => {
                        {
                            let prop_name = v8_str!(stringify!($prop));
                            $obj.get(scope,prop_name)
                        }
                    };
                }

                #[allow(unused_macros)]
                macro_rules! v8_bool {
                    ($expression:expr) => {
                        v8::Boolean::new(scope, $expression).into()
                    };
                }

                #[allow(unused_macros)]
                macro_rules! v8_object {
                    ({$$($prop:ident : $value:expr),*}) => {
                        {
                            let object = v8::Object::new(scope);
                            $$(
                                let prop_name = v8_str!(stringify!($prop));
                                let prop_value = $value; 
                                object.set(scope,prop_name,prop_value);
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

pub(in crate::features) use with_v8;

#[macro_export]
macro_rules! include_script {
    ($src:expr,$script:literal) => {
        {
            use $crate::features::common::runtime::UsingRuntime as _;
            $src.using_runtime(|runtime| {
                runtime.run(include_str!($script)).expect(concat!("Error loading {}", $script));
            });
        }
    };
}

pub(in crate::features) use include_script;