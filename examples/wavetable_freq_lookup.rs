fn main() {
    let sample_rate: i16 = 22050;
    
    for i in  121..271 {
        let freq:i16 = i as i16;
        let loop_t:i16 = sample_rate / freq;
        let key_inc:f32 = 600.0 / (loop_t-1) as f32;
        
        print!("pub static FREQ_{}: [i16; {}] = [", freq, loop_t);
    //    println!("  &FREQ_{}[0..],", freq);
        for i in 0..loop_t {  
           // println!("{} {} {} {} {}", i, loop_t, key_inc, (key_inc * i as f32), (key_inc * i as f32) as i16 );
           print!("{},", (key_inc * i as f32) as i16 );
        }
       println!("];");
    }
}