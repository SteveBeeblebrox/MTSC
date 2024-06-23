use std::io::prelude::*;
use std::fs::File;
use std::env;

use cxx_build;

macro_rules! cargo {
    ($expression:expr$(,)?) => {
        println!("cargo:{}", $expression)
    };
    ($lhs:expr, $rhs:expr$(, $expressions:tt)*$(,)?) => {
        cargo!(concat!($lhs,"=",$rhs)$(, $expressions)*)
    };
}

#[tokio::main]
async fn main() {
    // TypeScript
    #[cfg(feature = "transpile")]
    download_file(&format!("https://unpkg.com/typescript@{}/lib/typescript.js", env::var_os("CARGO_PKG_VERSION").unwrap().to_string_lossy()), &"src/features/transpile/typescript.js").await;

    // Terser
    #[cfg(feature = "minify")]
    download_file(&format!("https://unpkg.com/terser@{}/dist/bundle.min.js", "5.19.2"), &"src/features/minify/terser.js").await;

    // Wave
    #[cfg(feature = "preprocess")]
    compile_wave();

    #[cfg(any(feature = "transpile", feature = "compile"))]
    cargo!("rerun-if-env-changed", "CARGO_PKG_VERSION");
    
    #[cfg(feature = "preprocess")]
    cargo!("rerun-if-changed", "src/features/preprocess/");
    
    cargo!("rerun-if-changed", "build.rs");
}

#[cfg(feature = "common")]
async fn download_file(url: &str, path: &str) {
    let response = reqwest::get(url).await.expect(format!("Failed to download {}", url).as_str());
    let content =  response.text().await.expect(format!("Failed to download {}", url).as_str());

    let mut file = File::create(path).expect(format!("Failed to save {} to {}", url, path).as_str());
    file.write_all(content.as_bytes()).expect(format!("Failed to save {} to {}", url, path).as_str());
}

#[cfg(feature = "preprocess")]
fn compile_wave() {
    cxx_build::bridge("src/features/preprocess/wave.rs")
        .cpp(true).warnings(false)
        .file("src/features/preprocess/wave.cpp")
        .define("MTSC_VERSION", format!("\"{}\"",env::var_os("CARGO_PKG_VERSION").unwrap().to_string_lossy()).as_str())
        .static_flag(true)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-fno-access-control")
        .include("src/features/preprocess/patch/")
        .compile("cxxbridge-wave");

    cargo!("rustc-link-search","/usr/local/lib/");
    cargo!("rustc-link-lib","static","boost_atomic");
    cargo!("rustc-link-lib","static","boost_regex");
    cargo!("rustc-link-lib","static","boost_chrono");
    cargo!("rustc-link-lib","static","boost_filesystem");
    cargo!("rustc-link-lib","static","boost_thread");
    cargo!("rustc-link-lib","static","boost_wave");
}
