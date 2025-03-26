#pragma once
#include <cstddef>
#include <cstdint>

#if defined(_WIN32)
    #include <windows.h>
#else
    #include <unistd.h>
    #if defined(__APPLE__)
        #include <sys/syscall.h>
    #endif
#endif

#if !defined(__GNUC__) && !defined(__clang__)
    #include <cstring>
#endif


namespace lunos_fast_stdout {
    namespace detail {
        #if defined(__APPLE__)
            static constexpr int STD_OUT = 1;
            #ifdef __aarch64__
                static constexpr long WRITE_SYSCALL = SYS_write;
            #elif defined(__x86_64__)
                static constexpr long WRITE_SYSCALL = 0x2000004;
            #else
                #error "Unsupported Apple architecture"
            #endif
        #elif defined(__linux__)
            static constexpr int STD_OUT = 1;
            #if defined(__x86_64__)
                static constexpr long WRITE_SYSCALL = 1;
            #elif defined(__aarch64__)
                static constexpr long WRITE_SYSCALL = 64;
            #elif defined(__arm__)
                static constexpr long WRITE_SYSCALL = 4;
            #else
                #error "Unsupported Linux architecture"
            #endif
        #elif defined(_WIN32)

        #else
             #ifdef STDOUT_FILENO
                static constexpr int STD_OUT = STDOUT_FILENO;
             #else
                static constexpr int STD_OUT = 1;
             #endif
        #endif

        [[gnu::always_inline]] inline size_t write_stdout(const char* buf, size_t len) {
            size_t ret = 0;

            #if defined(__APPLE__) && defined(__aarch64__)
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
             #elif defined(__linux__) && defined(__aarch64__)
                register uint64_t x0 asm("x0") = STD_OUT;
                register uint64_t x1 asm("x1") = (uint64_t)buf;
                register uint64_t x2 asm("x2") = len;
                register uint64_t x8 asm("x8") = WRITE_SYSCALL;

                asm volatile(
                    "svc #0"
                    : "+r"(x0)
                    : "r"(x1), "r"(x2), "r"(x8)
                    : "memory", "cc"
                );
                ret = x0;
            #elif defined(__linux__) && defined(__arm__)
                register size_t r0 asm("r0") = STD_OUT;
                register size_t r1 asm("r1") = (size_t)buf;
                register size_t r2 asm("r2") = len;
                register size_t r7 asm("r7") = WRITE_SYSCALL;

                asm volatile(
                    "svc #0"
                    : "+r"(r0)
                    : "r"(r1), "r"(r2), "r"(r7)
                    : "memory", "cc"
                );
                ret = r0;
            #elif defined(_WIN32)
                HANDLE handle = GetStdHandle(STD_OUTPUT_HANDLE);
                DWORD written = 0;
                if (handle != INVALID_HANDLE_VALUE) {
                   DWORD len_dword = (len > MAXDWORD) ? MAXDWORD : static_cast<DWORD>(len);
                   if (!WriteFile(handle, buf, len_dword, &written, nullptr)) {
                       written = 0;
                   }
                } else {
                     written = 0;
                }
                ret = written;
            #else
                ssize_t write_ret = write(STD_OUT, buf, len);
                if (write_ret < 0) {
                    ret = 0;
                } else {
                    ret = static_cast<size_t>(write_ret);
                }
                return ret;
            #endif

            #if (defined(__linux__) || defined(__APPLE__)) && !defined(_WIN32)
                if (static_cast<ssize_t>(ret) < 0) {
                   ret = 0;
                }
            #endif

            return ret;
        }
    }

    [[gnu::always_inline]] inline void write_line(const char* buf, size_t len) {
        constexpr size_t MAX_STACK_BUF_SIZE = 256;
        char stack_buf[MAX_STACK_BUF_SIZE];

        if ((len + 1) <= MAX_STACK_BUF_SIZE) {
            #if defined(__GNUC__) || defined(__clang__)
                __builtin_memcpy(stack_buf, buf, len);
            #else
                memcpy(stack_buf, buf, len);
            #endif
            stack_buf[len] = '\n';
            detail::write_stdout(stack_buf, len + 1);
        } else {
            detail::write_stdout(buf, len);
            detail::write_stdout("\n", 1);
        }
    }
}
