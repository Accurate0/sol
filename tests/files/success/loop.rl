let mut x = 0;
loop {
    if x > 3 {
        print("exit loop");
        break;
    }

    x = x + 1;
    print(x);
}
