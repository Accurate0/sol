// const constant_value = 3;

let x = 3;
let y = 4;
let z = x + y;

fn print() {}

fn test(a) {
    let y = 1.3 + a;
    {
        let z = y + 3;
    }

    let z = y + 2;

//    return z;
}

fn main() {
    let x = 1.3 + 3;
    {
	print("Hello");
	print(x);
	let y = test(4);
    }

    print(x);
}


main();
