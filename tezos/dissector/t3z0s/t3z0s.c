// https://www.wireshark.org/docs/wsdg_html_chunked/ChDissectAdd.html

#include "config.h"
#include <epan/packet.h>
#include <epan/conversation.h>
#include <epan/dissectors/packet-tcp.h>
#include <stdio.h>

/* This is shared with Rust */

struct T3zosDissectorInfo {
    int hf_payload_len;
    int hf_packet_counter;
    int hf_phrase;
    int hf_word;
};

extern int t3z03s_dissect_packet(struct T3zosDissectorInfo*, tvbuff_t*, proto_tree*, const struct tcp_analysis*);
extern int t3z03s_free_conv_data(void*);
/* End of section shared with Rust */

static dissector_handle_t t3z0s_handle;
static int proto_t3z0s = -1;
static struct T3zosDissectorInfo info = {
    -1,
    -1,
    -1,
    -1,
};
static gint ett_t3z0s = -1; // Subtree

static gboolean wmem_cb(wmem_allocator_t* allocator, wmem_cb_event_t ev, void *data)
{
    switch (ev) {
        case WMEM_CB_FREE_EVENT:
            {FILE *f = fopen("/tmp/xyz.log", "a");
            fprintf(f, "Freeing memory allocator: %p %p\n", allocator, data);
            fclose(f);}
            t3z03s_free_conv_data(data);
            break;
        case WMEM_CB_DESTROY_EVENT:
            {FILE *f = fopen("/tmp/xyz.log", "a");
            fprintf(f, "destroy: %p\n", allocator);
            fclose(f);}
    }

    return TRUE;
}

/** An old style dissector proxy.
 *
 * Proxies the old style dissector interface to the new style.
 */
static
int dissect_t3z0s_old(tvbuff_t *tvb, packet_info *pinfo, proto_tree *tree, void* data)
{
    (void)data;
    conversation_t *conv = find_or_create_conversation(pinfo);
    DISSECTOR_ASSERT_HINT(conv, "find_or_create_conversation() returned NULL");

    // FIXME: Is callback registered only once?
    struct tcp_analysis *tcpd = get_tcp_conversation_data(conv, pinfo);
    void *convd = conversation_get_proto_data(conv, proto_t3z0s);
    if (!convd)
    {
        conversation_add_proto_data(conv, proto_t3z0s, (void*)0x1);
        wmem_register_callback(wmem_file_scope(), wmem_cb, tcpd);
    }

    proto_item *ti = proto_tree_add_item(tree, proto_t3z0s, tvb, 0, -1, ENC_NA);
    proto_tree *t_tree = proto_item_add_subtree(ti, ett_t3z0s);
    proto_tree_add_int64_format(t_tree, info.hf_payload_len, tvb, 0, 0, (int64_t)conv, "T3z0s conversation: %p", conv); // XYZ: Dbg.
    {FILE *f = fopen("/tmp/xyz.log", "a");
    fprintf(f, "conv: %p %p\n", wmem_file_scope(), conv);
    fclose(f);}
    return t3z03s_dissect_packet(&info, tvb, t_tree, tcpd);
}

static gboolean
dissect_t3z0s(tvbuff_t *tvb, packet_info *pinfo, proto_tree *tree _U_, void *data _U_)
{
    conversation_t *conv = NULL;

	/*** It's ours! ***/
	conv = find_or_create_conversation(pinfo);
	/* Mark it as ours. */
    conversation_set_dissector(conv, t3z0s_handle);

    (void)dissect_t3z0s_old(tvb, pinfo, tree, data);

    return TRUE;
}

void
proto_register_t3z0s(void)
{
    static hf_register_info hf[] = {
        { &info.hf_packet_counter,
            { "T3z0s Packet Counter", "t3z0s.packet_counter",
            FT_INT64, BASE_DEC,
            NULL, 0x0, NULL, HFILL }
        },
        { &info.hf_payload_len,
            { "T3z0s Payload Length", "t3z0s.payload_len",
            FT_INT64, BASE_DEC,
            NULL, 0x0, NULL, HFILL }
        },
        { &info.hf_phrase,
            { "T3z0s Phrase", "t3z0s.phrase",
            FT_STRING, BASE_NONE,
            NULL, 0x0, NULL, HFILL }
        },
        { &info.hf_word,
            { "T3z0s Word", "t3z0s.word",
            FT_STRING, BASE_NONE,
            NULL, 0x0, NULL, HFILL }
        },
    };

    static gint *ett[] = {
        &ett_t3z0s
    };

    proto_t3z0s = proto_register_protocol (
        "T3z0s Protocol", /* name        */
        "t3z0s",          /* short name  */
        "t3z0s"           /* filter_name */
        );

    proto_register_field_array(proto_t3z0s, hf, array_length(hf));
    proto_register_subtree_array(ett, array_length(ett));
}

void
proto_reg_handoff_t3z0s(void)
{
    t3z0s_handle = create_dissector_handle(dissect_t3z0s_old, proto_t3z0s);
    heur_dissector_add("tcp", dissect_t3z0s, "T3z0s", "t3z0s_tcp", proto_t3z0s, HEURISTIC_ENABLE);
    // dissector_add_uint("udp.port", 1024, t3z0s_handle);
    // dissector_add_uint("tcp.port", 1024, t3z0s_handle);
    // dissector_add_string("t3z0s.tun_node", "tun0", t3z0s_handle);
    // dissector_add_string("t3z0s.tun_proxy", "tun1", t3z0s_handle);
    // dissector_add_string("t3z0s.identity_file", "identity.json", t3z0s_handle);
}