use sha1::{Digest, Sha1};

pub fn upgrade_to_websocket(upgrade_key: &String) -> String {
    let accept_key = build_ws_accept_key(upgrade_key);

    return build_response(accept_key);
}

fn build_ws_accept_key(upgrade_key: &String) -> String {
    // Calculate accept key
    let mut hasher = Sha1::new();
    let appended = format!(
        "{0}{1}",
        upgrade_key, "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
    );
    hasher.input(appended.as_bytes());
    let hashed_result = hasher.result();
    let accept_key = base64::encode(hashed_result);

    return accept_key;
}

fn build_response(accept_key: String) -> String {
    let response = std::fmt::format(format_args!(
        "HTTP/1.1 101 Switching Protocols\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Accept: {}\r\n\
        Upgrade: websocket\r\n\r\n",
        accept_key
    ));

    return response;
}
