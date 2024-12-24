use hound::WavReader;




fn main() {
  let test =  include_bytes!("../data/soundbank_0/src/wav4.lwt");

  let mut open = WavReader::open("./data/soundbank_0/original/sinus.wav").unwrap();
  println!("Amount of sample = {:?}", open.duration());
  let mut counter = 0;
  for sample in open.samples::<i16>()
      .into_iter() {
          let output = sample.unwrap();
       //   println!("wav = {:?} ", output);
          let x = i16::from_le_bytes([test[counter*2],test[counter*2 + 1]]);
        //  println!("array = {:?} ", x);
          if output != x {
            println!("error");
          }
          counter = counter + 1;
      }

   
}