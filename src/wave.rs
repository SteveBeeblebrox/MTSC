use std::vec::Vec;

#[derive(Clone)]
pub enum Mode {
    NONE = -1,
    STANDARD = 0,
    COMMENT = 1
}

pub enum MessageType {
    ERROR = 1,
    WARNING = 2,
    EXCEPTION = 3
}

#[cxx::bridge(namespace = "wave")]
mod ffi {
    // Common types

    // Rust types exposed to C++
    extern "Rust" {}

    // C++ types exposed to Rust
    unsafe extern "C++" {
        include!("mtsc/src/wave.h");
        fn preprocess_text(text: String, filename: String, macros: Vec<String>) -> String;
    }
}


// fn callback(message_type: MessageType, filename: String, line: i32, message: String) {
//     match message_type {
//         MessageType::EXCEPTION => panic!("{}", message),
//         MessageType::ERROR => {
//             eprintln!("\x1b[91mpreprocessor error\x1b[0m: {} ({}:{})", message, filename, line);
//             exit(1); // use std::process::exit
//         },
//         MessageType::WARNING => eprintln!("\x1b[93mpreprocessor warning\x1b[0m: {} ({}:{})", message, filename, line)
//     };
// }

// // FIXME: line numbers are wrong with comment preprocessor
pub fn preprocess_text(text: String, filename: String, mode: Mode, macros: Vec<String>) -> Option<String> {
    return Some(ffi::preprocess_text(text,filename,macros));
}