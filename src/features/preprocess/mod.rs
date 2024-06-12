use super::common::Options;

mod wave;

pub fn preprocess(text: String, options: &Options) -> Option<String> {
    return wave::preprocess_text(text, options.filename.clone(), options.macros.clone(), options.include_paths.clone());
}