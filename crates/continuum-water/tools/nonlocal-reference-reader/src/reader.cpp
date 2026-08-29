#include "reader.hpp"

#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cerrno>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <fcntl.h>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <sys/stat.h>
#include <unistd.h>
#include <utility>
#include <vector>

namespace nextengine::nonlocal_reference_reader {
namespace {

constexpr std::string_view SCHEMA =
    "nextengine.nonlocal.nsr3b4dr1e-reference-attestation.v1";
constexpr std::string_view PROFILE_SCHEMA =
    "nextengine.nonlocal.nsr3b4dr1e-profile-self-test.v1";
constexpr std::string_view IDENTITY =
    "9cf5fc571fee7bc0be5585d9b467f90cd8a27f40d9cf39d6be999b0c466cbccc";
constexpr std::string_view R1D_PROFILE =
    "ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746";
constexpr std::string_view GENERATOR_SOURCE_ROOT =
    "1c23acc5680c724447a2353cbf68f85facafc9042ba65957663e05f00a2433ce";
constexpr std::string_view GENERATOR_BUILD_ROOT =
    "b8d5a5be6891ba2e6c449c4820c61d432a299ae236e04348a00af3f5103cf9db";
constexpr std::string_view PAYLOAD_PROFILE_ROOT =
    "71bed406888d2516b567f44761fb77bb4d50a3349e2ebd2f754db45583ec8f2c";
constexpr std::string_view AGGREGATE_PROFILE_ROOT =
    "2a992210cd745f859636a3032309ec587081e42e54f6fac55626c4696e5cebff";
constexpr std::string_view IDENTITY_PROJECTION =
    "nextengine.nonlocal.nsr3b4dr1e-reference-attestation|v1|"
    "parent=ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746|"
    "generator-source=1c23acc5680c724447a2353cbf68f85facafc9042ba65957663e05f00a2433ce|"
    "generator-build=b8d5a5be6891ba2e6c449c4820c61d432a299ae236e04348a00af3f5103cf9db|"
    "payload-profile=71bed406888d2516b567f44761fb77bb4d50a3349e2ebd2f754db45583ec8f2c|"
    "aggregate-profile=2a992210cd745f859636a3032309ec587081e42e54f6fac55626c4696e5cebff|"
    "reader=independent-cxx17;openat-nofollow;max67108864;"
    "cwrefv2-full-parse;canonical-reencode|"
    "mutations=file-final-byte-xor1;decoded-first-x-xor1|runs=2-byte-exact|"
    "credit=new-external-dfsph-reference-candidate";
constexpr std::string_view GENERATOR_BUILD_PROJECTION =
    "nextengine.nonlocal.nsr3b4dr1e-generator-build.v1|"
    "source=1c23acc5680c724447a2353cbf68f85facafc9042ba65957663e05f00a2433ce|"
    "commit=fbbd6eb2d0f53e049e4098d0b57bb50b31f35136|"
    "upstream=eccce86155776f6ac52d5080b1f720a52bf29450|"
    "upstream-lib=172e6777027564566d6282f8679f4d93cbd4cdc193fc236eb9fdaa210ea07d20|"
    "patch=e89cf9befc2a08a9c15bd6b290a97b3f6815da1ea699508c41c15645f86c33bc|"
    "compiler=g++-15.2.0|cmake=4.2.3|generator=ninja-1.13.2|mode=Release|"
    "cxx=17|float=binary64,avx-off,fma-off,nearest,ftz-off,fast-math-off|"
    "omp=1,dynamic-false|"
    "binary=1595984:8ba20fc747ecd5a098e978469c67209cc915e897047629271931a5396eb49737";
constexpr std::string_view PAYLOAD_PROFILE_PROJECTION =
    "nextengine.nonlocal.nsr3b4dr1e-payload-profile.v1|"
    "hydro=15920965:6d6b70c3feb6fd583d8740a901cc2577de7e3744d4b6c8dbd76a73d2a2710c2f:"
    "c430b679dfeec33a6ac12c51df75ddee7e7bc48484c6c05188219f0a727909a0:"
    "b5a831e370e25d5e8c1370df3ebe2fa4997b708b04a668b7d6e1ee74c496f0f6|"
    "dam=56501239:a0b030805034538420e0a5d90390f7117e645119013e9223ff1b334564cbcc13:"
    "8d0a0a85adba50d4784245d460c83757bcce92841d804f4739b81faf27fd5f09:"
    "17fdc16b4348eb8dc27c513d56a5eaf5e3376b1cca60e6b1e2751526c259d2f5|"
    "orifice=56501341:5c16d4dccd351a8dab0bf613a80702a8a5a3da1517371e1bb751cb8485cbc626:"
    "53d0db457091d7a7d6ede58a0628688b701850d6de30dacdb14ee9951d036961:"
    "50d18e45b3622cc92591637b7ad14b7aefd8dd1b87ea33a90a08dd9d96c47079";
constexpr std::string_view AGGREGATE_PROFILE_PROJECTION =
    "nextengine.nonlocal.nsr3b4dr1e-aggregate-profile.v1|"
    "hydro=6c5ce1fb89d335657d91922b513b29d0903d09f585e8815b6e37a289f3add73e:"
    "b65d550c8bcb0052f249b9cbde2d4fefe175bb690466560e8796db5682ddaf7a:"
    "7253b247811382836c7c7678f0e3df2e66dca1248f6b4ac04ce3758e144de498|"
    "dam=1a4fce89db7c09dcc28dbb2eb46abc03fceb4b2a0658d687bcd181ff96f600c8:"
    "16157e8fcb1ab8acad53cbe369fb2917940bde1303d01c0e0709af9e920b0158:"
    "893f43e62fb8adbb2627302d61a4a153cffc9d7616568e460012823f4fa8afbe|"
    "orifice=b6c06e1f68b3bc467b5bdf738b2b8034df87b7b206299ec3cd31ecb397a0af3d:"
    "d4d0ade221dd14b776076e4364f52319cc89ea3e844e0ccd09236e75c7990344:"
    "ff07ee7f84d50cf98745c1d3ec0aca9b79c7494bf9138797da1e63006761466e";
constexpr std::size_t MAXIMUM_BYTES = 64U * 1024U * 1024U;
constexpr std::uint32_t SAMPLE_COUNT = 6'000U;
constexpr std::size_t FEATURE_COUNT = 25U;
constexpr std::size_t FRAME_BYTES = 312'156U;
constexpr std::string_view MAGIC{"CWREFV2\0", 8U};
constexpr std::string_view R1D_PROFILE_PROJECTION =
    "nextengine.nonlocal.nsr3b4dr1d-full-generation|v1|"
    "parent=3f5a73693f05daf487898fd692160357c1775652a1c11e65205356243c57386f|"
    "schedule=hydro:0..1200/every24;dam:0..720/every4;orifice:0..720/every4|"
    "format=CWREFV2;frame=unchanged|"
    "summary=nearest-rank-q99-x-f64bits;nearest-rank-q99-y-f64bits;"
    "receiver-u32;domain-separated-sha256|"
    "publication=external-content-addressed;regular-file;no-tmp;"
    "verified-copy-only|runs=2-fresh-byte-exact-per-scenario|"
    "parallel=max3;omp1|report-order=hydro,dam,orifice|pressure-max=300|"
    "all-else=r1c5|credit=new-root-only";

constexpr std::string_view HYDRO_MANIFEST = R"(B4DR1D_SCENARIO_V1_BEGIN
scenario_id=CW-HYDRO-001
kind=hydrostatic-cube
box_um=0,0,0;1000000,1000000,1000000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=7d4e661d08de08b18d43a76342329b51f6ae98bca9baee3d850e0f403eae5606
boundary=two-layer-outer-complement
boundary_count=5824
boundary_root=25de85b5eeec041c12bbb5de10e00b8374457b4cf09cb61d99dfc4d5511d8d62
steps=1200
outputs=0..1200/every=24
B4DR1D_SCENARIO_V1_END
)";
constexpr std::string_view DAM_MANIFEST = R"(B4DR1D_SCENARIO_V1_BEGIN
scenario_id=CW-DAMBREAK-001
kind=dam-break
box_um=0,0,0;4000000,1000000,1000000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76
boundary=two-layer-outer-complement
boundary_count=16384
boundary_root=1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d
steps=720
outputs=0..720/every=4
B4DR1D_SCENARIO_V1_END
)";
constexpr std::string_view ORIFICE_MANIFEST = R"(B4DR1D_SCENARIO_V1_BEGIN
scenario_id=CW-ORIFICE-001
kind=orifice-release
box_um=0,0,0;2000000,1000000,1000000
wall_um=x=1000000;opening_y=200000..400000;opening_z=400000..600000;radius=25000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=21307ab2d1655ab5da33f3b5d4e887152601ed531679723b8452023df1f48425
boundary=source-chamber-two-layer-minus-safe-opening
boundary_count=5792
boundary_root=5d23bd8c407c1b1d6bb86fc6dda2250c36db4cde1227d9f48fc6239c728a2cb3
steps=720
outputs=0..720/every=4
B4DR1D_SCENARIO_V1_END
)";

struct ReferenceSpec {
    std::string_view scenario;
    std::string_view manifest;
    std::string_view manifest_root;
    std::size_t bytes;
    std::string_view file_sha256;
    std::uint32_t total_steps;
    std::uint32_t stride;
    std::uint32_t frames;
    std::string_view semantic_root;
    std::string_view q99_x_root;
    std::string_view q99_y_root;
    std::string_view receiver_root;
    std::uint32_t final_receiver_count;
};

constexpr std::array<ReferenceSpec, 3> REFERENCES = {{
    {
        "CW-HYDRO-001",
        HYDRO_MANIFEST,
        "c430b679dfeec33a6ac12c51df75ddee7e7bc48484c6c05188219f0a727909a0",
        15'920'965U,
        "6d6b70c3feb6fd583d8740a901cc2577de7e3744d4b6c8dbd76a73d2a2710c2f",
        1'200U,
        24U,
        51U,
        "b5a831e370e25d5e8c1370df3ebe2fa4997b708b04a668b7d6e1ee74c496f0f6",
        "6c5ce1fb89d335657d91922b513b29d0903d09f585e8815b6e37a289f3add73e",
        "b65d550c8bcb0052f249b9cbde2d4fefe175bb690466560e8796db5682ddaf7a",
        "7253b247811382836c7c7678f0e3df2e66dca1248f6b4ac04ce3758e144de498",
        0U,
    },
    {
        "CW-DAMBREAK-001",
        DAM_MANIFEST,
        "8d0a0a85adba50d4784245d460c83757bcce92841d804f4739b81faf27fd5f09",
        56'501'239U,
        "a0b030805034538420e0a5d90390f7117e645119013e9223ff1b334564cbcc13",
        720U,
        4U,
        181U,
        "17fdc16b4348eb8dc27c513d56a5eaf5e3376b1cca60e6b1e2751526c259d2f5",
        "1a4fce89db7c09dcc28dbb2eb46abc03fceb4b2a0658d687bcd181ff96f600c8",
        "16157e8fcb1ab8acad53cbe369fb2917940bde1303d01c0e0709af9e920b0158",
        "893f43e62fb8adbb2627302d61a4a153cffc9d7616568e460012823f4fa8afbe",
        0U,
    },
    {
        "CW-ORIFICE-001",
        ORIFICE_MANIFEST,
        "53d0db457091d7a7d6ede58a0628688b701850d6de30dacdb14ee9951d036961",
        56'501'341U,
        "5c16d4dccd351a8dab0bf613a80702a8a5a3da1517371e1bb751cb8485cbc626",
        720U,
        4U,
        181U,
        "50d18e45b3622cc92591637b7ad14b7aefd8dd1b87ea33a90a08dd9d96c47079",
        "b6c06e1f68b3bc467b5bdf738b2b8034df87b7b206299ec3cd31ecb397a0af3d",
        "d4d0ade221dd14b776076e4364f52319cc89ea3e844e0ccd09236e75c7990344",
        "ff07ee7f84d50cf98745c1d3ec0aca9b79c7494bf9138797da1e63006761466e",
        1'172U,
    },
}};

class Descriptor {
public:
    Descriptor() = default;
    explicit Descriptor(int value) : value_(value) {}
    Descriptor(const Descriptor &) = delete;
    Descriptor &operator=(const Descriptor &) = delete;
    Descriptor(Descriptor &&other) noexcept : value_(other.value_) {
        other.value_ = -1;
    }
    Descriptor &operator=(Descriptor &&other) noexcept {
        if (this != &other) {
            reset();
            value_ = other.value_;
            other.value_ = -1;
        }
        return *this;
    }
    ~Descriptor() { reset(); }
    int get() const { return value_; }
    explicit operator bool() const { return value_ >= 0; }

private:
    void reset() {
        if (value_ >= 0) {
            ::close(value_);
            value_ = -1;
        }
    }
    int value_ = -1;
};

Descriptor open_absolute_directory(std::string_view path) {
    if (path.empty() || path.front() != '/') {
        return Descriptor();
    }
    Descriptor current(::open("/", O_RDONLY | O_DIRECTORY | O_CLOEXEC));
    if (!current) {
        return Descriptor();
    }
    std::size_t cursor = 1U;
    while (cursor < path.size()) {
        while (cursor < path.size() && path[cursor] == '/') {
            ++cursor;
        }
        if (cursor == path.size()) {
            break;
        }
        const std::size_t end = path.find('/', cursor);
        const std::size_t component_end =
            end == std::string_view::npos ? path.size() : end;
        const std::string_view component = path.substr(
            cursor, component_end - cursor);
        if (component.empty() || component == "." || component == "..") {
            errno = EINVAL;
            return Descriptor();
        }
        const std::string owned_component(component);
        Descriptor next(::openat(
            current.get(),
            owned_component.c_str(),
            O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW));
        if (!next) {
            return Descriptor();
        }
        current = std::move(next);
        cursor = component_end;
    }
    return current;
}

struct FileResult {
    const ReferenceSpec *spec = nullptr;
    std::string status = "FAIL";
    std::string failure;
    bool path_admitted = false;
    bool regular_file = false;
    bool capacity_admitted = false;
    bool read_exact = false;
    bool file_hash_exact = false;
    bool manifest_exact = false;
    bool parse_exact = false;
    bool semantic_root_exact = false;
    bool aggregates_exact = false;
    bool serialized_mutation_rejected = false;
    bool decoded_mutation_rejected = false;
    std::string observed_sha256;
    std::string observed_semantic_root;
    std::string observed_q99_x_root;
    std::string observed_q99_y_root;
    std::string observed_receiver_root;
    bool passed = false;
};

void append_u32(std::vector<std::uint8_t> &output, std::uint32_t value) {
    for (unsigned int shift = 0U; shift < 32U; shift += 8U) {
        output.push_back(static_cast<std::uint8_t>(value >> shift));
    }
}

void append_u64(std::vector<std::uint8_t> &output, std::uint64_t value) {
    for (unsigned int shift = 0U; shift < 64U; shift += 8U) {
        output.push_back(static_cast<std::uint8_t>(value >> shift));
    }
}

std::string bytes_hash(const std::vector<std::uint8_t> &bytes) {
    return nextengine::nonlocal::sha256_hex(std::string_view(
        reinterpret_cast<const char *>(bytes.data()), bytes.size()));
}

std::string domain_hash(
    std::string_view domain,
    const std::vector<std::uint8_t> &bytes) {
    std::string projection;
    projection.reserve(domain.size() + 1U + bytes.size());
    projection.append(domain);
    projection.push_back('\0');
    projection.append(
        reinterpret_cast<const char *>(bytes.data()), bytes.size());
    return nextengine::nonlocal::sha256_hex(projection);
}

std::string scenario_root(std::string_view manifest) {
    std::string projection("nextengine.nonlocal.nsr3b4dr1d-scenario.v1");
    projection.push_back('\0');
    projection.append(manifest);
    return nextengine::nonlocal::sha256_hex(projection);
}

double from_bits(std::uint64_t bits) {
    double value = 0.0;
    static_assert(sizeof(value) == sizeof(bits));
    std::memcpy(&value, &bits, sizeof(value));
    return value;
}

class Parser {
public:
    Parser(const std::vector<std::uint8_t> &input, std::vector<std::uint8_t> &decoded)
        : input_(input), decoded_(decoded) {}

    std::uint32_t u32(bool emit = true) {
        require(4U);
        std::uint32_t value = 0U;
        for (unsigned int shift = 0U; shift < 32U; shift += 8U) {
            value |= static_cast<std::uint32_t>(input_[cursor_++]) << shift;
        }
        if (emit) {
            append_u32(decoded_, value);
        }
        return value;
    }

    std::uint64_t u64(bool emit = true) {
        require(8U);
        std::uint64_t value = 0U;
        for (unsigned int shift = 0U; shift < 64U; shift += 8U) {
            value |= static_cast<std::uint64_t>(input_[cursor_++]) << shift;
        }
        if (emit) {
            append_u64(decoded_, value);
        }
        return value;
    }

    double finite_f64() {
        const double value = from_bits(u64());
        if (!std::isfinite(value)) {
            throw std::runtime_error("NONFINITE_BINARY64");
        }
        return value;
    }

    std::string_view raw(std::size_t count) {
        require(count);
        const char *begin = reinterpret_cast<const char *>(input_.data() + cursor_);
        cursor_ += count;
        return std::string_view(begin, count);
    }

    std::size_t cursor() const { return cursor_; }
    std::size_t decoded_size() const { return decoded_.size(); }

private:
    void require(std::size_t count) const {
        if (cursor_ > input_.size() || input_.size() - cursor_ < count) {
            throw std::runtime_error("TRUNCATED_PAYLOAD");
        }
    }

    const std::vector<std::uint8_t> &input_;
    std::vector<std::uint8_t> &decoded_;
    std::size_t cursor_ = 0U;
};

double nearest_rank_q99(std::vector<double> values) {
    if (values.size() != SAMPLE_COUNT) {
        throw std::runtime_error("Q99_SAMPLE_COUNT_MISMATCH");
    }
    std::sort(values.begin(), values.end());
    const std::size_t rank = ((99U * values.size()) + 99U) / 100U;
    if (rank == 0U || rank > values.size()) {
        throw std::runtime_error("Q99_RANK_OUT_OF_RANGE");
    }
    return values[rank - 1U];
}

std::vector<std::uint8_t> aggregate_prefix(
    std::string_view domain,
    std::uint32_t frames) {
    std::vector<std::uint8_t> result(domain.begin(), domain.end());
    result.push_back(0U);
    append_u32(result, SAMPLE_COUNT);
    append_u32(result, frames);
    return result;
}

std::string expected_manifest(const ReferenceSpec &spec) {
    std::string result(R1D_PROFILE_PROJECTION);
    result.push_back('\n');
    result.append(spec.manifest);
    return result;
}

bool same_stat(const struct stat &before, const struct stat &after) {
    return before.st_dev == after.st_dev && before.st_ino == after.st_ino
        && before.st_mode == after.st_mode && before.st_size == after.st_size
        && before.st_mtim.tv_sec == after.st_mtim.tv_sec
        && before.st_mtim.tv_nsec == after.st_mtim.tv_nsec;
}

void set_failure(FileResult &result, std::string reason) {
    if (result.failure.empty()) {
        result.failure = std::move(reason);
    }
}

FileResult inspect_file(int root_descriptor, const ReferenceSpec &spec) {
    FileResult result;
    result.spec = &spec;
    Descriptor profile(::openat(
        root_descriptor,
        std::string(R1D_PROFILE).c_str(),
        O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW));
    if (!profile) {
        result.status = errno == ENOENT ? "MISSING_ARTIFACT" : "FAIL";
        result.failure = errno == ENOENT ? "MISSING_PROFILE" : "PROFILE_OPEN_REJECTED";
        return result;
    }
    Descriptor scenario(::openat(
        profile.get(),
        std::string(spec.scenario).c_str(),
        O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW));
    if (!scenario) {
        result.status = errno == ENOENT ? "MISSING_ARTIFACT" : "FAIL";
        result.failure = errno == ENOENT ? "MISSING_SCENARIO" : "SCENARIO_OPEN_REJECTED";
        return result;
    }
    const std::string filename = std::string(spec.file_sha256) + ".cwrefv2";
    Descriptor file(::openat(
        scenario.get(),
        filename.c_str(),
        O_RDONLY | O_CLOEXEC | O_NOFOLLOW));
    if (!file) {
        result.status = errno == ENOENT ? "MISSING_ARTIFACT" : "FAIL";
        result.failure = errno == ENOENT ? "MISSING_FILE"
                                         : (errno == ELOOP ? "SYMLINK_REJECTED"
                                                          : "FILE_OPEN_REJECTED");
        return result;
    }
    result.path_admitted = true;
    struct stat before {};
    if (::fstat(file.get(), &before) != 0) {
        result.failure = "FSTAT_BEFORE_FAILED";
        return result;
    }
    result.regular_file = S_ISREG(before.st_mode);
    if (!result.regular_file) {
        result.failure = "NOT_REGULAR_FILE";
        return result;
    }
    if (before.st_size < 0
        || static_cast<std::uintmax_t>(before.st_size) > MAXIMUM_BYTES) {
        result.failure = "REFERENCE_INPUT_CAPACITY_EXCEEDED";
        return result;
    }
    result.capacity_admitted = true;
    if (static_cast<std::uintmax_t>(before.st_size) != spec.bytes) {
        result.failure = "TOTAL_SIZE_MISMATCH";
        return result;
    }

    std::vector<std::uint8_t> bytes(spec.bytes);
    std::size_t offset = 0U;
    while (offset < bytes.size()) {
        const ssize_t count = ::read(
            file.get(), bytes.data() + offset, bytes.size() - offset);
        if (count < 0) {
            if (errno == EINTR) {
                continue;
            }
            result.failure = "READ_FAILED";
            return result;
        }
        if (count == 0) {
            result.failure = "TRUNCATED_DURING_READ";
            return result;
        }
        offset += static_cast<std::size_t>(count);
    }
    std::uint8_t trailing = 0U;
    const ssize_t trailing_count = ::read(file.get(), &trailing, 1U);
    struct stat after {};
    result.read_exact = trailing_count == 0 && ::fstat(file.get(), &after) == 0
        && same_stat(before, after);
    if (!result.read_exact) {
        result.failure = "FILE_CHANGED_DURING_READ";
        return result;
    }

    result.observed_sha256 = bytes_hash(bytes);
    result.file_hash_exact = result.observed_sha256 == spec.file_sha256;
    if (!result.file_hash_exact) {
        result.failure = "COMPLETE_FILE_HASH_MISMATCH";
        return result;
    }
    bytes.back() ^= 0x01U;
    result.serialized_mutation_rejected = bytes_hash(bytes) != spec.file_sha256;
    bytes.back() ^= 0x01U;
    if (!result.serialized_mutation_rejected) {
        result.failure = "SERIALIZED_MUTATION_NOT_REJECTED";
        return result;
    }

    try {
        std::vector<std::uint8_t> decoded;
        decoded.reserve(spec.bytes);
        Parser parser(bytes, decoded);
        if (parser.raw(MAGIC.size()) != MAGIC) {
            throw std::runtime_error("MAGIC_MISMATCH");
        }
        const std::uint32_t manifest_bytes = parser.u32(false);
        const std::string expected = expected_manifest(spec);
        if (manifest_bytes != expected.size()
            || parser.raw(manifest_bytes) != expected) {
            throw std::runtime_error("MANIFEST_MISMATCH");
        }
        result.manifest_exact = true;
        const std::uint32_t samples = parser.u32();
        const std::uint32_t frames = parser.u32();
        if (samples != SAMPLE_COUNT || frames != spec.frames) {
            throw std::runtime_error("COUNT_MISMATCH");
        }
        std::vector<std::uint8_t> q99_x = aggregate_prefix(
            "nextengine.nonlocal.nsr3b4dr1d-q99-x.v1", frames);
        std::vector<std::uint8_t> q99_y = aggregate_prefix(
            "nextengine.nonlocal.nsr3b4dr1d-q99-y.v1", frames);
        std::vector<std::uint8_t> receivers = aggregate_prefix(
            "nextengine.nonlocal.nsr3b4dr1d-receiver-count.v1", frames);
        std::size_t first_decoded_x = std::numeric_limits<std::size_t>::max();
        std::uint32_t final_receiver = 0U;
        for (std::uint32_t frame = 0U; frame < frames; ++frame) {
            const std::uint32_t step = parser.u32();
            const std::uint32_t pressure_iterations = parser.u32();
            const std::uint32_t divergence_iterations = parser.u32();
            const std::uint32_t receiver_count = parser.u32();
            const std::uint64_t pressure_error_bits = parser.u64();
            const std::uint64_t divergence_error_bits = parser.u64();
            const double density_min = parser.finite_f64();
            const double density_max = parser.finite_f64();
            const double max_speed = parser.finite_f64();
            if (!std::isfinite(from_bits(pressure_error_bits))
                || !std::isfinite(from_bits(divergence_error_bits))) {
                throw std::runtime_error("NONFINITE_SOLVER_ERROR");
            }
            const std::uint32_t expected_step = frame * spec.stride;
            if (step != expected_step || receiver_count > SAMPLE_COUNT) {
                throw std::runtime_error("FRAME_HEADER_MISMATCH");
            }
            if (frame == 0U) {
                if (pressure_iterations != 0U || divergence_iterations != 0U
                    || receiver_count != 0U || pressure_error_bits != 0U
                    || divergence_error_bits != 0U || density_min != 0.0
                    || density_max != 0.0 || max_speed != 0.0) {
                    throw std::runtime_error("INITIAL_DIAGNOSTICS_MISMATCH");
                }
            } else if (pressure_iterations < 2U || pressure_iterations > 300U
                || divergence_iterations < 1U || divergence_iterations > 100U
                || density_min > density_max || max_speed < 0.0) {
                throw std::runtime_error("FRAME_DIAGNOSTICS_MISMATCH");
            }
            for (std::size_t feature = 0U; feature < FEATURE_COUNT; ++feature) {
                const std::uint32_t count = parser.u32();
                if (frame == 0U && count != 0U) {
                    throw std::runtime_error("INITIAL_FEATURE_COUNT_MISMATCH");
                }
            }
            std::vector<double> x_positions;
            std::vector<double> y_positions;
            x_positions.reserve(SAMPLE_COUNT);
            y_positions.reserve(SAMPLE_COUNT);
            for (std::uint32_t id = 0U; id < SAMPLE_COUNT; ++id) {
                if (parser.u32() != id) {
                    throw std::runtime_error("STABLE_ID_MISMATCH");
                }
                if (frame == 0U && id == 0U) {
                    first_decoded_x = parser.decoded_size();
                }
                const double x = parser.finite_f64();
                const double y = parser.finite_f64();
                parser.finite_f64();
                parser.finite_f64();
                parser.finite_f64();
                parser.finite_f64();
                x_positions.push_back(x);
                y_positions.push_back(y);
            }
            append_u32(q99_x, step);
            double q99 = nearest_rank_q99(std::move(x_positions));
            std::uint64_t q99_bits = 0U;
            std::memcpy(&q99_bits, &q99, sizeof(q99_bits));
            append_u64(q99_x, q99_bits);
            append_u32(q99_y, step);
            q99 = nearest_rank_q99(std::move(y_positions));
            std::memcpy(&q99_bits, &q99, sizeof(q99_bits));
            append_u64(q99_y, q99_bits);
            append_u32(receivers, step);
            append_u32(receivers, receiver_count);
            final_receiver = receiver_count;
        }
        if (parser.cursor() != bytes.size()
            || decoded.size() != bytes.size() - 12U - expected.size()
            || (spec.frames - 1U) * spec.stride != spec.total_steps
            || bytes.size() != 12U + expected.size() + 8U
                + (FRAME_BYTES * spec.frames)
            || final_receiver != spec.final_receiver_count) {
            throw std::runtime_error("FINAL_LAYOUT_MISMATCH");
        }
        result.parse_exact = true;
        result.observed_semantic_root = domain_hash(
            "nextengine.nonlocal.nsr3b4dr1e-decoded-frames.v1", decoded);
        result.semantic_root_exact =
            result.observed_semantic_root == spec.semantic_root;
        result.observed_q99_x_root = bytes_hash(q99_x);
        result.observed_q99_y_root = bytes_hash(q99_y);
        result.observed_receiver_root = bytes_hash(receivers);
        result.aggregates_exact = result.observed_q99_x_root == spec.q99_x_root
            && result.observed_q99_y_root == spec.q99_y_root
            && result.observed_receiver_root == spec.receiver_root;
        if (!result.semantic_root_exact) {
            throw std::runtime_error("DECODED_SEMANTIC_ROOT_MISMATCH");
        }
        if (!result.aggregates_exact) {
            throw std::runtime_error("AGGREGATE_ROOT_MISMATCH");
        }
        if (first_decoded_x == std::numeric_limits<std::size_t>::max()
            || first_decoded_x >= decoded.size()) {
            throw std::runtime_error("DECODED_MUTATION_OFFSET_MISSING");
        }
        decoded[first_decoded_x] ^= 0x01U;
        result.decoded_mutation_rejected = domain_hash(
            "nextengine.nonlocal.nsr3b4dr1e-decoded-frames.v1", decoded)
            != spec.semantic_root;
        if (!result.decoded_mutation_rejected) {
            throw std::runtime_error("DECODED_MUTATION_NOT_REJECTED");
        }
        result.passed = true;
        result.status = "PASS";
    } catch (const std::exception &error) {
        set_failure(result, error.what());
    }
    return result;
}

std::string profile_failure_report(std::string_view reason) {
    std::ostringstream output;
    output << "schema=" << PROFILE_SCHEMA << '\n'
           << "identity=" << IDENTITY << '\n'
           << "status=FAIL\n"
           << "reason=" << reason << '\n'
           << "trajectory_started=false\n"
           << "b4e_execution_authorized=false\n"
           << "runtime_authority=false\n"
           << "production_authority=false\n";
    return output.str();
}

bool profile_exact() {
    return nextengine::nonlocal::sha256_hex(IDENTITY_PROJECTION) == IDENTITY
        && nextengine::nonlocal::sha256_hex(GENERATOR_BUILD_PROJECTION)
            == GENERATOR_BUILD_ROOT
        && nextengine::nonlocal::sha256_hex(PAYLOAD_PROFILE_PROJECTION)
            == PAYLOAD_PROFILE_ROOT
        && nextengine::nonlocal::sha256_hex(AGGREGATE_PROFILE_PROJECTION)
            == AGGREGATE_PROFILE_ROOT
        && scenario_root(HYDRO_MANIFEST) == REFERENCES[0].manifest_root
        && scenario_root(DAM_MANIFEST) == REFERENCES[1].manifest_root
        && scenario_root(ORIFICE_MANIFEST) == REFERENCES[2].manifest_root;
}

void append_file_report(std::ostringstream &output, const FileResult &result) {
    output << "file." << result.spec->scenario << '=' << result.status
           << ";failure=" << result.failure
           << ";bytes=" << result.spec->bytes
           << ";expected_sha256=" << result.spec->file_sha256
           << ";observed_sha256=" << result.observed_sha256
           << ";path=" << std::boolalpha << result.path_admitted
           << ";regular=" << result.regular_file
           << ";capacity=" << result.capacity_admitted
           << ";read=" << result.read_exact
           << ";hash=" << result.file_hash_exact
           << ";manifest=" << result.manifest_exact
           << ";parse=" << result.parse_exact
           << ";semantic=" << result.semantic_root_exact
           << ";aggregates=" << result.aggregates_exact
           << ";serialized_mutation=" << result.serialized_mutation_rejected
           << ";decoded_mutation=" << result.decoded_mutation_rejected << '\n';
}

} // namespace

ReaderRun run_profile_self_test() {
    if (!profile_exact()) {
        return {false, profile_failure_report("PROFILE_IDENTITY_MISMATCH")};
    }
    std::ostringstream output;
    output << "schema=" << PROFILE_SCHEMA << '\n'
           << "identity=" << IDENTITY << '\n'
           << "status=PASS\n"
           << "generator_source_root=" << GENERATOR_SOURCE_ROOT << '\n'
           << "generator_build_root=" << GENERATOR_BUILD_ROOT << '\n'
           << "payload_profile_root=" << PAYLOAD_PROFILE_ROOT << '\n'
           << "aggregate_profile_root=" << AGGREGATE_PROFILE_ROOT << '\n'
           << "scenario_roots_exact=true\n"
           << "trajectory_started=false\n"
           << "b4e_execution_authorized=false\n"
           << "runtime_authority=false\n"
           << "production_authority=false\n";
    return {true, output.str()};
}

ReaderRun run_attestation(std::string_view artifact_root) {
    if (!profile_exact()) {
        return {false, profile_failure_report("PROFILE_IDENTITY_MISMATCH")};
    }
    if (artifact_root.empty() || artifact_root.front() != '/') {
        return {false, profile_failure_report("ARTIFACT_ROOT_NOT_ABSOLUTE")};
    }
    if (artifact_root == "/tmp"
        || (artifact_root.size() >= 5U && artifact_root.substr(0U, 5U) == "/tmp/")) {
        return {false, profile_failure_report("TMP_ARTIFACT_ROOT_FORBIDDEN")};
    }
    Descriptor root = open_absolute_directory(artifact_root);
    if (!root) {
        return {false, profile_failure_report(
            errno == ENOENT ? "ARTIFACT_ROOT_MISSING" : "ARTIFACT_ROOT_REJECTED")};
    }
    struct stat root_status {};
    if (::fstat(root.get(), &root_status) != 0 || !S_ISDIR(root_status.st_mode)) {
        return {false, profile_failure_report("ARTIFACT_ROOT_NOT_DIRECTORY")};
    }

    std::array<FileResult, REFERENCES.size()> results;
    bool passed = true;
    std::string first_failure;
    for (std::size_t index = 0U; index < REFERENCES.size(); ++index) {
        results[index] = inspect_file(root.get(), REFERENCES[index]);
        passed = passed && results[index].passed;
        if (first_failure.empty() && !results[index].passed) {
            first_failure = std::string(REFERENCES[index].scenario) + ':'
                + results[index].failure;
        }
    }
    std::ostringstream output;
    output << "schema=" << SCHEMA << '\n'
           << "identity=" << IDENTITY << '\n'
           << "profile_root=" << R1D_PROFILE << '\n'
           << "status=" << (passed ? "PASS" : "FAIL") << '\n'
           << "first_failure=" << first_failure << '\n'
           << "artifact_root_supplied=true\n";
    for (const FileResult &result : results) {
        append_file_report(output, result);
    }
    output << "trajectory_started=false\n"
           << "new_external_dfsph_reference_candidate=" << std::boolalpha << passed << '\n'
           << "b4e_design_authorized=" << passed << '\n'
           << "b4e_execution_authorized=false\n"
           << "runtime_authority=false\n"
           << "production_authority=false\n";
    return {passed, output.str()};
}

ReaderRun reject_unknown_argument() {
    return {false, profile_failure_report("UNKNOWN_ARGUMENT")};
}

} // namespace nextengine::nonlocal_reference_reader
