extern crate libc;
use libc::{c_int, c_uint, c_void};
use std::{
    boxed::Box,
    collections::HashMap,
    option::Option,
    net::SocketAddr,
    convert::TryFrom,
};

use failure::Error;

use crypto::{
    hash::HashType,
    crypto_box::precompute,
    nonce::{NoncePair, generate_nonces},
};
use tezos_messages::p2p::{
    binary_message::BinaryChunk,
};

// TODO: DRF: Move ConnectionMessage from tezedge-debugger to some library or turn tezedge-debugger to a mod?
mod connection_message;
use connection_message::ConnectionMessage;

mod logger;
use logger::msg;

mod wireshark;
use wireshark::packet::packet_info;

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
}

// Struct that represents static data on C side
#[repr(C)]
pub struct T3zosDissectorInfo {
    hf_payload_len: c_int,
    hf_packet_counter: c_int,
    hf_phrase: c_int,
    hf_word: c_int
}

struct Conversation {
    counter: u32,
}
impl Conversation {
    pub fn new() -> Self {
        Conversation { counter: 0 }
    }

    pub fn process_unencrypted_msg(self: &Self, payload: Vec<u8>) -> Result<ConnectionMessage, Error> {
        let chunk = BinaryChunk::try_from(payload)?;
        let conn_msg = ConnectionMessage::try_from(chunk)?;
        Ok(conn_msg)
    }
}

fn get_info_safe<'a>(p_info: *const T3zosDissectorInfo) -> &'a T3zosDissectorInfo {
    unsafe { &*p_info }
}

fn get_data_safe<'a>(tvb: *mut tvbuff_t) -> &'a [u8] {
    let ulen = tvb_captured_length_safe(tvb);
    unsafe {
        // According to Wireshark documentation:
        //   https://www.wireshark.org/docs/wsar_html/group__tvbuff.html#ga31ba5c32b147f1f1e57dc8326e6fdc21
        // `get_raw_ptr()` should not be used, but it looks as easiest solution here.
        std::slice::from_raw_parts(
            tvb_get_ptr(tvb, 0, ulen as c_int),
            ulen as usize)
    }
}

fn tvb_get_guint8_safe(tvb: *mut tvbuff_t, offset: c_int /* gint */) -> u8 {
    unsafe { tvb_get_guint8(tvb, offset) }
}

fn tvb_captured_length_safe(tvb: *mut tvbuff_t) -> c_uint {
    unsafe { tvb_captured_length(tvb) }
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

fn add_word(info: &T3zosDissectorInfo, tvb: *mut tvbuff_t, proto_tree: *mut proto_tree, wbeg: c_int, wend: c_int) {
    let wlen = wend - wbeg;
    if wlen > 0 {
        proto_tree_add_item_safe(
            proto_tree,
            info.hf_word,
            tvb,
            wbeg,
            wlen,
            0x00000002, /* Encoding from proto.h */
        );
    }
}

fn is_space(ch: char) -> bool {
    ch == ' ' || ch == '\t' || ch == '\n'
}

fn add_words(info: &T3zosDissectorInfo, tvb: *mut tvbuff_t, proto_tree: *mut proto_tree, len: c_int) {
    let mut prev_space: c_int = -1;
    for i in 0..len {
        let uch = tvb_get_guint8_safe(tvb, i);
        let ch = uch as char;
        if is_space(ch) {
            add_word(info, tvb, proto_tree, prev_space + 1, i);
            prev_space = i;
        }
    }
    add_word(info, tvb, proto_tree, prev_space + 1, len);
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
        tcpd: *const tcp_analysis
) -> c_int {
    let info = get_info_safe(p_info);

    let ulen = tvb_captured_length_safe(tvb);
    let len = ulen as c_int;

    let opt_conv = get_conv_map().get_mut(&tcpd);
    let conv = match opt_conv {
        Some(x) => x,
        None => {
            get_conv_map().insert(tcpd, Conversation::new());
            get_conv_map().get_mut(&tcpd).unwrap()
        }
    };

    conv.counter += 1;

    proto_tree_add_int64_safe(proto_tree, info.hf_payload_len, tvb, 0, 0, len as i64);
    proto_tree_add_int64_safe(proto_tree, info.hf_packet_counter, tvb, 0, 0, conv.counter as i64);

    proto_tree_add_item_safe(
        proto_tree,
        info.hf_phrase,
        tvb,
        0,
        -1,
        0x00000002, /* Encoding from proto.h */
    );

    add_words(info, tvb, proto_tree, len);

    let payload = get_data_safe(tvb);
    let conn_msg_opt = conv.process_unencrypted_msg(payload.to_vec());

    msg(format!("packet: count:{} conn_msg_opt: {:?}", conv.counter, conn_msg_opt));

    len
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
