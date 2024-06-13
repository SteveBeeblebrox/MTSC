use std::path::{Path, PathBuf};
use std::io::{self, Read as _};
use std::process::exit;
use std::panic;
use std::fs;

use clap::{Arg, App};
use backtrace::Backtrace;
use same_file::is_same_file;
use or_panic::OrPanic as _;

use mtsc::{compile,Options};

fn main() {
    // CLI options
    let matches = App::new("MTSC")
        .version(clap::crate_version!())
        .version_short("v")
        .author(clap::crate_authors!())
        .about("A standalone TypeScript compiler with support for JSX, HTML script tags, preprocessing, and minification")

        .arg(Arg::with_name("target")
            .short("t")
            .long("target")
            .value_name("ES-VERSION")
            .help("Sets the JavaScript language and library version for compiled code")
            .default_value("esnext")
            .takes_value(true)
        )

        .arg(Arg::with_name("module")
            .short("m")
            .long("module")
            .help("Treat the input as a modern ES module (Enabled by default for .mts files and HTML script tags with type 'tsmodule' or 'module/typescript')")
        )

        .arg(Arg::with_name("jsx")
            .short("x")
            .long("jsx")
            .value_name("JSX-FACTORY")
            .help("Sets the JSX factory for compiled code (When this option is set but blank or when the file extension is .tsx, JSX is preserved as is; otherwise, it is interpreted as standard code)")
            .default_value("")
            .hide_default_value(true)
            .takes_value(true)
        )

        .arg(Arg::with_name("jsx-fragment")
            .long("jsx-fragment")
            .value_name("JSX-FRAGMENT")
            .help("Sets the JSX fragment factory for compiled code (Requires the jsx option to be set as well)")
            .default_value("null")
            .takes_value(true)
        )

        .arg(Arg::with_name("name")
            .short("n")
            .long("name")
            .value_name("NAME")
            .help("Sets the file name and extension when not available via the main arg (Such as reading from stdin or file descriptors; this can be used by the preprocessor and to infer other options)")
            .takes_value(true)
        )
        
        .arg(Arg::with_name("preprocess")
            .short("p")
            .long("preprocessor")
            .help("Enables comment preprocessor (looks for directives within single-line triple-slash comments, e.g. '///#define')")
        )

        .arg(Arg::with_name("define")
            .short("D")
            .long("define")
            .value_name("MACROS")
            .help("Define macros using the form 'MACRO(x)=definition' (Unused if preprocessor is not enabled)")
            .takes_value(true)
            .number_of_values(1)
            .multiple(true)
        )

        .arg(Arg::with_name("include-paths")
            .short("I")
            .value_name("PATH")
            .help("Add additional include search paths (Unused if preprocessor is not enabled)")
            .takes_value(true)
            .number_of_values(1)
            .multiple(true)
        )

        .arg(Arg::with_name("output")
            .short("o")
            .long("out")
            .value_name("OUTPUT")
            .help("Sets the output file to write transpiled code to instead of using the input file's name with the extension changed to .js or .html in the case of HTML files (When set to '-' or set but blank, output is written to stdout; if set to a directory and an input file is provided, the output file will be written to the given directory with the extension changed to .js/.html)")
            .default_value("")
            .hide_default_value(true)
            .takes_value(true)
        )

        .arg(Arg::with_name("minify")
            .short("M")
            .long("minify")
            .help("Enables minification using Terser (both compression and mangling) of output code; except for HTML files, '.min' is prepend to the output file extension (Currently ignored if parsing HTML)")
        )

        .arg(Arg::with_name("html")
            .long("html")
            .short("H")
            .help("Treat the input as an HTML file and transpile any script tags with the type attribute set to 'text/typescript', 'application/typescript', 'tsmodule', or 'module/typescript'")
        )

        .arg(Arg::with_name("verbose")
            .short("V")
            .long("verbose")
            .help("Prints verbose error messages")
        )

        .arg(Arg::with_name("INPUT")
            .help("Sets the input file to compile (Leave blank or set to '-' to read from stdin)")
            .index(1)
        )
        .get_matches();

        macro_rules! cflag {
            ($expression:expr) => {
                (matches.occurrences_of($expression) > 0)
            }
        }

        macro_rules! carg {
            ($expression:expr) => {
                matches.value_of($expression)
            }
        }

        macro_rules! cstrings {
            ($expression:expr) => {
                match matches.values_of($expression) {
                    Some(values) => values.into_iter().map(|v| String::from(v)).collect::<Vec<String>>(),
                    _ => vec![]
                }
            }
        }

        // Error handling
        let verbose = cflag!("verbose");
        panic::set_hook(Box::new(move |info| {
            eprintln!("\x1b[91;1merror\x1b[0m: {}", panic_message::panic_info_message(info));
            
            if verbose {
                eprintln!("{:?}", Backtrace::new());
            } else {
                eprintln!("rerun with -V for verbose error messages");
            }
            exit(1);
        }));

        // Read input
        let maybe_filename: Option<String> = carg!("INPUT").filter(|v| *v != "-").or_else(|| carg!("name")).map(|v| String::from(v));
        let maybe_ext: Option<String> = maybe_filename.as_ref().and_then(|v|
            Path::new(v.as_str()).extension().map(|s| String::from(s.to_str().expect("could not get extension from path")))
        );

        let text = match carg!("INPUT") {
            Some("-") | None => {
                let stdin = io::stdin();
                let mut stdin = stdin.lock();
                let mut line = String::new();
                stdin.read_to_string(&mut line).or_panic();
                String::from(line)
            },
            Some(value) => {
                fs::read_to_string(value).or_panic()
            }
        };

        let mut options = Options {
            target: String::from(carg!("target").unwrap()),
            module: cflag!("module"),
            transpile: true, // !cflag!("preserve")

            use_jsx: cflag!("jsx"),
            jsx_factory: carg!("jsx").filter(|s| *s != "").map(|s| String::from(s)),
            jsx_fragment: if carg!("jsx").is_some_and(|s| s != "") {carg!("jsx-factory").map(|s| String::from(s))} else {None},
            
            minify: cflag!("minify"),
            html: cflag!("html"),

            preprocess: cflag!("preprocess"),
            macros: cstrings!("define"),
            filename: maybe_filename.clone(),
            include_paths: cstrings!("include-paths"),
        };

        if let Some(ref ext) = maybe_ext {
            mtsc::util::update_options_by_ext(ext.clone(), &mut options, &Options {
                transpile: true,// !cflag!("preserve"),
                ..mtsc::util::all_ext_options()
            });
        }

        // Compile
        let result = compile(text, &options).unwrap();
        
        // Write output
        let maybe_result_ext = maybe_ext.map(|ext| mtsc::util::get_result_ext(ext,&options));

        match carg!("output") {
            Some("-") | Some("") if cflag!("output") => print!("{}",result),
            None | Some("") => {
                match maybe_filename {
                    Some(ref filename) => {
                        let mut path = PathBuf::from(filename);
                        if let Some(result_ext) = maybe_result_ext {
                            path.set_extension(result_ext);
                        }

                        if maybe_filename.is_some() && is_same_file(path.as_path(),maybe_filename.as_ref().unwrap()).or_panic() {
                            panic!("Output file is the same as the input");
                        }

                        fs::write(path,result.as_bytes()).or_panic();
                    },
                    None => print!("{}",result)
                }
            },
            Some(value) => {
                let mut path = PathBuf::from(value);

                if path.is_dir() {
                    path.push(maybe_filename.as_ref().unwrap_or(&String::from("out")));
                }

                if let Some(result_ext) = maybe_result_ext {
                    path.set_extension(result_ext);
                }

                if maybe_filename.is_some() && is_same_file(path.as_path(),maybe_filename.as_ref().unwrap()).or_panic() {
                    panic!("Output file is the same as the input");
                }

                fs::write(path,result.as_bytes()).or_panic();
            }
        }
}
