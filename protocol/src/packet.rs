use crate::packet::ReplyType::*;
use crate::packet::AddressType::{Ipv4, Domain, Ipv6};
use crate::packet::CmdType::{Connect, Bind, Udp};
use crate::packet::AuthType::*;
use crate::packet::SubVersion::V0;
use std::borrow::Borrow;
use std::ops::BitAnd;

/// this packet is for authentication method
/// selecting request when client finishes connecting.
#[derive(Debug, PartialEq)]
pub struct AuthSelectRequest {
    version: Version,
    n_methods: u8,
    methods: Vec<AuthType>,
}

pub fn parse_auth_select_request_packet(data: &[u8]) -> Result<AuthSelectRequest, String> {
    let len = data.len();
    if len < 2 {
        return Err("data not enough.".to_string());
    }

    // version
    let mut version = parse_version(data.get(0).cloned())?;

    // num of methods
    let n_methods = data.get(1).cloned().unwrap();
    let num = n_methods;

    // verify data len
    let total: usize = usize::from(2 + n_methods);
    if data.len() < total {
        return Err("data length not right.".to_string());
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

pub fn encode_auth_select_request(request: AuthSelectRequest) -> Result<Vec<u8>, &'static str> {
    let mut data = Vec::<u8>::new();

    let version_num = encode_version(&request.version)?;
    data.push(version_num);
    data.push(request.n_methods);

    let methods = request.methods;

    for i in 0..methods.len() {
        let auth_type = methods.get(i);
        let auth_num = encode_auth_type(auth_type.unwrap())?;
        data.push(auth_num);
    }


    Ok(data)
}


impl AuthSelectRequest {
    pub fn new(version: Version, n_methods: u8, methods: Vec<AuthType>) -> AuthSelectRequest {
        AuthSelectRequest {
            version,
            n_methods,
            methods,
        }
    }

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

pub fn encode_auth_select_reply(reply: &AuthSelectReply) -> Result<Vec<u8>, &'static str> {
    let version_num = encode_version(reply.version())?;
    let auth_num = encode_auth_type(reply.auth_type())?;

    let mut buffer = Vec::<u8>::new();
    buffer.push(version_num);
    buffer.push(auth_num);

    Ok(buffer)
}

impl AuthSelectReply {
    pub fn new(version: Version, auth_type: AuthType) -> AuthSelectReply {
        AuthSelectReply {
            version,
            method: auth_type,
        }
    }

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

pub fn parse_dst_service_request(data: &[u8]) -> Result<(DstServiceRequest, u8), &str> {
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
    let (address, address_len) = parse_dst_address(&data[4..data.len()],
                                                   &address_type)?;
    let len: usize = usize::from(address_len);
    let port = get_port(data.get(4 + len..6 + len).unwrap())?;
    let result = DstServiceRequest {
        version,
        cmd,
        reserve,
        address_type,
        address,
        port,
    };

    Ok((result, address_len))
}

pub fn parse_dst_address(data: &[u8], addr_type: &AddressType) -> Result<(String, u8), &'static str> {
    let len = data.len();
    match addr_type {
        Ipv4 => {
            if len < 4 {
                return Err("data not enough in ipv4 type.");
            }
            let address = get_ipv4_from_bytes(data.get(0..4).unwrap())?;
            //let port: u16 = get_port(data.get(4..6).unwrap())?;
            Ok((address, 4))
        }
        Ipv6 => {
            if len < 16 {
                return Err("data not enough in ipv6 type.");
            }
            let address = get_ipv6_from_bytes(data.get(0..16).unwrap())?;
            //let port = get_port(data.get(16..18).unwrap())?;
            //let port: u16 = (data[17] as u16 | (data[18] as u16) << 8);
            Ok((address, 16))
        }
        Domain => {
            let addr_len = usize::from(data.get(0).cloned().unwrap());
            if len < addr_len {
                return Err("data not enough in domain type.");
            }
            let byte_array = data.get(1..addr_len + 1).unwrap();
            let address = get_domain_from_bytes(byte_array)?;
            //let port = get_port(data.get(addr_len + 1..addr_len + 3).unwrap())?;
            // let port: u16 = (data[addr_len + 1] as u16 | (data[addr_len + 2] as u16) << 8);
            let len = addr_len + 1;
            Ok((address, len as u8))
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

pub fn encode_dst_service_request(request: DstServiceRequest) -> Result<Vec<u8>, &'static str> {
    let mut data = Vec::<u8>::new();
    let version = encode_version(&request.version)?;
    let cmd_type = encode_cmd(&request.cmd)?;
    let reserve = request.reserve;
    let address_type = encode_address_type(&request.address_type)?;
    // ipv4
    let mut address = encode_address_with_type(request.address, &request.address_type)?;

    data.push(version);
    data.push(cmd_type);
    data.push(0);
    data.push(address_type);

    data.append(&mut address);

    let port = request.port;
    let low_bit = port.bitand(0x00FF) as u8;
    let high_bit = (port >> 8) as u8;
    data.push(low_bit);
    data.push(high_bit);

    Ok(data)
}

impl DstServiceRequest {
    pub fn new(version: Version, cmd: CmdType, reserve: u8, address_type: AddressType
               , address: String, port: u16) -> DstServiceRequest {
        DstServiceRequest {
            version,
            cmd,
            reserve,
            address_type,
            address,
            port,
        }
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn cmd(&self) -> &CmdType {
        &self.cmd
    }

    pub fn address_type(&self) -> &AddressType {
        &self.address_type
    }

    pub fn address(&self) -> String {
        self.address.to_string()
    }

    pub fn port(&self) -> u16 {
        self.port
    }
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
    let (address, address_len) = parse_dst_address(&data[4..data.len()],
                                                   &address_type)?;
    let len: usize = usize::from(address_len);
    let port = get_port(data.get(4 + len..6 + len).unwrap())?;
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

pub fn encode_dst_service_reply(dst_reply: DstServiceReply) -> Result<Vec<u8>, &'static str> {
    let mut data = Vec::<u8>::new();
    let version = encode_version(&dst_reply.version)?;
    let reply = encode_reply_type(&dst_reply.reply)?;

    let address_type = encode_address_type(&dst_reply.address_type)?;
    let mut address =
        encode_address_with_type(dst_reply.address, &dst_reply.address_type)?;

    data.push(version);
    data.push(reply);
    data.push(0);
    data.push(address_type);

    if dst_reply.address_type == AddressType::Domain {
        let address_len = address.len() as u8;

        data.push(address_len);
    }
    data.append(&mut address);

    let port = dst_reply.port;
    let low_bit = port.bitand(0x00FF) as u8;
    let high_bit = (port >> 8) as u8;
    data.push(low_bit);
    data.push(high_bit);

    Ok(data)
}

pub fn encode_address_with_type(address: String, address_type: &AddressType)
                                -> Result<Vec<u8>, &'static str> {
    match address_type {
        Ipv4 => encode_address_for_ipv4(address),
        Domain => encode_address_as_domain(address),
        Ipv6 => Err("ipv6 is not support in encoding."),
    }
}

pub fn encode_address_as_domain(address: String) -> Result<Vec<u8>, &'static str> {
    Ok(address.as_bytes().to_vec())
}


pub fn encode_address_for_ipv4(address: String) -> Result<Vec<u8>, &'static str> {
    let list: Vec<_> = address.split(".").collect();
    let mut result = Vec::<u8>::new();

    for i in 0..list.len() {
        let num: u8 = match list.get(i).unwrap().parse() {
            Ok(value) => Ok(value),
            Err(e) => Err("parse address error.")
        }?;

        result.push(num);
    }

    Ok(result)
}


impl DstServiceReply {
    pub fn new(version: Version, reply: ReplyType
               , address_type: AddressType, address: String, port: u16) -> DstServiceReply {
        DstServiceReply {
            version,
            reply,
            reserve: 0,
            address_type,
            address,
            port,
        }
    }
}

pub struct UserPassAuthRequest {
    version: SubVersion,
    u_len: u8,
    name: String,
    p_len: u8,
    password: String,
}

impl UserPassAuthRequest {
    pub fn version(&self) -> &SubVersion {
        &self.version
    }

    pub fn u_len(&self) -> u8 {
        self.u_len
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn p_len(&self) -> u8 {
        self.p_len
    }

    pub fn password(&self) -> &String {
        &self.password
    }
}


pub fn parse_user_auth_request(data: &[u8]) -> Result<UserPassAuthRequest, &'static str> {
    let len = data.len();
    if len < 2 {
        return Err("data not enough when parsing auth request.");
    }

    let version = parse_sub_version(data.get(0).cloned())?;
    let (u_len, name) = parse_len_and_string(&data[1..])?;
    let start = usize::from(2 + u_len);
    let (p_len, password) = parse_len_and_string(&data[start..])?;

    let result = UserPassAuthRequest {
        version,
        u_len,
        name,
        p_len,
        password,
    };

    Ok(result)
}

pub fn parse_len_and_string(data: &[u8]) -> Result<(u8, String), &'static str> {
    let total = data.len();
    if total == 0 {
        return Err("data is not enough.");
    }

    let len = match data.get(0).cloned() {
        Some(size) => Ok(size),
        None => Err("len is none")
    }?;

    if total < usize::from(1 + len) {
        return Err("data is not enough.");
    }

    let end = usize::from(len + 1);
    let name = parse_string_from_bytes(data.get(1..end).unwrap())?;

    Ok((len, name))
}

pub fn parse_string_from_bytes(data: &[u8]) -> Result<String, &'static str> {
    match std::str::from_utf8(data) {
        Ok(addr) => Ok(String::from(addr)),
        Err(e) => Err("err from bytes to utf8 string.")
    }
}

pub struct UserPassAuthReply {
    version: SubVersion,
    status: AuthResult,
}

impl UserPassAuthReply {
    pub fn version(&self) -> &SubVersion {
        &self.version
    }

    pub fn status(&self) -> &AuthResult {
        &self.status
    }
}

pub fn parse_user_auth_reply(data: &[u8]) -> Result<UserPassAuthReply, &'static str> {
    let len = data.len();
    if len != 2 {
        return Err("data not enough.");
    }

    let version = parse_sub_version(data.get(0).cloned())?;
    let status = parse_auth_result(data.get(1).cloned())?;
    let result = UserPassAuthReply {
        version,
        status,
    };

    Ok(result)
}

/// socks version
#[derive(Debug, PartialEq)]
pub enum Version {
    Socks5,
    Others,
}

/// sub negotiation version
#[derive(Debug, PartialEq)]
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

pub fn encode_version(version: &Version) -> Result<u8, &'static str> {
    match version {
        Version::Socks5 => Ok(5),
        // never
        Version::Others => Err("proxy only support version 5.")
    }
}

fn parse_sub_version(version: Option<u8>) -> Result<SubVersion, &'static str> {
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

pub fn encode_auth_type(auth_type: &AuthType) -> Result<u8, &'static str> {
    match auth_type {
        Non => Ok(0),
        NamePassword => Ok(2),
        _ => Err("auth method not supported."),
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

pub fn encode_cmd(cmd_type: &CmdType) -> Result<u8, &'static str> {
    match cmd_type {
        Connect => Ok(1),
        Bind => Ok(2),
        Udp => Ok(3),
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

pub fn encode_address_type(address_type: &AddressType) -> Result<u8, &'static str> {
    match address_type {
        Ipv4 => Ok(1),
        Domain => Ok(3),
        Ipv6 => Ok(4)
    }
}

#[derive(Debug, PartialEq)]
pub enum AuthResult {
    Success,
    Failure,
}


fn parse_auth_result(result: Option<u8>) -> Result<AuthResult, &'static str> {
    match result {
        Some(0) => Ok(AuthResult::Success),
        Some(_) => Ok(AuthResult::Failure),
        _ => Err("auth reply is empty.")
    }
}

/// reply type enum
#[derive(Debug, PartialEq)]
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

pub fn encode_reply_type(reply_type: &ReplyType) -> Result<u8, &'static str> {
    match reply_type {
        Success => Ok(0),
        ServerFailure => Ok(1),
        ConnectionNotAllowed => Ok(2),
        NetWorkUnReachable => Ok(3),
        HostUnreachable => Ok(4),
        ConnectionRefuse => Ok(5),
        TTLExpired => Ok(6),
        CmdNotSupport => Ok(7),
        AddressTypeNotSupport => Ok(8),
        ReplyType::Others => Ok(9),
    }
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

/// client stage transfer enum
pub enum ClientStage {
    Init,
    SendAuthSelect,
    AuthSelectFinish,
    // todo add sub session of auth
    SendRequest,
    RequestFinish,
    SendContentRequest,
    ContentFinish,
}

/// server stage transfer enum
#[derive(Debug, PartialEq)]
pub enum ServerStage {
    Init,
    AuthSelectFinish,
    RequestFinish,
    ReceiveContent,
    ContentFinish,
}