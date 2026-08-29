#include <atomic>
#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <new>
#include <unistd.h>

namespace {

std::atomic<unsigned long long> calls{0U};
std::atomic<unsigned long long> bytes{0U};

void record(std::size_t size) noexcept {
    calls.fetch_add(1U, std::memory_order_relaxed);
    bytes.fetch_add(static_cast<unsigned long long>(size),
        std::memory_order_relaxed);
}

void* allocate(std::size_t size) {
    if (size == 0U) size = 1U;
    record(size);
    if (void* value = std::malloc(size)) return value;
    throw std::bad_alloc();
}

void* allocate_aligned(std::size_t size, std::size_t alignment) {
    if (size == 0U) size = 1U;
    record(size);
    void* value = nullptr;
    if (posix_memalign(&value, alignment, size) == 0) return value;
    throw std::bad_alloc();
}

struct Reporter final {
    ~Reporter() {
        char buffer[160]{};
        const int count = std::snprintf(buffer, sizeof(buffer),
            "allocation_probe_calls=%llu allocation_probe_bytes=%llu\n",
            calls.load(std::memory_order_relaxed),
            bytes.load(std::memory_order_relaxed));
        if (count > 0) {
            const ssize_t written = write(STDERR_FILENO, buffer,
                static_cast<std::size_t>(count));
            static_cast<void>(written);
        }
    }
};

Reporter reporter;

} // namespace

void* operator new(std::size_t size) { return allocate(size); }
void* operator new[](std::size_t size) { return allocate(size); }
void* operator new(std::size_t size, const std::nothrow_t&) noexcept {
    try { return allocate(size); } catch (...) { return nullptr; }
}
void* operator new[](std::size_t size, const std::nothrow_t&) noexcept {
    try { return allocate(size); } catch (...) { return nullptr; }
}
void* operator new(std::size_t size, std::align_val_t alignment) {
    return allocate_aligned(size, static_cast<std::size_t>(alignment));
}
void* operator new[](std::size_t size, std::align_val_t alignment) {
    return allocate_aligned(size, static_cast<std::size_t>(alignment));
}
void* operator new(std::size_t size, std::align_val_t alignment,
    const std::nothrow_t&) noexcept {
    try {
        return allocate_aligned(size, static_cast<std::size_t>(alignment));
    } catch (...) { return nullptr; }
}
void* operator new[](std::size_t size, std::align_val_t alignment,
    const std::nothrow_t&) noexcept {
    try {
        return allocate_aligned(size, static_cast<std::size_t>(alignment));
    } catch (...) { return nullptr; }
}

void operator delete(void* value) noexcept { std::free(value); }
void operator delete[](void* value) noexcept { std::free(value); }
void operator delete(void* value, std::size_t) noexcept { std::free(value); }
void operator delete[](void* value, std::size_t) noexcept { std::free(value); }
void operator delete(void* value, std::align_val_t) noexcept { std::free(value); }
void operator delete[](void* value, std::align_val_t) noexcept { std::free(value); }
void operator delete(void* value, std::size_t, std::align_val_t) noexcept {
    std::free(value);
}
void operator delete[](void* value, std::size_t, std::align_val_t) noexcept {
    std::free(value);
}
