#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static unsigned long long number(const char *text) {
    char *end = NULL;
    unsigned long long value = strtoull(text, &end, 10);
    if (!text[0] || !end || *end) {
        fprintf(stderr, "invalid number: %s\n", text);
        exit(2);
    }
    return value;
}

static void touch_memory(size_t bytes) {
    volatile unsigned char *memory = mmap(NULL, bytes, PROT_READ | PROT_WRITE,
                                          MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (memory == MAP_FAILED) {
        perror("mmap");
        exit(3);
    }
    for (size_t offset = 0; offset < bytes; offset += 4096)
        memory[offset] = (unsigned char)(offset >> 12);
}

static void write_heartbeat(void) {
    int output = open("heartbeat", O_WRONLY | O_CREAT | O_TRUNC, 0600);
    if (output < 0)
        _exit(4);
    for (;;) {
        char value[64];
        int length = snprintf(value, sizeof(value), "%lld\n", (long long)time(NULL));
        if (pwrite(output, value, (size_t)length, 0) < 0)
            _exit(5);
        fsync(output);
        usleep(20000);
    }
}

int main(int argc, char **argv) {
    if (argc < 2)
        return 2;
    if (!strcmp(argv[1], "normal")) {
        size_t bytes = argc > 2 ? (size_t)number(argv[2]) : 32U * 1024U * 1024U;
        touch_memory(bytes);
        puts("normal-ok");
        return 0;
    }
    if (!strcmp(argv[1], "memory")) {
        touch_memory((size_t)number(argv[2]));
        sleep(2);
        puts("memory-unexpectedly-survived");
        return 6;
    }
    if (!strcmp(argv[1], "pids")) {
        unsigned long long requested = number(argv[2]);
        pid_t *children = calloc((size_t)requested, sizeof(*children));
        unsigned long long created = 0;
        if (!children)
            return 7;
        while (created < requested) {
            pid_t child = fork();
            if (child < 0) {
                if (errno != EAGAIN) {
                    perror("fork");
                    return 8;
                }
                break;
            }
            if (!child) {
                pause();
                _exit(0);
            }
            children[created++] = child;
        }
        for (unsigned long long index = 0; index < created; ++index)
            kill(children[index], SIGKILL);
        for (unsigned long long index = 0; index < created; ++index)
            waitpid(children[index], NULL, 0);
        printf("pids-created=%llu requested=%llu errno=%d\n", created, requested, errno);
        return created < requested && errno == EAGAIN ? 0 : 9;
    }
    if (!strcmp(argv[1], "output")) {
        unsigned long long remaining = number(argv[2]);
        char block[4096];
        memset(block, 'x', sizeof(block));
        while (remaining) {
            size_t count = remaining < sizeof(block) ? (size_t)remaining : sizeof(block);
            ssize_t written = write(STDOUT_FILENO, block, count);
            if (written < 0)
                return errno == EFBIG ? 0 : 10;
            remaining -= (unsigned long long)written;
        }
        return 11;
    }
    if (!strcmp(argv[1], "artifact")) {
        unsigned long long remaining = number(argv[2]);
        char block[4096];
        memset(block, 'a', sizeof(block));
        int output = open("large-artifact.bin", O_WRONLY | O_CREAT | O_TRUNC, 0600);
        if (output < 0)
            return 14;
        while (remaining) {
            size_t count = remaining < sizeof(block) ? (size_t)remaining : sizeof(block);
            ssize_t written = write(output, block, count);
            if (written < 0)
                return 15;
            remaining -= (unsigned long long)written;
        }
        close(output);
        puts("artifact-ok");
        return 0;
    }
    if (!strcmp(argv[1], "detach")) {
        pid_t child = fork();
        if (child < 0)
            return 12;
        if (!child) {
            setsid();
            write_heartbeat();
        }
        puts("detached");
        return 0;
    }
    if (!strcmp(argv[1], "sleep")) {
        sleep((unsigned int)number(argv[2]));
        return 13;
    }
    return 2;
}
