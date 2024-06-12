use std::vec::Vec;
use std::process::exit;

#[allow(unused)]
pub enum MessageType {
    ERROR = 1,
    WARNING = 2,
    EXCEPTION = 3
}

#[cxx::bridge(namespace = "wave")]
mod ffi {
    // Common types

    // Rust types exposed to C++
    extern "Rust" {
        fn callback(message_type: i32, filename: String, line: i32, message: String);
    }

    // C++ types exposed to Rust
    unsafe extern "C++" {
        include!("mtsc/src/features/preprocess/wave.hpp");
        fn preprocess_text(text: String, filename: String, macros: Vec<String>, include_paths: Vec<String>) -> String;
    }
}

fn callback(message_type: i32, filename: String, line: i32, message: String) {
    match message_type {
        i if i == MessageType::ERROR as i32 => {
            eprintln!("\x1b[91mpreprocessor error\x1b[0m: {} ({}:{})", message.trim_start_matches("error: ").trim().split(' ').filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" "), filename, line);
            exit(3);
        },
        i if i == MessageType::WARNING as i32 => eprintln!("\x1b[93mpreprocessor warning\x1b[0m: {} ({}:{})", message.trim_start_matches("warning: ").trim().split(' ').filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" "), filename, line),
        _ => panic!("{} at {filename}:{line}", message.trim_start_matches("error: "))
    };
}

pub fn preprocess_text(text: String, filename: Option<String>, macros: Vec<String>, include_paths: Vec<String>) -> Option<String> {
    return Some(ffi::preprocess_text(text, filename.unwrap_or(String::from("<stdin>")), macros, include_paths));
}