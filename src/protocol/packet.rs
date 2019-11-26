use crate::protocol::packet::SessionStage::AuthSelect;
use crate::protocol::packet::AuthType::{Non, Gssapi, NamePassword, IanaAssigned, Reserved, NonAccept};
use crate::protocol::packet::CmdType::{Connect, Bind, Udp};
use crate::protocol::packet::AddressType::{Ipv4, Domain, Ipv6};

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
    if len < 2 {
        return Err("data not enough.");
    }

    // version
    let mut version = parse_version(data.get(0).cloned())?;

    // num of methods
    let n_methods = data.get(1).cloned().unwrap();
    let num = n_methods;

    // verify
    let total: usize = usize::from(2 + n_methods);
    if data.len() != total {
        return Err("data length not right.");
    }

    let mut i = 0;
    let mut methods = Vec::<AuthType>::new();
    while i < num {
        let index = usize::from(2 + i);
        let method = parse_auth_type(data.get(index).cloned())?;

        methods.push(method);
        i = i + 1;
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

impl AuthSelectReply {
    pub fn version(&self) -> &Version {
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

pub fn parse_dst_service_request(data: &[u8]) -> Result<DstServiceRequest, &str> {
    let len = data.len();
    if len < 4 {
        return Err("data not enough");
    }

    let version = parse_version(data.get(0).cloned())?;
    let cmd = parse_cmd(data.get(1).cloned())?;
    let reserve = match data.get(2).cloned() {
        Some(num) => num,
        None => 0
    };

    let addr_type = parse_address_type(data.get(3).cloned())?;

    Err("err")
}

fn parse_dst_address(data: &[u8], addr_type: AddressType) -> Result<String, &str> {
    let len = data.len();
    match addr_type {
        Ipv4 => {
            if len < 4 {
                return Err("data not enough in ipv4 type.");
            }
            parse_address_from_bytes(data.get(0..4).unwrap())
        }
        Ipv6 => {
            if len < 16 {
                return Err("data not enough in ipv6 type.");
            }
            parse_address_from_bytes(data.get(0..16).unwrap())
        }
        Domain => {
            let addr_len = usize::from(data.get(0).cloned().unwrap());
            if len < addr_len {
                return Err("data not enough in domain type.");
            }
            let byte_array = data.get(1..addr_len + 1).unwrap();
            parse_address_from_bytes(byte_array)
        }
    }
}

fn parse_address_from_bytes(bytes: &[u8]) -> Result<String, &str> {
    match std::str::from_utf8(bytes) {
        Ok(addr) => Ok(String::from(addr)),
        Err(e) => Err("err from bytes to utf8 string.")
    }
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

fn parse_version(version: Option<u8>) -> Result<Version, &'static str> {
    match version {
        Some(5) => Ok(Version::Socks5),
        Some(_) => Ok(Version::Others),
        None => Err("empty version num.")
    }
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

/// cmd type enum
pub enum CmdType {
    Connect,
    Bind,
    Udp,
}

fn parse_cmd(cmd: Option<u8>) -> Result<CmdType, &'static str> {
    match cmd {
        Some(1) => Ok(Connect),
        Some(2) => Ok(Bind),
        Some(3) => Ok(Udp),
        _ => Err("cmd type not support.")
    }
}

/// address type enum
#[derive(Debug, PartialEq)]
pub enum AddressType {
    Ipv4,
    Domain,
    Ipv6,
}

fn parse_address_type(addr_type: Option<u8>) -> Result<AddressType, &'static str> {
    match addr_type {
        Some(1) => Ok(Ipv4),
        Some(3) => Ok(Domain),
        Some(4) => Ok(Ipv6),
        _ => Err("address type not support.")
    }
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