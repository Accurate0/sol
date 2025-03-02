let mut this_is_an_array = ["test", "test2"];
let mut index = 0;
loop {
    print(this_is_an_array[index]);
    index = index + 1;
    if index >= 2 {
        break;
    }
}
