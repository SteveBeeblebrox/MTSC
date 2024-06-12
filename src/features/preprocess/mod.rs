use super::common::Options;

mod wave;

pub fn preprocess(text: String, _options: &Options) -> Option<String> {
    return wave::preprocess_text(text, "foobar".into(), vec![], vec![]);
}