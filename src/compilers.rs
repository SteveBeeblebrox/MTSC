

use crate::wave;
use wave::{preprocess_text};

use std::convert::TryFrom;
use std::default::Default;

use v8;

#[derive(Clone)]
pub struct CompileOptions {
    pub target: String,
    pub module: bool,
    pub use_jsx: bool,
    pub jsx_factory: Option<String>,
    pub jsx_fragment: Option<String>,

    pub use_preprocessor: bool,
    pub macros: Vec<String>,
    pub filename: Option<String>,
    pub include_paths: Vec<String>
}

#[derive(Clone)]
pub struct MinifyOptions {
    pub target: String,
    pub module: bool
}

impl From<CompileOptions> for MinifyOptions {
    fn from(options: CompileOptions) -> Self {
        MinifyOptions {
            target: options.target,
            module: options.module
        }
    }
}