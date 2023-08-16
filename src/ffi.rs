use std::os::raw::c_char;

#[derive(Copy)]
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

#[allow(non_camel_case_types)]
type cstr = *const c_char;

#[allow(non_camel_case_types)]
type message_callback_ptr = unsafe extern "C" fn(message_type: i32, p_filename: cstr, line: i32, p_message: cstr);

extern "C" {
    fn preprocess_text_ffi(p_text: cstr, p_filename: cstr, mode: i32,  macro_count: usize, p_macros: *const cstr, p_on_message: message_callback_ptr) -> cstr;
    fn free_preprocess_result_ffi(result: cstr);
}