pub mod packet;

use std::convert::{TryFrom, TryInto};
use std::net::IpAddr;
use std::slice::from_raw_parts;

impl TryFrom<packet::address> for IpAddr {
    type Error = &'static str;
    fn try_from(addr: packet::address) -> Result<Self, Self::Error> {
        let to_ip4 = || {
                let slice = unsafe { std::slice::from_raw_parts(addr.data as *const u8, 4) };
                let arr: [u8; 4] = slice.try_into().or_else(|_| Err("Cannot read IPv4 bytes"))?;
                Ok(IpAddr::from(arr))
        };
        let to_ip6 = || {
                let slice = unsafe { std::slice::from_raw_parts(addr.data as *const u8, 16) };
                let arr: [u8; 16] = slice.try_into().or_else(|_| Err("Cannot read IPv6 bytes"))?;
                Ok(IpAddr::from(arr))
        };
        match addr.type_ {
            address_type_AT_IPv4 => to_ip4(),
            address_type_AT_IPv6 => to_ip6(),
            _ => Err("Unexpected address type"),
        }
    }
}