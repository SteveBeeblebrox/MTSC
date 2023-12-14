use std::vec::Vec;
use std::ffi::{CString,CStr};
use std::process::exit;

include!("ffi.rs");

fn callback(message_type: MessageType, filename: String, line: i32, message: String) {
    match message_type {
        MessageType::EXCEPTION => panic!("{}", message),
        MessageType::ERROR => {
            eprintln!("\x1b[91mpreprocessor error\x1b[0m: {} ({}:{})", message, filename, line);
            exit(1);
        },
        MessageType::WARNING => eprintln!("\x1b[93mpreprocessor warning\x1b[0m: {} ({}:{})", message, filename, line)
    };
}

unsafe extern "C" fn callback_ffi(message_type: i32, p_filename: cstr, line: i32, p_message: cstr) {
    let message_type_enum = match message_type {
        i if i == MessageType::WARNING as i32 => MessageType::WARNING,
        i if i == MessageType::ERROR as i32 => MessageType::ERROR,
        _ => MessageType::EXCEPTION
    };
    callback(message_type_enum, String::from(CStr::from_ptr(p_filename).to_str().unwrap()), line, String::from(CStr::from_ptr(p_message).to_str().unwrap()));
}

// FIXME: line numbers are wrong with comment preprocessor
pub fn preprocess_text(text: String, filename: String, mode: Mode, macros: Vec<String>) -> Option<String> {
    unsafe {
        let c_text = CString::new(text).unwrap();
        let c_filename = CString::new(filename).unwrap();
        let c_macros = macros.iter().map(|m| CString::new(&(*m.clone())).unwrap()).collect::<Vec<CString>>();

        let p_result = preprocess_text_ffi(
            c_text.as_ptr(),
            c_filename.as_ptr(),
            mode as i32,
            macros.len(),
            (&c_macros.iter().map(|m| m.as_ptr()).collect::<Vec<cstr>>())[..].as_ptr(),
            callback_ffi
        );

        let result: String = String::from(CStr::from_ptr(p_result).to_str().unwrap());

        free_preprocess_result_ffi(p_result);

        Some(result)
    }
}