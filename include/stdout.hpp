#pragma once
#include <cstddef>
#include <cstdint>

#ifdef __APPLE__
#include <sys/syscall.h>
#endif

namespace lunos_fast_stdout {
    namespace detail {
        #if defined(__APPLE__)
            static constexpr int STD_OUT = 1;
            #ifdef __arm64__
                static constexpr long WRITE_SYSCALL = SYS_write;
            #else
                static constexpr long WRITE_SYSCALL = 0x2000004;
            #endif
        #elif defined(__linux__)
            static constexpr int STD_OUT = 1;
            static constexpr long WRITE_SYSCALL = 1;
        #elif defined(_WIN32)
            static constexpr int STD_OUT_HANDLE = -11;
        #endif

        inline size_t write_stdout(const char* buf, size_t len) {
            size_t ret = 0;

            #if defined(__APPLE__) && defined(__arm64__)
                // ARM64 macOS
                uint64_t x0 = STD_OUT;
                uint64_t x1 = (uint64_t)buf;
                uint64_t x2 = len;
                uint64_t x16 = WRITE_SYSCALL;

                asm volatile(
                    "mov x0, %1\n"
                    "mov x1, %2\n"
                    "mov x2, %3\n"
                    "mov x16, %4\n"
                    "svc #0x80\n"
                    "mov %0, x0"
                    : "=r"(ret)
                    : "r"(x0), "r"(x1), "r"(x2), "r"(x16)
                    : "x0", "x1", "x2", "x16", "memory", "cc"
                );
            #elif defined(__APPLE__) && defined(__x86_64__)
                // x86_64 Intel macOS
                asm volatile(
                    "movq %1, %%rdi\n"
                    "movq %2, %%rsi\n"
                    "movq %3, %%rdx\n"
                    "movq %4, %%rax\n"
                    "syscall\n"
                    "movq %%rax, %0\n"
                    : "=r"(ret)
                    : "g"((size_t)STD_OUT), "g"((size_t)buf), "g"(len), "g"((size_t)WRITE_SYSCALL)
                    : "rax", "rdi", "rsi", "rdx", "rcx", "r11", "memory"
                );
            #elif defined(__linux__) && defined(__x86_64__)
                // x86_64 Linux
                asm volatile(
                    "movq %1, %%rdi\n"
                    "movq %2, %%rsi\n"
                    "movq %3, %%rdx\n"
                    "movq %4, %%rax\n"
                    "syscall\n"
                    "movq %%rax, %0\n"
                    : "=r"(ret)
                    : "g"((size_t)STD_OUT), "g"((size_t)buf), "g"(len), "g"((size_t)WRITE_SYSCALL)
                    : "rax", "rdi", "rsi", "rdx", "rcx", "r11", "memory"
                );
            #elif defined(_WIN32)
                // Windows
                HANDLE handle = GetStdHandle(STD_OUTPUT_HANDLE);
                DWORD written;
                WriteFile(handle, buf, static_cast<DWORD>(len), &written, nullptr);
                ret = written;
            #else
                // Fallback
                #include <unistd.h>
                ret = write(STD_OUT, buf, len);
            #endif

            return ret;
        }
    }

    inline void write_line(const char* buf, size_t len) {
        detail::write_stdout(buf, len);
        const char newline = '\n';
        detail::write_stdout(&newline, 1);
    }
}
