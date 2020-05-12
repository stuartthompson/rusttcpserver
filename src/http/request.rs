pub fn get_request_type() {
    if request.connection == "Upgrade" && request.upgrade == "websocket" {
        // Indicate websocket upgrade request
    }
    if (request.verb == "GET") {
        // Handle GET request
    }
}