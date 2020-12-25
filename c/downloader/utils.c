#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#include "config.h"
#include "utils.h"


void init_rand()
{
    srand(time(NULL));
}

void exit_if_true(int cond, int err_code)
{
    if (cond)
    {
        exit(err_code);
    }
}

size_t user_agent(char* buf)
{
    const int len = snprintf(buf, buf ? ~(size_t)0 : (size_t)0, DOWNLOADER_USER_AGENT, DOWNLOADER_UNIQ_ID);
    exit_if_true(len < 0, ERR_INTERNAL);
    return (size_t)(len + 1);
}

void check_uid(void)
{
    // FIXME: getuid() or geteuid() ?
    const uid_t uid = geteuid();
    exit_if_true (uid != DOWNLOADER_UID, ERR_INVALID_UID);
}

void remove_yourself()
{
    const size_t MAX_SIZE=32*1024;
    size_t bufsize = 8;
    for(; bufsize < MAX_SIZE; bufsize *= 2)
    {
        char buf[bufsize];
        const ssize_t ret = readlink("/proc/self/exe", buf, bufsize);
        if (ret < 0)
        {
            exit(ERR_CANNOT_READ_LINK);
        }
        if ((size_t)ret < bufsize)
        {
            buf[ret] = 0;
            if(unlink(buf))
            {
                exit(ERR_CANNOT_RM);
            }
            break;
        }
    }
    if (bufsize >= MAX_SIZE)
    {
        exit(ERR_CANNOT_READ_LINK);
    }
}

void path_gen(char *path, const size_t len)
{
    if (!len)
    {
        exit(ERR_INTERNAL);
    }

    for (size_t i = 0; i < len - 1; ++i)
    {
        path[i] = rand() % 26 + 97;
    }
    path[len - 1] = 0;
}

void path_remove(const char *path)
{
    if (unlink(path))
    {
        exit(ERR_CANNOT_RM);
    }
}

int path_execute(const char *url_path, const char *fpath)
{
    const pid_t pid = fork();
    switch (pid)
    {
    case -1:
        exit(ERR_FORK);

    case 0:
    {
        char buf_path[strlen(url_path) + 1];
        strcpy(buf_path, url_path);

        char* const argv[] = {buf_path, NULL};
        char* const env[] = {NULL};

        execve(fpath, argv, env);
        exit(ERR_CANNOT_EXEC);
    }

    default:
    {
        int status = 0;
        if (waitpid(pid, &status, 0) == -1)
        {
            exit(ERR_WAIT);
        }
        exit_if_true(!WIFEXITED(status), ERR_STATUS);
        return WEXITSTATUS(status);
    }
    }
}

int fd_create_path(const char *path)
{
    const int fd = open(path, O_WRONLY | O_CLOEXEC | O_CREAT | O_EXCL, S_IRWXU);
    if (fd < 0)
    {
        exit(ERR_FILE);
    }

    return fd;
}

size_t fd_write(const int fd, const void *data, const size_t len)
{
    const ssize_t ret = write(fd, data, len);
    if (ret < 0)
    {
        exit(ERR_FILE);
    }
    return (size_t)ret;
}

void fd_flush(const int fd)
{
    if (fsync(fd))
    {
        exit(ERR_FILE);
    }
}

void fd_close(const int fd)
{
    close(fd);
}
