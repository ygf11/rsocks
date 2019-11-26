use crate::protocol::packet::SessionStage::AuthSelect;
use crate::protocol::packet::AuthType::{Non, Gssapi, NamePassword, IanaAssigned, Reserved, NonAccept};

/// this packet is for authentication method
/// selecting request when client finishes connecting.
#[derive(Debug, PartialEq)]
pub struct AuthSelectRequest {
    version: Version,
    n_methods: u8,
    methods: Vec<AuthType>,
}

pub fn parse_auth_select_request_packet(data: &[u8]) -> Result<AuthSelectRequest, &'static str> {
    let len = data.len();
    // version
    let mut version = parse_version(data.get(0).cloned())?;

    // num of methods
    let n_methods = data.get(1).cloned().unwrap();
    let num = n_methods;

    let mut i = 0;
    let mut methods = Vec::<AuthType>::new();
    while i < num {
        let index = usize::from(2 + i);
        let method = parse_auth_type(data.get(index).cloned())?;

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

fn parse_version(version: Option<u8>) -> Result<Version, &'static str> {
    match version {
        Some(5) => Ok(Version::Socks5),
        Some(_) => Ok(Version::Others),
        None => Err("empty version num.")
    }
}

fn parse_auth_type(auth: Option<u8>) -> Result<AuthType, &'static str> {
    match auth {
        Some(0) => Ok(Non),
        Some(1) => Ok(Gssapi),
        Some(2) => Ok(NamePassword),
        Some(3) => Ok(IanaAssigned),
        Some(0x80) => Ok(Reserved),
        Some(0xff) => Ok(NonAccept),
        _ => return Err("auth method not supported.")
    }
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

pub fn parse_auth_select_reply_packet(data: &[u8]) -> Result<AuthSelectReply, &'static str> {
    let len = data.len();
    if len != 2 {
        return Err("auth select reply packet length not right.");
    }

    let version = parse_version(data.get(0).cloned())?;
    let method = parse_auth_type(data.get(1).cloned())?;

    let result = AuthSelectReply {
        version,
        method,
    };

    Ok(result)
}

impl AuthSelectReply{
    pub fn version(&self) -> &Version{
        &self.version
    }

    pub fn auth_type(&self) -> &AuthType {
        &self.method
    }
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