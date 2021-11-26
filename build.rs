use std::io::prelude::*;
use std::fs::File;
use std::env;

#[tokio::main]
async fn main() {
    let version = env::var_os("CARGO_PKG_VERSION").unwrap();
    
    let response = reqwest::get(format!("https://unpkg.com/typescript@{}/lib/typescriptServices.js", version.to_string_lossy()).as_str()).await.expect(format!("Failed to download TypeScript ({})", version.to_string_lossy()).as_str());
    let content =  response.text().await.expect(format!("Failed to download TypeScript ({})", version.to_string_lossy()).as_str());

    let mut file = File::create("src/typescriptServices.js").expect(format!("Failed to download TypeScript ({})", version.to_string_lossy()).as_str());
    file.write_all(content.as_bytes()).expect(format!("Failed to download TypeScript ({})", version.to_string_lossy()).as_str());
    
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    println!("cargo:rerun-if-changed=build.rs");
}