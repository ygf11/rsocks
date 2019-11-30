mod unit_test {
    use crate::server::ChildHandler;

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

    //#[test]
    //fn handle_dst_request_test() {
    //    let mut child_handler = ChildHandler::new_test(None, false);
    //    let mut bytes = &[5, 0, 1, 1, 127, 0, 0, 1, 1, 4];

    //    let size = child_handler.handle_dst_request(bytes);

    //}
}