use downloader::Downloader;
use semver::Version;
use clap::{Arg, App};

use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::fs::File;
use std::fs;

use std::error::Error;
use std::convert::TryFrom;

fn main() {
    let matches = App::new("MTSC")
        .version(clap::crate_version!())
        .author("S B. <@gmail.com>")
        .about("A standalone TypeScript compiler")

        .arg(Arg::with_name("install")
            .short("i")
            .long("install")
            .value_name("TS-VERSION")
            .help("Install a compiler version")
            //.default_value("latest")
            .takes_value(true)
        )
        
        .arg(Arg::with_name("compiler")
            .short("c")
            .long("compiler")
            .value_name("COMPILER")
            .help("Sets the compiler version to use (default latest)")
            //.default_value("latest")
            .takes_value(true)
        )

        .arg(Arg::with_name("target")
            .short("t")
            .long("target")
            .value_name("ES-VERSION")
            .help("The JavaScript language and library version for compiled code")
            .default_value("esnext")
            .takes_value(true)
        )

        .arg(Arg::with_name("module")
            .short("m")
            .long("module")
            .value_name("MODULE-VERSION")
            .help("The module type for compiled code")
            .default_value("esnext")
            .takes_value(true)
        )

        .arg(Arg::with_name("INPUT")
            .help("Sets the input file to use")
            .index(1)
        )
        .get_matches();

    match matches.value_of("install") {
        Some(version) => {
            println!("Installing version {}. This will take a moment.", version);
            match install(version) {
                Ok(_) => (),
                Err(e) => eprintln!("{:?}", e)
            }
        },
        None => (),
    }

    let compiler = matches.value_of("compiler").unwrap_or("latest");
    println!("Compiler: {}", compiler);

    match matches.value_of("INPUT") {
        Some(target) => {
            match compile(compiler, target) {
                Ok(_) => (),
                Err(e) =>  {
                    eprintln!("{:?}", e);
                    return
                }
            }
        },
        None => (),
    }
}

const CACHE: &str = "./.mtsc";

fn check_version(text: &str) -> Result<&str, Box<dyn Error>> {
    match text {
        "latest" => Ok(text),
        _ => match Version::parse(text) {
            Err(e) => return Err("Install target is not a valid version.".into()),
            Ok(version) => Ok(text)

        }
    }
}

fn compile<'a>(compiler: &'a str, target: &'a str) -> Result<(), Box<dyn Error>> {
    match fs::read_to_string(Path::new(CACHE).join(Path::new(compiler))) {
        Ok(compilerSrc) => {

            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();

            let isolate = &mut v8::Isolate::new(Default::default());

            let scope = &mut v8::HandleScope::new(isolate);
            let context = v8::Context::new(scope);
            let scope = &mut v8::ContextScope::new(scope, context);

            let code = v8::String::new(scope, compilerSrc.as_str()).unwrap();
            
            let script = v8::Script::compile(scope, code, None).unwrap();
            let result = script.run(scope).unwrap();
            let result = result.to_string(scope).unwrap();

            let n = v8::String::new(scope, "ts").unwrap().into();
            let ts =
            context
            .global(scope)
            .get(scope, n)
            .expect("missing function Process");
            
let n = v8::String::new(scope, "transpile").unwrap().into();

            let transpile = ts.to_object(scope).unwrap()
            .get(scope, n)
            .expect("missing function Process").to_object(scope).unwrap();

            let transpile_function = v8::Local::<v8::Function>::try_from(transpile)
            .expect("function expected");

            let n = v8::String::new(scope, fs::read_to_string("test.ts").expect("Some error").as_str()).unwrap().into();

            let javascript = transpile_function
            .call(scope, 
            ts,
                &[n]
            ).unwrap().to_string(scope).unwrap().to_rust_string_lossy(scope);

            let mut path = PathBuf::from(target);
            path.set_extension("js");
            let mut file = File::create(path).unwrap();
            file.write_all(javascript.as_bytes());
            Ok(())
        },
        Err(e) => return Err(format!("Unable to read input file ({}).", e.to_string()).into())
    }
}

fn install(version: &str) -> Result<(), Box<dyn Error>> {
    check_version(version)?;

    match fs::create_dir_all(CACHE) {
        Ok(_) => (),
        Err(e) => return Err(format!("Unable to create cache directory ({}).", e.to_string()).into())
    }

    if version == "latest" && Path::new(CACHE).join(Path::new(version)).exists() {
        fs::remove_file(Path::new(CACHE).join(Path::new(version)));
    }

    if Path::new(CACHE).join(Path::new(version)).exists() {
        return Err("Install target already exists.".into())
    }

    let mut downloader = Downloader::builder()
        .download_folder(std::path::Path::new(CACHE))
        .parallel_requests(1)
        .retries(1)
        .build()
        .unwrap();

    let dl = downloader::Download::new(format!("https://unpkg.com/typescript@{}/lib/typescriptServices.js", version).as_str())
        .file_name(std::path::Path::new(version));

    match downloader.download(&[dl]) {
        Ok(results) => {
            for result in results {
                match result {
                    Ok(_) => (),
                    Err(e) => {
                        fs::remove_file(Path::new(CACHE).join(Path::new(version)));
                        return Err(format!("Error downloading install target ({}).", e.to_string()).into())
                    }
                };
            }
        },
        Err(e) => {
            fs::remove_file(Path::new(CACHE).join(Path::new(version)));
            return Err(format!("Error downloading install target ({}).", e.to_string()).into())
        }
    }

    Ok(())
}