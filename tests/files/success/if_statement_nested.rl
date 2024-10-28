if true {
    if true {
        if false {
            print("fail");
        } else if true {
            print("pass");
        }

        if false {
            print("fail");
        } else if false {
            print("fail");
        } else {
            print("pass");
        }
    }
} else if true {
    print("fail");
} else {
    print("fail");
}
