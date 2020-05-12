fn is_bit_set(input: u8, mask: u8) -> bool {
    return input & mask != 0;
}

/*
 * Parses a websocket frame.
 */
pub fn parse_frame(content: [u8; 4096], size: usize) -> String {
    let fin_bit: bool = is_bit_set(content[0], 0b10000000);
    let opcode = (content[0] & 0b00001111) as u8;
    let mask_bit: bool = is_bit_set(content[1], 0b10000000);
    let payload_len = (content[1] & 0b01111111) as u8;

    // Next 32-bits define the mask
    let mask:[u8; 4] = [content[2], content[3], content[4], content[5]];

    println!("FIN: {0} OpCode: {1} Mask bit: {2} Payload Len: {3}", fin_bit, opcode, mask_bit, payload_len);
    println!("Mask: {:?}", mask);

    let mut decoded: Vec<u8> = Vec::new();
    // TODO: Handle case where content length > 126
    for i in 0..size-6 {
        decoded.push(content[6+i] ^ mask[i % 4]);
    }

    let result = std::str::from_utf8(&decoded).expect("Error decoding websocket payload.");
    println!("Decoded: {0}", result);

    return String::from(result);
}