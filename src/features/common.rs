// Shared Code
use deno_core::{JsRuntime,RuntimeOptions};
use std::cell::RefCell;

thread_local! {
    pub(in crate::features) static SHARED_RUNTIME: RefCell<JsRuntime> = RefCell::new(JsRuntime::new(RuntimeOptions {
        ..Default::default()
    }));
}

pub fn init_v8() {
    thread_local! {
        static V8_INIT: std::sync::Once = std::sync::Once::new();
    }
    V8_INIT.with(|it| it.call_once(|| {
        with_v8! {
            use _ = SHARED_RUNTIME;
            /*runtime.add_near_heap_limit_callback(|current,_initial| {
                return current * 2;
            })*/
        }
    })); 
}

#[macro_export]
macro_rules! with_v8 {
    (use _ = $runtime_src:expr; $($body:tt)*) => {
        {
            $runtime_src.with(|_| {
                $($body)*
            })
        }
    };
    (use $runtime:ident = $runtime_src:expr; $($body:tt)*) => {
        {
            $runtime_src.with_borrow_mut(|$runtime| {
                $($body)*
            })
        }
    };
    (use $runtime:ident($scope:ident, $context:ident) = $runtime_src:expr; $($body:tt)*) => {
        {
            $runtime_src.with_borrow_mut(|$runtime| {
                use deno_core::v8;
                $runtime.v8_isolate().low_memory_notification();
                let $scope = &mut $runtime.handle_scope();
                let $context = $scope.get_current_context();

                #[allow(unused_macros)]
                macro_rules! v8_str {
                    ($expression:expr) => {
                        v8::String::new($scope, $expression)?.into()
                    };
                }
                
                #[allow(unused_macros)]
                macro_rules! v8_set {
                    ($obj:ident . $prop:ident = $value:expr) => {
                        {
                            let prop_name = v8_str!(stringify!($prop));
                            let prop_value = $value; 
                            $obj.set($scope,prop_name,prop_value)
                        }
                    };
                }
            
                #[allow(unused_macros)]
                macro_rules! v8_get {
                    ($obj:ident . $prop:ident) => {
                        {
                            let prop_name = v8_str!(stringify!($prop));
                            $obj.get($scope,prop_name)
                        }
                    };
                }

                #[allow(unused_macros)]
                macro_rules! v8_bool {
                    ($expression:expr) => {
                        v8::Boolean::new($scope, $expression).into()
                    };
                }

                #[allow(unused_macros)]
                macro_rules! v8_object {
                    ({$$($prop:ident : $value:expr),*}) => {
                        {
                            let object = v8::Object::new($scope);
                            $$(
                                let prop_name = v8_str!(stringify!($prop));
                                let prop_value = $value; 
                                object.set($scope,prop_name,prop_value);
                            )*
                            object.into()
                        }
                    };
                }

                #[allow(unused_macros)]
                macro_rules! global_this {
                    () => {
                        $context.global($scope)
                    }
                }
        
                $($body)*
            })
        }
    };
}

pub use with_v8;

#[macro_export]
macro_rules! include_script {
    ($runtime:ident, $script:literal) => {
        thread_local! {
            static SCRIPT_INIT: std::sync::Once = std::sync::Once::new();
        }
        SCRIPT_INIT.with(|it| it.call_once(|| {
            $runtime.with_borrow_mut(|runtime| runtime.execute_script($script,include_str!($script)).expect(concat!("Error loading {}", $script)));
        }));
    }
}

pub use include_script;