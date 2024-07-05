use cfg_if::cfg_if;

use crate::Options;
#[cfg(all(feature = "compile", feature = "transpile"))]
pub use crate::TSMode;

pub fn all_ext_options() -> Options {
    Options {
        #[cfg(all(feature = "transpile", feature = "compile"))]
        ts: TSMode::Transpile,
        #[cfg(all(not(feature = "transpile"), feature = "compile"))]
        compile: true,
        #[cfg(all(feature = "transpile", not(feature = "compile")))]
        transpile: true,

        #[cfg(any(feature = "transpile", feature = "compile"))]
        use_jsx: true,

        #[cfg(feature = "common")]
        module: true,

        #[cfg(feature = "html")]
        html: true,

        ..Default::default()
    }
}

macro_rules! optional {
    (#[cfg($meta:meta)] $expression:expr) => {
        {
            #[cfg($meta)]
            {Some($expression)}
            #[cfg(not($meta))]
            {None}
        }
    }
}

use std::path::{Path,PathBuf};
pub enum OptionSource {
    Mime(String),
    Name(String),
    Path(PathBuf),
    None
}

/*
O M | R
-------
p p | p
p t | t
p c | c

t p | t
t t | t
t c | c

c p | c
c t | c
c c | c
*/
pub fn update_options<'a>(source: OptionSource, options: &'a mut Options, mask: &'a Options) -> &'a mut Options {
    enum InternalOptionSource<'a> {
        Mime(&'a str),
        Extension(&'a str),
        SubExtension(&'a str),
    }

    fn update_options_internal<'a>(source: InternalOptionSource, options: &'a mut Options, mask: &'a Options) -> &'a mut Options {
        use InternalOptionSource::*;
        match source {
            SubExtension("p") | SubExtension("pre") => {
                optional!(#[cfg(feature="preprocess")] options.preprocess |= mask.preprocess); 
            },
            Mime("text/html") | Extension("html") => {
                optional!(#[cfg(feature="html")] options.html |= mask.html);
            },
            Mime("text/typescript") | Extension("ts") | SubExtension("ts") => {
                cfg_if! {
                    if #[cfg(all(feature = "compile", feature = "transpile"))] {
                        if mask.ts > options.ts {
                            options.ts = mask.ts;
                        }
                    } else if #[cfg(feature = "compile")] {
                        options.compile |= mask.compile;
                    } else if #[cfg(feature = "transpile")] {
                        options.transpile |= mask.transpile;
                    }
                }
            },
            Extension("mts") => {
                optional!(#[cfg(feature="common")] options.module |= mask.module);
                update_options_internal(Extension("ts"),options,mask);
            },
            Extension("mjs") => {
                optional!(#[cfg(feature="common")] options.module |= mask.module);
            },
            Extension("jsx") => {
                optional!(#[cfg(any(feature = "transpile", feature = "compile"))] options.use_jsx |= mask.use_jsx);
            },
            Extension("tsx") => {
                optional!(#[cfg(any(feature = "transpile", feature = "compile"))] options.use_jsx |= mask.use_jsx);
                update_options_internal(Extension("ts"),options,mask);
            },
            Mime("text/javascript") | Extension("js") | SubExtension("d") | SubExtension("min") | _ => {}
        }

        return options;
    }
    
    match &source {
        OptionSource::Mime(s) => {
            update_options_internal(InternalOptionSource::Mime(&s), options, mask);
        },
        OptionSource::Name(name) => {
            update_options(OptionSource::Path(PathBuf::from(name)), options, mask);
        },
        OptionSource::Path(path) => {
            path.file_stem().and_then(|stem| Path::new(stem).extension()).and_then(|ext| ext.to_str()).map(|s| update_options_internal(InternalOptionSource::SubExtension(s), options, mask));
            path.extension().and_then(|ext| ext.to_str()).map(|s| update_options_internal(InternalOptionSource::Extension(s), options, mask));    
        },
        OptionSource::None => {}
    }
    return options;
}

pub fn update_path<'a>(path: &'a mut PathBuf, options: &'a Options) -> &'a PathBuf {
    let initial_path = path.clone();

    fn get_result_subext(options: &Options) -> Option<&str> {
        if optional!(#[cfg(feature = "minify")] options.minify).unwrap_or_default() {
            Some("min")
        } else {
            None
        }
    }

    fn get_result_ext<'a>(maybe_initial_ext: Option<&'a str>, options: &'a Options) -> Option<&'a str> {
        return maybe_initial_ext.map(|initial_ext| {
            if optional!(#[cfg(feature = "html")] options.html).unwrap_or_default() {
                "html"
            } else if optional!(#[cfg(all(feature = "compile", not(feature = "transpile")))] options.compile).unwrap_or_default()
                || optional!(#[cfg(all(not(feature = "compile"), feature = "transpile"))] options.transpile).unwrap_or_default()
                || optional!(#[cfg(all(feature = "compile", feature = "transpile"))] options.ts != TSMode::Preserve).unwrap_or_default()
            {
                match initial_ext {
                    "mts" => "mjs",

                    #[cfg(any(feature = "compile", feature = "transpile"))]
                    "tsx" if options.jsx_factory == None && options.use_jsx => "jsx",

                    "ts" | _ => "js"
                }
            } else {
                initial_ext
            }
        });
    }
    
    // For subextensions that should be discarded like the 'p' in '*.p.ts',
    // removing the main extension will cause the subextension to be replaced later
    // (If a path has a subextension, it will have a normal extension as well and the next block will be executed)
    if path.file_stem().and_then(|stem| Path::new(stem).extension()).and_then(|ext| ext.to_str()).map(|ext| matches!(ext, "p" | "min")).unwrap_or_default() {
        path.set_extension("");
    }

    if let Some(ext) = get_result_ext(initial_path.extension().and_then(|ext| ext.to_str()), &options) {
        // If there is a subextension to add, prepend it to the value in the next set_extension call
        path.set_extension(get_result_subext(&options).map_or_else(|| String::from(ext), |subext| format!("{}.{}", subext, ext)));
    }
    
    return path;
}