#include "../../include/stdout.hpp"

extern "C" {
    size_t lunos_fast_stdout_writer(const char* buf, size_t len) {
        return lunos_fast_stdout::detail::write_stdout(buf, len);
    }
}
