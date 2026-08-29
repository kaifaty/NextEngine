#define R63ZM_RAW_PROBE_LIBRARY
#include "raw_semantic_probe.cpp"
#undef R63ZM_RAW_PROBE_LIBRARY

#include <bit>
#include <cfenv>
#include <cfloat>
#include <cmath>
#include <cstring>
#include <quadmath.h>

namespace {

using Binary128 = __float128;

constexpr Binary128 ZERO = static_cast<Binary128>(0.0);
constexpr Binary128 ONE = static_cast<Binary128>(1.0);

constexpr std::size_t ROWS = 102U;
constexpr std::size_t COLUMNS = 315U;
constexpr std::size_t ROLES = 6U;
constexpr std::size_t PRODUCT_WORK_FIELDS = 17U;

constexpr char PRODUCT_DOMAIN[] =
    "nextengine.nonlocal.r63zm.product-binary.v1";
constexpr char PRODUCT_SET_DOMAIN[] =
    "nextengine.nonlocal.r63zm.product-set-binary.v1";

static_assert(sizeof(Binary128) == 16U);
static_assert(std::endian::native == std::endian::little);
static_assert(__FLT128_MANT_DIG__ == 113);
static_assert(__FLT128_MAX_EXP__ == 16384);

struct Pair final {
    bool exact = false;
    bool no_underflow = false;
    Binary128 high = ZERO;
    Binary128 low = ZERO;
};

struct Dot2 final {
    bool exact = false;
    bool no_underflow = false;
    Binary128 value = ZERO;
    Binary128 bound = ZERO;
    Binary128 absolute_products = ZERO;
};

struct ProductWork final {
    std::uint64_t kernel_calls = 0U;
    std::uint64_t input_guard_checks = 0U;
    std::uint64_t inner_dots = 0U;
    std::uint64_t outer_dots = 0U;
    std::uint64_t inner_terms = 0U;
    std::uint64_t outer_terms = 0U;
    std::uint64_t propagation_terms = 0U;
    std::uint64_t scale_products = 0U;
    std::uint64_t fixed_result_spans = 0U;
    std::uint64_t result_component_writes = 0U;
    std::uint64_t vector_root_calls = 0U;
    std::uint64_t vector_root_metadata_bytes = 0U;
    std::uint64_t vector_root_payload_bytes = 0U;
    std::uint64_t product_root_calls = 0U;
    std::uint64_t product_root_bytes = 0U;
    std::uint64_t temporary_operand_copies = 0U;
    std::uint64_t package_allocations = 0U;

    std::array<std::uint64_t, PRODUCT_WORK_FIELDS> fields() const noexcept {
        return {{kernel_calls, input_guard_checks, inner_dots, outer_dots,
            inner_terms, outer_terms, propagation_terms, scale_products,
            fixed_result_spans, result_component_writes, vector_root_calls,
            vector_root_metadata_bytes, vector_root_payload_bytes,
            product_root_calls, product_root_bytes,
            temporary_operand_copies, package_allocations}};
    }
};

struct Product final {
    bool exact = false;
    bool no_underflow = false;
    std::array<Binary128, COLUMNS> intermediate{};
    std::array<Binary128, COLUMNS> intermediate_bound{};
    std::array<Binary128, ROWS> value{};
    std::array<Binary128, ROWS> bound{};
    std::array<r63zm::Digest, 4U> vector_roots{};
    r63zm::Digest raw_root{};
#ifndef R63ZM_PRODUCT_PROBE_LIBRARY
    r63zm::Digest legacy_root{};
#endif
    ProductWork work{};
};

bool normal_or_zero(Binary128 value) noexcept {
    return finiteq(value) != 0
        && (value == ZERO || fabsq(value) >= ldexpq(ONE, -16382));
}

Binary128 up_add(Binary128 left, Binary128 right) noexcept {
    if (left == ZERO && right == ZERO) return ZERO;
    return nextafterq(left + right, HUGE_VALQ);
}

Binary128 up_multiply(Binary128 left, Binary128 right) noexcept {
    if (left == ZERO || right == ZERO) return ZERO;
    return nextafterq(left * right, HUGE_VALQ);
}

Binary128 up_divide(Binary128 numerator, Binary128 denominator) noexcept {
    if (numerator == ZERO) return ZERO;
    return nextafterq(numerator / denominator, HUGE_VALQ);
}

Pair two_sum(Binary128 left, Binary128 right) noexcept {
    Pair result;
    result.high = left + right;
    const Binary128 virtual_right = result.high - left;
    result.low = (left - (result.high - virtual_right))
        + (right - virtual_right);
    result.no_underflow = normal_or_zero(left) && normal_or_zero(right)
        && normal_or_zero(result.high) && normal_or_zero(virtual_right)
        && normal_or_zero(result.low);
    result.exact = finiteq(result.high) != 0 && finiteq(result.low) != 0;
    return result;
}

Pair two_product(Binary128 left, Binary128 right) noexcept {
    Pair result;
    result.high = left * right;
    result.low = fmaq(left, right, -result.high);
    const bool product_not_lost = left == ZERO || right == ZERO
        || result.high != ZERO || result.low != ZERO;
    result.no_underflow = product_not_lost && normal_or_zero(left)
        && normal_or_zero(right) && normal_or_zero(result.high)
        && normal_or_zero(result.low);
    result.exact = finiteq(result.high) != 0 && finiteq(result.low) != 0;
    return result;
}

template <typename LeftAt, typename RightAt>
Dot2 dot2(std::size_t count, const LeftAt& left_at,
    const RightAt& right_at) noexcept {
    Dot2 dot;
    if (count == 0U) return dot;
    const Pair first = two_product(left_at(0U), right_at(0U));
    Binary128 primary = first.high;
    Binary128 correction = first.low;
    bool no_underflow = first.no_underflow;
    dot.absolute_products = up_multiply(
        fabsq(left_at(0U)), fabsq(right_at(0U)));
    for (std::size_t index = 1U; index < count; ++index) {
        const Pair product = two_product(left_at(index), right_at(index));
        const Pair sum = two_sum(primary, product.high);
        const Binary128 local = sum.low + product.low;
        correction = correction + local;
        primary = sum.high;
        no_underflow = no_underflow && product.no_underflow
            && sum.no_underflow && normal_or_zero(local)
            && normal_or_zero(correction);
        dot.absolute_products = up_add(dot.absolute_products,
            up_multiply(fabsq(left_at(index)), fabsq(right_at(index))));
    }
    dot.value = primary + correction;
    no_underflow = no_underflow && normal_or_zero(dot.value);
    const Binary128 unit = ldexpq(ONE, -112);
    const Binary128 count_unit = static_cast<Binary128>(count) * unit;
    const Binary128 gamma = up_divide(count_unit, ONE - count_unit);
    const Binary128 numerator = up_add(
        up_multiply(unit, fabsq(dot.value)),
        up_multiply(up_multiply(gamma, gamma), dot.absolute_products));
    dot.bound = up_divide(numerator, ONE - unit);
    dot.no_underflow = no_underflow;
    dot.exact = first.exact && finiteq(dot.value) != 0
        && finiteq(dot.bound) != 0 && dot.bound >= ZERO;
    return dot;
}

Binary128 load_binary128(const std::uint8_t* bytes) noexcept {
    Binary128 value = ZERO;
    std::memcpy(&value, bytes, sizeof(value));
    return value;
}

Binary128 load_binary128(View value) noexcept {
    return value.size == sizeof(Binary128)
        ? load_binary128(value.data) : ZERO;
}

Binary128 vector_at(VectorView value, std::size_t index) noexcept {
    return index < value.count
        ? load_binary128(value.data + index * sizeof(Binary128)) : ZERO;
}

std::array<std::uint8_t, 16U> canonical(Binary128 value) noexcept {
    std::array<std::uint8_t, 16U> host{};
    std::array<std::uint8_t, 16U> result{};
    std::memcpy(host.data(), &value, host.size());
    for (std::size_t index = 0U; index < result.size(); ++index)
        result[index] = host[result.size() - 1U - index];
    return result;
}

template <std::size_t Count>
r63zm::Digest raw_vector_root(
    const std::array<Binary128, Count>& values) noexcept {
    r63zm::Sha256 hash;
    update_tlv(hash, 0x01U, VECTOR_DOMAIN);
    update_tlv_be64(hash, 0x02U, values.size());
    update_tlv_header(hash, 0x03U, values.size() * sizeof(Binary128));
    for (Binary128 value : values) hash.update(canonical(value));
    return hash.finish();
}

void update_ascii_tlv(r63zm::Sha256& hash, std::uint8_t tag,
    const char* text, std::size_t count) noexcept {
    update_tlv_header(hash, tag, count);
    hash.update(std::span<const std::uint8_t>(
        reinterpret_cast<const std::uint8_t*>(text), count));
}

void update_byte_tlv(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint8_t value) noexcept {
    update_tlv_header(hash, tag, 1U);
    hash.update_byte(value);
}

void update_work_tlv(r63zm::Sha256& hash, std::uint8_t tag,
    const ProductWork& work) noexcept {
    const auto fields = work.fields();
    update_tlv_header(hash, tag, fields.size() * sizeof(std::uint64_t));
    for (std::uint64_t field : fields) update_be64(hash, field);
}

r63zm::Digest raw_product_root(std::size_t role, bool exact,
    bool no_underflow, const r63zm::Digest& input_root,
    const ProductWork& work,
    const std::array<r63zm::Digest, 4U>& vector_roots) noexcept {
    r63zm::Sha256 hash;
    update_ascii_tlv(hash, 0x01U, PRODUCT_DOMAIN,
        sizeof(PRODUCT_DOMAIN) - 1U);
    update_byte_tlv(hash, 0x02U, static_cast<std::uint8_t>(role));
    update_byte_tlv(hash, 0x03U, exact ? 1U : 0U);
    update_byte_tlv(hash, 0x04U, no_underflow ? 1U : 0U);
    update_tlv_be64(hash, 0x05U, ROWS);
    update_tlv_be64(hash, 0x06U, COLUMNS);
    update_tlv(hash, 0x07U, input_root);
    update_work_tlv(hash, 0x08U, work);
    for (std::size_t index = 0U; index < vector_roots.size(); ++index)
        update_tlv(hash, static_cast<std::uint8_t>(0x10U + index),
            vector_roots[index]);
    return hash.finish();
}

r63zm::Digest raw_product_set_root(
    const std::array<r63zm::Digest, ROLES>& roots) noexcept {
    r63zm::Sha256 hash;
    update_ascii_tlv(hash, 0x01U, PRODUCT_SET_DOMAIN,
        sizeof(PRODUCT_SET_DOMAIN) - 1U);
    update_tlv_be64(hash, 0x02U, roots.size());
    for (std::size_t role = 0U; role < roots.size(); ++role) {
        update_byte_tlv(hash, static_cast<std::uint8_t>(0x10U + 2U * role),
            static_cast<std::uint8_t>(role));
        update_tlv(hash, static_cast<std::uint8_t>(0x11U + 2U * role),
            roots[role]);
    }
    return hash.finish();
}

#ifndef R63ZM_PRODUCT_PROBE_LIBRARY
template <std::size_t Count>
r63zm::Digest legacy_vector_root(
    const std::array<Binary128, Count>& values) noexcept {
    r63zm::Sha256 hash;
    std::array<char, 128U> text{};
    int count = std::snprintf(text.data(), text.size(), "%zu", values.size());
    if (count <= 0 || static_cast<std::size_t>(count) >= text.size()) return {};
    hash.update(std::span<const std::uint8_t>(
        reinterpret_cast<const std::uint8_t*>(text.data()),
        static_cast<std::size_t>(count)));
    for (Binary128 value : values) {
        hash.update_byte(static_cast<std::uint8_t>(':'));
        count = quadmath_snprintf(text.data(), text.size(), "%Qa", value);
        if (count <= 0 || static_cast<std::size_t>(count) >= text.size())
            return {};
        hash.update(std::span<const std::uint8_t>(
            reinterpret_cast<const std::uint8_t*>(text.data()),
            static_cast<std::size_t>(count)));
    }
    return hash.finish();
}

r63zm::Digest legacy_product_root(
    const Product& product) noexcept {
    const std::array<r63zm::Digest, 4U> roots{{
        legacy_vector_root(product.intermediate),
        legacy_vector_root(product.intermediate_bound),
        legacy_vector_root(product.value),
        legacy_vector_root(product.bound)}};
    r63zm::Sha256 hash;
    std::array<char, 256U> prefix{};
    const int count = std::snprintf(prefix.data(), prefix.size(),
        "%d:%d:%zu:%zu:%llu:%llu:%llu:%llu:%llu:",
        product.exact ? 1 : 0, product.no_underflow ? 1 : 0,
        ROWS, COLUMNS,
        static_cast<unsigned long long>(product.work.inner_dots),
        static_cast<unsigned long long>(product.work.outer_dots),
        static_cast<unsigned long long>(product.work.inner_terms),
        static_cast<unsigned long long>(product.work.outer_terms),
        static_cast<unsigned long long>(product.work.scale_products));
    if (count <= 0 || static_cast<std::size_t>(count) >= prefix.size())
        return {};
    hash.update(std::span<const std::uint8_t>(
        reinterpret_cast<const std::uint8_t*>(prefix.data()),
        static_cast<std::size_t>(count)));
    for (std::size_t index = 0U; index < roots.size(); ++index) {
        const auto root_hex = hex(roots[index]);
        hash.update(std::span<const std::uint8_t>(
            reinterpret_cast<const std::uint8_t*>(root_hex.data()), 64U));
        if (index + 1U != roots.size())
            hash.update_byte(static_cast<std::uint8_t>(':'));
    }
    return hash.finish();
}
#endif

std::uint64_t product_root_material_bytes() noexcept {
    return (9U + sizeof(PRODUCT_DOMAIN) - 1U)
        + 3U * 10U + 2U * 17U + 41U
        + (9U + PRODUCT_WORK_FIELDS * 8U) + 4U * 41U;
}

void compute_product(Product& product, std::size_t role,
    const Projection& projection, const r63zm::Digest& input_root) noexcept {
    ++product.work.input_guard_checks;
    if (role >= projection.roles.size()
        || projection.dimension != ROWS || projection.columns != COLUMNS
        || projection.tangent.count != ROWS * COLUMNS
        || projection.roles[role].count != ROWS)
        return;
    ++product.work.kernel_calls;
    bool exact = true;
    bool no_underflow = true;
    for (std::size_t scalar = 0U; scalar < COLUMNS; ++scalar) {
        const Dot2 dot = dot2(ROWS,
            [&](std::size_t row) {
                return vector_at(projection.tangent,
                    row * COLUMNS + scalar);
            },
            [&](std::size_t row) {
                return vector_at(projection.roles[role], row);
            });
        product.intermediate[scalar] = dot.value;
        product.intermediate_bound[scalar] = dot.bound;
        exact = exact && dot.exact && finiteq(dot.value) != 0
            && finiteq(dot.bound) != 0;
        no_underflow = no_underflow && dot.no_underflow;
        ++product.work.inner_dots;
        product.work.inner_terms += ROWS;
        product.work.result_component_writes += 2U;
    }
    const Binary128 sigma = load_binary128(projection.sigma);
    for (std::size_t row = 0U; row < ROWS; ++row) {
        const Dot2 dot = dot2(COLUMNS,
            [&](std::size_t scalar) {
                return vector_at(projection.tangent,
                    row * COLUMNS + scalar);
            },
            [&](std::size_t scalar) {
                return product.intermediate[scalar];
            });
        Binary128 propagated = dot.bound;
        for (std::size_t scalar = 0U; scalar < COLUMNS; ++scalar) {
            propagated = up_add(propagated, up_multiply(
                fabsq(vector_at(projection.tangent,
                    row * COLUMNS + scalar)),
                product.intermediate_bound[scalar]));
            ++product.work.propagation_terms;
        }
        const Pair scaled = two_product(sigma, dot.value);
        product.value[row] = scaled.high;
        product.bound[row] = up_add(fabsq(scaled.low),
            up_multiply(fabsq(sigma), propagated));
        exact = exact && dot.exact && scaled.exact
            && finiteq(product.value[row]) != 0
            && finiteq(product.bound[row]) != 0;
        no_underflow = no_underflow && dot.no_underflow
            && scaled.no_underflow;
        ++product.work.outer_dots;
        product.work.outer_terms += COLUMNS;
        ++product.work.scale_products;
        product.work.result_component_writes += 2U;
    }
    product.work.fixed_result_spans = 4U;
    product.exact = exact && no_underflow
        && product.work.inner_dots == COLUMNS
        && product.work.outer_dots == ROWS
        && product.work.inner_terms == ROWS * COLUMNS
        && product.work.outer_terms == ROWS * COLUMNS
        && product.work.propagation_terms == ROWS * COLUMNS
        && product.work.scale_products == ROWS
        && product.work.result_component_writes
            == 2U * COLUMNS + 2U * ROWS;
    product.no_underflow = no_underflow;
    product.vector_roots = {{
        raw_vector_root(product.intermediate),
        raw_vector_root(product.intermediate_bound),
        raw_vector_root(product.value),
        raw_vector_root(product.bound)}};
    product.work.vector_root_calls = 4U;
    product.work.vector_root_metadata_bytes = 4U * 80U;
    product.work.vector_root_payload_bytes =
        sizeof(Binary128) * (2U * COLUMNS + 2U * ROWS);
    product.work.product_root_calls = 1U;
    product.work.product_root_bytes = product_root_material_bytes();
    product.raw_root = raw_product_root(role, product.exact,
        product.no_underflow, input_root, product.work,
        product.vector_roots);
}

bool product_work_exact(const ProductWork& work) noexcept {
    return work.kernel_calls == 1U && work.input_guard_checks == 1U
        && work.inner_dots == COLUMNS && work.outer_dots == ROWS
        && work.inner_terms == ROWS * COLUMNS
        && work.outer_terms == ROWS * COLUMNS
        && work.propagation_terms == ROWS * COLUMNS
        && work.scale_products == ROWS && work.fixed_result_spans == 4U
        && work.result_component_writes == 2U * COLUMNS + 2U * ROWS
        && work.vector_root_calls == 4U
        && work.vector_root_metadata_bytes == 320U
        && work.vector_root_payload_bytes
            == sizeof(Binary128) * (2U * COLUMNS + 2U * ROWS)
        && work.product_root_calls == 1U
        && work.product_root_bytes == product_root_material_bytes()
        && work.temporary_operand_copies == 0U
        && work.package_allocations == 0U;
}

} // namespace

#ifndef R63ZM_PRODUCT_PROBE_LIBRARY
int main(int argc, char** argv) {
    if (argc != 2 || std::fesetround(FE_TONEAREST) != 0
        || std::fegetround() != FE_TONEAREST || !read_cache(argv[1]))
        return 2;
    Cursor reader(cache);
    const Projection projection = parse(reader);
    std::array<r63zm::Digest, 7U> payload_roots{};
    payload_roots[0U] = vector_root(projection.tangent);
    for (std::size_t role = 0U; role < ROLES; ++role)
        payload_roots[role + 1U] = vector_root(projection.roles[role]);
    std::array<Product, ROLES> products{};
    std::array<r63zm::Digest, ROLES> raw_roots{};
    constexpr std::array<const char*, ROLES> EXPECTED_LEGACY_ROOTS{{
        "228ec462d65a0057fba4837bd11749eddc2852ac396007bd35d9d15266b58db3",
        "3622a42af801f7f9301a2f1f240f95bb4884138cb7dff671caa9eef9f6eed1f0",
        "3e03c48d5403474e0522b1c2e12669ec0c16e612b013cc2b6fc111522b560970",
        "e2ccb77d66f9970c82c88fdd5648d8987dc7e1f4d978b6ca48a6e0a2405fe3c4",
        "3b7c28cb9a881b76524b7870288774a4878fca9e958ab30842ac58dae539e465",
        "fafad4f0a12a2fc40e710d4bef5270efda9f6c9f5060e374138f435023af6fdb",
    }};
    bool exact = reader.ok() && reader.position() == cache.size()
        && projection.dimension == ROWS && projection.columns == COLUMNS
        && projection.tangent.count == ROWS * COLUMNS;
    for (std::size_t role = 0U; role < ROLES; ++role) {
        compute_product(products[role], role, projection,
            payload_roots[role + 1U]);
        raw_roots[role] = products[role].raw_root;
        exact = exact && products[role].exact
            && products[role].no_underflow
            && product_work_exact(products[role].work);
    }
    const r63zm::Digest product_set_root = raw_product_set_root(raw_roots);
    for (std::size_t role = 0U; role < ROLES; ++role) {
        products[role].legacy_root = legacy_product_root(products[role]);
        exact = exact && digest_equal(products[role].legacy_root,
            EXPECTED_LEGACY_ROOTS[role]);
    }
    const auto set_hex = hex(product_set_root);
    std::printf("{\"schema\":\"r63zm-fixed-product-probe-v1\","
        "\"status\":\"%s\",\"product_root_bytes\":%llu,"
        "\"product_set_root\":\"%s\",\"products\":[",
        exact ? "PASS" : "FAIL",
        static_cast<unsigned long long>(product_root_material_bytes()),
        set_hex.data());
    for (std::size_t role = 0U; role < ROLES; ++role) {
        const auto raw_hex = hex(products[role].raw_root);
        const auto legacy_hex = hex(products[role].legacy_root);
        std::printf("%s{\"role\":%zu,\"exact\":%s,"
            "\"no_underflow\":%s,\"raw_root\":\"%s\","
            "\"legacy_root\":\"%s\",\"vector_roots\":[",
            role == 0U ? "" : ",", role,
            products[role].exact ? "true" : "false",
            products[role].no_underflow ? "true" : "false",
            raw_hex.data(), legacy_hex.data());
        for (std::size_t index = 0U;
             index < products[role].vector_roots.size(); ++index) {
            const auto vector_hex = hex(products[role].vector_roots[index]);
            std::printf("%s\"%s\"", index == 0U ? "" : ",",
                vector_hex.data());
        }
        const auto fields = products[role].work.fields();
        std::printf("],\"work\":[");
        for (std::size_t index = 0U; index < fields.size(); ++index)
            std::printf("%s%llu", index == 0U ? "" : ",",
                static_cast<unsigned long long>(fields[index]));
        std::printf("]}");
    }
    std::printf("]}\n");
    return exact ? 0 : 1;
}
#endif
