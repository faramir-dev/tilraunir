extern crate libc;

use libc::{c_char, c_int, c_uint, c_void};
use std::{
    boxed::Box,
    collections::HashMap,
    ffi::CString,
    net::IpAddr,
    option::Option,
    net::{SocketAddr, SocketAddrV4, Ipv4Addr},
    convert::TryFrom,
};

use failure::Error;

use crypto::{
    hash::HashType,
    crypto_box::precompute,
    nonce::{NoncePair, generate_nonces},
};
use tezos_messages::p2p::{
    binary_message::{
        BinaryChunk,
        cache::CachedData,
    }
};
use std::fmt;

// TODO: DRF: Move ConnectionMessage from tezedge-debugger to some library or turn tezedge-debugger to a mod?
//mod connection_message;
//use connection_message::ConnectionMessage;
mod network;
use network::{
    connection_message::ConnectionMessage,
    msg_decoder::{EncryptedMessage, EncryptedMessageDecoder},
    raw_packet_msg::{RawPacketMessage, RawMessageDirection},
};

mod logger;
use logger::msg;

mod wireshark;
use wireshark::packet::packet_info;

mod error;
use error::{NotT3z0sStreamError, T3z0sNodeIdentityNotLoadedError, UnknownDecrypterError, PeerNotUpgradedError};

mod configuration;
use configuration::{get_configuration, Config};

// Opaque structs from Wireshark
#[repr(C)] pub struct tvbuff_t { _private: [u8; 0] }
#[repr(C)] pub struct tcp_analysis { _private: [u8; 0] }
#[repr(C)] pub struct proto_item { _private: [u8; 0] }
#[repr(C)] pub struct proto_tree { _private: [u8; 0] }
#[repr(C)] pub struct wmem_allocator_t { _private: [u8; 0] }

// Functions from Wireshark that are used by this dissector
extern "C" {
    fn tvb_get_guint8(tvb: *mut tvbuff_t, offset: c_int /* gint */) -> u8;
    fn tvb_get_ptr(tvb: *mut tvbuff_t, offset: c_int /* gint */, length: c_int /* gint */) -> *mut u8;
    fn tvb_captured_length(tvb: *mut tvbuff_t) -> c_uint /* guint */;
    fn tvb_captured_length_remaining(tvb: *mut tvbuff_t) -> c_uint /* guint */;
    fn wmem_packet_scope() -> *mut wmem_allocator_t;
    fn proto_tree_add_int64(
        proto_tree: *mut proto_tree,
        hfindex : c_int,
        tvb: *mut tvbuff_t,
        start: c_int,
        length: c_int,
        value: i64
    ) -> *mut proto_item;
    fn proto_tree_add_item_ret_string_and_length(
        proto_tree: *mut proto_tree,
        hfindex : c_int,
        tvb: *mut tvbuff_t,
        start: c_int,
        length: c_int,
        encoding: c_uint,
        scope: *mut wmem_allocator_t,
        retval: *mut *const u8,
        lenretval: *mut c_uint
    );
    fn proto_tree_add_string_format_value(
        proto_tree: *mut proto_tree,
        hfindex : c_int,
        tvb: *mut tvbuff_t,
        start: c_int,
        length: c_int,
        value: *const c_char,
        format: *const c_char,
        ...
    );
}

// Struct that represents static data on C side
#[repr(C)]
pub struct T3zosDissectorInfo {
    hf_payload_len: c_int,
    hf_packet_counter: c_int,
    hf_msg: c_int,
    hf_phrase: c_int,
    hf_word: c_int
}

// Data stored for every T3z0s stream
struct Conversation {
    counter: u64,
    /* *** PeerProcessor from Tezedge-Debugger *** */
    // addr: SocketAddr,
    conn_msgs: Vec<(ConnectionMessage, SocketAddr)>,
    is_initialized: bool,
    is_incoming: bool,
    is_dead: bool,
    waiting: bool,
    //handshake: u8,
    peer_id: String,
    public_key: Vec<u8>,
    incoming_decrypter: Option<EncryptedMessageDecoder>,
    outgoing_decrypter: Option<EncryptedMessageDecoder>,
}
impl Conversation {
    pub fn new() -> Self {
        Conversation {
            counter: 0,
            /* PeerProcessor */
            conn_msgs: Vec::with_capacity(2),
            is_initialized: false,
            is_incoming: false,
            is_dead: false,
            waiting: false,
            peer_id: Default::default(),
            public_key: Default::default(),
            incoming_decrypter: None,
            outgoing_decrypter: None,
        }
    }

    fn is_ok(&self) -> bool {
        match self.counter {
            0 => true,
            1 => self.conn_msgs.len() == 1,
            _ => self.conn_msgs.len() == 2,
        }
    }

    fn local_addr(&self) -> SocketAddr {
        assert!(self.conn_msgs.len() == 2);

        if self.is_incoming {
            self.conn_msgs[1].1
        } else {
            self.conn_msgs[0].1
        }
    }

    /*
    fn remote_addr(&self) -> SocketAddr {
        assert!(self.conn_msgs.len() == 2);

        if self.is_incoming {
            self.conn_msgs[0].1
        } else {
            self.conn_msgs[1].1
        }
    }

    fn local_conn_msg(&self) -> &ConnectionMessage {
        assert!(self.conn_msgs.len() == 2);

        if self.is_incoming {
            &self.conn_msgs[1].0
        } else {
            &self.conn_msgs[0].0
        }
    }

    fn remote_conn_msg(&self) -> &ConnectionMessage {
        assert!(self.conn_msgs.len() == 2);

        if self.is_incoming {
            &self.conn_msgs[0].0
        } else {
            &self.conn_msgs[1].0
        }
    }
    */

    fn inc_counter(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    pub fn process_connection_msg(payload: Vec<u8>) -> Result<ConnectionMessage, Error> {
        let chunk = BinaryChunk::try_from(payload)?;
        let conn_msg = ConnectionMessage::try_from(chunk)?;
        Ok(conn_msg)
    }

    fn upgrade(&mut self, configuration: &Config) -> Result<(), Error> {
        let ((first, _), (second, _)) = (&self.conn_msgs[0], &self.conn_msgs[1]);
        let first_pk = HashType::CryptoboxPublicKeyHash.bytes_to_string(&first.public_key);
        // FIXME: DRF: Use the same deserialization as in debugger.
        msg(format!("keys: first:{}; {:?}; second:{}; {:?}; configuration:{}; {}; secret-key:{}",
            HashType::CryptoboxPublicKeyHash.bytes_to_string(&first.public_key),
            first.public_key,
            HashType::CryptoboxPublicKeyHash.bytes_to_string(&second.public_key),
            second.public_key,
            HashType::CryptoboxPublicKeyHash.bytes_to_string(configuration.identity.public_key.as_bytes()),
            configuration.identity.public_key,
            configuration.identity.secret_key));
        let is_incoming = first_pk != configuration.identity.public_key;
        msg(format!("upgrade pks cmp: {} != {}", first_pk, configuration.identity.public_key));
        // FIXME: Kyras: Otocil jsem to, zda se mi, ze takto je spravne, v Debugerru je to naopak.
        let (received, sent) = if is_incoming {
            (first, second)
        } else {
            (second, first)
        };

        let sent_data = BinaryChunk::from_content(&sent.cache_reader().get().unwrap())?;
        let recv_data = BinaryChunk::from_content(&received.cache_reader().get().unwrap())?;
        msg(format!("sent_data:{:?}; recv_data:{:?}", sent_data.raw(), recv_data.raw()));

        let NoncePair { remote, local } = generate_nonces(
            &sent_data.raw(),
            &recv_data.raw(),
            is_incoming,
        );
        msg(format!("noncences: {:?};{:?}", remote, local));

        let remote_pk = HashType::CryptoboxPublicKeyHash.bytes_to_string(&received.public_key);
        msg(format!("remote_pk:{:?}", remote_pk));

        let precomputed_key = precompute(
            &hex::encode(&received.public_key),
            &configuration.identity.secret_key,
        )?;
        msg(format!("precomputed-key: received.public_key:{:?}; configuration.identity.secret_key:{:?}",
            &hex::encode(&received.public_key),
            &configuration.identity.secret_key));

        self.incoming_decrypter = Some(EncryptedMessageDecoder::new(precomputed_key.clone(), remote, remote_pk.clone()));
        self.outgoing_decrypter = Some(EncryptedMessageDecoder::new(precomputed_key, local, remote_pk.clone()));
        self.public_key = received.public_key.clone();
        self.peer_id = remote_pk;
        self.is_incoming = is_incoming;
        self.is_initialized = true;
        Ok(())
    }

    fn process_encrypted_msg(&mut self, msg: &mut RawPacketMessage) -> Result<Option<EncryptedMessage>, Error> {
        let decrypter = if msg.is_incoming() {
            &mut self.incoming_decrypter
        } else {
            &mut self.outgoing_decrypter
        };

        if let Some(ref mut decrypter) = decrypter {
            Ok(decrypter.recv_msg(msg))
        } else {
            Err(UnknownDecrypterError)?
        }
    }

    pub fn process_packet(
        self: &mut Self,
        info: &T3zosDissectorInfo, pinfo: &packet_info,
        tvb: *mut tvbuff_t, proto_tree: *mut proto_tree,
        tcpd: *const tcp_analysis
    ) -> Result<usize, Error> {

        if !self.is_ok() { Err(NotT3z0sStreamError)?; }

        let counter = self.inc_counter();
        let mut decrypted_msg = None;
        let mut dbg_direction = None;
        let mut dbg_srcaddr = None;
        let mut dbg_dstaddr = None;
        let payload = get_data_safe(tvb);
        if counter < 1 {
            assert!(false);
        } else if counter <= 2 {
            let conn_msg = Conversation::process_connection_msg(payload.to_vec())?;
            let ip_addr = IpAddr::try_from(pinfo.src)?;
            let sock_addr = SocketAddr::new(ip_addr, pinfo.srcport as u16);
            // FIXME: Can duplicate message happen? We use TCP stream, not raw packets stream.
            self.conn_msgs.push((conn_msg, sock_addr));
            if self.conn_msgs.len() == 2 {
                let configuration = get_configuration().ok_or(T3z0sNodeIdentityNotLoadedError)?;
                self.upgrade(&configuration)?;
                msg(format!("Upgraded peer! {}", self))
            }
        } else {
            let srcaddr = SocketAddr::new(IpAddr::try_from(pinfo.src)?, pinfo.srcport as u16);
            dbg_srcaddr = Some(srcaddr);
            dbg_dstaddr = Some(SocketAddr::new(IpAddr::try_from(pinfo.dst)?, pinfo.destport as u16));
            if self.is_initialized {
                msg(format!("local-addr:{}", self.local_addr()));
                let direction = if self.local_addr() == srcaddr {
                    RawMessageDirection::OUTGOING
                } else {
                    RawMessageDirection::INCOMING
                };
                dbg_direction = Some(direction);

                let mut raw = RawPacketMessage::new(
                    direction, payload
                );

                decrypted_msg = self.process_encrypted_msg(&mut raw)?;
            } else {
                Err(PeerNotUpgradedError)?;
            }
        }
        msg(format!("Conversation: {}; direction:{:?}; src-addr:{:?}; dst-addr:{:?}; decrypted-msg:{:?};", self, dbg_direction, dbg_srcaddr, dbg_dstaddr, decrypted_msg));
        proto_tree_add_string_safe(proto_tree, info.hf_msg, tvb, 0, 0, format!("payload: {}; {:?};", payload.len(), payload));
        proto_tree_add_string_safe(proto_tree, info.hf_msg, tvb, 0, 0, format!("counter: {};", counter));
        proto_tree_add_string_safe(proto_tree, info.hf_msg, tvb, 0, 0, format!("direction:{:?};", dbg_direction));
        proto_tree_add_string_safe(proto_tree, info.hf_msg, tvb, 0, 0, format!("src-addr:{:?};", dbg_srcaddr));
        proto_tree_add_string_safe(proto_tree, info.hf_msg, tvb, 0, 0, format!("dst-addr:{:?};", dbg_dstaddr));
        proto_tree_add_string_safe(proto_tree, info.hf_msg, tvb, 0, 0, format!("decrypted-msg:{:?};", decrypted_msg));
        proto_tree_add_string_safe(proto_tree, info.hf_msg, tvb, 0, 0, format!("self:{};", self));

        Ok(payload.len())
    }
}
impl fmt::Display for Conversation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "counter:{}; is_initialised:{}; is_incoming:{}; is_dead:{}; conn_msgs:{:?};", self.counter, self.is_initialized, self.is_incoming, self.is_dead,  self.conn_msgs)
    }
}


fn get_info_safe<'a>(p_info: *const T3zosDissectorInfo) -> &'a T3zosDissectorInfo {
    unsafe { &*p_info }
}

fn get_packet_info_safe<'a>(p_pinfo: *const packet_info) -> &'a packet_info {
    unsafe { &*p_pinfo }
}

fn get_data_safe<'a>(tvb: *mut tvbuff_t) -> &'a [u8] {
    unsafe {
        let ptr = tvb_get_ptr(tvb, 0, -1);
        let ulen = tvb_captured_length_remaining(tvb);
        // According to Wireshark documentation:
        //   https://www.wireshark.org/docs/wsar_html/group__tvbuff.html#ga31ba5c32b147f1f1e57dc8326e6fdc21
        // `get_raw_ptr()` should not be used, but it looks as easiest solution here.
        std::slice::from_raw_parts(
            ptr,
            ulen as usize)
    }
}

fn tvb_get_guint8_safe(tvb: *mut tvbuff_t, offset: c_int /* gint */) -> u8 {
    unsafe { tvb_get_guint8(tvb, offset) }
}

fn tvb_captured_length_safe(tvb: *mut tvbuff_t) -> c_uint {
    unsafe { tvb_captured_length(tvb) }
}

fn tvb_captured_length_remaining_safe(tvb: *mut tvbuff_t) -> c_uint {
    unsafe { tvb_captured_length_remaining(tvb) }
}

fn proto_tree_add_int64_safe(
    proto_tree: *mut proto_tree,
    hfindex : c_int,
    tvb: *mut tvbuff_t,
    start: c_int,
    length: c_int,
    value: i64
) -> *mut proto_item {
    unsafe { proto_tree_add_int64(proto_tree, hfindex, tvb, start, length, value) }
}

fn proto_tree_add_item_safe(
        proto_tree: *mut proto_tree,
        hfindex : c_int,
        tvb: *mut tvbuff_t,
        start: c_int,
        length: c_int,
        encoding: c_uint,
) {
    unsafe {
        let mut str: *const u8 = std::ptr::null_mut();
        let mut len: c_uint = 0;

        proto_tree_add_item_ret_string_and_length(
            proto_tree,
            hfindex,
            tvb,
            start,
            length,
            encoding,
            wmem_packet_scope(),
            &mut str,
            &mut len
        );
    }
}

fn proto_tree_add_string_safe(
        proto_tree: *mut proto_tree,
        hfindex : c_int,
        tvb: *mut tvbuff_t,
        start: c_int,
        length: c_int,
        value: String,
) {
    unsafe {
        let bytes_num = value.len();
        let b = value.as_bytes();

        proto_tree_add_string_format_value(
            proto_tree,
            hfindex,
            tvb,
            start,
            length,
            b.as_ptr() as *const c_char,
            b"%.*s\0".as_ptr() as *const c_char,
            bytes_num as c_int,
            b.as_ptr() as *const c_char,
        );
    }
}

static mut conversations_map: Option<HashMap<*const tcp_analysis, Conversation>> = None;

fn get_conv_map() -> &'static mut HashMap<*const tcp_analysis, Conversation> {
    unsafe { conversations_map.get_or_insert(HashMap::new()) }
}

#[no_mangle]
pub extern "C" fn t3z03s_free_conv_data(p_data: *mut c_void) {
    get_conv_map().remove(&(p_data as *const tcp_analysis));
}

#[no_mangle]
pub extern "C" fn t3z03s_dissect_packet(
        p_info: *const T3zosDissectorInfo,
        tvb: *mut tvbuff_t, proto_tree: *mut proto_tree,
        p_pinfo: *const packet_info, tcpd: *const tcp_analysis
) -> c_int {
    let info = get_info_safe(p_info);
    let pinfo = get_packet_info_safe(p_pinfo);

    let conv = get_conv_map().entry(tcpd).or_insert_with(|| Conversation::new());

    match conv.process_packet(info, pinfo, tvb, proto_tree, tcpd) {
        Err(e) => {
            msg(format!("E: Cannot process packet: {}", e));
            0 as c_int
        },
        Ok(size) => size as c_int
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
