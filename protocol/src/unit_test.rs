mod unit_test {
    use crate::packet::*;
    use crate::packet::AddressType::{Ipv4, Domain};

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

        match result {
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
        let bytes = [0, 0];

        let result = parse_user_auth_reply(&bytes);

        match result {
            Ok(reply) => {
                assert_eq!(SubVersion::V0, *reply.version());
                assert_eq!(AuthResult::Success, *reply.status());
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn encode_auth_select_reply_success() {
        let reply =
            AuthSelectReply::new(Version::Socks5, AuthType::Non);
        let data = encode_auth_select_reply(&reply);

        match data {
            Ok(buffer) => {
                let bytes = buffer.as_slice();
                assert_eq!(5, bytes[0]);
                assert_eq!(0, bytes[1]);
            }
            _ => unreachable!()
        }
    }

    #[test]
    fn encode_auth_select_reply_success_failed() {
        let reply =
            AuthSelectReply::new(Version::Socks5, AuthType::Non);

        let data = encode_auth_select_reply(&reply);

        match data {
            Ok(buffer) => {
                let bytes = buffer.as_slice();
                assert_eq!(5, bytes[0]);
                assert_eq!(0, bytes[1]);
            }
            Err(err) => assert_eq!("proxy only support version 5.", err)
        }
    }

    #[test]
    fn encode_dst_service_reply_success() {
        let reply = DstServiceReply::new(Version::Socks5
                                         , ReplyType::Success, AddressType::Ipv4
                                         ,"127.0.0.1".to_string(), 1024);

        let data = encode_dst_service_reply(reply);

        match data{
            Ok(buffer) => {
                let bytes = buffer.as_slice();
                println!("bytes:{:?}", bytes);
                assert_eq!(5, bytes[0]);
                assert_eq!(0, bytes[1]);
                assert_eq!(0, bytes[2]);
                assert_eq!(1, bytes[3]);
                assert_eq!(127, bytes[4]);
                assert_eq!(0, bytes[5]);
                assert_eq!(0, bytes[6]);
                assert_eq!(1, bytes[7]);
                assert_eq!(0, bytes[8]);
                assert_eq!(4, bytes[9]);
            }

            Err(err) => unreachable!()
        }
    }

    #[test]
    fn encode_auth_select_request_success(){
        let mut auth_types = Vec::<AuthType>::new();
        auth_types.push(AuthType::Non);
        auth_types.push(AuthType::NamePassword);
        let request = AuthSelectRequest::new(Version::Socks5
                                             ,2, auth_types);

        let data = encode_auth_select_request(request);

        match data {
            Ok(buffer) => {
                let bytes = buffer.as_slice();
                assert_eq!(5, bytes[0]);
                assert_eq!(2, bytes[1]);
                assert_eq!(0, bytes[2]);
                assert_eq!(2, bytes[3]);
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn  encode_dst_service_request_success(){
        let request = DstServiceRequest::new(
            Version::Socks5, CmdType::Connect, 0
            , AddressType::Ipv4,"127.0.0.1".to_string(), 1025);

        let data = encode_dst_service_request(request);

        match data {
            Ok(buffer) => {
                let bytes = buffer.as_slice();
                println!("bytes:{:?}", bytes);
                assert_eq!(5, bytes[0]);
                assert_eq!(1, bytes[1]);
                assert_eq!(0, bytes[2]);
                assert_eq!(1, bytes[3]);
                assert_eq!(127, bytes[4]);
                assert_eq!(0, bytes[5]);
                assert_eq!(0, bytes[6]);
                assert_eq!(1, bytes[7]);
                assert_eq!(1, bytes[8]);
                assert_eq!(4, bytes[9]);
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn  encode_dst_request_with_domain_success(){
        let request = DstServiceRequest::new(
            Version::Socks5, CmdType::Connect, 0
            , AddressType::Domain,"127.0.0.1".to_string(), 1025);

        let data = encode_dst_service_request(request);

        match data {
            Ok(buffer) => {
                let bytes = buffer.as_slice();
                println!("bytes:{:?}", bytes);
                assert_eq!(5, bytes[0]);
                assert_eq!(1, bytes[1]);
                assert_eq!(0, bytes[2]);
                assert_eq!(3, bytes[3]);
                assert_eq!(49, bytes[4]);
                assert_eq!(50, bytes[5]);
                assert_eq!(55, bytes[6]);
                assert_eq!(46, bytes[7]);

                assert_eq!(48, bytes[8]);
                assert_eq!(46, bytes[9]);
                assert_eq!(48, bytes[10]);
                assert_eq!(46, bytes[11]);
                assert_eq!(49, bytes[12]);

                assert_eq!(1, bytes[13]);
                assert_eq!(4, bytes[14]);
            }

            _ => unreachable!()
        }
    }

    #[test]
    fn handle_init_test(){

    }
}