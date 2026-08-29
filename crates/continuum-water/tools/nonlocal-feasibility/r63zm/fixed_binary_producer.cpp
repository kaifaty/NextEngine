#define R63ZM_PRODUCT_PROBE_LIBRARY
#include "fixed_product_probe.cpp"
#undef R63ZM_PRODUCT_PROBE_LIBRARY

#include <sys/stat.h>

namespace {

constexpr std::size_t READ_WORK_FIELDS = 26U;
constexpr std::size_t ADMISSION_WORK_FIELDS = 18U;
constexpr std::size_t SEAL_WORK_FIELDS = 16U;
constexpr std::size_t EVENT_SLOTS = 9U;
constexpr std::size_t PRODUCT_RECORD_BYTES = 8U
    + PRODUCT_WORK_FIELDS * 8U + 4U * 32U + 32U + 8U
    + ROWS * sizeof(Binary128);
constexpr std::size_t ARTIFACT_BYTES = 516U
    + READ_WORK_FIELDS * 8U
    + 8U + ADMISSION_WORK_FIELDS * 8U
    + SEAL_WORK_FIELDS * 8U
    + 7U * 32U + 8U
    + ROLES * PRODUCT_RECORD_BYTES + 16U;
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
constexpr std::size_t FOOTER_OFFSET = 12900U;
constexpr std::size_t PRODUCT_FLAGS_OFFSET = 0U;
constexpr std::size_t PRODUCT_WORK_OFFSET = 8U;
constexpr std::size_t PRODUCT_VECTOR_ROOTS_OFFSET = 144U;
constexpr std::size_t PRODUCT_ROOT_OFFSET = 272U;
constexpr std::size_t PRODUCT_VALUE_COUNT_OFFSET = 304U;
constexpr std::size_t PRODUCT_VALUES_OFFSET = 312U;

constexpr std::array<std::uint8_t, 8U> ARTIFACT_MAGIC{{
    'N','E','R','6','3','Z','M','1'}};
constexpr std::uint32_t ARTIFACT_VERSION = 3U;
constexpr std::uint8_t ROUTE_CANDIDATE = 0U;
constexpr std::uint8_t ROUTE_READ_REJECTED = 1U;
constexpr std::uint8_t ROUTE_ADMISSION_REJECTED = 2U;
constexpr std::uint8_t ROUTE_PRODUCT_REJECTED = 3U;

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

constexpr std::array<std::uint8_t, 16U> EXPECTED_INVERSE{{
    0x40U,0x00U,0x62U,0x4bU,0xd7U,0x3cU,0xb7U,0xabU,
    0x65U,0x90U,0x62U,0xc1U,0xb0U,0xceU,0xfeU,0x41U}};
constexpr std::array<std::uint8_t, 16U> EXPECTED_PROJECTED_SCALE{{
    0x40U,0x00U,0x62U,0x4bU,0xd7U,0x3cU,0xb7U,0xabU,
    0x65U,0x90U,0x62U,0xc1U,0xb0U,0xceU,0xfeU,0x40U}};
constexpr std::array<const char*, 7U> EXPECTED_PAYLOAD_ROOTS{{
    "03e18bacbb09013618182985941b23496d9e9bab0ca95cb3f9edfe22193e1ada",
    "2cb58eea5f6b17bc666a5d4b1c0c9ef644b2c10f154f5d651bf24e446ad6a729",
    "1cabfd31c413d1a36fc299955739196bb50924bebe744ae57c1c99f31a9fc5b9",
    "950cd25039eafe48f7bb5d5cbdd8b049fe6d14cd0a178338fd50c9d797710a5b",
    "1573eeeb09ca81f104de3d56e4bf268ac0e9839ce83525472bb9c51d803bf622",
    "4d59cf5bcaaea440e3f061d00cbb05872d4be75d408e9927050a3ffd1efff392",
    "1b5581ab5871dcd60b5ff1ce180d0572fcb9725dffe40570595c70fda866bc49",
}};

static_assert(PRODUCT_RECORD_BYTES == 1944U);
static_assert(ARTIFACT_BYTES == 12916U);
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
static_assert(EVENTS_OFFSET + EVENT_SLOTS * 32U == TRACE_OFFSET);
static_assert(TRACE_OFFSET + 32U == RESULT_OFFSET);
static_assert(RESULT_OFFSET + 32U == READ_WORK_OFFSET);
static_assert(READ_WORK_OFFSET + READ_WORK_FIELDS * 8U
    == ADMISSION_WORK_OFFSET);
static_assert(ADMISSION_WORK_OFFSET + 8U + ADMISSION_WORK_FIELDS * 8U
    == SEAL_WORK_OFFSET);
static_assert(SEAL_WORK_OFFSET + SEAL_WORK_FIELDS * 8U
    == PAYLOAD_ROOTS_OFFSET);
static_assert(PAYLOAD_ROOTS_OFFSET + 7U * 32U == PRODUCT_COUNT_OFFSET);
static_assert(PRODUCT_COUNT_OFFSET + 8U == PRODUCTS_OFFSET);
static_assert(PRODUCTS_OFFSET + ROLES * PRODUCT_RECORD_BYTES == FOOTER_OFFSET);
static_assert(FOOTER_OFFSET + 16U == ARTIFACT_BYTES);
static_assert(PRODUCT_FLAGS_OFFSET + 8U == PRODUCT_WORK_OFFSET);
static_assert(PRODUCT_WORK_OFFSET + PRODUCT_WORK_FIELDS * 8U
    == PRODUCT_VECTOR_ROOTS_OFFSET);
static_assert(PRODUCT_VECTOR_ROOTS_OFFSET + 4U * 32U
    == PRODUCT_ROOT_OFFSET);
static_assert(PRODUCT_ROOT_OFFSET + 32U == PRODUCT_VALUE_COUNT_OFFSET);
static_assert(PRODUCT_VALUE_COUNT_OFFSET + 8U == PRODUCT_VALUES_OFFSET);
static_assert(PRODUCT_VALUES_OFFSET + ROWS * sizeof(Binary128)
    == PRODUCT_RECORD_BYTES);

struct FixedReadStats final {
    std::uint64_t open_calls = 0U;
    std::uint64_t stat_calls = 0U;
    std::uint64_t read_calls = 0U;
    std::uint64_t bytes_read = 0U;
    std::uint64_t close_calls = 0U;
    std::uint64_t source_size = 0U;
    std::uint64_t source_hash_calls = 0U;
    std::uint64_t source_hash_bytes = 0U;
};

struct ReadMetrics final {
    FixedReadStats file{};
    ParseWork parse{};
    std::uint64_t payload_root_calls = 0U;
    std::uint64_t payload_root_metadata_bytes = 0U;
    std::uint64_t payload_root_payload_bytes = 0U;
    std::uint64_t selected_root_calls = 0U;
    std::uint64_t selected_root_bytes = 0U;
    std::uint64_t package_allocations = 0U;
    std::uint64_t work_root_calls = 0U;
    std::uint64_t work_root_bytes = 0U;

    std::array<std::uint64_t, READ_WORK_FIELDS> fields() const noexcept {
        return {{file.open_calls, file.stat_calls, file.read_calls,
            file.bytes_read, file.close_calls, file.source_size,
            file.source_hash_calls, file.source_hash_bytes,
            parse.take_calls, parse.take_bytes, parse.u8_reads,
            parse.u32_reads, parse.u64_reads, parse.string_reads,
            parse.vector_reads, parse.skip_operations,
            parse.structural_predicates, parse.views_created,
            payload_root_calls, payload_root_metadata_bytes,
            payload_root_payload_bytes, selected_root_calls,
            selected_root_bytes, package_allocations,
            work_root_calls, work_root_bytes}};
    }
};

struct AdmissionMetrics final {
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
    std::uint64_t work_root_calls = 0U;
    std::uint64_t work_root_bytes = 0U;
    std::uint64_t package_allocations = 0U;
    std::uint64_t route_decisions = 0U;

    std::array<std::uint64_t, ADMISSION_WORK_FIELDS> fields() const noexcept {
        return {{predicate_evaluations, first_failure_ordinal,
            digest_comparisons, digest_bytes_examined, text_comparisons,
            text_bytes_examined, scalar_comparisons, scalar_bytes_examined,
            binary128_loads, positivity_checks, finite_checks, division_calls,
            bundle_root_calls, bundle_root_bytes, work_root_calls,
            work_root_bytes, package_allocations, route_decisions}};
    }
};

struct SealMetrics final {
    std::uint64_t product_set_root_calls = 0U;
    std::uint64_t product_set_root_bytes = 0U;
    std::uint64_t event_root_calls = 0U;
    std::uint64_t event_root_bytes = 0U;
    std::uint64_t trace_root_calls = 0U;
    std::uint64_t trace_root_bytes = 0U;
    std::uint64_t body_root_calls = 0U;
    std::uint64_t body_root_bytes = 0U;
    std::uint64_t result_root_calls = 0U;
    std::uint64_t result_root_bytes = 0U;
    std::uint64_t product_records = 0U;
    std::uint64_t product_record_bytes = 0U;
    std::uint64_t output_components = 0U;
    std::uint64_t output_component_bytes = 0U;
    std::uint64_t route_decisions = 0U;
    std::uint64_t package_allocations = 0U;

    std::array<std::uint64_t, SEAL_WORK_FIELDS> fields() const noexcept {
        return {{product_set_root_calls, product_set_root_bytes,
            event_root_calls, event_root_bytes, trace_root_calls,
            trace_root_bytes, body_root_calls, body_root_bytes,
            result_root_calls, result_root_bytes, product_records,
            product_record_bytes, output_components, output_component_bytes,
            route_decisions, package_allocations}};
    }
};

template <std::size_t Count>
void update_work_array_tlv(r63zm::Sha256& hash, std::uint8_t tag,
    const std::array<std::uint64_t, Count>& work) noexcept {
    update_tlv_header(hash, tag, work.size() * sizeof(std::uint64_t));
    for (std::uint64_t field : work) update_be64(hash, field);
}

template <std::size_t DomainSize, std::size_t Count>
r63zm::Digest work_root(const char (&domain)[DomainSize],
    const std::array<std::uint64_t, Count>& work) noexcept {
    r63zm::Sha256 hash;
    update_ascii_tlv(hash, 0x01U, domain, DomainSize - 1U);
    update_work_array_tlv(hash, 0x02U, work);
    return hash.finish();
}

template <std::size_t DomainSize, std::size_t Count>
constexpr std::uint64_t work_root_bytes(
    const char (&)[DomainSize], const std::array<std::uint64_t, Count>&) {
    return (9U + DomainSize - 1U) + 9U + Count * 8U;
}

r63zm::Digest bundle_root(const r63zm::Digest& source,
    const r63zm::Digest& selected,
    const std::array<r63zm::Digest, 7U>& payloads) noexcept {
    r63zm::Sha256 hash;
    update_ascii_tlv(hash, 0x01U, BUNDLE_DOMAIN,
        sizeof(BUNDLE_DOMAIN) - 1U);
    update_tlv(hash, 0x02U, source);
    update_tlv(hash, 0x03U, selected);
    for (std::size_t index = 0U; index < payloads.size(); ++index)
        update_tlv(hash, static_cast<std::uint8_t>(0x10U + index),
            payloads[index]);
    return hash.finish();
}

constexpr std::uint64_t bundle_root_bytes() noexcept {
    return 9U + sizeof(BUNDLE_DOMAIN) - 1U + 9U * 41U;
}

constexpr std::uint64_t product_set_root_bytes() noexcept {
    return 9U + sizeof(PRODUCT_SET_DOMAIN) - 1U + 17U
        + ROLES * (10U + 41U);
}

r63zm::Digest event_root(std::uint8_t type, std::uint64_t ordinal,
    const r63zm::Digest& child, const r63zm::Digest& auxiliary) noexcept {
    r63zm::Sha256 hash;
    update_ascii_tlv(hash, 0x01U, EVENT_DOMAIN,
        sizeof(EVENT_DOMAIN) - 1U);
    update_byte_tlv(hash, 0x02U, type);
    update_tlv_be64(hash, 0x03U, ordinal);
    update_tlv(hash, 0x04U, child);
    update_tlv(hash, 0x05U, auxiliary);
    return hash.finish();
}

constexpr std::uint64_t event_root_bytes() noexcept {
    return 9U + sizeof(EVENT_DOMAIN) - 1U + 10U + 17U + 41U + 41U;
}

r63zm::Digest trace_root(std::uint8_t route, std::uint64_t event_count,
    const std::array<r63zm::Digest, EVENT_SLOTS>& events) noexcept {
    r63zm::Sha256 hash;
    update_ascii_tlv(hash, 0x01U, TRACE_DOMAIN,
        sizeof(TRACE_DOMAIN) - 1U);
    update_byte_tlv(hash, 0x02U, route);
    update_tlv_be64(hash, 0x03U, event_count);
    for (std::size_t index = 0U; index < event_count; ++index)
        update_tlv(hash, static_cast<std::uint8_t>(0x10U + index),
            events[index]);
    return hash.finish();
}

constexpr std::uint64_t trace_root_bytes(std::uint64_t event_count) noexcept {
    return 9U + sizeof(TRACE_DOMAIN) - 1U + 10U + 17U
        + event_count * 41U;
}

r63zm::Digest artifact_body_root(
    const std::array<std::uint8_t, ARTIFACT_BYTES>& bytes) noexcept {
    r63zm::Sha256 hash;
    update_ascii_tlv(hash, 0x01U, BODY_DOMAIN, sizeof(BODY_DOMAIN) - 1U);
    update_tlv_header(hash, 0x02U, bytes.size());
    hash.update(bytes);
    return hash.finish();
}

constexpr std::uint64_t body_root_bytes() noexcept {
    return 9U + sizeof(BODY_DOMAIN) - 1U + 9U + ARTIFACT_BYTES;
}

r63zm::Digest result_root(std::uint8_t route, std::uint64_t source_bytes,
    const r63zm::Digest& body) noexcept {
    r63zm::Sha256 hash;
    update_ascii_tlv(hash, 0x01U, RESULT_DOMAIN,
        sizeof(RESULT_DOMAIN) - 1U);
    update_tlv_be64(hash, 0x02U, ARTIFACT_VERSION);
    update_byte_tlv(hash, 0x03U, route);
    update_tlv_be64(hash, 0x04U, source_bytes);
    update_tlv(hash, 0x05U, body);
    update_tlv_be64(hash, 0x06U, ARTIFACT_BYTES);
    return hash.finish();
}

constexpr std::uint64_t result_root_bytes() noexcept {
    return 9U + sizeof(RESULT_DOMAIN) - 1U + 17U + 10U + 17U + 41U + 17U;
}

bool predicate(AdmissionMetrics& work, bool value) noexcept {
    ++work.predicate_evaluations;
    if (!value && work.first_failure_ordinal == 0U)
        work.first_failure_ordinal = work.predicate_evaluations;
    return value;
}

bool digest_is_counted(const r63zm::Digest& digest, const char* expected,
    AdmissionMetrics& work) noexcept {
    ++work.digest_comparisons;
    work.digest_bytes_examined += digest.size();
    return digest_equal(digest, expected);
}

bool text_is_counted(View value, const char* expected,
    AdmissionMetrics& work) noexcept {
    ++work.text_comparisons;
    std::size_t count = 0U;
    while (expected[count] != '\0') ++count;
    work.text_bytes_examined += value.size < count ? value.size : count;
    const std::size_t compared = value.size < count ? value.size : count;
    std::uint8_t difference = static_cast<std::uint8_t>(value.size != count);
    for (std::size_t index = 0U; index < compared; ++index)
        difference |= static_cast<std::uint8_t>(value.data[index]
            ^ static_cast<std::uint8_t>(expected[index]));
    return difference == 0U;
}

bool scalar_is_counted(View value,
    const std::array<std::uint8_t, 16U>& expected,
    AdmissionMetrics& work) noexcept {
    ++work.scalar_comparisons;
    work.scalar_bytes_examined += value.size < 16U ? value.size : 16U;
    return value.size == 16U && reversed(value) == expected;
}

r63zm::Digest tracked_vector_root(VectorView value,
    ReadMetrics& work) noexcept {
    ++work.payload_root_calls;
    work.payload_root_metadata_bytes += 80U;
    work.payload_root_payload_bytes += value.count * sizeof(Binary128);
    return vector_root(value);
}

r63zm::Digest tracked_selected_root(const Projection& projection,
    const std::array<r63zm::Digest, 7U>& roots,
    ReadMetrics& work) noexcept {
    ++work.selected_root_calls;
    work.selected_root_bytes += 847U;
    return selected_root(projection, roots);
}

bool read_cache_bounded(const char* path, FixedReadStats& work,
    r63zm::Digest& source) noexcept {
    constexpr std::size_t CHUNK = 4096U;
    ++work.open_calls;
    const int descriptor = open(path, O_RDONLY);
    if (descriptor < 0) {
        r63zm::Sha256 empty;
        ++work.source_hash_calls;
        source = empty.finish();
        return false;
    }
    struct stat metadata {};
    ++work.stat_calls;
    const bool stated = fstat(descriptor, &metadata) == 0
        && metadata.st_size >= 0;
    if (stated) work.source_size = static_cast<std::uint64_t>(metadata.st_size);
    r63zm::Sha256 hash;
    ++work.source_hash_calls;
    bool read_ok = stated;
    std::array<std::uint8_t, CHUNK> trailing{};
    std::uint64_t offset = 0U;
    while (read_ok && offset < work.source_size) {
        const std::size_t remaining = static_cast<std::size_t>(
            work.source_size - offset);
        std::size_t request = remaining < CHUNK ? remaining : CHUNK;
        std::uint8_t* destination = offset < CACHE_BYTES
            ? cache.data() + static_cast<std::size_t>(offset)
            : trailing.data();
        if (offset < CACHE_BYTES
            && request > CACHE_BYTES - static_cast<std::size_t>(offset))
            request = CACHE_BYTES - static_cast<std::size_t>(offset);
        ++work.read_calls;
        const ssize_t count = pread(descriptor, destination, request,
            static_cast<off_t>(offset));
        if (count <= 0) {
            read_ok = false;
            break;
        }
        hash.update(std::span<const std::uint8_t>(
            destination, static_cast<std::size_t>(count)));
        work.bytes_read += static_cast<std::uint64_t>(count);
        work.source_hash_bytes += static_cast<std::uint64_t>(count);
        offset += static_cast<std::uint64_t>(count);
        if (static_cast<std::size_t>(count) != request) read_ok = false;
    }
    ++work.close_calls;
    const bool closed = close(descriptor) == 0;
    source = hash.finish();
    return read_ok && closed && work.source_size == CACHE_BYTES
        && work.bytes_read == CACHE_BYTES;
}

bool admit_projection(const Projection& projection, const Cursor& reader,
    const r63zm::Digest& source, ReadMetrics& read,
    AdmissionMetrics& admission,
    std::array<r63zm::Digest, 7U>& payload_roots,
    r63zm::Digest& selected, r63zm::Digest& bundle) noexcept {
    if (!predicate(admission, reader.ok() && reader.position() == CACHE_BYTES))
        return false;
    if (!predicate(admission, projection.dimension == ROWS)) return false;
    if (!predicate(admission, projection.columns == COLUMNS)) return false;
    if (!predicate(admission,
            projection.tangent.count == ROWS * COLUMNS)) return false;
    if (!predicate(admission, text_is_counted(projection.declared_tangent,
            "114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1",
            admission))) return false;
    if (!predicate(admission, scalar_is_counted(
            projection.inverse, EXPECTED_INVERSE, admission))) return false;
    if (!predicate(admission, scalar_is_counted(
            projection.projected_scale, EXPECTED_PROJECTED_SCALE, admission)))
        return false;
    if (!predicate(admission, text_is_counted(projection.declared_projected,
            "0df32db6fb6c7b3a6fb5e3760c7ff810f92e31baa1544a0911a8e0b6a11274ef",
            admission))) return false;
    if (!predicate(admission, text_is_counted(projection.declared_original,
            "64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1",
            admission))) return false;
    ++admission.binary128_loads;
    const Binary128 sigma = load_binary128(projection.sigma);
    ++admission.positivity_checks;
    if (!predicate(admission, sigma > ZERO)) return false;
    ++admission.finite_checks;
    if (!predicate(admission, finiteq(sigma) != 0)) return false;
    ++admission.binary128_loads;
    const Binary128 inverse = load_binary128(projection.inverse);
    ++admission.division_calls;
    ++admission.scalar_comparisons;
    admission.scalar_bytes_examined += sizeof(Binary128);
    if (!predicate(admission, sigma == ONE / inverse)) return false;
    for (const VectorView role : projection.roles)
        if (!predicate(admission, role.count == ROWS)) return false;
    payload_roots[0U] = tracked_vector_root(projection.tangent, read);
    if (!predicate(admission, digest_is_counted(payload_roots[0U],
            EXPECTED_PAYLOAD_ROOTS[0U], admission))) return false;
    for (std::size_t role = 0U; role < ROLES; ++role) {
        payload_roots[role + 1U] = tracked_vector_root(
            projection.roles[role], read);
        if (!predicate(admission, digest_is_counted(payload_roots[role + 1U],
                EXPECTED_PAYLOAD_ROOTS[role + 1U], admission))) return false;
    }
    selected = tracked_selected_root(projection, payload_roots, read);
    if (!predicate(admission, digest_is_counted(selected,
            "dc1b5328d2999dadafa8915ebe10d36029cc5b5ba13f2657670de75ee4a466a5",
            admission))) return false;
    if (!predicate(admission, digest_is_counted(source,
            "23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84",
            admission))) return false;
    ++admission.bundle_root_calls;
    admission.bundle_root_bytes += bundle_root_bytes();
    bundle = bundle_root(source, selected, payload_roots);
    return true;
}

class ArtifactWriter final {
public:
    explicit ArtifactWriter(std::array<std::uint8_t, ARTIFACT_BYTES>& bytes)
        : bytes_(bytes) {}
    void raw(std::span<const std::uint8_t> value) noexcept {
        if (!ok_ || value.size() > bytes_.size() - position_) {
            ok_ = false;
            return;
        }
        for (std::uint8_t byte : value) bytes_[position_++] = byte;
    }
    template <std::size_t Count>
    void raw(const std::array<std::uint8_t, Count>& value) noexcept {
        raw(std::span<const std::uint8_t>(value));
    }
    void u8(std::uint8_t value) noexcept { raw({&value, 1U}); }
    void u32(std::uint32_t value) noexcept {
        for (std::size_t index = 0U; index < 4U; ++index)
            u8(static_cast<std::uint8_t>(value >> (24U - 8U * index)));
    }
    void u64(std::uint64_t value) noexcept {
        for (std::size_t index = 0U; index < 8U; ++index)
            u8(static_cast<std::uint8_t>(value >> (56U - 8U * index)));
    }
    template <std::size_t Count>
    void work(const std::array<std::uint64_t, Count>& value) noexcept {
        for (std::uint64_t field : value) u64(field);
    }
    void zeros(std::size_t count) noexcept {
        while (count-- != 0U) u8(0U);
    }
    void binary128(Binary128 value) noexcept { raw(canonical(value)); }
    bool complete() const noexcept {
        return ok_ && position_ == bytes_.size();
    }
private:
    std::array<std::uint8_t, ARTIFACT_BYTES>& bytes_;
    std::size_t position_ = 0U;
    bool ok_ = true;
};

bool serialize_artifact(std::array<std::uint8_t, ARTIFACT_BYTES>& artifact,
    std::uint64_t source_bytes, const r63zm::Digest& source,
    std::uint8_t route, const r63zm::Digest& selected,
    const r63zm::Digest& bundle, const r63zm::Digest& product_set,
    std::uint32_t event_count,
    const std::array<r63zm::Digest, EVENT_SLOTS>& events,
    const r63zm::Digest& trace,
    const std::array<std::uint64_t, READ_WORK_FIELDS>& read_work,
    bool admission_exact,
    const std::array<std::uint64_t, ADMISSION_WORK_FIELDS>& admission_work,
    const std::array<std::uint64_t, SEAL_WORK_FIELDS>& seal_work,
    const std::array<r63zm::Digest, 7U>& payload_roots,
    std::uint32_t product_count,
    const std::array<Product, ROLES>& products,
    std::uint64_t product_set_calls,
    std::uint64_t product_set_bytes) noexcept {
    ArtifactWriter writer(artifact);
    writer.raw(ARTIFACT_MAGIC);
    writer.u32(ARTIFACT_VERSION);
    writer.u64(ARTIFACT_BYTES);
    writer.u64(source_bytes);
    writer.raw(source);
    writer.u32(route);
    writer.raw(selected);
    writer.raw(bundle);
    writer.raw(product_set);
    writer.u32(event_count);
    for (const r63zm::Digest& event : events) writer.raw(event);
    writer.raw(trace);
    writer.zeros(32U);
    writer.work(read_work);
    writer.u8(admission_exact ? 1U : 0U);
    writer.zeros(7U);
    writer.work(admission_work);
    writer.work(seal_work);
    for (const r63zm::Digest& root : payload_roots) writer.raw(root);
    writer.u32(product_count);
    writer.zeros(4U);
    for (std::size_t role = 0U; role < ROLES; ++role) {
        const bool available = role < product_count;
        writer.u8(available && products[role].exact ? 1U : 0U);
        writer.u8(available && products[role].no_underflow ? 1U : 0U);
        writer.u8(available ? static_cast<std::uint8_t>(role) : 0U);
        writer.zeros(5U);
        if (available) {
            writer.work(products[role].work.fields());
            for (const r63zm::Digest& root : products[role].vector_roots)
                writer.raw(root);
            writer.raw(products[role].raw_root);
            writer.u64(ROWS);
            for (Binary128 value : products[role].value)
                writer.binary128(value);
        } else {
            writer.zeros(PRODUCT_WORK_FIELDS * 8U + 4U * 32U + 32U + 8U
                + ROWS * sizeof(Binary128));
        }
    }
    writer.u64(product_set_calls);
    writer.u64(product_set_bytes);
    return writer.complete();
}

void patch_result(std::array<std::uint8_t, ARTIFACT_BYTES>& artifact,
    const r63zm::Digest& result) noexcept {
    for (std::size_t index = 0U; index < result.size(); ++index)
        artifact[RESULT_OFFSET + index] = result[index];
}

bool write_fixed_file(const char* path,
    const std::array<std::uint8_t, ARTIFACT_BYTES>& bytes) noexcept {
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
    if (argc != 3 || std::fesetround(FE_TONEAREST) != 0
        || std::fegetround() != FE_TONEAREST)
        return 2;

    ReadMetrics read;
    AdmissionMetrics admission;
    SealMetrics seal;
    r63zm::Digest source{};
    r63zm::Digest selected{};
    r63zm::Digest bundle{};
    r63zm::Digest product_set{};
    std::array<r63zm::Digest, 7U> payload_roots{};
    std::array<Product, ROLES> products{};
    std::array<r63zm::Digest, ROLES> product_roots{};
    std::uint8_t route = ROUTE_READ_REJECTED;
    std::uint32_t product_count = 0U;
    bool admission_exact = false;

    const bool read_exact = read_cache_bounded(argv[1], read.file, source);
    Projection projection{};
    if (read_exact) {
        Cursor reader(std::span<const std::uint8_t>(cache), &read.parse);
        projection = parse(reader);
        admission_exact = admit_projection(projection, reader, source,
            read, admission, payload_roots, selected, bundle);
        ++admission.route_decisions;
        route = admission_exact ? ROUTE_CANDIDATE
                                : ROUTE_ADMISSION_REJECTED;
    }

    if (route == ROUTE_CANDIDATE) {
        bool products_exact = true;
        for (std::size_t role = 0U; role < ROLES; ++role) {
            compute_product(products[role], role, projection,
                payload_roots[role + 1U]);
            product_roots[role] = products[role].raw_root;
            products_exact = products_exact && products[role].exact
                && products[role].no_underflow
                && product_work_exact(products[role].work);
        }
        product_count = ROLES;
        ++seal.product_set_root_calls;
        seal.product_set_root_bytes += product_set_root_bytes();
        product_set = raw_product_set_root(product_roots);
        route = products_exact ? ROUTE_CANDIDATE : ROUTE_PRODUCT_REJECTED;
    }

    ++read.work_root_calls;
    read.work_root_bytes = work_root_bytes(READ_WORK_DOMAIN, read.fields());
    const auto read_work = read.fields();
    const r63zm::Digest read_work_digest =
        work_root(READ_WORK_DOMAIN, read_work);

    std::array<std::uint64_t, ADMISSION_WORK_FIELDS> admission_work{};
    r63zm::Digest admission_work_digest{};
    if (route != ROUTE_READ_REJECTED) {
        ++admission.work_root_calls;
        admission.work_root_bytes = work_root_bytes(
            ADMISSION_WORK_DOMAIN, admission.fields());
        admission_work = admission.fields();
        admission_work_digest = work_root(
            ADMISSION_WORK_DOMAIN, admission_work);
    }

    std::uint32_t event_count = 1U;
    if (route == ROUTE_ADMISSION_REJECTED) event_count = 2U;
    if (route == ROUTE_CANDIDATE || route == ROUTE_PRODUCT_REJECTED)
        event_count = EVENT_SLOTS;
    seal.product_records = product_count;
    seal.product_record_bytes =
        static_cast<std::uint64_t>(product_count) * PRODUCT_RECORD_BYTES;
    seal.output_components = static_cast<std::uint64_t>(product_count) * ROWS;
    seal.output_component_bytes = seal.output_components * sizeof(Binary128);
    seal.event_root_calls = event_count;
    seal.event_root_bytes = event_count * event_root_bytes();
    ++seal.trace_root_calls;
    seal.trace_root_bytes = trace_root_bytes(event_count);
    ++seal.body_root_calls;
    seal.body_root_bytes = body_root_bytes();
    ++seal.result_root_calls;
    seal.result_root_bytes = result_root_bytes();
    ++seal.route_decisions;
    const auto seal_work = seal.fields();
    std::array<r63zm::Digest, EVENT_SLOTS> events{};
    events[0U] = event_root(0U, 0U, source, read_work_digest);
    if (event_count >= 2U)
        events[1U] = event_root(1U, 1U,
            admission_exact ? selected : r63zm::Digest{},
            admission_work_digest);
    if (event_count == EVENT_SLOTS) {
        events[2U] = event_root(2U, 2U, bundle, selected);
        for (std::size_t role = 0U; role < ROLES; ++role)
            events[3U + role] = event_root(3U, 3U + role,
                product_roots[role], payload_roots[role + 1U]);
    }
    const r63zm::Digest trace = trace_root(route, event_count, events);

    std::array<std::uint8_t, ARTIFACT_BYTES> artifact{};
    if (!serialize_artifact(artifact, read.file.source_size, source, route,
            admission_exact ? selected : r63zm::Digest{},
            admission_exact ? bundle : r63zm::Digest{},
            product_count == ROLES ? product_set : r63zm::Digest{},
            event_count, events, trace, read_work, admission_exact,
            admission_work, seal_work,
            admission_exact ? payload_roots
                            : std::array<r63zm::Digest, 7U>{},
            product_count, products, seal.product_set_root_calls,
            seal.product_set_root_bytes))
        return 2;
    const r63zm::Digest body = artifact_body_root(artifact);
    const r63zm::Digest terminal = result_root(
        route, read.file.source_size, body);
    patch_result(artifact, terminal);
    if (!write_fixed_file(argv[2], artifact)) return 2;
    return route == ROUTE_CANDIDATE ? 0 : 1;
}
