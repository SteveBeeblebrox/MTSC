#include <stdint.h>

enum Mode {
    NONE = -1,
    STANDARD = 0,
    COMMENT = 1
};

enum MessageType {
    ERROR = 1,
    WARNING = 2,
    EXCEPTION = 3
};

typedef const char* cstr;
typedef int32_t i32;
typedef void (*message_callback_ptr)(i32 message_type, cstr p_filename, i32 line, cstr p_message);

extern "C" {
    cstr preprocess_text_ffi(cstr p_text, cstr p_filename, i32 mode, size_t macro_count, cstr* const p_macros, message_callback_ptr p_on_message);
    void free_preprocess_result_ffi(cstr result);
}