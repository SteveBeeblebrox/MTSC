use std::path::Path;
use downloader::{Download, Downloader};
use std::env;

fn main() {
    let version = env::var_os("CARGO_PKG_VERSION").unwrap();
    
    let mut downloader = Downloader::builder()
        .download_folder(Path::new("src"))
        .parallel_requests(1)
        .retries(1)
        .build()
        .unwrap();

    let typescript_services = Download::new(format!("https://unpkg.com/typescript@{}/lib/typescriptServices.js", version.to_string_lossy()).as_str());
    downloader.download(&[typescript_services]).expect(format!("Failed to download TypeScript ({})", version.to_string_lossy()).as_str());

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=build.rs");
}