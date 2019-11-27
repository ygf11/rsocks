use crate::protocol::packet::SessionStage::AuthSelect;
use crate::protocol::packet::AuthType::*;
use crate::protocol::packet::CmdType::{Connect, Bind, Udp};
use crate::protocol::packet::AddressType::{Ipv4, Domain, Ipv6};
use crate::protocol::packet::ReplyType::*;
use crate::protocol::packet::SubVersion::*;

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

    // verify data len
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
    reserve: u8,
    address_type: AddressType,
    address: String,
    port: u16,
}

pub fn parse_dst_service_request(data: &[u8]) -> Result<DstServiceRequest, &str> {
    let len = data.len();
    if len < 4 {
        return Err("data not enough in dst request packet.");
    }

    let version = parse_version(data.get(0).cloned())?;
    let cmd = parse_cmd(data.get(1).cloned())?;
    let reserve = match data.get(2).cloned() {
        Some(num) => num,
        None => 0
    };

    let address_type = parse_address_type(data.get(3).cloned())?;
    let (address, port) = parse_dst_address(&data[4..data.len()],
                                            &address_type)?;

    let result = DstServiceRequest {
        version,
        cmd,
        reserve,
        address_type,
        address,
        port,
    };

    Ok(result)
}

pub fn parse_dst_address(data: &[u8], addr_type: &AddressType) -> Result<(String, u16), &'static str> {
    let len = data.len();
    match addr_type {
        Ipv4 => {
            if len < 4 {
                return Err("data not enough in ipv4 type.");
            }
            let address = get_ipv4_from_bytes(data.get(0..4).unwrap())?;
            let port: u16 = get_port(data.get(4..6).unwrap())?;
            Ok((address, port))
        }
        Ipv6 => {
            if len < 16 {
                return Err("data not enough in ipv6 type.");
            }
            let address = get_ipv6_from_bytes(data.get(0..16).unwrap())?;
            let port = get_port(data.get(16..18).unwrap())?;
            //let port: u16 = (data[17] as u16 | (data[18] as u16) << 8);
            Ok((address, port))
        }
        Domain => {
            let addr_len = usize::from(data.get(0).cloned().unwrap());
            if len < addr_len {
                return Err("data not enough in domain type.");
            }
            let byte_array = data.get(1..addr_len + 1).unwrap();
            let address = get_domain_from_bytes(byte_array)?;
            let port = get_port(data.get(addr_len + 1..addr_len + 3).unwrap())?;
            // let port: u16 = (data[addr_len + 1] as u16 | (data[addr_len + 2] as u16) << 8);
            Ok((address, port))
        }
    }
}

pub fn get_domain_from_bytes(bytes: &[u8]) -> Result<String, &'static str> {
    parse_string_from_bytes(bytes)
}

pub fn get_ipv4_from_bytes(bytes: &[u8]) -> Result<String, &'static str> {
    let first = bytes[0].to_string();
    let second = bytes[1].to_string();
    let third = bytes[2].to_string();
    let forth = bytes[3].to_string();

    let mut result = String::new();
    result.push_str(first.as_str());
    result.push('.');
    result.push_str(second.as_str());
    result.push('.');
    result.push_str(third.as_str());
    result.push('.');
    result.push_str(forth.as_str());

    Ok(result)
}

pub fn get_ipv6_from_bytes(bytes: &[u8]) -> Result<String, &'static str> {
    // todo parse bytes to ipv6
    Ok(String::new())
}

pub fn get_port(bytes: &[u8]) -> Result<u16, &'static str> {
    let len = bytes.len();
    let first = bytes[0].clone();
    let second = bytes[1].clone();

    let port = (first as u16 | (second as u16) << 8);

    Ok(port)
}


/// his packet is for target destination service request from server
pub struct DstServiceReply {
    version: Version,
    reply: ReplyType,
    reserve: u8,
    address_type: AddressType,
    address: String,
    port: u16,
}

pub fn parse_dst_service_reply(data: &[u8]) -> Result<DstServiceReply, &'static str> {
    let len = data.len();
    if len < 4 {
        return Err("data not enough in dst reply packet.");
    }

    let version = parse_version(data.get(0).cloned())?;
    let reply = parse_reply_type(data.get(1).cloned())?;
    let reserve = match data.get(2).cloned() {
        Some(num) => num,
        None => 0
    };

    let address_type = parse_address_type(data.get(3).cloned())?;
    let (address, port) = parse_dst_address(&data[4..data.len()],
                                            &address_type)?;

    let result = DstServiceReply {
        version,
        reply,
        reserve,
        address_type,
        address,
        port,
    };

    Ok(result)
}

pub struct UserPassAuthRequest {
    version: SubVersion,
    u_len: u8,
    name: String,
    p_len: u8,
    password: String,
}

pub fn parse_user_auth_request(data:&[u8]) -> Result<UserPassAuthRequest, &'static str>{
    let len = data.len();
    if len < 2 {
        return Err("data not enough when parsing auth request.")
    }

    let version = parse_sub_version(data.get(0).cloned())?;
    let u_len = match data.get(1).cloned(){
        Some(size)  => Ok(size),
        None => Err("u_len is none")
    }?;

    Err("err")

}

pub fn parse_string_from_bytes(data:&[u8]) -> Result<String, &'static str>{
    match std::str::from_utf8(data) {
        Ok(addr) => Ok(String::from(addr)),
        Err(e) => Err("err from bytes to utf8 string.")
    }
}


/// socks version
#[derive(Debug, PartialEq)]
pub enum Version {
    Socks5,
    Others,
}

/// sub negotiation version
pub enum SubVersion {
    V0,
    Others,
}




pub fn parse_version(version: Option<u8>) -> Result<Version, &'static str> {
    match version {
        Some(5) => Ok(Version::Socks5),
        Some(_) => Ok(Version::Others),
        None => Err("empty version num.")
    }
}

fn parse_sub_version(version:Option<u8>) -> Result<SubVersion, &'static str> {
    match version {
        Some(0) => Ok(V0),
        Some(_) => Ok(SubVersion::Others),
        None => Err("empty sub version num.")
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

pub fn parse_auth_type(auth: Option<u8>) -> Result<AuthType, &'static str> {
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

pub fn parse_cmd(cmd: Option<u8>) -> Result<CmdType, &'static str> {
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
    HostUnreachable,
    ConnectionRefuse,
    TTLExpired,
    CmdNotSupport,
    AddressTypeNotSupport,
    Others,
}

fn parse_reply_type(reply_type: Option<u8>) -> Result<ReplyType, &'static str> {
    match reply_type {
        Some(0) => Ok(Success),
        Some(1) => Ok(ServerFailure),
        Some(2) => Ok(ConnectionNotAllowed),
        Some(3) => Ok(NetWorkUnReachable),
        Some(4) => Ok(HostUnreachable),
        Some(5) => Ok(ConnectionRefuse),
        Some(6) => Ok(TTLExpired),
        Some(7) => Ok(CmdNotSupport),
        Some(8) => Ok(AddressTypeNotSupport),
        Some(9) => Ok(ReplyType::Others),
        _ => Err("reply type not support.")
    }
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