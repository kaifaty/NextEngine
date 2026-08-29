#include "../r63zm/fixed_sha256.hpp"

#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <span>

namespace {

using Digest = r63zm::Digest;

constexpr std::size_t ReceiptBytes = 1368U;
constexpr std::size_t VersionAt = 8U;
constexpr std::size_t SizeAt = 12U;
constexpr std::size_t RouteAt = 20U;
constexpr std::size_t FlagsAt = 24U;
constexpr std::size_t CacheSizeAt = 32U;
constexpr std::size_t CacheRootAt = 40U;
constexpr std::size_t ParentSizeAt = 72U;
constexpr std::size_t ParentRootAt = 80U;
constexpr std::size_t AuditSizeAt = 112U;
constexpr std::size_t AuditRootAt = 120U;
constexpr std::size_t BundleRootAt = 152U;
constexpr std::size_t FactorRootAt = 184U;
constexpr std::size_t PermutationRootAt = 216U;
constexpr std::size_t InverseRootAt = 248U;
constexpr std::size_t RhsRootAt = 280U;
constexpr std::size_t BaselineRootAt = 312U;
constexpr std::size_t X0RootAt = 344U;
constexpr std::size_t Hx0RootAt = 376U;
constexpr std::size_t ResidualRootAt = 408U;
constexpr std::size_t ResidualBoundRootAt = 440U;
constexpr std::size_t Z0RootAt = 472U;
constexpr std::size_t RhoRootAt = 504U;
constexpr std::size_t WorkAt = 536U;
constexpr std::size_t WorkFields = 67U;
constexpr std::size_t EventCountAt = 1072U;
constexpr std::size_t EventsAt = 1080U;
constexpr std::size_t EventSlots = 7U;
constexpr std::size_t TraceAt = 1304U;
constexpr std::size_t ResultAt = 1336U;
static_assert(8U == VersionAt);
static_assert(VersionAt + 4U == SizeAt);
static_assert(SizeAt + 8U == RouteAt);
static_assert(RouteAt + 4U == FlagsAt);
static_assert(FlagsAt + 8U == CacheSizeAt);
static_assert(CacheSizeAt + 8U == CacheRootAt);
static_assert(CacheRootAt + 32U == ParentSizeAt);
static_assert(ParentSizeAt + 8U == ParentRootAt);
static_assert(ParentRootAt + 32U == AuditSizeAt);
static_assert(AuditSizeAt + 8U == AuditRootAt);
static_assert(AuditRootAt + 32U == BundleRootAt);
static_assert(BundleRootAt + 32U == FactorRootAt);
static_assert(FactorRootAt + 32U == PermutationRootAt);
static_assert(PermutationRootAt + 32U == InverseRootAt);
static_assert(InverseRootAt + 32U == RhsRootAt);
static_assert(RhsRootAt + 32U == BaselineRootAt);
static_assert(BaselineRootAt + 32U == X0RootAt);
static_assert(X0RootAt + 32U == Hx0RootAt);
static_assert(Hx0RootAt + 32U == ResidualRootAt);
static_assert(ResidualRootAt + 32U == ResidualBoundRootAt);
static_assert(ResidualBoundRootAt + 32U == Z0RootAt);
static_assert(Z0RootAt + 32U == RhoRootAt);
static_assert(RhoRootAt + 32U == WorkAt);
static_assert(WorkAt + WorkFields * 8U == EventCountAt);
static_assert(EventCountAt + 8U == EventsAt);
static_assert(EventsAt + EventSlots * 32U == TraceAt);
static_assert(TraceAt + 32U == ResultAt);
static_assert(ResultAt + 32U == ReceiptBytes);

std::uint32_t load_u32(const std::uint8_t* source) noexcept {
    return (static_cast<std::uint32_t>(source[0U]) << 24U)
        | (static_cast<std::uint32_t>(source[1U]) << 16U)
        | (static_cast<std::uint32_t>(source[2U]) << 8U)
        | static_cast<std::uint32_t>(source[3U]);
}

std::uint64_t load_u64(const std::uint8_t* source) noexcept {
    std::uint64_t value = 0U;
    for (std::size_t index = 0U; index < 8U; ++index)
        value = (value << 8U) | source[index];
    return value;
}

void store_u32(std::uint8_t* target, std::uint32_t value) noexcept {
    for (std::size_t index = 0U; index < 4U; ++index)
        target[index] = static_cast<std::uint8_t>(
            value >> (24U - 8U * index));
}

void store_u64(std::uint8_t* target, std::uint64_t value) noexcept {
    for (std::size_t index = 0U; index < 8U; ++index)
        target[index] = static_cast<std::uint8_t>(
            value >> (56U - 8U * index));
}

Digest load_digest(const std::array<std::uint8_t, ReceiptBytes>& receipt,
    std::size_t offset) noexcept {
    Digest value{};
    std::memcpy(value.data(), receipt.data() + offset, value.size());
    return value;
}

void store_digest(std::array<std::uint8_t, ReceiptBytes>& receipt,
    std::size_t offset, const Digest& value) noexcept {
    std::memcpy(receipt.data() + offset, value.data(), value.size());
}

void append_u64(r63zm::Sha256& hash, std::uint64_t value) noexcept {
    std::array<std::uint8_t, 8U> bytes{};
    store_u64(bytes.data(), value);
    hash.update(bytes);
}

void field_header(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t bytes) noexcept {
    hash.update_byte(tag);
    append_u64(hash, bytes);
}

void field_bytes(r63zm::Sha256& hash, std::uint8_t tag,
    const std::uint8_t* bytes, std::size_t count) noexcept {
    field_header(hash, tag, count);
    hash.update(std::span<const std::uint8_t>(bytes, count));
}

void field_text(r63zm::Sha256& hash, std::uint8_t tag,
    const char* text) noexcept {
    field_bytes(hash, tag, reinterpret_cast<const std::uint8_t*>(text),
        std::strlen(text));
}

void field_u64(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t value) noexcept {
    field_header(hash, tag, 8U);
    append_u64(hash, value);
}

void field_digest(r63zm::Sha256& hash, std::uint8_t tag,
    const Digest& value) noexcept {
    field_bytes(hash, tag, value.data(), value.size());
}

Digest work_root(
    const std::array<std::uint8_t, ReceiptBytes>& receipt) noexcept {
    r63zm::Sha256 hash;
    field_text(hash, 1U, "nextengine.nonlocal.r63zn.candidate-work.v1");
    field_u64(hash, 2U, WorkFields);
    field_header(hash, 3U, WorkFields * 8U);
    hash.update(std::span<const std::uint8_t>(
        receipt.data() + WorkAt, WorkFields * 8U));
    return hash.finish();
}

Digest event_root(std::uint64_t ordinal, const Digest& left,
    const Digest& right) noexcept {
    r63zm::Sha256 hash;
    field_text(hash, 1U, "nextengine.nonlocal.r63zn.event.v1");
    field_u64(hash, 2U, ordinal);
    field_digest(hash, 3U, left);
    field_digest(hash, 4U, right);
    return hash.finish();
}

Digest trace_root(std::uint32_t route,
    const std::array<std::uint8_t, ReceiptBytes>& receipt) noexcept {
    const std::uint64_t count = load_u64(receipt.data() + EventCountAt);
    r63zm::Sha256 hash;
    field_text(hash, 1U, "nextengine.nonlocal.r63zn.trace.v1");
    field_u64(hash, 2U, route);
    field_u64(hash, 3U, count);
    field_header(hash, 4U, count * 32U);
    hash.update(std::span<const std::uint8_t>(receipt.data() + EventsAt,
        static_cast<std::size_t>(count) * 32U));
    return hash.finish();
}

Digest result_root(std::uint32_t route,
    const std::array<std::uint8_t, ReceiptBytes>& receipt,
    const Digest& trace, const Digest& work) noexcept {
    const std::array<Digest, 3U> inputs{{load_digest(receipt, CacheRootAt),
        load_digest(receipt, ParentRootAt), load_digest(receipt, AuditRootAt)}};
    r63zm::Sha256 hash;
    field_text(hash, 1U, "nextengine.nonlocal.r63zn.result.v1");
    field_u64(hash, 2U, 3U);
    field_u64(hash, 3U, route);
    field_header(hash, 4U, inputs.size() * 32U);
    for (const Digest& input : inputs) hash.update(input);
    field_digest(hash, 5U, trace);
    field_digest(hash, 6U, work);
    field_u64(hash, 7U, ReceiptBytes);
    return hash.finish();
}

void reseal(std::array<std::uint8_t, ReceiptBytes>& receipt) noexcept {
    const std::uint32_t route = load_u32(receipt.data() + RouteAt);
    const Digest work = work_root(receipt);
    const Digest trace = trace_root(route, receipt);
    store_digest(receipt, TraceAt, trace);
    store_digest(receipt, ResultAt, result_root(route, receipt, trace, work));
}

bool read_receipt(const char* path,
    std::array<std::uint8_t, ReceiptBytes>& receipt) noexcept {
    std::FILE* file = std::fopen(path, "rb");
    if (file == nullptr) return false;
    const std::size_t count = std::fread(receipt.data(), 1U, receipt.size(),
        file);
    const int trailing = std::fgetc(file);
    const int close = std::fclose(file);
    return count == receipt.size() && trailing == EOF && close == 0;
}

bool write_receipt(const char* path,
    const std::array<std::uint8_t, ReceiptBytes>& receipt) noexcept {
    std::FILE* file = std::fopen(path, "wb");
    if (file == nullptr) return false;
    const std::size_t count = std::fwrite(receipt.data(), 1U,
        receipt.size(), file);
    const int close = std::fclose(file);
    return count == receipt.size() && close == 0;
}

bool parse_index(const char* text, std::size_t maximum,
    std::size_t& result) noexcept {
    if (text == nullptr || *text == '\0') return false;
    std::size_t value = 0U;
    for (const char* cursor = text; *cursor != '\0'; ++cursor) {
        if (*cursor < '0' || *cursor > '9') return false;
        value = value * 10U + static_cast<std::size_t>(*cursor - '0');
        if (value >= maximum) return false;
    }
    result = value;
    return true;
}

bool mutate_semantic_root(std::array<std::uint8_t, ReceiptBytes>& receipt,
    std::size_t root_index) noexcept {
    constexpr std::array<std::size_t, 6U> offsets{{X0RootAt, Hx0RootAt,
        ResidualRootAt, ResidualBoundRootAt, Z0RootAt, RhoRootAt}};
    if (root_index >= offsets.size()) return false;
    receipt[offsets[root_index]] ^= 0x01U;
    const Digest work = work_root(receipt);
    switch (root_index) {
    case 0U:
        store_digest(receipt, EventsAt + 2U * 32U, event_root(2U,
            load_digest(receipt, X0RootAt),
            load_digest(receipt, BaselineRootAt)));
        break;
    case 1U:
        store_digest(receipt, EventsAt + 3U * 32U, event_root(3U,
            load_digest(receipt, Hx0RootAt),
            load_digest(receipt, ParentRootAt)));
        break;
    case 2U:
    case 3U:
        store_digest(receipt, EventsAt + 4U * 32U, event_root(4U,
            load_digest(receipt, ResidualRootAt),
            load_digest(receipt, ResidualBoundRootAt)));
        break;
    case 4U:
        store_digest(receipt, EventsAt + 5U * 32U, event_root(5U,
            load_digest(receipt, Z0RootAt),
            load_digest(receipt, 184U)));
        break;
    case 5U:
        store_digest(receipt, EventsAt + 6U * 32U, event_root(6U,
            load_digest(receipt, RhoRootAt), work));
        break;
    default:
        return false;
    }
    reseal(receipt);
    return true;
}

bool apply(std::array<std::uint8_t, ReceiptBytes>& receipt,
    int argc, char** argv) noexcept {
    if (std::memcmp(receipt.data(), "NER63ZN1", 8U) != 0
        || load_u32(receipt.data() + 8U) != 3U
        || load_u64(receipt.data() + 12U) != ReceiptBytes
        || load_u32(receipt.data() + RouteAt) != 7U
        || load_u64(receipt.data() + EventCountAt) != EventSlots)
        return false;
    const char* command = argv[3U];
    if (std::strcmp(command, "malformed") == 0 && argc == 4) {
        receipt[0U] ^= 0x01U;
        return true;
    }
    if (std::strcmp(command, "seal") == 0 && argc == 4) {
        receipt[ResultAt] ^= 0x01U;
        return true;
    }
    if (std::strcmp(command, "route") == 0 && argc == 4) {
        store_u32(receipt.data() + RouteAt, 6U);
        reseal(receipt);
        return true;
    }
    if (std::strcmp(command, "event-delete") == 0 && argc == 4) {
        store_u64(receipt.data() + EventCountAt, EventSlots - 1U);
        std::memset(receipt.data() + EventsAt + 6U * 32U, 0, 32U);
        reseal(receipt);
        return true;
    }
    if (std::strcmp(command, "event-duplicate") == 0 && argc == 4) {
        std::memcpy(receipt.data() + EventsAt + 3U * 32U,
            receipt.data() + EventsAt + 2U * 32U, 32U);
        reseal(receipt);
        return true;
    }
    if (std::strcmp(command, "event-reorder") == 0 && argc == 4) {
        std::array<std::uint8_t, 32U> temporary{};
        std::memcpy(temporary.data(), receipt.data() + EventsAt, 32U);
        std::memcpy(receipt.data() + EventsAt,
            receipt.data() + EventsAt + 32U, 32U);
        std::memcpy(receipt.data() + EventsAt + 32U, temporary.data(), 32U);
        reseal(receipt);
        return true;
    }
    std::size_t index = 0U;
    if (std::strcmp(command, "work") == 0 && argc == 5
        && parse_index(argv[4U], WorkFields, index)) {
        const std::size_t offset = WorkAt + index * 8U;
        store_u64(receipt.data() + offset,
            load_u64(receipt.data() + offset) + 1U);
        const Digest work = work_root(receipt);
        store_digest(receipt, EventsAt + 6U * 32U, event_root(6U,
            load_digest(receipt, RhoRootAt), work));
        reseal(receipt);
        return true;
    }
    if (std::strcmp(command, "root") == 0 && argc == 5
        && parse_index(argv[4U], 6U, index))
        return mutate_semantic_root(receipt, index);
    return false;
}

} // namespace

int main(int argc, char** argv) {
    if (argc != 4 && argc != 5) return 64;
    std::array<std::uint8_t, ReceiptBytes> receipt{};
    if (!read_receipt(argv[1U], receipt)) return 65;
    if (!apply(receipt, argc, argv)) return 66;
    return write_receipt(argv[2U], receipt) ? 0 : 67;
}
