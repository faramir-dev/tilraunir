
#define _GNU_SOURCE

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>
#include <linux/types.h>
#include <netinet/in.h>
#include <netinet/ip.h>
#include <netinet/tcp.h>
#include <linux/netfilter.h>		/* for NF_ACCEPT */
#include <errno.h>

#include <libnetfilter_queue/pktbuff.h>
#include <libnetfilter_queue/libnetfilter_queue.h>
#include <libnetfilter_queue/libnetfilter_queue_ipv4.h>
#include <libnetfilter_queue/libnetfilter_queue_tcp.h>


// TODO: This program replaces all occurences of "ahoj" with "hola" in TCP packet payload.
//       It assumes that "ahoj" doesn't cross packet boundary, otherwise it is not replaced.

// FIXME: What about return values from functions? Are they assigned to variables with correct type?

/// Print buffer with binary data in human-readable form
static void print_buf(const unsigned char *buf, const size_t len)
{
    for (size_t i = 0; i < len; ++i)
    {
        const int c = buf[i];
        if (0x20 <= c && c < 0x7F)
        {
            fputc(c, stdout);
        }
        else
        {
            fprintf(stdout, "\\x%.02d", c);
        }
    }
}

/// Rewrite buffer: Find all occurences of "ahoj" and replace them with "hola".
/// @return true/false whether buffer has been rewritten
static bool rewrite_buf(unsigned char *buf, const size_t len)
{
    bool is_changed = false;

    static const char STR_AHOJ[] = "ahoj";
    static const char STR_HOLA[] = "hola";
    static const size_t LEN = sizeof(STR_AHOJ) - 1;
    for (char *p = buf, *end = p + len, *q = NULL; p < end; p = q + LEN)
    {
        q = memmem(p, end - p, STR_AHOJ, LEN);
        if (NULL == q)
            break;
        memcpy(q, STR_HOLA, LEN);
       is_changed = true;
    }

    return is_changed;
}

/// If the packet is TCP and if its payload contains at least one string "ahoj" then rewrite it.
/// Do not touch the payload otherwise.
/// This function calls `nfq_set_verdict()`
/// @return Return value of `nfq_set_verdict()`

static int rewrite_payload(struct nfq_q_handle *qh, const int id, unsigned char* rawData, const int len)
{
#define CHECK($COND, $MSG) \
if ($COND) \
{ \
    fprintf(stderr, "%s:%d %s\n", __FILE__, __LINE__, ($MSG)); \
    goto exit; \
}

    bool is_changed = false;

    struct pkt_buff * pkBuff = pktb_alloc(AF_INET, rawData, len, 0x1000);
    CHECK (NULL == pkBuff, "Issue while pktb allocate");

    struct iphdr *ip = nfq_ip_get_hdr(pkBuff);
    CHECK (NULL == ip, "Issue while ipv4 header parse");

    CHECK (nfq_ip_set_transport_header(pkBuff, ip) < 0, "Can't set transport header");

    if(ip->protocol == IPPROTO_TCP)
    {
        struct tcphdr *tcp = nfq_tcp_get_hdr(pkBuff);
        CHECK (NULL == tcp, "Issue while tcp header");

        unsigned char *tcp_payload = nfq_tcp_get_payload(tcp, pkBuff);
        CHECK (NULL == tcp_payload, "Issue while payload");

        unsigned int tcp_payload_len = nfq_tcp_get_payload_len(tcp, pkBuff);
        if (tcp_payload_len < 4*tcp->th_off)
        {
            fprintf(stderr, "payloadLen < 4*thoff: %u < %u", tcp_payload_len, 4*tcp->th_off);
            goto exit;
        }
        tcp_payload_len -= 4*tcp->th_off;

        print_buf(tcp_payload, tcp_payload_len);
   	    fputc('\n', stdout);

        is_changed = rewrite_buf(tcp_payload, tcp_payload_len);
        if (is_changed)
        {
            printf("Buffer changed\n");
            nfq_tcp_compute_checksum_ipv4(tcp, ip);
        }
    }

#undef CHECK
exit:
    {
        const int ret = is_changed ? nfq_set_verdict(qh, id, NF_ACCEPT, pktb_len(pkBuff), pktb_data(pkBuff))
                                   : nfq_set_verdict(qh, id, NF_ACCEPT, 0, NULL);

        if (NULL != pkBuff)
            pktb_free(pkBuff); // Don't forget to clean up

        return ret;
    }
}

/// Callback for Netfilter Queue
static int cb(struct nfq_q_handle *qh, struct nfgenmsg *nfmsg,
	    	  struct nfq_data *nfa, void *_data)
{
    printf("entering callback\n");

    int id = 0;
    struct nfqnl_msg_packet_hdr *ph;
    struct nfqnl_msg_packet_hw *hwph;
    uint32_t mark, ifi, uid, gid;
    int ret;
    unsigned char *data, *secdata;
    struct nfq_data *tb = nfa;
    bool is_changed = false;

    ph = nfq_get_msg_packet_hdr(tb);
    if (ph)
    {
        id = ntohl(ph->packet_id);
        printf("hw_protocol=0x%04x hook=%u id=%u ",
        ntohs(ph->hw_protocol), ph->hook, id);
    }

    hwph = nfq_get_packet_hw(tb);
    if (hwph)
    {
        int i, hlen = ntohs(hwph->hw_addrlen);

        printf("hw_src_addr=");
        for (i = 0; i < hlen-1; i++)
            printf("%02x:", hwph->hw_addr[i]);
        printf("%02x ", hwph->hw_addr[hlen-1]);
    }

    mark = nfq_get_nfmark(tb);
    if (mark)
        printf("mark=%u ", mark);

    ifi = nfq_get_indev(tb);
    if (ifi)
        printf("indev=%u ", ifi);

    ifi = nfq_get_outdev(tb);
    if (ifi)
        printf("outdev=%u ", ifi);
    ifi = nfq_get_physindev(tb);
    if (ifi)
        printf("physindev=%u ", ifi);

    ifi = nfq_get_physoutdev(tb);
    if (ifi)
        printf("physoutdev=%u ", ifi);

    if (nfq_get_uid(tb, &uid))
        printf("uid=%u ", uid);

    if (nfq_get_gid(tb, &gid))
        printf("gid=%u ", gid);

    ret = nfq_get_secctx(tb, &secdata);
    if (ret > 0)
        printf("secctx=\"%.*s\" ", ret, secdata);

    ret = nfq_get_payload(tb, &data);

    printf("payload_len=%d ", ret);
    int const verdict = (ret >= 0) ? rewrite_payload(qh, id, data, ret)
                                   : nfq_set_verdict(qh, id, NF_ACCEPT, 0, NULL);

    printf("callback finished\n");
    return verdict;
}

int main(int argc, char **argv)
{
    struct nfq_handle *h;
    struct nfq_q_handle *qh;
    int fd;
    int rv;
    uint32_t queue = 0;
    char buf[4096] __attribute__ ((aligned));

    if (argc == 2)
    {
        queue = atoi(argv[1]);
        if (queue > 65535)
        {
            fprintf(stderr, "Usage: %s [<0-65535>]\n", argv[0]);
            exit(EXIT_FAILURE);
        }
    }

    printf("opening library handle\n");
    h = nfq_open();
    if (!h)
    {
        fprintf(stderr, "error during nfq_open()\n");
        exit(1);
	}

    printf("unbinding existing nf_queue handler for AF_INET (if any)\n");
    if (nfq_unbind_pf(h, AF_INET) < 0)
    {
        fprintf(stderr, "error during nfq_unbind_pf()\n");
        exit(1);
    }

    printf("binding nfnetlink_queue as nf_queue handler for AF_INET\n");
    if (nfq_bind_pf(h, AF_INET) < 0)
    {
        fprintf(stderr, "error during nfq_bind_pf()\n");
        exit(1);
    }

    printf("binding this socket to queue '%d'\n", queue);
    qh = nfq_create_queue(h, queue, &cb, NULL);
    if (!qh)
    {
        fprintf(stderr, "error during nfq_create_queue()\n");
        exit(1);
    }

    printf("setting copy_packet mode\n");
    if (nfq_set_mode(qh, NFQNL_COPY_PACKET, 0xffff) < 0) {
        fprintf(stderr, "can't set packet_copy mode\n");
        exit(1);
    }

    printf("setting flags to request UID and GID\n");
    if (nfq_set_queue_flags(qh, NFQA_CFG_F_UID_GID, NFQA_CFG_F_UID_GID)) {
        fprintf(stderr, "This kernel version does not allow to "
        "retrieve process UID/GID.\n");
    }

    printf("setting flags to request security context\n");
    if (nfq_set_queue_flags(qh, NFQA_CFG_F_SECCTX, NFQA_CFG_F_SECCTX))
    {
        fprintf(stderr, "This kernel version does not allow to "
                "retrieve security context.\n");
    }

    printf("Waiting for packets...\n");

    fd = nfq_fd(h);

    for (;;)
    {
        if ((rv = recv(fd, buf, sizeof(buf), 0)) >= 0)
        {
            printf("pkt received\n");
            nfq_handle_packet(h, buf, rv);
            continue;
        }
        /* if your application is too slow to digest the packets that
         * are sent from kernel-space, the socket buffer that we use
         * to enqueue packets may fill up returning ENOBUFS. Depending
         * on your application, this error may be ignored. Please, see
         * the doxygen documentation of this library on how to improve
         * this situation.
         */
        if (rv < 0 && errno == ENOBUFS)
        {
            printf("losing packets!\n");
            continue;
        }
        perror("recv failed");
        break;
    }

    printf("unbinding from queue 0\n");
    nfq_destroy_queue(qh);

#ifdef INSANE
    /* normally, applications SHOULD NOT issue this command, since
     * it detaches other programs/sockets from AF_INET, too ! */
    printf("unbinding from AF_INET\n");
    nfq_unbind_pf(h, AF_INET);
#endif

    printf("closing library handle\n");
    nfq_close(h);

    return 0;
}
