use std::io::prelude::*;
use std::fs::File;
use std::env;

use cxx_build;

#[tokio::main]
async fn main() {
    
    // TypeScript
    download_file(&format!("https://unpkg.com/typescript@{}/lib/typescript.js", env::var_os("CARGO_PKG_VERSION").unwrap().to_string_lossy()), &"src/typescript.js").await;

    // Terser
    download_file(&format!("https://unpkg.com/terser@{}/dist/bundle.min.js", "5.19.2"), &"src/terser.js").await;

    // Wave
    compile_wave();

    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    println!("cargo:rerun-if-changed=src/wave.cpp");
    println!("cargo:rerun-if-changed=build.rs");
}

async fn download_file(url: &str, path: &str) {
    let response = reqwest::get(url).await.expect(format!("Failed to download {}", url).as_str());
    let content =  response.text().await.expect(format!("Failed to download {}", url).as_str());

    let mut file = File::create(path).expect(format!("Failed to save {} to {}", url, path).as_str());
    file.write_all(content.as_bytes()).expect(format!("Failed to save {} to {}", url, path).as_str());
}

fn compile_wave() {
    cxx_build::bridge("src/wave.rs")
        .cpp(true).warnings(false)
        .file("src/wave.cpp")
        .static_flag(true)
        .flag_if_supported("-std=c++11")
        .compile("cxxbridge-wave");

    println!("cargo:rustc-link-lib=static=boost_atomic");
    println!("cargo:rustc-link-lib=static=boost_regex");
    println!("cargo:rustc-link-lib=static=boost_chrono");
    println!("cargo:rustc-link-lib=static=boost_filesystem");
    println!("cargo:rustc-link-lib=static=boost_thread");
    println!("cargo:rustc-link-lib=static=boost_wave");
}
