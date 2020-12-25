#include <netdb.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>
#include <zlib.h>

#include "config.h"
#include "http.h"
#include "utils.h"

static void writeall(const int sock, const char* buf, const size_t total_len)
{
    for(size_t idx = 0; idx < total_len; )
    {
        const ssize_t ret = write(sock, &buf[idx], total_len - idx);
        exit_if_true(ret < 0, ERR_HTTP);
        idx +=(size_t)ret;
    }
}

static size_t readsome(const int sock, char* buf, const size_t len)
{
    const ssize_t ret = read(sock, buf, len);
    exit_if_true(ret < 0, ERR_HTTP);
    return (size_t)ret;
}

int http_open(const char *host, const int port)
{
    int sock = -1;
    struct addrinfo* addrs = NULL;
    STK_PRINTF(str_port, "%d", port);

    struct addrinfo hint;
    memset(&hint, 0, sizeof(hint));
    hint.ai_family = AF_UNSPEC;    /* Allow IPv4 or IPv6 */
    hint.ai_socktype = SOCK_STREAM; /* Datagram socket */
    hint.ai_flags = AI_PASSIVE;    /* For wildcard IP address */
    hint.ai_protocol = 0;          /* Any protocol */
    hint.ai_canonname = NULL;
    hint.ai_addr = NULL;
    hint.ai_next = NULL;

    exit_if_true(getaddrinfo(host, str_port, &hint, &addrs), ERR_HTTP);
    for (struct addrinfo* ai = addrs; ai != NULL; ai = ai->ai_next)
    {
        if (-1 == (sock = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol)))
        {
            continue;
        }

        if (connect(sock, ai->ai_addr, ai->ai_addrlen))
        {
            close(sock);
            sock = -1;
            continue;
        }

        break;
    }

    freeaddrinfo(addrs);
    exit_if_true(sock == -1, ERR_HTTP);
    return sock;
}

void http_send_req(const int fd_sock)
{
    const size_t user_agent_size = user_agent(NULL);
    char str_user_agent[user_agent_size];
    user_agent(str_user_agent);

    STK_PRINTF(req, "GET %s HTTP/1.1\nHost: %s\nUser-Agent: %s\n\n",
               DOWNLOADER_URL_PATH,
               DOWNLOADER_HOST,
               str_user_agent);
    writeall(fd_sock, req, sizeof(req) - 1);
}

void http_copy_response(const int fd_sock, const int fd_file)
{
    // FIXME: Read status line of the response and exit with error if != 200
    // FIXME: Handle seqeunce \n\r\n\r correctly.
    char buf[256];
    char deflate_buf[2*sizeof(buf)];

    int newlines = 0;
    size_t len = 0;

    z_stream zstrm;
    memset(&zstrm, 0, sizeof(zstrm));
    exit_if_true(Z_OK != inflateInit2(&zstrm, 15+32), ERR_ZIP);
    int inflate_ret = 0;
    do
    {
        len = readsome(fd_sock, buf, sizeof(buf));
        size_t idx = 0;
        if (newlines < 2 && len > 0)
        {
            do
            {
                switch (buf[idx])
                {
                case '\n':
                    ++newlines;
                    break;
                case '\r':
                    break;
                default:
                    newlines = 0;
                    break;
                }
            }
            while (++idx < len && newlines < 2);
        }
        const int flush = len == 0 ? Z_FINISH : Z_NO_FLUSH;
        if (newlines == 2)
        {
            zstrm.avail_in = len - idx;
            zstrm.next_in = (unsigned char*)&buf[idx];
            do
            {
                zstrm.avail_out = sizeof(deflate_buf);
                zstrm.next_out = (unsigned char*)deflate_buf;
                inflate_ret = inflate(&zstrm, flush);
                exit_if_true(inflate_ret == Z_NEED_DICT ||
                             inflate_ret == Z_DATA_ERROR ||
                             inflate_ret == Z_MEM_ERROR,
                             ERR_ZIP);
                fd_write(fd_file, deflate_buf, sizeof(deflate_buf) - zstrm.avail_out);
            }
            while (zstrm.avail_out == 0);
        }
    }
    while (len > 0 && inflate_ret != Z_STREAM_END);
    inflateEnd(&zstrm);
}
