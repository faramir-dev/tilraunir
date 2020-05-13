extern crate libc;
use libc::{c_int, c_uint};


// Opaque structs from Wireshark
#[repr(C)] pub struct tvbuff_t { _private: [u8; 0] }
#[repr(C)] pub struct proto_tree { _private: [u8; 0] }
#[repr(C)] pub struct wmem_allocator_t { _private: [u8; 0] }

// Functions from Wireshark that are used by this dissector
extern "C" {
    fn tvb_get_guint8(tvb: *mut tvbuff_t, offset: c_int /* gint */) -> u8;
    fn tvb_captured_length(tvb: *mut tvbuff_t) -> c_uint /* guint */;
    fn wmem_packet_scope() -> *mut wmem_allocator_t;
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
    hf_phrase: c_int,
    hf_word: c_int
}

fn get_info_safe<'a>(p_info: *const T3zosDissectorInfo) -> &'a T3zosDissectorInfo {
    unsafe { &*p_info }
}

fn tvb_get_guint8_safe(tvb: *mut tvbuff_t, offset: c_int /* gint */) -> u8 {
    unsafe { tvb_get_guint8(tvb, offset) }
}

fn tvb_captured_length_safe(tvb: *mut tvbuff_t) -> c_uint {
    unsafe { tvb_captured_length(tvb) }
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

#[no_mangle]
pub extern "C" fn t3z03s_dissect_packet(
        p_info: *const T3zosDissectorInfo,
        tvb: *mut tvbuff_t, proto_tree: *mut proto_tree
) -> c_int {
    let info = get_info_safe(p_info);

    let ulen = tvb_captured_length_safe(tvb);
    let len = ulen as c_int;

    proto_tree_add_item_safe(
        proto_tree,
        info.hf_phrase,
        tvb,
        0,
        -1,
        0x00000002, /* Encoding from proto.h */
    );

    add_words(info, tvb, proto_tree, len);

    len
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
