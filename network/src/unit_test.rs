mod unit_test {
    use crate::server::ChildHandler;
    use crate::http;
    use crate::http::*;
    use mio::Token;
    use crate::tokens::Tokens;

    #[test]
    fn handle_init_test() {
        let mut child_handler = ChildHandler::new_test(&Token(0));
        // let bytes = &[5, 1, 0];
        child_handler.receive_u8_data(5, false);
        child_handler.receive_u8_data(1, false);
        child_handler.receive_u8_data(0, false);

        let size = child_handler.handle_init_stage();

        match size {
            Ok(len) => {
                assert_eq!(2, len);
            }
            _ => unreachable!()
        }
    }

    #[test]
    fn set_token_success() {
        let mut child_handler = ChildHandler::new_test(&Token(0));

        child_handler.set_dst_token(Token(1));

        let token = child_handler.get_dst_token();

        match token.as_ref() {
            Some(t) => assert_eq!(1, t.0),
            _ => unreachable!()
        }
    }

    #[test]
    fn is_dst_token_empty_true() {
        let mut child_handler = ChildHandler::new_test(&Token(0));
        let empty = child_handler.is_dst_token_empty();

        assert_eq!(empty, true);
    }

    #[test]
    fn is_dst_token_empty_false() {
        let mut child_handler = ChildHandler::new_test(&Token(0));
        child_handler.set_dst_token(Token(1));
        let empty = child_handler.is_dst_token_empty();

        assert_eq!(empty, false);
    }

    #[test]
    fn test_generate_token() {
        let mut tokens = Tokens::new();
        let token = tokens.next();

        assert_eq!(Token(1), token);
    }

    #[test]
    fn parse_line_success() {
        let data = [71 as u8, 69, 84, 32, 47, 112, 114, 111, 120, 121, 46, 112, 97, 99,
            32, 72, 84, 84, 80, 47, 49, 46, 49, 13, 10];
        let line = http::parse_line(&data);
        match line {
            Ok((line_str, offset)) => {
                assert_eq!("GET /proxy.pac HTTP/1.1", line_str);
                assert_eq!(25, offset);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn parse_line_when_zero() {
        let data = [13 as u8, 10];
        let line = http::parse_line(&data);
        match line {
            Ok((line_str, offset)) => {
                assert_eq!("", line_str);
                assert_eq!(2, offset);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn is_http_request_finish_true() {
        let data = [71 as u8, 69, 84, 32, 47, 112, 114, 111, 120, 121, 46, 112, 97, 99,
            32, 72, 84, 84, 80, 47, 49, 46, 49, 13, 10, 72, 111, 115, 116, 58, 32, 49, 50, 55, 46,
            48, 46, 48, 46, 49, 58, 49, 48, 56, 57, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101, 110,
            116, 58, 32, 77, 111, 122, 105, 108, 108, 97, 47, 53, 46, 48, 32, 40, 77, 97, 99, 105, 110,
            116, 111, 115, 104, 59, 32, 73, 110, 116, 101, 108, 32, 77, 97, 99, 32, 79, 83, 32, 88,
            32, 49, 48, 46, 49, 52, 59, 32, 114, 118, 58, 55, 48, 46, 48, 41, 32, 71, 101, 99, 107,
            111, 47, 50, 48, 49, 48, 48, 49, 48, 49, 32, 70, 105, 114, 101, 102, 111, 120, 47, 55,
            48, 46, 48, 13, 10];
        let line = http::is_http_packet_finish(&data);
        match line {
            Ok(finish) => assert_eq!(true, finish),
            Err(msg) => println!("err msg:{:?}", msg),
        }
    }

    #[test]
    fn is_http_request_finish_false() {
        let data = [71 as u8, 69, 84, 32, 47, 112, 114, 111, 120, 121, 46, 112, 97, 99,
            32, 72, 84, 84, 80, 47, 49, 46, 49, 13, 10, 72, 111, 115, 116, 58, 32, 49, 50, 55, 46,
            48, 46, 48, 46, 49, 58, 49, 48, 56, 57, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101, 110,
            116, 58, 32, 77, 111, 122, 105, 108, 108, 97, 47, 53, 46, 48, 32, 40, 77, 97, 99, 105, 110,
            116, 111, 115, 104, 59, 32, 73, 110, 116, 101, 108, 32, 77, 97, 99, 32, 79, 83, 32, 88,
            32, 49, 48, 46, 49, 52, 59, 32, 114, 118, 58, 55, 48, 46, 48, 41, 32, 71, 101, 99, 107,
            111, 47, 50, 48, 49, 48, 48, 49, 48, 49, 32, 70, 105, 114, 101, 102, 111, 120, 47, 55,
            48, 46, 48];
        let line = http::is_http_packet_finish(&data);
        match line {
            Ok(finish) => assert_eq!(false, finish),
            Err(msg) => println!("err msg:{:?}", msg),
        }
    }

    #[test]
    fn parse_http_header_success() {
        let data = String::from("Host: 127.0.0.1:8080");
        let map = parse_http_header(&data);

        match map {
            Ok((name, value)) => {
                assert_eq!("Host", name);
                assert_eq!("127.0.0.1:8080", value);
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn parse_http_headers_failure_data_not_enough() {
        let data = [72 as u8, 111, 115, 116, 58, 32, 49, 50, 55, 46,
            48, 46, 48, 46, 49, 58, 49, 48, 56, 57, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101, 110,
            116, 58, 32, 77, 111, 122, 105, 108, 108, 97, 47, 53, 46, 48, 32, 40, 77, 97, 99, 105, 110,
            116, 111, 115, 104, 59, 32, 73, 110, 116, 101, 108, 32, 77, 97, 99, 32, 79, 83, 32, 88,
            32, 49, 48, 46, 49, 52, 59, 32, 114, 118, 58, 55, 48, 46, 48, 41, 32, 71, 101, 99, 107,
            111, 47, 50, 48, 49, 48, 48, 49, 48, 49, 32, 70, 105, 114, 101, 102, 111, 120, 47, 55,
            48, 46, 48, 13, 10];


        let headers = parse_http_headers(&data, &PacketType::Request);

        match headers {
            Err(e) => {
                assert_eq!("data not enough when parse http headers", e);
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn parse_http_headers_fail_data_not_enough() {
        let data = [72 as u8, 111, 115, 116, 58, 32, 49, 50, 55, 46,
            48, 46, 48, 46, 49, 58, 49, 48, 56, 57, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101, 110,
            116, 58, 32, 77, 111, 122, 105, 108, 108, 97, 47, 53, 46, 48, 32, 40, 77, 97, 99, 105, 110,
            116, 111, 115, 104, 59, 32, 73, 110, 116, 101, 108, 32, 77, 97, 99, 32, 79, 83, 32, 88,
            32, 49, 48, 46, 49, 52, 59, 32, 114, 118, 58, 55, 48, 46, 48, 41, 32, 71, 101, 99, 107,
            111, 47, 50, 48, 49, 48, 48, 49, 48, 49, 32, 70, 105, 114, 101, 102, 111, 120, 47, 55,
            48, 46, 48, 13, 10];


        let headers = parse_http_headers(&data, &PacketType::Request);

        match headers {
            Err(msg) => {
                assert_eq!("data not enough when parse http headers", msg);
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn parse_http_headers_fail_header_format_error() {
        let data = [72 as u8, 111, 115, 116, 59, 32, 49, 50, 55, 46,
            48, 46, 48, 46, 49, 59, 49, 48, 56, 57, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101, 110,
            116, 58, 32, 77, 111, 122, 105, 108, 108, 97, 47, 53, 46, 48, 32, 40, 77, 97, 99, 105, 110,
            116, 111, 115, 104, 59, 32, 73, 110, 116, 101, 108, 32, 77, 97, 99, 32, 79, 83, 32, 88,
            32, 49, 48, 46, 49, 52, 59, 32, 114, 118, 58, 55, 48, 46, 48, 41, 32, 71, 101, 99, 107,
            111, 47, 50, 48, 49, 48, 48, 49, 48, 49, 32, 70, 105, 114, 101, 102, 111, 120, 47, 55,
            48, 46, 48, 13, 10, 13, 10];

        let headers = parse_http_headers(&data, &PacketType::Request);

        match headers {
            Err(msg) => {
                assert_eq!("header formatter error.", msg);
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn parse_http_headers_success_with_others() {
        let data = [72 as u8, 111, 115, 116, 58, 32, 49, 50, 55, 46,
            48, 46, 48, 46, 49, 58, 49, 48, 56, 57, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101, 110,
            116, 58, 32, 77, 111, 122, 105, 108, 108, 97, 47, 53, 46, 48, 32, 40, 77, 97, 99, 105, 110,
            116, 111, 115, 104, 59, 32, 73, 110, 116, 101, 108, 32, 77, 97, 99, 32, 79, 83, 32, 88,
            32, 49, 48, 46, 49, 52, 59, 32, 114, 118, 58, 55, 48, 46, 48, 41, 32, 71, 101, 99, 107,
            111, 47, 50, 48, 49, 48, 48, 49, 48, 49, 32, 70, 105, 114, 101, 102, 111, 120, 47, 55,
            48, 46, 48, 13, 10, 13, 10];


        let headers = parse_http_headers(&data, &PacketType::Request);

        match headers {
            Ok((transfer_type, offset)) => {
                assert_eq!(HttpParseState::OtherRequest, transfer_type);
                assert_eq!(120, offset);
            }

            Err(_) => unreachable!()
        }
    }

    #[test]
    fn parse_http_headers_success_with_content_length() {
        let data = [67 as u8, 111, 110, 116, 101, 110, 116, 45, 108, 101, 110, 103,
            116, 104, 58, 32, 49, 48, 48, 48, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101, 110,
            116, 58, 32, 77, 111, 122, 105, 108, 108, 97, 47, 53, 46, 48, 32, 40, 77, 97, 99, 105, 110,
            116, 111, 115, 104, 59, 32, 73, 110, 116, 101, 108, 32, 77, 97, 99, 32, 79, 83, 32, 88,
            32, 49, 48, 46, 49, 52, 59, 32, 114, 118, 58, 55, 48, 46, 48, 41, 32, 71, 101, 99, 107,
            111, 47, 50, 48, 49, 48, 48, 49, 48, 49, 32, 70, 105, 114, 101, 102, 111, 120, 47, 55,
            48, 46, 48, 13, 10, 13, 10];


        let headers = parse_http_headers(&data, &PacketType::Request);

        match headers {
            Ok((transfer_type, offset)) => {
                assert_eq!(120, offset);
                match transfer_type {
                    HttpParseState::ContentLength(len) => assert_eq!(1000, len),
                    _ => unreachable!()
                }
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn parse_http_headers_success_with_transfer_encoding() {
        let data = [67 as u8, 111, 110, 116, 101, 110, 116, 45, 108, 101, 110, 103,
            116, 104, 58, 32, 49, 48, 48, 48, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101, 110,
            116, 58, 32, 77, 111, 122, 105, 108, 108, 97, 47, 53, 46, 48, 32, 40, 77, 97, 99, 105, 110,
            116, 111, 115, 104, 59, 32, 73, 110, 116, 101, 108, 32, 77, 97, 99, 32, 79, 83, 32, 88,
            32, 49, 48, 46, 49, 52, 59, 32, 114, 118, 58, 55, 48, 46, 48, 41, 32, 71, 101, 99, 107,
            111, 47, 50, 48, 49, 48, 48, 49, 48, 49, 32, 70, 105, 114, 101, 102, 111, 120, 47, 55,
            48, 46, 48, 13, 10, 84, 114, 97, 110, 115, 102, 101, 114, 45, 69, 110, 99, 111, 100, 105,
            110, 103, 58, 32, 99, 104, 117, 110, 107, 101, 100, 13, 10, 13, 10];


        let headers = parse_http_headers(&data, &PacketType::Request);

        match headers {
            Ok((transfer_type, offset)) => {
                assert_eq!(148, offset);
                assert_eq!(HttpParseState::TransferEncoding, transfer_type);
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn read_with_content_length_success() {
        let data = [67 as u8, 111, 110, 116, 101, 110, 116, 45, 108, 101, 110, 103,
            116, 104, 58, 32, 49, 48, 48, 48, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101, 110,
            116, 58, 32, 77, 111, 122, 105, 108, 108, 97, 47, 53, 46, 48, 32, 40, 77, 97, 99, 105, 110,
            116, 111, 115, 104, 59, 32, 73, 110, 116, 101, 108, 32, 77, 97, 99, 32, 79, 83, 32, 88,
            32, 49, 48, 46, 49, 52, 59, 32, 114, 118, 58, 55, 48, 46, 48, 41, 32, 71, 101, 99, 107,
            111, 47, 50, 48, 49, 48, 48, 49, 48, 49, 32, 70, 105, 114, 101, 102, 111, 120, 47, 55,
            48, 46, 48, 13, 10, 13, 10];


        let result = read_with_length(&data, 10);

        match result {
            Ok(offset) => {
                assert_eq!(10, offset);
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn read_with_length_failed() {
        let data = [67 as u8, 111, 110, 116, 101, 110, 116, 45, 108, 101, 110, 103,
            116, 104, 58, 32, 49, 48, 48, 48, 13, 10, 85, 115, 101, 114, 45, 65, 103, 101, 110,
            116, 58, 32, 77, 111, 122, 105, 108, 108, 97, 47, 53, 46, 48, 32, 40, 77, 97, 99, 105, 110,
            116, 111, 115, 104, 59, 32, 73, 110, 116, 101, 108, 32, 77, 97, 99, 32, 79, 83, 32, 88,
            32, 49, 48, 46, 49, 52, 59, 32, 114, 118, 58, 55, 48, 46, 48, 41, 32, 71, 101, 99, 107,
            111, 47, 50, 48, 49, 48, 48, 49, 48, 49, 32, 70, 105, 114, 101, 102, 111, 120, 47, 55,
            48, 46, 48, 13, 10, 13, 10];


        let result = read_with_length(&data, 150);

        match result {
            Err(msg) => {
                assert_eq!("data is not enough when read with content-length.", msg);
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn read_util_socket_closed_success() {
        let data = [0 as u8, 12, 3, 4];
        let result = read_util_close(&data, true);

        match result {
            Ok(size) => assert_eq!(4, size),
            _ => unreachable!()
        }
    }

    #[test]
    fn read_util_socket_closed_failed() {
        let data = [0 as u8, 12, 3, 4];
        let result = read_util_close(&data, false);

        match result {
            Err(msg) => {
                assert_eq!("data not enough when read content-util-socket-close.".to_string(), msg)
            }
            _ => unreachable!()
        }
    }
}