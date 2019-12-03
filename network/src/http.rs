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
    Others,
}

pub fn parse_http_request(data: &[u8]) {
    let mut content_length_present = false;
    let mut content_length = -1;
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
pub fn parse_http_headers(data: &[u8]) -> Result<(HttpParseState, usize), String> {
    let mut index = 0;
    let mut body_send_type = Others;
    loop {
        let (header, offset) = parse_line(&data[index..])?;

        // only \r\n --- end of headers
        if offset == 2{
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

