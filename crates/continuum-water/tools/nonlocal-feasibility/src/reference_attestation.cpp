#include "reference_attestation.hpp"

#include "sha256.hpp"

#include <array>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <limits>
#include <sstream>
#include <string>
#include <string_view>
#include <system_error>
#include <utility>

namespace nextengine::nonlocal::fcr {
namespace {

constexpr std::size_t MAXIMUM_REFERENCE_BYTES = 33'554'432U;
constexpr std::uint32_t SAMPLE_COUNT = 6'000U;
constexpr std::size_t HEADER_BYTES = 48U;
constexpr std::size_t BYTES_PER_SAMPLE = 24U;
constexpr std::size_t STEP_BYTES = 4U;
constexpr std::string_view MAGIC{"CWREFV1\0", 8U};

constexpr std::string_view IDENTITY_PROJECTION =
    "nextengine.nonlocal.nsr3b4d-reference-reattestation|v1|"
    "formula=nuv-variational-fcr2|boundary=split-static-boundary-r0|"
    "solver=nuv-newton-krylov-r0+outer-state-hessian-tape-v1|"
    "canonical=balanced-macro-publication|"
    "packaging=complete-lane-flat-adjacency-candidate|"
    "w0i=186e1e31c0aa2636525bbc54e4fe4335b8432e7e99eddf3221d08e0380b65c90|"
    "file_count=3|max_bytes=33554432|"
    "mutation=memory-byte-flip-before-trajectory";
constexpr std::string_view IDENTITY_SHA256 =
    "47c78bdb115c0e5d7ed7132a6de3e62e3ba9533396a7f346b510b354dc5222fe";
constexpr std::string_view W0I_ATTESTATION_ROOT =
    "186e1e31c0aa2636525bbc54e4fe4335b8432e7e99eddf3221d08e0380b65c90";

struct ReferenceSpec {
    const char* scenario;
    const char* path;
    std::uint32_t output_count;
    std::uint32_t final_step;
    std::uint32_t output_stride;
    std::size_t byte_count;
    const char* sha256;
    const char* scenario_root;
};

constexpr std::array<ReferenceSpec, 3> REFERENCES = {{
    {
        "CW-HYDRO-001",
        "/tmp/cwref-hydro-hard-contact-final.bin",
        51U,
        1'200U,
        24U,
        7'344'252U,
        "84ae867f5b336cd0bd51be6f29a6a2a1f27f702c424f1dbd0a8f735b9f4bb435",
        "030357596535f052469e1cc2729859106ba58baadaf49ec2d84eceff46521e63",
    },
    {
        "CW-DAMBREAK-001",
        "/tmp/cwref-dam-hard-contact-final.bin",
        181U,
        720U,
        4U,
        26'064'772U,
        "853d965489a40082a024aeee5a19f98aef054417014af8556fa212687d88d12c",
        "fb779545fd4429e74158d773b1aa51bd12af55c5d1b821251c9016bfae97f94c",
    },
    {
        "CW-ORIFICE-001",
        "/tmp/cwref-orifice-hard-contact-final.bin",
        181U,
        720U,
        4U,
        26'064'772U,
        "60e9b3538d621ef1a3f1ae77569640740df471fbe4e5ef8eaa813d3751930849",
        "13ab66fbbe7b167f3e69f8ddd0e3887120a9f586b57ff426cb240a8e66f991e1",
    },
}};

struct FileResult {
    const ReferenceSpec* spec = nullptr;
    std::string status;
    std::string failure;
    std::size_t observed_bytes = 0U;
    std::string observed_sha256;
    bool regular_file = false;
    bool capacity_admitted = false;
    bool read_exact = false;
    bool magic_exact = false;
    bool scenario_root_exact = false;
    bool output_count_exact = false;
    bool sample_count_exact = false;
    bool step_cadence_exact = false;
    bool total_size_exact = false;
    bool hash_exact = false;
    bool mutation_rejected = false;
    bool passed = false;
};

std::uint32_t read_u32_le(std::string_view bytes, std::size_t offset) {
    if (offset > bytes.size() || bytes.size() - offset < 4U) {
        return std::numeric_limits<std::uint32_t>::max();
    }
    return static_cast<std::uint32_t>(
        static_cast<std::uint8_t>(bytes[offset]))
        | (static_cast<std::uint32_t>(
               static_cast<std::uint8_t>(bytes[offset + 1U]))
           << 8U)
        | (static_cast<std::uint32_t>(
               static_cast<std::uint8_t>(bytes[offset + 2U]))
           << 16U)
        | (static_cast<std::uint32_t>(
               static_cast<std::uint8_t>(bytes[offset + 3U]))
           << 24U);
}

bool root_matches(std::string_view bytes, std::string_view expected_hex) {
    if (bytes.size() < 40U || expected_hex.size() != 64U) {
        return false;
    }
    constexpr std::string_view HEX = "0123456789abcdef";
    for (std::size_t index = 0U; index < 32U; ++index) {
        const auto high = HEX.find(expected_hex[2U * index]);
        const auto low = HEX.find(expected_hex[2U * index + 1U]);
        if (high == std::string_view::npos || low == std::string_view::npos) {
            return false;
        }
        const auto expected = static_cast<std::uint8_t>((high << 4U) | low);
        if (static_cast<std::uint8_t>(bytes[8U + index]) != expected) {
            return false;
        }
    }
    return true;
}

void set_failure(FileResult& result, std::string failure) {
    if (result.failure.empty()) {
        result.failure = std::move(failure);
    }
}

FileResult inspect(const ReferenceSpec& spec) {
    FileResult result;
    result.spec = &spec;

    std::error_code error;
    const std::filesystem::file_status status =
        std::filesystem::status(spec.path, error);
    if (error || !std::filesystem::exists(status)) {
        result.status = "MISSING_ARTIFACT";
        result.failure = "MISSING_ARTIFACT";
        return result;
    }
    result.regular_file = std::filesystem::is_regular_file(status);
    if (!result.regular_file) {
        result.status = "FAIL";
        result.failure = "NOT_REGULAR_FILE";
        return result;
    }

    const std::uintmax_t size = std::filesystem::file_size(spec.path, error);
    if (error || size > std::numeric_limits<std::size_t>::max()) {
        result.status = "FAIL";
        result.failure = "SIZE_PREFLIGHT_FAILED";
        return result;
    }
    result.observed_bytes = static_cast<std::size_t>(size);
    result.capacity_admitted = result.observed_bytes <= MAXIMUM_REFERENCE_BYTES;
    if (!result.capacity_admitted) {
        result.status = "FAIL";
        result.failure = "REFERENCE_INPUT_CAPACITY_EXCEEDED";
        return result;
    }

    std::ifstream input(spec.path, std::ios::binary);
    if (!input) {
        result.status = "FAIL";
        result.failure = "UNREADABLE_ARTIFACT";
        return result;
    }
    std::string bytes(result.observed_bytes, '\0');
    if (!bytes.empty()) {
        input.read(bytes.data(), static_cast<std::streamsize>(bytes.size()));
    }
    const std::uintmax_t size_after =
        std::filesystem::file_size(spec.path, error);
    result.read_exact = !error
        && static_cast<std::size_t>(input.gcount()) == bytes.size()
        && size_after == size;
    if (!result.read_exact) {
        result.status = "FAIL";
        result.failure = "SIZE_CHANGED_DURING_READ";
        return result;
    }

    const std::string_view view(bytes.data(), bytes.size());
    result.magic_exact = view.size() >= MAGIC.size()
        && view.substr(0U, MAGIC.size()) == MAGIC;
    result.scenario_root_exact = root_matches(view, spec.scenario_root);
    result.output_count_exact = read_u32_le(view, 40U) == spec.output_count;
    result.sample_count_exact = read_u32_le(view, 44U) == SAMPLE_COUNT;

    const std::size_t payload_bytes =
        static_cast<std::size_t>(SAMPLE_COUNT) * BYTES_PER_SAMPLE;
    const std::size_t record_bytes = STEP_BYTES + payload_bytes;
    const std::size_t expected_bytes = HEADER_BYTES
        + static_cast<std::size_t>(spec.output_count) * record_bytes;
    result.total_size_exact = expected_bytes == spec.byte_count
        && result.observed_bytes == spec.byte_count;

    result.step_cadence_exact = result.total_size_exact;
    std::size_t cursor = HEADER_BYTES;
    for (std::uint32_t output = 0U;
         result.step_cadence_exact && output < spec.output_count; ++output) {
        const std::uint32_t expected_step = output * spec.output_stride;
        result.step_cadence_exact = read_u32_le(view, cursor) == expected_step;
        cursor += record_bytes;
    }
    result.step_cadence_exact = result.step_cadence_exact
        && cursor == view.size()
        && (spec.output_count - 1U) * spec.output_stride == spec.final_step;

    result.observed_sha256 = nextengine::nonlocal::sha256_hex(view);
    result.hash_exact = result.observed_sha256 == spec.sha256;

    if (!bytes.empty()) {
        bytes.back() = static_cast<char>(
            static_cast<unsigned char>(bytes.back()) ^ 0x01U);
        result.mutation_rejected =
            nextengine::nonlocal::sha256_hex(
                std::string_view(bytes.data(), bytes.size()))
            != spec.sha256;
    }

    if (!result.magic_exact) {
        set_failure(result, "MAGIC_MISMATCH");
    }
    if (!result.scenario_root_exact) {
        set_failure(result, "SCENARIO_ROOT_MISMATCH");
    }
    if (!result.output_count_exact) {
        set_failure(result, "OUTPUT_COUNT_MISMATCH");
    }
    if (!result.sample_count_exact) {
        set_failure(result, "SAMPLE_COUNT_MISMATCH");
    }
    if (!result.step_cadence_exact) {
        set_failure(result, "STEP_CADENCE_MISMATCH");
    }
    if (!result.total_size_exact) {
        set_failure(result, "TOTAL_SIZE_MISMATCH");
    }
    if (!result.hash_exact) {
        set_failure(result, "COMPLETE_FILE_HASH_MISMATCH");
    }
    if (!result.mutation_rejected) {
        set_failure(result, "MUTATION_NOT_REJECTED");
    }

    result.passed = result.regular_file && result.capacity_admitted
        && result.read_exact && result.magic_exact
        && result.scenario_root_exact && result.output_count_exact
        && result.sample_count_exact && result.step_cadence_exact
        && result.total_size_exact && result.hash_exact
        && result.mutation_rejected;
    result.status = result.passed ? "PASS" : "FAIL";
    return result;
}

void append_bool(std::ostringstream& output, bool value) {
    output << (value ? "true" : "false");
}

void append_file(std::ostringstream& output, const FileResult& value) {
    output << "{\"scenario\":\"" << value.spec->scenario
           << "\",\"path\":\"" << value.spec->path
           << "\",\"status\":\"" << value.status
           << "\",\"failure\":\"" << value.failure
           << "\",\"expected_bytes\":" << value.spec->byte_count
           << ",\"observed_bytes\":" << value.observed_bytes
           << ",\"expected_sha256\":\"" << value.spec->sha256
           << "\",\"observed_sha256\":\"" << value.observed_sha256
           << "\",\"regular_file\":";
    append_bool(output, value.regular_file);
    output << ",\"capacity_admitted\":";
    append_bool(output, value.capacity_admitted);
    output << ",\"read_exact\":";
    append_bool(output, value.read_exact);
    output << ",\"magic_exact\":";
    append_bool(output, value.magic_exact);
    output << ",\"scenario_root_exact\":";
    append_bool(output, value.scenario_root_exact);
    output << ",\"output_count_exact\":";
    append_bool(output, value.output_count_exact);
    output << ",\"sample_count_exact\":";
    append_bool(output, value.sample_count_exact);
    output << ",\"step_cadence_exact\":";
    append_bool(output, value.step_cadence_exact);
    output << ",\"total_size_exact\":";
    append_bool(output, value.total_size_exact);
    output << ",\"hash_exact\":";
    append_bool(output, value.hash_exact);
    output << ",\"mutation_rejected\":";
    append_bool(output, value.mutation_rejected);
    output << '}';
}

} // namespace

ReferenceAttestationReport run_reference_attestation_controls() {
    const std::string observed_identity =
        nextengine::nonlocal::sha256_hex(IDENTITY_PROJECTION);
    const bool parent_identity_exact = observed_identity == IDENTITY_SHA256;

    std::array<FileResult, REFERENCES.size()> results{};
    bool all_files_passed = true;
    bool missing_artifact = false;
    std::string file_failure;
    for (std::size_t index = 0U; index < REFERENCES.size(); ++index) {
        results[index] = inspect(REFERENCES[index]);
        all_files_passed = all_files_passed && results[index].passed;
        missing_artifact = missing_artifact
            || results[index].status == "MISSING_ARTIFACT";
        if (file_failure.empty() && !results[index].passed) {
            file_failure = std::string(REFERENCES[index].scenario) + ':'
                + results[index].failure;
        }
    }
    const std::string first_failure = parent_identity_exact
        ? file_failure
        : "PARENT_IDENTITY_MISMATCH";

    ReferenceAttestationReport report;
    report.passed = parent_identity_exact && all_files_passed;
    const char* status = report.passed ? "PASS"
        : (!parent_identity_exact ? "FAIL"
                                  : (missing_artifact ? "MISSING_ARTIFACT"
                                                      : "FAIL"));

    std::ostringstream output;
    output << "{\"schema\":\"nextengine.nonlocal."
              "nsr3b4d_reference_reattestation.v1\""
           << ",\"identity_sha256\":\"" << IDENTITY_SHA256
           << "\",\"observed_identity_sha256\":\"" << observed_identity
           << "\",\"parent_identity_exact\":";
    append_bool(output, parent_identity_exact);
    output << ",\"w0i_attestation_root\":\"" << W0I_ATTESTATION_ROOT
           << "\",\"status\":\"" << status
           << "\",\"first_failure\":\"" << first_failure
           << "\",\"files\":[";
    for (std::size_t index = 0U; index < results.size(); ++index) {
        if (index != 0U) {
            output << ',';
        }
        append_file(output, results[index]);
    }
    output << "],\"trajectory_started\":false"
           << ",\"external_reference_candidate_selected\":";
    append_bool(output, report.passed);
    output << ",\"b4e_design_authorized\":";
    append_bool(output, report.passed);
    output << ",\"nominal_corpus_execution_authorized\":false"
           << ",\"runtime_authority\":false"
           << ",\"production_authority\":false}";
    report.json = output.str();
    return report;
}

} // namespace nextengine::nonlocal::fcr
