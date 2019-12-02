mod unit_test {
    use crate::server::ChildHandler;
    use crate::http;

    #[test]
    fn handle_init_test() {
        let mut child_handler = ChildHandler::new_test(false);
        // let bytes = &[5, 1, 0];
        child_handler.receive_u8_data(5);
        child_handler.receive_u8_data(1);
        child_handler.receive_u8_data(0);

        let size = child_handler.handle_init_stage();

        match size {
            Ok(len) => {
                assert_eq!(2, len);
            }
            _ => unreachable!()
        }
    }

    #[test]
    fn parse_line_test() {
        let data = [71 as u8, 69, 84, 32, 47, 112, 114, 111, 120, 121, 46, 112, 97, 99,
            32, 72, 84, 84, 80, 47, 49, 46, 49, 13, 10];
        let line = http::parse_line(&data);
        match line {
            Ok(line_str) => println!("line:{:?}", line_str),
            Err(msg) => println!("err msg:{:?}", msg),
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
}