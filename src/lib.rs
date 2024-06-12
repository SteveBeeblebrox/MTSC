mod features;
pub use features::common::Options;
#[cfg(all(feature = "compile", feature = "transpile"))]
pub use features::common::TSMode;

pub mod util;

use cfg_if::cfg_if;

pub fn compile_script<T: AsRef<str>>(text: T, options: &Options) -> Option<String> {
    let mut text = String::from(text.as_ref());

    #[cfg(feature = "preprocess")]
    if options.preprocess {
        text = features::preprocess(text,&options)?;
    }

    cfg_if! {
        if #[cfg(all(feature = "compile", feature = "transpile"))] {
            text = match options.ts {
                TSMode::Compile => features::compile(text,&options)?,
                TSMode::Transpile => features::transpile(text,&options)?,
                _ => text
            }
        } else if #[cfg(feature = "compile")] {
            if options.compile {
                text = features::compile(text,&options)?;
            }
        } else if #[cfg(feature = "transpile")] {
            if options.transpile {
                text = features::transpile(text,&options)?;
            }
        }
    }

    #[cfg(feature = "minify")]
    if options.minify {
        text = features::minify(text,&options)?;
    }

    return Some(text);
}

pub fn compile<T: AsRef<str>>(text: T, options: &Options) -> Option<String> {
    #[cfg(feature = "html")]
    if options.html {
        return features::compile_html(String::from(text.as_ref()), options);
    }
    
    return compile_script(text,options);
}