/// 两种解码:
/// 1. content-length
/// 2. transfer-encoding
///
///
///
static CR: u8 = 13;
static LF: u8 = 10;
static CONTENT_LENGTH: &'static str = "content-length";

/// judge http request/response is finished
pub fn is_http_packet_finish(data: &[u8]) -> Result<bool, String> {
    let mut index = 0;

    Ok(false)
}

pub fn parse_line(data: &[u8]) -> Result<(String, usize), String> {
    let start = 0;
    let mut cur = 0;
    loop {
        if cur >= data.len(){
            return Err("data not enough".to_string());
        }

        let next_byte = data[cur] & 0xFF;
        if next_byte == CR {
            cur = cur + 1;
            continue;
        }

        if next_byte == LF {
            let array = &data[start..cur + 1];
            return Ok((String::from_utf8_lossy(array).to_string(), cur));
        }

        cur = cur + 1;
    }
}