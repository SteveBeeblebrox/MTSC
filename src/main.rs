use clap::{Arg, App};

use std::convert::TryFrom;
use std::io::prelude::*;
use std::path::PathBuf;
use std::fs::File;
use std::fs;

static TYPESCRIPT_SERVICES: &str = include_str!(r"typescriptServices.js");
fn main() {
    let matches = App::new("MTSC")
        .version(clap::crate_version!())
        .author("S B. <@gmail.com>")
        .about("A standalone TypeScript compiler")
/*
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
        )*/

        .arg(Arg::with_name("output")
            .short("o")
            .long("out")
            .value_name("OUTPUT")
            .help("Sets the output file to write transpiled code to (Leave blank to write to the console)")
            .takes_value(true)
        )

        .arg(Arg::with_name("INPUT")
            .help("Sets the input file to compile")
            .index(1)
        )
        .get_matches();

        let input_file = matches.value_of("INPUT").expect("Input required");
        // TODO: Make error message
        let input_text = fs::read_to_string(input_file).expect("TODO");

        // TODO: Make error message
        let result = compile_typescript(input_text.as_str()).expect("Compile Error");

        match matches.value_of("output") {
            Some("") => print!("{}", result.as_str()),
            Some(path) => {
                // TODO: Make error message
                let mut file = File::create(path).expect("Error");
                // TODO: Make error message
                file.write_all(result.as_bytes()).expect("Error");
            },
            None => {
                let mut path = PathBuf::from(input_file);
                path.set_extension("js");
                // TODO: Make error message
                let mut file = File::create(path).expect("Error");
                // TODO: Make error message
                file.write_all(result.as_bytes()).expect("Error");
            }
        }
}

fn compile_typescript(text: &str) -> Option<String> {
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

    return Some(transpile_function.call(scope, ts_obj, &[text])?.to_string(scope)?
        .to_rust_string_lossy(scope))
}