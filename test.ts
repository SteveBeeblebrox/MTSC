type NumberLike = number | string
const x: NumberLike = '7';
function  doStuff(...args: string[]): boolean {
    return +x === 7;
}