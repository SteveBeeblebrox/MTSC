#pragma once
#include <string>
#include <vector>

#include <stdint.h>
typedef int32_t i32;

#include "rust/cxx.h"
#include "mtsc/src/features/preprocess/wave.rs.h"

namespace wave {

    enum MessageType {
        ERROR = 1,
        WARNING = 2,
        EXCEPTION = 3
    };

    rust::String preprocess_text(rust::String text, rust::String filename, const rust::Vec<rust::String> MACROS, const rust::Vec<rust::String> INCLUDE_PATHS);
}