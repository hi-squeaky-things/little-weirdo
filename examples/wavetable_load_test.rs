


fn main() {
  let test =  include_bytes!("../data/soundbank_0/src/wav0.lwt");


  println!("test {:?}", i16::from_le_bytes([test[0],test[1]]));
}