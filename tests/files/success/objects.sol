let y = 3;

let another_object = {
    inner_value: 32,
};

let x = {
    test: 1,
    test2: "testing",
    test3: y,
    test4: another_object,
    test5: {
        test6: {
            test7: 1999
        }
    }
};

print(x);
print(x.test);
print(x.test2);
print(x.test3);
print(x.test4);
print(x.test4.inner_value);
print(x.test5);
print(x.test5.test6);
print(x.test5.test6.test7);
