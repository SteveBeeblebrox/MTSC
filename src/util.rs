use cfg_if::cfg_if;

use crate::Options;
#[cfg(all(feature = "compile", feature = "transpile"))]
pub use crate::TSMode;

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
pub fn set_ext_options<'a, 'b>(ext: String, options: &'a mut Options, update_options: &'b Options) -> &'a mut Options {
    let ext = ext.as_str();
    match ext {
        #[cfg(feature = "html")]
        "html" => options.html |= update_options.html,

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

pub fn get_result_ext(ext: String, options: &Options) -> String {
    #[cfg(feature = "html")]
    if options.html {
        return String::from("html");
    }

    let ts = ts_enabled(options);

    return
        String::from(if minify_enabled(options) { "min." } else { "" })
        + match ext.as_str() {
            "ts" if ts => "js",
            "mts" if ts => "mjs",
            
            #[cfg(any(feature = "compile", feature = "transpile"))]
            "tsx" if options.jsx_factory == None && ts => "jsx",
            
            _ if ts => "js",

            _ => ext.as_str()
        };
}

#[inline(always)]
fn ts_enabled(options: &Options) -> bool {
    cfg_if! {
        if #[cfg(all(feature = "compile", feature = "transpile"))] {
            options.ts != TSMode::Preserve
        } else if #[cfg(feature = "compile")] {
            options.compile
        } else if #[cfg(feature = "transpile")] {
            options.transpile 
        } else {
            false
        }
    }
}

#[inline(always)]
fn minify_enabled(options: &Options) -> bool {
    cfg_if! {
        if #[cfg(feature = "minify")] {
            options.minify
        } else {
            false
        }
    }
}