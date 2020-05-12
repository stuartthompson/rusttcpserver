# Rust TCP Server

A simple Rust TCP server for me to learn with.


### Decoding Websocket Packets

```javascript
/*
 * Decodes "Hello" from a websocket packet frame
 *
 * Websocket packets are not just plain text.
 */

// [Server] Received 11 bytes.
// Byte  0 is 129: 10000001
// Byte  1 is 133: 10000101
// Byte  2 is 100: 01100100
// Byte  3 is  10: 00001010
// Byte  4 is  47: 00101111
// Byte  5 is  77: 01001101
// Byte  6 is  44: 00101100
// Byte  7 is 111: 01101111
// Byte  8 is  67: 01000011
// Byte  9 is  33: 00100001
// Byte 10 is  11: 00001011

// Mask key is 32 bits (bytes 2 - 5 in the above)
const mask_key = [100, 10, 47, 77];

// Encoded is the remainder (bytes 6 - 10 in the above)
const encoded = [44, 111, 67, 33, 11];

var decoded = [];

// XOR each encoded bit with the corresponding i modulo 4th bit of the mask
for (var i = 0; i < encoded.length; i++) {
  decoded[i] = encoded[i] ^ mask_key[i % 4];
}

// Log the decoded (now ASCII) bytes
console.log('Message: ' + decoded);

// Convert each char
for (var i = 0; i < decoded.length; i++) {
  console.log(String.fromCharCode(decoded[i]));
}
```