fn test(x: bool) {
    print(x);
    return true;
}

let x = test("testing");

if x {
    print("pass");
} else {
    print("fail");
}

if test() {
    print("pass");
} else {
    print("fail");
}

fn test2(x: string) {
    print(x);
}

print(test2("pass"));

let x = test("wow");
print(x);

print(test("wow"));
