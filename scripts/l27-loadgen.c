/*
 * l27-loadgen.c — KD-2 paired-bench load generator (lane L27).
 *
 * Recreates the RED-10 (red-10-timing-jitter-exploit.md W4) load classes:
 *   bw  <threads>  : per-thread streaming read+write over a private 512 MiB
 *                    buffer at 64B stride — LLC + DRAM saturation.
 *   cpu <threads>  : mixed int/FP dependent chains with data-dependent
 *                    branches — issue-port + branch-predictor contention.
 *   combined <bwthreads> <cputhreads>
 *
 * No sleeps: each thread runs a fixed iteration-count work loop launched
 * under the bench window; termination is cooperative via a SIGTERM-safer
 * volatile flag on SIGINT (the shell kills the process group after the
 * paired round). Repo untouched; binary built to /tmp by the driver script.
 *
 * KD-COVERS: KD-2
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <signal.h>

static volatile sig_atomic_t g_stop = 0;
static void on_int(int sig) { (void)sig; g_stop = 1; }

#define BUF_BYTES (512ULL * 1024ULL * 1024ULL)
#define STRIDE 64

static void *bw_thread(void *arg) {
    (void)arg;
    unsigned char *buf = malloc(BUF_BYTES);
    if (!buf) return NULL;
    memset(buf, 0x5A, BUF_BYTES);
    unsigned long long sink = 0;
    while (!g_stop) {
        for (size_t off = 0; off + STRIDE <= BUF_BYTES; off += STRIDE) {
            buf[off] = (unsigned char)(buf[off] ^ buf[off + (STRIDE - 1)]);
            sink += buf[off];
        }
    }
    FILE *devnull = fopen("/dev/null", "w");
    if (devnull) { fprintf(devnull, "%llu", sink); fclose(devnull); }
    free(buf);
    return NULL;
}

static void *cpu_thread(void *arg) {
    (void)arg;
    unsigned long seed = 0x9E3779B97F4A7C15ULL ^ (unsigned long)pthread_self();
    unsigned long long sink = 0;
    while (!g_stop) {
        unsigned long a = seed;
        for (int i = 0; i < 100000; i++) {
            a = a * 6364136223846793005ULL + 1442695040888963407ULL;
            unsigned long b = a >> 33;
            double f = (double)(b & 0xFFFFF) * 0.0009765625;
            f = f * f + (double)(b & 0xFF);
            if ((b & 1) == 0) sink += (unsigned long)f;
            else sink ^= b;
        }
        seed = a;
    }
    FILE *devnull = fopen("/dev/null", "w");
    if (devnull) { fprintf(devnull, "%llu", sink); fclose(devnull); }
    return NULL;
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: %s bw <threads> | cpu <threads> | combined <bw> <cpu>\n", argv[0]);
        return 2;
    }
    signal(SIGINT, on_int);
    signal(SIGTERM, on_int);

    pthread_t tids[128];
    int n = 0;
    if (!strcmp(argv[1], "bw") && argc == 3) {
        int k = atoi(argv[2]);
        for (int i = 0; i < k && n < 128; i++) pthread_create(&tids[n++], NULL, bw_thread, NULL);
    } else if (!strcmp(argv[1], "cpu") && argc == 3) {
        int k = atoi(argv[2]);
        for (int i = 0; i < k && n < 128; i++) pthread_create(&tids[n++], NULL, cpu_thread, NULL);
    } else if (!strcmp(argv[1], "combined") && argc == 4) {
        int kb = atoi(argv[2]), kc = atoi(argv[3]);
        for (int i = 0; i < kb && n < 128; i++) pthread_create(&tids[n++], NULL, bw_thread, NULL);
        for (int i = 0; i < kc && n < 128; i++) pthread_create(&tids[n++], NULL, cpu_thread, NULL);
    } else {
        fprintf(stderr, "bad args\n");
        return 2;
    }
    fprintf(stderr, "l27-loadgen: %d threads launched\n", n);
    for (int i = 0; i < n; i++) pthread_join(tids[i], NULL);
    return 0;
}
