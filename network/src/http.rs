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
pub enum HttpResult {
    End(usize),
    DataNotEnough,
}

#[derive(Debug, PartialEq)]
pub enum Kind {
    End(usize),
    DataNotEnough,
    Continue(usize),
}

#[derive(Debug, PartialEq)]
pub enum HttpParseState {
    OtherRequest,
    OtherResponse,
    ContentLength(usize),
    TransferEncoding,
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

pub fn get_end_of_http_packet(data: &[u8], packet_type: PacketType, socket_closed: bool)
                              -> Result<HttpResult, String> {
    // 1. parse initial line
    // 2. parse http headers
    // 3. receive util end

    let (line, initial_offset) = parse_line(data)?;

    let (transfer_type, headers_offset) =
        parse_http_headers(&data[initial_offset..], &packet_type)?;

    let pos = initial_offset + headers_offset;
    let starter = &data[pos..];
    match transfer_type {
        TransferEncoding => read_with_transfer_encoding(starter),
        OtherRequest => Ok(HttpResult::End(pos)),
        OtherResponse => read_util_close(starter, socket_closed),
        ContentLength(size) => read_with_length(starter, size),
    }
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
pub fn read_with_length(data: &[u8], total: usize) -> Result<HttpResult, String> {
    let mut to_read = total;
    let len = data.len();

    if len < total {
        return Ok(HttpResult::DataNotEnough);
    }

    Ok(HttpResult::End(total))
}

/// read content in transfer-encoding
pub fn read_with_transfer_encoding(data: &[u8]) -> Result<HttpResult, String> {
    // todo receive dst response
    let mut offset = 0 as usize;
    loop {
        let result = parse_chunk(&data[offset..])?;
        let len = match result {
            Kind::End(chunk_size) => {
                return Ok(HttpResult::End(offset+chunk_size));
            }
            Kind::Continue(offset) => offset,
            Kind::DataNotEnough => {
                return Ok(HttpResult::DataNotEnough)
            },
        };

        offset = offset + len;
        println!("chunk size");
    }

    //Ok(end)
}

/// read util socket closed
pub fn read_util_close(data: &[u8], socket_closed: bool) -> Result<HttpResult, String> {
    match socket_closed {
        true => Ok(HttpResult::End(data.len())),
        false => Ok(HttpResult::DataNotEnough),
    }
}

pub fn parse_chunk(data: &[u8]) -> Result<Kind, String> {
    let (line, first_offset) = parse_line(data)?;
    let raw_data = &data[0..first_offset - 2];
    let chunk_size = parse_chunk_size(raw_data);

    if chunk_size == 0 {
        let offset = parse_chunk_end(&data[first_offset..])?;
        return Ok(Kind::End(offset + first_offset));
    }

    let end_pos = first_offset + chunk_size;
    if data.len() < end_pos + 2 {
        return Ok(Kind::DataNotEnough);
    }

    let first_end = data[end_pos];
    let second_end = data[end_pos + 1];

    if first_end != CR || second_end != LF {
        return Err("chunk end is not correct.".to_string());
    }

    Ok(Kind::Continue(end_pos + 2))
}


pub fn parse_chunk_end(data: &[u8]) -> Result<usize, String> {
    let (line, offset) = parse_line(data)?;

    Ok(offset)
}

pub fn parse_chunk_size(data: &[u8]) -> usize {
    let total = data.len();
    let mut sum = 0 as usize;
    let mut base = 1 as usize;

    // todo skip other chars
    for i in 0..total {
        let index = total - i - 1;
        let hex = data[index];

        if hex >= 48 && hex <= 57 {
            let num = (hex - 48) as usize;
            sum = sum + num * base;
        }

        if hex >= 97 && hex <= 102 {
            let num = (hex - 87) as usize;
            sum = sum + num * base;
        }
        base = base * 16;
    }

    sum
}