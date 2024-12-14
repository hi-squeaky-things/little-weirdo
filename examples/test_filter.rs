use std::{f32::consts::PI, i16, str::MatchIndices};


const FX_SHIFT: u32 = 16;
const SHIFTED_1: u16 = u16::MAX;

fn ucfxmul(a: u16, b: u16) -> u32 {
    (a as u32* b as u32)>>FX_SHIFT
}

fn ifxmul(a: i32, b: u16) -> i32 {
    (a * b as i32)>>FX_SHIFT
}

fn fxmul(a: i64, b: i32) -> i64 {
    (a * b as i64) >> FX_SHIFT
}


fn main() {
  let q:u16 = 300;
  let f:u16 = 100;
  let fb:u32 = q as u32 + ucfxmul(q, SHIFTED_1 - f);
  println!("feedback {:?}", fb);
  let mut buf0:i64 = 0;
  let mut buf1:i64 = 0;
  
  let sample:i32 = -23232;

  let x = fxmul(fb as i64, 0);
  buf0 = buf0 + fxmul(( (sample as i64 - buf0) + x), f as i32);
  buf1 = buf1 + ifxmul((buf0-buf1) as i32, f) as i64;
  
  println!("buf0 {:?}", buf0);
  println!("buf1 {:?}", buf1);
  

}



