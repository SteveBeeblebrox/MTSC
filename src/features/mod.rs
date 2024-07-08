// Shared Code
mod common;
pub use common::Options;
#[cfg(all(feature = "transpile", feature = "compile"))]
pub use common::TSMode;

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