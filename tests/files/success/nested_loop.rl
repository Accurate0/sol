let mut x = 0;
loop {
    let mut y = 0;
    loop {
        if y > 3 {
            print("exit loop");
            break;
        }

        y = y + 1;
        print(y);
    }

    if x > 3 {
        print("exit loop");
        break;
    }

    x = x + 1;
    print(x);
}
