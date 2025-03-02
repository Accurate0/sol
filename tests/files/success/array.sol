let mut this_is_an_array = [2 + 2, 2 + 1, 2 * 1, 1 + 0];
let mut index = 0;
loop {
    print(this_is_an_array[index]);
    index = index + 1;
    if index >= 4 {
        break;
    }
}
