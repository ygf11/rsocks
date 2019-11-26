/// this packet is for authentication method
/// selecting request when client finishes connecting.
struct AuthSelectRequest {
    version: Version,
    n_methods: u8,
    methods: Vec<AuthType>,
}

/// this packet is for authentication method selecting reply from server
struct AuthSelectReply {
    version: Version,
    method: AuthType,
}

/// this packet is for target destination service request from client
struct DstServiceRequest {
    version: Version,
    cmd: CmdType,
    rev: u8,
    address_type: AddressType,
    address: String,
    port: u16,
}

/// his packet is for target destination service request from server
struct DstServiceReply {
    version: Version,
    reply: ReplyType,
    rsv: u8,
    address_type: AddressType,
    address: String,
    port: u16,
}

/// socks version
enum Version {
    Socks5,
    Others,
}

/// auth type enum
enum AuthType {
    Non,
    Gssapi,
    NamePassword,
    IanaAssigned,
    Reserved,
    NonAccept,
}

/// cmd type enum
enum CmdType {
    Connect,
    Bind,
    Udp,
}

/// address type enum
enum AddressType {
    Ipv4,
    Domain,
    Ipv6,
}

/// reply type enum
enum ReplyType {
    Success,
    ServerFailure,
    ConnectionNotAllowed,
    NetWorkUnReachable,
    ConnectionRefuse,
    TTLExpired,
    CmdNotSupport,
    AddressTypeNotSupport,
    Others,
}

