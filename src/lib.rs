mod features;
pub use features::common::Options;
#[cfg(all(feature = "compile", feature = "transpile"))]
pub use features::common::TSMode;

use cfg_if::cfg_if;

pub fn compile<T: AsRef<str>>(text: T, options: Options) -> Option<String> {
    let mut text = String::from(text.as_ref());

    cfg_if! {
        if #[cfg(feature = "preprocess")] {
            if options.preprocess {
                text = features::preprocess(text,&options)?;
            }
        }
    }
    cfg_if! {
        if #[cfg(all(feature = "compile", feature = "transpile"))] {
            text = match options.ts {
                TSMode::COMPILE => features::compile(text,&options)?,
                TSMode::TRANSPILE => features::transpile(text,&options)?,
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
    cfg_if! {
        if #[cfg(feature = "minify")] {
            if options.minify {
                text = features::minify(text,&options)?;
            }
        }
    }

    return Some(text);
}