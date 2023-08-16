use std::vec::Vec;
use std::ffi::CString;

include!("ffi.rs");

type MessageCallback = fn(message_type: MessageType, filename: String, line: i32, message: String);

pub fn preprocess_text(text: String, filename: String, mode: Mode, macros: Vec<String>, on_message: MessageCallback) -> Option<String> {
    unsafe {
        let c_text = CString::new(text).unwrap();
        let c_filename = CString::new(filename).unwrap();
        let c_macros = macros.iter().map(|m| CString::new(&(*m.clone())).unwrap()).collect::<Vec<CString>>();

        preprocess_text_ffi(
            c_text.as_ptr(),
            c_filename.as_ptr(),
            mode as i32,
            macros.len(),
            (&c_macros.iter().map(|m| m.as_ptr()).collect::<Vec<cstr>>())[..].as_ptr(),
            |message_type: i32, p_filename: cstr, line: i32, p_message: cstr| {
                println!("Got message!");
                // TODO: fix type
                //on_message(MessageType::ERROR, String::from(CStr::from_ptr(p_filename).to_str().unwrap()), line, String::from(CStr::from_ptr(p_message).to_str().unwrap()));
            }
        );
        Some(String::from(""))
    }
}

// fn agg<'a, F>(col: F) -> Box<FnMut(&CowRow) -> i32 + 'a> where F: Fn(&CowRow) -> i32 + 'a
// {
    
// }