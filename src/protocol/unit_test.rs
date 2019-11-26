mod unit_test {
    use crate::protocol::packet;
    use crate::protocol::packet::{parse_version, Version, AddressType, get_ipv4_from_bytes
                                  , get_domain_from_bytes, get_port, parse_dst_address};
    use crate::protocol::packet::AuthType::Non;
    use crate::protocol::packet::AddressType::Ipv4;

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
            Ok(addr) => assert_eq!("127.0.0.1", addr),
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
    fn get_dst_address_ipv4_success() {
        let bytes = [49, 50, 55, 46, 1,2];
        let result = parse_dst_address(&bytes, &Ipv4);

        match result {
            Ok((address, port)) => {
                assert_eq!("49.50.55.46", address);
                assert_eq!(513, port);
            }

            Err(e) => unreachable!()
        }
    }
}