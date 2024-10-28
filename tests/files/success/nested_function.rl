fn main() {
    fn nested() {
        fn nested2() {
            print("pass 2");
        }

        fn nested3() {
            print("pass 3");
        }

        print("pass");
        nested2();
    }

    nested();
    nested3();
}

main();
