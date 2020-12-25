#pragma once

int http_open(const char* host, const int port);
void http_send_req(const int fd_sock);
void http_copy_response(const int fd_sock, const int fd_file);
