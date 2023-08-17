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

FOO

// #warning Oh noo!
//x#error ouch
1+1

//#define X(a) 1
//X(1,2)