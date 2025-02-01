fn main() {
    let mut test:u8 = 0b1100_0011;
    println!("O {:b}", test);
    test = test | 1 << 4;
    println!("S {:b}", test);
    test = test & !(1 << 4);
    println!("U {:b}", test);
    
}