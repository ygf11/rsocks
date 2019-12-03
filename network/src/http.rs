use std::collections::HashMap;
use self::HttpParseState::*;

/// 两种解码:
/// 1. content-length
/// 2. transfer-encoding
///
///
static CR: u8 = 13;
static LF: u8 = 10;
static CONTENT_LENGTH: &'static str = "content-length";

#[derive(Debug, PartialEq)]
pub enum HttpParseState {
    ContentLength(usize),
    TransferEncoding,
    OtherRequest,
    OtherResponse,
}

impl HttpParseState {
    pub fn build_by_packet_type(packet_type: &PacketType) -> HttpParseState {
        match packet_type {
            Request => OtherRequest,
            Response => OtherResponse,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PacketType {
    Request,
    Response,
}

pub fn is_end_of_http_packet(data: &[u8], packet_type: PacketType, socket_closed: bool)
                             -> Result<bool, String> {
    // 1. parse initial line
    // 2. parse http headers
    // 3. receive util end

    let (line, offset) = parse_line(data)?;

    let (transfer_type, offset) =
        parse_http_headers(&data[offset..], &packet_type)?;

    Err("err".to_string())
}

/// judge http request/response is finished
pub fn is_http_packet_finish(data: &[u8]) -> Result<bool, String> {
    let mut index = 0;
    loop {
        let (line, offset) = match parse_line(&data[index..]) {
            Ok((line, offset)) => (line, offset),
            Err(msg) => return Ok(false),
        };

        index = index + offset;
        if index == data.len() {
            return Ok(true);
        }

        println!("line:{:?}", line);
    }

    Ok(true)
}

pub fn parse_first_line(data: &[u8]) -> Result<(String, usize), String> {
    parse_line(data)
}

///
/// four conditions:
/// 1. content-length: use content-length
/// 2. transfer-encoding: use chunk-size (over write content-length)
/// 3. http-request and non of each(1/2): non http body
/// 4. http-response and non of each(1/2): receive util connection close
///
///
pub fn parse_http_headers(data: &[u8], packet_type: &PacketType)
                          -> Result<(HttpParseState, usize), String> {
    let mut index = 0;
    let mut body_send_type = HttpParseState::build_by_packet_type(packet_type);
    loop {
        let (header, offset) = parse_line(&data[index..])?;

        // only \r\n --- end of headers
        if offset == 2 {
            return Ok((body_send_type, index + 2));
        }

        let (name, value) = parse_http_header(&header)?;

        if name.to_ascii_lowercase() == "transfer-encoding".to_string()
            && value.to_ascii_lowercase() == "chunked".to_string() {
            body_send_type = TransferEncoding;
        }

        if body_send_type != TransferEncoding
            && name.to_ascii_lowercase() == "content-length".to_string() {
            body_send_type = ContentLength(value.parse::<usize>().unwrap());
        }


        index = index + offset;
        if index == data.len() {
            return Err("data not enough when parse http headers".to_string());
        }

        println!("line:{:?}", header);
    }
}

pub fn parse_http_header(line: &String) -> Result<(String, String), String> {
    let new_line = line.replace(" ", "");
    let mut items: Vec<&str> = new_line.splitn(2, ":").collect();

    if items.len() < 2 {
        return Err("header formatter error.".to_string());
    }

    let name = String::from(items.remove(0));
    let value = String::from(items.remove(0));
    Ok((name, value))
}

pub fn parse_line(data: &[u8]) -> Result<(String, usize), String> {
    let start = 0;
    let mut cur = 0;
    loop {
        if cur >= data.len() {
            return Err("data not enough".to_string());
        }

        let next_byte = data[cur] & 0xFF;
        if next_byte == CR {
            cur = cur + 1;
            continue;
        }

        if next_byte == LF {
            if cur - 1 == start {
                return Ok((String::from(""), cur + 1));
            }

            let array = &data[start..cur - 1];
            return Ok((String::from_utf8_lossy(array).to_string(), cur + 1));
        }

        cur = cur + 1;
    }
}

/// read content in content_length
pub fn read_with_content_length(data: &[u8], total: usize) -> Result<usize, String> {
    let mut to_read = total;
    let len = data.len();

    if len < total {
        return Err("data is not enough when read with content-length.".to_string());
    }

    Ok(total)
}

/// read content in transfer-encoding
pub fn read_with_transfer_encoding(data: &[u8]) -> Result<usize, String> {
    // todo receive dst response
    Err("err".to_string())
}

/// read util socket closed
pub fn read_util_close(data: &[u8], socket_closed: bool) -> Result<usize, String> {
    match socket_closed{
        true => Ok(data.len()),
        false => Err("data not enough when read content-util-socket-close.".to_string())
    }
}
