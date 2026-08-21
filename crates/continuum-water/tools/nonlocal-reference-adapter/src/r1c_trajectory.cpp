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
#include <limits>
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
constexpr std::uint64_t DT_BITS = UINT64_C(0x3f71111111111111);
constexpr std::uint32_t SAMPLE_COUNT = 6'000;
constexpr std::uint32_t FRAME_COUNT = 25;
constexpr std::size_t FEATURE_COUNT = 25;
constexpr std::size_t MAX_FILE_BYTES = 64U * 1024U * 1024U;

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

FrameDiagnostics project_and_measure(
    SPH::FluidModel &model,
    SPH::TimeStepDFSPH &time_step,
    const R1CScenarioData &scenario,
    const std::vector<std::array<double, 3>> &previous_positions,
    std::uint32_t step) {
    FrameDiagnostics result;
    result.step = step;
    result.pressure_iterations = static_cast<std::uint32_t>(time_step.getNumIterations());
    result.divergence_iterations = time_step.getNumDivergenceIterations();
    result.pressure_error = time_step.getLastPressureError();
    result.divergence_error = time_step.getLastDivergenceError();
    require_finite(result.pressure_error, "PRESSURE_ERROR");
    require_finite(result.divergence_error, "DIVERGENCE_ERROR");
    if (!time_step.lastPressureConverged()) {
        throw std::runtime_error("PRESSURE_NOT_CONVERGED");
    }
    if (!time_step.lastDivergenceConverged()) {
        throw std::runtime_error("DIVERGENCE_NOT_CONVERGED");
    }
    if (result.pressure_iterations < 2U || result.pressure_iterations > 100U) {
        throw std::runtime_error("PRESSURE_ITERATIONS_OUT_OF_RANGE");
    }
    if (result.divergence_iterations < 1U || result.divergence_iterations > 100U) {
        throw std::runtime_error("DIVERGENCE_ITERATIONS_OUT_OF_RANGE");
    }
    if (to_bits(SPH::TimeManager::getCurrent()->getTimeStepSize()) != DT_BITS) {
        throw std::runtime_error("TIME_STEP_BITS_CHANGED");
    }

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
    return result;
}

std::string failure_report(
    std::string_view reason,
    bool simulation_created,
    bool trajectory_started) {
    std::ostringstream output;
    output << "schema=" << SCHEMA << '\n'
           << "contract_identity=" << CONTRACT_IDENTITY << '\n'
           << "status=FAIL\n"
           << "reason=" << reason << '\n'
           << "simulation_created=" << std::boolalpha << simulation_created << '\n'
           << "trajectory_started=" << trajectory_started << '\n';
    return output.str();
}

} // namespace

AdapterRun run_r1c_trajectory(std::string_view scenario_id, std::string_view output_dir) {
    bool simulation_created = false;
    bool trajectory_started = false;
    try {
        const AdapterRun manifest_preflight = run_r1c_manifest_preflight(false);
        if (!manifest_preflight.passed) {
            return {false, failure_report("MANIFEST_PREFLIGHT_NOT_PASS", false, false)};
        }
        const std::filesystem::path directory = validate_output_directory(output_dir);
        const R1CScenarioData scenario = build_r1c_scenario(scenario_id);

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
        payload.reserve(8U * 1024U * 1024U);
        const std::array<std::uint8_t, 8> magic = {'C', 'W', 'R', 'E', 'F', 'V', '2', 0};
        payload.insert(payload.end(), magic.begin(), magic.end());
        std::string manifest(r1c_profile_identity_projection());
        manifest.push_back('\n');
        manifest.append(scenario.manifest);
        append_u32(payload, checked_u32(manifest.size(), "MANIFEST_LENGTH"));
        payload.insert(payload.end(), manifest.begin(), manifest.end());
        append_u32(payload, SAMPLE_COUNT);
        append_u32(payload, FRAME_COUNT);

        SimulationGuard simulation_guard;
        SPH::Simulation *simulation = SPH::Simulation::getCurrent();
        simulation_created = true;
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
        time_step->setValue<unsigned int>(SPH::TimeStepDFSPH::MAX_ITERATIONS, 100U);
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
        append_frame(payload, *fluid_model, initial);
        std::uint32_t maximum_pressure_iterations = 0;
        std::uint32_t maximum_divergence_iterations = 0;
        std::uint64_t total_contact_hits = 0;
        std::uint32_t final_receiver_count = 0;
        trajectory_started = true;
        for (std::uint32_t step = 1; step < FRAME_COUNT; ++step) {
            std::vector<std::array<double, 3>> previous_positions(SAMPLE_COUNT);
            for (std::uint32_t id = 0; id < SAMPLE_COUNT; ++id) {
                previous_positions[id] = position_by_id(*fluid_model, id);
            }
            time_step->step();
            FrameDiagnostics diagnostics = project_and_measure(
                *fluid_model,
                *time_step,
                scenario,
                previous_positions,
                step);
            maximum_pressure_iterations =
                std::max(maximum_pressure_iterations, diagnostics.pressure_iterations);
            maximum_divergence_iterations =
                std::max(maximum_divergence_iterations, diagnostics.divergence_iterations);
            final_receiver_count = diagnostics.receiver_count;
            for (std::uint32_t count : diagnostics.feature_counts) {
                total_contact_hits += count;
            }
            append_frame(payload, *fluid_model, diagnostics);
        }
        const std::size_t expected_size =
            8U + 4U + manifest.size() + 4U + 4U + (312'156U * FRAME_COUNT);
        if (payload.size() != expected_size) {
            throw std::runtime_error("CWREFV2_SIZE_MISMATCH");
        }
        if (payload.size() > MAX_FILE_BYTES) {
            throw std::runtime_error("CWREFV2_EXCEEDS_CAPACITY");
        }
        const std::string payload_hash = nextengine::nonlocal::sha256_hex(
            std::string_view(
                reinterpret_cast<const char *>(payload.data()),
                payload.size()));
        const std::string filename =
            write_payload(directory, scenario.id, payload);

        std::ostringstream output;
        output << "schema=" << SCHEMA << '\n'
               << "contract_identity=" << CONTRACT_IDENTITY << '\n'
               << "scenario=" << scenario.id << '\n'
               << "status=PASS\n"
               << "simulation_created=true\n"
               << "trajectory_started=true\n"
               << "frames=" << FRAME_COUNT << '\n'
               << "samples=" << SAMPLE_COUNT << '\n'
               << "payload_file=" << filename << '\n'
               << "payload_bytes=" << payload.size() << '\n'
               << "payload_sha256=" << payload_hash << '\n'
               << "max_pressure_iterations=" << maximum_pressure_iterations << '\n'
               << "max_divergence_iterations=" << maximum_divergence_iterations << '\n'
               << "total_contact_hits=" << total_contact_hits << '\n'
               << "final_receiver_count=" << final_receiver_count << '\n'
               << "r1d_authorized=false\n"
               << "b4e_authorized=false\n";
        return {true, output.str()};
    } catch (const std::exception &error) {
        return {
            false,
            failure_report(error.what(), simulation_created, trajectory_started),
        };
    }
}

} // namespace nextengine::nonlocal_reference
