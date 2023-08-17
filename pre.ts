declare function ASSERT(condition: boolean);
//#define ASSERT(condition) assert(condition, #condition)

function assert(condition: boolean, message: string) {
    if(!condition) {
        throw new Error(message);
    }
}

let x = 1;

ASSERT(x == 1)

//#include "strc.ts"
strc(1 + 1)