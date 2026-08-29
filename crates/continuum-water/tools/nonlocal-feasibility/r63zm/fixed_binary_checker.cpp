#include "fixed_sha256.hpp"

#include <array>
#include <bit>
#include <cfenv>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fcntl.h>
#include <quadmath.h>
#include <span>
#include <sys/stat.h>
#include <unistd.h>

namespace {

using Quad = __float128;
using r63zm::Digest;

constexpr Quad QZERO = static_cast<Quad>(0.0);
constexpr Quad QONE = static_cast<Quad>(1.0);
constexpr std::size_t CACHE_SIZE = 1033625U;
constexpr std::size_t ARTIFACT_SIZE = 12916U;
constexpr std::size_t ROW_COUNT = 102U;
constexpr std::size_t COLUMN_COUNT = 315U;
constexpr std::size_t ROLE_COUNT = 6U;
constexpr std::size_t PAYLOAD_COUNT = 7U;
constexpr std::size_t EVENT_COUNT = 9U;
constexpr std::size_t PRODUCT_WORK_COUNT = 17U;
constexpr std::size_t READ_WORK_COUNT = 26U;
constexpr std::size_t ADMISSION_WORK_COUNT = 18U;
constexpr std::size_t SEAL_WORK_COUNT = 16U;
constexpr std::size_t CHECKER_WORK_COUNT = 26U;
constexpr std::size_t AUDIT_OUTPUT_SIZE = 420U;
constexpr std::size_t MAGIC_OFFSET = 0U;
constexpr std::size_t VERSION_OFFSET = 8U;
constexpr std::size_t TOTAL_BYTES_OFFSET = 12U;
constexpr std::size_t SOURCE_BYTES_OFFSET = 20U;
constexpr std::size_t SOURCE_ROOT_OFFSET = 28U;
constexpr std::size_t ROUTE_OFFSET = 60U;
constexpr std::size_t SELECTED_ROOT_OFFSET = 64U;
constexpr std::size_t BUNDLE_ROOT_OFFSET = 96U;
constexpr std::size_t PRODUCT_SET_ROOT_OFFSET = 128U;
constexpr std::size_t EVENT_COUNT_OFFSET = 160U;
constexpr std::size_t EVENTS_OFFSET = 164U;
constexpr std::size_t TRACE_OFFSET = 452U;
constexpr std::size_t RESULT_OFFSET = 484U;
constexpr std::size_t READ_WORK_OFFSET = 516U;
constexpr std::size_t ADMISSION_WORK_OFFSET = 724U;
constexpr std::size_t SEAL_WORK_OFFSET = 876U;
constexpr std::size_t PAYLOAD_ROOTS_OFFSET = 1004U;
constexpr std::size_t PRODUCT_COUNT_OFFSET = 1228U;
constexpr std::size_t PRODUCTS_OFFSET = 1236U;
constexpr std::size_t PRODUCT_RECORD_SIZE = 1944U;
constexpr std::size_t FOOTER_OFFSET = 12900U;
constexpr std::size_t PRODUCT_FLAGS_OFFSET = 0U;
constexpr std::size_t PRODUCT_WORK_OFFSET = 8U;
constexpr std::size_t PRODUCT_VECTOR_ROOTS_OFFSET = 144U;
constexpr std::size_t PRODUCT_ROOT_OFFSET = 272U;
constexpr std::size_t PRODUCT_VALUE_COUNT_OFFSET = 304U;
constexpr std::size_t PRODUCT_VALUES_OFFSET = 312U;
constexpr std::size_t AUDIT_MAGIC_OFFSET = 0U;
constexpr std::size_t AUDIT_VERSION_OFFSET = 8U;
constexpr std::size_t AUDIT_TOTAL_BYTES_OFFSET = 12U;
constexpr std::size_t AUDIT_ROUTE_OFFSET = 20U;
constexpr std::size_t AUDIT_FLAGS_OFFSET = 24U;
constexpr std::size_t AUDIT_COMPONENTS_OFFSET = 28U;
constexpr std::size_t AUDIT_WORK_OFFSET = 36U;
constexpr std::size_t AUDIT_CACHE_BYTES_OFFSET =
    AUDIT_WORK_OFFSET + CHECKER_WORK_COUNT * 8U;
constexpr std::size_t AUDIT_CACHE_ROOT_OFFSET =
    AUDIT_CACHE_BYTES_OFFSET + 8U;
constexpr std::size_t AUDIT_ARTIFACT_BYTES_OFFSET =
    AUDIT_CACHE_ROOT_OFFSET + 32U;
constexpr std::size_t AUDIT_ARTIFACT_ROOT_OFFSET =
    AUDIT_ARTIFACT_BYTES_OFFSET + 8U;
constexpr std::size_t AUDIT_PRODUCER_RESULT_OFFSET =
    AUDIT_ARTIFACT_ROOT_OFFSET + 32U;
constexpr std::size_t AUDIT_RECONSTRUCTED_RESULT_OFFSET =
    AUDIT_PRODUCER_RESULT_OFFSET + 32U;
constexpr std::size_t AUDIT_CHECKER_ROOT_OFFSET =
    AUDIT_RECONSTRUCTED_RESULT_OFFSET + 32U;

constexpr std::array<std::uint8_t, 8U> CACHE_MAGIC{{
    'N','E','F','P','R','B','1',0U}};
constexpr std::array<std::uint8_t, 8U> ARTIFACT_MAGIC{{
    'N','E','R','6','3','Z','M','1'}};
constexpr std::array<std::uint8_t, 8U> AUDIT_MAGIC{{
    'N','E','R','6','3','Q','C','1'}};

constexpr char VECTOR_DOMAIN[] =
    "nextengine.nonlocal.r63zm.binary128-vector.v1";
constexpr char SELECTED_DOMAIN[] =
    "nextengine.nonlocal.r63zm.selected-binary.v1";
constexpr char PRODUCT_DOMAIN[] =
    "nextengine.nonlocal.r63zm.product-binary.v1";
constexpr char PRODUCT_SET_DOMAIN[] =
    "nextengine.nonlocal.r63zm.product-set-binary.v1";
constexpr char BUNDLE_DOMAIN[] =
    "nextengine.nonlocal.r63zm.selected-bundle-binary.v1";
constexpr char READ_WORK_DOMAIN[] =
    "nextengine.nonlocal.r63zm.read-work-binary.v2";
constexpr char ADMISSION_WORK_DOMAIN[] =
    "nextengine.nonlocal.r63zm.admission-work-binary.v2";
constexpr char EVENT_DOMAIN[] =
    "nextengine.nonlocal.r63zm.event-binary.v2";
constexpr char TRACE_DOMAIN[] =
    "nextengine.nonlocal.r63zm.trace-binary.v2";
constexpr char BODY_DOMAIN[] =
    "nextengine.nonlocal.r63zm.artifact-body-binary.v1";
constexpr char RESULT_DOMAIN[] =
    "nextengine.nonlocal.r63zm.result-binary.v2";
constexpr char CHECKER_DOMAIN[] =
    "nextengine.nonlocal.r63zm.checker-audit-binary.v3";

static_assert(sizeof(Quad) == 16U);
static_assert(std::endian::native == std::endian::little);
static_assert(__FLT128_MANT_DIG__ == 113);
static_assert(__FLT128_MAX_EXP__ == 16384);
static_assert(MAGIC_OFFSET == 0U);
static_assert(MAGIC_OFFSET + 8U == VERSION_OFFSET);
static_assert(VERSION_OFFSET + 4U == TOTAL_BYTES_OFFSET);
static_assert(TOTAL_BYTES_OFFSET + 8U == SOURCE_BYTES_OFFSET);
static_assert(SOURCE_BYTES_OFFSET + 8U == SOURCE_ROOT_OFFSET);
static_assert(SOURCE_ROOT_OFFSET + 32U == ROUTE_OFFSET);
static_assert(ROUTE_OFFSET + 4U == SELECTED_ROOT_OFFSET);
static_assert(SELECTED_ROOT_OFFSET + 32U == BUNDLE_ROOT_OFFSET);
static_assert(BUNDLE_ROOT_OFFSET + 32U == PRODUCT_SET_ROOT_OFFSET);
static_assert(PRODUCT_SET_ROOT_OFFSET + 32U == EVENT_COUNT_OFFSET);
static_assert(EVENT_COUNT_OFFSET + 4U == EVENTS_OFFSET);
static_assert(EVENTS_OFFSET + EVENT_COUNT * 32U == TRACE_OFFSET);
static_assert(TRACE_OFFSET + 32U == RESULT_OFFSET);
static_assert(RESULT_OFFSET + 32U == READ_WORK_OFFSET);
static_assert(READ_WORK_OFFSET + READ_WORK_COUNT * 8U
    == ADMISSION_WORK_OFFSET);
static_assert(ADMISSION_WORK_OFFSET + 8U + ADMISSION_WORK_COUNT * 8U
    == SEAL_WORK_OFFSET);
static_assert(SEAL_WORK_OFFSET + SEAL_WORK_COUNT * 8U
    == PAYLOAD_ROOTS_OFFSET);
static_assert(PAYLOAD_ROOTS_OFFSET + PAYLOAD_COUNT * 32U
    == PRODUCT_COUNT_OFFSET);
static_assert(PRODUCT_COUNT_OFFSET + 8U == PRODUCTS_OFFSET);
static_assert(PRODUCTS_OFFSET + ROLE_COUNT * PRODUCT_RECORD_SIZE
    == FOOTER_OFFSET);
static_assert(FOOTER_OFFSET + 16U == ARTIFACT_SIZE);
static_assert(PRODUCT_FLAGS_OFFSET + 8U == PRODUCT_WORK_OFFSET);
static_assert(PRODUCT_WORK_OFFSET + PRODUCT_WORK_COUNT * 8U
    == PRODUCT_VECTOR_ROOTS_OFFSET);
static_assert(PRODUCT_VECTOR_ROOTS_OFFSET + 4U * 32U
    == PRODUCT_ROOT_OFFSET);
static_assert(PRODUCT_ROOT_OFFSET + 32U == PRODUCT_VALUE_COUNT_OFFSET);
static_assert(PRODUCT_VALUE_COUNT_OFFSET + 8U == PRODUCT_VALUES_OFFSET);
static_assert(PRODUCT_VALUES_OFFSET + ROW_COUNT * sizeof(Quad)
    == PRODUCT_RECORD_SIZE);
static_assert(AUDIT_MAGIC_OFFSET == 0U);
static_assert(AUDIT_MAGIC_OFFSET + 8U == AUDIT_VERSION_OFFSET);
static_assert(AUDIT_VERSION_OFFSET + 4U == AUDIT_TOTAL_BYTES_OFFSET);
static_assert(AUDIT_TOTAL_BYTES_OFFSET + 8U == AUDIT_ROUTE_OFFSET);
static_assert(AUDIT_ROUTE_OFFSET + 4U == AUDIT_FLAGS_OFFSET);
static_assert(AUDIT_FLAGS_OFFSET + 4U == AUDIT_COMPONENTS_OFFSET);
static_assert(AUDIT_COMPONENTS_OFFSET + 8U == AUDIT_WORK_OFFSET);
static_assert(AUDIT_WORK_OFFSET + CHECKER_WORK_COUNT * 8U
    == AUDIT_CACHE_BYTES_OFFSET);
static_assert(AUDIT_CACHE_BYTES_OFFSET + 8U == AUDIT_CACHE_ROOT_OFFSET);
static_assert(AUDIT_CACHE_ROOT_OFFSET + 32U
    == AUDIT_ARTIFACT_BYTES_OFFSET);
static_assert(AUDIT_ARTIFACT_BYTES_OFFSET + 8U
    == AUDIT_ARTIFACT_ROOT_OFFSET);
static_assert(AUDIT_ARTIFACT_ROOT_OFFSET + 32U
    == AUDIT_PRODUCER_RESULT_OFFSET);
static_assert(AUDIT_PRODUCER_RESULT_OFFSET + 32U
    == AUDIT_RECONSTRUCTED_RESULT_OFFSET);
static_assert(AUDIT_RECONSTRUCTED_RESULT_OFFSET + 32U
    == AUDIT_CHECKER_ROOT_OFFSET);
static_assert(AUDIT_CHECKER_ROOT_OFFSET + 32U == AUDIT_OUTPUT_SIZE);

std::array<std::uint8_t, CACHE_SIZE> cache_bytes{};
std::array<std::uint8_t, ARTIFACT_SIZE> artifact_bytes{};

struct Slice final {
    const std::uint8_t* data = nullptr;
    std::size_t size = 0U;
};

struct RawVector final {
    const std::uint8_t* data = nullptr;
    std::uint64_t count = 0U;
    bool canonical_big_endian = false;
};

struct FileWork final {
    std::uint64_t open_calls = 0U;
    std::uint64_t stat_calls = 0U;
    std::uint64_t read_calls = 0U;
    std::uint64_t bytes = 0U;
    std::uint64_t close_calls = 0U;
    std::uint64_t source_size = 0U;
    std::uint64_t hash_calls = 0U;
    std::uint64_t hash_bytes = 0U;
    Digest root{};
};

struct CheckerWork final {
    std::uint64_t cache_read_calls = 0U;
    std::uint64_t artifact_read_calls = 0U;
    std::uint64_t file_metadata_calls = 0U;
    std::uint64_t cache_bytes_read = 0U;
    std::uint64_t artifact_bytes_read = 0U;
    std::uint64_t cache_parse_checks = 0U;
    std::uint64_t artifact_structure_checks = 0U;
    std::uint64_t semantic_comparisons = 0U;
    std::uint64_t component_comparisons = 0U;
    std::uint64_t work_comparisons = 0U;
    std::uint64_t root_comparisons = 0U;
    std::uint64_t arithmetic_kernels = 0U;
    std::uint64_t inner_terms = 0U;
    std::uint64_t outer_terms = 0U;
    std::uint64_t propagation_terms = 0U;
    std::uint64_t hash_calls = 0U;
    std::uint64_t hash_bytes = 0U;
    std::uint64_t route_checks = 0U;
    std::uint64_t package_allocations = 0U;
    std::uint64_t audit_root_bytes = 0U;
    std::uint64_t model_take_calls = 0U;
    std::uint64_t model_take_bytes = 0U;
    std::uint64_t model_typed_operations = 0U;
    std::uint64_t model_reconstruction_operations = 0U;
    std::uint64_t unused_slot_checks = 0U;
    std::uint64_t unused_slot_bytes = 0U;

    std::array<std::uint64_t, CHECKER_WORK_COUNT> fields() const noexcept {
        return {{cache_read_calls, artifact_read_calls, file_metadata_calls,
            cache_bytes_read, artifact_bytes_read, cache_parse_checks,
            artifact_structure_checks, semantic_comparisons,
            component_comparisons, work_comparisons, root_comparisons,
            arithmetic_kernels, inner_terms, outer_terms, propagation_terms,
            hash_calls, hash_bytes, route_checks, package_allocations,
            audit_root_bytes, model_take_calls, model_take_bytes,
            model_typed_operations, model_reconstruction_operations,
            unused_slot_checks, unused_slot_bytes}};
    }
};

void record_hash(CheckerWork& work, std::uint64_t bytes) noexcept {
    ++work.hash_calls;
    work.hash_bytes += bytes;
}

void be64(r63zm::Sha256& hash, std::uint64_t value) noexcept {
    std::array<std::uint8_t, 8U> encoded{};
    for (std::size_t index = 0U; index < encoded.size(); ++index)
        encoded[index] = static_cast<std::uint8_t>(
            value >> (56U - 8U * index));
    hash.update(encoded);
}

void tlv_header(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t length) noexcept {
    hash.update_byte(tag);
    be64(hash, length);
}

void tlv_digest(r63zm::Sha256& hash, std::uint8_t tag,
    const Digest& digest) noexcept {
    tlv_header(hash, tag, digest.size());
    hash.update(digest);
}

void tlv_u64(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t value) noexcept {
    tlv_header(hash, tag, 8U);
    be64(hash, value);
}

void tlv_byte(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint8_t value) noexcept {
    tlv_header(hash, tag, 1U);
    hash.update_byte(value);
}

template <std::size_t Count>
void tlv_array(r63zm::Sha256& hash, std::uint8_t tag,
    const std::array<std::uint64_t, Count>& values) noexcept {
    tlv_header(hash, tag, values.size() * 8U);
    for (std::uint64_t value : values) be64(hash, value);
}

template <std::size_t Count>
void tlv_constant(r63zm::Sha256& hash, std::uint8_t tag,
    const char (&text)[Count]) noexcept {
    tlv_header(hash, tag, Count - 1U);
    hash.update(std::span<const std::uint8_t>(
        reinterpret_cast<const std::uint8_t*>(text), Count - 1U));
}

bool digest_same(const Digest& left, const Digest& right) noexcept {
    std::uint8_t difference = 0U;
    for (std::size_t index = 0U; index < left.size(); ++index)
        difference |= static_cast<std::uint8_t>(left[index] ^ right[index]);
    return difference == 0U;
}

std::array<char, 65U> digest_hex(const Digest& digest) noexcept {
    constexpr char alphabet[] = "0123456789abcdef";
    std::array<char, 65U> text{};
    for (std::size_t index = 0U; index < digest.size(); ++index) {
        text[2U * index] = alphabet[digest[index] >> 4U];
        text[2U * index + 1U] = alphabet[digest[index] & 0x0fU];
    }
    return text;
}

bool digest_is(const Digest& digest, const char* expected) noexcept {
    const auto actual = digest_hex(digest);
    for (std::size_t index = 0U; index < 64U; ++index)
        if (actual[index] != expected[index]) return false;
    return expected[64U] == '\0';
}

template <std::size_t Size>
bool fixed_read(const char* path, std::array<std::uint8_t, Size>& destination,
    FileWork& work) noexcept {
    constexpr std::size_t CHUNK = 4096U;
    ++work.open_calls;
    const int descriptor = open(path, O_RDONLY);
    if (descriptor < 0) {
        r63zm::Sha256 empty;
        ++work.hash_calls;
        work.root = empty.finish();
        return false;
    }
    struct stat metadata {};
    ++work.stat_calls;
    const bool stated = fstat(descriptor, &metadata) == 0
        && metadata.st_size >= 0;
    if (stated) work.source_size = static_cast<std::uint64_t>(metadata.st_size);
    r63zm::Sha256 hash;
    ++work.hash_calls;
    bool read_ok = stated;
    std::array<std::uint8_t, CHUNK> trailing{};
    std::uint64_t offset = 0U;
    while (read_ok && offset < work.source_size) {
        const std::size_t remaining = static_cast<std::size_t>(
            work.source_size - offset);
        std::size_t request = remaining < CHUNK ? remaining : CHUNK;
        std::uint8_t* output = offset < Size
            ? destination.data() + static_cast<std::size_t>(offset)
            : trailing.data();
        if (offset < Size
            && request > Size - static_cast<std::size_t>(offset))
            request = Size - static_cast<std::size_t>(offset);
        ++work.read_calls;
        const ssize_t count = pread(descriptor, output, request,
            static_cast<off_t>(offset));
        if (count <= 0) {
            read_ok = false;
            break;
        }
        hash.update(std::span<const std::uint8_t>(
            output, static_cast<std::size_t>(count)));
        work.bytes += static_cast<std::uint64_t>(count);
        work.hash_bytes += static_cast<std::uint64_t>(count);
        offset += static_cast<std::uint64_t>(count);
        if (static_cast<std::size_t>(count) != request) read_ok = false;
    }
    ++work.close_calls;
    const bool closed = close(descriptor) == 0;
    work.root = hash.finish();
    return read_ok && closed && work.source_size == Size && work.bytes == Size;
}

class LittleCursor final {
public:
    LittleCursor(std::span<const std::uint8_t> bytes, CheckerWork& work)
        : bytes_(bytes), work_(work) {}

    Slice bytes(std::size_t count) noexcept {
        ++work_.cache_parse_checks;
        if (!ok_ || count > bytes_.size() - offset_) {
            ok_ = false;
            return {};
        }
        const Slice result{bytes_.data() + offset_, count};
        offset_ += count;
        return result;
    }

    std::uint8_t byte() noexcept {
        const Slice value = bytes(1U);
        return value.data == nullptr ? 0U : value.data[0U];
    }

    bool boolean() noexcept {
        const std::uint8_t value = byte();
        ++work_.cache_parse_checks;
        if (value > 1U) ok_ = false;
        return value != 0U;
    }

    std::uint32_t u32() noexcept {
        const Slice value = bytes(4U);
        if (value.data == nullptr) return 0U;
        return static_cast<std::uint32_t>(value.data[0U])
            | (static_cast<std::uint32_t>(value.data[1U]) << 8U)
            | (static_cast<std::uint32_t>(value.data[2U]) << 16U)
            | (static_cast<std::uint32_t>(value.data[3U]) << 24U);
    }

    std::uint64_t u64() noexcept {
        const Slice value = bytes(8U);
        if (value.data == nullptr) return 0U;
        std::uint64_t result = 0U;
        for (std::size_t index = 0U; index < 8U; ++index)
            result |= static_cast<std::uint64_t>(value.data[index])
                << (8U * index);
        return result;
    }

    Slice string() noexcept {
        const std::uint64_t count = u64();
        ++work_.cache_parse_checks;
        if (count > CACHE_SIZE) {
            ok_ = false;
            return {};
        }
        return bytes(static_cast<std::size_t>(count));
    }

    RawVector vector(std::size_t width) noexcept {
        const std::uint64_t count = u64();
        ++work_.cache_parse_checks;
        if (width == 0U || count > (bytes_.size() - offset_) / width) {
            ok_ = false;
            return {};
        }
        const Slice payload = bytes(static_cast<std::size_t>(count) * width);
        return {payload.data, count, false};
    }

    void skip_string() noexcept { static_cast<void>(string()); }
    void skip_vector(std::size_t width) noexcept {
        static_cast<void>(vector(width));
    }

    bool ok() const noexcept { return ok_; }
    std::size_t offset() const noexcept { return offset_; }

private:
    std::span<const std::uint8_t> bytes_;
    CheckerWork& work_;
    std::size_t offset_ = 0U;
    bool ok_ = true;
};

void skip_certificate(LittleCursor& reader) noexcept {
    static_cast<void>(reader.boolean());
    if (!reader.ok()) return;
    static_cast<void>(reader.boolean());
    if (!reader.ok()) return;
    static_cast<void>(reader.bytes(3U * 8U));
    static_cast<void>(reader.bytes(8U));
    for (std::size_t index = 0U; index < 3U; ++index) reader.skip_string();
}

void skip_certificates(LittleCursor& reader) noexcept {
    const std::uint64_t count = reader.u64();
    if (count > 64U) {
        static_cast<void>(reader.bytes(CACHE_SIZE));
        return;
    }
    for (std::uint64_t index = 0U; index < count && reader.ok(); ++index)
        skip_certificate(reader);
}

void skip_profile(LittleCursor& reader) noexcept {
    for (std::size_t index = 0U; index < 3U && reader.ok(); ++index)
        static_cast<void>(reader.boolean());
    if (!reader.ok()) return;
    static_cast<void>(reader.bytes(6U * 8U));
    for (std::size_t index = 0U; index < 4U; ++index)
        reader.skip_vector(8U);
    static_cast<void>(reader.bytes(3U * 8U));
    for (std::size_t index = 0U; index < 4U; ++index)
        reader.skip_string();
}

struct SelectedCache final {
    std::uint64_t rows = 0U;
    std::uint64_t columns = 0U;
    std::array<RawVector, ROLE_COUNT> inputs{};
    RawVector tangent{};
    Slice sigma{};
    Slice inverse{};
    Slice projected_scale{};
    Slice tangent_identity{};
    Slice original_identity{};
    Slice projected_identity{};
};

bool bytes_equal(Slice left,
    std::span<const std::uint8_t> right) noexcept {
    if (left.size != right.size()) return false;
    for (std::size_t index = 0U; index < left.size; ++index)
        if (left.data[index] != right[index]) return false;
    return true;
}

SelectedCache decode_cache(LittleCursor& reader) noexcept {
    SelectedCache selected;
    if (!bytes_equal(reader.bytes(CACHE_MAGIC.size()), CACHE_MAGIC))
        return selected;
    if (reader.u32() != 1U || reader.u32() != 0x01020304U
        || reader.u32() != 8U || reader.u32() != 16U)
        return selected;
    static_cast<void>(reader.boolean());
    reader.skip_string();
    selected.rows = reader.u64();
    selected.columns = reader.u64();
    for (std::size_t index = 0U; index < 9U; ++index)
        reader.skip_string();
    if (reader.u64() != 3U) return selected;
    selected.inputs[2U] = reader.vector(16U);
    selected.inputs[3U] = reader.vector(16U);
    selected.inputs[4U] = reader.vector(16U);
    skip_certificates(reader);
    if (!reader.ok()) return selected;
    selected.tangent = reader.vector(16U);
    selected.sigma = reader.bytes(16U);
    selected.tangent_identity = reader.string();
    reader.skip_vector(8U);
    const std::uint64_t permutation = reader.u64();
    if (permutation > 4096U) return selected;
    for (std::uint64_t index = 0U; index < permutation; ++index)
        static_cast<void>(reader.u64());
    selected.inverse = reader.bytes(16U);
    for (std::size_t index = 0U; index < 3U; ++index)
        reader.skip_string();
    selected.inputs[1U] = reader.vector(16U);
    selected.inputs[0U] = reader.vector(16U);
    selected.projected_scale = reader.bytes(16U);
    selected.original_identity = reader.string();
    selected.projected_identity = reader.string();
    reader.skip_vector(8U);
    for (std::size_t index = 0U; index < 3U; ++index)
        reader.skip_string();
    skip_profile(reader);
    if (!reader.ok()) return selected;
    reader.skip_string();
    reader.skip_string();
    if (reader.u64() != 3U) return selected;
    reader.skip_vector(16U);
    reader.skip_vector(16U);
    selected.inputs[5U] = reader.vector(16U);
    skip_certificates(reader);
    if (!reader.ok()) return selected;
    for (std::size_t index = 0U; index < 3U; ++index)
        reader.skip_string();
    return selected;
}

struct ProducerParseWork final {
    std::uint64_t take_calls = 0U;
    std::uint64_t take_bytes = 0U;
    std::uint64_t u8_reads = 0U;
    std::uint64_t u32_reads = 0U;
    std::uint64_t u64_reads = 0U;
    std::uint64_t string_reads = 0U;
    std::uint64_t vector_reads = 0U;
    std::uint64_t skip_operations = 0U;
    std::uint64_t structural_predicates = 0U;
    std::uint64_t views_created = 0U;
};

class ProducerModelCursor final {
public:
    ProducerModelCursor(std::span<const std::uint8_t> bytes,
        ProducerParseWork& work, CheckerWork& checker)
        : bytes_(bytes), work_(work), checker_(checker) {}

    Slice bytes(std::size_t count) noexcept {
        ++work_.take_calls;
        ++checker_.model_take_calls;
        if (!ok_ || count > bytes_.size() - offset_) {
            ok_ = false;
            return {};
        }
        const Slice result{bytes_.data() + offset_, count};
        offset_ += count;
        work_.take_bytes += count;
        checker_.model_take_bytes += count;
        ++work_.views_created;
        return result;
    }
    std::uint8_t byte() noexcept {
        ++work_.u8_reads;
        ++checker_.model_typed_operations;
        const Slice value = bytes(1U);
        return value.data == nullptr ? 0U : value.data[0U];
    }
    std::uint32_t u32() noexcept {
        ++work_.u32_reads;
        ++checker_.model_typed_operations;
        const Slice value = bytes(4U);
        if (value.data == nullptr) return 0U;
        return static_cast<std::uint32_t>(value.data[0U])
            | (static_cast<std::uint32_t>(value.data[1U]) << 8U)
            | (static_cast<std::uint32_t>(value.data[2U]) << 16U)
            | (static_cast<std::uint32_t>(value.data[3U]) << 24U);
    }
    std::uint64_t u64() noexcept {
        ++work_.u64_reads;
        ++checker_.model_typed_operations;
        const Slice value = bytes(8U);
        if (value.data == nullptr) return 0U;
        std::uint64_t result = 0U;
        for (std::size_t index = 0U; index < 8U; ++index)
            result |= static_cast<std::uint64_t>(value.data[index])
                << (8U * index);
        return result;
    }
    Slice string() noexcept {
        ++work_.string_reads;
        ++checker_.model_typed_operations;
        const std::uint64_t count = u64();
        if (!expect(count <= CACHE_SIZE)) return {};
        return bytes(static_cast<std::size_t>(count));
    }
    RawVector vector(std::size_t width) noexcept {
        ++work_.vector_reads;
        ++checker_.model_typed_operations;
        const std::uint64_t count = u64();
        if (!expect(width != 0U
            && count <= (bytes_.size() - offset_) / width)) return {};
        const Slice value = bytes(static_cast<std::size_t>(count) * width);
        return {value.data, count, false};
    }
    void skip_string() noexcept {
        ++work_.skip_operations;
        ++checker_.model_typed_operations;
        static_cast<void>(string());
    }
    void skip_vector(std::size_t width) noexcept {
        ++work_.skip_operations;
        ++checker_.model_typed_operations;
        static_cast<void>(vector(width));
    }
    bool expect(bool value) noexcept {
        ++work_.structural_predicates;
        ++checker_.model_typed_operations;
        if (!value) ok_ = false;
        return value;
    }
    bool ok() const noexcept { return ok_; }
    std::size_t offset() const noexcept { return offset_; }

private:
    std::span<const std::uint8_t> bytes_;
    ProducerParseWork& work_;
    CheckerWork& checker_;
    std::size_t offset_ = 0U;
    bool ok_ = true;
};

void producer_skip_certificate(ProducerModelCursor& reader) noexcept {
    if (!reader.expect(reader.byte() <= 1U)) return;
    if (!reader.expect(reader.byte() <= 1U)) return;
    static_cast<void>(reader.bytes(3U * 8U));
    static_cast<void>(reader.bytes(8U));
    for (std::size_t index = 0U; index < 3U; ++index) reader.skip_string();
}

void producer_skip_certificates(ProducerModelCursor& reader) noexcept {
    const std::uint64_t count = reader.u64();
    if (!reader.expect(count <= 64U)) {
        static_cast<void>(reader.bytes(CACHE_SIZE));
        return;
    }
    for (std::uint64_t index = 0U; index < count && reader.ok(); ++index)
        producer_skip_certificate(reader);
}

void producer_skip_profile(ProducerModelCursor& reader) noexcept {
    for (std::size_t index = 0U; index < 3U; ++index)
        if (!reader.expect(reader.byte() <= 1U)) return;
    static_cast<void>(reader.bytes(6U * 8U));
    for (std::size_t index = 0U; index < 4U; ++index)
        reader.skip_vector(8U);
    static_cast<void>(reader.bytes(3U * 8U));
    for (std::size_t index = 0U; index < 4U; ++index)
        reader.skip_string();
}

SelectedCache decode_producer_model(ProducerModelCursor& reader) noexcept {
    SelectedCache value;
    if (!reader.expect(bytes_equal(
            reader.bytes(CACHE_MAGIC.size()), CACHE_MAGIC))) return value;
    if (!reader.expect(reader.u32() == 1U)) return value;
    if (!reader.expect(reader.u32() == 0x01020304U)) return value;
    if (!reader.expect(reader.u32() == 8U)) return value;
    if (!reader.expect(reader.u32() == 16U)) return value;
    if (!reader.expect(reader.byte() <= 1U)) return value;
    reader.skip_string();
    value.rows = reader.u64();
    value.columns = reader.u64();
    for (std::size_t index = 0U; index < 9U; ++index) reader.skip_string();
    if (!reader.expect(reader.u64() == 3U)) return value;
    value.inputs[2U] = reader.vector(16U);
    value.inputs[3U] = reader.vector(16U);
    value.inputs[4U] = reader.vector(16U);
    producer_skip_certificates(reader);
    if (!reader.ok()) return value;
    value.tangent = reader.vector(16U);
    value.sigma = reader.bytes(16U);
    value.tangent_identity = reader.string();
    reader.skip_vector(8U);
    const std::uint64_t index_count = reader.u64();
    if (!reader.expect(index_count <= 4096U)) return value;
    for (std::uint64_t index = 0U; index < index_count; ++index)
        static_cast<void>(reader.u64());
    value.inverse = reader.bytes(16U);
    for (std::size_t index = 0U; index < 3U; ++index) reader.skip_string();
    value.inputs[1U] = reader.vector(16U);
    value.inputs[0U] = reader.vector(16U);
    value.projected_scale = reader.bytes(16U);
    value.original_identity = reader.string();
    value.projected_identity = reader.string();
    reader.skip_vector(8U);
    for (std::size_t index = 0U; index < 3U; ++index) reader.skip_string();
    producer_skip_profile(reader);
    if (!reader.ok()) return value;
    reader.skip_string();
    reader.skip_string();
    if (!reader.expect(reader.u64() == 3U)) return value;
    reader.skip_vector(16U);
    reader.skip_vector(16U);
    value.inputs[5U] = reader.vector(16U);
    producer_skip_certificates(reader);
    if (!reader.ok()) return value;
    for (std::size_t index = 0U; index < 3U; ++index) reader.skip_string();
    return value;
}

struct ProducerReadExpectation final {
    ProducerParseWork parse{};
    std::uint64_t payload_root_calls = 0U;
    std::uint64_t payload_root_metadata_bytes = 0U;
    std::uint64_t payload_root_payload_bytes = 0U;
    std::uint64_t selected_root_calls = 0U;
    std::uint64_t selected_root_bytes = 0U;

    std::array<std::uint64_t, READ_WORK_COUNT> fields(
        const FileWork& file, CheckerWork& checker) const noexcept {
        checker.model_reconstruction_operations += READ_WORK_COUNT;
        return {{file.open_calls, file.stat_calls, file.read_calls, file.bytes,
            file.close_calls, file.source_size, file.hash_calls,
            file.hash_bytes, parse.take_calls, parse.take_bytes,
            parse.u8_reads, parse.u32_reads, parse.u64_reads,
            parse.string_reads, parse.vector_reads, parse.skip_operations,
            parse.structural_predicates, parse.views_created,
            payload_root_calls, payload_root_metadata_bytes,
            payload_root_payload_bytes, selected_root_calls,
            selected_root_bytes, 0U, 1U, 271U}};
    }
};

struct ProducerAdmissionExpectation final {
    CheckerWork* checker = nullptr;
    std::uint64_t predicate_evaluations = 0U;
    std::uint64_t first_failure_ordinal = 0U;
    std::uint64_t digest_comparisons = 0U;
    std::uint64_t digest_bytes_examined = 0U;
    std::uint64_t text_comparisons = 0U;
    std::uint64_t text_bytes_examined = 0U;
    std::uint64_t scalar_comparisons = 0U;
    std::uint64_t scalar_bytes_examined = 0U;
    std::uint64_t binary128_loads = 0U;
    std::uint64_t positivity_checks = 0U;
    std::uint64_t finite_checks = 0U;
    std::uint64_t division_calls = 0U;
    std::uint64_t bundle_root_calls = 0U;
    std::uint64_t bundle_root_bytes = 0U;

    std::array<std::uint64_t, ADMISSION_WORK_COUNT> fields(
        CheckerWork& work) const noexcept {
        work.model_reconstruction_operations += ADMISSION_WORK_COUNT;
        return {{predicate_evaluations, first_failure_ordinal,
            digest_comparisons, digest_bytes_examined, text_comparisons,
            text_bytes_examined, scalar_comparisons, scalar_bytes_examined,
            binary128_loads, positivity_checks, finite_checks, division_calls,
            bundle_root_calls, bundle_root_bytes, 1U, 212U, 0U, 1U}};
    }
};

struct ProducerSealExpectation final {
    std::uint64_t product_set_root_calls = 0U;
    std::uint64_t product_set_root_bytes = 0U;
    std::uint64_t event_count = 0U;
    std::uint64_t trace_bytes = 0U;
    std::uint64_t product_records = 0U;

    std::array<std::uint64_t, SEAL_WORK_COUNT> fields(
        CheckerWork& work) const noexcept {
        work.model_reconstruction_operations += SEAL_WORK_COUNT;
        return {{product_set_root_calls, product_set_root_bytes,
            event_count, event_count * 159U, 1U, trace_bytes,
            1U, 12983U, 1U, 153U, product_records,
            product_records * 1944U, product_records * ROW_COUNT,
            product_records * ROW_COUNT * sizeof(Quad), 1U, 0U}};
    }
};

class BigCursor final {
public:
    BigCursor(std::span<const std::uint8_t> bytes, CheckerWork& work)
        : bytes_(bytes), work_(work) {}

    Slice bytes(std::size_t count) noexcept {
        ++work_.artifact_structure_checks;
        if (!ok_ || count > bytes_.size() - offset_) {
            ok_ = false;
            return {};
        }
        const Slice result{bytes_.data() + offset_, count};
        offset_ += count;
        return result;
    }

    std::uint8_t byte() noexcept {
        const Slice value = bytes(1U);
        return value.data == nullptr ? 0U : value.data[0U];
    }

    std::uint32_t u32() noexcept {
        const Slice value = bytes(4U);
        if (value.data == nullptr) return 0U;
        std::uint32_t result = 0U;
        for (std::size_t index = 0U; index < 4U; ++index)
            result = (result << 8U) | value.data[index];
        return result;
    }

    std::uint64_t u64() noexcept {
        const Slice value = bytes(8U);
        if (value.data == nullptr) return 0U;
        std::uint64_t result = 0U;
        for (std::size_t index = 0U; index < 8U; ++index)
            result = (result << 8U) | value.data[index];
        return result;
    }

    Digest digest() noexcept {
        Digest result{};
        const Slice value = bytes(result.size());
        if (value.data != nullptr)
            for (std::size_t index = 0U; index < result.size(); ++index)
                result[index] = value.data[index];
        return result;
    }

    bool zeros(std::size_t count) noexcept {
        const Slice value = bytes(count);
        if (value.data == nullptr) return false;
        bool exact = true;
        for (std::size_t index = 0U; index < value.size; ++index) {
            ++work_.artifact_structure_checks;
            exact = exact && value.data[index] == 0U;
        }
        return exact;
    }

    template <std::size_t Count>
    std::array<std::uint64_t, Count> work() noexcept {
        std::array<std::uint64_t, Count> result{};
        for (std::uint64_t& field : result) field = u64();
        return result;
    }

    bool ok() const noexcept { return ok_; }
    std::size_t offset() const noexcept { return offset_; }

private:
    std::span<const std::uint8_t> bytes_;
    CheckerWork& work_;
    std::size_t offset_ = 0U;
    bool ok_ = true;
};

struct ArtifactProduct final {
    std::uint8_t exact = 0U;
    std::uint8_t no_underflow = 0U;
    std::uint8_t role = 0U;
    bool padding = false;
    std::array<std::uint64_t, PRODUCT_WORK_COUNT> work{};
    std::array<Digest, 4U> vector_roots{};
    Digest root{};
    std::uint64_t value_count = 0U;
    Slice value_bytes{};
};

struct Artifact final {
    bool padding = false;
    std::uint32_t version = 0U;
    std::uint64_t total_bytes = 0U;
    std::uint64_t source_bytes = 0U;
    Digest source_root{};
    std::uint32_t route = 0U;
    Digest selected_root{};
    Digest bundle_root{};
    Digest product_set_root{};
    std::uint32_t event_count = 0U;
    std::array<Digest, EVENT_COUNT> events{};
    Digest trace_root{};
    Digest result_root{};
    std::array<std::uint64_t, READ_WORK_COUNT> read_work{};
    std::uint8_t admission_exact = 0U;
    std::array<std::uint64_t, ADMISSION_WORK_COUNT> admission_work{};
    std::array<std::uint64_t, SEAL_WORK_COUNT> seal_work{};
    std::array<Digest, PAYLOAD_COUNT> payload_roots{};
    std::uint32_t product_count = 0U;
    std::array<ArtifactProduct, ROLE_COUNT> products{};
    std::uint64_t product_set_hash_calls = 0U;
    std::uint64_t product_set_hash_bytes = 0U;
};

Artifact decode_artifact(BigCursor& reader) noexcept {
    Artifact artifact;
    artifact.padding = bytes_equal(
        reader.bytes(ARTIFACT_MAGIC.size()), ARTIFACT_MAGIC);
    artifact.version = reader.u32();
    artifact.total_bytes = reader.u64();
    artifact.source_bytes = reader.u64();
    artifact.source_root = reader.digest();
    artifact.route = reader.u32();
    artifact.selected_root = reader.digest();
    artifact.bundle_root = reader.digest();
    artifact.product_set_root = reader.digest();
    artifact.event_count = reader.u32();
    for (Digest& event : artifact.events) event = reader.digest();
    artifact.trace_root = reader.digest();
    artifact.result_root = reader.digest();
    artifact.read_work = reader.work<READ_WORK_COUNT>();
    artifact.admission_exact = reader.byte();
    artifact.padding = reader.zeros(7U) && artifact.padding;
    artifact.admission_work = reader.work<ADMISSION_WORK_COUNT>();
    artifact.seal_work = reader.work<SEAL_WORK_COUNT>();
    for (Digest& root : artifact.payload_roots) root = reader.digest();
    artifact.product_count = reader.u32();
    artifact.padding = reader.zeros(4U) && artifact.padding;
    for (ArtifactProduct& product : artifact.products) {
        product.exact = reader.byte();
        product.no_underflow = reader.byte();
        product.role = reader.byte();
        product.padding = reader.zeros(5U);
        product.work = reader.work<PRODUCT_WORK_COUNT>();
        for (Digest& root : product.vector_roots) root = reader.digest();
        product.root = reader.digest();
        product.value_count = reader.u64();
        product.value_bytes = reader.bytes(ROW_COUNT * sizeof(Quad));
    }
    artifact.product_set_hash_calls = reader.u64();
    artifact.product_set_hash_bytes = reader.u64();
    return artifact;
}

std::array<std::uint8_t, 16U> canonical(Quad value) noexcept {
    std::array<std::uint8_t, 16U> host{};
    std::array<std::uint8_t, 16U> output{};
    std::memcpy(host.data(), &value, host.size());
    for (std::size_t index = 0U; index < output.size(); ++index)
        output[index] = host[output.size() - 1U - index];
    return output;
}

Quad from_little(const std::uint8_t* bytes) noexcept {
    if (bytes == nullptr) return QZERO;
    Quad value = QZERO;
    std::memcpy(&value, bytes, sizeof(value));
    return value;
}

Quad raw_at(const RawVector& vector, std::size_t index) noexcept {
    if (index >= vector.count) return QZERO;
    if (!vector.canonical_big_endian)
        return from_little(vector.data + index * sizeof(Quad));
    std::array<std::uint8_t, 16U> host{};
    for (std::size_t byte = 0U; byte < host.size(); ++byte)
        host[byte] = vector.data[index * sizeof(Quad)
            + host.size() - 1U - byte];
    return from_little(host.data());
}

Digest root_raw_vector(const RawVector& vector, CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, VECTOR_DOMAIN);
    tlv_u64(hash, 0x02U, vector.count);
    tlv_header(hash, 0x03U, vector.count * sizeof(Quad));
    for (std::uint64_t index = 0U; index < vector.count; ++index) {
        if (vector.canonical_big_endian) {
            hash.update(std::span<const std::uint8_t>(
                vector.data + index * sizeof(Quad), sizeof(Quad)));
        } else {
            std::array<std::uint8_t, 16U> bytes{};
            for (std::size_t byte = 0U; byte < bytes.size(); ++byte)
                bytes[byte] = vector.data[
                    index * sizeof(Quad) + bytes.size() - 1U - byte];
            hash.update(bytes);
        }
    }
    record_hash(work, 80U + vector.count * sizeof(Quad));
    return hash.finish();
}

template <std::size_t Count>
Digest root_quad_array(const std::array<Quad, Count>& values,
    CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, VECTOR_DOMAIN);
    tlv_u64(hash, 0x02U, values.size());
    tlv_header(hash, 0x03U, values.size() * sizeof(Quad));
    for (Quad value : values) hash.update(canonical(value));
    record_hash(work, 80U + values.size() * sizeof(Quad));
    return hash.finish();
}

Digest root_selected(const SelectedCache& selected,
    const std::array<Digest, PAYLOAD_COUNT>& roots,
    CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, SELECTED_DOMAIN);
    tlv_u64(hash, 0x02U, selected.rows);
    tlv_u64(hash, 0x03U, selected.columns);
    const auto inverse = canonical(from_little(selected.inverse.data));
    const auto projected = canonical(from_little(selected.projected_scale.data));
    const auto sigma = canonical(from_little(selected.sigma.data));
    tlv_header(hash, 0x04U, inverse.size()); hash.update(inverse);
    tlv_header(hash, 0x05U, projected.size()); hash.update(projected);
    tlv_header(hash, 0x06U, sigma.size()); hash.update(sigma);
    tlv_u64(hash, 0x07U, selected.tangent.count);
    tlv_digest(hash, 0x08U, roots[0U]);
    for (std::size_t role = 0U; role < ROLE_COUNT; ++role) {
        tlv_byte(hash, static_cast<std::uint8_t>(0x10U + 3U * role),
            static_cast<std::uint8_t>(role));
        tlv_u64(hash, static_cast<std::uint8_t>(0x11U + 3U * role),
            selected.inputs[role].count);
        tlv_digest(hash, static_cast<std::uint8_t>(0x12U + 3U * role),
            roots[role + 1U]);
    }
    tlv_header(hash, 0x30U, selected.tangent_identity.size);
    hash.update({selected.tangent_identity.data,
        selected.tangent_identity.size});
    tlv_header(hash, 0x31U, selected.projected_identity.size);
    hash.update({selected.projected_identity.data,
        selected.projected_identity.size});
    tlv_header(hash, 0x32U, selected.original_identity.size);
    hash.update({selected.original_identity.data,
        selected.original_identity.size});
    record_hash(work, 847U);
    return hash.finish();
}

struct EFT final {
    bool exact = false;
    bool normal = false;
    Quad primary = QZERO;
    Quad error = QZERO;
};

bool normal_value(Quad value) noexcept {
    return finiteq(value) != 0
        && (value == QZERO || fabsq(value) >= ldexpq(QONE, -16382));
}

Quad upper_add(Quad left, Quad right) noexcept {
    if (left == QZERO && right == QZERO) return QZERO;
    return nextafterq(left + right, HUGE_VALQ);
}

Quad upper_multiply(Quad left, Quad right) noexcept {
    if (left == QZERO || right == QZERO) return QZERO;
    return nextafterq(left * right, HUGE_VALQ);
}

Quad upper_divide(Quad numerator, Quad denominator) noexcept {
    if (numerator == QZERO) return QZERO;
    return nextafterq(numerator / denominator, HUGE_VALQ);
}

EFT exact_sum(Quad left, Quad right) noexcept {
    EFT value;
    value.primary = left + right;
    const Quad right_virtual = value.primary - left;
    value.error = (left - (value.primary - right_virtual))
        + (right - right_virtual);
    value.normal = normal_value(left) && normal_value(right)
        && normal_value(value.primary) && normal_value(right_virtual)
        && normal_value(value.error);
    value.exact = finiteq(value.primary) != 0 && finiteq(value.error) != 0;
    return value;
}

EFT exact_product(Quad left, Quad right) noexcept {
    EFT value;
    value.primary = left * right;
    value.error = fmaq(left, right, -value.primary);
    const bool retained = left == QZERO || right == QZERO
        || value.primary != QZERO || value.error != QZERO;
    value.normal = retained && normal_value(left) && normal_value(right)
        && normal_value(value.primary) && normal_value(value.error);
    value.exact = finiteq(value.primary) != 0 && finiteq(value.error) != 0;
    return value;
}

struct BoundedDot final {
    bool exact = false;
    bool normal = false;
    Quad value = QZERO;
    Quad bound = QZERO;
};

template <class Left, class Right>
BoundedDot replay_dot(std::size_t count, const Left& left,
    const Right& right) noexcept {
    BoundedDot output;
    if (count == 0U) return output;
    const EFT initial = exact_product(left(0U), right(0U));
    Quad primary = initial.primary;
    Quad correction = initial.error;
    Quad absolute = upper_multiply(fabsq(left(0U)), fabsq(right(0U)));
    bool normal = initial.normal;
    for (std::size_t index = 1U; index < count; ++index) {
        const EFT product = exact_product(left(index), right(index));
        const EFT sum = exact_sum(primary, product.primary);
        const Quad local = sum.error + product.error;
        correction = correction + local;
        primary = sum.primary;
        normal = normal && product.normal && sum.normal
            && normal_value(local) && normal_value(correction);
        absolute = upper_add(absolute,
            upper_multiply(fabsq(left(index)), fabsq(right(index))));
    }
    output.value = primary + correction;
    normal = normal && normal_value(output.value);
    const Quad unit = ldexpq(QONE, -112);
    const Quad count_unit = static_cast<Quad>(count) * unit;
    const Quad gamma = upper_divide(count_unit, QONE - count_unit);
    const Quad numerator = upper_add(
        upper_multiply(unit, fabsq(output.value)),
        upper_multiply(upper_multiply(gamma, gamma), absolute));
    output.bound = upper_divide(numerator, QONE - unit);
    output.normal = normal;
    output.exact = initial.exact && finiteq(output.value) != 0
        && finiteq(output.bound) != 0 && output.bound >= QZERO;
    return output;
}

struct ReplayProduct final {
    bool exact = false;
    bool no_underflow = false;
    std::array<Quad, COLUMN_COUNT> intermediate{};
    std::array<Quad, COLUMN_COUNT> intermediate_bound{};
    std::array<Quad, ROW_COUNT> value{};
    std::array<Quad, ROW_COUNT> bound{};
    std::array<Digest, 4U> roots{};
    Digest root{};
};

constexpr std::array<std::uint64_t, PRODUCT_WORK_COUNT> PRODUCT_WORK{{
    1U,1U,315U,102U,32130U,32130U,32130U,102U,4U,834U,4U,320U,
    13344U,1U,466U,0U,0U}};

Digest root_product(std::size_t role, bool exact, bool no_underflow,
    const Digest& input, const std::array<Digest, 4U>& roots,
    CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, PRODUCT_DOMAIN);
    tlv_byte(hash, 0x02U, static_cast<std::uint8_t>(role));
    tlv_byte(hash, 0x03U, exact ? 1U : 0U);
    tlv_byte(hash, 0x04U, no_underflow ? 1U : 0U);
    tlv_u64(hash, 0x05U, ROW_COUNT);
    tlv_u64(hash, 0x06U, COLUMN_COUNT);
    tlv_digest(hash, 0x07U, input);
    tlv_array(hash, 0x08U, PRODUCT_WORK);
    for (std::size_t index = 0U; index < roots.size(); ++index)
        tlv_digest(hash, static_cast<std::uint8_t>(0x10U + index),
            roots[index]);
    record_hash(work, 466U);
    return hash.finish();
}

void replay_product(ReplayProduct& output, std::size_t role,
    const SelectedCache& selected, const Digest& input_root,
    CheckerWork& work) noexcept {
    bool exact = true;
    bool normal = true;
    for (std::size_t column = 0U; column < COLUMN_COUNT; ++column) {
        const BoundedDot dot = replay_dot(ROW_COUNT,
            [&](std::size_t row) {
                return raw_at(selected.tangent, row * COLUMN_COUNT + column);
            },
            [&](std::size_t row) {
                return raw_at(selected.inputs[role], row);
            });
        output.intermediate[column] = dot.value;
        output.intermediate_bound[column] = dot.bound;
        exact = exact && dot.exact && finiteq(dot.value) != 0
            && finiteq(dot.bound) != 0;
        normal = normal && dot.normal;
    }
    const Quad sigma = from_little(selected.sigma.data);
    for (std::size_t row = 0U; row < ROW_COUNT; ++row) {
        const BoundedDot dot = replay_dot(COLUMN_COUNT,
            [&](std::size_t column) {
                return raw_at(selected.tangent,
                    row * COLUMN_COUNT + column);
            },
            [&](std::size_t column) {
                return output.intermediate[column];
            });
        Quad propagation = dot.bound;
        for (std::size_t column = 0U; column < COLUMN_COUNT; ++column)
            propagation = upper_add(propagation, upper_multiply(
                fabsq(raw_at(selected.tangent,
                    row * COLUMN_COUNT + column)),
                output.intermediate_bound[column]));
        const EFT scale = exact_product(sigma, dot.value);
        output.value[row] = scale.primary;
        output.bound[row] = upper_add(fabsq(scale.error),
            upper_multiply(fabsq(sigma), propagation));
        exact = exact && dot.exact && scale.exact
            && finiteq(output.value[row]) != 0
            && finiteq(output.bound[row]) != 0;
        normal = normal && dot.normal && scale.normal;
    }
    output.exact = exact && normal;
    output.no_underflow = normal;
    output.roots = {{root_quad_array(output.intermediate, work),
        root_quad_array(output.intermediate_bound, work),
        root_quad_array(output.value, work),
        root_quad_array(output.bound, work)}};
    output.root = root_product(role, output.exact, output.no_underflow,
        input_root, output.roots, work);
    ++work.arithmetic_kernels;
    work.inner_terms += ROW_COUNT * COLUMN_COUNT;
    work.outer_terms += ROW_COUNT * COLUMN_COUNT;
    work.propagation_terms += ROW_COUNT * COLUMN_COUNT;
}

Digest root_bundle(const Digest& source, const Digest& selected,
    const std::array<Digest, PAYLOAD_COUNT>& payloads,
    CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, BUNDLE_DOMAIN);
    tlv_digest(hash, 0x02U, source);
    tlv_digest(hash, 0x03U, selected);
    for (std::size_t index = 0U; index < payloads.size(); ++index)
        tlv_digest(hash, static_cast<std::uint8_t>(0x10U + index),
            payloads[index]);
    record_hash(work, 429U);
    return hash.finish();
}

Digest root_product_set(const std::array<Digest, ROLE_COUNT>& products,
    CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, PRODUCT_SET_DOMAIN);
    tlv_u64(hash, 0x02U, products.size());
    for (std::size_t role = 0U; role < products.size(); ++role) {
        tlv_byte(hash, static_cast<std::uint8_t>(0x10U + 2U * role),
            static_cast<std::uint8_t>(role));
        tlv_digest(hash, static_cast<std::uint8_t>(0x11U + 2U * role),
            products[role]);
    }
    record_hash(work, 379U);
    return hash.finish();
}

template <std::size_t DomainSize, std::size_t Count>
Digest root_work(const char (&domain)[DomainSize],
    const std::array<std::uint64_t, Count>& fields,
    std::uint64_t bytes, CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, domain);
    tlv_array(hash, 0x02U, fields);
    record_hash(work, bytes);
    return hash.finish();
}

Digest root_event(std::uint8_t type, std::uint64_t ordinal,
    const Digest& child, const Digest& auxiliary,
    CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, EVENT_DOMAIN);
    tlv_byte(hash, 0x02U, type);
    tlv_u64(hash, 0x03U, ordinal);
    tlv_digest(hash, 0x04U, child);
    tlv_digest(hash, 0x05U, auxiliary);
    record_hash(work, 159U);
    return hash.finish();
}

Digest root_trace(std::uint8_t route, std::uint64_t event_count,
    const std::array<Digest, EVENT_COUNT>& events,
    CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, TRACE_DOMAIN);
    tlv_byte(hash, 0x02U, route);
    tlv_u64(hash, 0x03U, event_count);
    for (std::size_t index = 0U; index < event_count; ++index)
        tlv_digest(hash, static_cast<std::uint8_t>(0x10U + index),
            events[index]);
    record_hash(work, 77U + event_count * 41U);
    return hash.finish();
}

Digest root_artifact_body(CheckerWork& work) noexcept {
    constexpr std::array<std::uint8_t, 32U> ZERO_RESULT{};
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, BODY_DOMAIN);
    tlv_header(hash, 0x02U, artifact_bytes.size());
    hash.update(std::span<const std::uint8_t>(
        artifact_bytes.data(), RESULT_OFFSET));
    hash.update(ZERO_RESULT);
    hash.update(std::span<const std::uint8_t>(
        artifact_bytes.data() + RESULT_OFFSET + ZERO_RESULT.size(),
        artifact_bytes.size() - RESULT_OFFSET - ZERO_RESULT.size()));
    record_hash(work, 12983U);
    return hash.finish();
}

Digest root_result(std::uint8_t route, std::uint64_t source_bytes,
    const Digest& body,
    CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, RESULT_DOMAIN);
    tlv_u64(hash, 0x02U, 3U);
    tlv_byte(hash, 0x03U, route);
    tlv_u64(hash, 0x04U, source_bytes);
    tlv_digest(hash, 0x05U, body);
    tlv_u64(hash, 0x06U, ARTIFACT_SIZE);
    record_hash(work, 153U);
    return hash.finish();
}

bool text_is(Slice value, const char* expected) noexcept {
    std::size_t count = 0U;
    while (expected[count] != '\0') ++count;
    const std::size_t compared = value.size < count ? value.size : count;
    std::uint8_t difference = static_cast<std::uint8_t>(value.size != count);
    for (std::size_t index = 0U; index < compared; ++index)
        difference |= static_cast<std::uint8_t>(value.data[index]
            ^ static_cast<std::uint8_t>(expected[index]));
    return difference == 0U;
}

bool scalar_is(Slice value,
    const std::array<std::uint8_t, 16U>& expected) noexcept {
    if (value.size != expected.size()) return false;
    std::uint8_t difference = 0U;
    for (std::size_t index = 0U; index < expected.size(); ++index)
        difference |= static_cast<std::uint8_t>(
            value.data[expected.size() - 1U - index] ^ expected[index]);
    return difference == 0U;
}

bool model_predicate(ProducerAdmissionExpectation& work,
    bool value) noexcept {
    ++work.checker->model_reconstruction_operations;
    ++work.predicate_evaluations;
    if (!value && work.first_failure_ordinal == 0U)
        work.first_failure_ordinal = work.predicate_evaluations;
    return value;
}

bool model_digest_is(const Digest& digest, const char* expected,
    ProducerAdmissionExpectation& work) noexcept {
    ++work.checker->model_reconstruction_operations;
    ++work.digest_comparisons;
    work.digest_bytes_examined += digest.size();
    return digest_is(digest, expected);
}

bool model_text_is(Slice value, const char* expected,
    ProducerAdmissionExpectation& work) noexcept {
    ++work.checker->model_reconstruction_operations;
    ++work.text_comparisons;
    std::size_t count = 0U;
    while (expected[count] != '\0') ++count;
    work.text_bytes_examined += value.size < count ? value.size : count;
    return text_is(value, expected);
}

bool model_scalar_is(Slice value,
    const std::array<std::uint8_t, 16U>& expected,
    ProducerAdmissionExpectation& work) noexcept {
    ++work.checker->model_reconstruction_operations;
    ++work.scalar_comparisons;
    work.scalar_bytes_examined += value.size < 16U ? value.size : 16U;
    return scalar_is(value, expected);
}

bool model_admission(const SelectedCache& selected,
    const ProducerModelCursor& reader, const Digest& source,
    ProducerReadExpectation& read,
    ProducerAdmissionExpectation& admission,
    std::array<Digest, PAYLOAD_COUNT>& payload_roots,
    Digest& selected_root, Digest& bundle_root,
    CheckerWork& checker) noexcept {
    constexpr std::array<std::uint8_t, 16U> INVERSE{{
        0x40U,0x00U,0x62U,0x4bU,0xd7U,0x3cU,0xb7U,0xabU,
        0x65U,0x90U,0x62U,0xc1U,0xb0U,0xceU,0xfeU,0x41U}};
    constexpr std::array<std::uint8_t, 16U> PROJECTED{{
        0x40U,0x00U,0x62U,0x4bU,0xd7U,0x3cU,0xb7U,0xabU,
        0x65U,0x90U,0x62U,0xc1U,0xb0U,0xceU,0xfeU,0x40U}};
    constexpr std::array<const char*, PAYLOAD_COUNT> EXPECTED_PAYLOADS{{
        "03e18bacbb09013618182985941b23496d9e9bab0ca95cb3f9edfe22193e1ada",
        "2cb58eea5f6b17bc666a5d4b1c0c9ef644b2c10f154f5d651bf24e446ad6a729",
        "1cabfd31c413d1a36fc299955739196bb50924bebe744ae57c1c99f31a9fc5b9",
        "950cd25039eafe48f7bb5d5cbdd8b049fe6d14cd0a178338fd50c9d797710a5b",
        "1573eeeb09ca81f104de3d56e4bf268ac0e9839ce83525472bb9c51d803bf622",
        "4d59cf5bcaaea440e3f061d00cbb05872d4be75d408e9927050a3ffd1efff392",
        "1b5581ab5871dcd60b5ff1ce180d0572fcb9725dffe40570595c70fda866bc49",
    }};
    if (!model_predicate(admission,
            reader.ok() && reader.offset() == CACHE_SIZE)) return false;
    if (!model_predicate(admission, selected.rows == ROW_COUNT)) return false;
    if (!model_predicate(admission,
            selected.columns == COLUMN_COUNT)) return false;
    if (!model_predicate(admission,
            selected.tangent.count == ROW_COUNT * COLUMN_COUNT)) return false;
    if (!model_predicate(admission, model_text_is(selected.tangent_identity,
            "114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1",
            admission))) return false;
    if (!model_predicate(admission,
            model_scalar_is(selected.inverse, INVERSE, admission))) return false;
    if (!model_predicate(admission,
            model_scalar_is(selected.projected_scale, PROJECTED, admission)))
        return false;
    if (!model_predicate(admission, model_text_is(selected.projected_identity,
            "0df32db6fb6c7b3a6fb5e3760c7ff810f92e31baa1544a0911a8e0b6a11274ef",
            admission))) return false;
    if (!model_predicate(admission, model_text_is(selected.original_identity,
            "64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1",
            admission))) return false;
    ++admission.binary128_loads;
    ++admission.checker->model_reconstruction_operations;
    const Quad sigma = from_little(selected.sigma.data);
    ++admission.positivity_checks;
    ++admission.checker->model_reconstruction_operations;
    if (!model_predicate(admission, sigma > QZERO)) return false;
    ++admission.finite_checks;
    ++admission.checker->model_reconstruction_operations;
    if (!model_predicate(admission, finiteq(sigma) != 0)) return false;
    ++admission.binary128_loads;
    ++admission.checker->model_reconstruction_operations;
    const Quad inverse = from_little(selected.inverse.data);
    ++admission.division_calls;
    ++admission.checker->model_reconstruction_operations;
    ++admission.scalar_comparisons;
    ++admission.checker->model_reconstruction_operations;
    admission.scalar_bytes_examined += sizeof(Quad);
    if (!model_predicate(admission, sigma == QONE / inverse)) return false;
    for (const RawVector& input : selected.inputs)
        if (!model_predicate(admission, input.count == ROW_COUNT)) return false;
    ++read.payload_root_calls;
    read.payload_root_metadata_bytes += 80U;
    read.payload_root_payload_bytes += selected.tangent.count * sizeof(Quad);
    payload_roots[0U] = root_raw_vector(selected.tangent, checker);
    if (!model_predicate(admission, model_digest_is(payload_roots[0U],
            EXPECTED_PAYLOADS[0U], admission))) return false;
    for (std::size_t role = 0U; role < ROLE_COUNT; ++role) {
        ++read.payload_root_calls;
        read.payload_root_metadata_bytes += 80U;
        read.payload_root_payload_bytes +=
            selected.inputs[role].count * sizeof(Quad);
        payload_roots[role + 1U] = root_raw_vector(
            selected.inputs[role], checker);
        if (!model_predicate(admission, model_digest_is(
                payload_roots[role + 1U], EXPECTED_PAYLOADS[role + 1U],
                admission))) return false;
    }
    ++read.selected_root_calls;
    read.selected_root_bytes += 847U;
    selected_root = root_selected(selected, payload_roots, checker);
    if (!model_predicate(admission, model_digest_is(selected_root,
            "dc1b5328d2999dadafa8915ebe10d36029cc5b5ba13f2657670de75ee4a466a5",
            admission))) return false;
    if (!model_predicate(admission, model_digest_is(source,
            "23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84",
            admission))) return false;
    ++admission.bundle_root_calls;
    ++admission.checker->model_reconstruction_operations;
    admission.bundle_root_bytes += 429U;
    bundle_root = root_bundle(source, selected_root, payload_roots, checker);
    return true;
}

struct IndependentAdmission final {
    bool accepted = false;
    std::uint64_t predicates = 0U;
    std::uint64_t first_failure = 0U;
    std::array<Digest, PAYLOAD_COUNT> payload_roots{};
    Digest selected_root{};
    Digest bundle_root{};
};

bool independent_predicate(IndependentAdmission& result, bool value,
    CheckerWork& work) noexcept {
    ++result.predicates;
    ++work.semantic_comparisons;
    if (!value && result.first_failure == 0U)
        result.first_failure = result.predicates;
    return value;
}

IndependentAdmission independent_admission(const SelectedCache& selected,
    bool parse_exact, const Digest& source, CheckerWork& work) noexcept {
    constexpr std::array<std::uint8_t, 16U> INVERSE{{
        0x40U,0x00U,0x62U,0x4bU,0xd7U,0x3cU,0xb7U,0xabU,
        0x65U,0x90U,0x62U,0xc1U,0xb0U,0xceU,0xfeU,0x41U}};
    constexpr std::array<std::uint8_t, 16U> PROJECTED{{
        0x40U,0x00U,0x62U,0x4bU,0xd7U,0x3cU,0xb7U,0xabU,
        0x65U,0x90U,0x62U,0xc1U,0xb0U,0xceU,0xfeU,0x40U}};
    constexpr std::array<const char*, PAYLOAD_COUNT> PAYLOADS{{
        "03e18bacbb09013618182985941b23496d9e9bab0ca95cb3f9edfe22193e1ada",
        "2cb58eea5f6b17bc666a5d4b1c0c9ef644b2c10f154f5d651bf24e446ad6a729",
        "1cabfd31c413d1a36fc299955739196bb50924bebe744ae57c1c99f31a9fc5b9",
        "950cd25039eafe48f7bb5d5cbdd8b049fe6d14cd0a178338fd50c9d797710a5b",
        "1573eeeb09ca81f104de3d56e4bf268ac0e9839ce83525472bb9c51d803bf622",
        "4d59cf5bcaaea440e3f061d00cbb05872d4be75d408e9927050a3ffd1efff392",
        "1b5581ab5871dcd60b5ff1ce180d0572fcb9725dffe40570595c70fda866bc49",
    }};
    IndependentAdmission result;
    if (!independent_predicate(result, parse_exact, work)) return result;
    if (!independent_predicate(result, selected.rows == ROW_COUNT, work))
        return result;
    if (!independent_predicate(result,
            selected.columns == COLUMN_COUNT, work)) return result;
    if (!independent_predicate(result,
            selected.tangent.count == ROW_COUNT * COLUMN_COUNT, work))
        return result;
    if (!independent_predicate(result, text_is(selected.tangent_identity,
            "114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1"),
            work)) return result;
    if (!independent_predicate(result,
            scalar_is(selected.inverse, INVERSE), work)) return result;
    if (!independent_predicate(result,
            scalar_is(selected.projected_scale, PROJECTED), work))
        return result;
    if (!independent_predicate(result, text_is(selected.projected_identity,
            "0df32db6fb6c7b3a6fb5e3760c7ff810f92e31baa1544a0911a8e0b6a11274ef"),
            work)) return result;
    if (!independent_predicate(result, text_is(selected.original_identity,
            "64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1"),
            work)) return result;
    const Quad sigma = from_little(selected.sigma.data);
    if (!independent_predicate(result, sigma > QZERO, work)) return result;
    if (!independent_predicate(result, finiteq(sigma) != 0, work))
        return result;
    const Quad inverse = from_little(selected.inverse.data);
    if (!independent_predicate(result, sigma == QONE / inverse, work))
        return result;
    for (const RawVector& input : selected.inputs)
        if (!independent_predicate(result,
                input.count == ROW_COUNT, work)) return result;
    result.payload_roots[0U] = root_raw_vector(selected.tangent, work);
    if (!independent_predicate(result,
            digest_is(result.payload_roots[0U], PAYLOADS[0U]), work))
        return result;
    for (std::size_t role = 0U; role < ROLE_COUNT; ++role) {
        result.payload_roots[role + 1U] = root_raw_vector(
            selected.inputs[role], work);
        if (!independent_predicate(result,
                digest_is(result.payload_roots[role + 1U],
                    PAYLOADS[role + 1U]), work)) return result;
    }
    result.selected_root = root_selected(
        selected, result.payload_roots, work);
    if (!independent_predicate(result, digest_is(result.selected_root,
            "dc1b5328d2999dadafa8915ebe10d36029cc5b5ba13f2657670de75ee4a466a5"),
            work)) return result;
    if (!independent_predicate(result, digest_is(source,
            "23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84"),
            work)) return result;
    result.bundle_root = root_bundle(
        source, result.selected_root, result.payload_roots, work);
    result.accepted = true;
    return result;
}

class ExpectedBodyHasher final {
public:
    ExpectedBodyHasher() {
        tlv_constant(hash_, 0x01U, BODY_DOMAIN);
        tlv_header(hash_, 0x02U, ARTIFACT_SIZE);
    }
    void raw(std::span<const std::uint8_t> value) noexcept {
        if (!ok_ || value.size() > ARTIFACT_SIZE - position_) {
            ok_ = false;
            return;
        }
        hash_.update(value);
        position_ += value.size();
    }
    template <std::size_t Count>
    void raw(const std::array<std::uint8_t, Count>& value) noexcept {
        raw(std::span<const std::uint8_t>(value));
    }
    void byte(std::uint8_t value) noexcept { raw({&value, 1U}); }
    void u32(std::uint32_t value) noexcept {
        for (std::size_t index = 0U; index < 4U; ++index)
            byte(static_cast<std::uint8_t>(value >> (24U - 8U * index)));
    }
    void u64(std::uint64_t value) noexcept {
        for (std::size_t index = 0U; index < 8U; ++index)
            byte(static_cast<std::uint8_t>(value >> (56U - 8U * index)));
    }
    template <std::size_t Count>
    void work(const std::array<std::uint64_t, Count>& fields) noexcept {
        for (std::uint64_t field : fields) u64(field);
    }
    void zeros(std::size_t count) noexcept {
        constexpr std::array<std::uint8_t, 256U> ZERO{};
        while (count != 0U) {
            const std::size_t chunk = count < ZERO.size() ? count : ZERO.size();
            raw(std::span<const std::uint8_t>(ZERO.data(), chunk));
            count -= chunk;
        }
    }
    Digest finish(CheckerWork& work) noexcept {
        if (!ok_ || position_ != ARTIFACT_SIZE) return {};
        record_hash(work, 12983U);
        return hash_.finish();
    }
private:
    r63zm::Sha256 hash_{};
    std::size_t position_ = 0U;
    bool ok_ = true;
};

Digest root_expected_artifact_body(std::uint8_t route,
    std::uint64_t source_bytes, const Digest& source,
    const Digest& selected, const Digest& bundle,
    const Digest& product_set, std::uint32_t event_count,
    const std::array<Digest, EVENT_COUNT>& events, const Digest& trace,
    const std::array<std::uint64_t, READ_WORK_COUNT>& read_work,
    bool admission_exact,
    const std::array<std::uint64_t, ADMISSION_WORK_COUNT>& admission_work,
    const std::array<std::uint64_t, SEAL_WORK_COUNT>& seal_work,
    const std::array<Digest, PAYLOAD_COUNT>& payload_roots,
    std::uint32_t product_count,
    const std::array<ReplayProduct, ROLE_COUNT>& products,
    CheckerWork& work) noexcept {
    ExpectedBodyHasher writer;
    writer.raw(ARTIFACT_MAGIC);
    writer.u32(3U);
    writer.u64(ARTIFACT_SIZE);
    writer.u64(source_bytes);
    writer.raw(source);
    writer.u32(route);
    writer.raw(selected);
    writer.raw(bundle);
    writer.raw(product_set);
    writer.u32(event_count);
    for (const Digest& event : events) writer.raw(event);
    writer.raw(trace);
    writer.zeros(32U);
    writer.work(read_work);
    writer.byte(admission_exact ? 1U : 0U);
    writer.zeros(7U);
    writer.work(admission_work);
    writer.work(seal_work);
    for (const Digest& root : payload_roots) writer.raw(root);
    writer.u32(product_count);
    writer.zeros(4U);
    for (std::size_t role = 0U; role < ROLE_COUNT; ++role) {
        const bool available = role < product_count;
        writer.byte(available && products[role].exact ? 1U : 0U);
        writer.byte(available && products[role].no_underflow ? 1U : 0U);
        writer.byte(available ? static_cast<std::uint8_t>(role) : 0U);
        writer.zeros(5U);
        if (available) {
            writer.work(PRODUCT_WORK);
            for (const Digest& root : products[role].roots) writer.raw(root);
            writer.raw(products[role].root);
            writer.u64(ROW_COUNT);
            for (Quad value : products[role].value) writer.raw(canonical(value));
        } else {
            writer.zeros(PRODUCT_WORK_COUNT * 8U + 4U * 32U + 32U + 8U
                + ROW_COUNT * sizeof(Quad));
        }
    }
    writer.u64(product_count == ROLE_COUNT ? 1U : 0U);
    writer.u64(product_count == ROLE_COUNT ? 379U : 0U);
    return writer.finish(work);
}

enum class CheckRoute : std::uint8_t {
    CandidateAccepted = 0U,
    ReadRejectedVerified = 1U,
    AdmissionRejectedVerified = 2U,
    ProductRejectedVerified = 3U,
    Malformed = 4U,
    Semantic = 5U,
    Work = 6U,
    Seal = 7U,
    Unknown = 8U,
};

Digest checker_root(CheckRoute route, bool semantic, bool work_exact,
    bool seals, const FileWork& cache_file, const FileWork& artifact_file,
    const Digest& producer_result, const Digest& reconstructed_result,
    CheckerWork& work) noexcept {
    r63zm::Sha256 hash;
    tlv_constant(hash, 0x01U, CHECKER_DOMAIN);
    tlv_byte(hash, 0x02U, static_cast<std::uint8_t>(route));
    tlv_byte(hash, 0x03U, semantic ? 1U : 0U);
    tlv_byte(hash, 0x04U, work_exact ? 1U : 0U);
    tlv_byte(hash, 0x05U, seals ? 1U : 0U);
    tlv_array(hash, 0x06U, work.fields());
    tlv_u64(hash, 0x07U, cache_file.source_size);
    tlv_digest(hash, 0x08U, cache_file.root);
    tlv_u64(hash, 0x09U, artifact_file.source_size);
    tlv_digest(hash, 0x0aU, artifact_file.root);
    tlv_digest(hash, 0x0bU, producer_result);
    tlv_digest(hash, 0x0cU, reconstructed_result);
    tlv_u64(hash, 0x0dU, AUDIT_OUTPUT_SIZE);
    return hash.finish();
}

class AuditWriter final {
public:
    explicit AuditWriter(std::array<std::uint8_t, AUDIT_OUTPUT_SIZE>& bytes)
        : bytes_(bytes) {}

    void raw(std::span<const std::uint8_t> value) noexcept {
        if (!ok_ || value.size() > bytes_.size() - offset_) {
            ok_ = false;
            return;
        }
        for (std::uint8_t byte : value) bytes_[offset_++] = byte;
    }

    template <std::size_t Count>
    void raw(const std::array<std::uint8_t, Count>& value) noexcept {
        raw(std::span<const std::uint8_t>(value));
    }

    void byte(std::uint8_t value) noexcept { raw({&value, 1U}); }

    void u32(std::uint32_t value) noexcept {
        for (std::size_t index = 0U; index < 4U; ++index)
            byte(static_cast<std::uint8_t>(value >> (24U - 8U * index)));
    }

    void u64(std::uint64_t value) noexcept {
        for (std::size_t index = 0U; index < 8U; ++index)
            byte(static_cast<std::uint8_t>(value >> (56U - 8U * index)));
    }

    template <std::size_t Count>
    void work(const std::array<std::uint64_t, Count>& values) noexcept {
        for (std::uint64_t value : values) u64(value);
    }

    bool complete() const noexcept {
        return ok_ && offset_ == bytes_.size();
    }

private:
    std::array<std::uint8_t, AUDIT_OUTPUT_SIZE>& bytes_;
    std::size_t offset_ = 0U;
    bool ok_ = true;
};

bool write_audit(const char* path,
    const std::array<std::uint8_t, AUDIT_OUTPUT_SIZE>& bytes) noexcept {
    const int descriptor = open(path, O_WRONLY | O_CREAT | O_TRUNC, 0600);
    if (descriptor < 0) return false;
    std::size_t offset = 0U;
    while (offset < bytes.size()) {
        const ssize_t count = write(descriptor, bytes.data() + offset,
            bytes.size() - offset);
        if (count <= 0) {
            static_cast<void>(close(descriptor));
            return false;
        }
        offset += static_cast<std::size_t>(count);
    }
    return close(descriptor) == 0;
}

} // namespace


int main(int argc, char** argv) {
    CheckerWork checker;
    if (argc != 4 || std::fesetround(FE_TONEAREST) != 0
        || std::fegetround() != FE_TONEAREST)
        return 2;

    FileWork cache_file;
    FileWork artifact_file;
    const bool cache_read = fixed_read(argv[1], cache_bytes, cache_file);
    const bool artifact_read = fixed_read(
        argv[2], artifact_bytes, artifact_file);
    checker.cache_read_calls = cache_file.read_calls;
    checker.artifact_read_calls = artifact_file.read_calls;
    checker.file_metadata_calls = cache_file.open_calls + cache_file.stat_calls
        + cache_file.close_calls + artifact_file.open_calls
        + artifact_file.stat_calls + artifact_file.close_calls;
    checker.cache_bytes_read = cache_file.bytes;
    checker.artifact_bytes_read = artifact_file.bytes;
    checker.hash_calls = cache_file.hash_calls + artifact_file.hash_calls;
    checker.hash_bytes = cache_file.hash_bytes + artifact_file.hash_bytes;

    BigCursor artifact_reader(artifact_bytes, checker);
    const Artifact artifact = decode_artifact(artifact_reader);

    ProducerReadExpectation producer_read;
    ProducerAdmissionExpectation producer_admission;
    producer_admission.checker = &checker;
    std::array<Digest, PAYLOAD_COUNT> model_payload_roots{};
    Digest model_selected_root{};
    Digest model_bundle_root{};
    SelectedCache model_selected{};
    bool model_admission_exact = false;
    if (cache_read) {
        ProducerModelCursor model_reader(
            cache_bytes, producer_read.parse, checker);
        model_selected = decode_producer_model(model_reader);
        model_admission_exact = model_admission(model_selected, model_reader,
            cache_file.root, producer_read, producer_admission,
            model_payload_roots, model_selected_root,
            model_bundle_root, checker);
    }

    SelectedCache independent_selected{};
    bool independent_parse_exact = false;
    if (cache_read) {
        LittleCursor independent_reader(cache_bytes, checker);
        independent_selected = decode_cache(independent_reader);
        independent_parse_exact = independent_reader.ok()
            && independent_reader.offset() == CACHE_SIZE;
    }
    const IndependentAdmission independent = cache_read
        ? independent_admission(independent_selected,
            independent_parse_exact, cache_file.root, checker)
        : IndependentAdmission{};
    const bool admission_exact = independent.accepted;
    const auto& expected_payload_roots = independent.payload_roots;
    const Digest& expected_selected_root = independent.selected_root;
    const Digest& expected_bundle_root = independent.bundle_root;
    std::uint8_t expected_producer_route = !cache_read ? 1U
        : admission_exact ? 0U : 2U;

    auto semantic = [&](bool value) noexcept {
        ++checker.semantic_comparisons;
        return value;
    };
    auto compare_work = [&](std::uint64_t actual,
                            std::uint64_t expected) noexcept {
        ++checker.work_comparisons;
        return actual == expected;
    };
    auto compare_root = [&](const Digest& actual,
                            const Digest& expected) noexcept {
        ++checker.root_comparisons;
        return digest_same(actual, expected);
    };
    const Digest zero{};
    auto digest_zero = [&](const Digest& value) noexcept {
        ++checker.unused_slot_checks;
        checker.unused_slot_bytes += value.size();
        return digest_same(value, zero);
    };
    auto slice_zero = [&](Slice value) noexcept {
        ++checker.unused_slot_checks;
        if (value.data == nullptr) return false;
        checker.unused_slot_bytes += value.size;
        std::uint8_t combined = 0U;
        for (std::size_t index = 0U; index < value.size; ++index)
            combined |= value.data[index];
        return combined == 0U;
    };
    auto work_zero = [&](const auto& fields) noexcept {
        ++checker.unused_slot_checks;
        checker.unused_slot_bytes += fields.size() * sizeof(std::uint64_t);
        std::uint64_t combined = 0U;
        for (std::uint64_t field : fields) combined |= field;
        return combined == 0U;
    };
    auto unused_scalar_zero = [&](auto value) noexcept {
        ++checker.unused_slot_checks;
        checker.unused_slot_bytes += sizeof(value);
        return value == 0;
    };

    std::array<ReplayProduct, ROLE_COUNT> replayed{};
    std::array<Digest, ROLE_COUNT> expected_product_roots{};
    Digest expected_product_set{};
    bool independent_semantics = true;
    if (cache_read) {
        independent_semantics = semantic(
            model_admission_exact == admission_exact)
            && independent_semantics;
        independent_semantics = semantic(
            producer_admission.first_failure_ordinal
                == independent.first_failure)
            && independent_semantics;
        if (model_admission_exact && admission_exact) {
            for (std::size_t index = 0U; index < PAYLOAD_COUNT; ++index)
                independent_semantics = compare_root(
                    model_payload_roots[index],
                    expected_payload_roots[index]) && independent_semantics;
            independent_semantics = compare_root(
                model_selected_root, expected_selected_root)
                && independent_semantics;
            independent_semantics = compare_root(
                model_bundle_root, expected_bundle_root)
                && independent_semantics;
        }
    }
    if (admission_exact) {
        independent_semantics = semantic(
            independent_selected.rows == ROW_COUNT) && independent_semantics;
        independent_semantics = semantic(
            independent_selected.columns == COLUMN_COUNT)
            && independent_semantics;
        bool products_exact = true;
        for (std::size_t role = 0U; role < ROLE_COUNT; ++role) {
            replay_product(replayed[role], role, independent_selected,
                expected_payload_roots[role + 1U], checker);
            expected_product_roots[role] = replayed[role].root;
            products_exact = products_exact && replayed[role].exact
                && replayed[role].no_underflow;
        }
        expected_product_set = root_product_set(
            expected_product_roots, checker);
        expected_producer_route = products_exact ? 0U : 3U;
    }

    const std::uint32_t expected_event_count =
        expected_producer_route == 1U ? 1U
        : expected_producer_route == 2U ? 2U : EVENT_COUNT;
    const std::uint32_t expected_product_count =
        expected_event_count == EVENT_COUNT ? ROLE_COUNT : 0U;
    const auto expected_read_work = producer_read.fields(cache_file, checker);
    const auto expected_admission_work = expected_producer_route == 1U
        ? std::array<std::uint64_t, ADMISSION_WORK_COUNT>{}
        : producer_admission.fields(checker);
    ProducerSealExpectation producer_seal;
    producer_seal.product_set_root_calls =
        expected_product_count == ROLE_COUNT ? 1U : 0U;
    producer_seal.product_set_root_bytes =
        expected_product_count == ROLE_COUNT ? 379U : 0U;
    producer_seal.event_count = expected_event_count;
    producer_seal.trace_bytes = 77U + expected_event_count * 41U;
    producer_seal.product_records = expected_product_count;
    const auto expected_seal_work = producer_seal.fields(checker);

    bool malformed = !artifact_read || !artifact_reader.ok()
        || artifact_reader.offset() != artifact_bytes.size()
        || !artifact.padding || artifact.version != 3U
        || artifact.total_bytes != ARTIFACT_SIZE
        || artifact.admission_exact > 1U;
    ++checker.route_checks;
    const bool unknown = artifact.route > 3U;
    if (!unknown) {
        const std::uint32_t route_events = artifact.route == 1U ? 1U
            : artifact.route == 2U ? 2U : EVENT_COUNT;
        const std::uint32_t route_products = route_events == EVENT_COUNT
            ? ROLE_COUNT : 0U;
        malformed = malformed || artifact.event_count != route_events
            || artifact.product_count != route_products
            || artifact.admission_exact
                != static_cast<std::uint8_t>(artifact.route == 0U
                    || artifact.route == 3U);
        for (std::size_t index = 0U; index < artifact.events.size(); ++index)
            if (index >= route_events)
                malformed = malformed || !digest_zero(artifact.events[index]);
        for (std::size_t role = 0U; role < artifact.products.size(); ++role) {
            const ArtifactProduct& product = artifact.products[role];
            malformed = malformed || !product.padding;
            if (role < route_products) {
                malformed = malformed || product.exact > 1U
                    || product.no_underflow > 1U || product.role != role
                    || product.value_count != ROW_COUNT
                    || product.value_bytes.data == nullptr;
            } else {
                malformed = malformed || !unused_scalar_zero(product.exact)
                    || !unused_scalar_zero(product.no_underflow)
                    || !unused_scalar_zero(product.role)
                    || !unused_scalar_zero(product.value_count)
                    || !work_zero(product.work)
                    || !digest_zero(product.root)
                    || !slice_zero(product.value_bytes);
                for (const Digest& root : product.vector_roots)
                    malformed = malformed || !digest_zero(root);
            }
        }
    }

    bool semantic_exact = independent_semantics;
    semantic_exact = semantic(artifact.route == expected_producer_route)
        && semantic_exact;
    semantic_exact = semantic(artifact.source_bytes == cache_file.source_size)
        && semantic_exact;
    semantic_exact = semantic(artifact.admission_exact
        == static_cast<std::uint8_t>(admission_exact)) && semantic_exact;
    if (admission_exact) {
        for (std::size_t role = 0U; role < ROLE_COUNT; ++role) {
            const ArtifactProduct& product = artifact.products[role];
            semantic_exact = semantic(product.role == role) && semantic_exact;
            semantic_exact = semantic(product.exact
                == static_cast<std::uint8_t>(replayed[role].exact))
                && semantic_exact;
            semantic_exact = semantic(product.no_underflow
                == static_cast<std::uint8_t>(replayed[role].no_underflow))
                && semantic_exact;
            for (std::size_t component = 0U; component < ROW_COUNT;
                 ++component) {
                const auto expected = canonical(replayed[role].value[component]);
                bool same = true;
                for (std::size_t byte = 0U; byte < expected.size(); ++byte)
                    same = same && product.value_bytes.data[
                        component * sizeof(Quad) + byte] == expected[byte];
                ++checker.component_comparisons;
                semantic_exact = same && semantic_exact;
            }
        }
    }

    bool work_exact = true;
    for (std::size_t index = 0U; index < READ_WORK_COUNT; ++index)
        work_exact = compare_work(
            artifact.read_work[index], expected_read_work[index]) && work_exact;
    for (std::size_t index = 0U; index < ADMISSION_WORK_COUNT; ++index)
        work_exact = compare_work(artifact.admission_work[index],
            expected_admission_work[index]) && work_exact;
    for (std::size_t index = 0U; index < SEAL_WORK_COUNT; ++index)
        work_exact = compare_work(
            artifact.seal_work[index], expected_seal_work[index]) && work_exact;
    for (std::size_t role = 0U; role < ROLE_COUNT; ++role)
        for (std::size_t index = 0U; index < PRODUCT_WORK_COUNT; ++index)
            work_exact = compare_work(artifact.products[role].work[index],
                role < expected_product_count ? PRODUCT_WORK[index] : 0U)
                && work_exact;
    work_exact = compare_work(artifact.product_set_hash_calls,
        expected_product_count == ROLE_COUNT ? 1U : 0U) && work_exact;
    work_exact = compare_work(artifact.product_set_hash_bytes,
        expected_product_count == ROLE_COUNT ? 379U : 0U) && work_exact;

    const Digest expected_read_work_root = root_work(
        READ_WORK_DOMAIN, expected_read_work, 271U, checker);
    Digest expected_admission_work_root{};
    if (expected_producer_route != 1U)
        expected_admission_work_root = root_work(ADMISSION_WORK_DOMAIN,
            expected_admission_work, 212U, checker);
    std::array<Digest, EVENT_COUNT> expected_events{};
    expected_events[0U] = root_event(
        0U, 0U, cache_file.root, expected_read_work_root, checker);
    if (expected_event_count >= 2U)
        expected_events[1U] = root_event(1U, 1U,
            admission_exact ? expected_selected_root : zero,
            expected_admission_work_root, checker);
    if (expected_event_count == EVENT_COUNT) {
        expected_events[2U] = root_event(2U, 2U,
            expected_bundle_root, expected_selected_root, checker);
        for (std::size_t role = 0U; role < ROLE_COUNT; ++role)
            expected_events[3U + role] = root_event(3U, 3U + role,
                expected_product_roots[role], expected_payload_roots[role + 1U],
                checker);
    }
    const Digest expected_trace = root_trace(expected_producer_route,
        expected_event_count, expected_events, checker);

    const Digest observed_body = root_artifact_body(checker);
    const Digest observed_self_result = root_result(
        static_cast<std::uint8_t>(artifact.route), artifact.source_bytes,
        observed_body, checker);
    const Digest expected_body = root_expected_artifact_body(
        expected_producer_route, cache_file.source_size, cache_file.root,
        admission_exact ? expected_selected_root : zero,
        admission_exact ? expected_bundle_root : zero,
        expected_product_count == ROLE_COUNT ? expected_product_set : zero,
        expected_event_count, expected_events, expected_trace,
        expected_read_work, admission_exact, expected_admission_work,
        expected_seal_work,
        admission_exact ? expected_payload_roots
                        : std::array<Digest, PAYLOAD_COUNT>{},
        expected_product_count, replayed, checker);
    const Digest expected_result = root_result(expected_producer_route,
        cache_file.source_size, expected_body, checker);

    bool seals_exact = true;
    seals_exact = compare_root(artifact.source_root, cache_file.root)
        && seals_exact;
    seals_exact = compare_root(artifact.selected_root,
        admission_exact ? expected_selected_root : zero) && seals_exact;
    seals_exact = compare_root(artifact.bundle_root,
        admission_exact ? expected_bundle_root : zero) && seals_exact;
    seals_exact = compare_root(artifact.product_set_root,
        expected_product_count == ROLE_COUNT ? expected_product_set : zero)
        && seals_exact;
    for (std::size_t index = 0U; index < PAYLOAD_COUNT; ++index)
        seals_exact = compare_root(artifact.payload_roots[index],
            admission_exact ? expected_payload_roots[index] : zero)
            && seals_exact;
    for (std::size_t index = 0U; index < expected_events.size(); ++index)
        seals_exact = compare_root(artifact.events[index],
            expected_events[index]) && seals_exact;
    seals_exact = compare_root(artifact.trace_root, expected_trace)
        && seals_exact;
    seals_exact = compare_root(artifact.result_root, observed_self_result)
        && seals_exact;
    seals_exact = compare_root(artifact.result_root, expected_result)
        && seals_exact;
    if (admission_exact) {
        for (std::size_t role = 0U; role < ROLE_COUNT; ++role) {
            for (std::size_t index = 0U; index < 4U; ++index)
                seals_exact = compare_root(
                    artifact.products[role].vector_roots[index],
                    replayed[role].roots[index]) && seals_exact;
            seals_exact = compare_root(artifact.products[role].root,
                replayed[role].root) && seals_exact;
        }
    }

    ++checker.route_checks;
    CheckRoute route = CheckRoute::CandidateAccepted;
    if (malformed) route = CheckRoute::Malformed;
    else if (unknown) route = CheckRoute::Unknown;
    else if (!semantic_exact) route = CheckRoute::Semantic;
    else if (!work_exact) route = CheckRoute::Work;
    else if (!seals_exact) route = CheckRoute::Seal;
    else if (expected_producer_route == 1U)
        route = CheckRoute::ReadRejectedVerified;
    else if (expected_producer_route == 2U)
        route = CheckRoute::AdmissionRejectedVerified;
    else if (expected_producer_route == 3U)
        route = CheckRoute::ProductRejectedVerified;

    constexpr std::uint64_t AUDIT_ROOT_BYTES =
        9U + sizeof(CHECKER_DOMAIN) - 1U + 4U * 10U
        + 9U + CHECKER_WORK_COUNT * 8U
        + 17U + 41U + 17U + 41U + 41U + 41U + 17U;
    checker.audit_root_bytes = AUDIT_ROOT_BYTES;
    ++checker.hash_calls;
    checker.hash_bytes += AUDIT_ROOT_BYTES;
    const Digest audit = checker_root(route, semantic_exact, work_exact,
        seals_exact, cache_file, artifact_file, artifact.result_root,
        expected_result, checker);

    static_assert(AUDIT_OUTPUT_SIZE == 420U);
    static_assert(AUDIT_CHECKER_ROOT_OFFSET + 32U == AUDIT_OUTPUT_SIZE);
    std::array<std::uint8_t, AUDIT_OUTPUT_SIZE> output{};
    AuditWriter writer(output);
    writer.raw(AUDIT_MAGIC);
    writer.u32(3U);
    writer.u64(AUDIT_OUTPUT_SIZE);
    writer.u32(static_cast<std::uint32_t>(route));
    writer.byte(semantic_exact ? 1U : 0U);
    writer.byte(work_exact ? 1U : 0U);
    writer.byte(seals_exact ? 1U : 0U);
    writer.byte(0U);
    writer.u64(checker.component_comparisons);
    writer.work(checker.fields());
    writer.u64(cache_file.source_size);
    writer.raw(cache_file.root);
    writer.u64(artifact_file.source_size);
    writer.raw(artifact_file.root);
    writer.raw(artifact.result_root);
    writer.raw(expected_result);
    writer.raw(audit);
    if (!writer.complete() || !write_audit(argv[3], output)) return 2;
    return static_cast<std::uint8_t>(route) <= 3U ? 0 : 1;
}
