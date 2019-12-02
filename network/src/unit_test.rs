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
        let data = [71 as u8, 69, 84, 32, 47, 32, 72, 84, 84, 80, 47, 49, 46, 49, 13, 10];

        let line = http::parse_line(&data);
        match line {
            Ok(line_str) => println!("line:{:?}", line_str),
            Err(msg) => println!("err msg:{:?}", msg),
        }
    }
    //#[test]
    //fn handle_dst_request_test() {
    //    let mut child_handler = ChildHandler::new_test(None, false);
    //    let mut bytes = &[5, 0, 1, 1, 127, 0, 0, 1, 1, 4];

    //    let size = child_handler.handle_dst_request(bytes);

    //}
}