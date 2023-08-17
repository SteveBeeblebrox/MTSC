use std::vec::Vec;
use std::ffi::{CString,CStr};

include!("ffi.rs");

fn callback(message_type: MessageType, filename: String, line: i32, message: String) {
    match message_type {
        MessageType::EXCEPTION => panic!("{}", message),
        MessageType::ERROR => eprintln!("\x1b[91mpreprocessor error\x1b[0m: {} ({}:{})", message, filename, line),
        MessageType::WARNING => eprintln!("\x1b[93mpreprocessor warning\x1b[0m: {} ({}:{})", message, filename, line)
    };
    // use #line 1 "file" to set file when reading from stdin, use - or something else unique as the original name
    // FIXME: line numbers are wrong with comment preprocessor}
}

unsafe extern "C" fn callback_ffi(message_type: i32, p_filename: cstr, line: i32, p_message: cstr) {
    let message_type_enum = match message_type {
        i if i == MessageType::WARNING as i32 => MessageType::WARNING,
        i if i == MessageType::ERROR as i32 => MessageType::ERROR,
        _ => MessageType::EXCEPTION
    };
    callback(message_type_enum, String::from(CStr::from_ptr(p_filename).to_str().unwrap()), line, String::from(CStr::from_ptr(p_message).to_str().unwrap()));
}

pub fn preprocess_text(text: String, filename: String, mode: Mode, macros: Vec<String>) -> Option<String> {
    unsafe {
        let c_text = CString::new(text).unwrap();
        let c_filename = CString::new(filename).unwrap();
        let c_macros = macros.iter().map(|m| CString::new(&(*m.clone())).unwrap()).collect::<Vec<CString>>();

        let result = preprocess_text_ffi(
            c_text.as_ptr(),
            c_filename.as_ptr(),
            mode as i32,
            macros.len(),
            (&c_macros.iter().map(|m| m.as_ptr()).collect::<Vec<cstr>>())[..].as_ptr(),
            callback_ffi
        );

        free_preprocess_result_ffi(result);
        Some(String::from(""))
    }
}