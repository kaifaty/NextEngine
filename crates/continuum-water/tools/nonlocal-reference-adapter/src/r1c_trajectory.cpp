#include "r1c_trajectory.hpp"

#include "r1c_manifest.hpp"
#include "sha256.hpp"

#include "SPlisHSPlasH/BoundaryModel_Akinci2012.h"
#include "SPlisHSPlasH/DFSPH/TimeStepDFSPH.h"
#include "SPlisHSPlasH/FluidModel.h"
#include "SPlisHSPlasH/Simulation.h"
#include "SPlisHSPlasH/StaticRigidBody.h"
#include "SPlisHSPlasH/TimeManager.h"
#include "SPlisHSPlasH/XSPH.h"

#include <algorithm>
#include <array>
#include <cerrno>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fcntl.h>
#include <iomanip>
#include <limits>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <system_error>
#include <unistd.h>
#include <utility>
#include <vector>

namespace nextengine::nonlocal_reference {
namespace {

constexpr std::string_view SCHEMA =
    "nextengine.nonlocal.nsr3b4dr1c-trajectory.v1";
constexpr std::string_view CONTRACT_IDENTITY =
    "865570e18864ec55cdbbbbad8b9cfa3f200a085087ecf272366c342144488927";
constexpr std::string_view DIAGNOSTIC_SCHEMA =
    "nextengine.nonlocal.nsr3b4dr1c2-failure-observability.v1";
constexpr std::string_view DIAGNOSTIC_CONTRACT_IDENTITY =
    "cf4e7dced6d2a597ea3ee8267daaab587c4fadb97fe20daa58643ab2412012cc";
constexpr std::string_view PRESSURE_SWEEP_SCHEMA =
    "nextengine.nonlocal.nsr3b4dr1c3-pressure-cap-sweep.v1";
constexpr std::string_view PRESSURE_SWEEP_CONTRACT_IDENTITY =
    "926e594fedec03e9c3b08aa76fc57a22988049e97f60c5e47113387aae879678";
constexpr std::string_view R1C4_SCHEMA =
    "nextengine.nonlocal.nsr3b4dr1c4-trajectory.v1";
constexpr std::string_view R1C4_CONTRACT_IDENTITY =
    "7490aa5390296c5f7fc29a56f7039458ced1969ddee555bfe0ea07f50d49be41";
constexpr std::string_view R1C4_PROFILE_IDENTITY_PROJECTION =
    "nextengine.nonlocal.nsr3b4dr1c4-pressure-cap-reclosure|v1|"
    "parent=865570e18864ec55cdbbbbad8b9cfa3f200a085087ecf272366c342144488927|"
    "diagnostic=926e594fedec03e9c3b08aa76fc57a22988049e97f60c5e47113387aae879678|"
    "change=pressure-max:100->300|"
    "observed=hydro-step1:iterations220,error:0x3fb965727028bcc7,"
    "threshold:0x3fb999999999999a|volume=.000125|mass=.125|all-else=r1c1|"
    "runs=2-fresh-byte-exact-per-scenario|"
    "order=hydro,dam,orifice;stop-first-failure|credit=new-root-only";
constexpr std::string_view R1C5_SCHEMA =
    "nextengine.nonlocal.nsr3b4dr1c5-trajectory.v1";
constexpr std::string_view R1C5_CONTRACT_IDENTITY =
    "3f5a73693f05daf487898fd692160357c1775652a1c11e65205356243c57386f";
constexpr std::string_view R1C5_PROFILE_IDENTITY_PROJECTION =
    "nextengine.nonlocal.nsr3b4dr1c5-orifice-domain-reclosure|v1|"
    "parent=7490aa5390296c5f7fc29a56f7039458ced1969ddee555bfe0ea07f50d49be41|"
    "failed_report=507:73b880fc4bf7cbd30447808f80d68bd278d9bf34617cb812011311695d923100|"
    "correction=orifice-domain-x-max:1->2;boundary-source-nx:20-unchanged|"
    "geometry=hydro:1,dam:4,orifice:2|pressure-max=300|all-else=r1c4|"
    "runs=2-fresh-byte-exact-per-scenario|"
    "order=hydro,dam,orifice;stop-first-failure|credit=new-root-only";
constexpr std::string_view R1D_SCHEMA =
    "nextengine.nonlocal.nsr3b4dr1d-full-generation.v1";
constexpr std::string_view R1D_MANIFEST_SCHEMA =
    "nextengine.nonlocal.nsr3b4dr1d-manifest-preflight.v1";
constexpr std::string_view R1D_CONTRACT_IDENTITY =
    "ba34b4e3b12986ebc831320d6811551d5311a6774a64f079aabe3a5eaa6bb746";
constexpr std::string_view R1D_PROFILE_IDENTITY_PROJECTION =
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
constexpr std::string_view R1D_HYDRO_MANIFEST = R"(B4DR1D_SCENARIO_V1_BEGIN
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
constexpr std::string_view R1D_DAM_MANIFEST = R"(B4DR1D_SCENARIO_V1_BEGIN
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
constexpr std::string_view R1D_ORIFICE_MANIFEST = R"(B4DR1D_SCENARIO_V1_BEGIN
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
constexpr std::uint64_t DT_BITS = UINT64_C(0x3f71111111111111);
constexpr std::uint32_t SAMPLE_COUNT = 6'000;
constexpr std::uint32_t FRAME_COUNT = 25;
constexpr std::size_t FRAME_BYTES = 312'156U;
constexpr std::size_t FEATURE_COUNT = 25;
constexpr std::size_t MAX_FILE_BYTES = 64U * 1024U * 1024U;

enum class TrajectoryMode {
    R1C,
    R1C2,
    R1C3PressureSweep,
    R1C4,
    R1C5,
    R1D,
};

std::string_view schema_for(TrajectoryMode mode) {
    if (mode == TrajectoryMode::R1C2) {
        return DIAGNOSTIC_SCHEMA;
    }
    if (mode == TrajectoryMode::R1C3PressureSweep) {
        return PRESSURE_SWEEP_SCHEMA;
    }
    if (mode == TrajectoryMode::R1C4) {
        return R1C4_SCHEMA;
    }
    if (mode == TrajectoryMode::R1C5) {
        return R1C5_SCHEMA;
    }
    if (mode == TrajectoryMode::R1D) {
        return R1D_SCHEMA;
    }
    return SCHEMA;
}

std::string_view contract_for(TrajectoryMode mode) {
    if (mode == TrajectoryMode::R1C2) {
        return DIAGNOSTIC_CONTRACT_IDENTITY;
    }
    if (mode == TrajectoryMode::R1C3PressureSweep) {
        return PRESSURE_SWEEP_CONTRACT_IDENTITY;
    }
    if (mode == TrajectoryMode::R1C4) {
        return R1C4_CONTRACT_IDENTITY;
    }
    if (mode == TrajectoryMode::R1C5) {
        return R1C5_CONTRACT_IDENTITY;
    }
    if (mode == TrajectoryMode::R1D) {
        return R1D_CONTRACT_IDENTITY;
    }
    return CONTRACT_IDENTITY;
}

struct R1DSchedule {
    std::string_view scenario_id;
    std::string_view manifest;
    std::string_view manifest_root;
    std::uint32_t total_steps;
    std::uint32_t output_stride;
    std::uint32_t frame_count;
    std::size_t expected_payload_bytes;
};

constexpr std::array<R1DSchedule, 3> R1D_SCHEDULES = {{
    {
        "CW-HYDRO-001",
        R1D_HYDRO_MANIFEST,
        "c430b679dfeec33a6ac12c51df75ddee7e7bc48484c6c05188219f0a727909a0",
        1'200U,
        24U,
        51U,
        15'920'965U,
    },
    {
        "CW-DAMBREAK-001",
        R1D_DAM_MANIFEST,
        "8d0a0a85adba50d4784245d460c83757bcce92841d804f4739b81faf27fd5f09",
        720U,
        4U,
        181U,
        56'501'239U,
    },
    {
        "CW-ORIFICE-001",
        R1D_ORIFICE_MANIFEST,
        "53d0db457091d7a7d6ede58a0628688b701850d6de30dacdb14ee9951d036961",
        720U,
        4U,
        181U,
        56'501'341U,
    },
}};

const R1DSchedule &r1d_schedule(std::string_view scenario_id) {
    for (const R1DSchedule &schedule : R1D_SCHEDULES) {
        if (schedule.scenario_id == scenario_id) {
            return schedule;
        }
    }
    throw std::runtime_error("UNKNOWN_R1D_SCENARIO");
}

std::string r1d_scenario_root(std::string_view manifest) {
    constexpr char DOMAIN[] = "nextengine.nonlocal.nsr3b4dr1d-scenario.v1";
    std::string projection(DOMAIN, sizeof(DOMAIN));
    projection.append(manifest);
    return nextengine::nonlocal::sha256_hex(projection);
}

std::size_t expected_payload_bytes(
    std::string_view profile,
    std::string_view manifest,
    std::uint32_t frame_count) {
    return 20U + profile.size() + 1U + manifest.size()
        + (FRAME_BYTES * static_cast<std::size_t>(frame_count));
}

std::uint64_t to_bits(double value) {
    std::uint64_t bits = 0;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    return bits;
}

double from_bits(std::uint64_t bits) {
    double value = 0.0;
    std::memcpy(&value, &bits, sizeof(value));
    return value;
}

const double DT = from_bits(DT_BITS);

void require_finite(double value, std::string_view field) {
    if (!std::isfinite(value)) {
        throw std::runtime_error(std::string(field) + "_NOT_FINITE");
    }
}

void append_u32(std::vector<std::uint8_t> &bytes, std::uint32_t value) {
    for (unsigned int shift = 0; shift < 32; shift += 8) {
        bytes.push_back(static_cast<std::uint8_t>(value >> shift));
    }
}

void append_u64(std::vector<std::uint8_t> &bytes, std::uint64_t value) {
    for (unsigned int shift = 0; shift < 64; shift += 8) {
        bytes.push_back(static_cast<std::uint8_t>(value >> shift));
    }
}

void append_f64(std::vector<std::uint8_t> &bytes, double value) {
    require_finite(value, "SERIALIZED_FLOAT");
    append_u64(bytes, to_bits(value));
}

struct FrameDiagnostics {
    std::uint32_t step = 0;
    std::uint32_t pressure_iterations = 0;
    std::uint32_t divergence_iterations = 0;
    std::uint32_t receiver_count = 0;
    double pressure_error = 0.0;
    double divergence_error = 0.0;
    double density_min = 0.0;
    double density_max = 0.0;
    double max_speed = 0.0;
    bool pressure_converged = false;
    bool divergence_converged = false;
    std::uint64_t time_step_bits = 0;
    std::array<std::uint32_t, FEATURE_COUNT> feature_counts{};
};

class SimulationGuard {
public:
    SimulationGuard() = default;
    SimulationGuard(const SimulationGuard &) = delete;
    SimulationGuard &operator=(const SimulationGuard &) = delete;
    ~SimulationGuard() {
        if (SPH::Simulation::hasCurrent()) {
            delete SPH::Simulation::getCurrent();
        }
    }
};

std::filesystem::path validate_output_directory(std::string_view raw_path) {
    const std::filesystem::path path(raw_path);
    if (!path.is_absolute()) {
        throw std::runtime_error("OUTPUT_DIRECTORY_NOT_ABSOLUTE");
    }
    const std::filesystem::file_status status = std::filesystem::symlink_status(path);
    if (!std::filesystem::exists(status) || !std::filesystem::is_directory(status)
        || std::filesystem::is_symlink(status)) {
        throw std::runtime_error("OUTPUT_DIRECTORY_NOT_FRESH_DIRECTORY");
    }
    if (std::filesystem::directory_iterator(path) != std::filesystem::directory_iterator()) {
        throw std::runtime_error("OUTPUT_DIRECTORY_NOT_EMPTY");
    }
    return path;
}

std::uint32_t checked_u32(std::size_t value, std::string_view field) {
    if (value > std::numeric_limits<std::uint32_t>::max()) {
        throw std::runtime_error(std::string(field) + "_EXCEEDS_U32");
    }
    return static_cast<std::uint32_t>(value);
}

std::array<double, 3> position_by_id(SPH::FluidModel &model, std::uint32_t id) {
    const unsigned int index = model.getParticleIndex(id);
    if (index >= model.numActiveParticles() || model.getParticleId(index) != id) {
        throw std::runtime_error("STABLE_ID_POSITION_MAP_MISMATCH");
    }
    const Vector3r &value = model.getPosition(index);
    return {value[0], value[1], value[2]};
}

std::array<double, 3> velocity_by_id(SPH::FluidModel &model, std::uint32_t id) {
    const unsigned int index = model.getParticleIndex(id);
    if (index >= model.numActiveParticles() || model.getParticleId(index) != id) {
        throw std::runtime_error("STABLE_ID_VELOCITY_MAP_MISMATCH");
    }
    const Vector3r &value = model.getVelocity(index);
    return {value[0], value[1], value[2]};
}

void append_frame(
    std::vector<std::uint8_t> &payload,
    SPH::FluidModel &model,
    const FrameDiagnostics &diagnostics) {
    append_u32(payload, diagnostics.step);
    append_u32(payload, diagnostics.pressure_iterations);
    append_u32(payload, diagnostics.divergence_iterations);
    append_u32(payload, diagnostics.receiver_count);
    append_f64(payload, diagnostics.pressure_error);
    append_f64(payload, diagnostics.divergence_error);
    append_f64(payload, diagnostics.density_min);
    append_f64(payload, diagnostics.density_max);
    append_f64(payload, diagnostics.max_speed);
    for (std::uint32_t count : diagnostics.feature_counts) {
        append_u32(payload, count);
    }
    for (std::uint32_t id = 0; id < SAMPLE_COUNT; ++id) {
        const std::array<double, 3> position = position_by_id(model, id);
        const std::array<double, 3> velocity = velocity_by_id(model, id);
        append_u32(payload, id);
        for (double component : position) {
            append_f64(payload, component);
        }
        for (double component : velocity) {
            append_f64(payload, component);
        }
    }
}

struct AggregateProjections {
    std::vector<std::uint8_t> q99_x;
    std::vector<std::uint8_t> q99_y;
    std::vector<std::uint8_t> receiver_count;
};

std::vector<std::uint8_t> aggregate_projection(
    std::string_view domain,
    std::uint32_t frame_count) {
    std::vector<std::uint8_t> projection(domain.begin(), domain.end());
    projection.push_back(0U);
    append_u32(projection, SAMPLE_COUNT);
    append_u32(projection, frame_count);
    return projection;
}

AggregateProjections make_aggregate_projections(std::uint32_t frame_count) {
    return {
        aggregate_projection(
            "nextengine.nonlocal.nsr3b4dr1d-q99-x.v1",
            frame_count),
        aggregate_projection(
            "nextengine.nonlocal.nsr3b4dr1d-q99-y.v1",
            frame_count),
        aggregate_projection(
            "nextengine.nonlocal.nsr3b4dr1d-receiver-count.v1",
            frame_count),
    };
}

double nearest_rank_q99(std::vector<double> values) {
    if (values.size() != SAMPLE_COUNT) {
        throw std::runtime_error("Q99_SAMPLE_COUNT_MISMATCH");
    }
    for (double value : values) {
        require_finite(value, "Q99_POSITION");
    }
    std::sort(values.begin(), values.end());
    const std::size_t one_based_rank =
        ((99U * values.size()) + 99U) / 100U;
    if (one_based_rank == 0U || one_based_rank > values.size()) {
        throw std::runtime_error("Q99_RANK_OUT_OF_RANGE");
    }
    return values[one_based_rank - 1U];
}

void append_aggregate_frame(
    AggregateProjections &projections,
    SPH::FluidModel &model,
    const FrameDiagnostics &diagnostics) {
    std::vector<double> x_positions;
    std::vector<double> y_positions;
    x_positions.reserve(SAMPLE_COUNT);
    y_positions.reserve(SAMPLE_COUNT);
    for (std::uint32_t id = 0; id < SAMPLE_COUNT; ++id) {
        const std::array<double, 3> position = position_by_id(model, id);
        x_positions.push_back(position[0]);
        y_positions.push_back(position[1]);
    }
    append_u32(projections.q99_x, diagnostics.step);
    append_f64(projections.q99_x, nearest_rank_q99(std::move(x_positions)));
    append_u32(projections.q99_y, diagnostics.step);
    append_f64(projections.q99_y, nearest_rank_q99(std::move(y_positions)));
    append_u32(projections.receiver_count, diagnostics.step);
    append_u32(projections.receiver_count, diagnostics.receiver_count);
}

std::string projection_hash(const std::vector<std::uint8_t> &projection) {
    return nextengine::nonlocal::sha256_hex(std::string_view(
        reinterpret_cast<const char *>(projection.data()),
        projection.size()));
}

void write_all(int descriptor, const std::vector<std::uint8_t> &payload) {
    std::size_t offset = 0;
    while (offset < payload.size()) {
        const ssize_t written = ::write(
            descriptor,
            payload.data() + offset,
            payload.size() - offset);
        if (written < 0) {
            if (errno == EINTR) {
                continue;
            }
            throw std::system_error(errno, std::generic_category(), "write payload");
        }
        if (written == 0) {
            throw std::runtime_error("PAYLOAD_WRITE_MADE_NO_PROGRESS");
        }
        offset += static_cast<std::size_t>(written);
    }
}

std::string write_payload(
    const std::filesystem::path &directory,
    std::string_view scenario_id,
    const std::vector<std::uint8_t> &payload) {
    const std::string filename = std::string(scenario_id) + ".cwrefv2";
    const std::filesystem::path final_path = directory / filename;
    const std::filesystem::path partial_path = directory / (filename + ".partial");
    int descriptor = ::open(
        partial_path.c_str(),
        O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC | O_NOFOLLOW,
        0600);
    if (descriptor < 0) {
        throw std::system_error(errno, std::generic_category(), "open partial payload");
    }
    bool descriptor_open = true;
    bool final_published = false;
    try {
        write_all(descriptor, payload);
        if (::fsync(descriptor) != 0) {
            throw std::system_error(errno, std::generic_category(), "fsync payload");
        }
        if (::close(descriptor) != 0) {
            descriptor_open = false;
            throw std::system_error(errno, std::generic_category(), "close payload");
        }
        descriptor_open = false;
        if (::rename(partial_path.c_str(), final_path.c_str()) != 0) {
            throw std::system_error(errno, std::generic_category(), "rename payload");
        }
        final_published = true;
        const int directory_descriptor =
            ::open(directory.c_str(), O_RDONLY | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW);
        if (directory_descriptor < 0) {
            throw std::system_error(errno, std::generic_category(), "open output directory");
        }
        const int sync_result = ::fsync(directory_descriptor);
        const int sync_error = errno;
        const int close_result = ::close(directory_descriptor);
        if (sync_result != 0) {
            throw std::system_error(sync_error, std::generic_category(), "fsync directory");
        }
        if (close_result != 0) {
            throw std::system_error(errno, std::generic_category(), "close directory");
        }
    } catch (...) {
        if (descriptor_open) {
            ::close(descriptor);
        }
        std::error_code ignored;
        std::filesystem::remove(partial_path, ignored);
        if (final_published) {
            std::filesystem::remove(final_path, ignored);
        }
        throw;
    }
    return filename;
}

FrameDiagnostics capture_solver_diagnostics(
    SPH::TimeStepDFSPH &time_step,
    std::uint32_t step) {
    FrameDiagnostics result;
    result.step = step;
    result.pressure_iterations = static_cast<std::uint32_t>(time_step.getNumIterations());
    result.divergence_iterations = time_step.getNumDivergenceIterations();
    result.pressure_error = time_step.getLastPressureError();
    result.divergence_error = time_step.getLastDivergenceError();
    result.pressure_converged = time_step.lastPressureConverged();
    result.divergence_converged = time_step.lastDivergenceConverged();
    result.time_step_bits = to_bits(SPH::TimeManager::getCurrent()->getTimeStepSize());
    return result;
}

void validate_solver_diagnostics(
    const FrameDiagnostics &result,
    std::uint32_t pressure_cap,
    bool require_pressure_convergence) {
    require_finite(result.pressure_error, "PRESSURE_ERROR");
    require_finite(result.divergence_error, "DIVERGENCE_ERROR");
    if (require_pressure_convergence && !result.pressure_converged) {
        throw std::runtime_error("PRESSURE_NOT_CONVERGED");
    }
    if (!result.divergence_converged) {
        throw std::runtime_error("DIVERGENCE_NOT_CONVERGED");
    }
    if (result.pressure_iterations < 2U || result.pressure_iterations > pressure_cap) {
        throw std::runtime_error("PRESSURE_ITERATIONS_OUT_OF_RANGE");
    }
    if (result.divergence_iterations < 1U || result.divergence_iterations > 100U) {
        throw std::runtime_error("DIVERGENCE_ITERATIONS_OUT_OF_RANGE");
    }
    if (result.time_step_bits != DT_BITS) {
        throw std::runtime_error("TIME_STEP_BITS_CHANGED");
    }
}

void project_contact_and_measure(
    SPH::FluidModel &model,
    const R1CScenarioData &scenario,
    const std::vector<std::array<double, 3>> &previous_positions,
    FrameDiagnostics &result) {
    result.density_min = std::numeric_limits<double>::infinity();
    result.density_max = -std::numeric_limits<double>::infinity();
    for (std::uint32_t id = 0; id < SAMPLE_COUNT; ++id) {
        const unsigned int index = model.getParticleIndex(id);
        if (index >= model.numActiveParticles() || model.getParticleId(index) != id) {
            throw std::runtime_error("STABLE_ID_CONTACT_MAP_MISMATCH");
        }
        const Vector3r &upstream_velocity = model.getVelocity(index);
        const R1CContactProjection projection = project_r1c_contact(
            previous_positions[id],
            {upstream_velocity[0], upstream_velocity[1], upstream_velocity[2]},
            scenario.x_max,
            scenario.orifice);
        model.setVelocity(
            index,
            Vector3r(
                projection.velocity[0], projection.velocity[1], projection.velocity[2]));
        model.setPosition(
            index,
            Vector3r(
                projection.position[0], projection.position[1], projection.position[2]));
        for (std::size_t feature = 0; feature < FEATURE_COUNT; ++feature) {
            result.feature_counts[feature] += projection.feature_counts[feature];
        }
        const double density = model.getDensity(index);
        require_finite(density, "DENSITY");
        result.density_min = std::min(result.density_min, density);
        result.density_max = std::max(result.density_max, density);
        const double speed_squared =
            (projection.velocity[0] * projection.velocity[0])
            + (projection.velocity[1] * projection.velocity[1])
            + (projection.velocity[2] * projection.velocity[2]);
        require_finite(speed_squared, "SPEED_SQUARED");
        result.max_speed = std::max(result.max_speed, std::sqrt(speed_squared));
        if (scenario.orifice && projection.position[0] > 1.0) {
            ++result.receiver_count;
        }
    }
    require_finite(result.density_min, "DENSITY_MIN");
    require_finite(result.density_max, "DENSITY_MAX");
    require_finite(result.max_speed, "MAX_SPEED");
    if (result.density_min > result.density_max) {
        throw std::runtime_error("DENSITY_RANGE_INVERTED");
    }
}

std::string hex_u64(std::uint64_t value) {
    std::ostringstream output;
    output << "0x" << std::hex << std::setfill('0') << std::setw(16) << value;
    return output.str();
}

std::string failure_report(
    std::string_view reason,
    bool simulation_created,
    bool trajectory_started,
    TrajectoryMode mode,
    std::uint32_t pressure_cap,
    std::string_view failure_phase,
    const std::optional<FrameDiagnostics> &diagnostics) {
    std::ostringstream output;
    output << "schema=" << schema_for(mode) << '\n'
           << "contract_identity=" << contract_for(mode) << '\n'
           << "status=FAIL\n"
           << "reason=" << reason << '\n'
           << "simulation_created=" << std::boolalpha << simulation_created << '\n'
           << "trajectory_started=" << trajectory_started << '\n';
    if (mode == TrajectoryMode::R1C3PressureSweep) {
        output << "pressure_cap=" << pressure_cap << '\n';
    }
    if (mode != TrajectoryMode::R1C && diagnostics.has_value()) {
        output << "failure_phase=" << failure_phase << '\n'
               << "failure_step=" << diagnostics->step << '\n'
               << "pressure_iterations=" << diagnostics->pressure_iterations << '\n'
               << "pressure_error_bits=" << hex_u64(to_bits(diagnostics->pressure_error))
               << '\n'
               << "pressure_converged=" << diagnostics->pressure_converged << '\n'
               << "divergence_iterations=" << diagnostics->divergence_iterations << '\n'
               << "divergence_error_bits="
               << hex_u64(to_bits(diagnostics->divergence_error)) << '\n'
               << "divergence_converged=" << diagnostics->divergence_converged << '\n'
               << "time_step_bits=" << hex_u64(diagnostics->time_step_bits) << '\n';
    }
    return output.str();
}

std::string r1d_manifest_rejected_report(std::string_view reason) {
    std::ostringstream output;
    output << "schema=" << R1D_MANIFEST_SCHEMA << '\n'
           << "contract_identity=" << R1D_CONTRACT_IDENTITY << '\n'
           << "status=REJECTED\n"
           << "reason=" << reason << '\n'
           << "simulation_created=false\n"
           << "trajectory_started=false\n"
           << "r1e_authorized=false\n"
           << "b4e_authorized=false\n";
    return output.str();
}

AdapterRun run_r1d_manifest_preflight_impl(bool force_schedule_mismatch) {
    const std::string process_failure = process_preflight_failure();
    if (!process_failure.empty()) {
        return {false, r1d_manifest_rejected_report(process_failure)};
    }
    const AdapterRun r1c_preflight = run_r1c_manifest_preflight(false);
    if (!r1c_preflight.passed) {
        return {false, r1d_manifest_rejected_report("R1C_PREFLIGHT_NOT_PASS")};
    }
    try {
        if (nextengine::nonlocal::sha256_hex(R1D_PROFILE_IDENTITY_PROJECTION)
            != R1D_CONTRACT_IDENTITY) {
            throw std::runtime_error("R1D_PROFILE_IDENTITY_MISMATCH");
        }
        std::vector<double> initial_x;
        std::vector<double> initial_y;
        initial_x.reserve(SAMPLE_COUNT);
        initial_y.reserve(SAMPLE_COUNT);
        for (std::uint32_t iy = 0; iy < 15U; ++iy) {
            for (std::uint32_t iz = 0; iz < 20U; ++iz) {
                for (std::uint32_t ix = 0; ix < 20U; ++ix) {
                    initial_x.push_back(
                        static_cast<double>(25'000U + (50'000U * ix))
                        / 1'000'000.0);
                    initial_y.push_back(
                        static_cast<double>(25'000U + (50'000U * iy))
                        / 1'000'000.0);
                }
            }
        }
        if (to_bits(nearest_rank_q99(std::move(initial_x)))
                != UINT64_C(0x3fef333333333333)
            || to_bits(nearest_rank_q99(std::move(initial_y)))
                != UINT64_C(0x3fe7333333333333)) {
            throw std::runtime_error("R1D_Q99_INITIAL_LATTICE_MISMATCH");
        }
        std::ostringstream scenario_report;
        for (const R1DSchedule &schedule : R1D_SCHEDULES) {
            const std::string actual_root = r1d_scenario_root(schedule.manifest);
            std::string_view expected_root = schedule.manifest_root;
            if (force_schedule_mismatch
                && schedule.scenario_id == "CW-ORIFICE-001") {
                expected_root = R1D_CONTRACT_IDENTITY;
            }
            if (actual_root != expected_root) {
                throw std::runtime_error(
                    std::string(schedule.scenario_id) + ":SCHEDULE_MANIFEST_ROOT");
            }
            if (schedule.output_stride == 0U
                || (schedule.total_steps % schedule.output_stride) != 0U
                || (schedule.total_steps / schedule.output_stride) + 1U
                    != schedule.frame_count) {
                throw std::runtime_error(
                    std::string(schedule.scenario_id) + ":SCHEDULE_FRAME_COUNT");
            }
            const std::size_t actual_payload_bytes = expected_payload_bytes(
                R1D_PROFILE_IDENTITY_PROJECTION,
                schedule.manifest,
                schedule.frame_count);
            if (actual_payload_bytes != schedule.expected_payload_bytes) {
                throw std::runtime_error(
                    std::string(schedule.scenario_id) + ":PAYLOAD_SIZE");
            }
            if (actual_payload_bytes > MAX_FILE_BYTES) {
                throw std::runtime_error(
                    std::string(schedule.scenario_id) + ":PAYLOAD_CAPACITY");
            }
            scenario_report << "scenario." << schedule.scenario_id
                            << "=PASS;manifest_bytes=" << schedule.manifest.size()
                            << ";manifest_root=" << actual_root
                            << ";total_steps=" << schedule.total_steps
                            << ";output_stride=" << schedule.output_stride
                            << ";frames=" << schedule.frame_count
                            << ";payload_bytes=" << actual_payload_bytes << '\n';
        }
        std::ostringstream output;
        output << "schema=" << R1D_MANIFEST_SCHEMA << '\n'
               << "contract_identity=" << R1D_CONTRACT_IDENTITY << '\n'
               << "parent_identity=" << R1C5_CONTRACT_IDENTITY << '\n'
               << "status=PASS\n"
               << "simulation_created=false\n"
               << "trajectory_started=false\n"
               << scenario_report.str()
               << "q99_initial_lattice_checked=true\n"
               << "schedule_mismatch_rejected=true\n"
               << "full_generation_authorized=true\n"
               << "r1e_authorized=false\n"
               << "b4e_authorized=false\n";
        return {true, output.str()};
    } catch (const std::exception &error) {
        return {false, r1d_manifest_rejected_report(error.what())};
    }
}

AdapterRun run_r1c_trajectory_impl(
    std::string_view scenario_id,
    std::string_view output_dir,
    TrajectoryMode mode,
    std::uint32_t pressure_cap) {
    bool simulation_created = false;
    bool trajectory_started = false;
    std::string_view failure_phase = "pre_simulation";
    std::optional<FrameDiagnostics> failure_diagnostics;
    try {
        const AdapterRun manifest_preflight = run_r1c_manifest_preflight(false);
        if (!manifest_preflight.passed) {
            return {
                false,
                failure_report(
                    "MANIFEST_PREFLIGHT_NOT_PASS",
                    false,
                    false,
                    mode,
                    pressure_cap,
                    failure_phase,
                    failure_diagnostics),
            };
        }
        const std::filesystem::path directory = validate_output_directory(output_dir);
        const R1CScenarioData scenario = build_r1c_scenario(scenario_id);
        std::uint32_t total_steps = 24U;
        std::uint32_t output_stride = 1U;
        std::uint32_t frame_count = FRAME_COUNT;
        std::string_view scenario_manifest = scenario.manifest;
        std::string_view scenario_manifest_root;
        std::size_t frozen_expected_payload_bytes = 0U;
        if (mode == TrajectoryMode::R1D) {
            const AdapterRun r1d_preflight = run_r1d_manifest_preflight_impl(false);
            if (!r1d_preflight.passed) {
                throw std::runtime_error("R1D_MANIFEST_PREFLIGHT_NOT_PASS");
            }
            const R1DSchedule &schedule = r1d_schedule(scenario_id);
            total_steps = schedule.total_steps;
            output_stride = schedule.output_stride;
            frame_count = schedule.frame_count;
            scenario_manifest = schedule.manifest;
            scenario_manifest_root = schedule.manifest_root;
            frozen_expected_payload_bytes = schedule.expected_payload_bytes;
        }

        std::vector<Vector3r> fluid_positions;
        std::vector<Vector3r> fluid_velocities(SAMPLE_COUNT, Vector3r::Zero());
        std::vector<unsigned int> object_ids(SAMPLE_COUNT, 0U);
        fluid_positions.reserve(SAMPLE_COUNT);
        for (const std::array<double, 3> &position : scenario.fluid_positions) {
            fluid_positions.emplace_back(position[0], position[1], position[2]);
        }
        std::vector<Vector3r> boundary_positions;
        boundary_positions.reserve(scenario.boundary_positions.size());
        for (const std::array<double, 3> &position : scenario.boundary_positions) {
            boundary_positions.emplace_back(position[0], position[1], position[2]);
        }

        std::vector<std::uint8_t> payload;
        const std::size_t computed_expected_payload_bytes = expected_payload_bytes(
            mode == TrajectoryMode::R1D
                ? R1D_PROFILE_IDENTITY_PROJECTION
                : (mode == TrajectoryMode::R1C5
                    ? R1C5_PROFILE_IDENTITY_PROJECTION
                    : (mode == TrajectoryMode::R1C4
                        ? R1C4_PROFILE_IDENTITY_PROJECTION
                        : r1c_profile_identity_projection())),
            scenario_manifest,
            frame_count);
        payload.reserve(computed_expected_payload_bytes);
        const std::array<std::uint8_t, 8> magic = {'C', 'W', 'R', 'E', 'F', 'V', '2', 0};
        payload.insert(payload.end(), magic.begin(), magic.end());
        std::string_view profile_identity = r1c_profile_identity_projection();
        if (mode == TrajectoryMode::R1C4) {
            profile_identity = R1C4_PROFILE_IDENTITY_PROJECTION;
        } else if (mode == TrajectoryMode::R1C5) {
            profile_identity = R1C5_PROFILE_IDENTITY_PROJECTION;
        } else if (mode == TrajectoryMode::R1D) {
            profile_identity = R1D_PROFILE_IDENTITY_PROJECTION;
        }
        std::string manifest(profile_identity);
        manifest.push_back('\n');
        manifest.append(scenario_manifest);
        append_u32(payload, checked_u32(manifest.size(), "MANIFEST_LENGTH"));
        payload.insert(payload.end(), manifest.begin(), manifest.end());
        append_u32(payload, SAMPLE_COUNT);
        append_u32(payload, frame_count);

        SimulationGuard simulation_guard;
        SPH::Simulation *simulation = SPH::Simulation::getCurrent();
        simulation_created = true;
        failure_phase = "configuration";
        simulation->init(static_cast<Real>(0.025), false);
        simulation->setBoundaryHandlingMethod(SPH::BoundaryHandlingMethods::Akinci2012);
        simulation->setValue<int>(
            SPH::Simulation::KERNEL_METHOD,
            SPH::Simulation::ENUM_KERNEL_PRECOMPUTED_CUBIC);
        simulation->setValue<int>(
            SPH::Simulation::GRAD_KERNEL_METHOD,
            SPH::Simulation::ENUM_GRADKERNEL_PRECOMPUTED_CUBIC);
        simulation->setValue<int>(
            SPH::Simulation::CFL_METHOD,
            SPH::Simulation::ENUM_CFL_NONE);
        simulation->setValue<bool>(SPH::Simulation::ENABLE_Z_SORT, true);
        simulation->setValue<unsigned int>(SPH::Simulation::STEPS_PER_Z_SORT, 500U);
        Vector3r gravity(0.0, -9.81, 0.0);
        simulation->setVecValue<Real>(SPH::Simulation::GRAVITATION, gravity.data());
        simulation->addFluidModel(
            "Fluid",
            SAMPLE_COUNT,
            fluid_positions.data(),
            fluid_velocities.data(),
            object_ids.data(),
            0U);
        SPH::FluidModel *fluid_model = simulation->getFluidModel(0);
        fluid_model->setDensity0(1000.0);
        fluid_model->getVolume(0) = 0.000125;
        for (std::uint32_t id = 0; id < SAMPLE_COUNT; ++id) {
            fluid_model->setMass(id, 0.125);
        }
        fluid_model->setDragMethod(0U);
        fluid_model->setElasticityMethod(0U);
        fluid_model->setSurfaceTensionMethod(0U);
        fluid_model->setViscosityMethod(0U);
        fluid_model->setVorticityMethod(0U);
        fluid_model->getXSPH()->setValue<Real>(SPH::XSPH::FLUID_COEFFICIENT, 0.0);
        fluid_model->getXSPH()->setValue<Real>(
            SPH::XSPH::BOUNDARY_COEFFICIENT,
            0.0);

        auto *rigid_body = new SPH::StaticRigidBody();
        rigid_body->setPosition0(Vector3r::Zero());
        rigid_body->setPosition(Vector3r::Zero());
        rigid_body->setRotation0(Quaternionr::Identity());
        rigid_body->setRotation(Quaternionr::Identity());
        auto *boundary_model = new SPH::BoundaryModel_Akinci2012();
        boundary_model->initModel(
            rigid_body,
            checked_u32(boundary_positions.size(), "BOUNDARY_COUNT"),
            boundary_positions.data());
        simulation->addBoundaryModel(boundary_model);

        simulation->setSimulationMethod(SPH::Simulation::ENUM_SIMULATION_DFSPH);
        auto *time_step = dynamic_cast<SPH::TimeStepDFSPH *>(simulation->getTimeStep());
        if (time_step == nullptr) {
            throw std::runtime_error("DFSPH_TIME_STEP_NOT_SELECTED");
        }
        time_step->setValue<unsigned int>(SPH::TimeStepDFSPH::MIN_ITERATIONS, 2U);
        time_step->setValue<unsigned int>(
            SPH::TimeStepDFSPH::MAX_ITERATIONS,
            pressure_cap);
        time_step->setValue<Real>(SPH::TimeStepDFSPH::MAX_ERROR, 0.01);
        time_step->setValue<unsigned int>(SPH::TimeStepDFSPH::MAX_ITERATIONS_V, 100U);
        time_step->setValue<Real>(SPH::TimeStepDFSPH::MAX_ERROR_V, 0.1);
        time_step->setValue<bool>(SPH::TimeStepDFSPH::USE_DIVERGENCE_SOLVER, true);
        SPH::TimeManager::getCurrent()->setTime(0.0);
        SPH::TimeManager::getCurrent()->setTimeStepSize(DT);

        simulation->setSimulationInitialized(true);
        boundary_model->deferredInit();
        simulation->performNeighborhoodSearchSort();
        simulation->updateBoundaryVolume();

        FrameDiagnostics initial;
        std::optional<AggregateProjections> aggregates;
        if (mode == TrajectoryMode::R1D) {
            aggregates = make_aggregate_projections(frame_count);
            append_aggregate_frame(*aggregates, *fluid_model, initial);
        }
        append_frame(payload, *fluid_model, initial);
        std::uint32_t serialized_frames = 1U;
        std::uint32_t maximum_pressure_iterations = 0;
        std::uint32_t maximum_divergence_iterations = 0;
        std::uint64_t total_contact_hits = 0;
        std::uint32_t final_receiver_count = 0;
        trajectory_started = true;
        for (std::uint32_t step = 1; step <= total_steps; ++step) {
            std::vector<std::array<double, 3>> previous_positions(SAMPLE_COUNT);
            for (std::uint32_t id = 0; id < SAMPLE_COUNT; ++id) {
                previous_positions[id] = position_by_id(*fluid_model, id);
            }
            failure_phase = "upstream_step";
            time_step->step();
            failure_diagnostics = capture_solver_diagnostics(*time_step, step);
            failure_phase = "solver_validation";
            validate_solver_diagnostics(
                *failure_diagnostics,
                pressure_cap,
                mode != TrajectoryMode::R1C3PressureSweep);
            if (mode == TrajectoryMode::R1C3PressureSweep) {
                std::ostringstream output;
                output << "schema=" << schema_for(mode) << '\n'
                       << "contract_identity=" << contract_for(mode) << '\n'
                       << "scenario=" << scenario.id << '\n'
                       << "status=PASS\n"
                       << "simulation_created=true\n"
                       << "trajectory_started=true\n"
                       << "step=" << failure_diagnostics->step << '\n'
                       << "pressure_cap=" << pressure_cap << '\n'
                       << "pressure_iterations="
                       << failure_diagnostics->pressure_iterations << '\n'
                       << "pressure_error_bits="
                       << hex_u64(to_bits(failure_diagnostics->pressure_error)) << '\n'
                       << "pressure_converged=" << std::boolalpha
                       << failure_diagnostics->pressure_converged << '\n'
                       << "divergence_iterations="
                       << failure_diagnostics->divergence_iterations << '\n'
                       << "divergence_error_bits="
                       << hex_u64(to_bits(failure_diagnostics->divergence_error)) << '\n'
                       << "divergence_converged="
                       << failure_diagnostics->divergence_converged << '\n'
                       << "time_step_bits="
                       << hex_u64(failure_diagnostics->time_step_bits) << '\n'
                       << "payload_written=false\n"
                       << "diagnostic_only=true\n"
                       << "r1c_authorized=false\n"
                       << "r1d_authorized=false\n"
                       << "b4e_authorized=false\n";
                return {true, output.str()};
            }
            failure_phase = "contact_projection";
            project_contact_and_measure(
                *fluid_model,
                scenario,
                previous_positions,
                *failure_diagnostics);
            const FrameDiagnostics &diagnostics = *failure_diagnostics;
            maximum_pressure_iterations =
                std::max(maximum_pressure_iterations, diagnostics.pressure_iterations);
            maximum_divergence_iterations =
                std::max(maximum_divergence_iterations, diagnostics.divergence_iterations);
            final_receiver_count = diagnostics.receiver_count;
            for (std::uint32_t count : diagnostics.feature_counts) {
                total_contact_hits += count;
            }
            if ((step % output_stride) == 0U) {
                if (aggregates.has_value()) {
                    append_aggregate_frame(*aggregates, *fluid_model, diagnostics);
                }
                append_frame(payload, *fluid_model, diagnostics);
                ++serialized_frames;
            }
        }
        failure_phase = "serialization";
        if (serialized_frames != frame_count) {
            throw std::runtime_error("CWREFV2_FRAME_COUNT_MISMATCH");
        }
        const std::size_t expected_size =
            20U + manifest.size() + (FRAME_BYTES * frame_count);
        if (expected_size != computed_expected_payload_bytes
            || payload.size() != expected_size) {
            throw std::runtime_error("CWREFV2_SIZE_MISMATCH");
        }
        if (mode == TrajectoryMode::R1D
            && payload.size() != frozen_expected_payload_bytes) {
            throw std::runtime_error("R1D_FROZEN_PAYLOAD_SIZE_MISMATCH");
        }
        if (payload.size() > MAX_FILE_BYTES) {
            throw std::runtime_error("CWREFV2_EXCEEDS_CAPACITY");
        }
        const std::string payload_hash = nextengine::nonlocal::sha256_hex(
            std::string_view(
                reinterpret_cast<const char *>(payload.data()),
                payload.size()));
        failure_phase = "publication";
        const std::string filename =
            write_payload(directory, scenario.id, payload);

        std::ostringstream output;
        output << "schema=" << schema_for(mode) << '\n'
               << "contract_identity=" << contract_for(mode) << '\n'
               << "scenario=" << scenario.id << '\n'
               << "status=PASS\n"
               << "simulation_created=true\n"
               << "trajectory_started=true\n"
               << "frames=" << frame_count << '\n'
               << "samples=" << SAMPLE_COUNT << '\n'
               << "payload_file=" << filename << '\n'
               << "payload_bytes=" << payload.size() << '\n'
               << "payload_sha256=" << payload_hash << '\n'
               << "max_pressure_iterations=" << maximum_pressure_iterations << '\n'
               << "max_divergence_iterations=" << maximum_divergence_iterations << '\n'
               << "total_contact_hits=" << total_contact_hits << '\n'
               << "final_receiver_count=" << final_receiver_count << '\n';
        if (mode == TrajectoryMode::R1D) {
            if (!aggregates.has_value()) {
                throw std::runtime_error("R1D_AGGREGATES_NOT_INITIALIZED");
            }
            output << "total_steps=" << total_steps << '\n'
                   << "output_stride=" << output_stride << '\n'
                   << "scenario_manifest_root=" << scenario_manifest_root << '\n'
                   << "q99_x_root=" << projection_hash(aggregates->q99_x) << '\n'
                   << "q99_y_root=" << projection_hash(aggregates->q99_y) << '\n'
                   << "receiver_count_root="
                   << projection_hash(aggregates->receiver_count) << '\n'
                   << "full_generation_candidate=true\n";
        }
        if (mode == TrajectoryMode::R1C2) {
            output << "diagnostic_only=true\n"
                   << "r1c_authorized=false\n";
        }
        if (mode == TrajectoryMode::R1D) {
            output << "r1e_authorized=false\n"
                   << "b4e_authorized=false\n";
        } else {
            output << "r1d_authorized=false\n"
                   << "b4e_authorized=false\n";
        }
        return {true, output.str()};
    } catch (const std::exception &error) {
        return {
            false,
            failure_report(
                error.what(),
                simulation_created,
                trajectory_started,
                mode,
                pressure_cap,
                failure_phase,
                failure_diagnostics),
        };
    }
}

} // namespace

AdapterRun run_r1c_trajectory(std::string_view scenario_id, std::string_view output_dir) {
    return run_r1c_trajectory_impl(
        scenario_id,
        output_dir,
        TrajectoryMode::R1C,
        100U);
}

AdapterRun run_r1c_trajectory_diagnostic(
    std::string_view scenario_id,
    std::string_view output_dir) {
    return run_r1c_trajectory_impl(
        scenario_id,
        output_dir,
        TrajectoryMode::R1C2,
        100U);
}

AdapterRun run_r1c_pressure_cap_sweep_point(
    std::string_view pressure_cap,
    std::string_view output_dir) {
    constexpr std::array<std::uint32_t, 8> ALLOWED_CAPS = {
        25U, 50U, 75U, 100U, 125U, 150U, 200U, 300U,
    };
    std::optional<std::uint32_t> parsed_cap;
    for (std::uint32_t allowed : ALLOWED_CAPS) {
        if (pressure_cap == std::to_string(allowed)) {
            parsed_cap = allowed;
            break;
        }
    }
    if (!parsed_cap.has_value()) {
        return {
            false,
            failure_report(
                "PRESSURE_CAP_NOT_ALLOWED",
                false,
                false,
                TrajectoryMode::R1C3PressureSweep,
                0U,
                "pre_simulation",
                std::nullopt),
        };
    }
    return run_r1c_trajectory_impl(
        "CW-HYDRO-001",
        output_dir,
        TrajectoryMode::R1C3PressureSweep,
        *parsed_cap);
}

AdapterRun run_r1c4_trajectory(
    std::string_view scenario_id,
    std::string_view output_dir) {
    return run_r1c_trajectory_impl(
        scenario_id,
        output_dir,
        TrajectoryMode::R1C4,
        300U);
}

AdapterRun run_r1c5_trajectory(
    std::string_view scenario_id,
    std::string_view output_dir) {
    return run_r1c_trajectory_impl(
        scenario_id,
        output_dir,
        TrajectoryMode::R1C5,
        300U);
}

AdapterRun run_r1d_manifest_preflight(bool force_schedule_mismatch) {
    return run_r1d_manifest_preflight_impl(force_schedule_mismatch);
}

AdapterRun run_r1d_generation(
    std::string_view scenario_id,
    std::string_view output_dir) {
    return run_r1c_trajectory_impl(
        scenario_id,
        output_dir,
        TrajectoryMode::R1D,
        300U);
}

} // namespace nextengine::nonlocal_reference
