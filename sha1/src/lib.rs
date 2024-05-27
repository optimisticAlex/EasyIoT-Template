#![no_std]


pub fn compute_sha1(input: &[u8]) -> [u8; 20] {
    let mut hash: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
    let mut res = [0u8; 20];
    let mut block = [0u8; 64];
    let mut idx = 0;
    //process complete blocks
    while input.len() - idx >= 64{
        for i in 0..64{
            block[i] = input[idx+i];
        }
        sha1_process_block(block, &mut hash);
        idx += 64;
    }
    //last block with padding
    for i in 0..(input.len() - idx){
        block[i] = input[idx+i];
    }
    sha1_pad_block(&mut block, input.len()-idx, (input.len()*8)as u64, &mut hash);
    //write hash to result
    for i in 0..20{
        res[i] = (hash[i>>2] >> 8 * ( 3 - ( i & 0x03 ) )) as u8;
    }
    res
}

fn sha1_process_block(block: [u8; 64], hash: & mut [u32; 5]){
    let mut abcde = hash.clone(); /* Word buffers */
    let mut w = [0u32; 80]; /* Word sequence */
    let mut temp: u32; /* Temporary word valu */

    for t in 0..16{/* Initialize the first 16 words in the array w */
        w[t] = (block[t * 4] as u32) << 24;
        w[t] |= (block[t * 4 + 1] as u32) << 16;
        w[t] |= (block[t * 4 + 2] as u32) << 8;
        w[t] |= block[t * 4 + 3] as u32;
    }
    for t in 16..80{/* Initialize the rest */
        w[t] = (w[t-3] ^ w[t-8] ^ w[t-14] ^ w[t-16]).rotate_left(1);
    }
    for t in 0..80usize{ /* compute hash */
        temp = abcde[0].rotate_left(5)
                       .wrapping_add(sha1_f(t, abcde[1], abcde[2], abcde[3]))
                       .wrapping_add(abcde[4])
                       .wrapping_add(w[t])
                       .wrapping_add(sha1_k(t));
        abcde[4] = abcde[3];
        abcde[3] = abcde[2];
        abcde[2] = abcde[1].rotate_left(30);
        abcde[1] = abcde[0];
        abcde[0] = temp;
    }
    for i in 0..5{ /* update hash */
        hash[i] = hash[i].wrapping_add(abcde[i]);
    }
}

fn sha1_f(t: usize, b: u32, c: u32, d: u32) -> u32{
    match t{
        0..=19 => (b & c) | ((!b) & d),
        20..=39 => b ^ c ^ d,
        40..=59 => (b & c) | (b & d) | (c & d),
        60..=79 => b ^ c ^ d,
        _ => 0
    }
}

fn sha1_k(t: usize) -> u32{
    match t{
        0..=19 => 0x5A827999,
        20..=39 => 0x6ED9EBA1,
        40..=59 => 0x8F1BBCDC,
        60..=79 => 0xCA62C1D6,
        _ => 0
    }
}

fn sha1_pad_block(block: & mut [u8; 64], start: usize, len: u64, hash: &mut [u32; 5]){
    block[start] = 0b10000000;
    for i in start+1..block.len(){
        block[i] = 0;
    }

    if start >= 56{//if len does not fit in the current block
        sha1_process_block(*block, hash);
        for i in 0..block.len(){
            block[i] = 0;
        }
    }

    block[56] = (len >> 56) as u8;
    block[57] = (len >> 48) as u8;
    block[58] = (len >> 40) as u8;
    block[59] = (len >> 32) as u8;
    block[60] = (len >> 24) as u8;
    block[61] = (len >> 16) as u8;
    block[62] = (len >> 8) as u8;
    block[63] = (len >> 0) as u8;
    sha1_process_block(*block, hash);
}


//helper functions

pub fn sha1_str_2_bytes(input: & str) -> [u8; 20]{
    if input.len() != 40{
        panic!("Invalid input length, expected 40 hex-characters.");
    }
    let mut res = [0u8; 20];
    for i in 0..20{
        res[i] = u8::from_str_radix(&input[i*2..i*2+2], 16).unwrap();
    }
    res
}

pub fn sha1_bytes_2_str(input: &[u8; 20]) -> heapless::String<40>{
    use core::fmt::Write;
    let mut res = heapless::String::<40>::new();
    for &byte in input.iter().take(20) {
        write!(res, "{:02x}", byte).unwrap();
    }
    res
}

pub fn sha1_bytes_2_base64(input: &[u8; 20]) -> heapless::String<28>{
    let mut res = heapless::String::<28>::new();
    let base64_table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for i in 0..6{
        res.push(base64_table[(input[i*3] >> 2) as usize] as char).unwrap();
        res.push(base64_table[(((input[i*3] & 0b11) << 4) | (input[i*3+1] >> 4)) as usize] as char).unwrap();
        res.push(base64_table[(((input[i*3+1] & 0b1111) << 2) | (input[i*3+2] >> 6)) as usize] as char).unwrap();
        res.push(base64_table[(input[i*3+2] & 0b111111) as usize] as char).unwrap();
    }
    res.push(base64_table[(input[18] >> 2) as usize] as char).unwrap();
    res.push(base64_table[(((input[18] & 0b11) << 4) | (input[19] >> 4)) as usize] as char).unwrap();
    res.push(base64_table[((input[19] & 0b1111) << 2) as usize] as char).unwrap();
    res.push('=').unwrap();
    res
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sec_websocket_accept() {
        let expected_sha1: [u8; 20] = [0xb3, 0x7a, 0x4f, 0x2c, 0xc0, 0x62, 0x4f, 0x16, 0x90, 0xf6, 0x46, 0x06, 0xcf, 0x38, 0x59, 0x45, 0xb2, 0xbe, 0xc4, 0xea];
        assert_eq!( expected_sha1, compute_sha1("dGhlIHNhbXBsZSBub25jZQ==258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes()));
    }

    //TODO: add the tests from the rfc sha1 c example
}
