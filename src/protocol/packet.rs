/// this packet is for authentication method
/// selecting request when client finishes connecting.
struct AuthSelectRequest {
    version: u8,
    n_methods: u8,
    methods: Vec<u8>,
}

/// this packet is for authentication method selecting reply from server
struct AuthSelectReply {
    version: u8,
    method: u8,
}

/// this packet is for target destination service request from client
struct DstServiceRequest {
    version: u8,
    cmd: u8,
    rev: u8,
    address_type: u8,
    address: Vec<u8>,
    port: u16,
}

/// his packet is for target destination service request from server
struct DstServiceReply {
    version: u8,
    reply: u8,
    rsv: u8,
    address_type: u8,
    address: Vec<u8>,
    port: u16,
}

/// socks version
enum Version{
    SOCKS5,
    OTHERS,
}

enum AuthType{
    NON,
    GSSAPI,
    NAME_PASSWORD,
    IANA_ASSIGNED,
    RESERVED,
    NON_ACCEPT,
}



