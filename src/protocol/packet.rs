use crate::protocol::packet::SessionStage::AuthSelect;

/// this packet is for authentication method
/// selecting request when client finishes connecting.
#[derive(Debug, PartialEq)]
pub struct AuthSelectRequest {
    version: Version,
    n_methods: u8,
    methods: Vec<AuthType>,
}

pub fn parseAuthSelectRequest(data: &[u8]) -> Result<AuthSelectRequest, &str> {
    let len = data.len();
    // version
    let version = match data.get(0) {
        Some(5) => Version::Socks5,
        Some(_) => Version::Others,
        None => return Err("empty version num."),
    };
    // num of methods
    let n_methods = data.get(1).cloned().unwrap();
    let num = n_methods;

    let mut i = 0;
    let mut methods = Vec::<AuthType>::new();
    while i < num {
        let index = usize::from(2 + i);
        let method = match data.get(index) {
            Some(0) => AuthType::Non,
            Some(1) => AuthType::Gssapi,
            Some(2) => AuthType::NamePassword,
            Some(3) => AuthType::IanaAssigned,
            Some(0x80) => AuthType::Reserved,
            Some(0xff) => AuthType::NonAccept,
            _ => return Err("not support auth method.")
        };

        methods.push(method);
        i = i + 1;
    }

    // verify
    let total: usize = usize::from(2 + n_methods);
    if data.len() != total {
        return Err("too many data.");
    }

    let result = AuthSelectRequest {
        version,
        n_methods,
        methods,
    };

    Ok(result)
}

impl AuthSelectRequest {
    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn n_methods(&self) -> u8 {
        self.n_methods
    }

    pub fn methods(&self) -> &Vec<AuthType> {
        &self.methods
    }
}

/// this packet is for authentication method selecting reply from server
pub struct AuthSelectReply {
    version: Version,
    method: AuthType,
}

/// this packet is for target destination service request from client
pub struct DstServiceRequest {
    version: Version,
    cmd: CmdType,
    rev: u8,
    address_type: AddressType,
    address: String,
    port: u16,
}

/// his packet is for target destination service request from server
pub struct DstServiceReply {
    version: Version,
    reply: ReplyType,
    rsv: u8,
    address_type: AddressType,
    address: String,
    port: u16,
}

/// socks version
#[derive(Debug, PartialEq)]
pub enum Version {
    Socks5,
    Others,
}

/// auth type enum
#[derive(Debug, PartialEq)]
pub enum AuthType {
    Non,
    Gssapi,
    NamePassword,
    IanaAssigned,
    Reserved,
    NonAccept,
}

/// cmd type enum
pub enum CmdType {
    Connect,
    Bind,
    Udp,
}

/// address type enum
#[derive(Debug, PartialEq)]
pub enum AddressType {
    Ipv4,
    Domain,
    Ipv6,
}

/// reply type enum
pub enum ReplyType {
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

/// session状态
pub enum SessionStage {
    Init,
    AuthSelect,
    AuthSelectFinish,
    // todo add sub session of auth
    Request,
    RequestFinish,
    ContentRequest,
    ContentFinish,
}