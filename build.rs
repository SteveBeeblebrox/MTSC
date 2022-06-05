use std::io::prelude::*;
use std::fs::File;
use std::env;

#[tokio::main]
async fn main() {
    
    // TypeScript
    download_file(&format!("https://unpkg.com/typescript@{}/lib/typescriptServices.js", env::var_os("CARGO_PKG_VERSION").unwrap().to_string_lossy()), &"src/typescriptServices.js").await;

    // Terser
    download_file(&format!("https://unpkg.com/terser@{}/dist/bundle.min.js", "5.14.0"), &"src/terser.js").await;

    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    println!("cargo:rerun-if-changed=build.rs");
}

async fn download_file(url: &str, path: &str) {
    let response = reqwest::get(url).await.expect(format!("Failed to download {}", url).as_str());
    let content =  response.text().await.expect(format!("Failed to download {}", url).as_str());

    let mut file = File::create(path).expect(format!("Failed to save {} to {}", url, path).as_str());
    file.write_all(content.as_bytes()).expect(format!("Failed to save {} to {}", url, path).as_str());
}