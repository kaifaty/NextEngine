#include <quadmath.h>

#include "../r63zm/fixed_sha256.hpp"

#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstring>

namespace {

__extension__ using Q = __float128;

constexpr std::size_t CACHE_BYTES = 1033625U;
constexpr std::size_t ARTIFACT_BYTES = 12916U;
constexpr std::size_t AUDIT_BYTES = 420U;
constexpr std::size_t N = 102U;
constexpr std::size_t FACTOR_N = N * N;
constexpr std::size_t ROLE2_VALUE_OFFSET = 5436U;
constexpr const char* CACHE_SHA256 =
    "23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84";
constexpr const char* ARTIFACT_SHA256 =
    "ac6946e872799baef366d8a6648e7bb5cd70c6f2acc326747fdf153c471f0b87";
constexpr const char* AUDIT_SHA256 =
    "fd4bcf0094e9f6ab8c64080c3a22566e3a23d2544e187641075350519a281f80";
constexpr const char* ARTIFACT_RESULT_SHA256 =
    "7eaedbd3775bac423c4e5dfadc25c18f0ffe2ffe181e5080675ff0a457a37da3";
constexpr const char* PRODUCT_SET_SHA256 =
    "6ac66c134a3b8eea010de7e66bab209aa995071089eb8b96bd14804e2722a06b";
constexpr const char* ROLE2_VALUE_SHA256 =
    "8b6373db6132ee119eff020cb53c01c7287d3d49e70a2d6ad7c387b7dd37dcce";
constexpr const char* CHECKER_ROOT_SHA256 =
    "b4a7969c6676b30e87fff470f95e9c098f8951b2a477314213c359c6164c80b4";
constexpr std::size_t WORK_COUNTERS = 36U;
constexpr std::size_t EVENTS = 7U;
constexpr std::size_t RECEIPT_BYTES = 1120U;
constexpr std::size_t RECEIPT_VERSION_OFFSET = 8U;
constexpr std::size_t RECEIPT_SIZE_OFFSET = 12U;
constexpr std::size_t RECEIPT_ROUTE_OFFSET = 20U;
constexpr std::size_t RECEIPT_FLAGS_OFFSET = 24U;
constexpr std::size_t RECEIPT_CACHE_SIZE_OFFSET = 32U;
constexpr std::size_t RECEIPT_CACHE_ROOT_OFFSET = 40U;
constexpr std::size_t RECEIPT_ARTIFACT_SIZE_OFFSET = 72U;
constexpr std::size_t RECEIPT_ARTIFACT_ROOT_OFFSET = 80U;
constexpr std::size_t RECEIPT_AUDIT_SIZE_OFFSET = 112U;
constexpr std::size_t RECEIPT_AUDIT_ROOT_OFFSET = 120U;
constexpr std::size_t RECEIPT_BUNDLE_ROOT_OFFSET = 152U;
constexpr std::size_t RECEIPT_FACTOR_ROOT_OFFSET = 184U;
constexpr std::size_t RECEIPT_PERMUTATION_ROOT_OFFSET = 216U;
constexpr std::size_t RECEIPT_INVERSE_ROOT_OFFSET = 248U;
constexpr std::size_t RECEIPT_RHS_ROOT_OFFSET = 280U;
constexpr std::size_t RECEIPT_BASELINE0_ROOT_OFFSET = 312U;
constexpr std::size_t RECEIPT_X0_ROOT_OFFSET = 344U;
constexpr std::size_t RECEIPT_HX0_ROOT_OFFSET = 376U;
constexpr std::size_t RECEIPT_RESIDUAL_ROOT_OFFSET = 408U;
constexpr std::size_t RECEIPT_RESIDUAL_BOUND_ROOT_OFFSET = 440U;
constexpr std::size_t RECEIPT_Z0_ROOT_OFFSET = 472U;
constexpr std::size_t RECEIPT_RHO_ROOT_OFFSET = 504U;
constexpr std::size_t RECEIPT_WORK_OFFSET = 536U;
constexpr std::size_t RECEIPT_EVENT_COUNT_OFFSET = 824U;
constexpr std::size_t RECEIPT_EVENTS_OFFSET = 832U;
constexpr std::size_t RECEIPT_TRACE_ROOT_OFFSET = 1056U;
constexpr std::size_t RECEIPT_RESULT_ROOT_OFFSET = 1088U;
static_assert(RECEIPT_WORK_OFFSET + WORK_COUNTERS * 8U
    == RECEIPT_EVENT_COUNT_OFFSET);
static_assert(RECEIPT_EVENT_COUNT_OFFSET + 8U == RECEIPT_EVENTS_OFFSET);
static_assert(RECEIPT_EVENTS_OFFSET + EVENTS * 32U
    == RECEIPT_TRACE_ROOT_OFFSET);
static_assert(RECEIPT_TRACE_ROOT_OFFSET + 32U == RECEIPT_RESULT_ROOT_OFFSET);
static_assert(RECEIPT_RESULT_ROOT_OFFSET + 32U == RECEIPT_BYTES);

struct ByteSpan {
    const std::uint8_t* data = nullptr;
    std::size_t size = 0U;
};

struct VectorSpan {
    ByteSpan bytes;
    std::uint64_t count = 0U;
};

class LittleReader final {
public:
    LittleReader(const std::uint8_t* data, std::size_t size) noexcept
        : begin_(data), cursor_(data), end_(data + size) {}

    ByteSpan bytes(std::size_t count) noexcept {
        if (!valid_ || count > static_cast<std::size_t>(end_ - cursor_)) {
            valid_ = false;
            return {};
        }
        const ByteSpan result{cursor_, count};
        cursor_ += count;
        ++reads_;
        bytes_ += count;
        return result;
    }

    std::uint8_t u8() noexcept {
        const ByteSpan value = bytes(1U);
        return value.data == nullptr ? 0U : value.data[0U];
    }

    std::uint32_t u32() noexcept {
        const ByteSpan value = bytes(4U);
        if (value.data == nullptr) return 0U;
        return static_cast<std::uint32_t>(value.data[0U])
            | (static_cast<std::uint32_t>(value.data[1U]) << 8U)
            | (static_cast<std::uint32_t>(value.data[2U]) << 16U)
            | (static_cast<std::uint32_t>(value.data[3U]) << 24U);
    }

    std::uint64_t u64() noexcept {
        const ByteSpan value = bytes(8U);
        if (value.data == nullptr) return 0U;
        std::uint64_t result = 0U;
        for (std::size_t index = 0U; index < 8U; ++index)
            result |= static_cast<std::uint64_t>(value.data[index])
                << (8U * index);
        return result;
    }

    bool boolean() noexcept {
        const std::uint8_t value = u8();
        require(value <= 1U);
        return value != 0U;
    }

    ByteSpan text() noexcept {
        const std::uint64_t count = u64();
        require(count <= CACHE_BYTES);
        return valid_ ? bytes(static_cast<std::size_t>(count)) : ByteSpan{};
    }

    VectorSpan vector(std::size_t width) noexcept {
        const std::uint64_t count = u64();
        require(width != 0U && count <= CACHE_BYTES / width);
        if (!valid_ || count > static_cast<std::uint64_t>(
                static_cast<std::size_t>(end_ - cursor_) / width))
            return fail_vector();
        return {bytes(static_cast<std::size_t>(count) * width), count};
    }

    void require(bool condition) noexcept {
        ++predicates_;
        valid_ = valid_ && condition;
    }

    bool complete() const noexcept { return valid_ && cursor_ == end_; }
    std::uint64_t reads() const noexcept { return reads_; }
    std::uint64_t byte_count() const noexcept { return bytes_; }
    std::uint64_t predicates() const noexcept { return predicates_; }

private:
    VectorSpan fail_vector() noexcept {
        valid_ = false;
        return {};
    }

    const std::uint8_t* begin_;
    const std::uint8_t* cursor_;
    const std::uint8_t* end_;
    bool valid_ = true;
    std::uint64_t reads_ = 0U;
    std::uint64_t bytes_ = 0U;
    std::uint64_t predicates_ = 0U;
};

void skip_certificate(LittleReader& reader) noexcept {
    static_cast<void>(reader.boolean());
    static_cast<void>(reader.boolean());
    static_cast<void>(reader.bytes(3U * 8U));
    static_cast<void>(reader.bytes(8U));
    for (std::size_t index = 0U; index < 3U; ++index)
        static_cast<void>(reader.text());
}

void skip_certificates(LittleReader& reader) noexcept {
    const std::uint64_t count = reader.u64();
    reader.require(count <= 64U);
    for (std::uint64_t index = 0U; index < count; ++index)
        skip_certificate(reader);
}

void skip_profile(LittleReader& reader) noexcept {
    for (std::size_t index = 0U; index < 3U; ++index)
        static_cast<void>(reader.boolean());
    static_cast<void>(reader.bytes(6U * 8U));
    for (std::size_t index = 0U; index < 4U; ++index)
        static_cast<void>(reader.vector(8U));
    static_cast<void>(reader.bytes(3U * 8U));
    for (std::size_t index = 0U; index < 4U; ++index)
        static_cast<void>(reader.text());
}

struct CacheSelection {
    bool exact = false;
    std::uint64_t dimension = 0U;
    std::uint64_t columns = 0U;
    VectorSpan baseline0;
    VectorSpan factor;
    VectorSpan permutation;
    ByteSpan inverse;
    VectorSpan original_rhs;
    std::uint64_t parse_reads = 0U;
    std::uint64_t parse_bytes = 0U;
    std::uint64_t parse_predicates = 0U;
};

CacheSelection parse_cache(
    const std::array<std::uint8_t, CACHE_BYTES>& cache) noexcept {
    constexpr std::array<std::uint8_t, 8U> magic{
        'N','E','F','P','R','B','1',0U};
    LittleReader reader(cache.data(), cache.size());
    CacheSelection result;
    const ByteSpan observed_magic = reader.bytes(magic.size());
    reader.require(observed_magic.data != nullptr
        && std::memcmp(observed_magic.data, magic.data(), magic.size()) == 0);
    reader.require(reader.u32() == 1U);
    reader.require(reader.u32() == 0x01020304U);
    reader.require(reader.u32() == 8U);
    reader.require(reader.u32() == 16U);
    static_cast<void>(reader.boolean());
    static_cast<void>(reader.text());
    result.dimension = reader.u64();
    result.columns = reader.u64();
    for (std::size_t index = 0U; index < 9U; ++index)
        static_cast<void>(reader.text());
    reader.require(reader.u64() == 3U);
    result.baseline0 = reader.vector(16U);
    static_cast<void>(reader.vector(16U));
    static_cast<void>(reader.vector(16U));
    skip_certificates(reader);
    static_cast<void>(reader.vector(16U));
    static_cast<void>(reader.bytes(16U));
    static_cast<void>(reader.text());
    result.factor = reader.vector(8U);
    const std::uint64_t permutation_count = reader.u64();
    reader.require(permutation_count <= 4096U);
    result.permutation = {
        reader.bytes(static_cast<std::size_t>(permutation_count) * 8U),
        permutation_count};
    result.inverse = reader.bytes(16U);
    for (std::size_t index = 0U; index < 3U; ++index)
        static_cast<void>(reader.text());
    result.original_rhs = reader.vector(16U);
    static_cast<void>(reader.vector(16U));
    static_cast<void>(reader.bytes(16U));
    static_cast<void>(reader.text());
    static_cast<void>(reader.text());
    static_cast<void>(reader.vector(8U));
    for (std::size_t index = 0U; index < 3U; ++index)
        static_cast<void>(reader.text());
    skip_profile(reader);
    static_cast<void>(reader.text());
    static_cast<void>(reader.text());
    reader.require(reader.u64() == 3U);
    static_cast<void>(reader.vector(16U));
    static_cast<void>(reader.vector(16U));
    static_cast<void>(reader.vector(16U));
    skip_certificates(reader);
    for (std::size_t index = 0U; index < 3U; ++index)
        static_cast<void>(reader.text());
    result.exact = reader.complete();
    result.parse_reads = reader.reads();
    result.parse_bytes = reader.byte_count();
    result.parse_predicates = reader.predicates();
    return result;
}

struct Accumulator {
    Q sum = static_cast<Q>(0.0);
    Q correction = static_cast<Q>(0.0);

    void add(Q value) noexcept {
        const Q next = sum + value;
        correction += fabsq(sum) >= fabsq(value)
            ? (sum - next) + value
            : (value - next) + sum;
        sum = next;
    }

    Q value() const noexcept { return sum + correction; }
};

bool normal_or_zero(Q value) noexcept {
    return finiteq(value) != 0
        && (value == static_cast<Q>(0.0)
            || fabsq(value) >= ldexpq(static_cast<Q>(1.0), -16382));
}

Q up_add(Q left, Q right) noexcept {
    if (left == static_cast<Q>(0.0) && right == static_cast<Q>(0.0))
        return static_cast<Q>(0.0);
    return nextafterq(left + right, HUGE_VALQ);
}

Q up_multiply(Q left, Q right) noexcept {
    if (left == static_cast<Q>(0.0) || right == static_cast<Q>(0.0))
        return static_cast<Q>(0.0);
    return nextafterq(left * right, HUGE_VALQ);
}

Q up_divide(Q numerator, Q denominator) noexcept {
    if (numerator == static_cast<Q>(0.0)) return static_cast<Q>(0.0);
    return nextafterq(numerator / denominator, HUGE_VALQ);
}

struct Pair {
    bool exact = false;
    bool no_underflow = false;
    Q high = static_cast<Q>(0.0);
    Q low = static_cast<Q>(0.0);
};

Pair two_sum(Q left, Q right) noexcept {
    Pair result;
    result.high = left + right;
    const Q virtual_right = result.high - left;
    result.low = (left - (result.high - virtual_right))
        + (right - virtual_right);
    result.no_underflow = normal_or_zero(left) && normal_or_zero(right)
        && normal_or_zero(result.high) && normal_or_zero(virtual_right)
        && normal_or_zero(result.low);
    result.exact = finiteq(result.high) != 0 && finiteq(result.low) != 0;
    return result;
}

Pair two_product(Q left, Q right) noexcept {
    Pair result;
    result.high = left * right;
    result.low = fmaq(left, right, -result.high);
    const bool product_not_lost = left == static_cast<Q>(0.0)
        || right == static_cast<Q>(0.0)
        || result.high != static_cast<Q>(0.0)
        || result.low != static_cast<Q>(0.0);
    result.no_underflow = product_not_lost && normal_or_zero(left)
        && normal_or_zero(right) && normal_or_zero(result.high)
        && normal_or_zero(result.low);
    result.exact = finiteq(result.high) != 0 && finiteq(result.low) != 0;
    return result;
}

struct Dot2 {
    bool exact = false;
    bool no_underflow = false;
    Q value = static_cast<Q>(0.0);
    Q bound = static_cast<Q>(0.0);
    Q absolute_products = static_cast<Q>(0.0);
    std::size_t products = 0U;
    std::size_t sums = 0U;
};

Dot2 dot2(const Q* left, const Q* right, std::size_t count) noexcept {
    Dot2 result;
    if (count == 0U) return result;
    const Pair first = two_product(left[0U], right[0U]);
    Q primary = first.high;
    Q correction = first.low;
    bool exact = first.exact;
    bool no_underflow = first.no_underflow;
    result.absolute_products = up_multiply(fabsq(left[0U]), fabsq(right[0U]));
    result.products = 1U;
    for (std::size_t index = 1U; index < count; ++index) {
        const Pair product = two_product(left[index], right[index]);
        const Pair sum = two_sum(primary, product.high);
        const Q local = sum.low + product.low;
        correction = correction + local;
        primary = sum.high;
        exact = exact && product.exact && sum.exact;
        no_underflow = no_underflow && product.no_underflow
            && sum.no_underflow && normal_or_zero(local)
            && normal_or_zero(correction);
        result.absolute_products = up_add(result.absolute_products,
            up_multiply(fabsq(left[index]), fabsq(right[index])));
        ++result.products;
        ++result.sums;
    }
    result.value = primary + correction;
    no_underflow = no_underflow && normal_or_zero(result.value);
    const Q one = static_cast<Q>(1.0);
    const Q unit = ldexpq(one, -112);
    const Q count_unit = static_cast<Q>(count) * unit;
    const Q gamma = up_divide(count_unit, one - count_unit);
    const Q numerator = up_add(up_multiply(unit, fabsq(result.value)),
        up_multiply(up_multiply(gamma, gamma), result.absolute_products));
    result.bound = up_divide(numerator, one - unit);
    result.no_underflow = no_underflow;
    result.exact = exact && finiteq(result.value) != 0
        && finiteq(result.bound) != 0 && result.bound >= static_cast<Q>(0.0);
    return result;
}

struct Solve {
    bool exact = false;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::array<Q, N> scaled{};
    std::array<Q, N> intermediate{};
    std::array<Q, N> permuted{};
    std::array<Q, N> solution{};
};

Solve solve(const std::array<Q, FACTOR_N>& upper,
    const std::array<std::uint64_t, N>& permutation,
    const std::array<Q, N>& rhs, Q inverse) noexcept {
    Solve result;
    bool finite = finiteq(inverse) != 0 && inverse > static_cast<Q>(0.0);
    for (std::size_t row = 0U; row < N; ++row) {
        finite = finite && permutation[row] < N;
        if (!finite) break;
        result.scaled[row] = rhs[permutation[row]] * inverse;
        finite = finite && finiteq(result.scaled[row]) != 0;
        Accumulator dot;
        for (std::size_t column = 0U; column < row; ++column) {
            dot.add(upper[column * N + row] * result.intermediate[column]);
            ++result.forward_terms;
        }
        const Q diagonal = upper[row * N + row];
        finite = finite && finiteq(diagonal) != 0
            && diagonal != static_cast<Q>(0.0);
        if (!finite) break;
        result.intermediate[row] = (result.scaled[row] - dot.value()) / diagonal;
        finite = finite && finiteq(result.intermediate[row]) != 0;
        ++result.divisions;
    }
    if (finite) {
        for (std::size_t reverse = N; reverse-- > 0U;) {
            Accumulator dot;
            for (std::size_t column = reverse + 1U; column < N; ++column) {
                dot.add(upper[reverse * N + column] * result.permuted[column]);
                ++result.backward_terms;
            }
            const Q diagonal = upper[reverse * N + reverse];
            finite = finite && finiteq(diagonal) != 0
                && diagonal != static_cast<Q>(0.0);
            if (!finite) break;
            result.permuted[reverse] =
                (result.intermediate[reverse] - dot.value()) / diagonal;
            finite = finite && finiteq(result.permuted[reverse]) != 0;
            ++result.divisions;
        }
    }
    for (std::size_t index = 0U; index < N; ++index)
        if (permutation[index] < N)
            result.solution[permutation[index]] = result.permuted[index];
    result.exact = finite && result.forward_terms == 5151U
        && result.backward_terms == 5151U && result.divisions == 204U;
    return result;
}

template <std::size_t Size>
bool read_file(const char* path, std::array<std::uint8_t, Size>& bytes) noexcept {
    std::FILE* file = std::fopen(path, "rb");
    if (file == nullptr) return false;
    const std::size_t count = std::fread(bytes.data(), 1U, bytes.size(), file);
    const int trailing = std::fgetc(file);
    const int close = std::fclose(file);
    return count == bytes.size() && trailing == EOF && close == 0;
}

std::uint8_t hex_nibble(char value) noexcept {
    if (value >= '0' && value <= '9')
        return static_cast<std::uint8_t>(value - '0');
    if (value >= 'a' && value <= 'f')
        return static_cast<std::uint8_t>(value - 'a' + 10);
    return 0xffU;
}

bool digest_equal_hex(const r63zm::Digest& digest, const char* expected) noexcept {
    for (std::size_t index = 0U; index < digest.size(); ++index) {
        const std::uint8_t high = hex_nibble(expected[2U * index]);
        const std::uint8_t low = hex_nibble(expected[2U * index + 1U]);
        if (high > 0x0fU || low > 0x0fU
            || digest[index] != static_cast<std::uint8_t>((high << 4U) | low))
            return false;
    }
    return expected[64U] == '\0';
}

template <std::size_t Size>
bool hash_matches(const std::array<std::uint8_t, Size>& bytes,
    const char* expected) noexcept {
    return digest_equal_hex(r63zm::sha256(bytes), expected);
}

bool bytes_equal_hex(const std::uint8_t* bytes, const char* expected,
    std::size_t count) noexcept {
    for (std::size_t index = 0U; index < count; ++index) {
        const std::uint8_t high = hex_nibble(expected[2U * index]);
        const std::uint8_t low = hex_nibble(expected[2U * index + 1U]);
        if (high > 0x0fU || low > 0x0fU
            || bytes[index] != static_cast<std::uint8_t>((high << 4U) | low))
            return false;
    }
    return expected[2U * count] == '\0';
}

std::uint32_t be32(const std::uint8_t* bytes) noexcept {
    return (static_cast<std::uint32_t>(bytes[0U]) << 24U)
        | (static_cast<std::uint32_t>(bytes[1U]) << 16U)
        | (static_cast<std::uint32_t>(bytes[2U]) << 8U)
        | static_cast<std::uint32_t>(bytes[3U]);
}

std::uint64_t be64(const std::uint8_t* bytes) noexcept {
    std::uint64_t value = 0U;
    for (std::size_t index = 0U; index < 8U; ++index)
        value = (value << 8U) | bytes[index];
    return value;
}

bool parent_artifact_valid(
    const std::array<std::uint8_t, ARTIFACT_BYTES>& bytes) noexcept {
    constexpr std::array<std::uint8_t, 8U> magic{
        'N','E','R','6','3','Z','M','1'};
    constexpr std::size_t role2 = 1236U + 2U * 1944U;
    return hash_matches(bytes, ARTIFACT_SHA256)
        && std::memcmp(bytes.data(), magic.data(), magic.size()) == 0
        && be32(bytes.data() + 8U) == 3U
        && be64(bytes.data() + 12U) == ARTIFACT_BYTES
        && be32(bytes.data() + 60U) == 0U
        && bytes_equal_hex(bytes.data() + 128U, PRODUCT_SET_SHA256, 32U)
        && bytes_equal_hex(bytes.data() + 484U, ARTIFACT_RESULT_SHA256, 32U)
        && be32(bytes.data() + 1228U) == 6U
        && bytes[role2] == 1U && bytes[role2 + 1U] == 1U
        && bytes[role2 + 2U] == 2U
        && be64(bytes.data() + role2 + 304U) == N
        && bytes_equal_hex(bytes.data() + role2 + 144U + 64U,
            ROLE2_VALUE_SHA256, 32U);
}

bool parent_audit_valid(
    const std::array<std::uint8_t, AUDIT_BYTES>& bytes) noexcept {
    constexpr std::array<std::uint8_t, 8U> magic{
        'N','E','R','6','3','Q','C','1'};
    return hash_matches(bytes, AUDIT_SHA256)
        && std::memcmp(bytes.data(), magic.data(), magic.size()) == 0
        && be32(bytes.data() + 8U) == 3U
        && be64(bytes.data() + 12U) == AUDIT_BYTES
        && be32(bytes.data() + 20U) == 0U
        && bytes[24U] == 1U && bytes[25U] == 1U && bytes[26U] == 1U
        && bytes_equal_hex(bytes.data() + 388U, CHECKER_ROOT_SHA256, 32U);
}

void hash_be64(r63zm::Sha256& hash, std::uint64_t value) noexcept {
    std::array<std::uint8_t, 8U> bytes{};
    for (std::size_t index = 0U; index < bytes.size(); ++index)
        bytes[index] = static_cast<std::uint8_t>(
            value >> (56U - 8U * index));
    hash.update(bytes);
}

void hash_tlv_header(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t length) noexcept {
    hash.update_byte(tag);
    hash_be64(hash, length);
}

void hash_tlv(r63zm::Sha256& hash, std::uint8_t tag,
    const std::uint8_t* data, std::size_t size) noexcept {
    hash_tlv_header(hash, tag, size);
    hash.update(std::span<const std::uint8_t>(data, size));
}

void hash_tlv_text(r63zm::Sha256& hash, std::uint8_t tag,
    const char* text) noexcept {
    hash_tlv(hash, tag, reinterpret_cast<const std::uint8_t*>(text),
        std::strlen(text));
}

void hash_tlv_u64(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t value) noexcept {
    hash_tlv_header(hash, tag, 8U);
    hash_be64(hash, value);
}

void hash_tlv_digest(r63zm::Sha256& hash, std::uint8_t tag,
    const r63zm::Digest& value) noexcept {
    hash_tlv(hash, tag, value.data(), value.size());
}

std::array<std::uint8_t, 16U> canonical_q(Q value) noexcept {
    std::array<std::uint8_t, 16U> native{};
    std::array<std::uint8_t, 16U> canonical{};
    std::memcpy(native.data(), &value, native.size());
    for (std::size_t index = 0U; index < canonical.size(); ++index)
        canonical[index] = native[canonical.size() - 1U - index];
    return canonical;
}

template <std::size_t Count>
r63zm::Digest q_vector_root(const std::array<Q, Count>& values) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U,
        "nextengine.nonlocal.r63zn.binary128-vector.v1");
    hash_tlv_u64(hash, 2U, Count);
    hash_tlv_header(hash, 3U, Count * 16U);
    for (Q value : values) hash.update(canonical_q(value));
    return hash.finish();
}

r63zm::Digest binary64_factor_root(ByteSpan values,
    std::uint64_t count) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U,
        "nextengine.nonlocal.r63zn.binary64-factor.v1");
    hash_tlv_u64(hash, 2U, N);
    hash_tlv_u64(hash, 3U, count);
    hash_tlv_header(hash, 4U, count * 8U);
    for (std::size_t index = 0U; index < count; ++index)
        for (std::size_t byte = 0U; byte < 8U; ++byte)
            hash.update_byte(values.data[index * 8U + 7U - byte]);
    return hash.finish();
}

r63zm::Digest permutation_root(
    const std::array<std::uint64_t, N>& values) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U,
        "nextengine.nonlocal.r63zn.permutation.v1");
    hash_tlv_u64(hash, 2U, values.size());
    hash_tlv_header(hash, 3U, values.size() * 8U);
    for (std::uint64_t value : values) hash_be64(hash, value);
    return hash.finish();
}

r63zm::Digest q_scalar_root(const char* domain, Q value) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, domain);
    const std::array<std::uint8_t, 16U> bytes = canonical_q(value);
    hash_tlv(hash, 2U, bytes.data(), bytes.size());
    return hash.finish();
}

r63zm::Digest rho_root(const Dot2& value) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, "nextengine.nonlocal.r63zn.rho0.v1");
    hash_tlv_u64(hash, 2U, value.exact ? 1U : 0U);
    hash_tlv_u64(hash, 3U, value.no_underflow ? 1U : 0U);
    const std::array<std::uint8_t, 16U> scalar = canonical_q(value.value);
    const std::array<std::uint8_t, 16U> bound = canonical_q(value.bound);
    const std::array<std::uint8_t, 16U> absolute =
        canonical_q(value.absolute_products);
    hash_tlv(hash, 4U, scalar.data(), scalar.size());
    hash_tlv(hash, 5U, bound.data(), bound.size());
    hash_tlv(hash, 6U, absolute.data(), absolute.size());
    return hash.finish();
}

template <std::size_t Count>
r63zm::Digest digest_set_root(const char* domain,
    const std::array<r63zm::Digest, Count>& values) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, domain);
    hash_tlv_u64(hash, 2U, values.size());
    hash_tlv_header(hash, 3U, values.size() * 32U);
    for (const r63zm::Digest& value : values) hash.update(value);
    return hash.finish();
}

r63zm::Digest work_root(
    const std::array<std::uint64_t, WORK_COUNTERS>& values) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, "nextengine.nonlocal.r63zn.candidate-work.v1");
    hash_tlv_u64(hash, 2U, values.size());
    hash_tlv_header(hash, 3U, values.size() * 8U);
    for (std::uint64_t value : values) hash_be64(hash, value);
    return hash.finish();
}

r63zm::Digest event_root(std::uint64_t ordinal,
    const r63zm::Digest& left, const r63zm::Digest& right) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, "nextengine.nonlocal.r63zn.event.v1");
    hash_tlv_u64(hash, 2U, ordinal);
    hash_tlv_digest(hash, 3U, left);
    hash_tlv_digest(hash, 4U, right);
    return hash.finish();
}

r63zm::Digest trace_root(std::uint64_t route,
    const std::array<r63zm::Digest, EVENTS>& events) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, "nextengine.nonlocal.r63zn.trace.v1");
    hash_tlv_u64(hash, 2U, route);
    hash_tlv_u64(hash, 3U, events.size());
    hash_tlv_header(hash, 4U, events.size() * 32U);
    for (const r63zm::Digest& event : events) hash.update(event);
    return hash.finish();
}

r63zm::Digest result_root(std::uint64_t route,
    const std::array<r63zm::Digest, 3U>& inputs,
    const r63zm::Digest& trace, const r63zm::Digest& work) noexcept {
    r63zm::Sha256 hash;
    hash_tlv_text(hash, 1U, "nextengine.nonlocal.r63zn.result.v1");
    hash_tlv_u64(hash, 2U, 2U);
    hash_tlv_u64(hash, 3U, route);
    hash_tlv_header(hash, 4U, inputs.size() * 32U);
    for (const r63zm::Digest& input : inputs) hash.update(input);
    hash_tlv_digest(hash, 5U, trace);
    hash_tlv_digest(hash, 6U, work);
    hash_tlv_u64(hash, 7U, RECEIPT_BYTES);
    return hash.finish();
}

void put_be32(std::uint8_t* target, std::uint32_t value) noexcept {
    for (std::size_t index = 0U; index < 4U; ++index)
        target[index] = static_cast<std::uint8_t>(
            value >> (24U - 8U * index));
}

void put_be64(std::uint8_t* target, std::uint64_t value) noexcept {
    for (std::size_t index = 0U; index < 8U; ++index)
        target[index] = static_cast<std::uint8_t>(
            value >> (56U - 8U * index));
}

void put_digest(std::uint8_t* target, const r63zm::Digest& value) noexcept {
    std::memcpy(target, value.data(), value.size());
}

template <std::size_t Size>
bool write_file(const char* path,
    const std::array<std::uint8_t, Size>& bytes) noexcept {
    std::FILE* file = std::fopen(path, "wb");
    if (file == nullptr) return false;
    const std::size_t count = std::fwrite(bytes.data(), 1U, bytes.size(), file);
    const int close = std::fclose(file);
    return count == bytes.size() && close == 0;
}

Q little_q(ByteSpan bytes, std::size_t index) noexcept {
    Q value{};
    if (bytes.data != nullptr && (index + 1U) * sizeof(value) <= bytes.size)
        std::memcpy(&value, bytes.data + index * sizeof(value), sizeof(value));
    return value;
}

double little_double(ByteSpan bytes, std::size_t index) noexcept {
    double value = 0.0;
    if (bytes.data != nullptr && (index + 1U) * sizeof(value) <= bytes.size)
        std::memcpy(&value, bytes.data + index * sizeof(value), sizeof(value));
    return value;
}

std::uint64_t little_u64(ByteSpan bytes, std::size_t index) noexcept {
    std::uint64_t value = 0U;
    if (bytes.data != nullptr && (index + 1U) * sizeof(value) <= bytes.size)
        std::memcpy(&value, bytes.data + index * sizeof(value), sizeof(value));
    return value;
}

Q artifact_q(const std::array<std::uint8_t, ARTIFACT_BYTES>& bytes,
    std::size_t offset) noexcept {
    std::array<std::uint8_t, 16U> reversed{};
    for (std::size_t index = 0U; index < reversed.size(); ++index)
        reversed[index] = bytes[offset + reversed.size() - 1U - index];
    Q value{};
    std::memcpy(&value, reversed.data(), sizeof(value));
    return value;
}

} // namespace

int main(int argc, char** argv) {
    if (argc != 5) return 64;
    std::array<std::uint8_t, CACHE_BYTES> cache{};
    std::array<std::uint8_t, ARTIFACT_BYTES> artifact{};
    std::array<std::uint8_t, AUDIT_BYTES> audit{};
    if (!read_file(argv[1], cache) || !read_file(argv[2], artifact)
        || !read_file(argv[3], audit)) return 65;
    if (!hash_matches(cache, CACHE_SHA256)
        || !parent_artifact_valid(artifact) || !parent_audit_valid(audit))
        return 66;
    const CacheSelection selection = parse_cache(cache);
    if (!selection.exact || selection.dimension != N
        || selection.columns != 315U || selection.baseline0.count != N
        || selection.factor.count != FACTOR_N
        || selection.permutation.count != N
        || selection.inverse.size != sizeof(Q)
        || selection.original_rhs.count != N)
        return 67;

    std::array<Q, FACTOR_N> upper{};
    for (std::size_t index = 0U; index < upper.size(); ++index)
        upper[index] = static_cast<Q>(little_double(selection.factor.bytes,
            index));
    std::array<std::uint64_t, N> permutation{};
    std::array<bool, N> seen{};
    bool permutation_exact = true;
    for (std::size_t index = 0U; index < N; ++index) {
        permutation[index] = little_u64(selection.permutation.bytes, index);
        permutation_exact = permutation_exact && permutation[index] < N
            && !seen[permutation[index]];
        if (permutation[index] < N) seen[permutation[index]] = true;
    }
    std::array<Q, N> rhs{};
    std::array<Q, N> baseline0{};
    std::array<Q, N> hx0{};
    for (std::size_t index = 0U; index < N; ++index) {
        rhs[index] = little_q(selection.original_rhs.bytes, index);
        baseline0[index] = little_q(selection.baseline0.bytes, index);
        hx0[index] = artifact_q(artifact, ROLE2_VALUE_OFFSET + 16U * index);
    }
    const Q inverse = little_q(selection.inverse, 0U);
    const Solve start = solve(upper, permutation, rhs, inverse);
    std::size_t x0_matches = 0U;
    for (std::size_t index = 0U; index < N; ++index)
        x0_matches += std::memcmp(&start.solution[index], &baseline0[index],
            sizeof(Q)) == 0 ? 1U : 0U;

    std::array<Q, N> residual{};
    std::array<Q, N> residual_bound{};
    bool residual_exact = true;
    bool residual_no_underflow = true;
    std::size_t residual_products = 0U;
    std::size_t residual_sums = 0U;
    const Q one = static_cast<Q>(1.0);
    for (std::size_t index = 0U; index < N; ++index) {
        const std::array<Q, 2U> left{rhs[index], -hx0[index]};
        const std::array<Q, 2U> right{one, one};
        const Dot2 dot = dot2(left.data(), right.data(), left.size());
        residual[index] = dot.value;
        residual_bound[index] = dot.bound;
        residual_exact = residual_exact && dot.exact;
        residual_no_underflow = residual_no_underflow && dot.no_underflow;
        residual_products += dot.products;
        residual_sums += dot.sums;
    }
    const Solve preconditioned = solve(upper, permutation, residual, inverse);
    const Dot2 rho = dot2(residual.data(), preconditioned.solution.data(), N);
    const Q rho_lower = rho.value - rho.bound;
    const bool positive = rho.exact && rho.no_underflow
        && rho_lower > static_cast<Q>(0.0);
    const bool success = permutation_exact && start.exact && x0_matches == N
        && residual_exact && residual_no_underflow
        && residual_products == 204U && residual_sums == 102U
        && preconditioned.exact && rho.products == 102U
        && rho.sums == 101U && positive;
    if (!success) return 68;

    const r63zm::Digest cache_root = r63zm::sha256(cache);
    const r63zm::Digest artifact_root = r63zm::sha256(artifact);
    const r63zm::Digest audit_root = r63zm::sha256(audit);
    const r63zm::Digest factor_root = binary64_factor_root(
        selection.factor.bytes, selection.factor.count);
    const r63zm::Digest perm_root = permutation_root(permutation);
    const r63zm::Digest inverse_root = q_scalar_root(
        "nextengine.nonlocal.r63zn.inverse-scale.v1", inverse);
    const r63zm::Digest rhs_root = q_vector_root(rhs);
    const r63zm::Digest baseline_root = q_vector_root(baseline0);
    const r63zm::Digest x_root = q_vector_root(start.solution);
    const r63zm::Digest hx_root = q_vector_root(hx0);
    const r63zm::Digest residual_root = q_vector_root(residual);
    const r63zm::Digest residual_bounds_root = q_vector_root(residual_bound);
    const r63zm::Digest z_root = q_vector_root(preconditioned.solution);
    const r63zm::Digest rho0_root = rho_root(rho);
    const std::array<r63zm::Digest, 9U> bundle_fields{{
        cache_root, artifact_root, audit_root, factor_root, perm_root,
        inverse_root, rhs_root, baseline_root, hx_root}};
    const r63zm::Digest bundle_root = digest_set_root(
        "nextengine.nonlocal.r63zn.input-bundle.v1", bundle_fields);

    constexpr std::uint64_t input_bytes =
        CACHE_BYTES + ARTIFACT_BYTES + AUDIT_BYTES;
    const std::uint64_t canonical_root_bytes =
        7U * (std::strlen(
            "nextengine.nonlocal.r63zn.binary128-vector.v1") + 1667U)
        + (std::strlen(
            "nextengine.nonlocal.r63zn.binary64-factor.v1") + 83284U)
        + (std::strlen(
            "nextengine.nonlocal.r63zn.permutation.v1") + 851U)
        + (std::strlen(
            "nextengine.nonlocal.r63zn.inverse-scale.v1") + 34U)
        + (std::strlen("nextengine.nonlocal.r63zn.rho0.v1") + 118U);
    const std::uint64_t bundle_work_root_bytes =
        std::strlen("nextengine.nonlocal.r63zn.input-bundle.v1") + 419U
        + std::strlen("nextengine.nonlocal.r63zn.candidate-work.v1") + 323U;
    const std::uint64_t event_root_bytes = EVENTS
        * (std::strlen("nextengine.nonlocal.r63zn.event.v1") + 108U);
    const std::uint64_t terminal_root_bytes =
        std::strlen("nextengine.nonlocal.r63zn.trace.v1") + 267U
        + std::strlen("nextengine.nonlocal.r63zn.result.v1") + 247U;
    const std::array<std::uint64_t, WORK_COUNTERS> work{{
        3U, input_bytes, 3U, input_bytes,
        selection.parse_reads, selection.parse_bytes,
        selection.parse_predicates, 12U, 8U,
        FACTOR_N, N, 3U * N + 1U, FACTOR_N,
        2U, 20604U, 408U, N,
        N, 2U * N, 1U, N,
        3U * N, 2U * N - 1U, 4U, 1U,
        11U, canonical_root_bytes, 2U, bundle_work_root_bytes,
        EVENTS, event_root_bytes, 2U, terminal_root_bytes,
        1U, RECEIPT_BYTES, 0U}};
    const r63zm::Digest candidate_work_root = work_root(work);
    const std::array<r63zm::Digest, 3U> input_roots{{
        cache_root, artifact_root, audit_root}};
    const r63zm::Digest parent_set_root = digest_set_root(
        "nextengine.nonlocal.r63zn.parent-set.v1", input_roots);
    const std::array<r63zm::Digest, EVENTS> events{{
        event_root(0U, parent_set_root, bundle_root),
        event_root(1U, bundle_root, factor_root),
        event_root(2U, x_root, baseline_root),
        event_root(3U, hx_root, artifact_root),
        event_root(4U, residual_root, residual_bounds_root),
        event_root(5U, z_root, factor_root),
        event_root(6U, rho0_root, candidate_work_root)}};
    constexpr std::uint64_t route = 7U;
    const r63zm::Digest trace = trace_root(route, events);
    const r63zm::Digest result = result_root(
        route, input_roots, trace, candidate_work_root);

    std::array<std::uint8_t, RECEIPT_BYTES> receipt{};
    constexpr std::array<std::uint8_t, 8U> receipt_magic{
        'N','E','R','6','3','Z','N','1'};
    std::memcpy(receipt.data(), receipt_magic.data(), receipt_magic.size());
    put_be32(receipt.data() + RECEIPT_VERSION_OFFSET, 2U);
    put_be64(receipt.data() + RECEIPT_SIZE_OFFSET, RECEIPT_BYTES);
    put_be32(receipt.data() + RECEIPT_ROUTE_OFFSET, route);
    receipt[RECEIPT_FLAGS_OFFSET] = 1U;
    receipt[RECEIPT_FLAGS_OFFSET + 1U] = 1U;
    receipt[RECEIPT_FLAGS_OFFSET + 2U] = 1U;
    put_be64(receipt.data() + RECEIPT_CACHE_SIZE_OFFSET, CACHE_BYTES);
    put_digest(receipt.data() + RECEIPT_CACHE_ROOT_OFFSET, cache_root);
    put_be64(receipt.data() + RECEIPT_ARTIFACT_SIZE_OFFSET, ARTIFACT_BYTES);
    put_digest(receipt.data() + RECEIPT_ARTIFACT_ROOT_OFFSET, artifact_root);
    put_be64(receipt.data() + RECEIPT_AUDIT_SIZE_OFFSET, AUDIT_BYTES);
    put_digest(receipt.data() + RECEIPT_AUDIT_ROOT_OFFSET, audit_root);
    put_digest(receipt.data() + RECEIPT_BUNDLE_ROOT_OFFSET, bundle_root);
    put_digest(receipt.data() + RECEIPT_FACTOR_ROOT_OFFSET, factor_root);
    put_digest(receipt.data() + RECEIPT_PERMUTATION_ROOT_OFFSET, perm_root);
    put_digest(receipt.data() + RECEIPT_INVERSE_ROOT_OFFSET, inverse_root);
    put_digest(receipt.data() + RECEIPT_RHS_ROOT_OFFSET, rhs_root);
    put_digest(receipt.data() + RECEIPT_BASELINE0_ROOT_OFFSET, baseline_root);
    put_digest(receipt.data() + RECEIPT_X0_ROOT_OFFSET, x_root);
    put_digest(receipt.data() + RECEIPT_HX0_ROOT_OFFSET, hx_root);
    put_digest(receipt.data() + RECEIPT_RESIDUAL_ROOT_OFFSET, residual_root);
    put_digest(receipt.data() + RECEIPT_RESIDUAL_BOUND_ROOT_OFFSET,
        residual_bounds_root);
    put_digest(receipt.data() + RECEIPT_Z0_ROOT_OFFSET, z_root);
    put_digest(receipt.data() + RECEIPT_RHO_ROOT_OFFSET, rho0_root);
    for (std::size_t index = 0U; index < work.size(); ++index)
        put_be64(receipt.data() + RECEIPT_WORK_OFFSET + index * 8U,
            work[index]);
    put_be64(receipt.data() + RECEIPT_EVENT_COUNT_OFFSET, EVENTS);
    for (std::size_t index = 0U; index < events.size(); ++index)
        put_digest(receipt.data() + RECEIPT_EVENTS_OFFSET + index * 32U,
            events[index]);
    put_digest(receipt.data() + RECEIPT_TRACE_ROOT_OFFSET, trace);
    put_digest(receipt.data() + RECEIPT_RESULT_ROOT_OFFSET, result);
    return write_file(argv[4], receipt) ? 0 : 69;
}
