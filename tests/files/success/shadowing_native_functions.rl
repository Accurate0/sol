let x = 3;
let y = 4;
let z = x + y;

fn print() {
}

fn test(a: int) {
    let y = 1.3 + a;
    print("y: ", y);

    {
        let z = y + 3;
        print("z: ", z);
    }

    let z = y + 2;
    print("z: ", z);
}

fn main() {
    let x = 1.3 + 3;
    {
        print("Hello");
        print("x: ", x);
        let y = test(4);
        print("y: ", y);
    }

    print(x);
}


main();
