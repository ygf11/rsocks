mod unit_test {
    use crate::server::ChildHandler;

    #[test]
    fn handle_init_test() {
        let mut child_handler = ChildHandler::new_test(None, false);
        let bytes = &[5, 1, 0];

        let size = child_handler.handle_init_stage(bytes);

        match size {
            Ok(len) => {
                assert_eq!(2, len);
            }
            _ => unreachable!()
        }
    }
}