mod unit_test {
    use crate::protocol::packet;
    use crate::protocol::packet::*;
    use crate::protocol::packet::AuthType::Non;
    use crate::protocol::packet::AddressType::{Ipv4, Domain};

    #[test]
    fn parse_version_socks5_success() {
        let num = Some(5);

        let version = parse_version(num);

        match version {
            Ok(v) => assert_eq!(v, Version::Socks5),
            Err(e) => unreachable!()
        }
    }

    #[test]
    fn parse_version_others_success() {
        let num = Some(4);

        let version = parse_version(num);

        match version {
            Ok(v) => assert_eq!(v, Version::Others),
            Err(e) => unreachable!()
        }
    }

    #[test]
    fn parse_version_failed() {
        let num = None;

        let version = parse_version(num);

        match version {
            Ok(v) => assert_eq!(v, Version::Others),
            Err(e) => assert_eq!("empty version num.", e)
        }
    }

    #[test]
    fn parse_ipv4_from_bytes_success() {
        let bytes = [49, 50, 55, 46, 48, 46, 48, 46, 49];
        let address = get_ipv4_from_bytes(&bytes);

        match address {
            Ok(addr) => assert_eq!("49.50.55.46", addr),
            Err(e) => assert_eq!("err from bytes to utf8 string.", e)
        }
    }

    #[test]
    fn parse_domain_from_bytes_success() {
        let bytes = [49, 50, 55, 46, 48, 46, 48, 46, 49];
        let result = get_domain_from_bytes(&bytes);

        match result {
            Ok((address)) => {
                assert_eq!("127.0.0.1", address);
            }

            Err(e) => unreachable!()
        }
    }

    #[test]
    fn get_port_success() {
        let bytes = [1, 2];
        let result = get_port(&bytes);

        match result {
            Ok((address)) => {
                assert_eq!(513, address);
            }

            Err(e) => unreachable!()
        }
    }

    #[test]
    fn get_dst_ipv4_address_success() {
        let bytes = [49, 50, 55, 46, 1, 2];
        let result = parse_dst_address(&bytes, &Ipv4);

        match result {
            Ok((address, port)) => {
                assert_eq!("49.50.55.46", address);
                assert_eq!(513, port);
            }

            Err(e) => unreachable!()
        }
    }

    #[test]
    fn get_dst_domain_address_success() {
        let domain = "www.baidu.com";
        let mut bytes = [13, 119, 119,
            119, 46, 98, 97, 105, 100, 117, 46, 99, 111, 109, 1, 1];

        let result = parse_dst_address(&bytes, &Domain);

        match result {
            Ok((address, port)) => {
                assert_eq!("www.baidu.com", address);
                assert_eq!(257, port);
            }
            Err(e) => unreachable!()
        }
    }

    #[test]
    fn parse_len_and_string_success() {
        let mut bytes = [13, 109, 105, 111, 45, 97, 110, 100, 45, 116, 111, 107, 105, 111];

        let result = parse_len_and_string(&bytes);

        match result {
            Ok((len, name)) => {
                assert_eq!(13 as u8, len);
                assert_eq!("mio-and-tokio", name);
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn parse_len_and_string_failed() {
        let mut bytes = [];

        let result = parse_len_and_string(&bytes);

        match result {
            Ok((len, name)) => {
                assert_eq!(13 as u8, len);
                assert_eq!("mio-and-tokio", name);
            }

            Err(msg) => assert_eq!("data is not enough.", msg)
        }
    }

    #[test]
    fn parse_user_auth_request_success() {
        let bytes = [0, 13, 109, 105, 111, 45, 97, 110, 100, 45, 116, 111, 107, 105, 111
            , 6, 49, 50, 51, 52, 53, 54];

        let result = parse_user_auth_request(&bytes);

        match result{
            Ok(request) => {
                assert_eq!(SubVersion::V0, *request.version());
                assert_eq!(13 as u8, request.u_len());
                assert_eq!("mio-and-tokio", request.name());
                assert_eq!(6, request.p_len());
                assert_eq!("123456", request.password());
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn parse_user_auth_reply_success() {
        let bytes = [0, 13, 109, 105, 111, 45, 97, 110, 100, 45, 116, 111, 107, 105, 111
            , 6, 49, 50, 51, 52, 53, 54];

        let result = parse_user_auth_request(&bytes);

        match result{
            Ok(request) => {
                assert_eq!(SubVersion::V0, *request.version());
                assert_eq!(13 as u8, request.u_len());
                assert_eq!("mio-and-tokio", request.name());
                assert_eq!(6, request.p_len());
                assert_eq!("123456", request.password());
            }

            _ => unreachable!()
        }
    }

}