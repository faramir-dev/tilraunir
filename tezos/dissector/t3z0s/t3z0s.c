// https://www.wireshark.org/docs/wsdg_html_chunked/ChDissectAdd.html

#include "config.h"
#include <epan/packet.h>

static int proto_t3z0s = -1;
static int hf_phrase = -1;
static int hf_word = -1;
static gint ett_t3z0s = -1; // Subtree

static void add_phrase(tvbuff_t *tvb, proto_tree *t_tree)
{
    const guint8 *str = NULL;
    gint len = -1;
    proto_tree_add_item_ret_string_and_length(t_tree, hf_phrase, tvb, 0, -1, ENC_UTF_8, wmem_packet_scope(), &str, &len);
}

static void add_word(tvbuff_t *tvb, proto_tree *t_tree, const int beg, const int end)
{
    const int word_length = end - beg;
    if (word_length <= 0)
        return;

    const guint8 *_str = NULL;
    gint _len = -1;
    proto_tree_add_item_ret_string_and_length(t_tree, hf_word, tvb, beg, word_length, ENC_UTF_8, wmem_packet_scope(), &_str, &_len);
}


static int is_space(const guint8 c)
{
    return c == ' ' || c == '\t' || c == '\n';
}

static void add_words(tvbuff_t *tvb, proto_tree *t_tree)
{
    const int len = tvb_captured_length(tvb);
    int prev = -1;
    for (int i = 0; i < len; ++i)
    {
        const guint8 c = tvb_get_guint8(tvb, i);
        if (is_space(c))
        {
            add_word(tvb, t_tree, prev + 1, i);
            prev = i;
        }
    }
    add_word(tvb, t_tree, prev + 1, len);
}

static int
dissect_t3z0s(tvbuff_t *tvb, packet_info *pinfo, proto_tree *tree _U_, void *data _U_)
{
    col_set_str(pinfo->cinfo, COL_PROTOCOL, "t3z0s");
    /* Clear the info column */
    col_clear(pinfo->cinfo,COL_INFO);

    proto_item *ti = proto_tree_add_item(tree, proto_t3z0s, tvb, 0, -1, ENC_NA);
    proto_tree *t_tree = proto_item_add_subtree(ti, ett_t3z0s);
    add_phrase(tvb, t_tree);
    add_words(tvb, t_tree);
    return tvb_captured_length(tvb);
}

void
proto_register_t3z0s(void)
{
    static hf_register_info hf[] = {
        { &hf_phrase,
            { "T3z0s Phrase", "t3z0s.phrase",
            FT_STRING, BASE_NONE,
            NULL, 0x0, NULL, HFILL }
        },
        { &hf_word,
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
    static dissector_handle_t t3z0s_handle;

    t3z0s_handle = create_dissector_handle(dissect_t3z0s, proto_t3z0s);
    dissector_add_uint("udp.port", 1024, t3z0s_handle);
    // dissector_add_uint("tcp.port", 1024, t3z0s_handle);
    // dissector_add_string("t3z0s.tun_node", "tun0", t3z0s_handle);
    // dissector_add_string("t3z0s.tun_proxy", "tun1", t3z0s_handle);
    // dissector_add_string("t3z0s.identity_file", "identity.json", t3z0s_handle);
}