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

/*
O U | R
-------
p p | p
p t | t
p c | t

t p | t
t t | t
t c | t

c p | c
c t | c
c c | c
*/
pub fn update_options_by_ext<'a, 'b>(ext: String, options: &'a mut Options, update_options: &'b Options) -> &'a mut Options {
    let ext = ext.as_str();
    match ext {
        #[cfg(feature = "html")]
        "html" => {
            options.html |= update_options.html;
            cfg_if! {
                if #[cfg(all(feature = "compile", feature = "transpile"))] {
                    if update_options.ts > options.ts {
                        options.ts = TSMode::Transpile;
                    }
                } else if #[cfg(feature = "transpile")] {
                    options.transpile |= update_options.transpile;
                }
            }
        }

        #[cfg(feature = "transpile")]
        "jsx" => {
            options.use_jsx |= update_options.use_jsx;
            cfg_if! {
                if #[cfg(all(feature = "compile", feature = "transpile"))] {
                    if update_options.ts > options.ts {
                        options.ts = TSMode::Transpile;
                    }
                } else if #[cfg(feature = "transpile")] {
                    options.transpile |= update_options.transpile;
                }
            }
        }

        #[cfg(feature = "common")]
        "ts" | "mts" | "tsx" => {
            cfg_if! {
                if #[cfg(all(feature = "compile", feature = "transpile"))] {
                    if update_options.ts > options.ts {
                        options.ts = update_options.ts;
                    }
                } else if #[cfg(feature = "compile")] {
                    options.compile |= update_options.compile;
                } else if #[cfg(feature = "transpile")] {
                    options.transpile |= update_options.transpile;
                }
            }

            match ext {
                "mts" => options.module |= update_options.module,

                #[cfg(any(feature = "transpile", feature = "compile"))]
                "tsx" => options.use_jsx |= update_options.use_jsx,

                _ => {}
            }
        },
        
        #[cfg(feature = "common")]
        "mjs" => options.module |= update_options.module,

        _ => {}
    }

    return options;
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

pub fn get_result_ext(ext: String, options: &Options) -> String {
    #[cfg(feature = "html")]
    if options.html {
        return String::from("html");
    }

    let result_ext = if
                optional!(#[cfg(feature = "compile")] options.compile).unwrap_or_default()
                || optional!(#[cfg(feature = "transpile")] options.transpile).unwrap_or_default()
                || optional!(#[cfg(all(feature = "compile", feature = "transpile"))] options.ts != TSMode::Preserve).unwrap_or_default()
            {
                match ext.as_str() {
                    "ts" => "js",
                    "mts" => "mjs",
                    
                    #[cfg(any(feature = "compile", feature = "transpile"))]
                    "tsx" if options.jsx_factory == None => "jsx",
                    
                    _ => "js",
                }
            } else {
                ext.as_str()
            }
        ;

    return String::from(if optional!(#[cfg(feature = "minify")] options.minify).unwrap_or_default() {"min."} else {""}) + result_ext;
}


pub enum OptionSource<'a> {
    Mime(&'a str),
    Extension(&'a str),
    SubExtension(&'a str),
    None
}

pub fn update_options<'a, 'b>(source: OptionSource<'a>, options: &'b mut Options, mask: &Options) -> &'b mut Options {
    use OptionSource::*;

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
            update_options(Extension("ts"),options,mask);
        },
        Extension("mjs") => {
            optional!(#[cfg(feature="common")] options.module |= mask.module);
        },
        Extension("jsx") => {
            optional!(#[cfg(any(feature = "transpile", feature = "compile"))] options.use_jsx |= mask.use_jsx);
        },
        Extension("tsx") => {
            optional!(#[cfg(any(feature = "transpile", feature = "compile"))] options.use_jsx |= mask.use_jsx);
            update_options(Extension("ts"),options,mask);
        },
        Mime("text/javascript") | Extension("js") | SubExtension("d") | SubExtension("min") | _ => {}
    }

    return options;
}



use std::path::{Path,PathBuf};
pub fn update_path(path: &PathBuf, options: &Options) -> PathBuf {
    return PathBuf::new();
}