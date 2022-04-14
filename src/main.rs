use clap::{Arg, App};

use std::convert::TryFrom;
use std::io::prelude::*;
use std::path::PathBuf;
use std::path::Path;
use std::fs::File;
use std::fs;

use std::io::Read;
use std::io;

static TYPESCRIPT_SERVICES: &str = include_str!(r"typescriptServices.js");
fn main() {
    let matches = App::new("MTSC")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("A standalone TypeScript compiler")

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
            .value_name("MODULE-VERSION")
            .help("Sets the module type for compiled code")
            .default_value("esnext")
            .takes_value(true)
        )

        .arg(Arg::with_name("jsx")
            .short("x")
            .long("jsx")
            .value_name("JSX")
            .help("Sets the JSX factory for compiled code (When this option is set but blank or when the file extension is .tsx, JSX is preserved as is; otherwise, it is interpreted as standard code)")
            .default_value("")
            .takes_value(true)
        )

        .arg(Arg::with_name("jsx-fragment")
            .long("jsx-fragment")
            .value_name("JSX-FRAGMENT")
            .help("Sets the JSX fragment for compiled code (Requires the jsx option to be set as well)")
            .default_value("null")
            .takes_value(true)
        )

        .arg(Arg::with_name("output")
            .short("o")
            .long("out")
            .value_name("OUTPUT")
            .help("Sets the output file to write transpiled code to instead of using the input file's name with the extension changed to .js (When set but blank, output is written to stdout; if set to a directory and an input file is provided, the output file will be written to the given directory with the extension changed to .js)")
            .default_value("")
            .takes_value(true)
        )

        .arg(Arg::with_name("INPUT")
            .help("Sets the input file to compile (Leave blank to read from stdin)")
            .index(1)
        )
        .get_matches();

        let (input_file, input_text, input_type) = match matches.value_of("INPUT") {
            Some(value) => (Some(String::from(value)), fs::read_to_string(value).expect("Error reading target file"), Path::new(value).extension().expect("Error getting file extension").to_str().expect("Error getting file extension").to_string()),
            None => {
                let stdin = io::stdin();
                let mut stdin = stdin.lock();
                let mut line = String::new();

                stdin.read_to_string(&mut line).expect("Error reading stdin");
                (None, String::from(line), String::from(""))
            }
        };

        let (use_jsx, jsx_factory, jsx_fragment) = match matches.value_of("jsx") {
            Some("") if matches.occurrences_of("jsx") > 0 => (if matches.occurrences_of("jsx") > 0 {true} else {false}, None, None),
            None | Some("") => (input_type == "tsx", None, None),
            Some(value) => (true, Some(String::from(value)), Some(String::from(matches.value_of("jsx-fragment").unwrap()))),
        };

        let result = compile_typescript(input_text.as_str(), CompileOptions {
            target: String::from(matches.value_of("target").unwrap()),
            module: String::from(matches.value_of("module").unwrap()),
            use_jsx,
            jsx_factory,
            jsx_fragment
        }).expect("Error compiling TypeScript");

        match matches.value_of("output") {
            Some("") if matches.occurrences_of("output") > 0 => print!("{}", result.as_str()),
            None | Some("") => {
                match input_file {
                    Some(input_file) => {
                        let mut path = PathBuf::from(input_file);
                        path.set_extension("js");
                        let mut file = File::create(path).expect("Error creating output file");
                        file.write_all(result.as_bytes()).expect("Error writing to output file");
                    },
                    None => print!("{}", result.as_str())
                }
            }
            Some(path) => {
                let path = if Path::new(path).exists() && fs::metadata(path).expect("Error reading file metadata").is_dir() && input_file.is_some() {
                    let mut path = PathBuf::from(path);
                    path.push(Path::new(&input_file.unwrap().to_string()).file_name().expect("Error getting file name").to_str().expect("Error getting file name"));
                    path.set_extension("js");
                    path
                } else {
                    PathBuf::from(path)
                };

                let mut file = File::create(path).expect("Error creating output file");
                file.write_all(result.as_bytes()).expect("Error writing to output file");
            }
        }
}

struct CompileOptions {
    target: String,
    module: String,
    use_jsx: bool,
    jsx_factory: Option<String>,
    jsx_fragment: Option<String>
}

fn compile_typescript(text: &str, options: CompileOptions) -> Option<String> {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();
    
    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    let ts_compiler = v8::String::new(scope, TYPESCRIPT_SERVICES)?;
    
    let script = v8::Script::compile(scope, ts_compiler, None)?;
    script.run(scope)?;

    let ts_obj_name = v8::String::new(scope, "ts")?.into();
    let ts_obj = context.global(scope).get(scope, ts_obj_name)?;
    
    let transpile_func_name = v8::String::new(scope, "transpile")?.into();
    let transpile_function = ts_obj.to_object(scope)?.get(scope, transpile_func_name)?.to_object(scope)?;
    let transpile_function = v8::Local::<v8::Function>::try_from(transpile_function).ok()?;

    let text = v8::String::new(scope, text)?.into();

    let args = v8::Object::new(scope);

    let target_prop_name = v8::String::new(scope, "target")?.into();
    let target_prop_value = v8::String::new(scope, options.target.as_str())?.into();
    args.set(scope, target_prop_name, target_prop_value);

    let module_prop_name = v8::String::new(scope, "module")?.into();
    let module_prop_value = v8::String::new(scope, options.module.as_str())?.into();
    args.set(scope, module_prop_name, module_prop_value);

    if options.use_jsx {
        let jsx_factory_prop_name = v8::String::new(scope, "jsx")?.into();
        let jsx_factory_prop_value = v8::String::new(scope, if options.jsx_factory.is_some() {"react"} else {"preserve"})?.into();
        args.set(scope, jsx_factory_prop_name, jsx_factory_prop_value);
        
        if options.jsx_factory.is_some() {
             let jsx_factory_prop_name = v8::String::new(scope, "jsxFactory")?.into();
            let jsx_factory_prop_value = v8::String::new(scope, options.jsx_factory.unwrap().as_str())?.into();
            args.set(scope, jsx_factory_prop_name, jsx_factory_prop_value);

            let jsx_fragment_prop_name = v8::String::new(scope, "jsxFragmentFactory")?.into();
            let jsx_fragment_prop_value = v8::String::new(scope, options.jsx_fragment.unwrap().as_str())?.into();
            args.set(scope, jsx_fragment_prop_name, jsx_fragment_prop_value);
        }
    }

    return Some(transpile_function.call(scope, ts_obj, &[text, args.into()])?.to_string(scope)?
        .to_rust_string_lossy(scope))
}