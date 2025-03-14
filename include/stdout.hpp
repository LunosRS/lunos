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

        [[gnu::always_inline]] inline size_t write_stdout(const char* buf, size_t len) {
            size_t ret = 0;

            #if defined(__APPLE__) && defined(__arm64__)
                // ARM64 macOS - optimized to use fewer moves
                register uint64_t x0 asm("x0") = STD_OUT;
                register uint64_t x1 asm("x1") = (uint64_t)buf;
                register uint64_t x2 asm("x2") = len;
                register uint64_t x16 asm("x16") = WRITE_SYSCALL;

                asm volatile(
                    "svc #0x80"
                    : "+r"(x0)
                    : "r"(x1), "r"(x2), "r"(x16)
                    : "memory", "cc"
                );
                ret = x0;
            #elif defined(__APPLE__) && defined(__x86_64__)
                // x86_64 Intel macOS - optimized register allocation
                register size_t rax asm("rax") = WRITE_SYSCALL;
                register size_t rdi asm("rdi") = STD_OUT;
                register size_t rsi asm("rsi") = (size_t)buf;
                register size_t rdx asm("rdx") = len;

                asm volatile(
                    "syscall"
                    : "+r"(rax)
                    : "r"(rdi), "r"(rsi), "r"(rdx)
                    : "rcx", "r11", "memory"
                );
                ret = rax;
            #elif defined(__linux__) && defined(__x86_64__)
                // x86_64 Linux - optimized register allocation
                register size_t rax asm("rax") = WRITE_SYSCALL;
                register size_t rdi asm("rdi") = STD_OUT;
                register size_t rsi asm("rsi") = (size_t)buf;
                register size_t rdx asm("rdx") = len;

                asm volatile(
                    "syscall"
                    : "+r"(rax)
                    : "r"(rdi), "r"(rsi), "r"(rdx)
                    : "rcx", "r11", "memory"
                );
                ret = rax;
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

    [[gnu::always_inline]] inline void write_line(const char* buf, size_t len) {
        // Allocate space on stack for buffer + newline
        char stack_buf[256];
        if (len < 255) {
            // Fast path - use stack buffer
            __builtin_memcpy(stack_buf, buf, len);
            stack_buf[len] = '\n';
            detail::write_stdout(stack_buf, len + 1);
        } else {
            // Slow path - use two writes
            detail::write_stdout(buf, len);
            detail::write_stdout("\n", 1);
        }
    }
}
