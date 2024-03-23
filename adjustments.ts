// See wave.cpp
// \u{fffe}0;       Scope: apply_input_adjustment (marks escaped newlines)
// \u{ffff}17;      Scope: preprocess_text (escape normal hashtags like those in private properties)
// \u{ffff}91;      Scope: preprocess_text (mark split lines)

function apply_input_adjustment(text: string): string {
    // handle #! here
    return text
        .replace(/\\\r?\n/gum, '\u{fffe}0;') // Unescape newlines
        /////////////////////////////////
        
        .replace(/^(?=\s*?#)/gum,'\u{ffff}17;') // Block normal #...
        .replace(/^(\s*?)\/\/\/(?=\s*?#)/gum,'$1\u{ffff}91;\n') // Split ///#...

        /////////////////////////////////
        .replace(/\u{fffe}0;/gum, '\\\n') // Re-escape newlines

        + "\n"; // Add an extra \n to the end; wave fails on a trailing comment
}


function apply_output_adjustment(text: string): string {
    return text
        .replace(/\u{ffff}17;/gum,'') // Unblock normal #...
        .replace(/\u{ffff}91;\r?\n(?=\s*?#)/gum, '///') // Unsplit unused ///#
        .replace(/\u{ffff}91;\r?\n/gum, '') // Discard used ///# splits
}