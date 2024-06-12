/*mod compilers;
use compilers::{compile_typescript, compile_html, CompileOptions, minify_javascript, MinifyOptions};

mod wave;

use clap::{Arg, App};

use backtrace::Backtrace;

use std::io::prelude::*;
use std::path::PathBuf;
use std::path::Path;
use std::fs::File;
use std::fs;

use std::io::Read;
use std::io;
use std::panic;

use std::process::exit;*/

use mtsc;

fn main() {
    let result = mtsc::compile(r#"
        ///#define strc(x) #x
        let a: number = 1+1;
        let x: string = strc(117);
    "#, &mtsc::Options {
        preprocess: true,
        transpile: true,
        minify: true,
        ..Default::default()
    }).unwrap();
    println!("{}",result);
    /*
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
            .help("Sets the file name and extension when not available via the main arg (Such as reading from stdin or file descriptors)")
            .takes_value(true)
        )

        .arg(Arg::with_name("preprocessor")
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

        let verbose = matches.occurrences_of("verbose") > 0;
        if cfg!(not(debug_assertions)) {

        }
        panic::set_hook(Box::new(move |info| {
            eprintln!("\x1b[91;1merror\x1b[0m: {}", panic_message::panic_info_message(info));
            
            if verbose {
                eprintln!("{:?}", Backtrace::new());
            } else {
                eprintln!("rerun with -V for verbose error messages");
            }
            exit(1);
        }));

        // Determine input file (or stdin)
        let (input_file, input_text, input_type) = match matches.value_of("INPUT") {
            Some("-") | None => {
                let stdin = io::stdin();
                let mut stdin = stdin.lock();
                let mut line = String::new();

                stdin.read_to_string(&mut line).expect("could not read stdin");
                (
                    matches.value_of("name").map(|s| String::from(s)), 
                    String::from(line),
                    matches.value_of("name").map(|s| Path::new(s).extension().expect("could not get target file extension from name").to_str().unwrap()).unwrap_or("").to_string(), 
                )
            },
            Some(value) => (
                Some(String::from(value)), 
                fs::read_to_string(value).ok().expect("could not read target file"),
                Path::new(value).extension().or(matches.value_of("name").map(|s| Path::new(s).extension()).flatten()).expect("could not get target file extension").to_str().unwrap().to_string()
            )
        };

        // Determine jsx configuration
        let (use_jsx, jsx_factory, jsx_fragment) = match matches.value_of("jsx") {
            Some("") if matches.occurrences_of("jsx") > 0 => (if matches.occurrences_of("jsx") > 0 {true} else {false}, None, None),
            None | Some("") => (input_type == "tsx", None, None),
            Some(value) => (true, Some(String::from(value)), Some(String::from(matches.value_of("jsx-fragment").unwrap()))),
        };

        let html = matches.occurrences_of("html") > 0;

        let minify = matches.occurrences_of("minify") > 0 && !html;

        let output_type = match input_type.as_str() {
            _ if html => "html",
            "ts" => "js",
            "tsx" if jsx_factory == None => "jsx",
            "mts" => "mjs",
            _ => "js"
        };
        let output_type = if minify {
            format!("min.{}", output_type)
        } else {
            String::from(output_type)
        };

        let macros: Vec<String> = match matches.values_of("define") {
            Some(values) => values.into_iter().map(|v| String::from(v)).collect::<Vec<String>>(),
            _ => vec![]
        };

        let include_paths: Vec<String> = match matches.values_of("include-paths") {
            Some(values) => values.into_iter().map(|v| String::from(v)).collect::<Vec<String>>(),
            _ => vec![]
        };
        
        let use_preprocessor = matches.occurrences_of("preprocessor") > 0;

        let options = CompileOptions {
            target: String::from(matches.value_of("target").unwrap()),
            module: matches.occurrences_of("module") > 0 || input_type == "mts",
            use_jsx,
            jsx_factory,
            jsx_fragment,

            use_preprocessor,
            macros,
            filename: input_file.clone(),
            include_paths
        };

        let result = if html {
            compile_html(input_text.as_str(), options.clone()).expect("error compiling HTML")
        } else {
            compile_typescript(input_text.as_str(), options.clone()).expect("error compiling TypeScript")
        };

        let result = if minify {
            minify_javascript(result.as_str(), MinifyOptions::from(options.clone())).expect("error minifying JavaScript")
        } else {
            result
        };

        match matches.value_of("output") {
            Some("-") | Some("") if matches.occurrences_of("output") > 0 => print!("{}", result.as_str()),
            None | Some("") => {
                match input_file {
                    Some(input_file) => {
                        let mut path = PathBuf::from(input_file);
                        path.set_extension(output_type.as_str());
                        let mut file = File::create(path).expect("could not create output file");
                        file.write_all(result.as_bytes()).expect("could not write to output file");
                    },
                    None => print!("{}", result.as_str())
                }
            }
            Some(path) => {
                let path = if Path::new(path).exists() && fs::metadata(path).expect("could not reading file metadata").is_dir() && input_file.is_some() {
                    let mut path = PathBuf::from(path);
                    path.push(Path::new(&input_file.unwrap().to_string()).file_name().expect("could not get file name").to_str().expect("could not get file name"));
                    path.set_extension(output_type.as_str());
                    path
                } else {
                    PathBuf::from(path)
                };

                let mut file = File::create(path).expect("could not create output file");
                file.write_all(result.as_bytes()).expect("could not write to output file");
            }
        }

        */
}
