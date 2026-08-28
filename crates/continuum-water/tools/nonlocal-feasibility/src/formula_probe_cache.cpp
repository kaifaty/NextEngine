#include "formula_probe_cache.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cstdint>
#include <fstream>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <vector>

namespace nextengine::nonlocal::fcr {
namespace {

constexpr std::array<char, 8U> FORMULA_PROBE_CACHE_MAGIC{
    'N', 'E', 'F', 'P', 'R', 'B', '1', '\0'};
constexpr std::uint32_t FORMULA_PROBE_CACHE_VERSION = 1U;
constexpr std::uint32_t FORMULA_PROBE_ENDIAN_MARKER = 0x01020304U;
constexpr std::uint64_t FORMULA_PROBE_MAX_STRING = 1U << 20U;
constexpr std::uint64_t FORMULA_PROBE_MAX_VECTOR = 1U << 20U;
constexpr std::uint64_t FORMULA_PROBE_MAX_COLLECTION = 64U;
constexpr std::uint64_t FORMULA_PROBE_MAX_DIMENSION = 4096U;

class CacheWriter {
public:
    explicit CacheWriter(const std::string& path)
        : stream_(path, std::ios::binary | std::ios::trunc) {
        if (!stream_) throw std::runtime_error(
            "cannot open formula probe cache for writing: " + path);
    }

    template <typename T>
    void pod(const T& value) {
        static_assert(std::is_trivially_copyable_v<T>);
        stream_.write(reinterpret_cast<const char*>(&value), sizeof(T));
        require();
    }

    void boolean(bool value) {
        pod(static_cast<std::uint8_t>(value ? 1U : 0U));
    }

    void size(std::size_t value) {
        pod(static_cast<std::uint64_t>(value));
    }

    void string(const std::string& value) {
        size(value.size());
        stream_.write(value.data(), static_cast<std::streamsize>(value.size()));
        require();
    }

    template <typename T>
    void pod_vector(const std::vector<T>& values) {
        static_assert(std::is_trivially_copyable_v<T>);
        size(values.size());
        if (!values.empty())
            stream_.write(reinterpret_cast<const char*>(values.data()),
                static_cast<std::streamsize>(values.size() * sizeof(T)));
        require();
    }

    void index_vector(const std::vector<std::size_t>& values) {
        size(values.size());
        for (std::size_t value : values) size(value);
    }

    void finish() {
        stream_.flush();
        require();
    }

private:
    void require() {
        if (!stream_) throw std::runtime_error(
            "failed while writing formula probe cache");
    }

    std::ofstream stream_;
};

class CacheReader {
public:
    explicit CacheReader(const std::string& path)
        : stream_(path, std::ios::binary) {
        if (!stream_) throw std::runtime_error(
            "cannot open formula probe cache for reading: " + path);
    }

    template <typename T>
    T pod() {
        static_assert(std::is_trivially_copyable_v<T>);
        T value{};
        stream_.read(reinterpret_cast<char*>(&value), sizeof(T));
        require();
        return value;
    }

    bool boolean() {
        const std::uint8_t value = pod<std::uint8_t>();
        if (value > 1U) throw std::runtime_error(
            "invalid boolean in formula probe cache");
        return value != 0U;
    }

    std::size_t size(std::uint64_t maximum = FORMULA_PROBE_MAX_VECTOR) {
        const std::uint64_t value = pod<std::uint64_t>();
        if (value > maximum
            || value > std::numeric_limits<std::size_t>::max())
            throw std::runtime_error("oversized formula probe cache field");
        return static_cast<std::size_t>(value);
    }

    std::string string() {
        std::string result(size(FORMULA_PROBE_MAX_STRING), '\0');
        if (!result.empty())
            stream_.read(result.data(),
                static_cast<std::streamsize>(result.size()));
        require();
        return result;
    }

    template <typename T>
    std::vector<T> pod_vector(
        std::uint64_t maximum = FORMULA_PROBE_MAX_VECTOR) {
        static_assert(std::is_trivially_copyable_v<T>);
        std::vector<T> result(size(maximum));
        if (!result.empty())
            stream_.read(reinterpret_cast<char*>(result.data()),
                static_cast<std::streamsize>(result.size() * sizeof(T)));
        require();
        return result;
    }

    std::vector<std::size_t> index_vector(
        std::uint64_t maximum = FORMULA_PROBE_MAX_VECTOR) {
        std::vector<std::size_t> result(size(maximum));
        for (std::size_t& value : result) value = size();
        return result;
    }

    void require_eof() {
        char trailing = 0;
        if (stream_.read(&trailing, 1)) throw std::runtime_error(
            "trailing bytes in formula probe cache");
        if (!stream_.eof()) throw std::runtime_error(
            "failed while finalizing formula probe cache read");
    }

private:
    void require() {
        if (!stream_) throw std::runtime_error(
            "truncated formula probe cache");
    }

    std::ifstream stream_;
};

void write_profile(CacheWriter& writer, const FormulaProbeProfile& profile) {
    writer.boolean(profile.exact);
    writer.boolean(profile.contained);
    writer.boolean(profile.contractive);
    writer.size(profile.width);
    writer.size(profile.dimension);
    writer.size(profile.vector_containments);
    writer.size(profile.matrix_containments);
    writer.size(profile.nonzero_vector_lows);
    writer.size(profile.nonzero_matrix_lows);
    writer.pod_vector(profile.vector_components);
    writer.pod_vector(profile.vector_radius);
    writer.pod_vector(profile.matrix_components);
    writer.pod_vector(profile.matrix_radius);
    writer.pod(profile.rho_upper);
    writer.pod(profile.maximum_vector_radius);
    writer.pod(profile.maximum_matrix_radius);
    writer.string(profile.exact_root);
    writer.string(profile.component_root);
    writer.string(profile.radius_root);
    writer.string(profile.root);
}

FormulaProbeProfile read_profile(CacheReader& reader) {
    FormulaProbeProfile profile;
    profile.exact = reader.boolean();
    profile.contained = reader.boolean();
    profile.contractive = reader.boolean();
    profile.width = reader.size();
    profile.dimension = reader.size();
    profile.vector_containments = reader.size();
    profile.matrix_containments = reader.size();
    profile.nonzero_vector_lows = reader.size();
    profile.nonzero_matrix_lows = reader.size();
    profile.vector_components = reader.pod_vector<double>();
    profile.vector_radius = reader.pod_vector<double>();
    profile.matrix_components = reader.pod_vector<double>();
    profile.matrix_radius = reader.pod_vector<double>();
    profile.rho_upper = reader.pod<double>();
    profile.maximum_vector_radius = reader.pod<double>();
    profile.maximum_matrix_radius = reader.pod<double>();
    profile.exact_root = reader.string();
    profile.component_root = reader.string();
    profile.radius_root = reader.string();
    profile.root = reader.string();
    return profile;
}

void write_certificate(
    CacheWriter& writer, const FormulaProbeCertificate& certificate) {
    writer.boolean(certificate.exact);
    writer.boolean(certificate.passed);
    writer.size(certificate.positive);
    writer.size(certificate.negative);
    writer.size(certificate.unresolved);
    writer.pod(certificate.error_upper);
    writer.string(certificate.solution_root);
    writer.string(certificate.sign_root);
    writer.string(certificate.root);
}

FormulaProbeCertificate read_certificate(CacheReader& reader) {
    FormulaProbeCertificate certificate;
    certificate.exact = reader.boolean();
    certificate.passed = reader.boolean();
    certificate.positive = reader.size();
    certificate.negative = reader.size();
    certificate.unresolved = reader.size();
    certificate.error_upper = reader.pod<double>();
    certificate.solution_root = reader.string();
    certificate.sign_root = reader.string();
    certificate.root = reader.string();
    return certificate;
}

void write_solutions(CacheWriter& writer,
    const std::vector<std::vector<FormulaProbeBinary128>>& solutions) {
    writer.size(solutions.size());
    for (const std::vector<FormulaProbeBinary128>& solution : solutions)
        writer.pod_vector(solution);
}

std::vector<std::vector<FormulaProbeBinary128>> read_solutions(
    CacheReader& reader) {
    std::vector<std::vector<FormulaProbeBinary128>> result(
        reader.size(FORMULA_PROBE_MAX_COLLECTION));
    for (std::vector<FormulaProbeBinary128>& solution : result)
        solution = reader.pod_vector<FormulaProbeBinary128>();
    return result;
}

void write_certificates(CacheWriter& writer,
    const std::vector<FormulaProbeCertificate>& certificates) {
    writer.size(certificates.size());
    for (const FormulaProbeCertificate& certificate : certificates)
        write_certificate(writer, certificate);
}

std::vector<FormulaProbeCertificate> read_certificates(
    CacheReader& reader) {
    std::vector<FormulaProbeCertificate> result(
        reader.size(FORMULA_PROBE_MAX_COLLECTION));
    for (FormulaProbeCertificate& certificate : result)
        certificate = read_certificate(reader);
    return result;
}

} // namespace

void write_formula_probe_parent_fixture(
    const std::string& path, const FormulaProbeParentFixture& fixture) {
    if (!formula_probe_parent_fixture_valid(fixture))
        throw std::runtime_error(
            "refusing to write invalid formula probe parent fixture");
    CacheWriter writer(path);
    for (char value : FORMULA_PROBE_CACHE_MAGIC) writer.pod(value);
    writer.pod(FORMULA_PROBE_CACHE_VERSION);
    writer.pod(FORMULA_PROBE_ENDIAN_MARKER);
    writer.pod(static_cast<std::uint32_t>(sizeof(double)));
    writer.pod(static_cast<std::uint32_t>(sizeof(FormulaProbeBinary128)));
    writer.boolean(fixture.exact);
    writer.string(fixture.schema);
    writer.size(fixture.dimension);
    writer.size(fixture.tangent_columns);
    writer.string(fixture.r63y_stdout_sha256);
    writer.string(fixture.r63z_stdout_sha256);
    writer.string(fixture.r63za_stdout_sha256);
    writer.string(fixture.r63zb_stdout_sha256);
    writer.string(fixture.r63zb_result_sha256);
    writer.string(fixture.r63x_semantic);
    writer.string(fixture.r63x_profile_root);
    writer.string(fixture.r63x_endpoint_root);
    writer.string(fixture.r63x_solution_set_root);
    write_solutions(writer, fixture.baseline_solutions);
    write_certificates(writer, fixture.baseline_certificates);
    writer.pod_vector(fixture.tangent);
    writer.pod(fixture.sigma);
    writer.string(fixture.tangent_root);
    writer.pod_vector(fixture.factor_upper);
    writer.index_vector(fixture.permutation);
    writer.pod(fixture.inverse_scale);
    writer.string(fixture.factor_fixture_root);
    writer.string(fixture.factor_root);
    writer.string(fixture.permutation_root);
    writer.pod_vector(fixture.original_rhs);
    writer.pod_vector(fixture.projected_rhs);
    writer.pod(fixture.projected_scale);
    writer.string(fixture.original_rhs_root);
    writer.string(fixture.projected_rhs_root);
    writer.pod_vector(fixture.common_components);
    writer.string(fixture.common_semantic);
    writer.string(fixture.common_component_root);
    writer.string(fixture.common_root);
    write_profile(writer, fixture.verifier_profile);
    writer.string(fixture.verifier_semantic);
    writer.string(fixture.verifier_profile_root);
    write_solutions(writer, fixture.common_projected_solutions);
    write_certificates(writer, fixture.common_projected_certificates);
    writer.string(fixture.finite_transaction_root);
    writer.string(fixture.common_projected_comparator_root);
    writer.string(fixture.root);
    writer.finish();
}

FormulaProbeParentFixture read_formula_probe_parent_fixture(
    const std::string& path) {
    CacheReader reader(path);
    for (char expected : FORMULA_PROBE_CACHE_MAGIC)
        if (reader.pod<char>() != expected)
            throw std::runtime_error("invalid formula probe cache magic");
    if (reader.pod<std::uint32_t>() != FORMULA_PROBE_CACHE_VERSION
        || reader.pod<std::uint32_t>() != FORMULA_PROBE_ENDIAN_MARKER
        || reader.pod<std::uint32_t>() != sizeof(double)
        || reader.pod<std::uint32_t>() != sizeof(FormulaProbeBinary128))
        throw std::runtime_error("incompatible formula probe cache profile");
    FormulaProbeParentFixture fixture;
    fixture.exact = reader.boolean();
    fixture.schema = reader.string();
    fixture.dimension = reader.size(FORMULA_PROBE_MAX_DIMENSION);
    fixture.tangent_columns = reader.size(FORMULA_PROBE_MAX_VECTOR);
    fixture.r63y_stdout_sha256 = reader.string();
    fixture.r63z_stdout_sha256 = reader.string();
    fixture.r63za_stdout_sha256 = reader.string();
    fixture.r63zb_stdout_sha256 = reader.string();
    fixture.r63zb_result_sha256 = reader.string();
    fixture.r63x_semantic = reader.string();
    fixture.r63x_profile_root = reader.string();
    fixture.r63x_endpoint_root = reader.string();
    fixture.r63x_solution_set_root = reader.string();
    fixture.baseline_solutions = read_solutions(reader);
    fixture.baseline_certificates = read_certificates(reader);
    fixture.tangent = reader.pod_vector<FormulaProbeBinary128>();
    fixture.sigma = reader.pod<FormulaProbeBinary128>();
    fixture.tangent_root = reader.string();
    fixture.factor_upper = reader.pod_vector<double>();
    fixture.permutation = reader.index_vector(FORMULA_PROBE_MAX_DIMENSION);
    fixture.inverse_scale = reader.pod<FormulaProbeBinary128>();
    fixture.factor_fixture_root = reader.string();
    fixture.factor_root = reader.string();
    fixture.permutation_root = reader.string();
    fixture.original_rhs = reader.pod_vector<FormulaProbeBinary128>();
    fixture.projected_rhs = reader.pod_vector<FormulaProbeBinary128>();
    fixture.projected_scale = reader.pod<FormulaProbeBinary128>();
    fixture.original_rhs_root = reader.string();
    fixture.projected_rhs_root = reader.string();
    fixture.common_components = reader.pod_vector<double>();
    fixture.common_semantic = reader.string();
    fixture.common_component_root = reader.string();
    fixture.common_root = reader.string();
    fixture.verifier_profile = read_profile(reader);
    fixture.verifier_semantic = reader.string();
    fixture.verifier_profile_root = reader.string();
    fixture.common_projected_solutions = read_solutions(reader);
    fixture.common_projected_certificates = read_certificates(reader);
    fixture.finite_transaction_root = reader.string();
    fixture.common_projected_comparator_root = reader.string();
    fixture.root = reader.string();
    reader.require_eof();
    if (!formula_probe_parent_fixture_valid(fixture))
        throw std::runtime_error("formula probe cache fixture validation failed");
    return fixture;
}

namespace {

class TangentBoundaryCacheReader {
public:
    TangentBoundaryCacheReader(const std::string& path,
        FormulaProbeTangentBoundaryReadWork& work)
        : stream_(path, std::ios::binary), work_(work) {
        if (!stream_) throw std::runtime_error(
            "cannot open formula probe cache for selective reading: " + path);
    }

    template <typename T>
    T pod() {
        static_assert(std::is_trivially_copyable_v<T>);
        T value{};
        stream_.read(reinterpret_cast<char*>(&value), sizeof(T));
        require();
        return value;
    }

    std::size_t size(std::uint64_t maximum = FORMULA_PROBE_MAX_VECTOR) {
        ++work_.bounded_length_reads;
        const std::uint64_t value = pod<std::uint64_t>();
        if (value > maximum
            || value > std::numeric_limits<std::size_t>::max())
            throw std::runtime_error(
                "oversized selective formula probe cache field");
        return static_cast<std::size_t>(value);
    }

    std::size_t selected_size(
        std::uint64_t maximum = FORMULA_PROBE_MAX_VECTOR) {
        ++work_.selected_size_fields;
        return size(maximum);
    }

    template <typename T>
    T selected_scalar() {
        ++work_.selected_scalar_fields;
        return pod<T>();
    }

    std::string selected_string() {
        const std::size_t count = size(FORMULA_PROBE_MAX_STRING);
        ++work_.selected_string_fields;
        work_.selected_string_bytes += count;
        std::string result(count, '\0');
        if (!result.empty()) {
            stream_.read(result.data(), static_cast<std::streamsize>(count));
            require();
        }
        return result;
    }

    template <typename T>
    std::vector<T> selected_vector(
        std::uint64_t maximum = FORMULA_PROBE_MAX_VECTOR) {
        static_assert(std::is_trivially_copyable_v<T>);
        const std::size_t count = size(maximum);
        ++work_.selected_vector_fields;
        ++work_.selected_vector_allocations;
        work_.selected_vector_components += count;
        std::vector<T> result(count);
        if (!result.empty()) {
            stream_.read(reinterpret_cast<char*>(result.data()),
                static_cast<std::streamsize>(count * sizeof(T)));
            require();
        }
        return result;
    }

    void skip_boolean() {
        ++work_.skipped_boolean_fields;
        const std::uint8_t value = pod<std::uint8_t>();
        if (value > 1U) throw std::runtime_error(
            "invalid skipped boolean in formula probe cache");
    }

    template <typename T>
    void skip_scalar() {
        ++work_.skipped_scalar_fields;
        static_cast<void>(pod<T>());
    }

    void skip_size(std::uint64_t maximum = FORMULA_PROBE_MAX_VECTOR) {
        ++work_.skipped_scalar_fields;
        static_cast<void>(size(maximum));
    }

    void skip_string() {
        const std::size_t count = size(FORMULA_PROBE_MAX_STRING);
        ++work_.skipped_string_fields;
        skip_bytes(count);
    }

    template <typename T>
    void skip_vector(std::uint64_t maximum = FORMULA_PROBE_MAX_VECTOR) {
        static_assert(std::is_trivially_copyable_v<T>);
        const std::size_t count = size(maximum);
        ++work_.skipped_vector_fields;
        if (count > std::numeric_limits<std::size_t>::max() / sizeof(T))
            throw std::runtime_error(
                "oversized selective formula probe cache vector");
        skip_bytes(count * sizeof(T));
    }

    void skip_index_vector() {
        const std::size_t count = size(FORMULA_PROBE_MAX_DIMENSION);
        ++work_.skipped_vector_fields;
        for (std::size_t index = 0U; index < count; ++index)
            skip_size(FORMULA_PROBE_MAX_VECTOR);
    }

    void require_eof() {
        ++work_.eof_checks;
        char trailing = 0;
        if (stream_.read(&trailing, 1)) throw std::runtime_error(
            "trailing bytes in selective formula probe cache");
        if (!stream_.eof()) throw std::runtime_error(
            "failed while finalizing selective formula probe cache read");
    }

private:
    void skip_bytes(std::size_t count) {
        std::array<char, 4096U> buffer{};
        work_.skipped_payload_bytes += count;
        while (count != 0U) {
            const std::size_t chunk = std::min(count, buffer.size());
            stream_.read(buffer.data(), static_cast<std::streamsize>(chunk));
            require();
            count -= chunk;
        }
    }

    void require() {
        if (!stream_) throw std::runtime_error(
            "truncated selective formula probe cache");
    }

    std::ifstream stream_;
    FormulaProbeTangentBoundaryReadWork& work_;
};

void skip_tangent_boundary_certificate(TangentBoundaryCacheReader& reader) {
    reader.skip_boolean();
    reader.skip_boolean();
    reader.skip_size();
    reader.skip_size();
    reader.skip_size();
    reader.skip_scalar<double>();
    reader.skip_string();
    reader.skip_string();
    reader.skip_string();
}

void skip_tangent_boundary_certificates(TangentBoundaryCacheReader& reader,
    FormulaProbeTangentBoundaryReadWork& work) {
    const std::size_t count = reader.size(FORMULA_PROBE_MAX_COLLECTION);
    work.skipped_collection_elements += count;
    for (std::size_t index = 0U; index < count; ++index)
        skip_tangent_boundary_certificate(reader);
}

void skip_tangent_boundary_profile(TangentBoundaryCacheReader& reader) {
    reader.skip_boolean();
    reader.skip_boolean();
    reader.skip_boolean();
    for (std::size_t index = 0U; index < 6U; ++index)
        reader.skip_size();
    reader.skip_vector<double>();
    reader.skip_vector<double>();
    reader.skip_vector<double>();
    reader.skip_vector<double>();
    reader.skip_scalar<double>();
    reader.skip_scalar<double>();
    reader.skip_scalar<double>();
    reader.skip_string();
    reader.skip_string();
    reader.skip_string();
    reader.skip_string();
}

std::string tangent_boundary_read_work_root(
    const FormulaProbeTangentBoundaryReadWork& work) {
    std::ostringstream material;
    material << work.header_predicate_checks << ':'
        << work.projection_predicate_checks << ':'
        << work.bounded_length_reads << ':' << work.selected_size_fields << ':'
        << work.selected_scalar_fields << ':' << work.selected_string_fields
        << ':' << work.selected_string_bytes << ':'
        << work.selected_vector_fields << ':'
        << work.selected_vector_allocations << ':'
        << work.selected_vector_components << ':'
        << work.skipped_boolean_fields << ':' << work.skipped_scalar_fields
        << ':' << work.skipped_string_fields << ':'
        << work.skipped_vector_fields << ':'
        << work.skipped_collection_elements << ':'
        << work.skipped_payload_bytes << ':' << work.eof_checks << ':'
        << work.receipt_root_derivations << ':'
        << work.result_root_derivations;
    return sha256_hex(material.str());
}

FormulaProbeTangentBoundaryRead finish_tangent_boundary_read(bool exact,
    const std::string& failure_stage,
    std::optional<FormulaProbeTangentBoundaryFixture> fixture,
    FormulaProbeTangentBoundaryReadWork work) {
    ++work.receipt_root_derivations;
    ++work.result_root_derivations;
    work.root = tangent_boundary_read_work_root(work);
    std::ostringstream material;
    material << exact << ':' << failure_stage << ':';
    if (fixture.has_value()) {
        material << fixture->dimension << ':' << fixture->tangent_columns << ':'
            << fixture->tangent_root << ':' << fixture->projected_rhs_root
            << ':' << fixture->original_rhs_root;
    }
    material << ':' << work.root;
    FormulaProbeTangentBoundaryRead result;
    result.exact = exact;
    result.failure_stage = failure_stage;
    result.fixture = std::move(fixture);
    result.work = std::move(work);
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

FormulaProbeTangentBoundaryRead read_formula_probe_tangent_boundary_fixture(
    const std::string& path) {
    FormulaProbeTangentBoundaryReadWork work;
    try {
        TangentBoundaryCacheReader reader(path, work);
        for (char expected : FORMULA_PROBE_CACHE_MAGIC) {
            ++work.header_predicate_checks;
            if (reader.pod<char>() != expected)
                return finish_tangent_boundary_read(false, "cache_magic",
                    std::nullopt, std::move(work));
        }
        ++work.header_predicate_checks;
        if (reader.pod<std::uint32_t>() != FORMULA_PROBE_CACHE_VERSION)
            return finish_tangent_boundary_read(false, "cache_version",
                std::nullopt, std::move(work));
        ++work.header_predicate_checks;
        if (reader.pod<std::uint32_t>() != FORMULA_PROBE_ENDIAN_MARKER)
            return finish_tangent_boundary_read(false, "cache_endian",
                std::nullopt, std::move(work));
        ++work.header_predicate_checks;
        if (reader.pod<std::uint32_t>() != sizeof(double))
            return finish_tangent_boundary_read(false, "cache_double_size",
                std::nullopt, std::move(work));
        ++work.header_predicate_checks;
        if (reader.pod<std::uint32_t>() != sizeof(FormulaProbeBinary128))
            return finish_tangent_boundary_read(false, "cache_binary128_size",
                std::nullopt, std::move(work));

        FormulaProbeTangentBoundaryFixture fixture;
        reader.skip_boolean();
        reader.skip_string();
        fixture.dimension = reader.selected_size(FORMULA_PROBE_MAX_DIMENSION);
        fixture.tangent_columns = reader.selected_size();
        for (std::size_t index = 0U; index < 9U; ++index)
            reader.skip_string();

        const std::size_t baseline_count =
            reader.size(FORMULA_PROBE_MAX_COLLECTION);
        ++work.projection_predicate_checks;
        if (baseline_count != 3U)
            return finish_tangent_boundary_read(false,
                "baseline_solution_count", std::nullopt, std::move(work));
        fixture.baseline_solution_0 =
            reader.selected_vector<FormulaProbeBinary128>();
        fixture.baseline_solution_1 =
            reader.selected_vector<FormulaProbeBinary128>();
        fixture.baseline_solution_2 =
            reader.selected_vector<FormulaProbeBinary128>();
        skip_tangent_boundary_certificates(reader, work);

        fixture.tangent = reader.selected_vector<FormulaProbeBinary128>();
        fixture.sigma = reader.selected_scalar<FormulaProbeBinary128>();
        fixture.tangent_root = reader.selected_string();
        reader.skip_vector<double>();
        reader.skip_index_vector();
        fixture.inverse_scale = reader.selected_scalar<FormulaProbeBinary128>();
        reader.skip_string();
        reader.skip_string();
        reader.skip_string();
        fixture.original_rhs =
            reader.selected_vector<FormulaProbeBinary128>();
        fixture.projected_rhs =
            reader.selected_vector<FormulaProbeBinary128>();
        fixture.projected_scale =
            reader.selected_scalar<FormulaProbeBinary128>();
        fixture.original_rhs_root = reader.selected_string();
        fixture.projected_rhs_root = reader.selected_string();
        reader.skip_vector<double>();
        reader.skip_string();
        reader.skip_string();
        reader.skip_string();
        skip_tangent_boundary_profile(reader);
        reader.skip_string();
        reader.skip_string();

        const std::size_t common_count =
            reader.size(FORMULA_PROBE_MAX_COLLECTION);
        ++work.projection_predicate_checks;
        if (common_count != 3U)
            return finish_tangent_boundary_read(false,
                "common_solution_count", std::nullopt, std::move(work));
        reader.skip_vector<FormulaProbeBinary128>();
        reader.skip_vector<FormulaProbeBinary128>();
        work.skipped_collection_elements += 2U;
        fixture.common_projected_solution_2 =
            reader.selected_vector<FormulaProbeBinary128>();
        skip_tangent_boundary_certificates(reader, work);
        reader.skip_string();
        reader.skip_string();
        reader.skip_string();
        reader.require_eof();
        return finish_tangent_boundary_read(true, "", std::move(fixture),
            std::move(work));
    } catch (const std::exception&) {
        return finish_tangent_boundary_read(false, "cache_decode",
            std::nullopt, std::move(work));
    }
}

} // namespace nextengine::nonlocal::fcr
