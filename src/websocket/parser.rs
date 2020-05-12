/*
 * Parses a websocket frame.
 */
pub fn parse_frame(content: [u8; 4096], size: usize) -> String {
    let _fin_bit: bool = (content[0] & 0b10000000) != 0; // Bit 0 has fin bit
    let _rsv1: bool = (content[0] & 0b01000000) != 0; // Bit 1 contains reserved flag 1
    let _rsv2: bool = (content[0] & 0b00100000) != 0; // Bit 2 contains reserved flag 2
    let _rsv3: bool = (content[0] & 0b00010000) != 0; // Bit 3 contains reserved flag 3
    let _opcode = (content[0] & 0b00001111) as u8; // Bits 4 - 7 contain opcode (1-3 are reserved)
    let _mask_bit: bool = (content[1] & 0b10000000) != 0; // Bit 8 contains mask flag
    let _payload_len = (content[1] & 0b01111111) as u8; // Bits 9 - 15 contain payload length

    // TODO: Handle case where payload length > 126

    // Next 32-bits define the mask (in short payload case)
    let mask:[u8; 4] = [content[2], content[3], content[4], content[5]];

    // Decode payload content (XOR payload bits with mask bits)
    let mut decoded: Vec<u8> = Vec::new();
    for i in 0..size-6 {
        decoded.push(content[6+i] ^ mask[i % 4]); // 32 mask bits are used repeatedly
    }

    // Convert decoded payload into string
    let result = std::str::from_utf8(&decoded).expect("Error decoding websocket payload.");
    
    return String::from(result);
}