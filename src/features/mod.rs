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

// Shared V8 Code
#[cfg(feature = "common")]
mod common;

#[cfg(feature = "common")]
pub use common::init_v8;

// Compile Feature
#[cfg(feature = "compile")]
mod compile;
#[cfg(feature = "compile")]
pub use compile::compile;


// Transpile Feature
#[cfg(feature = "transpile")]
mod transpile;
#[cfg(feature = "transpile")]
pub use transpile::transpile;


// Minify Feature
#[cfg(feature = "minify")]
mod minify;
#[cfg(feature = "minify")]
pub use minify::minify;


// Preprocessor Feature 
#[cfg(feature = "preprocess")]
mod preprocess;
#[cfg(feature = "preprocess")]
pub use preprocess::preprocess;


// HTML Feature
#[cfg(all(feature = "html", not(any(feature = "compile", feature = "transpile"))))]
compile_error!("feature \"html\" requires feature \"compile\" and/or feature \"transpile\" to be enabled");
#[cfg(feature = "html")]
mod html;
#[cfg(feature = "html")]
pub use html::compile_html;