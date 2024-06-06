// Shared Code
use std::sync::Once;
use fancy_default::Default;

#[cfg(feature = "common")]
static V8_INIT: Once = Once::new();

#[cfg(feature = "common")]
pub fn init() {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

#[cfg(all(feature = "transpile", feature = "compile"))]
#[derive(Default)]
pub enum TSMode {
    Transpile,
    Compile,
    #[default]
    Preserve
}

#[derive(Clone,Default)]
pub struct Options {
    #[cfg(feature = "common")]
    #[default(expr=String::from("es2022"))]
    pub target: String,
    #[cfg(feature = "common")]
    pub module: bool,

    // Compile and Transpile Features
    #[cfg(all(feature = "transpile", feature = "compile"))]
    #[allow(unused)]
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