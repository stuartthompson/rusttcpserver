pub struct HttpRequest {
    pub verb: String,
    pub path: String,
    pub protocol: String,
    pub host: String,
    pub connection: String,
    pub cache_control: String,
    pub user_agent: String,
    pub accept: String,
    pub accept_encoding: String,
    pub accept_language: String,
    pub sec_websocket_version: String,
    pub sec_websocket_key: String,
    pub upgrade: String,
    pub sec_websocket_extensions: String
}

pub fn parse_http_request(request: &str) -> HttpRequest {
    let request_parts: Vec<&str> = request.split("\n").collect();
    
    // Parse verb and version
    let vpv: Vec<&str> = request_parts[0].split(" ").collect();

    let mut host: String = String::from("");
    let mut connection: String = String::from("");
    let mut cache_control: String = String::from("");
    let mut user_agent: String = String::from("");
    let mut accept: String = String::from("");
    let mut accept_encoding: String = String::from("");
    let mut accept_language: String = String::from("");
    let mut sec_websocket_version: String = String::from("");
    let mut sec_websocket_key: String = String::from("");
    let mut upgrade: String = String::from("");
    let mut sec_websocket_extensions: String = String::from("");

    for (_, x) in request_parts.iter().enumerate() {
        let parts: Vec<&str> = x.split(":").collect();
        if parts[0] == "Host" {
            host = String::from(parts[1].trim());
        }
        if parts[0] == "Connection" {
            connection = String::from(parts[1].trim());
        }
        if parts[0] == "Cache-Control" {
            cache_control = String::from(parts[1].trim());
        }
        if parts[0] == "User-Agent" {
            user_agent = String::from(parts[1].trim());
        }
        if parts[0] == "Accept" {
            accept = String::from(parts[1].trim());
        }
        if parts[0] == "Accept-Encoding" {
            accept_encoding = String::from(parts[1].trim());
        }
        if parts[0] == "Accept-Language" {
            accept_language = String::from(parts[1].trim());
        }
        if parts[0] == "Sec-WebSocket-Version" {
            sec_websocket_version = String::from(parts[1].trim());
        }
        if parts[0] == "Sec-WebSocket-Key" {
            sec_websocket_key = String::from(parts[1].trim());
        }
        if parts[0] == "Upgrade" {
            upgrade = String::from(parts[1].trim());
        }
        if parts[0] == "Sec-WebSocket-Extensions" {
            sec_websocket_extensions = String::from(parts[1].trim());
        }
    }

    // Start building http request
    let parsed: HttpRequest = HttpRequest {
        verb: String::from(vpv[0].trim()),
        path: String::from(vpv[1].trim()),
        protocol: String::from(vpv[2].trim()),
        host: host,
        connection: connection,
        cache_control: cache_control,
        user_agent: user_agent,
        accept: accept,
        accept_encoding: accept_encoding,
        accept_language: accept_language,
        sec_websocket_version: sec_websocket_version,
        sec_websocket_key: sec_websocket_key,
        upgrade: upgrade,
        sec_websocket_extensions: sec_websocket_extensions
    };

    return parsed;
}