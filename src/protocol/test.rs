mod test {
    use crate::protocol::packet::{Version, AuthType, AddressType};

    #[test]
    fn auth_select_test1() {
        use super::super::packet;
        let mut packets = [0 as u8; 3];
        packets[0] = 5;
        packets[1] = 1;
        packets[2] = 0;

        println!("value:{:?}", packets);

        let result = packet::parse_auth_select_request_packet(&packets);

        match result {
            Ok(request) => {
                assert_eq!(Version::Socks5, *request.version());
                assert_eq!(1, request.n_methods());
                assert_eq!(AuthType::Non, *request.methods().get(0).unwrap());
            }

            Err(msg) => println!("err message:{}", msg)
        }
    }

    #[test]
    fn auth_select_test2() {
        use super::super::packet;
        let mut packets = [0 as u8; 4];
        packets[0] = 5;
        packets[1] = 2;
        packets[2] = 0;
        packets[3] = 1;

        println!("value:{:?}", packets);

        let result = packet::parse_auth_select_request_packet(&packets);

        match result {
            Ok(request) => {
                assert_eq!(Version::Socks5, *request.version());
                assert_eq!(2, request.n_methods());
                assert_eq!(AuthType::Non, *request.methods().get(0).unwrap());
                assert_eq!(AuthType::Gssapi, *request.methods().get(1).unwrap())
            }

            Err(msg) => println!("err message:{}", msg)
        }
    }

    #[test]
    fn auth_select_test3() {
        use super::super::packet;
        let mut packets = [0 as u8; 4];
        packets[0] = 4;
        packets[1] = 2;
        packets[2] = 0;
        packets[3] = 2;

        println!("value:{:?}", packets);

        let result = packet::parse_auth_select_request_packet(&packets);

        match result {
            Ok(request) => {
                assert_eq!(Version::Others, *request.version());
                assert_eq!(2, request.n_methods());
                assert_eq!(AuthType::Non, *request.methods().get(0).unwrap());
                assert_eq!(AuthType::NamePassword, *request.methods().get(1).unwrap())
            }

            Err(msg) => println!("err message:{}", msg)
        }
    }
}
