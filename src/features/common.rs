// Shared Code
use std::sync::Once;
use fancy_default::Default;

#[cfg(all(feature = "common", not(feature = "external-v8")))]
static V8_INIT: Once = Once::new();

#[cfg(feature = "common")]
pub fn init() {
    #[cfg(not(feature = "external-v8"))]
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

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