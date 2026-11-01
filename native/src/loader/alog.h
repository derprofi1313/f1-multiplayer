#pragma once
#include <windows.h>
#include <stdio.h>
#include <string.h>

/* Asynchronous loader.log writer. Callers format the body on their own thread and push it with
   its UTC time and thread id into a bounded lock-free ring (Vyukov MPMC slots, one consumer); a
   background thread adds the local-time prefix and writes the lines in batches of up to 64 KB,
   waking every ALOG_PERIOD_MS or when the ring is half full. A producer that finds the ring full
   drains it itself under the drain lock and retries (a slot claimed by a preempted producer holds the
   drain back until it commits), so no line is dropped and each thread's lines keep their order.
   alog_flush() writes everything queued before it returns (crash lines call it), and
   alog_flush_at_exit() drains without waiting on a writer thread the process already killed. */

#define ALOG_SLOTS 1024u
#define ALOG_TEXT 2048u
#define ALOG_LINE 2048u
#define ALOG_BATCH (64u * 1024u)
#define ALOG_PERIOD_MS 50u
#define ALOG_FLUSH_WAIT_MS 250u

typedef struct {
    unsigned long long seq;
    FILETIME utc;
    DWORD tid;
    unsigned int len;
    char text[ALOG_TEXT];
} AlogSlot;

typedef struct {
    HANDLE file;
    HANDLE wake;
    HANDLE thread;
    DWORD thread_id;
    CRITICAL_SECTION drain_lock;
    AlogSlot *slots;
    unsigned long long head __attribute__((aligned(64)));
    unsigned long long tail __attribute__((aligned(64)));
    volatile LONG woken;
    volatile LONG stop;
    volatile LONG live;
    volatile LONG helped;
    volatile LONG bypassed;
    char batch[ALOG_BATCH];
    unsigned int batch_len;
} Alog;

static Alog alog_state;

static int alog_format_line(char *out, const char *body, unsigned int n, const FILETIME *utc, DWORD tid)
{
    FILETIME local;
    SYSTEMTIME t;
    if (!FileTimeToLocalFileTime(utc, &local) || !FileTimeToSystemTime(&local, &t))
        memset(&t, 0, sizeof t);
    int k = snprintf(out, ALOG_LINE, "%04u-%02u-%02u %02u:%02u:%02u.%03u [%5lu] ", t.wYear, t.wMonth, t.wDay, t.wHour,
                     t.wMinute, t.wSecond, t.wMilliseconds, tid);
    if (k < 0)
        k = 0;
    if (n > ALOG_LINE - (unsigned int)k - 3)
        n = ALOG_LINE - (unsigned int)k - 3;
    memcpy(out + k, body, n);
    k += (int)n;
    out[k++] = '\r';
    out[k++] = '\n';
    return k;
}

static void alog_write_batch(Alog *a)
{
    DWORD w;
    if (a->batch_len)
        WriteFile(a->file, a->batch, a->batch_len, &w, NULL);
    a->batch_len = 0;
}

static unsigned int alog_drain_locked(Alog *a)
{
    unsigned int count = 0;
    for (;;) {
        unsigned long long pos = a->tail;
        AlogSlot *s = &a->slots[pos & (ALOG_SLOTS - 1)];
        unsigned long long seq = __atomic_load_n(&s->seq, __ATOMIC_ACQUIRE);
        if (seq != pos + 1)
            break;
        if (a->batch_len + ALOG_LINE > ALOG_BATCH)
            alog_write_batch(a);
        a->batch_len += (unsigned int)alog_format_line(a->batch + a->batch_len, s->text, s->len, &s->utc, s->tid);
        __atomic_store_n(&s->seq, pos + ALOG_SLOTS, __ATOMIC_RELEASE);
        __atomic_store_n(&a->tail, pos + 1, __ATOMIC_RELEASE);
        count++;
    }
    alog_write_batch(a);
    return count;
}

static DWORD WINAPI alog_writer(LPVOID p)
{
    Alog *a = (Alog *)p;
    while (!__atomic_load_n(&a->stop, __ATOMIC_ACQUIRE)) {
        WaitForSingleObject(a->wake, ALOG_PERIOD_MS);
        __atomic_store_n(&a->woken, 0, __ATOMIC_RELEASE);
        EnterCriticalSection(&a->drain_lock);
        alog_drain_locked(a);
        LeaveCriticalSection(&a->drain_lock);
    }
    return 0;
}

static int alog_start(HANDLE file)
{
    Alog *a = &alog_state;
    if (a->live || file == INVALID_HANDLE_VALUE)
        return 0;
    a->slots = (AlogSlot *)VirtualAlloc(NULL, sizeof(AlogSlot) * ALOG_SLOTS, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
    if (!a->slots)
        return 0;
    for (unsigned int i = 0; i < ALOG_SLOTS; i++)
        a->slots[i].seq = i;
    a->head = a->tail = 0;
    a->stop = a->woken = 0;
    a->file = file;
    InitializeCriticalSectionAndSpinCount(&a->drain_lock, 4000);
    a->wake = CreateEventW(NULL, FALSE, FALSE, NULL);
    a->thread = a->wake ? CreateThread(NULL, 64 * 1024, alog_writer, a, 0, &a->thread_id) : NULL;
    if (!a->thread) {
        if (a->wake)
            CloseHandle(a->wake);
        DeleteCriticalSection(&a->drain_lock);
        VirtualFree(a->slots, 0, MEM_RELEASE);
        a->slots = NULL;
        return 0;
    }
    __atomic_store_n(&a->live, 1, __ATOMIC_RELEASE);
    return 1;
}

static int alog_try_push(Alog *a, const char *body, unsigned int n, const FILETIME *utc, DWORD tid)
{
    unsigned long long pos = __atomic_load_n(&a->head, __ATOMIC_RELAXED);
    for (;;) {
        AlogSlot *s = &a->slots[pos & (ALOG_SLOTS - 1)];
        unsigned long long seq = __atomic_load_n(&s->seq, __ATOMIC_ACQUIRE);
        long long dif = (long long)(seq - pos);
        if (dif == 0) {
            if (__atomic_compare_exchange_n(&a->head, &pos, pos + 1, 1, __ATOMIC_RELAXED, __ATOMIC_RELAXED)) {
                s->utc = *utc;
                s->tid = tid;
                s->len = n;
                memcpy(s->text, body, n);
                __atomic_store_n(&s->seq, pos + 1, __ATOMIC_RELEASE);
                unsigned long long backlog = pos + 1 - __atomic_load_n(&a->tail, __ATOMIC_ACQUIRE);
                if (backlog >= ALOG_SLOTS / 2 && !__atomic_exchange_n(&a->woken, 1, __ATOMIC_ACQ_REL))
                    SetEvent(a->wake);
                return 1;
            }
        } else if (dif < 0) {
            return 0;
        } else {
            pos = __atomic_load_n(&a->head, __ATOMIC_RELAXED);
        }
    }
}

static int alog_lock_bounded(Alog *a, DWORD wait_ms)
{
    DWORD start = GetTickCount();
    while (!TryEnterCriticalSection(&a->drain_lock)) {
        if (GetTickCount() - start >= wait_ms)
            return 0;
        Sleep(1);
    }
    return 1;
}

static int alog_push(const char *body, unsigned int n, const FILETIME *utc, DWORD tid)
{
    Alog *a = &alog_state;
    if (!__atomic_load_n(&a->live, __ATOMIC_ACQUIRE))
        return 0;
    if (n > ALOG_TEXT)
        n = ALOG_TEXT;
    for (int attempt = 0; attempt < 1000; attempt++) {
        if (alog_try_push(a, body, n, utc, tid))
            return 1;
        if (attempt < 8) {
            __atomic_fetch_add(&a->helped, 1, __ATOMIC_RELAXED);
            EnterCriticalSection(&a->drain_lock);
            alog_drain_locked(a);
            LeaveCriticalSection(&a->drain_lock);
        } else {
            Sleep(attempt < 100 ? 0 : 1);
        }
    }
    __atomic_fetch_add(&a->bypassed, 1, __ATOMIC_RELAXED);
    return 0;
}

static void alog_flush(void)
{
    Alog *a = &alog_state;
    if (!__atomic_load_n(&a->live, __ATOMIC_ACQUIRE) || GetCurrentThreadId() == a->thread_id)
        return;
    if (!alog_lock_bounded(a, ALOG_FLUSH_WAIT_MS))
        return;
    alog_drain_locked(a);
    LeaveCriticalSection(&a->drain_lock);
}

static void alog_flush_at_exit(void)
{
    Alog *a = &alog_state;
    if (!__atomic_load_n(&a->live, __ATOMIC_ACQUIRE))
        return;
    __atomic_store_n(&a->stop, 1, __ATOMIC_RELEASE);
    if (TryEnterCriticalSection(&a->drain_lock)) {
        alog_drain_locked(a);
        LeaveCriticalSection(&a->drain_lock);
    } else {
        alog_drain_locked(a);
    }
}

static inline void alog_stop(void)
{
    Alog *a = &alog_state;
    if (!__atomic_load_n(&a->live, __ATOMIC_ACQUIRE))
        return;
    __atomic_store_n(&a->stop, 1, __ATOMIC_RELEASE);
    SetEvent(a->wake);
    WaitForSingleObject(a->thread, 2000);
    EnterCriticalSection(&a->drain_lock);
    alog_drain_locked(a);
    LeaveCriticalSection(&a->drain_lock);
    CloseHandle(a->thread);
    CloseHandle(a->wake);
    DeleteCriticalSection(&a->drain_lock);
    VirtualFree(a->slots, 0, MEM_RELEASE);
    a->slots = NULL;
    __atomic_store_n(&a->live, 0, __ATOMIC_RELEASE);
}
