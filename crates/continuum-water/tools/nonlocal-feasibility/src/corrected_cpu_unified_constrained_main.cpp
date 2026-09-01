#include "corrected_cpu_unified_constrained.hpp"
#include "corrected_cuda_full_step.hpp"
#include "sha256.hpp"

#if __has_include("ncgp15_generated_compile_profile.hpp")
#include "ncgp15_generated_compile_profile.hpp"
#endif

#include <algorithm>
#include <array>
#include <cerrno>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <numeric>
#include <optional>
#include <set>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>
#include <tuple>
#include <utility>
#include <vector>

#if __cplusplus != 201703L
#error "NCGP15 requires exact ISO C++17 mode"
#endif

namespace nextengine::nonlocal::ncgp15 {
namespace {

#ifndef NCGP15_CONTRACT_ROOT
#define NCGP15_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP15_CONTRACT_SHA
#define NCGP15_CONTRACT_SHA "unconfigured"
#endif
#ifndef NCGP15_SOURCE_ROOT
#define NCGP15_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP15_SOURCE_COMMIT
#define NCGP15_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP15_SOURCE_TREE
#define NCGP15_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP15_COMPILER_FAMILY
#define NCGP15_COMPILER_FAMILY "unconfigured"
#endif
#ifndef NCGP15_COMPILER_VERSION
#define NCGP15_COMPILER_VERSION "unconfigured"
#endif
#ifndef NCGP15_COMPILER_FLAGS
#define NCGP15_COMPILER_FLAGS "unconfigured"
#endif
#ifndef NCGP15_GENERATED_COMPILE_COMMAND_ROOT
#define NCGP15_GENERATED_COMPILE_COMMAND_ROOT "unconfigured"
#endif
#ifndef NCGP15_GENERATED_COMPILE_COMMAND_MATERIAL_HEX
#define NCGP15_GENERATED_COMPILE_COMMAND_MATERIAL_HEX "unconfigured"
#endif
#ifndef NCGP15_PARENT_CONTRACT_FILE_SHA
#define NCGP15_PARENT_CONTRACT_FILE_SHA "unconfigured"
#endif
#ifndef NCGP15_PARENT_CONTRACT_ROOT
#define NCGP15_PARENT_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP15_PARENT_SOURCE_ROOT
#define NCGP15_PARENT_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP15_PARENT_SOURCE_COMMIT
#define NCGP15_PARENT_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP15_PARENT_SOURCE_TREE
#define NCGP15_PARENT_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP15_PARENT_BINARY_ROOT
#define NCGP15_PARENT_BINARY_ROOT "unconfigured"
#endif
#ifndef NCGP15_PARENT_STDOUT_ROOT
#define NCGP15_PARENT_STDOUT_ROOT "unconfigured"
#endif
#ifndef NCGP15_PARENT_RESULT_ROOT
#define NCGP15_PARENT_RESULT_ROOT "unconfigured"
#endif
#ifndef NCGP15_PARENT_TIGHT_FIXTURE_ROOT
#define NCGP15_PARENT_TIGHT_FIXTURE_ROOT "unconfigured"
#endif
#ifndef NCGP15_PARENT_TIGHT_LANE_ROOT
#define NCGP15_PARENT_TIGHT_LANE_ROOT "unconfigured"
#endif

constexpr std::string_view kSchema =
    "nextengine.nonlocal.ncgp15.result.v1";
constexpr std::string_view kInvocation =
    "nonlocal-corrected-cpu-unified-constrained "
    "--unified-constrained-surface-viscosity";
constexpr std::string_view kExpectedContractSha =
    "edce0d46d8ea1772e7c5ee6d5af21a46b27d5ebc623471cbfaf530032c5b6cf6";
constexpr std::string_view kExpectedContractCommit =
    "593801cd863aac9a335a21b56bdf24c315320a49";
constexpr std::string_view kExpectedContractTree =
    "1d0adcfca74a03332003f9072f3bdc5a5f67b27c";
constexpr std::string_view kExpectedParentFixture =
    "6dbaddf563b825e28e37aee58606f25cf50b17479cc3260e919103be79f7c2b2";
constexpr std::string_view kExpectedParentLane =
    "8d13d0eefb7e27acafb0588c052ed252a1246ec8e3c5cd5ebe4c748b59796ba6";
constexpr long double kPi15 =
    3.141592653589793238462643383279502884L;

void put_u8(std::string& bytes, std::uint8_t value) {
    bytes.push_back(static_cast<char>(value));
}

void put_u32(std::string& bytes, std::uint32_t value) {
    for (unsigned shift = 0U; shift < 32U; shift += 8U) {
        bytes.push_back(static_cast<char>((value >> shift) & 0xffU));
    }
}

void put_u64(std::string& bytes, std::uint64_t value) {
    for (unsigned shift = 0U; shift < 64U; shift += 8U) {
        bytes.push_back(static_cast<char>((value >> shift) & 0xffU));
    }
}

void put_string(std::string& bytes, std::string_view value) {
    put_u64(bytes, static_cast<std::uint64_t>(value.size()));
    bytes.append(value.data(), value.size());
}

void put_f32(std::string& bytes, float value) {
    if (!std::isfinite(value)) {
        throw std::runtime_error("NCGP15 nonfinite binary32 root field");
    }
    std::uint32_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value), "binary32 size mismatch");
    std::memcpy(&bits, &value, sizeof(bits));
    put_u32(bytes, bits);
}

void put_f32_raw(std::string& bytes, float value) {
    std::uint32_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value), "binary32 size mismatch");
    std::memcpy(&bits, &value, sizeof(bits));
    put_u32(bytes, bits);
}

double checked_f64(long double value) {
    if (!std::isfinite(value)) {
        throw std::runtime_error("NCGP15 nonfinite portable scalar");
    }
    const double narrowed = static_cast<double>(value);
    if (!std::isfinite(narrowed)) {
        throw std::runtime_error("NCGP15 binary64 narrowing overflow");
    }
    return narrowed;
}

void put_f64(std::string& bytes, long double value) {
    const double narrowed = checked_f64(value);
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(narrowed),
        "binary64 size mismatch");
    std::memcpy(&bits, &narrowed, sizeof(bits));
    put_u64(bytes, bits);
}

void put_vec(std::string& bytes, Vec3l value) {
    put_f64(bytes, value.x);
    put_f64(bytes, value.y);
    put_f64(bytes, value.z);
}

std::string root15(std::string bytes) {
    return sha256_hex(bytes);
}

bool root_shape15(std::string_view value) {
    return value.size() == 64U
        && std::all_of(value.begin(), value.end(), [](char item) {
            return (item >= '0' && item <= '9')
                || (item >= 'a' && item <= 'f');
        });
}

bool git_object_shape15(std::string_view value) {
    return value.size() == 40U
        && std::all_of(value.begin(), value.end(), [](char item) {
            return (item >= '0' && item <= '9')
                || (item >= 'a' && item <= 'f');
        });
}

bool exact_string(std::string_view lhs, std::string_view rhs) {
    return lhs == rhs;
}

std::optional<std::string> decode_hex15(std::string_view value) {
    if ((value.size() & 1U) != 0U) return std::nullopt;
    const auto nibble = [](char item) -> std::optional<std::uint8_t> {
        if (item >= '0' && item <= '9') {
            return static_cast<std::uint8_t>(item - '0');
        }
        if (item >= 'a' && item <= 'f') {
            return static_cast<std::uint8_t>(10 + item - 'a');
        }
        return std::nullopt;
    };
    std::string bytes;
    bytes.reserve(value.size() / 2U);
    for (std::size_t index = 0U; index < value.size(); index += 2U) {
        const std::optional<std::uint8_t> high = nibble(value[index]);
        const std::optional<std::uint8_t> low = nibble(value[index + 1U]);
        if (!high.has_value() || !low.has_value()) return std::nullopt;
        bytes.push_back(static_cast<char>((*high << 4U) | *low));
    }
    return bytes;
}

long double canonical_f32(long double value) {
    return static_cast<long double>(static_cast<float>(value));
}

Vec3l canonical_vec(Vec3l value) {
    return {canonical_f32(value.x), canonical_f32(value.y),
        canonical_f32(value.z)};
}

bool finite_state15(const State15& state) {
    return std::all_of(state.dynamic.begin(), state.dynamic.end(),
               [](const Dynamic15& item) {
                   return finite15(item.reference) && finite15(item.position)
                       && finite15(item.velocity);
               })
        && std::all_of(state.ghosts.begin(), state.ghosts.end(),
            [](const Ghost15& item) { return finite15(item.position); });
}

std::vector<std::size_t> stable_order15(const State15& state) {
    std::vector<std::size_t> order(state.dynamic.size());
    std::iota(order.begin(), order.end(), 0U);
    std::sort(order.begin(), order.end(), [&](std::size_t lhs,
                                           std::size_t rhs) {
        return state.dynamic[lhs].id < state.dynamic[rhs].id;
    });
    return order;
}

bool unique_ids15(const State15& state) {
    std::set<std::uint32_t> ids;
    for (const Dynamic15& item : state.dynamic) {
        if (!ids.insert(item.id).second) return false;
    }
    for (const Ghost15& item : state.ghosts) {
        if (!ids.insert(item.id).second) return false;
    }
    return true;
}

std::array<long double, 26> gate_values15(const Gates15& gates) {
    return {gates.inner_rms_relative, gates.inner_max_relative,
        gates.density_maximum, gates.density_rms, gates.kkt_rms_m,
        gates.kkt_maximum_m, gates.complementarity,
        gates.multiplier_fixed_point, gates.density_relative_l2,
        gates.gradient_relative_l2, gates.gradient_maximum_n,
        gates.pair_position_m, gates.normal_decay_relative,
        gates.tangential_change_absolute, gates.center_of_mass_relative,
        gates.surface_energy_relative_excess,
        gates.surface_energy_absolute_excess_j,
        gates.reversible_energy_drift, gates.one_step_position_rms_m,
        gates.one_step_position_maximum_m,
        gates.trajectory_position_rms_m,
        gates.trajectory_position_maximum_m,
        gates.trajectory_velocity_rms_mps,
        gates.trajectory_velocity_maximum_mps,
        gates.momentum_residual, gates.energy_excess};
}

std::string profile_root15(const Profile15& profile, Work15* work = nullptr) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.profile.v1");
    put_string(bytes, profile.id);
    for (long double value : std::array<long double, 21>{profile.dt,
             profile.spacing, profile.horizon, profile.surface_r0,
             profile.mass, profile.rho0, profile.kernel_scale, profile.beta,
             profile.lambda_v, profile.mu_v, profile.gamma,
             profile.gravity.x, profile.gravity.y, profile.gravity.z,
             profile.basin.x, profile.basin.y, profile.basin.z,
             static_cast<long double>(profile.ghost_layers),
             static_cast<long double>(profile.maximum_dynamic_samples),
             static_cast<long double>(profile.maximum_neighbors),
             static_cast<long double>(profile.maximum_outer_updates)}) {
        put_f64(bytes, value);
    }
    put_u64(bytes, profile.maximum_accepted_inner_iterations);
    put_u32(bytes, profile.maximum_line_search_backtracks);
    for (long double value : std::array<long double, 4>{profile.armijo,
             profile.initial_alpha, profile.line_search_shrink,
             profile.finite_difference_scale}) {
        put_f64(bytes, value);
    }
    for (long double value : gate_values15(profile.gates)) put_f64(bytes, value);
    put_f64(bytes, profile.gates.internal_force_closure);
    if (work != nullptr) {
        work->portable_fields += 56U;
        ++work->hash_derivations;
    }
    return root15(std::move(bytes));
}

std::string parent_profile_root15(const Profile15& profile) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.profile.v1");
    put_string(bytes, profile.id);
    for (long double value : std::array<long double, 19>{profile.dt,
             profile.spacing, profile.horizon, profile.mass, profile.rho0,
             profile.kernel_scale, profile.beta, 0.0L, 0.0L, 0.0L,
             profile.gravity.x, profile.gravity.y, profile.gravity.z,
             profile.basin.x, profile.basin.y, profile.basin.z,
             static_cast<long double>(profile.ghost_layers),
             static_cast<long double>(profile.maximum_dynamic_samples),
             static_cast<long double>(profile.maximum_neighbors)}) {
        put_f64(bytes, value);
    }
    return root15(std::move(bytes));
}

void append_base_dynamic15(std::string& bytes, const Dynamic15& item,
    bool raw = false) {
    put_u32(bytes, item.id);
    for (Vec3l value :
         std::array<Vec3l, 3>{item.reference, item.position, item.velocity}) {
        const float x = static_cast<float>(value.x);
        const float y = static_cast<float>(value.y);
        const float z = static_cast<float>(value.z);
        if (raw) {
            put_f32_raw(bytes, x);
            put_f32_raw(bytes, y);
            put_f32_raw(bytes, z);
        } else {
            put_f32(bytes, x);
            put_f32(bytes, y);
            put_f32(bytes, z);
        }
    }
}

void append_base_ghost15(std::string& bytes, const Ghost15& item,
    bool raw = false) {
    put_u32(bytes, item.id);
    const float x = static_cast<float>(item.position.x);
    const float y = static_cast<float>(item.position.y);
    const float z = static_cast<float>(item.position.z);
    if (raw) {
        put_f32_raw(bytes, x);
        put_f32_raw(bytes, y);
        put_f32_raw(bytes, z);
    } else {
        put_f32(bytes, x);
        put_f32(bytes, y);
        put_f32(bytes, z);
    }
}

std::string parent_fixture_root15(const Profile15& profile,
    const State15& state) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.fixture.v1");
    put_string(bytes, "TIGHT-128");
    put_string(bytes, parent_profile_root15(profile));
    put_u64(bytes, state.dynamic.size());
    for (const Dynamic15& item : state.dynamic) append_base_dynamic15(bytes, item);
    put_u64(bytes, state.ghosts.size());
    for (const Ghost15& item : state.ghosts) append_base_ghost15(bytes, item);
    return root15(std::move(bytes));
}

struct Input15 {
    std::string phase;
    std::string lane;
    std::string profile_root;
    std::string parent_fixture_root;
    std::string coordinate_tag = "BASE_BINARY32_WIDENED_LONG_DOUBLE";
    State15 base;
    Vec3l translation;
    GravityMode15 gravity_mode = GravityMode15::Profile;
    BoundaryMode15 boundary_mode = BoundaryMode15::AnalyticBox;
    TermMask15 terms;
    Mutation15 mutation = Mutation15::None;
    std::string root;
    Work15 root_work;
};

std::string input_root15(const Input15& input, Work15* work = nullptr) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.input.v1");
    put_string(bytes, input.phase);
    put_string(bytes, input.lane);
    put_string(bytes, input.profile_root);
    put_string(bytes, input.parent_fixture_root);
    put_string(bytes, input.coordinate_tag);
    const bool raw = input.coordinate_tag == "MALFORMED_RAW_BINARY32";
    put_u64(bytes, input.base.dynamic.size());
    for (const Dynamic15& item : input.base.dynamic) {
        append_base_dynamic15(bytes, item, raw);
    }
    put_u64(bytes, input.base.ghosts.size());
    for (const Ghost15& item : input.base.ghosts) {
        append_base_ghost15(bytes, item, raw);
    }
    put_vec(bytes, input.translation);
    put_u8(bytes, static_cast<std::uint8_t>(input.gravity_mode));
    put_u8(bytes, static_cast<std::uint8_t>(input.boundary_mode));
    put_u8(bytes, input.terms.density_constraint ? 1U : 0U);
    put_u8(bytes, input.terms.normal_viscosity ? 1U : 0U);
    put_u8(bytes, input.terms.tangential_viscosity ? 1U : 0U);
    put_u8(bytes, input.terms.surface ? 1U : 0U);
    put_u8(bytes, static_cast<std::uint8_t>(input.mutation));
    if (work != nullptr) {
        work->portable_fields += 18U + 10U * input.base.dynamic.size()
            + 4U * input.base.ghosts.size();
        ++work->hash_derivations;
    }
    return root15(std::move(bytes));
}

State15 materialize_input15(const Input15& input) {
    State15 state = input.base;
    for (Dynamic15& item : state.dynamic) {
        item.reference += input.translation;
        item.position += input.translation;
    }
    for (Ghost15& item : state.ghosts) item.position += input.translation;
    return state;
}

std::string state_root15(std::string_view input_root, std::uint32_t step,
    const State15& state, Work15* work = nullptr) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.state.v1");
    put_string(bytes, input_root);
    put_u32(bytes, step);
    const std::vector<std::size_t> order = stable_order15(state);
    put_u64(bytes, order.size());
    for (std::size_t index : order) {
        const Dynamic15& item = state.dynamic[index];
        put_u32(bytes, item.id);
        put_vec(bytes, item.reference);
        put_vec(bytes, item.position);
        put_vec(bytes, item.velocity);
    }
    if (work != nullptr) {
        work->portable_fields += 4U + 10U * order.size();
        ++work->hash_derivations;
    }
    return root15(std::move(bytes));
}

std::string pressure_root15(std::string_view state_root,
    const State15& state, const std::vector<long double>& pi,
    const Profile15& profile, Work15* work = nullptr) {
    if (pi.size() != state.dynamic.size()) {
        throw std::runtime_error("NCGP15 pressure vector size mismatch");
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.pressure.v1");
    put_string(bytes, state_root);
    const std::vector<std::size_t> order = stable_order15(state);
    put_u64(bytes, order.size());
    for (std::size_t index : order) {
        put_u32(bytes, state.dynamic[index].id);
        put_f64(bytes, pi[index]);
        put_f64(bytes, (profile.rho0 / profile.mass) * pi[index]);
        put_u8(bytes, pi[index] > 0.0L ? 1U : 0U);
    }
    if (work != nullptr) {
        work->portable_fields += 3U + 4U * order.size();
        ++work->hash_derivations;
    }
    return root15(std::move(bytes));
}

std::string contact_root15(std::string_view state_root,
    const std::vector<Contact15>& contact, Work15* work = nullptr) {
    std::vector<Contact15> ordered = contact;
    std::sort(ordered.begin(), ordered.end(), [](const Contact15& lhs,
                                               const Contact15& rhs) {
        return lhs.id < rhs.id;
    });
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.contact.v1");
    put_string(bytes, state_root);
    put_u64(bytes, ordered.size());
    for (const Contact15& item : ordered) {
        put_u32(bytes, item.id);
        put_u8(bytes, item.mask);
        put_vec(bytes, item.impulse);
    }
    if (work != nullptr) {
        work->portable_fields += 3U + 5U * ordered.size();
        ++work->hash_derivations;
    }
    return root15(std::move(bytes));
}

std::string work_root15(const Work15& work) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.work.v1");
    for (std::uint64_t value : work_values15(work)) put_u64(bytes, value);
    return root15(std::move(bytes));
}

void append_bool(std::string& bytes, bool value) {
    put_u8(bytes, value ? 1U : 0U);
}

void append_id_vec_array15(std::string& bytes, const State15& state,
    const std::vector<Vec3l>& values) {
    const std::vector<std::size_t> order = stable_order15(state);
    const std::size_t count = std::min(order.size(), values.size());
    put_u64(bytes, count);
    for (std::size_t ordered = 0U; ordered < count; ++ordered) {
        const std::size_t index = order[ordered];
        put_u32(bytes, state.dynamic[index].id);
        put_vec(bytes, values[index]);
    }
}

void append_id_scalar_array15(std::string& bytes, const State15& state,
    const std::vector<long double>& values) {
    const std::vector<std::size_t> order = stable_order15(state);
    const std::size_t count = std::min(order.size(), values.size());
    put_u64(bytes, count);
    for (std::size_t ordered = 0U; ordered < count; ++ordered) {
        const std::size_t index = order[ordered];
        put_u32(bytes, state.dynamic[index].id);
        put_f64(bytes, values[index]);
    }
}

struct DensityGraphRecord15 {
    std::uint32_t owner = 0U;
    std::uint32_t neighbor = 0U;
    bool ghost = false;
};

struct PairRecord15 {
    std::uint32_t first = 0U;
    std::uint32_t second = 0U;
};

struct GraphObservables15 {
    std::vector<DensityGraphRecord15> density;
    std::vector<PairRecord15> surface;
    std::vector<PairRecord15> viscosity;
    std::uint32_t maximum_degree = 0U;
    bool capacity_overflow = false;
    bool invalid = false;
};

bool counted_predicate_main15(Work15& work, bool value) {
    ++work.scalar_predicates;
    return value;
}

GraphObservables15 graph_observables15(const State15& accepted,
    const Step15& step, const SolverOptions15& options, Work15& work) {
    GraphObservables15 result;
    work.graph_builds += 2U;
    const State15& state = step.state.dynamic.empty() ? accepted : step.state;
    const auto current_order = stable_order15(state);
    const auto accepted_order = stable_order15(accepted);
    std::vector<std::size_t> ghost_order(state.ghosts.size());
    std::iota(ghost_order.begin(), ghost_order.end(), 0U);
    std::sort(ghost_order.begin(), ghost_order.end(),
        [&](std::size_t lhs, std::size_t rhs) {
            return state.ghosts[lhs].id < state.ghosts[rhs].id;
        });
    for (std::size_t owner_order = 0U; owner_order < current_order.size();
         ++owner_order) {
        const std::size_t owner_index = current_order[owner_order];
        const Dynamic15& owner = state.dynamic[owner_index];
        std::uint32_t degree = 0U;
        for (std::size_t neighbor_index : current_order) {
            ++work.graph_candidates;
            const Dynamic15& neighbor = state.dynamic[neighbor_index];
            const long double r = norm15(owner.position - neighbor.position);
            if (!counted_predicate_main15(work,
                    r <= options.profile.horizon)) continue;
            result.density.push_back({owner.id, neighbor.id, false});
            ++work.accepted_pairs;
            ++degree;
        }
        for (std::size_t ghost_index : ghost_order) {
            ++work.graph_candidates;
            const Ghost15& ghost = state.ghosts[ghost_index];
            const long double r = norm15(owner.position - ghost.position);
            const bool within = counted_predicate_main15(work,
                r <= options.profile.horizon);
            const bool positive = counted_predicate_main15(work, r > 0.0L);
            if (within && !positive) result.invalid = true;
            if (!within) continue;
            result.density.push_back({owner.id, ghost.id, true});
            ++work.accepted_pairs;
            ++degree;
        }
        result.maximum_degree = std::max(result.maximum_degree, degree);
        if (counted_predicate_main15(work,
                degree > options.profile.maximum_neighbors)) {
            result.capacity_overflow = true;
        }
        for (std::size_t rhs_order = owner_order + 1U;
             rhs_order < current_order.size(); ++rhs_order) {
            ++work.surface_pair_tests;
            const Dynamic15& rhs = state.dynamic[current_order[rhs_order]];
            const long double r = norm15(owner.position - rhs.position);
            const bool positive = counted_predicate_main15(work, r > 0.0L);
            const bool within = counted_predicate_main15(work,
                r < 3.0L * options.profile.surface_r0);
            if (!positive) result.invalid = true;
            if (positive && within) {
                result.surface.push_back({owner.id, rhs.id});
                ++work.surface_active_pairs;
            }
        }
    }
    for (std::size_t a = 0U; a < accepted_order.size(); ++a) {
        const Dynamic15& lhs = accepted.dynamic[accepted_order[a]];
        for (std::size_t b = a + 1U; b < accepted_order.size(); ++b) {
            const Dynamic15& rhs = accepted.dynamic[accepted_order[b]];
            const long double r = norm15(lhs.position - rhs.position);
            const bool positive = counted_predicate_main15(work, r > 0.0L);
            const bool within = counted_predicate_main15(work,
                r <= options.profile.horizon);
            if (!positive) result.invalid = true;
            if (!(positive && within)) continue;
            result.viscosity.push_back({lhs.id, rhs.id});
            ++work.viscosity_pairs;
        }
    }
    return result;
}

struct ExpectedGraphObservables15 {
    GraphObservables15 value;
    Work15 work;
};

ExpectedGraphObservables15 expected_graph_observables15(const State15& accepted,
    const State15& current, const SolverOptions15& options) {
    ExpectedGraphObservables15 result;
    Work15& expected = result.work;
    expected.graph_builds = 2U;
    const std::uint64_t dynamic = accepted.dynamic.size();
    const std::uint64_t ghosts = accepted.ghosts.size();
    expected.graph_candidates = dynamic * (dynamic + ghosts);
    expected.surface_pair_tests = dynamic
        * (dynamic == 0U ? 0U : dynamic - 1U) / 2U;
    const std::uint64_t accepted_dynamic = accepted.dynamic.size();
    const std::uint64_t viscosity_tests = accepted_dynamic
        * (accepted_dynamic == 0U ? 0U : accepted_dynamic - 1U) / 2U;
    expected.scalar_predicates = expected.graph_candidates
        + dynamic * ghosts + 2U * expected.surface_pair_tests
        + 2U * viscosity_tests + dynamic;
    const std::vector<std::size_t> current_order = stable_order15(current);
    const std::vector<std::size_t> accepted_order = stable_order15(accepted);
    std::vector<std::size_t> ghost_order(current.ghosts.size());
    std::iota(ghost_order.begin(), ghost_order.end(), 0U);
    std::sort(ghost_order.begin(), ghost_order.end(),
        [&](std::size_t lhs, std::size_t rhs) {
            return current.ghosts[lhs].id < current.ghosts[rhs].id;
        });
    for (std::size_t owner_order = 0U; owner_order < current_order.size();
         ++owner_order) {
        const Dynamic15& owner = current.dynamic[current_order[owner_order]];
        std::uint32_t degree = 0U;
        for (std::size_t neighbor_index : current_order) {
            const Dynamic15& neighbor = current.dynamic[neighbor_index];
            if (norm15(owner.position - neighbor.position)
                    <= options.profile.horizon) {
                ++expected.accepted_pairs;
                ++degree;
                result.value.density.push_back(
                    {owner.id, neighbor.id, false});
            }
        }
        for (std::size_t ghost_index : ghost_order) {
            const Ghost15& ghost = current.ghosts[ghost_index];
            const long double distance = norm15(owner.position
                - ghost.position);
            if (distance <= options.profile.horizon) {
                ++expected.accepted_pairs;
                ++degree;
                result.value.density.push_back({owner.id, ghost.id, true});
                result.value.invalid = result.value.invalid
                    || !(distance > 0.0L);
            }
        }
        result.value.maximum_degree = std::max(
            result.value.maximum_degree, degree);
        result.value.capacity_overflow = result.value.capacity_overflow
            || degree > options.profile.maximum_neighbors;
        for (std::size_t rhs_order = owner_order + 1U;
             rhs_order < current_order.size(); ++rhs_order) {
            const Dynamic15& rhs = current.dynamic[current_order[rhs_order]];
            const long double r = norm15(owner.position - rhs.position);
            if (r > 0.0L && r < 3.0L * options.profile.surface_r0) {
                ++expected.surface_active_pairs;
                result.value.surface.push_back({owner.id, rhs.id});
            }
            result.value.invalid = result.value.invalid || !(r > 0.0L);
        }
    }
    for (std::size_t lhs_order = 0U; lhs_order < accepted_order.size();
         ++lhs_order) {
        const Dynamic15& lhs = accepted.dynamic[accepted_order[lhs_order]];
        for (std::size_t rhs_order = lhs_order + 1U;
             rhs_order < accepted_order.size(); ++rhs_order) {
            const Dynamic15& rhs = accepted.dynamic[accepted_order[rhs_order]];
            const long double r = norm15(lhs.position - rhs.position);
            if (r > 0.0L && r <= options.profile.horizon) {
                ++expected.viscosity_pairs;
                result.value.viscosity.push_back({lhs.id, rhs.id});
            }
            result.value.invalid = result.value.invalid || !(r > 0.0L);
        }
    }
    return result;
}

struct StepGate15 {
    bool finite = false;
    bool nonnegative_multipliers = false;
    bool density = false;
    bool kkt = false;
    bool complementarity = false;
    bool multiplier_fixed_point = false;
    bool active_signature = false;
    bool containment = false;
    bool capacity = false;
    bool trace = false;
    bool identity = false;
    bool density_oracle = false;
    bool manufactured_active_empty = false;
    bool accepted = false;
};

struct DensityCross15 {
    long double relative_l2 = 0.0L;
    bool constraint_signature_exact = false;
    bool pass = false;
    Work15 actual_work;
    Work15 expected_work;
};

long double pairwise_scalar_sum15(std::vector<long double> values);

std::vector<std::pair<std::uint32_t, std::size_t>> id_index15(
    const State15& state);

DensityCross15 density_cross15(const State15& accepted,
    const Step15& step, const SolverOptions15& options) {
    DensityCross15 result;
    State15 canonical_accepted = accepted;
    std::sort(canonical_accepted.dynamic.begin(),
        canonical_accepted.dynamic.end(), [](const Dynamic15& lhs,
                                             const Dynamic15& rhs) {
            return lhs.id < rhs.id;
        });
    std::sort(canonical_accepted.ghosts.begin(), canonical_accepted.ghosts.end(),
        [](const Ghost15& lhs, const Ghost15& rhs) {
            return lhs.id < rhs.id;
        });
    const auto current_index = id_index15(step.state);
    const bool size_exact = counted_predicate_main15(result.actual_work,
        current_index.size() == canonical_accepted.dynamic.size());
    ++result.expected_work.scalar_predicates;
    std::vector<Vec3l> positions;
    positions.reserve(canonical_accepted.dynamic.size());
    bool ids_exact = size_exact;
    const std::size_t count = std::min(current_index.size(),
        canonical_accepted.dynamic.size());
    result.expected_work.scalar_predicates +=
        canonical_accepted.dynamic.size();
    for (std::size_t index = 0U; index < count; ++index) {
        const bool id_exact = counted_predicate_main15(result.actual_work,
            current_index[index].first
                == canonical_accepted.dynamic[index].id);
        ids_exact = id_exact && ids_exact;
        positions.push_back(step.state.dynamic[
            current_index[index].second].position);
    }
    if (!ids_exact) return result;
    Work15 candidate_work;
    Work15 oracle_work;
    const Density15 candidate = candidate_density15(canonical_accepted,
        positions, options, candidate_work);
    const Density15 oracle = oracle_density15(canonical_accepted,
        positions, options, oracle_work);
    add_work15(result.actual_work, candidate_work);
    add_work15(result.actual_work, oracle_work);
    add_work15(result.expected_work, candidate_expected_density_work15(
        canonical_accepted, positions, options));
    add_work15(result.expected_work, oracle_expected_density_work15(
        canonical_accepted, positions, options));
    bool payload_exact = counted_predicate_main15(result.actual_work,
        candidate.valid);
    payload_exact = counted_predicate_main15(result.actual_work,
        oracle.valid) && payload_exact;
    payload_exact = counted_predicate_main15(result.actual_work,
        candidate.rho.size() == positions.size()) && payload_exact;
    payload_exact = counted_predicate_main15(result.actual_work,
        oracle.rho.size() == positions.size()) && payload_exact;
    result.expected_work.scalar_predicates += 4U;
    if (!payload_exact) return result;
    std::vector<long double> error_squares;
    std::vector<long double> oracle_squares;
    error_squares.reserve(positions.size());
    oracle_squares.reserve(positions.size());
    bool signature_exact = true;
    for (std::size_t index = 0U; index < positions.size(); ++index) {
        const long double difference = candidate.rho[index]
            - oracle.rho[index];
        error_squares.push_back(difference * difference);
        oracle_squares.push_back(oracle.rho[index] * oracle.rho[index]);
        const bool candidate_positive = counted_predicate_main15(
            result.actual_work,
            candidate.rho[index] / options.profile.rho0 - 1.0L > 0.0L);
        const bool oracle_positive = counted_predicate_main15(
            result.actual_work,
            oracle.rho[index] / options.profile.rho0 - 1.0L > 0.0L);
        signature_exact = counted_predicate_main15(result.actual_work,
            candidate_positive == oracle_positive) && signature_exact;
        result.expected_work.scalar_predicates += 3U;
    }
    const long double denominator = pairwise_scalar_sum15(
        std::move(oracle_squares));
    const bool denominator_positive = counted_predicate_main15(
        result.actual_work, denominator > 0.0L);
    ++result.expected_work.scalar_predicates;
    if (denominator_positive) {
        result.relative_l2 = std::sqrt(pairwise_scalar_sum15(
            std::move(error_squares)) / denominator);
    }
    result.constraint_signature_exact = signature_exact;
    const bool relative_pass = counted_predicate_main15(result.actual_work,
        result.relative_l2 <= options.profile.gates.density_relative_l2);
    ++result.expected_work.scalar_predicates;
    result.pass = denominator_positive && relative_pass;
    return result;
}

std::vector<std::uint32_t> active_ids15(const Step15& step);

bool trace_grammar15(const Step15& step, const SolverOptions15& options,
    Work15& work) {
    const auto& stages = step.trace.reached_stages;
    bool valid = counted_predicate_main15(work,
        !stages.empty() && stages.front() == "PROJECTION");
    const auto allowed_transition = [&](std::string_view previous,
                                        std::string_view next) {
        if (previous == "PROJECTION") {
            return next == "PROJECTION_BOX" || next == "PROJECTION"
                || next == "INNER_GATE" || next == "FINAL_GATE";
        }
        if (previous == "PROJECTION_BOX") {
            return next == "PROJECTION" || next == "INNER_GATE"
                || next == "CONTACT";
        }
        if (previous == "INNER_GATE") {
            return next == "SLOPE" || next == "MULTIPLIER_UPDATE";
        }
        if (previous == "SLOPE") return next == "LINE_TRIAL";
        if (previous == "LINE_TRIAL") return next == "ARMIJO";
        if (previous == "ARMIJO") {
            return next == "LINE_TRIAL" || next == "PROJECTION";
        }
        if (previous == "MULTIPLIER_UPDATE") {
            return next == "PROJECTION";
        }
        if (previous == "CONTACT") return next == "FINAL_GATE";
        if (previous == "FINAL_GATE") {
            return next == "TOPOLOGY" || next == "PROJECTION";
        }
        return false;
    };
    for (std::size_t index = 1U; index < stages.size(); ++index) {
        valid = counted_predicate_main15(work,
            allowed_transition(stages[index - 1U], stages[index])) && valid;
    }
    const auto count_stage = [&](std::string_view token) {
        return static_cast<std::uint64_t>(std::count(stages.begin(),
            stages.end(), token));
    };
    const std::uint64_t projections = count_stage("PROJECTION");
    const std::uint64_t projection_boxes = count_stage("PROJECTION_BOX");
    const std::uint64_t inner_gates = count_stage("INNER_GATE");
    const std::uint64_t slopes = count_stage("SLOPE");
    const std::uint64_t line_trials = count_stage("LINE_TRIAL");
    const std::uint64_t armijo = count_stage("ARMIJO");
    const std::uint64_t updates = count_stage("MULTIPLIER_UPDATE");
    const std::uint64_t contacts = count_stage("CONTACT");
    const std::uint64_t final_gates = count_stage("FINAL_GATE");
    const std::uint64_t topology = count_stage("TOPOLOGY");
    std::uint64_t mode0_gradient = 0U;
    std::uint64_t mode0_objective = 0U;
    std::uint64_t mode1_gradient = 0U;
    const std::size_t evaluations = std::min(
        step.trace.evaluation_modes.size(),
        step.trace.evaluation_gradient.size());
    for (std::size_t index = 0U; index < evaluations; ++index) {
        const std::uint8_t mode = step.trace.evaluation_modes[index];
        const std::uint8_t gradient = step.trace.evaluation_gradient[index];
        const bool pair_valid = (mode == 0U && gradient <= 1U)
            || (mode == 1U && gradient == 1U);
        valid = counted_predicate_main15(work, pair_valid) && valid;
        mode0_gradient += mode == 0U && gradient == 1U ? 1U : 0U;
        mode0_objective += mode == 0U && gradient == 0U ? 1U : 0U;
        mode1_gradient += mode == 1U && gradient == 1U ? 1U : 0U;
    }
    const bool box = options.boundary_mode == BoundaryMode15::AnalyticBox;
    const std::uint64_t accepted_line_trials =
        step.trace.accepted_inner_iterations
        + std::accumulate(step.trace.backtracks.begin(),
            step.trace.backtracks.end(), std::uint64_t{0U});
    const std::array<bool, 14> schedule{{
        projections == 1U + inner_gates + final_gates,
        slopes + updates == inner_gates,
        updates == step.trace.outer_updates,
        final_gates == updates,
        projection_boxes == (box ? projections : 0U),
        contacts == (box ? final_gates : 0U),
        topology == (step.accepted ? 1U : 0U),
        line_trials == armijo,
        mode0_gradient == inner_gates,
        mode0_objective == line_trials,
        mode1_gradient == final_gates,
        step.work.line_search_trials == line_trials,
        step.work.outer_multiplier_updates == updates,
        line_trials == accepted_line_trials}};
    for (bool predicate : schedule) {
        valid = counted_predicate_main15(work, predicate) && valid;
    }
    const bool terminal = step.accepted
        ? (!stages.empty() && stages.back() == "TOPOLOGY")
        : (step.work_ceiling && !stages.empty()
            && (stages.back() == "SLOPE"
                || stages.back() == "FINAL_GATE"));
    valid = counted_predicate_main15(work, terminal) && valid;
    return valid;
}

std::uint64_t expected_trace_grammar_predicates15(const Step15& step) {
    const std::uint64_t stages = step.trace.reached_stages.size();
    const std::uint64_t transitions = stages == 0U ? 0U : stages - 1U;
    const std::uint64_t evaluations = std::min(
        step.trace.evaluation_modes.size(),
        step.trace.evaluation_gradient.size());
    return 1U + transitions + evaluations + 14U + 1U;
}

Work15 expected_gate_work15(const State15& accepted, const Step15& step,
    const GraphObservables15&, const SolverOptions15& options) {
    Work15 result;
    const std::uint64_t dynamic = accepted.dynamic.size();
    const std::uint64_t ghost = accepted.ghosts.size();
    const bool final_payload = step.accepted
        || (!step.trace.reached_stages.empty()
            && step.trace.reached_stages.back() == "FINAL_GATE");
    const std::uint64_t contact = final_payload
            && options.boundary_mode == BoundaryMode15::AnalyticBox
        ? dynamic : 0U;
    const std::uint64_t semantic_vectors =
        (final_payload ? 6U : 4U) * dynamic;
    const std::uint64_t evaluation_vectors =
        step.trace.evaluation_positions.size() * dynamic;
    const std::uint64_t vector_records = semantic_vectors
        + evaluation_vectors;
    const std::uint64_t evaluation_multiplier_values =
        step.trace.evaluation_multipliers.size() * dynamic;
    const std::uint64_t fixed_active_schedule = dynamic == 0U
        ? 0U : 3U * dynamic - 1U;
    result.scalar_predicates = 9U * dynamic + 3U * ghost + 62U
        + dynamic + dynamic + 3U * contact
        + 3U * vector_records + step.trace.accepted_objectives.size()
        + step.trace.evaluation_positions.size()
        + evaluation_multiplier_values
        + step.trace.evaluation_multipliers.size()
        + step.trace.evaluation_modes.size()
        + step.trace.evaluation_gradient.size()
        + step.trace.backtracks.size()
        + 2U * step.trace.outer_inner_offsets.size()
        + step.trace.reached_stages.size()
        + dynamic + fixed_active_schedule
        + 2U + 4U * accepted.dynamic.size() + 4U * accepted.ghosts.size()
        + 2U * contact
        + (options.boundary_mode == BoundaryMode15::AnalyticBox
                ? 6U * dynamic : 0U);
    result.scalar_predicates += expected_trace_grammar_predicates15(step);
    return result;
}

StepGate15 gate_step15(const State15& accepted, const Step15& step,
    const GraphObservables15& graph, const SolverOptions15& options,
    const DensityCross15& density_cross, Work15& work) {
    StepGate15 gate;
    bool finite = true;
    const auto finite_scalar = [&](long double value) {
        finite = counted_predicate_main15(work, std::isfinite(value)) && finite;
    };
    const auto finite_vec = [&](Vec3l value) {
        finite_scalar(value.x);
        finite_scalar(value.y);
        finite_scalar(value.z);
    };
    for (const Dynamic15& item : step.state.dynamic) {
        finite_vec(item.reference);
        finite_vec(item.position);
        finite_vec(item.velocity);
    }
    for (const Ghost15& item : step.state.ghosts) finite_vec(item.position);
    for (long double value : std::array<long double, 11>{
             step.metrics.objective, step.metrics.nonpressure_energy,
             step.metrics.inertia_energy, step.metrics.viscosity_energy,
             step.metrics.surface_energy,
             step.metrics.maximum_positive_density_strain,
             step.metrics.rms_positive_density_strain,
             step.metrics.kkt_rms_m, step.metrics.kkt_maximum_m,
             step.metrics.complementarity,
             step.metrics.multiplier_fixed_point}) finite_scalar(value);
    for (long double value : step.pi) finite_scalar(value);
    for (long double value : step.rho) finite_scalar(value);
    for (const Contact15& item : step.contact) finite_vec(item.impulse);
    for (const std::vector<Vec3l>* values :
         std::array<const std::vector<Vec3l>*, 6>{&step.kkt_gradient,
             &step.kkt_direction, &step.dynamic_pressure_force,
             &step.ghost_pressure_force, &step.viscosity_force,
             &step.surface_force}) {
        for (Vec3l value : *values) finite_vec(value);
    }
    for (long double value : step.trace.accepted_objectives) {
        finite_scalar(value);
    }
    for (const std::vector<Vec3l>& positions :
         step.trace.evaluation_positions) {
        for (Vec3l value : positions) finite_vec(value);
    }
    for (const std::vector<long double>& multipliers :
         step.trace.evaluation_multipliers) {
        for (long double value : multipliers) finite_scalar(value);
    }
    for (std::uint8_t mode : step.trace.evaluation_modes) {
        finite = counted_predicate_main15(work, mode <= 1U) && finite;
    }
    bool trace_exact = true;
    for (std::uint8_t gradient : step.trace.evaluation_gradient) {
        trace_exact = counted_predicate_main15(work, gradient <= 1U)
            && trace_exact;
    }
    const std::size_t dynamic = step.state.dynamic.size();
    const std::size_t expected_contact =
        options.boundary_mode == BoundaryMode15::AnalyticBox ? dynamic : 0U;
    for (bool size_exact : std::array<bool, 9>{step.pi.size() == dynamic,
             step.rho.size() == dynamic,
             step.contact.size() == expected_contact,
             step.kkt_gradient.size() == dynamic,
             step.kkt_direction.size() == dynamic,
             step.dynamic_pressure_force.size() == dynamic,
             step.ghost_pressure_force.size() == dynamic,
             step.viscosity_force.size() == dynamic,
             step.surface_force.size() == dynamic}) {
        finite = counted_predicate_main15(work, size_exact) && finite;
    }
    for (const std::vector<Vec3l>& positions :
         step.trace.evaluation_positions) {
        finite = counted_predicate_main15(work,
            positions.size() == dynamic) && finite;
    }
    for (const std::vector<long double>& multipliers :
         step.trace.evaluation_multipliers) {
        finite = counted_predicate_main15(work,
            multipliers.size() == dynamic) && finite;
    }
    for (bool array_size_exact : std::array<bool, 3>{
             step.trace.evaluation_multipliers.size()
                 == step.trace.evaluation_positions.size(),
             step.trace.evaluation_modes.size()
                 == step.trace.evaluation_positions.size(),
             step.trace.evaluation_gradient.size()
                 == step.trace.evaluation_positions.size()}) {
        finite = counted_predicate_main15(work, array_size_exact) && finite;
    }
    for (bool fixed_trace_predicate : std::array<bool, 5>{
             step.trace.outer_updates
                 <= options.profile.maximum_outer_updates,
             step.trace.accepted_inner_iterations
                 <= options.profile.maximum_accepted_inner_iterations,
             step.trace.backtracks.size()
                 == step.trace.accepted_inner_iterations,
             step.trace.accepted_objectives.size()
                 == step.trace.accepted_inner_iterations,
             step.trace.outer_inner_offsets.size()
                 == step.trace.outer_updates}) {
        trace_exact = counted_predicate_main15(work, fixed_trace_predicate)
            && trace_exact;
    }
    for (std::uint32_t backtrack : step.trace.backtracks) {
        trace_exact = counted_predicate_main15(work,
            backtrack <= options.profile.maximum_line_search_backtracks)
            && trace_exact;
    }
    std::uint64_t previous_offset = 0U;
    for (std::uint64_t offset : step.trace.outer_inner_offsets) {
        const bool in_range = counted_predicate_main15(work,
            offset <= step.trace.accepted_inner_iterations);
        const bool monotone = counted_predicate_main15(work,
            offset >= previous_offset);
        trace_exact = in_range && monotone && trace_exact;
        previous_offset = offset;
    }
    trace_exact = counted_predicate_main15(work,
        step.trace.outer_inner_offsets.empty()
            ? step.trace.outer_updates == 0U
            : step.trace.outer_inner_offsets.back()
                == step.trace.accepted_inner_iterations) && trace_exact;
    constexpr std::array<std::string_view, 11> stage_vocabulary{
        "PROJECTION", "PROJECTION_BOX", "INNER_GATE", "SLOPE",
        "LINE_TRIAL", "ARMIJO", "MULTIPLIER_UPDATE", "CONTACT",
        "FINAL_GATE", "TOPOLOGY", "EVALUATION"};
    for (const std::string& stage : step.trace.reached_stages) {
        const bool known = std::find(stage_vocabulary.begin(),
            stage_vocabulary.end(), stage) != stage_vocabulary.end();
        trace_exact = counted_predicate_main15(work, known) && trace_exact;
    }
    const auto stage_count = [&](std::string_view token) {
        return static_cast<std::uint64_t>(std::count(
            step.trace.reached_stages.begin(),
            step.trace.reached_stages.end(), token));
    };
    for (bool schedule_predicate : std::array<bool, 5>{
             stage_count("MULTIPLIER_UPDATE") == step.trace.outer_updates,
             stage_count("TOPOLOGY") <= 1U,
             stage_count("FINAL_GATE") <= step.trace.outer_updates,
             stage_count("CONTACT") <= stage_count("FINAL_GATE"),
             stage_count("PROJECTION_BOX") <= stage_count("PROJECTION")}) {
        trace_exact = counted_predicate_main15(work, schedule_predicate)
            && trace_exact;
    }
    gate.trace = trace_grammar15(step, options, work) && trace_exact;
    gate.finite = finite;
    gate.nonnegative_multipliers = true;
    for (long double value : step.pi) {
        gate.nonnegative_multipliers = counted_predicate_main15(
            work, value >= 0.0L) && gate.nonnegative_multipliers;
    }
    const bool density_maximum = counted_predicate_main15(work,
        step.metrics.maximum_positive_density_strain
            <= options.profile.gates.density_maximum);
    const bool density_rms = counted_predicate_main15(work,
        step.metrics.rms_positive_density_strain
            <= options.profile.gates.density_rms);
    gate.density = density_maximum && density_rms;
    const bool kkt_rms = counted_predicate_main15(work,
        step.metrics.kkt_rms_m <= options.profile.gates.kkt_rms_m);
    const bool kkt_maximum = counted_predicate_main15(work,
        step.metrics.kkt_maximum_m <= options.profile.gates.kkt_maximum_m);
    gate.kkt = kkt_rms && kkt_maximum;
    gate.complementarity = counted_predicate_main15(work,
        step.metrics.complementarity
            <= options.profile.gates.complementarity);
    gate.multiplier_fixed_point = counted_predicate_main15(work,
        step.metrics.multiplier_fixed_point
            <= options.profile.gates.multiplier_fixed_point);
    std::vector<std::uint32_t> derived_active;
    for (std::size_t index = 0U; index < step.pi.size(); ++index) {
        if (counted_predicate_main15(work, step.pi[index] > 0.0L)
                && index < step.state.dynamic.size()) {
            derived_active.push_back(step.state.dynamic[index].id);
        }
    }
    bool active_exact = counted_predicate_main15(work,
        step.active_multiplier_ids.size() == derived_active.size());
    active_exact = counted_predicate_main15(work,
        step.metrics.active_multiplier_count == derived_active.size())
        && active_exact;
    const std::size_t expected_dynamic = accepted.dynamic.size();
    for (std::size_t index = 1U; index < expected_dynamic; ++index) {
        const bool either_absent = index >= step.active_multiplier_ids.size();
        active_exact = counted_predicate_main15(work,
            either_absent || step.active_multiplier_ids[index - 1U]
                < step.active_multiplier_ids[index]) && active_exact;
    }
    for (std::size_t index = 0U; index < expected_dynamic; ++index) {
        const bool derived_present = index < derived_active.size();
        const bool published_present = index < step.active_multiplier_ids.size();
        const bool equal = derived_present == published_present
            && (!derived_present || step.active_multiplier_ids[index]
                == derived_active[index]);
        active_exact = counted_predicate_main15(work, equal) && active_exact;
    }
    gate.active_signature = active_exact;
    bool contained = options.boundary_mode
        == BoundaryMode15::UnboundedManufactured;
    if (options.boundary_mode == BoundaryMode15::AnalyticBox) {
        contained = true;
        const long double lower = options.profile.spacing * 0.5L;
        const Vec3l upper{options.profile.basin.x - lower,
            options.profile.basin.y - lower,
            options.profile.basin.z - lower};
        for (const Dynamic15& item : step.state.dynamic) {
            for (bool inside : std::array<bool, 6>{item.position.x >= lower,
                     item.position.y >= lower, item.position.z >= lower,
                     item.position.x <= upper.x, item.position.y <= upper.y,
                     item.position.z <= upper.z}) {
                contained = counted_predicate_main15(work, inside) && contained;
            }
        }
    }
    gate.containment = contained;
    gate.capacity = counted_predicate_main15(work,
            step.state.dynamic.size()
                <= options.profile.maximum_dynamic_samples);
    gate.capacity = counted_predicate_main15(work,
            graph.maximum_degree <= options.profile.maximum_neighbors)
        && gate.capacity;
    gate.capacity = counted_predicate_main15(work, !graph.capacity_overflow)
        && gate.capacity;
    gate.capacity = counted_predicate_main15(work, !graph.invalid)
        && gate.capacity;
    gate.capacity = counted_predicate_main15(work, step.apparatus_valid)
        && gate.capacity;
    bool identity = counted_predicate_main15(work,
        step.state.dynamic.size() == accepted.dynamic.size());
    identity = counted_predicate_main15(work,
        step.state.ghosts.size() == accepted.ghosts.size()) && identity;
    const std::vector<std::size_t> observed_dynamic_order = stable_order15(
        step.state);
    const std::vector<std::size_t> accepted_dynamic_order = stable_order15(
        accepted);
    const std::size_t dynamic_identity_count = std::min(
        observed_dynamic_order.size(), accepted_dynamic_order.size());
    for (std::size_t index = 0U; index < dynamic_identity_count; ++index) {
        const Dynamic15& observed = step.state.dynamic[
            observed_dynamic_order[index]];
        const Dynamic15& expected = accepted.dynamic[
            accepted_dynamic_order[index]];
        for (bool equal : std::array<bool, 4>{observed.id == expected.id,
                 observed.reference.x == observed.position.x,
                 observed.reference.y == observed.position.y,
                 observed.reference.z == observed.position.z}) {
            identity = counted_predicate_main15(work, equal) && identity;
        }
    }
    std::vector<std::size_t> observed_ghost_order(step.state.ghosts.size());
    std::vector<std::size_t> accepted_ghost_order(accepted.ghosts.size());
    std::iota(observed_ghost_order.begin(), observed_ghost_order.end(), 0U);
    std::iota(accepted_ghost_order.begin(), accepted_ghost_order.end(), 0U);
    const auto ghost_less = [](const Ghost15& lhs, const Ghost15& rhs) {
        return lhs.id < rhs.id;
    };
    std::sort(observed_ghost_order.begin(), observed_ghost_order.end(),
        [&](std::size_t lhs, std::size_t rhs) {
            return ghost_less(step.state.ghosts[lhs], step.state.ghosts[rhs]);
        });
    std::sort(accepted_ghost_order.begin(), accepted_ghost_order.end(),
        [&](std::size_t lhs, std::size_t rhs) {
            return ghost_less(accepted.ghosts[lhs], accepted.ghosts[rhs]);
        });
    const std::size_t ghost_identity_count = std::min(
        observed_ghost_order.size(), accepted_ghost_order.size());
    for (std::size_t index = 0U; index < ghost_identity_count; ++index) {
        const Ghost15& observed = step.state.ghosts[
            observed_ghost_order[index]];
        const Ghost15& expected = accepted.ghosts[
            accepted_ghost_order[index]];
        for (bool equal : std::array<bool, 4>{observed.id == expected.id,
                 observed.position.x == expected.position.x,
                 observed.position.y == expected.position.y,
                 observed.position.z == expected.position.z}) {
            identity = counted_predicate_main15(work, equal) && identity;
        }
    }
    std::vector<Contact15> identity_contact = step.contact;
    std::sort(identity_contact.begin(), identity_contact.end(),
        [](const Contact15& lhs, const Contact15& rhs) {
            return lhs.id < rhs.id;
        });
    for (std::size_t index = 0U; index < identity_contact.size(); ++index) {
        const bool id_exact = index < observed_dynamic_order.size()
            && identity_contact[index].id == step.state.dynamic[
                observed_dynamic_order[index]].id;
        const bool mask_valid = (identity_contact[index].mask & 0xc0U) == 0U;
        identity = counted_predicate_main15(work, id_exact) && identity;
        identity = counted_predicate_main15(work, mask_valid) && identity;
    }
    gate.identity = identity;
    gate.density_oracle = density_cross.pass;
    gate.manufactured_active_empty = counted_predicate_main15(work,
        options.boundary_mode != BoundaryMode15::UnboundedManufactured
            || step.active_multiplier_ids.empty());
    gate.accepted = true;
    for (bool predicate : std::array<bool, 14>{gate.finite,
             gate.nonnegative_multipliers, gate.density, gate.kkt,
             gate.complementarity, gate.multiplier_fixed_point,
             gate.active_signature, gate.containment, gate.capacity,
             gate.trace, gate.identity, gate.density_oracle,
             gate.manufactured_active_empty,
             step.accepted}) {
        gate.accepted = counted_predicate_main15(work, predicate)
            && gate.accepted;
    }
    return gate;
}

std::string step_root15(const Step15& step, const StepGate15& gate,
    const GraphObservables15& graph, const DensityCross15& density_cross,
    std::string_view expected_work_root, std::string_view actual_work_root) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.step.v1");
    for (std::string_view root : std::array<std::string_view, 7>{
             step.input_root, step.previous_state_root, step.state_root,
             step.pressure_root, step.contact_root, expected_work_root,
             actual_work_root}) {
        put_string(bytes, root);
    }
    put_string(bytes, step.outcome);
    append_bool(bytes, step.apparatus_valid);
    append_bool(bytes, step.work_ceiling);
    append_bool(bytes, step.accepted);
    for (bool predicate : std::array<bool, 14>{gate.finite,
             gate.nonnegative_multipliers, gate.density, gate.kkt,
             gate.complementarity, gate.multiplier_fixed_point,
             gate.active_signature, gate.containment, gate.capacity,
             gate.trace, gate.identity, gate.density_oracle,
             gate.manufactured_active_empty, gate.accepted}) {
        append_bool(bytes, predicate);
    }
    put_f64(bytes, density_cross.relative_l2);
    append_bool(bytes, density_cross.constraint_signature_exact);
    for (long double value : std::array<long double, 12>{
             step.metrics.objective,
             step.metrics.nonpressure_energy, step.metrics.inertia_energy,
             step.metrics.viscosity_energy, step.metrics.surface_energy,
             step.metrics.maximum_positive_density_strain,
             step.metrics.rms_positive_density_strain,
             step.metrics.kkt_rms_m, step.metrics.kkt_maximum_m,
             step.metrics.complementarity,
             step.metrics.multiplier_fixed_point,
             static_cast<long double>(step.metrics.active_multiplier_count)}) {
        put_f64(bytes, value);
    }
    put_u32(bytes, step.trace.outer_updates);
    put_u64(bytes, step.trace.accepted_inner_iterations);
    put_u64(bytes, step.trace.backtracks.size());
    for (std::uint32_t value : step.trace.backtracks) put_u32(bytes, value);
    put_u64(bytes, step.trace.outer_inner_offsets.size());
    for (std::uint64_t value : step.trace.outer_inner_offsets) {
        put_u64(bytes, value);
    }
    put_u64(bytes, step.trace.accepted_objectives.size());
    for (long double value : step.trace.accepted_objectives) {
        put_f64(bytes, value);
    }
    put_u64(bytes, step.trace.evaluation_positions.size());
    for (const std::vector<Vec3l>& positions :
         step.trace.evaluation_positions) {
        append_id_vec_array15(bytes, step.state, positions);
    }
    put_u64(bytes, step.trace.evaluation_multipliers.size());
    for (const std::vector<long double>& multipliers :
         step.trace.evaluation_multipliers) {
        append_id_scalar_array15(bytes, step.state, multipliers);
    }
    put_u64(bytes, step.trace.evaluation_modes.size());
    for (std::uint8_t value : step.trace.evaluation_modes) {
        put_u8(bytes, value);
    }
    put_u64(bytes, step.trace.evaluation_gradient.size());
    for (std::uint8_t value : step.trace.evaluation_gradient) {
        put_u8(bytes, value);
    }
    put_u64(bytes, step.trace.reached_stages.size());
    for (const std::string& value : step.trace.reached_stages) {
        put_string(bytes, value);
    }
    put_u64(bytes, step.active_multiplier_ids.size());
    for (std::uint32_t id : step.active_multiplier_ids) put_u32(bytes, id);
    const std::vector<std::size_t> state_order = stable_order15(step.state);
    const std::size_t density_records = std::min(
        state_order.size(), step.rho.size());
    put_u64(bytes, density_records);
    for (std::size_t ordered = 0U; ordered < density_records; ++ordered) {
        const std::size_t index = state_order[ordered];
        put_u32(bytes, step.state.dynamic[index].id);
        put_f64(bytes, step.rho[index]);
    }
    put_u64(bytes, graph.density.size());
    for (const DensityGraphRecord15& record : graph.density) {
        put_u32(bytes, record.owner);
        put_u32(bytes, record.neighbor);
        append_bool(bytes, record.ghost);
    }
    put_u64(bytes, graph.surface.size());
    for (const PairRecord15& record : graph.surface) {
        put_u32(bytes, record.first);
        put_u32(bytes, record.second);
    }
    put_u64(bytes, graph.viscosity.size());
    for (const PairRecord15& record : graph.viscosity) {
        put_u32(bytes, record.first);
        put_u32(bytes, record.second);
    }
    put_u32(bytes, graph.maximum_degree);
    append_bool(bytes, graph.capacity_overflow);
    append_bool(bytes, graph.invalid);
    append_id_vec_array15(bytes, step.state, step.kkt_gradient);
    append_id_vec_array15(bytes, step.state, step.kkt_direction);
    append_id_vec_array15(bytes, step.state, step.dynamic_pressure_force);
    append_id_vec_array15(bytes, step.state, step.ghost_pressure_force);
    append_id_vec_array15(bytes, step.state, step.viscosity_force);
    append_id_vec_array15(bytes, step.state, step.surface_force);
    return root15(std::move(bytes));
}

std::string nonfinite_step_root15(const Step15& step,
    std::string_view failure_field, std::uint64_t failure_index,
    std::string_view failure_class,
    std::string_view expected_work_root, std::string_view actual_work_root) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.step.v1");
    for (std::string_view root : std::array<std::string_view, 7>{
             step.input_root, step.previous_state_root, std::string_view{},
             std::string_view{}, std::string_view{}, expected_work_root,
             actual_work_root}) {
        put_string(bytes, root);
    }
    put_string(bytes, step.outcome);
    append_bool(bytes, step.apparatus_valid);
    append_bool(bytes, step.work_ceiling);
    append_bool(bytes, step.accepted);
    put_string(bytes, failure_field);
    put_u64(bytes, failure_index);
    put_string(bytes, failure_class);
    return root15(std::move(bytes));
}

struct Comparison15 {
    long double position_rms = 0.0L;
    long double position_maximum = 0.0L;
    long double velocity_rms = 0.0L;
    long double velocity_maximum = 0.0L;
    long double density_relative_l2 = 0.0L;
    bool active_ids_exact = false;
    bool contact_masks_exact = false;
    bool category_exact = false;
    bool trace_exact = false;
    bool pass = false;
};

long double pairwise_scalar_sum15(std::vector<long double> values);

std::vector<std::pair<std::uint32_t, std::size_t>> id_index15(
    const State15& state) {
    std::vector<std::pair<std::uint32_t, std::size_t>> result;
    result.reserve(state.dynamic.size());
    for (std::size_t index = 0U; index < state.dynamic.size(); ++index) {
        result.emplace_back(state.dynamic[index].id, index);
    }
    std::sort(result.begin(), result.end());
    return result;
}

std::vector<std::uint32_t> active_ids15(const Step15& step) {
    std::vector<std::uint32_t> result;
    if (step.pi.size() != step.state.dynamic.size()) return result;
    for (std::size_t index = 0U; index < step.pi.size(); ++index) {
        if (step.pi[index] > 0.0L) result.push_back(step.state.dynamic[index].id);
    }
    std::sort(result.begin(), result.end());
    return result;
}

std::vector<std::pair<std::uint32_t, std::uint8_t>> contact_signature15(
    const Step15& step) {
    std::vector<std::pair<std::uint32_t, std::uint8_t>> result;
    result.reserve(step.contact.size());
    for (const Contact15& item : step.contact) {
        result.emplace_back(item.id, item.mask);
    }
    std::sort(result.begin(), result.end());
    return result;
}

Comparison15 compare_steps15(const Step15& candidate, const Step15& oracle,
    const Gates15& gates, Work15* work = nullptr) {
    Comparison15 result;
    const auto predicate = [&](bool value) {
        return work == nullptr ? value
            : counted_predicate_main15(*work, value);
    };
    const auto candidate_index = id_index15(candidate.state);
    const auto oracle_index = id_index15(oracle.state);
    const bool size_exact = predicate(
        candidate_index.size() == oracle_index.size());
    const bool nonempty = predicate(!candidate_index.empty());
    if (!size_exact || !nonempty) {
        return result;
    }
    std::vector<long double> position_squares;
    std::vector<long double> velocity_squares;
    std::vector<long double> density_error_squares;
    std::vector<long double> density_oracle_squares;
    position_squares.reserve(candidate_index.size());
    velocity_squares.reserve(candidate_index.size());
    density_error_squares.reserve(candidate_index.size());
    density_oracle_squares.reserve(candidate_index.size());
    for (std::size_t ordered = 0U; ordered < candidate_index.size(); ++ordered) {
        if (!predicate(candidate_index[ordered].first
                == oracle_index[ordered].first)) {
            return result;
        }
        const std::size_t ci = candidate_index[ordered].second;
        const std::size_t oi = oracle_index[ordered].second;
        const long double position_error = norm15(
            candidate.state.dynamic[ci].position
            - oracle.state.dynamic[oi].position);
        const long double velocity_error = norm15(
            candidate.state.dynamic[ci].velocity
            - oracle.state.dynamic[oi].velocity);
        position_squares.push_back(position_error * position_error);
        velocity_squares.push_back(velocity_error * velocity_error);
        result.position_maximum =
            std::max(result.position_maximum, position_error);
        result.velocity_maximum =
            std::max(result.velocity_maximum, velocity_error);
        const bool candidate_density_present = predicate(
            ci < candidate.rho.size());
        const bool oracle_density_present = predicate(oi < oracle.rho.size());
        if (candidate_density_present && oracle_density_present) {
            const long double difference = candidate.rho[ci] - oracle.rho[oi];
            density_error_squares.push_back(difference * difference);
            density_oracle_squares.push_back(oracle.rho[oi] * oracle.rho[oi]);
        }
    }
    const long double count = static_cast<long double>(candidate_index.size());
    result.position_rms = std::sqrt(
        pairwise_scalar_sum15(std::move(position_squares)) / count);
    result.velocity_rms = std::sqrt(
        pairwise_scalar_sum15(std::move(velocity_squares)) / count);
    const long double density_error_square = pairwise_scalar_sum15(
        std::move(density_error_squares));
    const long double density_oracle_square = pairwise_scalar_sum15(
        std::move(density_oracle_squares));
    const bool density_denominator_positive = predicate(
        density_oracle_square > 0.0L);
    result.density_relative_l2 = density_denominator_positive
        ? std::sqrt(density_error_square / density_oracle_square)
        : std::numeric_limits<long double>::infinity();
    result.active_ids_exact = predicate(
        active_ids15(candidate) == active_ids15(oracle));
    result.contact_masks_exact = predicate(
        contact_signature15(candidate) == contact_signature15(oracle));
    const bool outcome_exact = predicate(candidate.outcome == oracle.outcome);
    const bool accepted_exact = predicate(candidate.accepted == oracle.accepted);
    const bool ceiling_exact = predicate(
        candidate.work_ceiling == oracle.work_ceiling);
    result.category_exact = outcome_exact && accepted_exact && ceiling_exact;
    const bool outer_exact = predicate(candidate.trace.outer_updates
        == oracle.trace.outer_updates);
    const bool inner_exact = predicate(
        candidate.trace.accepted_inner_iterations
        == oracle.trace.accepted_inner_iterations);
    const bool backtracks_exact = predicate(
        candidate.trace.backtracks == oracle.trace.backtracks);
    result.trace_exact = outer_exact && inner_exact && backtracks_exact;
    const bool position_rms_ok = predicate(
        result.position_rms <= gates.one_step_position_rms_m);
    const bool position_maximum_ok = predicate(
        result.position_maximum <= gates.one_step_position_maximum_m);
    const bool density_ok = predicate(
        result.density_relative_l2 <= gates.density_relative_l2);
    const bool active_ok = predicate(result.active_ids_exact);
    const bool contact_ok = predicate(result.contact_masks_exact);
    const bool category_ok = predicate(result.category_exact);
    const bool trace_ok = predicate(result.trace_exact);
    result.pass = position_rms_ok && position_maximum_ok && density_ok
        && active_ok && contact_ok && category_ok && trace_ok;
    return result;
}

Work15 expected_comparison_work15(std::size_t expected_records) {
    Work15 expected;
    expected.scalar_predicates = 18U + 3U * expected_records;
    return expected;
}

State15 tight_fixture15(const Profile15& profile) {
    State15 state;
    state.dynamic.reserve(128U);
    for (std::size_t iz = 0U; iz < 8U; ++iz) {
        for (std::size_t iy = 0U; iy < 4U; ++iy) {
            for (std::size_t ix = 0U; ix < 4U; ++ix) {
                const std::size_t logical = ix + 4U * (iy + 4U * iz);
                const Vec3l position = canonical_vec({
                    0.025L + 0.05L * static_cast<long double>(ix),
                    0.025L + 0.05L * static_cast<long double>(iy),
                    0.025L + 0.05L * static_cast<long double>(iz)});
                state.dynamic.push_back({static_cast<std::uint32_t>(
                        1000U + 17U * logical), position, position, {}});
            }
        }
    }
    gpu_full_step::NonlocalGpuProfile source =
        gpu_full_step::nonlocal_water_corrected_profile();
    source.lambda = 0.0;
    source.mu = 0.0;
    source.gamma = 0.0;
    source.basin_extent = {static_cast<double>(profile.basin.x),
        static_cast<double>(profile.basin.y),
        static_cast<double>(profile.basin.z)};
    const std::vector<gpu_full_step::NonlocalGpuGhost> ghosts =
        gpu_full_step::make_basin_ghosts(source);
    state.ghosts.reserve(ghosts.size());
    for (const gpu_full_step::NonlocalGpuGhost& ghost : ghosts) {
        state.ghosts.push_back({ghost.sample_id,
            {static_cast<long double>(static_cast<float>(ghost.position.x)),
                static_cast<long double>(static_cast<float>(ghost.position.y)),
                static_cast<long double>(static_cast<float>(ghost.position.z))}});
    }
    return state;
}

State15 pair_fixture15(Vec3l lhs_position, Vec3l lhs_velocity,
    Vec3l rhs_position, Vec3l rhs_velocity) {
    State15 state;
    state.dynamic.push_back(
        {101U, canonical_vec(lhs_position), canonical_vec(lhs_position),
            canonical_vec(lhs_velocity)});
    state.dynamic.push_back(
        {211U, canonical_vec(rhs_position), canonical_vec(rhs_position),
            canonical_vec(rhs_velocity)});
    return state;
}

State15 tetra_fixture15() {
    State15 state;
    const std::array<Vec3l, 4> positions{{{0.10L, 0.10L, 0.10L},
        {0.15L, 0.10L, 0.10L}, {0.10L, 0.15L, 0.10L},
        {0.10L, 0.10L, 0.15L}}};
    const std::array<Vec3l, 4> velocities{{{-0.15L, -0.15L, -0.15L},
        {0.15L, 0.0L, 0.0L}, {0.0L, 0.15L, 0.0L},
        {0.0L, 0.0L, 0.15L}}};
    for (std::size_t index = 0U; index < positions.size(); ++index) {
        state.dynamic.push_back({static_cast<std::uint32_t>(301U + 101U * index),
            canonical_vec(positions[index]), canonical_vec(positions[index]),
            canonical_vec(velocities[index])});
    }
    return state;
}

State15 permuted_state15(const State15& input) {
    State15 result = input;
    if (result.dynamic.empty()) return result;
    std::vector<Dynamic15> dynamic;
    dynamic.reserve(result.dynamic.size());
    std::vector<bool> seen(result.dynamic.size(), false);
    for (std::size_t input_index = 0U; input_index < result.dynamic.size();
         ++input_index) {
        const std::size_t logical =
            (23449ULL * input_index + 7919ULL) % result.dynamic.size();
        if (seen[logical]) {
            throw std::runtime_error("NCGP15 physical permutation not bijective");
        }
        seen[logical] = true;
        dynamic.push_back(result.dynamic[logical]);
    }
    result.dynamic = std::move(dynamic);
    return result;
}

SolverOptions15 manufactured_options15(const Profile15& profile,
    TermMask15 terms, Mutation15 mutation = Mutation15::None) {
    SolverOptions15 options;
    options.profile = profile;
    options.terms = terms;
    options.gravity_mode = GravityMode15::Zero;
    options.boundary_mode = BoundaryMode15::UnboundedManufactured;
    options.mutation = mutation;
    return options;
}

SolverOptions15 confined_options15(const Profile15& profile,
    TermMask15 terms, Mutation15 mutation = Mutation15::None) {
    SolverOptions15 options;
    options.profile = profile;
    options.terms = terms;
    options.gravity_mode = GravityMode15::Profile;
    options.boundary_mode = BoundaryMode15::AnalyticBox;
    options.mutation = mutation;
    return options;
}

bool zero_work15(const Work15& work) {
    const auto values = work_values15(work);
    return std::all_of(values.begin(), values.end(),
        [](std::uint64_t value) { return value == 0U; });
}

struct WorkSeal15 {
    Work15 expected;
    Work15 actual;
    std::string expected_root;
    std::string actual_root;
    bool exact = false;
};

WorkSeal15 seal_work15(const Work15& actual, const Work15& expected) {
    WorkSeal15 seal;
    seal.expected = expected;
    seal.actual = actual;
    seal.expected_root = work_root15(expected);
    seal.actual_root = work_root15(actual);
    seal.exact = equal_work15(actual, expected);
    return seal;
}

std::uint64_t step_portable_fields15(const Step15& step,
    const GraphObservables15& graph) {
    const std::uint64_t force_records = step.dynamic_pressure_force.size()
        + step.ghost_pressure_force.size() + step.viscosity_force.size()
        + step.surface_force.size();
    const std::uint64_t kkt_records = step.kkt_gradient.size()
        + step.kkt_direction.size();
    std::uint64_t evaluation_fields = step.trace.evaluation_modes.size()
        + step.trace.evaluation_gradient.size()
        + step.trace.reached_stages.size();
    for (const std::vector<Vec3l>& positions :
         step.trace.evaluation_positions) {
        evaluation_fields += 1U + 4U * positions.size();
    }
    for (const std::vector<long double>& multipliers :
         step.trace.evaluation_multipliers) {
        evaluation_fields += 1U + 2U * multipliers.size();
    }
    const std::uint64_t density_records = std::min(
        step.state.dynamic.size(), step.rho.size());
    return 64U + 2U * density_records + step.trace.backtracks.size()
        + step.trace.outer_inner_offsets.size()
        + step.trace.accepted_objectives.size()
        + evaluation_fields
        + step.active_multiplier_ids.size()
        + 3U * graph.density.size() + 2U * graph.surface.size()
        + 2U * graph.viscosity.size() + 4U * force_records
        + 4U * kkt_records;
}

std::uint64_t expected_step_portable_fields15(const Step15& step,
    const GraphObservables15& expected_graph, std::size_t accepted_count) {
    const bool final_payload = step.accepted
        || (!step.trace.reached_stages.empty()
            && step.trace.reached_stages.back() == "FINAL_GATE");
    const std::uint64_t dynamic = accepted_count;
    const std::uint64_t force_records = 4U * dynamic;
    const std::uint64_t kkt_records = final_payload ? 2U * dynamic : 0U;
    std::uint64_t evaluation_fields = step.trace.evaluation_modes.size()
        + step.trace.evaluation_gradient.size()
        + step.trace.reached_stages.size()
        + step.trace.evaluation_positions.size() * (1U + 4U * dynamic)
        + step.trace.evaluation_multipliers.size() * (1U + 2U * dynamic);
    const std::uint64_t active_records = final_payload
        ? std::count_if(step.pi.begin(), step.pi.end(),
            [](long double value) { return value > 0.0L; })
        : 0U;
    return 64U + 2U * dynamic + step.trace.backtracks.size()
        + step.trace.outer_inner_offsets.size()
        + step.trace.accepted_objectives.size() + evaluation_fields
        + active_records + 3U * expected_graph.density.size()
        + 2U * expected_graph.surface.size()
        + 2U * expected_graph.viscosity.size() + 4U * force_records
        + 4U * kkt_records;
}

Work15 expected_root_work15(const Step15& step,
    const GraphObservables15& graph, std::size_t accepted_count,
    bool pressure_sealed, const SolverOptions15& options) {
    Work15 result = step.expected_work;
    const std::uint64_t state_count = accepted_count;
    const bool final_payload = step.accepted
        || (!step.trace.reached_stages.empty()
            && step.trace.reached_stages.back() == "FINAL_GATE");
    const std::uint64_t contact_count = final_payload
            && options.boundary_mode == BoundaryMode15::AnalyticBox
        ? accepted_count : 0U;
    result.portable_fields += 4U + 10U * accepted_count;
    result.portable_fields += 4U + 10U * state_count;
    if (pressure_sealed) {
        result.portable_fields += 3U + 4U * state_count;
    }
    result.portable_fields += 3U + 5U * contact_count;
    result.portable_fields += expected_step_portable_fields15(step, graph,
        accepted_count);
    result.hash_derivations += pressure_sealed ? 5U : 4U;
    return result;
}

struct FinalStep15 {
    std::string implementation;
    std::uint32_t one_based_step = 0U;
    SolverOptions15 options;
    State15 accepted_input;
    Step15 value;
    GraphObservables15 graph;
    DensityCross15 density_cross;
    StepGate15 gate;
    std::string failure_field;
    std::uint64_t failure_index = 0U;
    std::string failure_class;
    WorkSeal15 work;
    bool root_closed = false;
    bool private_commit_ready = false;
    bool transaction_committed = false;
};

bool main_step_apparatus15(const FinalStep15& step);

bool finite_step_payload15(const Step15& step) {
    bool finite = finite_state15(step.state);
    for (long double value : step.pi) finite = std::isfinite(value) && finite;
    for (long double value : step.rho) finite = std::isfinite(value) && finite;
    for (const Contact15& item : step.contact) {
        finite = finite15(item.impulse) && finite;
    }
    for (const std::vector<Vec3l>* values :
         std::array<const std::vector<Vec3l>*, 6>{&step.kkt_gradient,
             &step.kkt_direction, &step.dynamic_pressure_force,
             &step.ghost_pressure_force, &step.viscosity_force,
             &step.surface_force}) {
        for (Vec3l value : *values) finite = finite15(value) && finite;
    }
    for (long double value : std::array<long double, 11>{
             step.metrics.objective, step.metrics.nonpressure_energy,
             step.metrics.inertia_energy, step.metrics.viscosity_energy,
             step.metrics.surface_energy,
             step.metrics.maximum_positive_density_strain,
             step.metrics.rms_positive_density_strain,
             step.metrics.kkt_rms_m, step.metrics.kkt_maximum_m,
             step.metrics.complementarity,
             step.metrics.multiplier_fixed_point}) {
        finite = std::isfinite(value) && finite;
    }
    for (long double value : step.trace.accepted_objectives) {
        finite = std::isfinite(value) && finite;
    }
    for (const std::vector<Vec3l>& positions :
         step.trace.evaluation_positions) {
        for (Vec3l value : positions) finite = finite15(value) && finite;
    }
    for (const std::vector<long double>& multipliers :
         step.trace.evaluation_multipliers) {
        for (long double value : multipliers) {
            finite = std::isfinite(value) && finite;
        }
    }
    return finite;
}

std::pair<std::string, std::uint64_t> first_nonfinite_step15(
    const Step15& step) {
    const auto vector_field = [](const std::vector<Vec3l>& values,
                                  std::string_view name)
        -> std::optional<std::pair<std::string, std::uint64_t>> {
        for (std::size_t index = 0U; index < values.size(); ++index) {
            if (!finite15(values[index])) {
                return std::pair<std::string, std::uint64_t>{
                    std::string(name), index};
            }
        }
        return std::nullopt;
    };
    for (std::size_t index = 0U; index < step.state.dynamic.size(); ++index) {
        const Dynamic15& item = step.state.dynamic[index];
        if (!finite15(item.reference)) return {"state.reference", index};
        if (!finite15(item.position)) return {"state.position", index};
        if (!finite15(item.velocity)) return {"state.velocity", index};
    }
    for (std::size_t index = 0U; index < step.state.ghosts.size(); ++index) {
        if (!finite15(step.state.ghosts[index].position)) {
            return {"state.ghost_position", index};
        }
    }
    const auto scalar_field = [](const std::vector<long double>& values,
                                  std::string_view name)
        -> std::optional<std::pair<std::string, std::uint64_t>> {
        for (std::size_t index = 0U; index < values.size(); ++index) {
            if (!std::isfinite(values[index])) {
                return std::pair<std::string, std::uint64_t>{
                    std::string(name), index};
            }
        }
        return std::nullopt;
    };
    if (const auto failure = scalar_field(step.pi, "pressure.pi")) {
        return *failure;
    }
    if (const auto failure = scalar_field(step.rho, "density.rho")) {
        return *failure;
    }
    for (std::size_t index = 0U; index < step.contact.size(); ++index) {
        if (!finite15(step.contact[index].impulse)) {
            return {"contact.impulse", index};
        }
    }
    for (const auto& field : std::array<
             std::pair<const std::vector<Vec3l>*, std::string_view>, 6>{
             std::pair{&step.kkt_gradient,
                 std::string_view{"kkt.gradient"}},
             std::pair{&step.kkt_direction,
                 std::string_view{"kkt.direction"}},
             std::pair{&step.dynamic_pressure_force,
                 std::string_view{"force.dynamic_pressure"}},
             std::pair{&step.ghost_pressure_force,
                 std::string_view{"force.ghost_pressure"}},
             std::pair{&step.viscosity_force,
                 std::string_view{"force.viscosity"}},
             std::pair{&step.surface_force,
                 std::string_view{"force.surface"}}}) {
        if (const auto failure = vector_field(*field.first, field.second)) {
            return *failure;
        }
    }
    const std::array<long double, 11> metrics{
        step.metrics.objective, step.metrics.nonpressure_energy,
        step.metrics.inertia_energy, step.metrics.viscosity_energy,
        step.metrics.surface_energy,
        step.metrics.maximum_positive_density_strain,
        step.metrics.rms_positive_density_strain,
        step.metrics.kkt_rms_m, step.metrics.kkt_maximum_m,
        step.metrics.complementarity,
        step.metrics.multiplier_fixed_point};
    for (std::size_t index = 0U; index < metrics.size(); ++index) {
        if (!std::isfinite(metrics[index])) return {"metrics", index};
    }
    if (const auto failure = scalar_field(step.trace.accepted_objectives,
            "trace.accepted_objective")) {
        return *failure;
    }
    for (std::size_t evaluation = 0U;
         evaluation < step.trace.evaluation_positions.size(); ++evaluation) {
        if (const auto failure = vector_field(
                step.trace.evaluation_positions[evaluation],
                "trace.evaluation_position")) {
            return {failure->first, evaluation};
        }
    }
    for (std::size_t evaluation = 0U;
         evaluation < step.trace.evaluation_multipliers.size(); ++evaluation) {
        if (const auto failure = scalar_field(
                step.trace.evaluation_multipliers[evaluation],
                "trace.evaluation_multiplier")) {
            return {failure->first, evaluation};
        }
    }
    return {"unknown", 0U};
}

std::string nonfinite_classification15(long double value) {
    if (std::isnan(value)) return std::signbit(value) ? "NEGATIVE_NAN"
                                                      : "POSITIVE_NAN";
    if (std::isinf(value)) return std::signbit(value) ? "NEGATIVE_INFINITY"
                                                      : "POSITIVE_INFINITY";
    return "FINITE";
}

std::string first_nonfinite_classification15(const Step15& step) {
    const auto scalar = [](long double value) -> std::optional<std::string> {
        if (!std::isfinite(value)) return nonfinite_classification15(value);
        return std::nullopt;
    };
    const auto vector = [&](Vec3l value) -> std::optional<std::string> {
        if (const auto found = scalar(value.x)) return found;
        if (const auto found = scalar(value.y)) return found;
        return scalar(value.z);
    };
    for (const Dynamic15& item : step.state.dynamic) {
        if (const auto found = vector(item.reference)) return *found;
        if (const auto found = vector(item.position)) return *found;
        if (const auto found = vector(item.velocity)) return *found;
    }
    for (const Ghost15& item : step.state.ghosts) {
        if (const auto found = vector(item.position)) return *found;
    }
    for (long double value : step.pi) {
        if (const auto found = scalar(value)) return *found;
    }
    for (long double value : step.rho) {
        if (const auto found = scalar(value)) return *found;
    }
    for (const Contact15& item : step.contact) {
        if (const auto found = vector(item.impulse)) return *found;
    }
    for (const std::vector<Vec3l>* values :
         std::array<const std::vector<Vec3l>*, 6>{&step.kkt_gradient,
             &step.kkt_direction, &step.dynamic_pressure_force,
             &step.ghost_pressure_force, &step.viscosity_force,
             &step.surface_force}) {
        for (Vec3l value : *values) {
            if (const auto found = vector(value)) return *found;
        }
    }
    for (long double value : std::array<long double, 11>{
             step.metrics.objective, step.metrics.nonpressure_energy,
             step.metrics.inertia_energy, step.metrics.viscosity_energy,
             step.metrics.surface_energy,
             step.metrics.maximum_positive_density_strain,
             step.metrics.rms_positive_density_strain,
             step.metrics.kkt_rms_m, step.metrics.kkt_maximum_m,
             step.metrics.complementarity,
             step.metrics.multiplier_fixed_point}) {
        if (const auto found = scalar(value)) return *found;
    }
    for (long double value : step.trace.accepted_objectives) {
        if (const auto found = scalar(value)) return *found;
    }
    for (const std::vector<Vec3l>& values :
         step.trace.evaluation_positions) {
        for (Vec3l value : values) {
            if (const auto found = vector(value)) return *found;
        }
    }
    for (const std::vector<long double>& values :
         step.trace.evaluation_multipliers) {
        for (long double value : values) {
            if (const auto found = scalar(value)) return *found;
        }
    }
    return "UNCLASSIFIED_NONFINITE";
}

std::string semantic_step_signature15(const FinalStep15& closed,
    std::string_view canonical_input_root);

FinalStep15 close_step15(std::string implementation, const State15& accepted,
    std::string input_root, std::uint32_t one_based_step,
    const SolverOptions15& options, Step15 value) {
    FinalStep15 result;
    result.implementation = std::move(implementation);
    result.one_based_step = one_based_step;
    result.options = options;
    result.accepted_input = accepted;
    result.value = std::move(value);
    result.value.input_root = std::move(input_root);
    Work15 semantic_root_work;
    result.value.previous_state_root = state_root15(result.value.input_root,
        one_based_step - 1U, accepted, &semantic_root_work);
    if (!finite_step_payload15(result.value)) {
        std::tie(result.failure_field, result.failure_index) =
            first_nonfinite_step15(result.value);
        result.failure_class = first_nonfinite_classification15(result.value);
        result.value.outcome = "NONFINITE";
        result.value.apparatus_valid = false;
        result.value.accepted = false;
        result.value.work_ceiling = false;
        add_work15(result.value.work, semantic_root_work);
        result.value.expected_work.portable_fields += 4U
            + 10U * accepted.dynamic.size();
        ++result.value.expected_work.hash_derivations;
        result.value.work.portable_fields += 15U;
        ++result.value.work.hash_derivations;
        result.value.expected_work.portable_fields += 15U;
        ++result.value.expected_work.hash_derivations;
        result.work = seal_work15(result.value.work,
            result.value.expected_work);
        result.value.work_root = result.work.actual_root;
        result.value.result_root = nonfinite_step_root15(result.value,
            result.failure_field, result.failure_index, result.failure_class,
            result.work.expected_root, result.work.actual_root);
        result.root_closed = root_shape15(result.value.input_root)
            && root_shape15(result.value.previous_state_root)
            && root_shape15(result.value.work_root)
            && root_shape15(result.value.result_root);
        result.private_commit_ready = false;
        result.transaction_committed = false;
        return result;
    }
    result.value.state_root = state_root15(result.value.input_root,
        one_based_step, result.value.state, &semantic_root_work);
    const bool pressure_shape = result.value.pi.size()
        == result.value.state.dynamic.size();
    if (pressure_shape) {
        result.value.pressure_root = pressure_root15(result.value.state_root,
            result.value.state, result.value.pi, options.profile,
            &semantic_root_work);
    } else {
        result.value.pressure_root.clear();
    }
    result.value.contact_root = contact_root15(result.value.state_root,
        result.value.contact, &semantic_root_work);
    Work15 graph_work;
    result.graph = graph_observables15(accepted, result.value, options,
        graph_work);
    add_work15(result.value.work, graph_work);
    const ExpectedGraphObservables15 expected_graph =
        expected_graph_observables15(accepted, result.value.state, options);
    add_work15(result.value.expected_work, expected_graph.work);
    result.density_cross = density_cross15(accepted, result.value, options);
    add_work15(result.value.work, result.density_cross.actual_work);
    add_work15(result.value.expected_work,
        result.density_cross.expected_work);
    Work15 gate_work;
    result.gate = gate_step15(accepted, result.value, result.graph, options,
        result.density_cross, gate_work);
    add_work15(result.value.work, gate_work);
    add_work15(result.value.expected_work, expected_gate_work15(
        accepted, result.value, result.graph, options));
    result.value.expected_work = expected_root_work15(result.value,
        expected_graph.value, accepted.dynamic.size(), pressure_shape,
        options);
    semantic_root_work.portable_fields += step_portable_fields15(
        result.value, result.graph);
    ++semantic_root_work.hash_derivations;
    add_work15(result.value.work, semantic_root_work);
    result.work = seal_work15(result.value.work, result.value.expected_work);
    result.value.work_root = result.work.actual_root;
    result.value.result_root = step_root15(result.value, result.gate,
        result.graph, result.density_cross,
        result.work.expected_root, result.work.actual_root);
    result.root_closed = root_shape15(result.value.input_root)
        && root_shape15(result.value.previous_state_root)
        && root_shape15(result.value.state_root)
        && root_shape15(result.value.pressure_root)
        && root_shape15(result.value.contact_root)
        && root_shape15(result.value.result_root);
    result.private_commit_ready = result.root_closed && result.work.exact
        && result.value.apparatus_valid && !result.value.work_ceiling
        && result.gate.accepted;
    result.transaction_committed = false;
    return result;
}

using StepFunction15 = Step15 (*)(const State15&, const SolverOptions15&);

FinalStep15 run_closed_step15(std::string implementation,
    StepFunction15 function, const State15& accepted,
    const Input15& input, std::uint32_t one_based_step,
    const SolverOptions15& options) {
    FinalStep15 result = close_step15(std::move(implementation), accepted,
        input.root, one_based_step, options, function(accepted, options));
    result.transaction_committed = result.private_commit_ready;
    return result;
}

FinalStep15 run_private_step15(std::string implementation,
    StepFunction15 function, const State15& accepted,
    const Input15& input, std::uint32_t one_based_step,
    const SolverOptions15& options) {
    return close_step15(std::move(implementation), accepted, input.root,
        one_based_step, options, function(accepted, options));
}

struct RootedReceipt15 {
    std::string name;
    std::string variant;
    std::string typed_outcome = "NOT_RUN";
    std::string input_root;
    std::vector<std::string> child_roots;
    WorkSeal15 work;
    bool pass = false;
    bool apparatus_valid = false;
    std::string result_root;
};

std::string receipt_root15(const RootedReceipt15& receipt) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.term-oracle.v1");
    put_string(bytes, receipt.name);
    put_string(bytes, receipt.variant);
    put_string(bytes, receipt.input_root);
    put_string(bytes, receipt.typed_outcome);
    append_bool(bytes, receipt.pass);
    append_bool(bytes, receipt.apparatus_valid);
    put_u64(bytes, receipt.child_roots.size());
    for (const std::string& child : receipt.child_roots) put_string(bytes, child);
    put_string(bytes, receipt.work.expected_root);
    put_string(bytes, receipt.work.actual_root);
    return root15(std::move(bytes));
}

std::uint64_t semantic_signature_fields15(const FinalStep15& closed) {
    const std::uint64_t count = closed.value.state.dynamic.size();
    return 2U * (4U + 10U * count) + (3U + 4U * count)
        + (3U + 5U * closed.value.contact.size())
        + step_portable_fields15(closed.value, closed.graph);
}

std::uint64_t expected_semantic_signature_fields15(
    const FinalStep15& closed) {
    const std::uint64_t count = closed.accepted_input.dynamic.size();
    const bool final_payload = closed.value.accepted
        || (!closed.value.trace.reached_stages.empty()
            && closed.value.trace.reached_stages.back() == "FINAL_GATE");
    const std::uint64_t contact = final_payload
            && closed.options.boundary_mode == BoundaryMode15::AnalyticBox
        ? count : 0U;
    const ExpectedGraphObservables15 expected_graph =
        expected_graph_observables15(closed.accepted_input,
            closed.value.state, closed.options);
    return 2U * (4U + 10U * count) + (3U + 4U * count)
        + (3U + 5U * contact)
        + expected_step_portable_fields15(closed.value,
            expected_graph.value, count);
}

RootedReceipt15 skipped_receipt15(std::string name, std::string variant,
    std::string typed_outcome) {
    RootedReceipt15 result;
    result.name = std::move(name);
    result.variant = std::move(variant);
    result.typed_outcome = std::move(typed_outcome);
    result.work = seal_work15({}, {});
    if (!zero_work15(result.work.actual)
            || !zero_work15(result.work.expected)) {
        throw std::logic_error("NCGP15 typed skip has nonzero work");
    }
    result.result_root = receipt_root15(result);
    return result;
}

long double kernel_omega15(long double distance, const Profile15& profile,
    bool missing_chain = false) {
    if (!(distance >= 0.0L) || distance > profile.horizon) return 0.0L;
    const long double q = 2.0L * distance / profile.horizon;
    const long double alpha = profile.kernel_scale * 3.0L
        / (2.0L * kPi15 * profile.horizon * profile.horizon
            * profile.horizon);
    const long double chain = missing_chain ? 1.0L
        : 2.0L / profile.horizon;
    if (q < 1.0L) {
        return -alpha * (-2.0L * q + 1.5L * q * q) * chain;
    }
    const long double tail = 2.0L - q;
    return alpha * tail * tail * 0.5L * chain;
}

long double surface_C_main15(long double distance, long double r0) {
    const long double q = distance / r0;
    if (q <= 1.0L) {
        return r0 * (q * q * q / 3.0L - q - 2.0L / 3.0L);
    }
    if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        return r0 * (q - shifted * shifted * shifted / 3.0L
            - 8.0L / 3.0L);
    }
    return 0.0L;
}

long double pairwise_scalar_sum15(std::vector<long double> values) {
    if (values.empty()) return 0.0L;
    while (values.size() > 1U) {
        std::vector<long double> next;
        next.reserve((values.size() + 1U) / 2U);
        for (std::size_t index = 0U; index < values.size(); index += 2U) {
            next.push_back(index + 1U == values.size()
                ? values[index] : values[index] + values[index + 1U]);
        }
        values = std::move(next);
    }
    return values.front();
}

Vec3l pairwise_vec_sum15(std::vector<Vec3l> values) {
    if (values.empty()) return {};
    while (values.size() > 1U) {
        std::vector<Vec3l> next;
        next.reserve((values.size() + 1U) / 2U);
        for (std::size_t index = 0U; index < values.size(); index += 2U) {
            next.push_back(index + 1U == values.size()
                ? values[index] : values[index] + values[index + 1U]);
        }
        values = std::move(next);
    }
    return values.front();
}

long double surface_energy15(const State15& state,
    const SolverOptions15& options, Work15* work = nullptr) {
    if (!options.terms.surface) return 0.0L;
    const auto order = stable_order15(state);
    std::vector<long double> terms;
    for (std::size_t a = 0U; a < order.size(); ++a) {
        for (std::size_t b = a + 1U; b < order.size(); ++b) {
            if (work != nullptr) ++work->surface_pair_tests;
            const long double distance = norm15(
                state.dynamic[order[a]].position
                - state.dynamic[order[b]].position);
            const bool positive = work == nullptr ? distance > 0.0L
                : counted_predicate_main15(*work, distance > 0.0L);
            const bool within = work == nullptr
                ? distance < 3.0L * options.profile.surface_r0
                : counted_predicate_main15(*work,
                    distance < 3.0L * options.profile.surface_r0);
            if (!positive || !within) {
                continue;
            }
            if (work != nullptr) ++work->surface_active_pairs;
            terms.push_back(2.0L * options.profile.gamma
                * options.profile.mass * options.profile.mass
                * surface_C_main15(distance, options.profile.surface_r0));
        }
    }
    return pairwise_scalar_sum15(std::move(terms));
}

Vec3l momentum15(const State15& state, const Profile15& profile) {
    std::vector<Vec3l> terms;
    const auto order = stable_order15(state);
    terms.reserve(order.size());
    for (std::size_t index : order) {
        terms.push_back(profile.mass * state.dynamic[index].velocity);
    }
    return pairwise_vec_sum15(std::move(terms));
}

long double mechanical_energy15(const State15& state,
    const SolverOptions15& options, Work15* work = nullptr) {
    std::vector<long double> terms;
    terms.push_back(surface_energy15(state, options, work));
    const Vec3l gravity = options.gravity_mode == GravityMode15::Profile
        ? options.profile.gravity : Vec3l{};
    const auto order = stable_order15(state);
    terms.reserve(1U + 2U * order.size());
    for (std::size_t index : order) {
        const Dynamic15& item = state.dynamic[index];
        terms.push_back(0.5L * options.profile.mass
            * dot15(item.velocity, item.velocity));
        terms.push_back(options.profile.mass * -dot15(gravity, item.position));
    }
    return pairwise_scalar_sum15(std::move(terms));
}

Work15 expected_mechanical_energy_work15(const State15& state,
    const SolverOptions15& options) {
    Work15 expected;
    if (!options.terms.surface) return expected;
    const auto order = stable_order15(state);
    for (std::size_t a = 0U; a < order.size(); ++a) {
        for (std::size_t b = a + 1U; b < order.size(); ++b) {
            ++expected.surface_pair_tests;
            expected.scalar_predicates += 2U;
            const long double distance = norm15(
                state.dynamic[order[a]].position
                - state.dynamic[order[b]].position);
            if (distance > 0.0L
                && distance < 3.0L * options.profile.surface_r0) {
                ++expected.surface_active_pairs;
            }
        }
    }
    return expected;
}

long double force_closure15(const std::vector<Vec3l>& forces,
    Work15* work = nullptr) {
    std::vector<Vec3l> ordered = forces;
    std::vector<long double> norms;
    norms.reserve(ordered.size());
    for (Vec3l force : forces) {
        if (work != nullptr) {
            static_cast<void>(counted_predicate_main15(
                *work, finite15(force)));
        }
        norms.push_back(norm15(force));
    }
    const Vec3l sum = pairwise_vec_sum15(std::move(ordered));
    const long double scale = pairwise_scalar_sum15(std::move(norms));
    return norm15(sum) / std::max(scale, 1.0L);
}

struct Topology15 {
    std::uint64_t edges = 0U;
    std::uint64_t components = 0U;
    std::uint64_t satellites = 0U;
};

Topology15 topology15(const State15& state, const Profile15& profile,
    Work15* work = nullptr) {
    Topology15 result;
    const std::size_t count = state.dynamic.size();
    if (count == 0U) return result;
    std::vector<std::size_t> parent(count);
    std::iota(parent.begin(), parent.end(), 0U);
    const auto find = [&](std::size_t start, auto&& self) -> std::size_t {
        if (parent[start] == start) return start;
        parent[start] = self(parent[start], self);
        return parent[start];
    };
    const auto unite = [&](std::size_t lhs, std::size_t rhs) {
        const std::size_t a = find(lhs, find);
        const std::size_t b = find(rhs, find);
        if (a != b) parent[b] = a;
    };
    for (std::size_t lhs = 0U; lhs < count; ++lhs) {
        for (std::size_t rhs = lhs + 1U; rhs < count; ++rhs) {
            if (work != nullptr) ++work->oracle_pair_tests;
            const long double distance = norm15(
                state.dynamic[lhs].position - state.dynamic[rhs].position);
            const bool accepted = work == nullptr
                ? distance <= profile.horizon
                : counted_predicate_main15(*work,
                    distance <= profile.horizon);
            if (!accepted) continue;
            ++result.edges;
            if (work != nullptr) ++work->topology_edges;
            unite(lhs, rhs);
        }
    }
    std::set<std::size_t> roots;
    for (std::size_t index = 0U; index < count; ++index) {
        roots.insert(find(index, find));
    }
    result.components = roots.size();
    std::size_t minimum_id_index = 0U;
    for (std::size_t index = 1U; index < count; ++index) {
        if (state.dynamic[index].id < state.dynamic[minimum_id_index].id) {
            minimum_id_index = index;
        }
    }
    const std::size_t principal_root = find(minimum_id_index, find);
    std::uint64_t principal_size = 0U;
    for (std::size_t index = 0U; index < count; ++index) {
        if (find(index, find) == principal_root) ++principal_size;
    }
    result.satellites = static_cast<std::uint64_t>(count) - principal_size;
    return result;
}

std::array<std::uint64_t, 5> ghost_support15(const Step15& step,
    const Profile15& profile, Work15* work = nullptr) {
    std::array<std::uint64_t, 5> result{{0U, 0U, 0U, 0U, 0U}};
    const auto dynamic_order = stable_order15(step.state);
    std::vector<std::size_t> ghost_order(step.state.ghosts.size());
    std::iota(ghost_order.begin(), ghost_order.end(), 0U);
    std::sort(ghost_order.begin(), ghost_order.end(),
        [&](std::size_t lhs, std::size_t rhs) {
            return step.state.ghosts[lhs].id < step.state.ghosts[rhs].id;
        });
    for (std::size_t dynamic_index : dynamic_order) {
        const Vec3l position = step.state.dynamic[dynamic_index].position;
        for (std::size_t ghost_index : ghost_order) {
            if (work != nullptr) ++work->oracle_pair_tests;
            const Vec3l ghost = step.state.ghosts[ghost_index].position;
            const bool accepted = work == nullptr
                ? norm15(position - ghost) <= profile.horizon
                : counted_predicate_main15(*work,
                    norm15(position - ghost) <= profile.horizon);
            if (!accepted) continue;
            const std::array<bool, 5> faces{ghost.x < 0.0L,
                ghost.x > profile.basin.x, ghost.y < 0.0L,
                ghost.y > profile.basin.y, ghost.z < 0.0L};
            for (std::size_t face = 0U; face < faces.size(); ++face) {
                const bool classified = work == nullptr ? faces[face]
                    : counted_predicate_main15(*work, faces[face]);
                if (classified) ++result[face];
            }
        }
    }
    return result;
}

struct TrajectoryMetrics15 {
    long double position_rms = 0.0L;
    long double position_maximum = 0.0L;
    long double velocity_rms = 0.0L;
    long double velocity_maximum = 0.0L;
    long double momentum_residual = 0.0L;
    long double energy_excess = 0.0L;
    long double pressure_force_closure = 0.0L;
    long double viscosity_force_closure = 0.0L;
    long double surface_force_closure = 0.0L;
    std::uint64_t components = 0U;
    std::uint64_t satellites = 0U;
    long double inset_penetration = 0.0L;
    std::uint64_t top_contacts = 0U;
    std::array<std::uint64_t, 5> support{{0U, 0U, 0U, 0U, 0U}};
    bool support_every_step = false;
    std::string first_failed_gate;
    bool pass = false;
};

Work15 expected_trajectory_metrics_work15(const State15& initial,
    const std::vector<FinalStep15>& steps, const SolverOptions15& options) {
    Work15 expected;
    if (steps.empty()) return expected;
    const auto enumerate_surface = [&](const State15& state) {
        if (!options.terms.surface) return;
        for (std::size_t lhs = 0U; lhs < state.dynamic.size(); ++lhs) {
            for (std::size_t rhs = lhs + 1U;
                 rhs < state.dynamic.size(); ++rhs) {
                ++expected.surface_pair_tests;
                expected.scalar_predicates += 2U;
                const long double distance = norm15(
                    state.dynamic[lhs].position
                    - state.dynamic[rhs].position);
                if (distance > 0.0L
                        && distance < 3.0L * options.profile.surface_r0) {
                    ++expected.surface_active_pairs;
                }
            }
        }
    };
    enumerate_surface(initial);
    const std::uint64_t dynamic = initial.dynamic.size();
    const std::uint64_t ghosts = initial.ghosts.size();
    const std::uint64_t force_and_contact_predicates = 3U * dynamic
        + (options.boundary_mode == BoundaryMode15::AnalyticBox
                ? dynamic : 0U);
    for (const FinalStep15& step : steps) {
        const State15& state = step.value.state;
        enumerate_surface(state);
        expected.oracle_pair_tests += dynamic * ghosts
            + dynamic * (dynamic - (dynamic == 0U ? 0U : 1U)) / 2U;
        expected.scalar_predicates += dynamic * ghosts
            + dynamic * (dynamic - (dynamic == 0U ? 0U : 1U)) / 2U;
        std::uint64_t supported_ghost_pairs = 0U;
        for (const Dynamic15& item : state.dynamic) {
            for (const Ghost15& ghost : state.ghosts) {
                if (norm15(item.position - ghost.position)
                        <= options.profile.horizon) {
                    ++supported_ghost_pairs;
                }
            }
        }
        for (std::size_t lhs = 0U; lhs < state.dynamic.size(); ++lhs) {
            for (std::size_t rhs = lhs + 1U; rhs < state.dynamic.size(); ++rhs) {
                if (norm15(state.dynamic[lhs].position
                        - state.dynamic[rhs].position)
                        <= options.profile.horizon) {
                    ++expected.topology_edges;
                }
            }
        }
        expected.scalar_predicates += 6U + dynamic
            + (options.boundary_mode == BoundaryMode15::AnalyticBox
                    ? 6U * dynamic : 0U)
            + 5U * supported_ghost_pairs
            + force_and_contact_predicates;
    }
    expected.scalar_predicates += 14U;
    return expected;
}

TrajectoryMetrics15 trajectory_metrics15(const State15& initial,
    const std::vector<FinalStep15>& steps, const SolverOptions15& options,
    Work15* work = nullptr) {
    TrajectoryMetrics15 result;
    if (steps.empty()) return result;
    Work15 local_work;
    const auto initial_index = id_index15(initial);
    const Vec3l initial_momentum = momentum15(initial, options.profile);
    const long double initial_energy = mechanical_energy15(
        initial, options, &local_work);
    std::vector<Vec3l> accumulated_step_impulses;
    result.support.fill(std::numeric_limits<std::uint64_t>::max());
    result.support_every_step = true;
    for (std::size_t step_index = 0U; step_index < steps.size(); ++step_index) {
        const FinalStep15& closed = steps[step_index];
        const State15& state = closed.value.state;
        const auto state_index = id_index15(state);
        const bool size_exact = counted_predicate_main15(
            local_work, state_index.size() == initial_index.size());
        if (!size_exact) return result;
        std::vector<long double> position_squares;
        std::vector<long double> velocity_squares;
        position_squares.reserve(state_index.size());
        velocity_squares.reserve(state_index.size());
        bool ids_exact = true;
        for (std::size_t ordered = 0U; ordered < state_index.size(); ++ordered) {
            ids_exact = counted_predicate_main15(local_work,
                    state_index[ordered].first
                        == initial_index[ordered].first)
                && ids_exact;
            const Dynamic15& start = initial.dynamic[initial_index[ordered].second];
            const Dynamic15& current = state.dynamic[state_index[ordered].second];
            const long double position_error = norm15(
                current.position - start.position);
            const long double velocity_error = norm15(
                current.velocity - start.velocity);
            position_squares.push_back(position_error * position_error);
            velocity_squares.push_back(velocity_error * velocity_error);
            result.position_maximum =
                std::max(result.position_maximum, position_error);
            result.velocity_maximum =
                std::max(result.velocity_maximum, velocity_error);
            if (options.boundary_mode == BoundaryMode15::AnalyticBox) {
                const long double lower = options.profile.spacing * 0.5L;
                const Vec3l upper{options.profile.basin.x - lower,
                    options.profile.basin.y - lower,
                    options.profile.basin.z - lower};
                std::array<long double, 6> penetration{
                    lower - current.position.x,
                    lower - current.position.y,
                    lower - current.position.z,
                    current.position.x - upper.x,
                    current.position.y - upper.y,
                    current.position.z - upper.z};
                for (long double value : penetration) {
                    static_cast<void>(counted_predicate_main15(
                        local_work, value <= 0.0L));
                    result.inset_penetration = std::max(
                        result.inset_penetration, std::max(value, 0.0L));
                }
            }
        }
        if (!ids_exact) return result;
        const long double count = static_cast<long double>(state_index.size());
        result.position_rms = std::max(result.position_rms,
            std::sqrt(pairwise_scalar_sum15(std::move(position_squares))
                / count));
        result.velocity_rms = std::max(result.velocity_rms,
            std::sqrt(pairwise_scalar_sum15(std::move(velocity_squares))
                / count));
        result.pressure_force_closure = std::max(
            result.pressure_force_closure,
            force_closure15(closed.value.dynamic_pressure_force,
                &local_work));
        result.viscosity_force_closure = std::max(
            result.viscosity_force_closure,
            force_closure15(closed.value.viscosity_force, &local_work));
        result.surface_force_closure = std::max(
            result.surface_force_closure,
            force_closure15(closed.value.surface_force, &local_work));

        std::vector<Vec3l> ordered_ghost_force;
        ordered_ghost_force.reserve(state_index.size());
        for (const auto& id_and_index : state_index) {
            const std::size_t index = id_and_index.second;
            ordered_ghost_force.push_back(
                index < closed.value.ghost_pressure_force.size()
                ? closed.value.ghost_pressure_force[index] : Vec3l{});
        }
        const Vec3l ghost_force = pairwise_vec_sum15(
            std::move(ordered_ghost_force));
        const Vec3l gravity = options.gravity_mode == GravityMode15::Profile
            ? options.profile.gravity : Vec3l{};
        const Vec3l gravity_force = static_cast<long double>(
            state.dynamic.size()) * options.profile.mass * gravity;
        const std::array<std::uint64_t, 5> support = ghost_support15(
            closed.value, options.profile, &local_work);
        for (std::size_t face = 0U; face < support.size(); ++face) {
            result.support[face] = std::min(result.support[face],
                support[face]);
            result.support_every_step = counted_predicate_main15(
                local_work, support[face] > 0U)
                && result.support_every_step;
        }
        std::vector<Contact15> ordered_contact = closed.value.contact;
        std::sort(ordered_contact.begin(), ordered_contact.end(),
            [](const Contact15& lhs, const Contact15& rhs) {
                return lhs.id < rhs.id;
            });
        std::vector<Vec3l> contact_impulses;
        contact_impulses.reserve(ordered_contact.size());
        for (const Contact15& contact : ordered_contact) {
            contact_impulses.push_back(contact.impulse);
            if (counted_predicate_main15(
                    local_work, (contact.mask & 32U) != 0U)) {
                ++result.top_contacts;
            }
        }
        const Vec3l box_impulse = pairwise_vec_sum15(
            std::move(contact_impulses));
        accumulated_step_impulses.push_back(options.profile.dt
            * (gravity_force + ghost_force) + box_impulse);
        const Vec3l accumulated_external = pairwise_vec_sum15(
            accumulated_step_impulses);
        const long double elapsed = static_cast<long double>(step_index + 1U)
            * options.profile.dt;
        const long double total_mass = static_cast<long double>(state.dynamic.size())
            * options.profile.mass;
        const long double denominator = std::max({norm15(initial_momentum),
            elapsed * total_mass * norm15(gravity),
            options.profile.spacing * total_mass / options.profile.dt});
        const long double momentum_residual = norm15(momentum15(state,
            options.profile) - initial_momentum - accumulated_external)
            / denominator;
        result.momentum_residual =
            std::max(result.momentum_residual, momentum_residual);
        const long double current_energy = mechanical_energy15(
            state, options, &local_work);
        result.energy_excess = std::max(result.energy_excess,
            std::max(current_energy - initial_energy, 0.0L)
                / std::max(std::fabs(initial_energy), 1.0L));
        const Topology15 topology = topology15(
            state, options.profile, &local_work);
        result.components = std::max(result.components, topology.components);
        result.satellites = std::max(result.satellites, topology.satellites);
    }
    const Gates15& gates = options.profile.gates;
    constexpr std::array<std::string_view, 14> gate_names{
        "POSITION_RMS", "POSITION_MAXIMUM", "VELOCITY_RMS",
        "VELOCITY_MAXIMUM", "MOMENTUM_RESIDUAL", "ENERGY_EXCESS",
        "PRESSURE_FORCE_CLOSURE", "VISCOSITY_FORCE_CLOSURE",
        "SURFACE_FORCE_CLOSURE", "TOPOLOGY_COMPONENTS",
        "TOPOLOGY_SATELLITES", "INSET_PENETRATION", "TOP_CONTACT",
        "GHOST_SUPPORT"};
    const std::array<bool, 14> gate_values{
             result.position_rms <= gates.trajectory_position_rms_m,
             result.position_maximum <= gates.trajectory_position_maximum_m,
             result.velocity_rms <= gates.trajectory_velocity_rms_mps,
             result.velocity_maximum <= gates.trajectory_velocity_maximum_mps,
             result.momentum_residual <= gates.momentum_residual,
             result.energy_excess <= gates.energy_excess,
             result.pressure_force_closure <= gates.internal_force_closure,
             result.viscosity_force_closure <= gates.internal_force_closure,
             result.surface_force_closure <= gates.internal_force_closure,
             result.components == 1U, result.satellites == 0U,
             result.inset_penetration == 0.0L,
             result.top_contacts == 0U, result.support_every_step};
    result.pass = true;
    for (std::size_t gate = 0U; gate < gate_values.size(); ++gate) {
        const bool predicate = counted_predicate_main15(local_work,
            gate_values[gate]);
        if (!predicate && result.first_failed_gate.empty()) {
            result.first_failed_gate = gate_names[gate];
        }
        result.pass = predicate
            && result.pass;
    }
    if (work != nullptr) add_work15(*work, local_work);
    return result;
}

Input15 make_input15(std::string phase, std::string lane,
    const Profile15& profile, const State15& base, Vec3l translation,
    GravityMode15 gravity_mode, BoundaryMode15 boundary_mode,
    TermMask15 terms, Mutation15 mutation,
    std::string coordinate_tag = "BASE_BINARY32_WIDENED_LONG_DOUBLE") {
    Input15 input;
    input.phase = std::move(phase);
    input.lane = std::move(lane);
    input.profile_root = profile_root15(profile);
    input.parent_fixture_root = NCGP15_PARENT_TIGHT_FIXTURE_ROOT;
    input.coordinate_tag = std::move(coordinate_tag);
    input.base = base;
    input.translation = translation;
    input.gravity_mode = gravity_mode;
    input.boundary_mode = boundary_mode;
    input.terms = terms;
    input.mutation = mutation;
    input.root = input_root15(input, &input.root_work);
    return input;
}

Work15 actual_input_work15(const Input15& input) {
    Work15 result = input.root_work;
    result.fixture_records = input.base.dynamic.size();
    result.ghost_records = input.base.ghosts.size();
    return result;
}

Work15 input_work15(const Input15& input) {
    Work15 result;
    result.fixture_records = input.base.dynamic.size();
    result.ghost_records = input.base.ghosts.size();
    result.portable_fields = 18U + 10U * input.base.dynamic.size()
        + 4U * input.base.ghosts.size();
    result.hash_derivations = 1U;
    return result;
}

void add_step_work15(Work15& actual, Work15& expected,
    const FinalStep15& step) {
    add_work15(actual, step.work.actual);
    add_work15(expected, step.work.expected);
}

void close_receipt15(RootedReceipt15& receipt, const Work15& actual,
    const Work15& expected) {
    receipt.work = seal_work15(actual, expected);
    receipt.apparatus_valid = receipt.apparatus_valid && receipt.work.exact;
    receipt.pass = receipt.pass && receipt.apparatus_valid;
    receipt.result_root = receipt_root15(receipt);
}

struct TermControl15 {
    RootedReceipt15 receipt;
    std::vector<Input15> inputs;
    std::vector<FinalStep15> steps;
    std::vector<long double> analytical;
    std::vector<long double> observed;
    std::vector<std::string> labels;
    std::vector<std::string> semantic_roots;
    std::vector<Vec3l> candidate_gradient;
    std::vector<Vec3l> finite_difference_gradient;
    std::string gradient_root;
};

std::string scalar_observable_root15(std::string_view case_name,
    std::string_view variant, std::string_view input_root,
    const std::vector<long double>& analytical,
    const std::vector<long double>& observed, std::string_view outcome) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.term-oracle.v1");
    put_string(bytes, case_name);
    put_string(bytes, variant);
    put_string(bytes, input_root);
    put_string(bytes, "CHECKED_BINARY64");
    put_u64(bytes, analytical.size());
    for (long double value : analytical) put_f64(bytes, value);
    put_u64(bytes, observed.size());
    for (long double value : observed) put_f64(bytes, value);
    put_string(bytes, outcome);
    return root15(std::move(bytes));
}

std::string gradient_observable_root15(std::string_view case_name,
    std::string_view variant, std::string_view input_root,
    const State15& state, const std::vector<Vec3l>& candidate,
    const std::vector<Vec3l>& finite_difference, long double relative_l2,
    long double maximum_absolute, const Gates15& gates,
    std::string_view outcome, Work15* work = nullptr) {
    if (candidate.size() != state.dynamic.size()
        || finite_difference.size() != state.dynamic.size()) {
        throw std::logic_error("NCGP15 gradient observable shape mismatch");
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.term-oracle.v1");
    put_string(bytes, case_name);
    put_string(bytes, variant);
    put_string(bytes, input_root);
    put_string(bytes, "STABLE_ID_CANDIDATE_AND_CENTRAL_FD_GRADIENT");
    const auto order = stable_order15(state);
    put_u64(bytes, order.size());
    for (std::size_t index : order) {
        put_u32(bytes, state.dynamic[index].id);
        put_vec(bytes, candidate[index]);
        put_vec(bytes, finite_difference[index]);
    }
    put_f64(bytes, relative_l2);
    put_f64(bytes, maximum_absolute);
    put_f64(bytes, gates.gradient_relative_l2);
    put_f64(bytes, gates.gradient_maximum_n);
    put_string(bytes, outcome);
    if (work != nullptr) {
        work->portable_fields += 11U + 7U * order.size();
        ++work->hash_derivations;
    }
    return root15(std::move(bytes));
}

void add_scalar_root_work15(Work15& actual, Work15& expected,
    std::size_t analytical_count, std::size_t observed_count) {
    const std::uint64_t fields = 8U + analytical_count + observed_count;
    actual.portable_fields += fields;
    expected.portable_fields += fields;
    ++actual.hash_derivations;
    ++expected.hash_derivations;
}

TermControl15 normal_viscosity_control15(const Profile15& profile) {
    TermControl15 result;
    result.receipt.name = "normal-viscosity-pair";
    result.receipt.variant = "corrected";
    const TermMask15 terms{true, true, false, false};
    const State15 base = pair_fixture15({0.10L, 0.10L, 0.10L},
        {-1.0L, 0.0L, 0.0L}, {0.20L, 0.10L, 0.10L},
        {1.0L, 0.0L, 0.0L});
    const Input15 input = make_input15("A", result.receipt.name, profile,
        base, {}, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured, terms, Mutation15::None);
    result.inputs.push_back(input);
    result.receipt.input_root = input.root;
    const State15 state = materialize_input15(input);
    const SolverOptions15 options = manufactured_options15(profile, terms);
    result.steps.push_back(run_closed_step15("candidate-csr", candidate_step15,
        state, input, 1U, options));
    result.steps.push_back(run_closed_step15("oracle-all-pairs", oracle_step15,
        state, input, 1U, options));
    const Vec3l axis = (state.dynamic[1].position
        - state.dynamic[0].position)
        / norm15(state.dynamic[1].position - state.dynamic[0].position);
    const long double initial_relative = dot15(
        state.dynamic[1].velocity - state.dynamic[0].velocity, axis);
    const long double omega = kernel_omega15(norm15(
        state.dynamic[1].position - state.dynamic[0].position), profile);
    const long double expected_relative = initial_relative
        / (1.0L + 2.0L * profile.lambda_v * omega * profile.dt
            / profile.rho0);
    result.analytical = {expected_relative};
    result.labels = {"implicit_relative_normal_velocity"};
    bool pass = true;
    for (const FinalStep15& step : result.steps) {
        const long double observed = dot15(
            step.value.state.dynamic[1].velocity
                - step.value.state.dynamic[0].velocity,
            axis);
        result.observed.push_back(observed);
        const long double relative = std::fabs(observed - expected_relative)
            / std::fabs(expected_relative);
        pass = step.transaction_committed
            && relative <= profile.gates.normal_decay_relative && pass;
        result.receipt.child_roots.push_back(step.value.result_root);
    }
    Work15 comparison_work;
    const Comparison15 comparison = compare_steps15(result.steps[0].value,
        result.steps[1].value, profile.gates, &comparison_work);
    result.observed.push_back(comparison.position_rms);
    result.labels.push_back("candidate_oracle_position_rms_m");
    pass = pass && comparison.pass;
    const std::string observable = scalar_observable_root15(
        result.receipt.name, result.receipt.variant, input.root,
        result.analytical, result.observed, pass ? "PASS" : "REJECTED");
    result.receipt.child_roots.push_back(observable);
    Work15 actual = actual_input_work15(input);
    Work15 expected = input_work15(input);
    for (const FinalStep15& step : result.steps) {
        add_step_work15(actual, expected, step);
    }
    add_work15(actual, comparison_work);
    add_work15(expected, expected_comparison_work15(state.dynamic.size()));
    add_scalar_root_work15(actual, expected, result.analytical.size(),
        result.observed.size());
    result.receipt.apparatus_valid = comparison.pass
        && std::all_of(result.steps.begin(),
        result.steps.end(), [](const FinalStep15& step) {
            return main_step_apparatus15(step);
        });
    result.receipt.pass = pass;
    result.receipt.typed_outcome = pass ? "PASS" : "TERM_REJECTED";
    close_receipt15(result.receipt, actual, expected);
    return result;
}

TermControl15 tangential_zero_control15(const Profile15& profile) {
    TermControl15 result;
    result.receipt.name = "tangential-mu-zero-pair";
    result.receipt.variant = "corrected";
    const TermMask15 terms{true, false, true, false};
    const State15 base = pair_fixture15({0.10L, 0.10L, 0.10L},
        {0.0L, -1.0L, 0.0L}, {0.20L, 0.10L, 0.10L},
        {0.0L, 1.0L, 0.0L});
    const Input15 input = make_input15("A", result.receipt.name, profile,
        base, {}, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured, terms, Mutation15::None);
    result.inputs.push_back(input);
    result.receipt.input_root = input.root;
    const State15 state = materialize_input15(input);
    const SolverOptions15 options = manufactured_options15(profile, terms);
    result.steps.push_back(run_closed_step15("candidate-csr", candidate_step15,
        state, input, 1U, options));
    result.steps.push_back(run_closed_step15("oracle-all-pairs", oracle_step15,
        state, input, 1U, options));
    long double maximum = 0.0L;
    bool pass = true;
    for (const FinalStep15& step : result.steps) {
        for (std::size_t index = 0U; index < state.dynamic.size(); ++index) {
            const Vec3l expected_position = state.dynamic[index].position
                + profile.dt * state.dynamic[index].velocity;
            maximum = std::max(maximum, norm15(
                step.value.state.dynamic[index].position - expected_position));
            maximum = std::max(maximum, norm15(
                step.value.state.dynamic[index].velocity
                    - state.dynamic[index].velocity));
        }
        pass = step.transaction_committed && pass;
        result.receipt.child_roots.push_back(step.value.result_root);
    }
    Work15 comparison_work;
    const Comparison15 comparison = compare_steps15(result.steps[0].value,
        result.steps[1].value, profile.gates, &comparison_work);
    pass = pass && comparison.pass
        && maximum <= profile.gates.tangential_change_absolute;
    result.analytical = {0.0L};
    result.observed = {maximum, comparison.position_rms};
    result.labels = {"expected_absolute_change",
        "candidate_oracle_position_rms_m"};
    result.receipt.child_roots.push_back(scalar_observable_root15(
        result.receipt.name, result.receipt.variant, input.root,
        result.analytical, result.observed, pass ? "PASS" : "REJECTED"));
    Work15 actual = actual_input_work15(input);
    Work15 expected = input_work15(input);
    for (const FinalStep15& step : result.steps) {
        add_step_work15(actual, expected, step);
    }
    add_work15(actual, comparison_work);
    add_work15(expected, expected_comparison_work15(state.dynamic.size()));
    add_scalar_root_work15(actual, expected, result.analytical.size(),
        result.observed.size());
    result.receipt.apparatus_valid = comparison.pass
        && std::all_of(result.steps.begin(),
        result.steps.end(), [](const FinalStep15& step) {
            return main_step_apparatus15(step);
        });
    result.receipt.pass = pass;
    result.receipt.typed_outcome = pass ? "PASS" : "TERM_REJECTED";
    close_receipt15(result.receipt, actual, expected);
    return result;
}

TermControl15 surface_pair_control15(const Profile15& profile,
    std::string name, long double left_x, long double right_x,
    Vec3l translation) {
    TermControl15 result;
    result.receipt.name = std::move(name);
    result.receipt.variant = translation.x == 0.0L
        ? "base" : (translation.x < 1000.0L ? "translation-0.75"
                                            : "translation-100000");
    const TermMask15 terms{true, false, false, true};
    const State15 base = pair_fixture15({left_x, 0.10L, 0.10L}, {},
        {right_x, 0.10L, 0.10L}, {});
    const Input15 input = make_input15("A", result.receipt.name, profile,
        base, translation, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured, terms, Mutation15::None,
        translation.x == 0.0L ? "BASE_BINARY32_WIDENED_LONG_DOUBLE"
            : "BASE_BINARY32_WIDENED_PLUS_BINARY64_TRANSLATION");
    result.inputs.push_back(input);
    result.receipt.input_root = input.root;
    const State15 state = materialize_input15(input);
    const SolverOptions15 options = manufactured_options15(profile, terms);
    result.steps.push_back(run_closed_step15("candidate-csr", candidate_step15,
        state, input, 1U, options));
    result.steps.push_back(run_closed_step15("oracle-all-pairs", oracle_step15,
        state, input, 1U, options));
    const State15 permuted_base = permuted_state15(base);
    const Input15 permuted_input = make_input15("A",
        result.receipt.name + "-permuted", profile, permuted_base,
        translation, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured, terms, Mutation15::None,
        translation.x == 0.0L ? "PHYSICALLY_PERMUTED_BINARY32"
            : "PHYSICALLY_PERMUTED_BINARY32_PLUS_BINARY64_TRANSLATION");
    result.inputs.push_back(permuted_input);
    result.steps.push_back(run_closed_step15("candidate-csr-permuted",
        candidate_step15, materialize_input15(permuted_input),
        permuted_input, 1U, options));
    const long double initial_separation = norm15(
        state.dynamic[1].position - state.dynamic[0].position);
    const long double expected_separation =
        oracle_surface_pair_root15(initial_separation, options);
    const Vec3l initial_com = 0.5L
        * (state.dynamic[0].position + state.dynamic[1].position);
    result.analytical = {expected_separation, 0.0L};
    result.labels = {"bracketed_separation_m", "com_relative_error"};
    bool pass = true;
    for (const FinalStep15& step : result.steps) {
        const long double separation = norm15(
            step.value.state.dynamic[1].position
                - step.value.state.dynamic[0].position);
        const Vec3l com = 0.5L
            * (step.value.state.dynamic[0].position
                + step.value.state.dynamic[1].position);
        const long double com_error = norm15(com - initial_com)
            / std::max(profile.spacing, 1.0e-30L);
        result.observed.push_back(separation);
        result.observed.push_back(com_error);
        pass = step.transaction_committed
            && std::fabs(separation - expected_separation)
                <= profile.gates.pair_position_m
            && com_error <= profile.gates.center_of_mass_relative && pass;
        result.receipt.child_roots.push_back(step.value.result_root);
    }
    Work15 comparison_work;
    const Comparison15 comparison = compare_steps15(result.steps[0].value,
        result.steps[1].value, profile.gates, &comparison_work);
    const Comparison15 permutation = compare_steps15(result.steps[0].value,
        result.steps[2].value, profile.gates, &comparison_work);
    result.semantic_roots.push_back(semantic_step_signature15(
        result.steps[0], input.root));
    result.semantic_roots.push_back(semantic_step_signature15(
        result.steps[2], input.root));
    const bool semantic_exact = counted_predicate_main15(comparison_work,
        result.semantic_roots[0] == result.semantic_roots[1]);
    result.observed.push_back(comparison.position_rms);
    result.observed.push_back(permutation.position_rms);
    pass = pass && comparison.pass && permutation.pass && semantic_exact;
    result.receipt.child_roots.push_back(scalar_observable_root15(
        result.receipt.name, result.receipt.variant, input.root,
        result.analytical, result.observed, pass ? "PASS" : "REJECTED"));
    result.receipt.child_roots.insert(result.receipt.child_roots.end(),
        result.semantic_roots.begin(), result.semantic_roots.end());
    Work15 actual = actual_input_work15(input);
    Work15 expected = input_work15(input);
    add_work15(actual, actual_input_work15(permuted_input));
    add_work15(expected, input_work15(permuted_input));
    for (const FinalStep15& step : result.steps) {
        add_step_work15(actual, expected, step);
    }
    add_work15(actual, comparison_work);
    add_work15(expected, expected_comparison_work15(state.dynamic.size()));
    add_work15(expected, expected_comparison_work15(state.dynamic.size()));
    ++expected.scalar_predicates;
    add_scalar_root_work15(actual, expected, result.analytical.size(),
        result.observed.size());
    const std::uint64_t semantic_fields = semantic_signature_fields15(
            result.steps[0])
        + semantic_signature_fields15(result.steps[2]);
    const std::uint64_t expected_semantic_fields =
        expected_semantic_signature_fields15(result.steps[0])
        + expected_semantic_signature_fields15(result.steps[2]);
    actual.portable_fields += semantic_fields;
    expected.portable_fields += expected_semantic_fields;
    actual.hash_derivations += 10U;
    expected.hash_derivations += 10U;
    result.receipt.apparatus_valid = comparison.pass && permutation.pass
        && semantic_exact && std::all_of(result.steps.begin(),
        result.steps.end(), [](const FinalStep15& step) {
            return main_step_apparatus15(step);
        });
    result.receipt.pass = pass;
    result.receipt.typed_outcome = pass ? "PASS" : "TERM_REJECTED";
    close_receipt15(result.receipt, actual, expected);
    return result;
}

TermControl15 tetra_control15(const Profile15& profile) {
    TermControl15 result;
    result.receipt.name = "combined-tetrahedron";
    result.receipt.variant = "corrected";
    const TermMask15 terms{true, true, false, true};
    const State15 base = tetra_fixture15();
    const Input15 input = make_input15("A", result.receipt.name, profile,
        base, {}, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured, terms, Mutation15::None);
    result.inputs.push_back(input);
    result.receipt.input_root = input.root;
    const State15 state = materialize_input15(input);
    const SolverOptions15 options = manufactured_options15(profile, terms);
    result.steps.push_back(run_closed_step15("candidate-csr", candidate_step15,
        state, input, 1U, options));
    result.steps.push_back(run_closed_step15("oracle-all-pairs", oracle_step15,
        state, input, 1U, options));
    Work15 gradient_work;
    std::vector<Vec3l> positions;
    positions.reserve(state.dynamic.size());
    for (const Dynamic15& item : state.dynamic) positions.push_back(item.position);
    const std::vector<long double> multipliers(state.dynamic.size(), 0.0L);
    const EnergyGradient15 candidate_gradient = candidate_energy_gradient15(
        state, positions, multipliers, options,
        MultiplierMode15::AugmentedLagrangian, true, gradient_work);
    const Work15 expected_gradient_work =
        candidate_expected_energy_gradient_work15(
            state, positions, multipliers, options, true);
    const GradientCheck15 gradient_check = oracle_central_difference_gradient15(
        state, positions, multipliers, options, candidate_gradient.gradient);
    result.candidate_gradient = candidate_gradient.gradient;
    result.finite_difference_gradient = gradient_check.finite_difference;
    Work15 comparison_work;
    const Comparison15 comparison = compare_steps15(result.steps[0].value,
        result.steps[1].value, profile.gates, &comparison_work);
    bool pass = candidate_gradient.valid && gradient_check.valid
        && gradient_check.relative_l2 <= profile.gates.gradient_relative_l2
        && gradient_check.maximum_absolute_n
            <= profile.gates.gradient_maximum_n
        && comparison.pass && active_ids15(result.steps[0].value).empty()
        && active_ids15(result.steps[1].value).empty();
    for (const FinalStep15& step : result.steps) {
        pass = step.transaction_committed && pass;
        result.receipt.child_roots.push_back(step.value.result_root);
    }
    result.analytical = {profile.gates.gradient_relative_l2,
        profile.gates.gradient_maximum_n};
    result.observed = {gradient_check.relative_l2,
        gradient_check.maximum_absolute_n, comparison.position_rms,
        comparison.position_maximum};
    result.labels = {"gradient_relative_l2", "gradient_maximum_n",
        "candidate_oracle_position_rms_m",
        "candidate_oracle_position_maximum_m"};
    result.receipt.child_roots.push_back(scalar_observable_root15(
        result.receipt.name, result.receipt.variant, input.root,
        result.analytical, result.observed, pass ? "PASS" : "REJECTED"));
    Work15 actual = actual_input_work15(input);
    Work15 expected = input_work15(input);
    for (const FinalStep15& step : result.steps) {
        add_step_work15(actual, expected, step);
    }
    add_work15(actual, gradient_work);
    add_work15(actual, gradient_check.work);
    add_work15(actual, comparison_work);
    add_work15(expected, expected_gradient_work);
    add_work15(expected, gradient_check.expected_work);
    add_work15(expected, expected_comparison_work15(state.dynamic.size()));
    Work15 gradient_root_work;
    result.gradient_root = gradient_observable_root15(result.receipt.name,
        result.receipt.variant, input.root, state, result.candidate_gradient,
        result.finite_difference_gradient, gradient_check.relative_l2,
        gradient_check.maximum_absolute_n, profile.gates,
        pass ? "PASS" : "REJECTED", &gradient_root_work);
    result.receipt.child_roots.push_back(result.gradient_root);
    add_work15(actual, gradient_root_work);
    expected.portable_fields += 11U + 7U * state.dynamic.size();
    ++expected.hash_derivations;
    add_scalar_root_work15(actual, expected, result.analytical.size(),
        result.observed.size());
    result.receipt.apparatus_valid = candidate_gradient.valid
        && gradient_check.valid
        && equal_work15(gradient_work, expected_gradient_work)
        && equal_work15(
            gradient_check.work, gradient_check.expected_work)
        && comparison.pass
        && std::all_of(result.steps.begin(), result.steps.end(),
            [](const FinalStep15& step) {
                return main_step_apparatus15(step);
            });
    result.receipt.pass = pass;
    result.receipt.typed_outcome = pass ? "PASS" : "TERM_REJECTED";
    close_receipt15(result.receipt, actual, expected);
    return result;
}

struct EnergyControl15 {
    RootedReceipt15 receipt;
    Input15 input;
    std::vector<FinalStep15> forward;
    std::vector<FinalStep15> reverse;
    long double maximum_positive_excess = 0.0L;
    long double maximum_positive_excess_j = 0.0L;
    long double reversible_energy_drift = 0.0L;
    long double return_position_error = 0.0L;
    Work15 observable_work;
    Work15 expected_observable_work;
    bool observable_complete = false;
};

bool append_sequential_step15(std::vector<FinalStep15>& output,
    State15& accepted, const Input15& input, std::uint32_t one_based_step,
    const SolverOptions15& options, StepFunction15 function,
    std::string implementation) {
    FinalStep15 closed = run_closed_step15(std::move(implementation), function,
        accepted, input, one_based_step, options);
    const bool committed = closed.transaction_committed;
    if (committed) accepted = closed.value.state;
    output.push_back(std::move(closed));
    return committed;
}

EnergyControl15 surface_energy_control15(const Profile15& profile) {
    EnergyControl15 result;
    result.receipt.name = "surface-tetrahedron-energy-32";
    result.receipt.variant = "corrected";
    const TermMask15 terms{true, false, false, true};
    const State15 base = tetra_fixture15();
    const Input15 input = make_input15("A", result.receipt.name, profile,
        base, {}, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured, terms, Mutation15::None);
    result.input = input;
    result.receipt.input_root = input.root;
    const SolverOptions15 options = manufactured_options15(profile, terms);
    State15 accepted = materialize_input15(input);
    const long double initial_energy = mechanical_energy15(accepted, options,
        &result.observable_work);
    add_work15(result.expected_observable_work,
        expected_mechanical_energy_work15(accepted, options));
    bool pass = true;
    for (std::uint32_t step = 1U; step <= 32U; ++step) {
        const bool committed = append_sequential_step15(result.forward,
            accepted, input, step, options, candidate_step15, "candidate-csr");
        pass = pass && committed;
        if (!committed) break;
        const long double energy = mechanical_energy15(accepted, options,
            &result.observable_work);
        add_work15(result.expected_observable_work,
            expected_mechanical_energy_work15(accepted, options));
        const long double excess_j = std::max(energy - initial_energy, 0.0L);
        result.maximum_positive_excess_j = std::max(
            result.maximum_positive_excess_j, excess_j);
        result.maximum_positive_excess = std::max(
            result.maximum_positive_excess,
            excess_j / std::max(std::fabs(initial_energy), 1.0L));
    }
    pass = pass && result.forward.size() == 32U
        && result.maximum_positive_excess
            <= profile.gates.surface_energy_relative_excess
        && result.maximum_positive_excess_j
            <= profile.gates.surface_energy_absolute_excess_j;
    for (const FinalStep15& step : result.forward) {
        result.receipt.child_roots.push_back(step.value.result_root);
    }
    result.receipt.child_roots.push_back(scalar_observable_root15(
        result.receipt.name, result.receipt.variant, input.root,
        {profile.gates.surface_energy_relative_excess,
            profile.gates.surface_energy_absolute_excess_j},
        {result.maximum_positive_excess,
            result.maximum_positive_excess_j},
        pass ? "PASS" : "REJECTED"));
    Work15 actual = actual_input_work15(input);
    Work15 expected = input_work15(input);
    for (const FinalStep15& step : result.forward) {
        add_step_work15(actual, expected, step);
    }
    add_work15(actual, result.observable_work);
    add_work15(expected, result.expected_observable_work);
    add_scalar_root_work15(actual, expected, 2U, 2U);
    result.receipt.apparatus_valid = equal_work15(result.observable_work,
        result.expected_observable_work)
        && std::all_of(result.forward.begin(),
        result.forward.end(), [](const FinalStep15& step) {
            return main_step_apparatus15(step);
        });
    result.receipt.pass = pass;
    result.receipt.typed_outcome = pass ? "PASS" : "ENERGY_REJECTED";
    close_receipt15(result.receipt, actual, expected);
    return result;
}

EnergyControl15 reversible_energy_control15(const Profile15& profile) {
    EnergyControl15 result;
    result.receipt.name = "surface-tetrahedron-reversible-16-plus-16";
    result.receipt.variant = "corrected-inviscid";
    const TermMask15 terms{true, false, false, true};
    const State15 base = tetra_fixture15();
    const Input15 input = make_input15("A", result.receipt.name, profile,
        base, {}, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured, terms, Mutation15::None);
    result.input = input;
    result.receipt.input_root = input.root;
    const SolverOptions15 options = manufactured_options15(profile, terms);
    const State15 initial = materialize_input15(input);
    State15 accepted = initial;
    const long double initial_energy = mechanical_energy15(initial, options,
        &result.observable_work);
    add_work15(result.expected_observable_work,
        expected_mechanical_energy_work15(initial, options));
    bool pass = true;
    for (std::uint32_t step = 1U; step <= 16U; ++step) {
        const bool committed = append_sequential_step15(result.forward,
            accepted, input, step, options, candidate_step15, "candidate-csr");
        pass = pass && committed;
        if (!committed) break;
    }
    if (result.forward.size() == 16U && pass) {
        for (Dynamic15& item : accepted.dynamic) item.velocity = -item.velocity;
        for (std::uint32_t step = 17U; step <= 32U; ++step) {
            const bool committed = append_sequential_step15(result.reverse,
                accepted, input, step, options, candidate_step15,
                "candidate-csr-time-reversed");
            pass = pass && committed;
            if (!committed) break;
        }
    }
    result.observable_complete = result.forward.size() == 16U
        && result.reverse.size() == 16U && pass;
    if (result.observable_complete) {
        const long double final_energy = mechanical_energy15(accepted, options,
            &result.observable_work);
        add_work15(result.expected_observable_work,
            expected_mechanical_energy_work15(accepted, options));
        result.reversible_energy_drift = std::fabs(
            final_energy - initial_energy)
            / std::max(std::fabs(initial_energy), 1.0L);
        const auto initial_index = id_index15(initial);
        const auto final_index = id_index15(accepted);
        std::vector<long double> squares;
        squares.reserve(initial_index.size());
        for (std::size_t index = 0U; index < initial_index.size(); ++index) {
            const Vec3l difference =
                initial.dynamic[initial_index[index].second].position
                - accepted.dynamic[final_index[index].second].position;
            squares.push_back(dot15(difference, difference));
        }
        result.return_position_error = std::sqrt(
            pairwise_scalar_sum15(std::move(squares))
            / static_cast<long double>(initial_index.size()));
    }
    pass = pass && result.forward.size() == 16U
        && result.reverse.size() == 16U
        && result.reversible_energy_drift
            <= profile.gates.reversible_energy_drift;
    for (const FinalStep15& step : result.forward) {
        result.receipt.child_roots.push_back(step.value.result_root);
    }
    for (const FinalStep15& step : result.reverse) {
        result.receipt.child_roots.push_back(step.value.result_root);
    }
    result.receipt.child_roots.push_back(scalar_observable_root15(
        result.receipt.name, result.receipt.variant, input.root,
        {profile.gates.reversible_energy_drift, 16.0L, 16.0L},
        {result.reversible_energy_drift,
            result.return_position_error,
            static_cast<long double>(result.forward.size()),
            static_cast<long double>(result.reverse.size()),
            result.observable_complete ? 1.0L : 0.0L},
        pass ? "PASS" : "REJECTED"));
    Work15 actual = actual_input_work15(input);
    Work15 expected = input_work15(input);
    for (const FinalStep15& step : result.forward) {
        add_step_work15(actual, expected, step);
    }
    for (const FinalStep15& step : result.reverse) {
        add_step_work15(actual, expected, step);
    }
    add_work15(actual, result.observable_work);
    add_work15(expected, result.expected_observable_work);
    add_scalar_root_work15(actual, expected, 3U, 5U);
    result.receipt.apparatus_valid = equal_work15(result.observable_work,
            result.expected_observable_work)
        && std::all_of(result.forward.begin(), result.forward.end(),
            [](const FinalStep15& step) {
                return main_step_apparatus15(step);
            })
        && std::all_of(result.reverse.begin(), result.reverse.end(),
            [](const FinalStep15& step) {
                return main_step_apparatus15(step);
            });
    result.receipt.pass = pass;
    result.receipt.typed_outcome = pass ? "PASS" : "ENERGY_REJECTED";
    close_receipt15(result.receipt, actual, expected);
    return result;
}

long double maximum_state_difference15(const State15& lhs,
    const State15& rhs) {
    const auto lhs_index = id_index15(lhs);
    const auto rhs_index = id_index15(rhs);
    if (lhs_index.size() != rhs_index.size()) {
        return std::numeric_limits<long double>::infinity();
    }
    long double result = 0.0L;
    for (std::size_t index = 0U; index < lhs_index.size(); ++index) {
        if (lhs_index[index].first != rhs_index[index].first) {
            return std::numeric_limits<long double>::infinity();
        }
        const Dynamic15& a = lhs.dynamic[lhs_index[index].second];
        const Dynamic15& b = rhs.dynamic[rhs_index[index].second];
        result = std::max(result, norm15(a.position - b.position));
        result = std::max(result, norm15(a.velocity - b.velocity));
    }
    return result;
}

struct MutationControl15 {
    RootedReceipt15 receipt;
    std::vector<Input15> inputs;
    std::optional<FinalStep15> corrected;
    std::optional<FinalStep15> oracle;
    std::optional<FinalStep15> mutated;
    Comparison15 corrected_oracle;
    Comparison15 mutated_oracle;
    long double state_difference = 0.0L;
    std::string production_outcome = "NOT_RUN";
    bool production_work_exact = false;
    bool roots_distinct = false;
    bool expected_rejection_observed = false;
};

MutationControl15 solver_mutation_control15(const Profile15& profile,
    std::string name, const State15& base, TermMask15 terms,
    GravityMode15 gravity, BoundaryMode15 boundary, Mutation15 mutation) {
    MutationControl15 result;
    result.receipt.name = std::move(name);
    result.receipt.variant = std::string(mutation_name15(mutation));
    const Input15 corrected_input = make_input15("A-MUTATION",
        result.receipt.name + "-corrected", profile, base, {}, gravity,
        boundary, terms, Mutation15::None);
    const Input15 mutated_input = make_input15("A-MUTATION",
        result.receipt.name + "-mutated", profile, base, {}, gravity,
        boundary, terms, mutation);
    result.inputs.push_back(corrected_input);
    result.inputs.push_back(mutated_input);
    result.receipt.input_root = mutated_input.root;
    const State15 corrected_state = materialize_input15(corrected_input);
    const State15 mutated_state = materialize_input15(mutated_input);
    SolverOptions15 corrected_options;
    corrected_options.profile = profile;
    corrected_options.terms = terms;
    corrected_options.gravity_mode = gravity;
    corrected_options.boundary_mode = boundary;
    SolverOptions15 mutated_options = corrected_options;
    mutated_options.mutation = mutation;
    result.corrected = run_closed_step15("candidate-csr-corrected",
        candidate_step15, corrected_state, corrected_input, 1U,
        corrected_options);
    result.oracle = run_closed_step15("oracle-all-pairs-corrected",
        oracle_step15, corrected_state, corrected_input, 1U,
        corrected_options);
    result.mutated = run_closed_step15("candidate-csr-mutated",
        candidate_step15, mutated_state, mutated_input, 1U, mutated_options);
    Work15 comparison_work;
    result.corrected_oracle = compare_steps15(result.corrected->value,
        result.oracle->value, profile.gates, &comparison_work);
    result.mutated_oracle = compare_steps15(result.mutated->value,
        result.oracle->value, profile.gates, &comparison_work);
    result.state_difference = maximum_state_difference15(
        result.corrected->value.state, result.mutated->value.state);
    result.production_outcome = result.mutated->value.outcome;
    result.production_work_exact = result.corrected->work.exact
        && result.mutated->work.exact;
    result.roots_distinct = corrected_input.root != mutated_input.root
        && result.corrected->value.result_root
            != result.mutated->value.result_root;
    const bool mutated_typed_noncommit =
        !result.mutated->transaction_committed
        && result.mutated->root_closed && result.mutated->work.exact
        && result.mutated->value.apparatus_valid
        && finite_step_payload15(result.mutated->value);
    result.expected_rejection_observed =
        result.corrected->transaction_committed
        && result.oracle->transaction_committed
        && result.corrected_oracle.pass
        && result.roots_distinct
        && (mutated_typed_noncommit || !result.mutated_oracle.pass);
    result.receipt.child_roots = {result.corrected->value.result_root,
        result.oracle->value.result_root, result.mutated->value.result_root,
        scalar_observable_root15(result.receipt.name,
            result.receipt.variant, mutated_input.root, {1.0L, 0.0L},
            {result.corrected_oracle.pass ? 1.0L : 0.0L,
                result.mutated_oracle.pass ? 1.0L : 0.0L,
                result.state_difference},
            result.expected_rejection_observed ? "EXPECTED_REJECTION"
                                               : "MUTATION_SURVIVED")};
    Work15 actual = actual_input_work15(corrected_input);
    Work15 expected = input_work15(corrected_input);
    add_work15(actual, actual_input_work15(mutated_input));
    add_work15(expected, input_work15(mutated_input));
    add_step_work15(actual, expected, *result.corrected);
    add_step_work15(actual, expected, *result.oracle);
    add_step_work15(actual, expected, *result.mutated);
    add_work15(actual, comparison_work);
    add_work15(expected, expected_comparison_work15(
        corrected_state.dynamic.size()));
    add_work15(expected, expected_comparison_work15(
        corrected_state.dynamic.size()));
    add_scalar_root_work15(actual, expected, 2U, 3U);
    result.receipt.apparatus_valid = result.corrected->root_closed
        && result.oracle->root_closed && result.mutated->root_closed
        && result.corrected->work.exact && result.oracle->work.exact
        && result.mutated->work.exact
        && main_step_apparatus15(*result.corrected)
        && main_step_apparatus15(*result.oracle)
        && result.corrected_oracle.pass
        && result.mutated->value.apparatus_valid;
    result.receipt.pass = result.expected_rejection_observed;
    result.receipt.typed_outcome = result.expected_rejection_observed
        ? "EXPECTED_REJECTION" : "MUTATION_SURVIVED";
    close_receipt15(result.receipt, actual, expected);
    return result;
}

Work15 expected_candidate_admission_rejection_work15(const State15& state) {
    Work15 expected;
    expected.fixture_records = state.dynamic.size();
    expected.ghost_records = state.ghosts.size();
    constexpr std::uint64_t kProfileAndOptionPredicates = 58U;
    expected.scalar_predicates = kProfileAndOptionPredicates + 2U
        + 4U * state.dynamic.size() + 2U * state.ghosts.size();
    return expected;
}

Work15 expected_candidate_capacity_rejection_work15(const State15& state,
    const Profile15& profile) {
    Work15 expected = expected_candidate_admission_rejection_work15(state);
    ++expected.graph_builds;
    const auto order = stable_order15(state);
    if (order.empty()) return expected;
    const auto cell = [&](Vec3l value) {
        return std::array<long long, 3>{
            static_cast<long long>(std::floor(value.x / profile.horizon)),
            static_cast<long long>(std::floor(value.y / profile.horizon)),
            static_cast<long long>(std::floor(value.z / profile.horizon))};
    };
    const auto reference_cell = cell(state.dynamic[order.front()].position);
    if (!std::all_of(order.begin(), order.end(), [&](std::size_t index) {
            return cell(state.dynamic[index].position) == reference_cell;
        })) {
        throw std::logic_error(
            "NCGP15 capacity fixture escaped its independently checked cell");
    }
    for (std::size_t owner : order) {
        std::uint64_t degree = 0U;
        for (std::size_t neighbor : order) {
            ++expected.graph_candidates;
            const long double distance = norm15(state.dynamic[owner].position
                - state.dynamic[neighbor].position);
            if (distance > profile.horizon) continue;
            ++expected.accepted_pairs;
            ++degree;
            if ((owner != neighbor && distance == 0.0L)
                    || degree > profile.maximum_neighbors) {
                return expected;
            }
        }
    }
    return expected;
}

FinalStep15 close_raw_rejection_step15(const State15& invalid,
    const Input15& input, const SolverOptions15& options, Step15 step,
    const Work15& independently_expected) {
    FinalStep15 result;
    result.implementation = "candidate-csr-rejected-raw-input";
    result.one_based_step = 1U;
    result.options = options;
    result.accepted_input = invalid;
    result.value = std::move(step);
    result.value.input_root = input.root;
    result.value.state = {};
    result.value.pi.clear();
    result.value.rho.clear();
    result.value.contact.clear();
    result.graph = {};
    result.value.expected_work = independently_expected;
    result.value.expected_work.portable_fields += step_portable_fields15(
        result.value, result.graph);
    ++result.value.expected_work.hash_derivations;
    result.value.work.portable_fields += step_portable_fields15(
        result.value, result.graph);
    ++result.value.work.hash_derivations;
    result.work = seal_work15(result.value.work, result.value.expected_work);
    result.value.work_root = result.work.actual_root;
    result.value.result_root = step_root15(result.value, result.gate,
        result.graph, result.density_cross, result.work.expected_root,
        result.work.actual_root);
    result.root_closed = root_shape15(input.root)
        && root_shape15(result.value.work_root)
        && root_shape15(result.value.result_root);
    return result;
}

MutationControl15 local_rejection_control15(const Profile15& profile,
    std::string name, std::string variant, State15 invalid,
    std::string expected_outcome) {
    MutationControl15 result;
    result.receipt.name = std::move(name);
    result.receipt.variant = std::move(variant);
    const TermMask15 terms{true, true, false, true};
    const bool nonfinite = !finite_state15(invalid);
    const bool duplicate = !unique_ids15(invalid);
    const bool capacity_case = !nonfinite && !duplicate
        && invalid.dynamic.size() > profile.maximum_neighbors;
    const std::string production_expected = capacity_case
        ? "CAPACITY_OVERFLOW" : "INVALID_INPUT";
    const State15 baseline = tetra_fixture15();
    const Input15 baseline_input = make_input15("A-MUTATION",
        result.receipt.name + "-baseline", profile, baseline, {},
        GravityMode15::Zero, BoundaryMode15::UnboundedManufactured, terms,
        Mutation15::None);
    const Input15 input = make_input15("A-MUTATION", result.receipt.name,
        profile, invalid, {}, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured, terms, Mutation15::None,
        nonfinite ? "MALFORMED_RAW_BINARY32"
                  : "BASE_BINARY32_WIDENED_LONG_DOUBLE");
    result.inputs.push_back(baseline_input);
    result.inputs.push_back(input);
    result.receipt.input_root = input.root;
    const SolverOptions15 options = manufactured_options15(profile, terms);
    result.corrected = run_closed_step15("candidate-csr-baseline",
        candidate_step15, materialize_input15(baseline_input), baseline_input,
        1U, options);
    const State15 materialized_invalid = materialize_input15(input);
    Step15 raw_rejection = candidate_step15(materialized_invalid, options);
    result.production_outcome = raw_rejection.outcome;
    const Work15 independently_expected = capacity_case
        ? expected_candidate_capacity_rejection_work15(
            materialized_invalid, profile)
        : expected_candidate_admission_rejection_work15(materialized_invalid);
    result.production_work_exact = equal_work15(raw_rejection.work,
            independently_expected)
        && equal_work15(raw_rejection.expected_work, independently_expected);
    result.mutated = close_raw_rejection_step15(materialized_invalid,
        input, options, std::move(raw_rejection), independently_expected);
    result.roots_distinct = baseline_input.root != input.root
        && result.corrected->work.actual_root != result.mutated->work.actual_root
        && result.corrected->value.result_root
            != result.mutated->value.result_root;
    result.expected_rejection_observed = result.production_work_exact
        && result.production_outcome == production_expected
        && !result.mutated->value.accepted
        && !result.mutated->transaction_committed && result.roots_distinct;
    result.receipt.typed_outcome = result.expected_rejection_observed
        ? std::move(expected_outcome) : "MUTATION_SURVIVED";
    result.receipt.child_roots.push_back(
        result.corrected->value.result_root);
    result.receipt.child_roots.push_back(result.mutated->value.result_root);
    result.receipt.child_roots.push_back(scalar_observable_root15(
        result.receipt.name, result.receipt.variant, input.root,
        {1.0L, 1.0L, 1.0L},
        {result.production_outcome == production_expected ? 1.0L : 0.0L,
            result.production_work_exact ? 1.0L : 0.0L,
            result.roots_distinct ? 1.0L : 0.0L},
        result.receipt.typed_outcome));
    Work15 actual = actual_input_work15(baseline_input);
    Work15 expected = input_work15(baseline_input);
    add_work15(actual, actual_input_work15(input));
    add_work15(expected, input_work15(input));
    add_step_work15(actual, expected, *result.corrected);
    add_step_work15(actual, expected, *result.mutated);
    add_scalar_root_work15(actual, expected, 3U, 3U);
    result.receipt.apparatus_valid = result.corrected->root_closed
        && result.corrected->work.exact
        && result.corrected->value.apparatus_valid
        && result.mutated->root_closed && result.mutated->work.exact
        && result.production_work_exact;
    result.receipt.pass = result.expected_rejection_observed;
    close_receipt15(result.receipt, actual, expected);
    return result;
}

State15 current_reference_swap_fixture15() {
    return pair_fixture15({0.10L, 0.10L, 0.10L},
        {-5.0L, 0.0L, 0.0L}, {0.20L, 0.10L, 0.10L},
        {5.0L, 0.0L, 0.0L});
}

State15 neighbor_capacity_fixture15() {
    State15 state;
    state.dynamic.reserve(258U);
    for (std::size_t index = 0U; index < 258U; ++index) {
        const long double theta = 2.0L * kPi15
            * static_cast<long double>(index) / 258.0L;
        const long double z = 1.0e-5L * static_cast<long double>(index % 7U);
        const Vec3l position = canonical_vec({0.10L + 0.001L * std::cos(theta),
            0.10L + 0.001L * std::sin(theta), 0.10L + z});
        state.dynamic.push_back({static_cast<std::uint32_t>(50000U + index),
            position, position, {}});
    }
    return state;
}

struct TransactionControl15 {
    RootedReceipt15 receipt;
    Input15 input;
    FinalStep15 private_step;
    std::string prior_state_root;
    std::string prior_pressure_root;
    std::string normal_commit_state_root;
    std::string normal_commit_pressure_root;
    std::string post_rejection_state_root;
    std::string post_rejection_pressure_root;
    std::string verifier_rejection_state_root;
    std::string verifier_rejection_pressure_root;
    Work15 verifier_actual_work;
    Work15 verifier_expected_work;
    std::string verifier_actual_work_root;
    std::string verifier_expected_work_root;
    std::uint64_t private_scratch_records = 0U;
    std::uint64_t normal_post_scratch_records = 0U;
    std::uint64_t forced_post_scratch_records = 0U;
    std::uint64_t verifier_post_scratch_records = 0U;
    bool final_work_verified = false;
    bool normal_post_scratch_empty = false;
    bool forced_post_scratch_empty = false;
    bool verifier_post_scratch_empty = false;
    bool normal_hook_fired = false;
    bool forced_hook_fired = false;
    bool verifier_hook_fired = false;
    bool verifier_failure_observed = false;
    bool normal_commit_exact = false;
    bool rollback_exact = false;
    bool verifier_rollback_exact = false;
};

struct Disposition15 {
    bool hook_fired = false;
    bool forced = false;
    bool final_work_verified = false;
    bool work_mismatch_observed = false;
    State15 published;
    std::vector<long double> published_pi;
    std::uint64_t private_records = 0U;
    std::uint64_t post_records = 0U;
    bool post_empty = false;
};

std::uint64_t scratch_records15(const Step15& step,
    const GraphObservables15& graph) {
    std::uint64_t records = step.state.dynamic.size()
        + step.state.ghosts.size() + step.pi.size() + step.rho.size()
        + step.active_multiplier_ids.size() + step.contact.size()
        + step.kkt_gradient.size() + step.kkt_direction.size()
        + step.dynamic_pressure_force.size()
        + step.ghost_pressure_force.size() + step.viscosity_force.size()
        + step.surface_force.size() + step.trace.backtracks.size()
        + step.trace.outer_inner_offsets.size()
        + step.trace.accepted_objectives.size()
        + step.trace.evaluation_modes.size()
        + step.trace.evaluation_gradient.size()
        + step.trace.reached_stages.size() + graph.density.size()
        + graph.surface.size() + graph.viscosity.size();
    for (const std::vector<Vec3l>& values :
         step.trace.evaluation_positions) records += values.size();
    for (const std::vector<long double>& values :
         step.trace.evaluation_multipliers) records += values.size();
    return records;
}

bool scratch_empty15(const Step15& step, const GraphObservables15& graph,
    const WorkSeal15& work) {
    return scratch_records15(step, graph) == 0U
        && step.input_root.empty() && step.previous_state_root.empty()
        && step.state_root.empty() && step.pressure_root.empty()
        && step.contact_root.empty() && step.work_root.empty()
        && step.result_root.empty() && zero_work15(step.work)
        && zero_work15(step.expected_work) && zero_work15(work.actual)
        && zero_work15(work.expected) && work.actual_root.empty()
        && work.expected_root.empty() && !work.exact;
}

Disposition15 dispose_private_step15(const State15& prior,
    const FinalStep15& private_step, bool force_failure,
    const Work15* verifier_actual_override = nullptr) {
    Disposition15 result;
    result.forced = force_failure;
    Step15 private_scratch = private_step.value;
    GraphObservables15 graph_scratch = private_step.graph;
    WorkSeal15 work_scratch = private_step.work;
    result.private_records = scratch_records15(private_scratch,
        graph_scratch);
    const Work15& verified_actual = verifier_actual_override == nullptr
        ? private_step.work.actual : *verifier_actual_override;
    const bool verifier_exact = equal_work15(verified_actual,
        private_step.work.expected);
    result.work_mismatch_observed = !verifier_exact;
    result.final_work_verified = private_step.root_closed
        && verifier_exact && private_step.private_commit_ready;
    result.hook_fired = force_failure && result.final_work_verified;
    const bool commit = result.final_work_verified && !force_failure;
    result.published = commit ? private_step.value.state : prior;
    result.published_pi = commit ? private_step.value.pi
        : std::vector<long double>(prior.dynamic.size(), 0.0L);
    private_scratch = Step15{};
    graph_scratch = GraphObservables15{};
    work_scratch = WorkSeal15{};
    result.post_records = scratch_records15(private_scratch, graph_scratch);
    result.post_empty = scratch_empty15(private_scratch, graph_scratch,
        work_scratch);
    return result;
}

std::string transaction_observable_root15(std::string_view input_root,
    const TransactionControl15& control, Work15* work = nullptr) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.term-oracle.v1");
    put_string(bytes, "post-private-seal-rollback");
    put_string(bytes, "normal-and-forced-disposition");
    put_string(bytes, input_root);
    put_string(bytes, control.prior_state_root);
    put_string(bytes, control.prior_pressure_root);
    put_string(bytes, control.private_step.value.result_root);
    put_string(bytes, control.private_step.value.state_root);
    put_string(bytes, control.private_step.value.pressure_root);
    put_string(bytes, control.private_step.value.contact_root);
    put_string(bytes, control.normal_commit_state_root);
    put_string(bytes, control.normal_commit_pressure_root);
    put_string(bytes, control.post_rejection_state_root);
    put_string(bytes, control.post_rejection_pressure_root);
    put_string(bytes, control.verifier_rejection_state_root);
    put_string(bytes, control.verifier_rejection_pressure_root);
    put_string(bytes, control.verifier_actual_work_root);
    put_string(bytes, control.verifier_expected_work_root);
    const State15& private_state = control.private_step.value.state;
    const auto order = stable_order15(private_state);
    put_u64(bytes, order.size());
    for (std::size_t index : order) {
        put_u32(bytes, private_state.dynamic[index].id);
        put_vec(bytes, private_state.dynamic[index].position);
        put_vec(bytes, private_state.dynamic[index].velocity);
        put_f64(bytes, index < control.private_step.value.pi.size()
                ? control.private_step.value.pi[index] : 0.0L);
        put_f64(bytes, index < control.private_step.value.rho.size()
                ? control.private_step.value.rho[index] : 0.0L);
    }
    put_u64(bytes, control.normal_post_scratch_records);
    put_u64(bytes, control.forced_post_scratch_records);
    put_u64(bytes, control.verifier_post_scratch_records);
    append_bool(bytes, control.final_work_verified);
    append_bool(bytes, control.normal_post_scratch_empty);
    append_bool(bytes, control.forced_post_scratch_empty);
    append_bool(bytes, control.verifier_post_scratch_empty);
    append_bool(bytes, control.normal_hook_fired);
    append_bool(bytes, control.forced_hook_fired);
    append_bool(bytes, control.verifier_hook_fired);
    append_bool(bytes, control.verifier_failure_observed);
    append_bool(bytes, control.normal_commit_exact);
    append_bool(bytes, control.rollback_exact);
    append_bool(bytes, control.verifier_rollback_exact);
    put_string(bytes, control.rollback_exact && control.normal_commit_exact
            && control.verifier_rollback_exact
            ? "PASS" : "TRANSACTION_INVALID");
    if (work != nullptr) {
        work->portable_fields += 34U + 9U * order.size();
        ++work->hash_derivations;
    }
    return root15(std::move(bytes));
}

TransactionControl15 transaction_control15(const Profile15& profile) {
    TransactionControl15 result;
    result.receipt.name = "post-private-seal-rollback";
    result.receipt.variant = "forced-failure-before-commit";
    const TermMask15 terms{true, true, false, true};
    const State15 base = tetra_fixture15();
    const Input15 input = make_input15("A-TRANSACTION",
        result.receipt.name, profile, base, {}, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured, terms, Mutation15::None);
    result.input = input;
    result.receipt.input_root = input.root;
    const State15 accepted = materialize_input15(input);
    const SolverOptions15 options = manufactured_options15(profile, terms);
    Work15 transaction_root_work;
    result.prior_state_root = state_root15(input.root, 0U, accepted,
        &transaction_root_work);
    const std::vector<long double> prior_pi(accepted.dynamic.size(), 0.0L);
    result.prior_pressure_root = pressure_root15(result.prior_state_root,
        accepted, prior_pi, profile, &transaction_root_work);
    result.private_step = run_private_step15("candidate-csr-private",
        candidate_step15, accepted, input, 1U, options);
    const Disposition15 normal = dispose_private_step15(accepted,
        result.private_step, false);
    const Disposition15 forced = dispose_private_step15(accepted,
        result.private_step, true);
    result.verifier_expected_work = result.private_step.work.expected;
    result.verifier_actual_work = result.private_step.work.actual;
    ++result.verifier_actual_work.scalar_predicates;
    result.verifier_expected_work_root = work_root15(
        result.verifier_expected_work);
    result.verifier_actual_work_root = work_root15(
        result.verifier_actual_work);
    const Disposition15 verifier_rejected = dispose_private_step15(accepted,
        result.private_step, false, &result.verifier_actual_work);
    result.private_scratch_records = normal.private_records;
    result.normal_post_scratch_records = normal.post_records;
    result.forced_post_scratch_records = forced.post_records;
    result.verifier_post_scratch_records = verifier_rejected.post_records;
    result.final_work_verified = normal.final_work_verified
        && forced.final_work_verified;
    result.normal_post_scratch_empty = normal.post_empty;
    result.forced_post_scratch_empty = forced.post_empty;
    result.verifier_post_scratch_empty = verifier_rejected.post_empty;
    result.normal_hook_fired = normal.hook_fired;
    result.forced_hook_fired = forced.hook_fired;
    result.verifier_hook_fired = verifier_rejected.hook_fired;
    result.verifier_failure_observed =
        verifier_rejected.work_mismatch_observed
        && !verifier_rejected.final_work_verified;
    result.normal_commit_state_root = state_root15(input.root, 1U,
        normal.published, &transaction_root_work);
    result.normal_commit_pressure_root = pressure_root15(
        result.normal_commit_state_root, normal.published,
        normal.published_pi, profile, &transaction_root_work);
    result.post_rejection_state_root = state_root15(input.root, 0U,
        forced.published, &transaction_root_work);
    result.post_rejection_pressure_root = pressure_root15(
        result.post_rejection_state_root, forced.published,
        forced.published_pi, profile, &transaction_root_work);
    result.verifier_rejection_state_root = state_root15(input.root, 0U,
        verifier_rejected.published, &transaction_root_work);
    result.verifier_rejection_pressure_root = pressure_root15(
        result.verifier_rejection_state_root, verifier_rejected.published,
        verifier_rejected.published_pi, profile, &transaction_root_work);
    Work15 transaction_comparison_work;
    const bool final_verified = counted_predicate_main15(
        transaction_comparison_work, result.final_work_verified);
    const bool normal_hook = counted_predicate_main15(
        transaction_comparison_work, !result.normal_hook_fired);
    const bool normal_state = counted_predicate_main15(
        transaction_comparison_work, result.normal_commit_state_root
            == result.private_step.value.state_root);
    const bool normal_pressure = counted_predicate_main15(
        transaction_comparison_work, result.normal_commit_pressure_root
            == result.private_step.value.pressure_root);
    const bool normal_records = counted_predicate_main15(
        transaction_comparison_work,
        result.normal_post_scratch_records == 0U);
    const bool normal_empty = counted_predicate_main15(
        transaction_comparison_work, result.normal_post_scratch_empty);
    result.normal_commit_exact = final_verified && normal_hook
        && normal_state && normal_pressure && normal_records && normal_empty;
    const bool forced_hook = counted_predicate_main15(
        transaction_comparison_work, result.forced_hook_fired);
    const bool forced_state = counted_predicate_main15(
        transaction_comparison_work,
        result.prior_state_root == result.post_rejection_state_root);
    const bool forced_pressure = counted_predicate_main15(
        transaction_comparison_work,
        result.prior_pressure_root == result.post_rejection_pressure_root);
    const bool forced_records = counted_predicate_main15(
        transaction_comparison_work,
        result.forced_post_scratch_records == 0U);
    const bool forced_empty = counted_predicate_main15(
        transaction_comparison_work, result.forced_post_scratch_empty);
    const bool private_nonempty = counted_predicate_main15(
        transaction_comparison_work,
        result.private_scratch_records > accepted.dynamic.size());
    result.rollback_exact = final_verified && forced_hook && forced_state
        && forced_pressure && forced_records && forced_empty
        && private_nonempty;
    const bool verifier_failure = counted_predicate_main15(
        transaction_comparison_work, result.verifier_failure_observed);
    const bool verifier_hook = counted_predicate_main15(
        transaction_comparison_work, !result.verifier_hook_fired);
    const bool verifier_state = counted_predicate_main15(
        transaction_comparison_work, result.prior_state_root
            == result.verifier_rejection_state_root);
    const bool verifier_pressure = counted_predicate_main15(
        transaction_comparison_work, result.prior_pressure_root
            == result.verifier_rejection_pressure_root);
    const bool verifier_records = counted_predicate_main15(
        transaction_comparison_work,
        result.verifier_post_scratch_records == 0U);
    const bool verifier_empty = counted_predicate_main15(
        transaction_comparison_work, result.verifier_post_scratch_empty);
    result.verifier_rollback_exact = verifier_failure && verifier_hook
        && verifier_state && verifier_pressure && verifier_records
        && verifier_empty;
    const std::string transaction_root = transaction_observable_root15(
        input.root, result, &transaction_root_work);
    result.receipt.child_roots = {result.private_step.value.result_root,
        result.prior_state_root, result.prior_pressure_root,
        result.post_rejection_state_root, result.post_rejection_pressure_root,
        result.normal_commit_state_root, result.normal_commit_pressure_root,
        result.verifier_rejection_state_root,
        result.verifier_rejection_pressure_root,
        result.verifier_actual_work_root,
        result.verifier_expected_work_root,
        transaction_root};
    Work15 actual = actual_input_work15(input);
    Work15 expected = input_work15(input);
    add_step_work15(actual, expected, result.private_step);
    add_work15(actual, transaction_root_work);
    add_work15(actual, transaction_comparison_work);
    const std::uint64_t state_fields = 4U + 10U * accepted.dynamic.size();
    const std::uint64_t pressure_fields = 3U
        + 4U * accepted.dynamic.size();
    expected.portable_fields += 4U * state_fields
        + 4U * pressure_fields + 34U
        + 9U * accepted.dynamic.size();
    expected.hash_derivations += 9U;
    expected.scalar_predicates += 18U;
    result.receipt.apparatus_valid = result.private_step.root_closed
        && result.private_step.work.exact;
    result.receipt.pass = result.rollback_exact && result.normal_commit_exact
        && result.verifier_rollback_exact;
    result.receipt.typed_outcome = result.receipt.pass
        ? "EXPECTED_FORCED_REJECTION_ROLLED_BACK"
        : "TRANSACTION_INVALID";
    close_receipt15(result.receipt, actual, expected);
    return result;
}

std::string semantic_step_signature15(const FinalStep15& closed,
    std::string_view canonical_input_root) {
    Step15 canonical = closed.value;
    const auto order = stable_order15(canonical.state);
    const auto reorder_vec = [&](const std::vector<Vec3l>& values) {
        std::vector<Vec3l> result;
        result.reserve(order.size());
        for (std::size_t index : order) {
            result.push_back(index < values.size() ? values[index] : Vec3l{});
        }
        return result;
    };
    const auto reorder_scalar = [&](const std::vector<long double>& values) {
        std::vector<long double> result;
        result.reserve(order.size());
        for (std::size_t index : order) {
            result.push_back(index < values.size() ? values[index] : 0.0L);
        }
        return result;
    };
    std::vector<Dynamic15> dynamic;
    dynamic.reserve(order.size());
    for (std::size_t index : order) {
        dynamic.push_back(canonical.state.dynamic[index]);
    }
    canonical.state.dynamic = std::move(dynamic);
    std::sort(canonical.state.ghosts.begin(), canonical.state.ghosts.end(),
        [](const Ghost15& lhs, const Ghost15& rhs) {
            return lhs.id < rhs.id;
        });
    canonical.pi = reorder_scalar(canonical.pi);
    canonical.rho = reorder_scalar(canonical.rho);
    canonical.kkt_gradient = reorder_vec(canonical.kkt_gradient);
    canonical.kkt_direction = reorder_vec(canonical.kkt_direction);
    canonical.dynamic_pressure_force = reorder_vec(
        canonical.dynamic_pressure_force);
    canonical.ghost_pressure_force = reorder_vec(
        canonical.ghost_pressure_force);
    canonical.viscosity_force = reorder_vec(canonical.viscosity_force);
    canonical.surface_force = reorder_vec(canonical.surface_force);
    for (std::vector<Vec3l>& positions :
         canonical.trace.evaluation_positions) {
        positions = reorder_vec(positions);
    }
    for (std::vector<long double>& multipliers :
         canonical.trace.evaluation_multipliers) {
        multipliers = reorder_scalar(multipliers);
    }
    std::sort(canonical.contact.begin(), canonical.contact.end(),
        [](const Contact15& lhs, const Contact15& rhs) {
            return lhs.id < rhs.id;
        });
    std::sort(canonical.active_multiplier_ids.begin(),
        canonical.active_multiplier_ids.end());
    State15 accepted = closed.accepted_input;
    std::sort(accepted.dynamic.begin(), accepted.dynamic.end(),
        [](const Dynamic15& lhs, const Dynamic15& rhs) {
            return lhs.id < rhs.id;
        });
    std::sort(accepted.ghosts.begin(), accepted.ghosts.end(),
        [](const Ghost15& lhs, const Ghost15& rhs) {
            return lhs.id < rhs.id;
        });
    canonical.input_root = std::string(canonical_input_root);
    canonical.previous_state_root = state_root15(canonical_input_root,
        closed.one_based_step - 1U, accepted);
    canonical.state_root = state_root15(canonical_input_root,
        closed.one_based_step, canonical.state);
    canonical.pressure_root = pressure_root15(canonical.state_root,
        canonical.state, canonical.pi, closed.options.profile);
    canonical.contact_root = contact_root15(canonical.state_root,
        canonical.contact);
    return step_root15(canonical, closed.gate, closed.graph,
        closed.density_cross,
        closed.work.expected_root, closed.work.actual_root);
}

struct MaskReceipt15 {
    RootedReceipt15 receipt;
    Input15 canonical_input;
    Input15 permuted_input;
    FinalStep15 candidate;
    FinalStep15 oracle;
    FinalStep15 permuted_candidate;
    FinalStep15 permuted_oracle;
    Comparison15 candidate_oracle;
    Comparison15 permuted_candidate_oracle;
    Comparison15 permutation;
    Comparison15 oracle_permutation;
    TrajectoryMetrics15 candidate_physics;
    TrajectoryMetrics15 oracle_physics;
    TrajectoryMetrics15 permuted_candidate_physics;
    TrajectoryMetrics15 permuted_oracle_physics;
    Work15 physics_work;
    Work15 expected_physics_work;
    Work15 correspondence_work;
    Work15 expected_correspondence_work;
    std::array<std::string, 4> route_first_physical_gate;
    std::string first_physical_gate;
    std::string observable_root;
    bool semantic_permutation_exact = false;
    bool physical_gate_exact = false;
    bool work_ceiling = false;
    bool physical_rejected = false;
};

bool main_step_apparatus15(const FinalStep15& step) {
    if (step.value.work_ceiling) {
        return step.root_closed && step.work.exact
            && step.value.apparatus_valid
            && finite_step_payload15(step.value) && step.gate.capacity
            && step.gate.trace && step.gate.identity
            && step.gate.nonnegative_multipliers
            && step.gate.density_oracle;
    }
    return step.root_closed && step.work.exact
        && step.value.apparatus_valid && step.gate.finite
        && step.gate.capacity && step.gate.trace && step.gate.identity
        && step.gate.active_signature && step.gate.density_oracle;
}

std::string first_physical_gate15(const FinalStep15& step,
    const TrajectoryMetrics15& metrics) {
    const std::array<std::pair<bool, std::string_view>, 7> step_gates{{
        {step.gate.nonnegative_multipliers, "NONNEGATIVE_MULTIPLIERS"},
        {step.gate.density, "DENSITY"}, {step.gate.kkt, "KKT"},
        {step.gate.complementarity, "COMPLEMENTARITY"},
        {step.gate.multiplier_fixed_point, "MULTIPLIER_FIXED_POINT"},
        {step.gate.containment, "CONTAINMENT"},
        {step.gate.manufactured_active_empty,
            "MANUFACTURED_ACTIVE_SIGNATURE"}}};
    for (const auto& gate : step_gates) {
        if (!gate.first) return std::string(gate.second);
    }
    if (!step.value.accepted) return "SOLVER_ACCEPTANCE";
    return metrics.first_failed_gate;
}

std::string mask_observable_root15(const MaskReceipt15& mask,
    Work15* work = nullptr) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.term-oracle.v1");
    put_string(bytes, mask.receipt.name);
    put_string(bytes, "mask-correspondence-and-physics");
    put_string(bytes, mask.canonical_input.root);
    const auto put_comparison = [&](const Comparison15& value) {
        for (long double scalar : std::array<long double, 5>{
                 value.position_rms, value.position_maximum,
                 value.velocity_rms, value.velocity_maximum,
                 value.density_relative_l2}) {
            put_f64(bytes, scalar);
        }
        for (bool predicate : std::array<bool, 5>{value.active_ids_exact,
                 value.contact_masks_exact, value.category_exact,
                 value.trace_exact, value.pass}) {
            append_bool(bytes, predicate);
        }
    };
    for (const Comparison15* comparison :
         std::array<const Comparison15*, 4>{&mask.candidate_oracle,
             &mask.permuted_candidate_oracle, &mask.permutation,
             &mask.oracle_permutation}) {
        put_comparison(*comparison);
    }
    const auto put_metrics = [&](const TrajectoryMetrics15& value) {
        for (long double scalar : std::array<long double, 10>{
                 value.position_rms, value.position_maximum,
                 value.velocity_rms, value.velocity_maximum,
                 value.momentum_residual, value.energy_excess,
                 value.pressure_force_closure,
                 value.viscosity_force_closure,
                 value.surface_force_closure, value.inset_penetration}) {
            put_f64(bytes, scalar);
        }
        put_u64(bytes, value.components);
        put_u64(bytes, value.satellites);
        put_u64(bytes, value.top_contacts);
        for (std::uint64_t support : value.support) put_u64(bytes, support);
        append_bool(bytes, value.support_every_step);
        append_bool(bytes, value.pass);
        put_string(bytes, value.first_failed_gate);
    };
    for (const TrajectoryMetrics15* metrics :
         std::array<const TrajectoryMetrics15*, 4>{&mask.candidate_physics,
             &mask.oracle_physics, &mask.permuted_candidate_physics,
             &mask.permuted_oracle_physics}) {
        put_metrics(*metrics);
    }
    append_bool(bytes, mask.semantic_permutation_exact);
    append_bool(bytes, mask.physical_gate_exact);
    append_bool(bytes, mask.work_ceiling);
    append_bool(bytes, mask.physical_rejected);
    for (const std::string& gate : mask.route_first_physical_gate) {
        put_string(bytes, gate);
    }
    put_string(bytes, mask.first_physical_gate);
    if (work != nullptr) {
        work->portable_fields += 137U;
        ++work->hash_derivations;
    }
    return root15(std::move(bytes));
}

MaskReceipt15 run_mask15(const Profile15& profile, const State15& tight,
    std::string name, TermMask15 terms) {
    MaskReceipt15 result;
    result.receipt.name = name;
    result.receipt.variant = "candidate-oracle-permuted";
    result.canonical_input = make_input15("B", name, profile, tight, {},
        GravityMode15::Profile, BoundaryMode15::AnalyticBox, terms,
        Mutation15::None);
    State15 permuted = permuted_state15(tight);
    result.permuted_input = make_input15("B", name + "-permuted", profile,
        permuted, {}, GravityMode15::Profile, BoundaryMode15::AnalyticBox,
        terms, Mutation15::None, "PHYSICALLY_PERMUTED_BINARY32");
    result.receipt.input_root = result.canonical_input.root;
    const SolverOptions15 options = confined_options15(profile, terms);
    const State15 canonical_state = materialize_input15(result.canonical_input);
    const State15 permuted_state = materialize_input15(result.permuted_input);
    result.candidate = run_private_step15("candidate-csr", candidate_step15,
        canonical_state, result.canonical_input, 1U, options);
    result.oracle = run_private_step15("oracle-all-pairs", oracle_step15,
        canonical_state, result.canonical_input, 1U, options);
    result.permuted_candidate = run_private_step15("candidate-csr-permuted",
        candidate_step15, permuted_state, result.permuted_input, 1U, options);
    result.permuted_oracle = run_private_step15("oracle-all-pairs-permuted",
        oracle_step15, permuted_state, result.permuted_input, 1U, options);
    result.candidate_oracle = compare_steps15(result.candidate.value,
        result.oracle.value, profile.gates, &result.correspondence_work);
    add_work15(result.expected_correspondence_work,
        expected_comparison_work15(canonical_state.dynamic.size()));
    result.permuted_candidate_oracle = compare_steps15(
        result.permuted_candidate.value, result.permuted_oracle.value,
        profile.gates, &result.correspondence_work);
    add_work15(result.expected_correspondence_work,
        expected_comparison_work15(canonical_state.dynamic.size()));
    result.permutation = compare_steps15(result.candidate.value,
        result.permuted_candidate.value, profile.gates,
        &result.correspondence_work);
    add_work15(result.expected_correspondence_work,
        expected_comparison_work15(canonical_state.dynamic.size()));
    result.oracle_permutation = compare_steps15(result.oracle.value,
        result.permuted_oracle.value, profile.gates,
        &result.correspondence_work);
    add_work15(result.expected_correspondence_work,
        expected_comparison_work15(canonical_state.dynamic.size()));
    const std::string canonical_semantic = semantic_step_signature15(
        result.candidate, result.canonical_input.root);
    const std::string permuted_semantic = semantic_step_signature15(
        result.permuted_candidate, result.canonical_input.root);
    const std::string oracle_semantic = semantic_step_signature15(
        result.oracle, result.canonical_input.root);
    const std::string permuted_oracle_semantic = semantic_step_signature15(
        result.permuted_oracle, result.canonical_input.root);
    const bool candidate_semantic_exact = counted_predicate_main15(
        result.correspondence_work,
        canonical_semantic == permuted_semantic);
    const bool oracle_semantic_exact = counted_predicate_main15(
        result.correspondence_work,
        oracle_semantic == permuted_oracle_semantic);
    result.expected_correspondence_work.scalar_predicates += 2U;
    result.semantic_permutation_exact = candidate_semantic_exact
        && oracle_semantic_exact;
    const std::vector<FinalStep15> candidate_steps{result.candidate};
    const std::vector<FinalStep15> oracle_steps{result.oracle};
    const std::vector<FinalStep15> permuted_candidate_steps{
        result.permuted_candidate};
    const std::vector<FinalStep15> permuted_oracle_steps{
        result.permuted_oracle};
    result.candidate_physics = trajectory_metrics15(canonical_state,
        candidate_steps, options, &result.physics_work);
    result.oracle_physics = trajectory_metrics15(canonical_state,
        oracle_steps, options, &result.physics_work);
    result.permuted_candidate_physics = trajectory_metrics15(permuted_state,
        permuted_candidate_steps, options, &result.physics_work);
    result.permuted_oracle_physics = trajectory_metrics15(permuted_state,
        permuted_oracle_steps, options, &result.physics_work);
    add_work15(result.expected_physics_work,
        expected_trajectory_metrics_work15(canonical_state,
            candidate_steps, options));
    add_work15(result.expected_physics_work,
        expected_trajectory_metrics_work15(canonical_state,
            oracle_steps, options));
    add_work15(result.expected_physics_work,
        expected_trajectory_metrics_work15(permuted_state,
            permuted_candidate_steps, options));
    add_work15(result.expected_physics_work,
        expected_trajectory_metrics_work15(permuted_state,
            permuted_oracle_steps, options));
    result.work_ceiling = result.candidate.value.work_ceiling
        || result.oracle.value.work_ceiling
        || result.permuted_candidate.value.work_ceiling
        || result.permuted_oracle.value.work_ceiling;
    const bool step_apparatus = main_step_apparatus15(result.candidate)
        && main_step_apparatus15(result.oracle)
        && main_step_apparatus15(result.permuted_candidate)
        && main_step_apparatus15(result.permuted_oracle);
    result.route_first_physical_gate = {
        first_physical_gate15(result.candidate, result.candidate_physics),
        first_physical_gate15(result.oracle, result.oracle_physics),
        first_physical_gate15(result.permuted_candidate,
            result.permuted_candidate_physics),
        first_physical_gate15(result.permuted_oracle,
            result.permuted_oracle_physics)};
    result.physical_gate_exact = std::all_of(
        result.route_first_physical_gate.begin() + 1,
        result.route_first_physical_gate.end(), [&](const std::string& gate) {
            return gate == result.route_first_physical_gate.front();
        });
    if (result.physical_gate_exact) {
        result.first_physical_gate = result.route_first_physical_gate.front();
    }
    const bool correspondence_apparatus = result.semantic_permutation_exact
        && result.candidate_oracle.pass
        && result.permuted_candidate_oracle.pass && result.permutation.pass
        && result.oracle_permutation.pass && result.physical_gate_exact;
    result.receipt.apparatus_valid = step_apparatus
        && correspondence_apparatus;
    const bool private_steps_ready = result.candidate.private_commit_ready
        && result.oracle.private_commit_ready
        && result.permuted_candidate.private_commit_ready
        && result.permuted_oracle.private_commit_ready;
    const bool pass = result.receipt.apparatus_valid && !result.work_ceiling
        && private_steps_ready && result.candidate_physics.pass
        && result.oracle_physics.pass && result.permuted_candidate_physics.pass
        && result.permuted_oracle_physics.pass;
    result.candidate.transaction_committed = pass;
    result.oracle.transaction_committed = pass;
    result.permuted_candidate.transaction_committed = pass;
    result.permuted_oracle.transaction_committed = pass;
    result.physical_rejected = result.receipt.apparatus_valid
        && !result.work_ceiling && !pass;
    result.receipt.pass = pass;
    if (!result.receipt.apparatus_valid) {
        result.receipt.typed_outcome = "APPARATUS_INVALID";
    } else if (result.work_ceiling) {
        result.receipt.typed_outcome = "SOLVER_WORK_CEILING";
    } else {
        result.receipt.typed_outcome = pass ? "PASS" : "PHYSICAL_REJECTED";
    }
    Work15 mask_observable_work;
    result.observable_root = mask_observable_root15(result,
        &mask_observable_work);
    result.receipt.child_roots = {result.candidate.value.result_root,
        result.oracle.value.result_root,
        result.permuted_candidate.value.result_root,
        result.permuted_oracle.value.result_root, canonical_semantic,
        permuted_semantic, oracle_semantic, permuted_oracle_semantic,
        result.observable_root};
    Work15 actual = actual_input_work15(result.canonical_input);
    Work15 expected = input_work15(result.canonical_input);
    add_work15(actual, actual_input_work15(result.permuted_input));
    add_work15(expected, input_work15(result.permuted_input));
    add_step_work15(actual, expected, result.candidate);
    add_step_work15(actual, expected, result.oracle);
    add_step_work15(actual, expected, result.permuted_candidate);
    add_step_work15(actual, expected, result.permuted_oracle);
    add_work15(actual, result.physics_work);
    add_work15(expected, result.expected_physics_work);
    add_work15(actual, result.correspondence_work);
    add_work15(expected, result.expected_correspondence_work);
    add_work15(actual, mask_observable_work);
    expected.portable_fields += 137U;
    ++expected.hash_derivations;
    const std::uint64_t semantic_fields =
        semantic_signature_fields15(result.candidate)
        + semantic_signature_fields15(result.permuted_candidate)
        + semantic_signature_fields15(result.oracle)
        + semantic_signature_fields15(result.permuted_oracle);
    const std::uint64_t expected_semantic_fields =
        expected_semantic_signature_fields15(result.candidate)
        + expected_semantic_signature_fields15(result.permuted_candidate)
        + expected_semantic_signature_fields15(result.oracle)
        + expected_semantic_signature_fields15(result.permuted_oracle);
    actual.portable_fields += semantic_fields;
    expected.portable_fields += expected_semantic_fields;
    actual.hash_derivations += 20U;
    expected.hash_derivations += 20U;
    close_receipt15(result.receipt, actual, expected);
    return result;
}

std::string trajectory_root15(std::string_view input_root,
    const std::vector<FinalStep15>& accepted,
    const std::vector<FinalStep15>& rejected, std::string_view final_state_root,
    const TrajectoryMetrics15& metrics, const WorkSeal15& work,
    std::string_view outcome) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.trajectory.v1");
    put_string(bytes, input_root);
    put_u64(bytes, accepted.size());
    for (const FinalStep15& step : accepted) {
        put_string(bytes, step.value.result_root);
    }
    put_u64(bytes, rejected.size());
    for (const FinalStep15& step : rejected) {
        put_string(bytes, step.value.result_root);
    }
    put_string(bytes, final_state_root);
    for (long double value : std::array<long double, 10>{metrics.position_rms,
             metrics.position_maximum, metrics.velocity_rms,
             metrics.velocity_maximum, metrics.momentum_residual,
             metrics.energy_excess, metrics.pressure_force_closure,
             metrics.viscosity_force_closure, metrics.surface_force_closure,
             metrics.inset_penetration}) {
        put_f64(bytes, value);
    }
    put_u64(bytes, metrics.components);
    put_u64(bytes, metrics.satellites);
    put_u64(bytes, metrics.top_contacts);
    for (std::uint64_t support : metrics.support) put_u64(bytes, support);
    append_bool(bytes, metrics.support_every_step);
    put_string(bytes, metrics.first_failed_gate);
    append_bool(bytes, metrics.pass);
    put_string(bytes, work.expected_root);
    put_string(bytes, work.actual_root);
    put_string(bytes, outcome);
    return root15(std::move(bytes));
}

struct TrajectoryPath15 {
    std::string name;
    Input15 input;
    State15 initial;
    State15 final_accepted;
    std::vector<FinalStep15> accepted;
    std::vector<FinalStep15> rejected;
    TrajectoryMetrics15 metrics;
    WorkSeal15 work;
    std::string outcome = "NOT_RUN";
    std::string root;
};

void close_trajectory_path15(TrajectoryPath15& path,
    const SolverOptions15&) {
    Work15 actual = actual_input_work15(path.input);
    Work15 expected = input_work15(path.input);
    for (const FinalStep15& step : path.accepted) {
        add_step_work15(actual, expected, step);
    }
    for (const FinalStep15& step : path.rejected) {
        add_step_work15(actual, expected, step);
    }
    const std::uint64_t trajectory_fields = 29U + path.accepted.size()
        + path.rejected.size();
    actual.portable_fields += trajectory_fields;
    expected.portable_fields += trajectory_fields;
    ++actual.hash_derivations;
    ++expected.hash_derivations;
    path.work = seal_work15(actual, expected);
    std::string final_root;
    if (!path.accepted.empty()) {
        final_root = path.accepted.back().value.state_root;
    } else if (!path.rejected.empty()) {
        final_root = path.rejected.front().value.previous_state_root;
    } else {
        throw std::logic_error("NCGP15 trajectory has no attempted step");
    }
    path.root = trajectory_root15(path.input.root, path.accepted,
        path.rejected, final_root, path.metrics, path.work, path.outcome);
}

struct PhaseC15 {
    RootedReceipt15 receipt;
    TrajectoryPath15 candidate;
    TrajectoryPath15 oracle;
    TrajectoryPath15 permuted;
    std::vector<Comparison15> candidate_oracle;
    std::vector<Comparison15> permutation;
    std::vector<bool> semantic_exact;
    std::vector<std::string> semantic_roots;
    std::vector<TrajectoryMetrics15> candidate_attempt_physics;
    std::vector<TrajectoryMetrics15> oracle_attempt_physics;
    std::vector<TrajectoryMetrics15> permuted_attempt_physics;
    std::vector<bool> attempt_physics_evaluated;
    std::vector<bool> physical_gate_exact;
    Comparison15 candidate_oracle_maxima;
    Comparison15 permutation_maxima;
    Work15 correspondence_work;
    Work15 expected_correspondence_work;
    std::string observable_root;
    bool work_ceiling = false;
    bool physical_rejected = false;
};

Comparison15 comparison_maxima15(const std::vector<Comparison15>& values,
    std::size_t prefix_count) {
    Comparison15 result;
    prefix_count = std::min(prefix_count, values.size());
    if (prefix_count == 0U) return result;
    result.active_ids_exact = true;
    result.contact_masks_exact = true;
    result.category_exact = true;
    result.trace_exact = true;
    result.pass = true;
    for (std::size_t index = 0U; index < prefix_count; ++index) {
        const Comparison15& value = values[index];
        result.position_rms = std::max(result.position_rms,
            value.position_rms);
        result.position_maximum = std::max(result.position_maximum,
            value.position_maximum);
        result.velocity_rms = std::max(result.velocity_rms,
            value.velocity_rms);
        result.velocity_maximum = std::max(result.velocity_maximum,
            value.velocity_maximum);
        result.density_relative_l2 = std::max(result.density_relative_l2,
            value.density_relative_l2);
        result.active_ids_exact = result.active_ids_exact
            && value.active_ids_exact;
        result.contact_masks_exact = result.contact_masks_exact
            && value.contact_masks_exact;
        result.category_exact = result.category_exact && value.category_exact;
        result.trace_exact = result.trace_exact && value.trace_exact;
        result.pass = result.pass && value.pass;
    }
    return result;
}

void append_comparison_payload15(std::string& bytes,
    const Comparison15& value) {
    for (long double scalar : std::array<long double, 5>{value.position_rms,
             value.position_maximum, value.velocity_rms,
             value.velocity_maximum, value.density_relative_l2}) {
        put_f64(bytes, scalar);
    }
    for (bool predicate : std::array<bool, 5>{value.active_ids_exact,
             value.contact_masks_exact, value.category_exact,
             value.trace_exact, value.pass}) {
        append_bool(bytes, predicate);
    }
}

void append_trajectory_metrics_payload15(std::string& bytes,
    const TrajectoryMetrics15& metrics) {
    for (long double value : std::array<long double, 10>{
             metrics.position_rms, metrics.position_maximum,
             metrics.velocity_rms, metrics.velocity_maximum,
             metrics.momentum_residual, metrics.energy_excess,
             metrics.pressure_force_closure,
             metrics.viscosity_force_closure,
             metrics.surface_force_closure, metrics.inset_penetration}) {
        put_f64(bytes, value);
    }
    put_u64(bytes, metrics.components);
    put_u64(bytes, metrics.satellites);
    put_u64(bytes, metrics.top_contacts);
    for (std::uint64_t support : metrics.support) put_u64(bytes, support);
    append_bool(bytes, metrics.support_every_step);
    put_string(bytes, metrics.first_failed_gate);
    append_bool(bytes, metrics.pass);
}

std::uint64_t phase_c_observable_fields15(std::size_t attempts) {
    return 28U + 88U * attempts;
}

std::string phase_c_observable_root15(const PhaseC15& phase,
    Work15* work = nullptr) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.term-oracle.v1");
    put_string(bytes, phase.receipt.name);
    put_string(bytes, "trajectory-correspondence-maxima");
    put_string(bytes, phase.candidate.input.root);
    put_u64(bytes, phase.candidate_oracle.size());
    put_u64(bytes, phase.candidate.accepted.size());
    for (std::size_t index = 0U; index < phase.candidate_oracle.size();
         ++index) {
        append_comparison_payload15(bytes, phase.candidate_oracle[index]);
        append_comparison_payload15(bytes, phase.permutation[index]);
        append_bool(bytes, phase.semantic_exact[index]);
        put_string(bytes, phase.semantic_roots[2U * index]);
        put_string(bytes, phase.semantic_roots[2U * index + 1U]);
        append_trajectory_metrics_payload15(bytes,
            phase.candidate_attempt_physics[index]);
        append_trajectory_metrics_payload15(bytes,
            phase.oracle_attempt_physics[index]);
        append_trajectory_metrics_payload15(bytes,
            phase.permuted_attempt_physics[index]);
        append_bool(bytes, phase.attempt_physics_evaluated[index]);
        append_bool(bytes, phase.physical_gate_exact[index]);
    }
    append_comparison_payload15(bytes, phase.candidate_oracle_maxima);
    append_comparison_payload15(bytes, phase.permutation_maxima);
    append_bool(bytes, phase.work_ceiling);
    append_bool(bytes, phase.physical_rejected);
    if (work != nullptr) {
        work->portable_fields += phase_c_observable_fields15(
            phase.candidate_oracle.size());
        ++work->hash_derivations;
    }
    return root15(std::move(bytes));
}

PhaseC15 run_phase_c15(const Profile15& profile, const State15& tight) {
    PhaseC15 result;
    result.receipt.name = "phase-c-pvs-16";
    result.receipt.variant = "candidate-oracle-permuted-transactional";
    const TermMask15 terms{true, true, false, true};
    const SolverOptions15 options = confined_options15(profile, terms);
    result.candidate.name = "candidate-csr";
    result.oracle.name = "oracle-all-pairs";
    result.permuted.name = "candidate-csr-permuted";
    result.candidate.input = make_input15("C", "PVS-candidate", profile,
        tight, {}, GravityMode15::Profile, BoundaryMode15::AnalyticBox,
        terms, Mutation15::None);
    result.oracle.input = make_input15("C", "PVS-oracle", profile, tight,
        {}, GravityMode15::Profile, BoundaryMode15::AnalyticBox, terms,
        Mutation15::None);
    result.permuted.input = make_input15("C", "PVS-permuted", profile,
        permuted_state15(tight), {}, GravityMode15::Profile,
        BoundaryMode15::AnalyticBox, terms, Mutation15::None,
        "PHYSICALLY_PERMUTED_BINARY32");
    result.receipt.input_root = result.candidate.input.root;
    result.candidate.initial = materialize_input15(result.candidate.input);
    result.oracle.initial = materialize_input15(result.oracle.input);
    result.permuted.initial = materialize_input15(result.permuted.input);
    result.candidate.final_accepted = result.candidate.initial;
    result.oracle.final_accepted = result.oracle.initial;
    result.permuted.final_accepted = result.permuted.initial;
    bool prefix = true;
    for (std::uint32_t one_based_step = 1U;
         one_based_step <= 16U && prefix; ++one_based_step) {
        FinalStep15 candidate = run_private_step15("candidate-csr",
            candidate_step15, result.candidate.final_accepted,
            result.candidate.input, one_based_step, options);
        FinalStep15 oracle = run_private_step15("oracle-all-pairs",
            oracle_step15, result.oracle.final_accepted, result.oracle.input,
            one_based_step, options);
        FinalStep15 permuted = run_private_step15("candidate-csr-permuted",
            candidate_step15, result.permuted.final_accepted,
            result.permuted.input, one_based_step, options);
        const Comparison15 correspondence = compare_steps15(candidate.value,
            oracle.value, profile.gates, &result.correspondence_work);
        add_work15(result.expected_correspondence_work,
            expected_comparison_work15(result.candidate.initial.dynamic.size()));
        const Comparison15 permutation = compare_steps15(candidate.value,
            permuted.value, profile.gates, &result.correspondence_work);
        add_work15(result.expected_correspondence_work,
            expected_comparison_work15(result.candidate.initial.dynamic.size()));
        result.candidate_oracle.push_back(correspondence);
        result.permutation.push_back(permutation);
        const std::string candidate_semantic = semantic_step_signature15(
            candidate, result.candidate.input.root);
        const std::string permuted_semantic = semantic_step_signature15(
            permuted, result.candidate.input.root);
        const bool semantic_exact = counted_predicate_main15(
            result.correspondence_work,
            candidate_semantic == permuted_semantic);
        ++result.expected_correspondence_work.scalar_predicates;
        result.semantic_exact.push_back(semantic_exact);
        result.semantic_roots.push_back(candidate_semantic);
        result.semantic_roots.push_back(permuted_semantic);
        result.correspondence_work.portable_fields +=
            semantic_signature_fields15(candidate)
            + semantic_signature_fields15(permuted);
        result.correspondence_work.hash_derivations += 10U;
        result.expected_correspondence_work.portable_fields +=
            expected_semantic_signature_fields15(candidate)
            + expected_semantic_signature_fields15(permuted);
        result.expected_correspondence_work.hash_derivations += 10U;
        const bool prephysical_ready = candidate.private_commit_ready
            && oracle.private_commit_ready && permuted.private_commit_ready
            && correspondence.pass && permutation.pass && semantic_exact;
        TrajectoryMetrics15 candidate_physics;
        TrajectoryMetrics15 oracle_physics;
        TrajectoryMetrics15 permuted_physics;
        const auto evaluate_prefix = [&](const TrajectoryPath15& path,
                                         const FinalStep15& trial) {
            std::vector<FinalStep15> prefix_steps = path.accepted;
            prefix_steps.push_back(trial);
            Work15 actual_metrics_work;
            const TrajectoryMetrics15 metrics = trajectory_metrics15(
                path.initial, prefix_steps, options, &actual_metrics_work);
            add_work15(result.correspondence_work, actual_metrics_work);
            add_work15(result.expected_correspondence_work,
                expected_trajectory_metrics_work15(path.initial,
                    prefix_steps, options));
            return metrics;
        };
        if (prephysical_ready) {
            candidate_physics = evaluate_prefix(result.candidate, candidate);
            oracle_physics = evaluate_prefix(result.oracle, oracle);
            permuted_physics = evaluate_prefix(result.permuted, permuted);
        } else {
            candidate_physics.first_failed_gate = first_physical_gate15(
                candidate, candidate_physics);
            oracle_physics.first_failed_gate = first_physical_gate15(
                oracle, oracle_physics);
            permuted_physics.first_failed_gate = first_physical_gate15(
                permuted, permuted_physics);
        }
        bool physical_exact = counted_predicate_main15(
            result.correspondence_work,
            candidate_physics.pass == oracle_physics.pass);
        physical_exact = counted_predicate_main15(
            result.correspondence_work,
            candidate_physics.pass == permuted_physics.pass)
            && physical_exact;
        physical_exact = counted_predicate_main15(
            result.correspondence_work,
            candidate_physics.first_failed_gate
                == oracle_physics.first_failed_gate) && physical_exact;
        physical_exact = counted_predicate_main15(
            result.correspondence_work,
            candidate_physics.first_failed_gate
                == permuted_physics.first_failed_gate) && physical_exact;
        result.expected_correspondence_work.scalar_predicates += 4U;
        result.candidate_attempt_physics.push_back(candidate_physics);
        result.oracle_attempt_physics.push_back(oracle_physics);
        result.permuted_attempt_physics.push_back(permuted_physics);
        result.attempt_physics_evaluated.push_back(prephysical_ready);
        result.physical_gate_exact.push_back(physical_exact);
        const bool accept_triplet = prephysical_ready && physical_exact
            && candidate_physics.pass && oracle_physics.pass
            && permuted_physics.pass;
        candidate.transaction_committed = accept_triplet;
        oracle.transaction_committed = accept_triplet;
        permuted.transaction_committed = accept_triplet;
        result.work_ceiling = result.work_ceiling
            || candidate.value.work_ceiling || oracle.value.work_ceiling
            || permuted.value.work_ceiling;
        if (accept_triplet) {
            result.candidate.metrics = candidate_physics;
            result.oracle.metrics = oracle_physics;
            result.permuted.metrics = permuted_physics;
            result.candidate.final_accepted = candidate.value.state;
            result.oracle.final_accepted = oracle.value.state;
            result.permuted.final_accepted = permuted.value.state;
            result.candidate.accepted.push_back(std::move(candidate));
            result.oracle.accepted.push_back(std::move(oracle));
            result.permuted.accepted.push_back(std::move(permuted));
        } else {
            result.candidate.rejected.push_back(std::move(candidate));
            result.oracle.rejected.push_back(std::move(oracle));
            result.permuted.rejected.push_back(std::move(permuted));
            prefix = false;
        }
    }
    result.candidate.outcome = result.candidate.accepted.size() == 16U
        ? "PASS" : (result.work_ceiling ? "SOLVER_WORK_CEILING"
                                         : "REJECTED_PRIVATE_TRIAL");
    result.oracle.outcome = result.candidate.outcome;
    result.permuted.outcome = result.candidate.outcome;
    close_trajectory_path15(result.candidate, options);
    close_trajectory_path15(result.oracle, options);
    close_trajectory_path15(result.permuted, options);
    result.candidate_oracle_maxima = comparison_maxima15(
        result.candidate_oracle, result.candidate.accepted.size());
    result.permutation_maxima = comparison_maxima15(result.permutation,
        result.candidate.accepted.size());
    const auto steps_valid = [](const std::vector<FinalStep15>& steps) {
        return std::all_of(steps.begin(), steps.end(),
            [](const FinalStep15& step) {
                return main_step_apparatus15(step);
            });
    };
    const std::size_t attempts = result.candidate_oracle.size();
    const bool correspondence_apparatus = attempts > 0U
        && result.permutation.size() == attempts
        && result.semantic_exact.size() == attempts
        && result.candidate_attempt_physics.size() == attempts
        && result.oracle_attempt_physics.size() == attempts
        && result.permuted_attempt_physics.size() == attempts
        && result.attempt_physics_evaluated.size() == attempts
        && result.physical_gate_exact.size() == attempts
        && std::all_of(result.candidate_oracle.begin(),
            result.candidate_oracle.end(),
            [](const Comparison15& value) { return value.pass; })
        && std::all_of(result.permutation.begin(), result.permutation.end(),
            [](const Comparison15& value) { return value.pass; })
        && std::all_of(result.semantic_exact.begin(),
            result.semantic_exact.end(), [](bool value) { return value; })
        && std::all_of(result.physical_gate_exact.begin(),
            result.physical_gate_exact.end(), [](bool value) { return value; });
    const bool apparatus = result.candidate.work.exact
        && result.oracle.work.exact && result.permuted.work.exact
        && root_shape15(result.candidate.root) && root_shape15(result.oracle.root)
        && root_shape15(result.permuted.root)
        && steps_valid(result.candidate.accepted)
        && steps_valid(result.oracle.accepted)
        && steps_valid(result.permuted.accepted)
        && steps_valid(result.candidate.rejected)
        && steps_valid(result.oracle.rejected)
        && steps_valid(result.permuted.rejected)
        && result.candidate.accepted.size() + result.candidate.rejected.size()
            == attempts
        && result.oracle.accepted.size() + result.oracle.rejected.size()
            == attempts
        && result.permuted.accepted.size() + result.permuted.rejected.size()
            == attempts
        && correspondence_apparatus;
    const bool pass = apparatus && !result.work_ceiling
        && result.candidate.accepted.size() == 16U
        && result.oracle.accepted.size() == 16U
        && result.permuted.accepted.size() == 16U
        && result.candidate.metrics.pass && result.oracle.metrics.pass
        && result.permuted.metrics.pass;
    result.physical_rejected = apparatus && !result.work_ceiling && !pass;
    result.receipt.apparatus_valid = apparatus;
    result.receipt.pass = pass;
    if (!apparatus) result.receipt.typed_outcome = "APPARATUS_INVALID";
    else if (result.work_ceiling) {
        result.receipt.typed_outcome = "SOLVER_WORK_CEILING";
    } else {
        result.receipt.typed_outcome = pass ? "PASS"
            : "SHORT_TRAJECTORY_REJECTED";
    }
    result.observable_root = phase_c_observable_root15(result,
        &result.correspondence_work);
    result.expected_correspondence_work.portable_fields +=
        phase_c_observable_fields15(result.candidate_oracle.size());
    ++result.expected_correspondence_work.hash_derivations;
    result.receipt.child_roots = {result.candidate.root, result.oracle.root,
        result.permuted.root, result.observable_root};
    result.receipt.child_roots.insert(result.receipt.child_roots.end(),
        result.semantic_roots.begin(), result.semantic_roots.end());
    Work15 actual;
    Work15 expected;
    add_work15(actual, result.candidate.work.actual);
    add_work15(actual, result.oracle.work.actual);
    add_work15(actual, result.permuted.work.actual);
    add_work15(expected, result.candidate.work.expected);
    add_work15(expected, result.oracle.work.expected);
    add_work15(expected, result.permuted.work.expected);
    add_work15(actual, result.correspondence_work);
    add_work15(expected, result.expected_correspondence_work);
    close_receipt15(result.receipt, actual, expected);
    return result;
}

long double kernel_value_main15(long double distance,
    const Profile15& profile) {
    if (!(distance >= 0.0L) || distance > profile.horizon) return 0.0L;
    const long double q = 2.0L * distance / profile.horizon;
    const long double alpha = profile.kernel_scale * 3.0L
        / (2.0L * kPi15 * profile.horizon * profile.horizon
            * profile.horizon);
    if (q < 1.0L) {
        return alpha * (2.0L / 3.0L - q * q + 0.5L * q * q * q);
    }
    const long double tail = 2.0L - q;
    return alpha * tail * tail * tail / 6.0L;
}

struct ObservableControl15 {
    RootedReceipt15 receipt;
    std::vector<Input15> inputs;
    std::vector<long double> analytical;
    std::vector<long double> observed;
    std::vector<std::string> labels;
};

ObservableControl15 retained_parent_formula_control15(const Profile15& profile,
    const State15& tight, std::string parent_fixture_root) {
    ObservableControl15 result;
    RootedReceipt15& receipt = result.receipt;
    receipt.name = "retained-fcr0-fcr1-and-ncgp14";
    receipt.variant = "frozen-parent-identity";
    const TermMask15 terms{true, true, false, true};
    const Input15 input = make_input15("A", receipt.name, profile,
        tetra_fixture15(), {}, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured, terms, Mutation15::None);
    result.inputs.push_back(input);
    receipt.input_root = input.root;
    const long double epsilon = profile.spacing * 0x1p-18L;
    const long double distance = canonical_f32(0.10L);
    const long double central = (kernel_value_main15(distance + epsilon, profile)
            - kernel_value_main15(distance - epsilon, profile))
        / (2.0L * epsilon);
    const long double analytic = -kernel_omega15(distance, profile);
    const long double derivative_error = std::fabs(central - analytic)
        / std::max(std::fabs(analytic), 1.0L);
    const State15 tetra = materialize_input15(input);
    const SolverOptions15 options = manufactured_options15(profile, terms);
    std::vector<Vec3l> production_positions;
    production_positions.reserve(tetra.dynamic.size());
    for (const Dynamic15& item : tetra.dynamic) {
        production_positions.push_back(item.position);
    }
    const std::vector<long double> production_pi(tetra.dynamic.size(), 0.0L);
    Work15 production_work;
    const EnergyGradient15 production = candidate_energy_gradient15(tetra,
        production_positions, production_pi, options,
        MultiplierMode15::AugmentedLagrangian, true, production_work);
    const Work15 expected_production_work =
        candidate_expected_energy_gradient_work15(tetra,
            production_positions, production_pi, options, true);
    const std::uint64_t unique_pairs = tetra.dynamic.size()
        * (tetra.dynamic.size() - 1U) / 2U;
    std::vector<std::vector<Vec3l>> surface_terms(tetra.dynamic.size());
    for (std::size_t lhs = 0U; lhs < tetra.dynamic.size(); ++lhs) {
        for (std::size_t rhs = lhs + 1U; rhs < tetra.dynamic.size(); ++rhs) {
            const Vec3l difference = tetra.dynamic[lhs].position
                - tetra.dynamic[rhs].position;
            const long double r = norm15(difference);
            if (!(r > 0.0L && r < 3.0L * profile.surface_r0)) continue;
            const long double q = r / profile.surface_r0;
            const long double cs = q <= 1.0L ? q * q - 1.0L
                : 1.0L - (q - 2.0L) * (q - 2.0L);
            const Vec3l endpoint = 2.0L * profile.gamma * profile.mass
                * profile.mass * cs * (difference / r);
            surface_terms[lhs].push_back(-endpoint);
            surface_terms[rhs].push_back(endpoint);
        }
    }
    std::vector<Vec3l> surface_force;
    surface_force.reserve(surface_terms.size());
    for (std::vector<Vec3l>& terms_for_particle : surface_terms) {
        surface_force.push_back(
            pairwise_vec_sum15(std::move(terms_for_particle)));
    }
    const long double momentum_closure = force_closure15(surface_force);
    Work15 production_closure_work;
    const long double production_momentum_closure = force_closure15(
        production.surface_force, &production_closure_work);
    const std::uint64_t production_pair_count =
        production_work.surface_pair_tests;
    const bool parent_identity = exact_string(parent_fixture_root,
            kExpectedParentFixture)
        && exact_string(NCGP15_PARENT_TIGHT_FIXTURE_ROOT,
            kExpectedParentFixture)
        && exact_string(NCGP15_PARENT_TIGHT_LANE_ROOT, kExpectedParentLane)
        && root_shape15(NCGP15_PARENT_RESULT_ROOT)
        && exact_string(NCGP15_PARENT_BINARY_ROOT,
            "f924209d34ce75ed679bda043156d0d4f79ec13f266ea7ce4bdd6cb5141cab14")
        && exact_string(NCGP15_PARENT_STDOUT_ROOT,
            "15a92dff1e927f4137c43a607a9b31e30200a76e1ebd49a99127543d7a98bd37")
        && tight.dynamic.size() == 128U && tight.ghosts.size() == 1608U;
    result.analytical = {0.0L, 6.0L, 0.0L, 6.0L, 0.0L};
    result.observed = {derivative_error,
        static_cast<long double>(unique_pairs), momentum_closure,
        static_cast<long double>(production_pair_count),
        production_momentum_closure};
    result.labels = {"kernel_derivative_relative_error",
        "independent_unique_dynamic_pair_count",
        "independent_surface_momentum_closure",
        "candidate_surface_pair_count",
        "candidate_surface_momentum_closure"};
    receipt.pass = derivative_error <= 1.0e-8L && unique_pairs == 6U
        && momentum_closure <= profile.gates.internal_force_closure
        && production.valid && production_pair_count == 6U
        && production_momentum_closure
            <= profile.gates.internal_force_closure
        && parent_identity;
    receipt.apparatus_valid = parent_identity && production.valid
        && equal_work15(production_work, expected_production_work);
    receipt.typed_outcome = receipt.pass ? "PASS"
        : "RETAINED_CONTROL_INVALID";
    receipt.child_roots = {std::move(parent_fixture_root),
        std::string(NCGP15_PARENT_RESULT_ROOT), scalar_observable_root15(
            receipt.name, receipt.variant, input.root,
            result.analytical, result.observed, receipt.typed_outcome)};
    Work15 actual = actual_input_work15(input);
    Work15 expected = input_work15(input);
    actual.finite_difference_evaluations = 2U;
    expected.finite_difference_evaluations = 2U;
    actual.surface_pair_tests = unique_pairs;
    expected.surface_pair_tests = 6U;
    actual.surface_active_pairs = unique_pairs;
    expected.surface_active_pairs = 6U;
    actual.scalar_predicates = 11U;
    expected.scalar_predicates = 11U;
    add_work15(actual, production_work);
    add_work15(expected, expected_production_work);
    add_work15(actual, production_closure_work);
    expected.scalar_predicates += tetra.dynamic.size();
    add_scalar_root_work15(actual, expected, 5U, 5U);
    close_receipt15(receipt, actual, expected);
    return result;
}

TermControl15 zero_coefficient_control15(const Profile15& profile,
    bool surface) {
    TermControl15 result;
    result.receipt.name = surface ? "gamma-zero-comparator"
                                  : "lambda-v-zero-comparator";
    result.receipt.variant = "exact-zero-term-mask";
    State15 base;
    const TermMask15 zero_terms{true, false, false, false};
    TermMask15 active_terms;
    if (surface) {
        base = pair_fixture15({0.0800L, 0.10L, 0.10L}, {},
            {0.1200L, 0.10L, 0.10L}, {});
        active_terms = {true, false, false, true};
    } else {
        base = pair_fixture15({0.10L, 0.10L, 0.10L},
            {-1.0L, 0.0L, 0.0L}, {0.20L, 0.10L, 0.10L},
            {1.0L, 0.0L, 0.0L});
        active_terms = {true, true, false, false};
    }
    const Input15 zero_input = make_input15("A-COMPARATOR",
        result.receipt.name, profile, base, {}, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured, zero_terms, Mutation15::None);
    const Input15 active_input = make_input15("A-COMPARATOR",
        result.receipt.name + "-active", profile, base, {},
        GravityMode15::Zero, BoundaryMode15::UnboundedManufactured,
        active_terms, Mutation15::None);
    result.inputs.push_back(zero_input);
    result.inputs.push_back(active_input);
    result.receipt.input_root = zero_input.root;
    const SolverOptions15 zero_options = manufactured_options15(profile,
        zero_terms);
    const SolverOptions15 active_options = manufactured_options15(profile,
        active_terms);
    const State15 zero_state = materialize_input15(zero_input);
    const State15 active_state = materialize_input15(active_input);
    result.steps.push_back(run_closed_step15("candidate-zero", candidate_step15,
        zero_state, zero_input, 1U, zero_options));
    result.steps.push_back(run_closed_step15("candidate-active",
        candidate_step15, active_state, active_input, 1U, active_options));
    long double zero_change = 0.0L;
    for (std::size_t index = 0U; index < zero_state.dynamic.size(); ++index) {
        const Vec3l ballistic = zero_state.dynamic[index].position
            + profile.dt * zero_state.dynamic[index].velocity;
        zero_change = std::max(zero_change, norm15(
            result.steps[0].value.state.dynamic[index].position - ballistic));
    }
    const long double active_difference = maximum_state_difference15(
        result.steps[0].value.state, result.steps[1].value.state);
    const bool pass = result.steps[0].transaction_committed
        && result.steps[1].transaction_committed
        && zero_change <= profile.gates.tangential_change_absolute
        && active_difference > profile.gates.tangential_change_absolute
        && result.steps[0].value.state_root != result.steps[1].value.state_root
        && zero_input.root != active_input.root;
    result.analytical = {0.0L};
    result.observed = {zero_change, active_difference};
    result.labels = {"zero_term_change", "active_state_difference"};
    for (const FinalStep15& step : result.steps) {
        result.receipt.child_roots.push_back(step.value.result_root);
    }
    result.receipt.child_roots.push_back(scalar_observable_root15(
        result.receipt.name, result.receipt.variant, zero_input.root,
        result.analytical, result.observed, pass ? "PASS" : "REJECTED"));
    Work15 actual = actual_input_work15(zero_input);
    Work15 expected = input_work15(zero_input);
    add_work15(actual, actual_input_work15(active_input));
    add_work15(expected, input_work15(active_input));
    for (const FinalStep15& step : result.steps) {
        add_step_work15(actual, expected, step);
    }
    add_scalar_root_work15(actual, expected, 1U, 2U);
    result.receipt.apparatus_valid = std::all_of(result.steps.begin(),
        result.steps.end(), [](const FinalStep15& step) {
            return main_step_apparatus15(step);
        });
    result.receipt.pass = pass;
    result.receipt.typed_outcome = pass ? "PASS" : "COMPARATOR_REJECTED";
    close_receipt15(result.receipt, actual, expected);
    return result;
}

ObservableControl15 translation_consistency_control15(
    const std::array<TermControl15, 3>& controls) {
    ObservableControl15 result;
    RootedReceipt15& receipt = result.receipt;
    receipt.name = controls[0].receipt.name + "-translation-consistency";
    receipt.variant = "0-0.75-100000";
    receipt.input_root = controls[0].receipt.input_root;
    bool apparatus = true;
    const long double tolerance = controls[0].steps[0]
        .options.profile.gates.pair_position_m;
    bool pass = true;
    const FinalStep15& base = controls[0].steps[0];
    const State15 base_initial = pair_fixture15(
        controls[0].receipt.name.find("repulsive") != std::string::npos
            ? Vec3l{0.0800L, 0.10L, 0.10L}
            : Vec3l{0.0575L, 0.10L, 0.10L},
        {}, controls[0].receipt.name.find("repulsive") != std::string::npos
            ? Vec3l{0.1200L, 0.10L, 0.10L}
            : Vec3l{0.1425L, 0.10L, 0.10L}, {});
    const long double base_separation = norm15(
        base.value.state.dynamic[1].position
            - base.value.state.dynamic[0].position);
    const long double base_displacement = norm15(
        base.value.state.dynamic[0].position
            - base_initial.dynamic[0].position);
    std::vector<long double> observed;
    for (const TermControl15& control : controls) {
        apparatus = apparatus && control.receipt.apparatus_valid;
        pass = pass && control.receipt.pass;
        receipt.child_roots.push_back(control.receipt.result_root);
        const FinalStep15& step = control.steps[0];
        const long double separation = norm15(
            step.value.state.dynamic[1].position
                - step.value.state.dynamic[0].position);
        const long double translation = control.receipt.variant == "base"
            ? 0.0L : (control.receipt.variant == "translation-0.75"
                ? 0.75L : 100000.0L);
        const long double displacement = norm15(
            step.value.state.dynamic[0].position
                - (base_initial.dynamic[0].position
                    + Vec3l{translation, translation, translation}));
        observed.push_back(std::fabs(separation - base_separation));
        observed.push_back(std::fabs(displacement - base_displacement));
        pass = pass
            && observed[observed.size() - 2U]
                <= tolerance
            && observed.back() <= tolerance;
    }
    result.analytical = {tolerance};
    result.observed = observed;
    result.labels = {"translation_position_tolerance_m"};
    receipt.child_roots.push_back(scalar_observable_root15(receipt.name,
        receipt.variant, receipt.input_root,
        result.analytical, result.observed,
        pass ? "PASS" : "REJECTED"));
    Work15 actual;
    Work15 expected;
    add_scalar_root_work15(actual, expected, 1U, observed.size());
    receipt.apparatus_valid = apparatus;
    receipt.pass = pass;
    receipt.typed_outcome = pass ? "PASS" : "REPRESENTATION_REJECTED";
    close_receipt15(receipt, actual, expected);
    return result;
}

struct PhaseA15 {
    std::vector<TermControl15> terms;
    std::vector<EnergyControl15> energy;
    std::vector<MutationControl15> mutations;
    std::vector<ObservableControl15> representation;
    ObservableControl15 retained;
    TransactionControl15 transaction;
    RootedReceipt15 receipt;
    bool work_ceiling = false;
    bool physical_rejected = false;
};

PhaseA15 run_phase_a15(const Profile15& profile, const State15& tight,
    const std::string& parent_fixture_root) {
    PhaseA15 result;
    result.retained = retained_parent_formula_control15(profile, tight,
        parent_fixture_root);
    result.terms.push_back(normal_viscosity_control15(profile));
    result.terms.push_back(tangential_zero_control15(profile));
    std::array<TermControl15, 3> repulsive{
        surface_pair_control15(profile, "surface-repulsive-pair", 0.0800L,
            0.1200L, {}),
        surface_pair_control15(profile, "surface-repulsive-pair", 0.0800L,
            0.1200L, {0.75L, 0.75L, 0.75L}),
        surface_pair_control15(profile, "surface-repulsive-pair", 0.0800L,
            0.1200L, {100000.0L, 100000.0L, 100000.0L})};
    std::array<TermControl15, 3> attractive{
        surface_pair_control15(profile, "surface-attractive-pair", 0.0575L,
            0.1425L, {}),
        surface_pair_control15(profile, "surface-attractive-pair", 0.0575L,
            0.1425L, {0.75L, 0.75L, 0.75L}),
        surface_pair_control15(profile, "surface-attractive-pair", 0.0575L,
            0.1425L, {100000.0L, 100000.0L, 100000.0L})};
    result.representation.push_back(
        translation_consistency_control15(repulsive));
    result.representation.push_back(
        translation_consistency_control15(attractive));
    for (TermControl15& control : repulsive) {
        result.terms.push_back(std::move(control));
    }
    for (TermControl15& control : attractive) {
        result.terms.push_back(std::move(control));
    }
    result.terms.push_back(tetra_control15(profile));
    result.terms.push_back(zero_coefficient_control15(profile, false));
    result.terms.push_back(zero_coefficient_control15(profile, true));
    result.energy.push_back(surface_energy_control15(profile));
    result.energy.push_back(reversible_energy_control15(profile));

    const TermMask15 pressure_terms{true, false, false, false};
    const TermMask15 normal_terms{true, true, false, false};
    const TermMask15 surface_terms{true, false, false, true};
    result.mutations.push_back(solver_mutation_control15(profile,
        "missing-kernel-derivative-2-over-h", tight, pressure_terms,
        GravityMode15::Profile, BoundaryMode15::AnalyticBox,
        Mutation15::MissingKernelChain));
    result.mutations.push_back(solver_mutation_control15(profile,
        "half-normal-viscosity", pair_fixture15({0.10L, 0.10L, 0.10L},
            {-1.0L, 0.0L, 0.0L}, {0.20L, 0.10L, 0.10L},
            {1.0L, 0.0L, 0.0L}), normal_terms, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured,
        Mutation15::HalfNormalViscosity));
    result.mutations.push_back(solver_mutation_control15(profile,
        "wrong-surface-sign", pair_fixture15({0.0800L, 0.10L, 0.10L}, {},
            {0.1200L, 0.10L, 0.10L}, {}), surface_terms,
        GravityMode15::Zero, BoundaryMode15::UnboundedManufactured,
        Mutation15::WrongSurfaceSign));
    result.mutations.push_back(solver_mutation_control15(profile,
        "missing-surface-factor-two", pair_fixture15(
            {0.0575L, 0.10L, 0.10L}, {}, {0.1425L, 0.10L, 0.10L}, {}),
        surface_terms, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured,
        Mutation15::MissingSurfaceFactorTwo));
    result.mutations.push_back(solver_mutation_control15(profile,
        "current-reference-viscosity-graph-swap",
        current_reference_swap_fixture15(), normal_terms, GravityMode15::Zero,
        BoundaryMode15::UnboundedManufactured,
        Mutation15::CurrentReferenceViscosityGraph));
    result.mutations.push_back(solver_mutation_control15(profile,
        "finite-pressure-penalty-substitution", tight, pressure_terms,
        GravityMode15::Profile, BoundaryMode15::AnalyticBox,
        Mutation15::FinitePressurePenalty));
    State15 nonfinite = tetra_fixture15();
    nonfinite.dynamic[0].position.x =
        std::numeric_limits<long double>::quiet_NaN();
    result.mutations.push_back(local_rejection_control15(profile,
        "malformed-nonfinite-state", "nonfinite-position", nonfinite,
        "EXPECTED_NONFINITE_REJECTION"));
    State15 duplicate = tetra_fixture15();
    duplicate.dynamic[1].id = duplicate.dynamic[0].id;
    result.mutations.push_back(local_rejection_control15(profile,
        "permutation-identity-loss", "duplicate-stable-id", duplicate,
        "EXPECTED_IDENTITY_REJECTION"));
    result.mutations.push_back(local_rejection_control15(profile,
        "capacity-overflow", "neighbor-row-overflow",
        neighbor_capacity_fixture15(), "EXPECTED_CAPACITY_REJECTION"));
    result.transaction = transaction_control15(profile);
    Work15 ceiling_work;
    Work15 expected_ceiling_work;
    const auto observe_ceiling = [&](const FinalStep15& step) {
        const bool reached_ceiling = counted_predicate_main15(ceiling_work,
            step.value.work_ceiling);
        ++expected_ceiling_work.scalar_predicates;
        result.work_ceiling = reached_ceiling || result.work_ceiling;
    };
    for (const TermControl15& control : result.terms) {
        for (const FinalStep15& step : control.steps) observe_ceiling(step);
    }
    for (const EnergyControl15& control : result.energy) {
        for (const FinalStep15& step : control.forward) observe_ceiling(step);
        for (const FinalStep15& step : control.reverse) observe_ceiling(step);
    }
    result.receipt.name = "phase-a";
    result.receipt.variant = "ordered-controls-and-mutations";
    result.receipt.input_root = result.retained.receipt.input_root;
    result.receipt.child_roots.push_back(result.retained.receipt.result_root);
    Work15 actual = result.retained.receipt.work.actual;
    Work15 expected = result.retained.receipt.work.expected;
    add_work15(actual, ceiling_work);
    add_work15(expected, expected_ceiling_work);
    bool apparatus = result.retained.receipt.apparatus_valid
        && result.retained.receipt.pass;
    bool physical_pass = true;
    for (const TermControl15& control : result.terms) {
        result.receipt.child_roots.push_back(control.receipt.result_root);
        add_work15(actual, control.receipt.work.actual);
        add_work15(expected, control.receipt.work.expected);
        apparatus = apparatus && control.receipt.apparatus_valid;
        physical_pass = physical_pass && control.receipt.pass;
    }
    for (const ObservableControl15& control : result.representation) {
        result.receipt.child_roots.push_back(control.receipt.result_root);
        add_work15(actual, control.receipt.work.actual);
        add_work15(expected, control.receipt.work.expected);
        apparatus = apparatus && control.receipt.apparatus_valid
            && control.receipt.pass;
    }
    for (const EnergyControl15& control : result.energy) {
        result.receipt.child_roots.push_back(control.receipt.result_root);
        add_work15(actual, control.receipt.work.actual);
        add_work15(expected, control.receipt.work.expected);
        apparatus = apparatus && control.receipt.apparatus_valid;
        physical_pass = physical_pass && control.receipt.pass;
    }
    for (const MutationControl15& control : result.mutations) {
        result.receipt.child_roots.push_back(control.receipt.result_root);
        add_work15(actual, control.receipt.work.actual);
        add_work15(expected, control.receipt.work.expected);
        apparatus = apparatus && control.receipt.apparatus_valid
            && control.receipt.pass;
    }
    result.receipt.child_roots.push_back(result.transaction.receipt.result_root);
    add_work15(actual, result.transaction.receipt.work.actual);
    add_work15(expected, result.transaction.receipt.work.expected);
    apparatus = apparatus && result.transaction.receipt.apparatus_valid
        && result.transaction.receipt.pass;
    result.receipt.child_roots.push_back(scalar_observable_root15(
        "phase-a-work-ceiling", "aggregate", result.receipt.input_root,
        {0.0L}, {result.work_ceiling ? 1.0L : 0.0L},
        result.work_ceiling ? "SOLVER_WORK_CEILING" : "NO_WORK_CEILING"));
    add_scalar_root_work15(actual, expected, 1U, 1U);
    apparatus = apparatus && !result.work_ceiling;
    result.physical_rejected = apparatus && !physical_pass;
    result.receipt.apparatus_valid = apparatus;
    result.receipt.pass = apparatus && physical_pass;
    result.receipt.typed_outcome = result.work_ceiling
        ? "SOLVER_WORK_CEILING"
        : (!apparatus ? "APPARATUS_INVALID"
            : (physical_pass ? "PASS" : "TERM_REJECTED"));
    close_receipt15(result.receipt, actual, expected);
    return result;
}

struct BinaryIdentity15 {
    std::uint64_t bytes = 0U;
    std::string root;
};

BinaryIdentity15 read_self_binary15() {
    std::ifstream input("/proc/self/exe", std::ios::binary);
    if (!input) {
        throw std::runtime_error("NCGP15 cannot open /proc/self/exe");
    }
    std::ostringstream buffer;
    buffer << input.rdbuf();
    if (!input.good() && !input.eof()) {
        throw std::runtime_error("NCGP15 cannot read /proc/self/exe");
    }
    const std::string bytes = buffer.str();
    if (bytes.empty()) {
        throw std::runtime_error("NCGP15 executable is empty");
    }
    return {static_cast<std::uint64_t>(bytes.size()), sha256_hex(bytes)};
}

struct Identity15 {
    Profile15 profile;
    State15 tight;
    BinaryIdentity15 binary;
    std::string profile_root;
    std::string reconstructed_parent_fixture_root;
    WorkSeal15 work;
    bool contract = false;
    bool source = false;
    bool parent = false;
    bool fixture = false;
    bool pass = false;
};

Identity15 make_identity15() {
    Identity15 result;
    result.profile = frozen_profile15();
    result.tight = tight_fixture15(result.profile);
    result.binary = read_self_binary15();
    result.profile_root = profile_root15(result.profile);
    result.reconstructed_parent_fixture_root = parent_fixture_root15(
        result.profile, result.tight);
    const std::optional<std::string> compile_command_material = decode_hex15(
        NCGP15_GENERATED_COMPILE_COMMAND_MATERIAL_HEX);
    result.contract = exact_string(NCGP15_CONTRACT_SHA, kExpectedContractSha)
        && exact_string(NCGP15_CONTRACT_ROOT, kExpectedContractSha);
    result.source = root_shape15(NCGP15_SOURCE_ROOT)
        && git_object_shape15(NCGP15_SOURCE_COMMIT)
        && git_object_shape15(NCGP15_SOURCE_TREE)
        && root_shape15(result.binary.root)
        && !exact_string(NCGP15_COMPILER_FAMILY, "unconfigured")
        && !exact_string(NCGP15_COMPILER_VERSION, "unconfigured")
        && !exact_string(NCGP15_COMPILER_FLAGS, "unconfigured")
        && root_shape15(NCGP15_GENERATED_COMPILE_COMMAND_ROOT)
        && compile_command_material.has_value()
        && !compile_command_material->empty()
        && exact_string(sha256_hex(*compile_command_material),
            NCGP15_GENERATED_COMPILE_COMMAND_ROOT);
    result.parent = exact_string(NCGP15_PARENT_CONTRACT_FILE_SHA,
            "4573690e22e79c999b2cdcd609747d0cb061ee59df475dd9040450c6678fd880")
        && exact_string(NCGP15_PARENT_CONTRACT_ROOT,
            "e6d9cdce3a67818024c68ad2da7f4d2405613b7b27953678530b4a415609779f")
        && exact_string(NCGP15_PARENT_SOURCE_ROOT,
            "a3f980e56dff5ce0e599f74b2048092ebac4d4b2d875f8d1c2a4d918a40ccd16")
        && exact_string(NCGP15_PARENT_SOURCE_COMMIT,
            "d1cfe76c4d2a342e99dc0954ba7e322709af1784")
        && exact_string(NCGP15_PARENT_SOURCE_TREE,
            "bbdf9f83918bd21698396e069907b6c3aa198255")
        && exact_string(NCGP15_PARENT_BINARY_ROOT,
            "f924209d34ce75ed679bda043156d0d4f79ec13f266ea7ce4bdd6cb5141cab14")
        && exact_string(NCGP15_PARENT_STDOUT_ROOT,
            "15a92dff1e927f4137c43a607a9b31e30200a76e1ebd49a99127543d7a98bd37")
        && exact_string(NCGP15_PARENT_RESULT_ROOT,
            "54c89f7a0bd4fd13920db325a2b401591cd8cfca3690fc694440d28b42f54221")
        && exact_string(NCGP15_PARENT_TIGHT_LANE_ROOT, kExpectedParentLane);
    result.fixture = exact_string(result.reconstructed_parent_fixture_root,
            kExpectedParentFixture)
        && exact_string(NCGP15_PARENT_TIGHT_FIXTURE_ROOT,
            kExpectedParentFixture)
        && result.tight.dynamic.size() == 128U
        && result.tight.ghosts.size() == 1608U
        && unique_ids15(result.tight) && finite_state15(result.tight);
    Work15 actual;
    Work15 expected;
    actual.fixture_records = result.tight.dynamic.size();
    expected.fixture_records = 128U;
    actual.ghost_records = result.tight.ghosts.size();
    expected.ghost_records = 1608U;
    actual.portable_fields = 56U + 21U + 5U
        + 10U * result.tight.dynamic.size()
        + 4U * result.tight.ghosts.size();
    expected.portable_fields = 56U + 21U + 5U + 10U * 128U
        + 4U * 1608U;
    actual.hash_derivations = 5U;
    expected.hash_derivations = 5U;
    actual.scalar_predicates = 29U;
    expected.scalar_predicates = 29U;
    result.work = seal_work15(actual, expected);
    result.pass = result.contract && result.source && result.parent
        && result.fixture && result.work.exact;
    return result;
}

struct MaskEnvelope15 {
    std::optional<MaskReceipt15> executed;
    RootedReceipt15 receipt;
};

struct Finalization15 {
    std::string status = "APPARATUS_INCONCLUSIVE";
    std::string first_failure_stage;
    std::string first_failure_cause;
    std::string first_failing_mask;
    bool h15a = false;
    bool h15b = false;
    bool h15c = false;
    bool h15d = false;
    bool cause_not_unique = false;
    WorkSeal15 total_work;
    std::string result_root;
    int exit_code = 2;
};

struct Report15 {
    Identity15 identity;
    std::optional<PhaseA15> phase_a;
    RootedReceipt15 phase_a_receipt;
    std::array<MaskEnvelope15, 4> masks;
    std::optional<PhaseC15> phase_c;
    RootedReceipt15 phase_c_receipt;
    Finalization15 finalization;
};

std::string final_result_root15(const Report15& report,
    const WorkSeal15& total_work) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp15.result.v1");
    for (std::string_view value : std::array<std::string_view, 25>{
             NCGP15_CONTRACT_ROOT, NCGP15_CONTRACT_SHA, NCGP15_SOURCE_COMMIT,
             NCGP15_SOURCE_TREE, NCGP15_SOURCE_ROOT,
             report.identity.binary.root, report.identity.profile_root,
             report.identity.reconstructed_parent_fixture_root,
             NCGP15_COMPILER_FAMILY, NCGP15_COMPILER_VERSION,
             NCGP15_COMPILER_FLAGS, NCGP15_GENERATED_COMPILE_COMMAND_ROOT,
             NCGP15_GENERATED_COMPILE_COMMAND_MATERIAL_HEX,
             NCGP15_PARENT_CONTRACT_FILE_SHA,
             NCGP15_PARENT_CONTRACT_ROOT, NCGP15_PARENT_SOURCE_ROOT,
             NCGP15_PARENT_SOURCE_COMMIT, NCGP15_PARENT_SOURCE_TREE,
             NCGP15_PARENT_BINARY_ROOT, NCGP15_PARENT_STDOUT_ROOT,
             NCGP15_PARENT_RESULT_ROOT, NCGP15_PARENT_TIGHT_FIXTURE_ROOT,
             NCGP15_PARENT_TIGHT_LANE_ROOT,
             kExpectedContractCommit, kExpectedContractTree}) {
        put_string(bytes, value);
    }
    put_string(bytes, report.finalization.status);
    put_string(bytes, report.finalization.first_failure_stage);
    put_string(bytes, report.finalization.first_failure_cause);
    put_string(bytes, report.finalization.first_failing_mask);
    append_bool(bytes, report.finalization.h15a);
    append_bool(bytes, report.finalization.h15b);
    append_bool(bytes, report.finalization.h15c);
    append_bool(bytes, report.finalization.h15d);
    append_bool(bytes, report.finalization.cause_not_unique);
    put_string(bytes, report.phase_a_receipt.result_root);
    for (const MaskEnvelope15& mask : report.masks) {
        put_string(bytes, mask.receipt.result_root);
    }
    put_string(bytes, report.phase_c_receipt.result_root);
    put_string(bytes, total_work.expected_root);
    put_string(bytes, total_work.actual_root);
    return root15(std::move(bytes));
}

std::uint64_t final_result_fields15() {
    return 43U;
}

Report15 run_report15() {
    Report15 report;
    report.identity = make_identity15();
    constexpr std::array<std::string_view, 4> mask_names{"P", "PV", "PS",
        "PVS"};
    for (std::size_t index = 0U; index < report.masks.size(); ++index) {
        report.masks[index].receipt = skipped_receipt15(std::string(mask_names[index]),
            "ordered-mask", "NOT_RUN_BY_PRECEDENCE");
    }
    report.phase_a_receipt = skipped_receipt15("phase-a",
        "ordered-controls-and-mutations", "NOT_RUN_BY_PRECEDENCE");
    report.phase_c_receipt = skipped_receipt15("phase-c-pvs-16",
        "candidate-oracle-permuted-transactional",
        "NOT_RUN_BY_PRECEDENCE");

    if (!report.identity.pass) {
        report.finalization.first_failure_stage = "IDENTITY";
        report.finalization.first_failure_cause = "IDENTITY_INVALID";
    } else {
        report.phase_a = run_phase_a15(report.identity.profile,
            report.identity.tight,
            report.identity.reconstructed_parent_fixture_root);
        report.phase_a_receipt = report.phase_a->receipt;
        if (report.phase_a->work_ceiling) {
            report.finalization.first_failure_stage = "PHASE_A";
            report.finalization.first_failure_cause = "SOLVER_WORK_CEILING";
            report.finalization.cause_not_unique = true;
        } else if (!report.phase_a_receipt.apparatus_valid) {
            report.finalization.first_failure_stage = "PHASE_A";
            report.finalization.first_failure_cause = "APPARATUS_INVALID";
        } else if (!report.phase_a_receipt.pass) {
            report.finalization.first_failure_stage = "PHASE_A";
            report.finalization.first_failure_cause = "TERM_REJECTED";
        }
    }

    bool can_run_next_mask = report.identity.pass && report.phase_a.has_value()
        && report.phase_a_receipt.apparatus_valid
        && report.phase_a_receipt.pass && !report.phase_a->work_ceiling;
    bool earlier_mask_physical_failure = false;
    bool mask_work_ceiling_without_prior_failure = false;
    const std::array<TermMask15, 4> masks{{
        {true, false, false, false}, {true, true, false, false},
        {true, false, false, true}, {true, true, false, true}}};
    for (std::size_t index = 0U;
         index < report.masks.size() && can_run_next_mask;
         ++index) {
        report.masks[index].executed = run_mask15(report.identity.profile,
            report.identity.tight, std::string(mask_names[index]), masks[index]);
        report.masks[index].receipt =
            report.masks[index].executed->receipt;
        const MaskReceipt15& mask = *report.masks[index].executed;
        if (!mask.receipt.apparatus_valid) {
            report.finalization.first_failure_stage = "PHASE_B";
            report.finalization.first_failure_cause = "APPARATUS_INVALID";
            report.finalization.first_failing_mask = mask.receipt.name;
            can_run_next_mask = false;
        } else if (mask.work_ceiling) {
            if (!earlier_mask_physical_failure) {
                report.finalization.first_failure_stage = "PHASE_B";
                report.finalization.first_failure_cause =
                    "SOLVER_WORK_CEILING";
                report.finalization.first_failing_mask = mask.receipt.name;
                mask_work_ceiling_without_prior_failure = true;
                report.finalization.cause_not_unique = true;
            }
            can_run_next_mask = false;
        } else if (!mask.receipt.pass) {
            if (!earlier_mask_physical_failure) {
                report.finalization.first_failure_stage = "PHASE_B";
                report.finalization.first_failure_cause =
                    "PHYSICAL_REJECTED";
                report.finalization.first_failing_mask = mask.receipt.name;
            }
            earlier_mask_physical_failure = true;
        }
    }
    const bool every_mask = std::all_of(report.masks.begin(),
        report.masks.end(), [](const MaskEnvelope15& mask) {
            return mask.executed.has_value() && mask.receipt.pass;
        });
    if (can_run_next_mask && every_mask) {
        report.phase_c = run_phase_c15(report.identity.profile,
            report.identity.tight);
        report.phase_c_receipt = report.phase_c->receipt;
        if (!report.phase_c_receipt.apparatus_valid) {
            report.finalization.first_failure_stage = "PHASE_C";
            report.finalization.first_failure_cause = "APPARATUS_INVALID";
        } else if (report.phase_c->work_ceiling) {
            report.finalization.first_failure_stage = "PHASE_C";
            report.finalization.first_failure_cause = "SOLVER_WORK_CEILING";
            report.finalization.cause_not_unique = true;
        } else if (!report.phase_c_receipt.pass) {
            report.finalization.first_failure_stage = "PHASE_C";
            report.finalization.first_failure_cause =
                "SHORT_TRAJECTORY_REJECTED";
        }
    }

    const bool apparatus = report.identity.pass
        && report.phase_a_receipt.apparatus_valid
        && std::all_of(report.masks.begin(), report.masks.end(),
            [](const MaskEnvelope15& mask) {
                return !mask.executed.has_value()
                    || mask.receipt.apparatus_valid;
            })
        && (!report.phase_c.has_value()
            || report.phase_c_receipt.apparatus_valid);
    const bool phase_a_rejected = report.phase_a.has_value()
        && report.phase_a->physical_rejected;
    const bool phase_b_rejected = std::any_of(report.masks.begin(),
        report.masks.end(), [](const MaskEnvelope15& mask) {
            return mask.executed.has_value()
                && mask.executed->physical_rejected;
        });
    const auto phase_a_term_pass = [&](std::string_view name) {
        if (!report.phase_a.has_value()) return false;
        const auto found = std::find_if(report.phase_a->terms.begin(),
            report.phase_a->terms.end(), [&](const TermControl15& control) {
                return control.receipt.name == name;
            });
        return found != report.phase_a->terms.end()
            && found->receipt.apparatus_valid && found->receipt.pass;
    };
    const bool all_masks_observed = std::all_of(report.masks.begin(),
        report.masks.end(), [](const MaskEnvelope15& mask) {
            return mask.executed.has_value()
                && mask.receipt.apparatus_valid
                && !mask.executed->work_ceiling;
        });
    const auto mask_pass = [&](std::size_t index) {
        return report.masks[index].executed.has_value()
            && report.masks[index].receipt.pass;
    };
    const auto mask_gate = [&](std::size_t index) -> std::string_view {
        return report.masks[index].executed.has_value()
            ? std::string_view(report.masks[index].executed
                  ->first_physical_gate)
            : std::string_view{};
    };
    const auto surface_gate = [](std::string_view gate) {
        return gate == "ENERGY_EXCESS"
            || gate == "SURFACE_FORCE_CLOSURE";
    };
    const auto viscosity_gate = [](std::string_view gate) {
        return gate == "ENERGY_EXCESS"
            || gate == "VISCOSITY_FORCE_CLOSURE";
    };
    if (all_masks_observed && mask_pass(0U) && mask_pass(1U)
            && !mask_pass(2U) && !mask_pass(3U)
            && !mask_gate(2U).empty()
            && mask_gate(2U) == mask_gate(3U)
            && surface_gate(mask_gate(2U))
            && phase_a_term_pass("gamma-zero-comparator")) {
        report.finalization.h15b = true;
    }
    if (all_masks_observed && mask_pass(0U) && !mask_pass(1U)
            && mask_pass(2U) && !mask_pass(3U)
            && !mask_gate(1U).empty()
            && mask_gate(1U) == mask_gate(3U)
            && viscosity_gate(mask_gate(1U))
            && phase_a_term_pass("lambda-v-zero-comparator")
            && phase_a_term_pass("normal-viscosity-pair")) {
        report.finalization.h15c = true;
    }
    const bool solver_work_ceiling = mask_work_ceiling_without_prior_failure
        || (report.phase_c.has_value() && report.phase_c->work_ceiling
            && every_mask);
    const bool phase_c_rejected = report.phase_c.has_value()
        && report.phase_c->physical_rejected;
    const bool supported = every_mask && report.phase_c.has_value()
        && report.phase_c_receipt.pass;
    if (!apparatus) {
        report.finalization.status = "APPARATUS_INCONCLUSIVE";
        report.finalization.exit_code = 2;
    } else if (solver_work_ceiling) {
        report.finalization.status = "SOLVER_WORK_CEILING_INCONCLUSIVE";
        report.finalization.exit_code = 0;
    } else if (phase_a_rejected
            || (phase_b_rejected
                && (report.finalization.h15b
                    || report.finalization.h15c))) {
        report.finalization.status = "UNIFIED_CONSTRAINED_TERM_REFUTED";
        report.finalization.exit_code = 0;
    } else if (phase_b_rejected) {
        report.finalization.status = "UNEXPECTED_RESULT_INCONCLUSIVE";
        report.finalization.cause_not_unique = true;
        report.finalization.exit_code = 0;
    } else if (phase_c_rejected) {
        report.finalization.status =
            "UNIFIED_CONSTRAINED_SHORT_TRAJECTORY_REFUTED";
        report.finalization.exit_code = 0;
    } else if (supported) {
        report.finalization.status = "UNIFIED_CONSTRAINED_SHORT_SUPPORTED";
        report.finalization.h15a = true;
        report.finalization.exit_code = 0;
    } else {
        report.finalization.status = "UNEXPECTED_RESULT_INCONCLUSIVE";
        report.finalization.cause_not_unique = true;
        report.finalization.exit_code = 0;
    }

    Work15 total_actual = report.identity.work.actual;
    Work15 total_expected = report.identity.work.expected;
    add_work15(total_actual, report.phase_a_receipt.work.actual);
    add_work15(total_expected, report.phase_a_receipt.work.expected);
    for (const MaskEnvelope15& mask : report.masks) {
        add_work15(total_actual, mask.receipt.work.actual);
        add_work15(total_expected, mask.receipt.work.expected);
    }
    add_work15(total_actual, report.phase_c_receipt.work.actual);
    add_work15(total_expected, report.phase_c_receipt.work.expected);
    total_actual.portable_fields += final_result_fields15();
    total_expected.portable_fields += final_result_fields15();
    ++total_actual.hash_derivations;
    ++total_expected.hash_derivations;
    report.finalization.total_work = seal_work15(total_actual, total_expected);
    if (!report.finalization.total_work.exact) {
        report.finalization.status = "APPARATUS_INCONCLUSIVE";
        report.finalization.first_failure_stage = "TOTAL_WORK";
        report.finalization.first_failure_cause = "WORK_MISMATCH";
        report.finalization.first_failing_mask.clear();
        report.finalization.h15a = false;
        report.finalization.h15b = false;
        report.finalization.h15c = false;
        report.finalization.h15d = false;
        report.finalization.cause_not_unique = false;
        report.finalization.exit_code = 2;
    }
    report.finalization.result_root = final_result_root15(report,
        report.finalization.total_work);
    return report;
}

void emit_json_string15(std::ostream& output, std::string_view value) {
    output << '"';
    for (unsigned char byte : value) {
        switch (byte) {
        case '"': output << "\\\""; break;
        case '\\': output << "\\\\"; break;
        case '\b': output << "\\b"; break;
        case '\f': output << "\\f"; break;
        case '\n': output << "\\n"; break;
        case '\r': output << "\\r"; break;
        case '\t': output << "\\t"; break;
        default:
            if (byte < 0x20U) {
                const char* digits = "0123456789abcdef";
                output << "\\u00" << digits[(byte >> 4U) & 0x0fU]
                       << digits[byte & 0x0fU];
            } else {
                output << static_cast<char>(byte);
            }
        }
    }
    output << '"';
}

void emit_bool15(std::ostream& output, bool value) {
    output << (value ? "true" : "false");
}

void emit_number15(std::ostream& output, long double value) {
    if (!std::isfinite(value)) {
        output << "null";
    } else {
        output << checked_f64(value);
    }
}

void emit_vec15(std::ostream& output, Vec3l value) {
    output << '[';
    emit_number15(output, value.x);
    output << ',';
    emit_number15(output, value.y);
    output << ',';
    emit_number15(output, value.z);
    output << ']';
}

void emit_work15(std::ostream& output, const Work15& work) {
    constexpr std::array<std::string_view, 24> names{
        "fixture_records", "ghost_records", "graph_builds",
        "graph_candidates", "accepted_pairs", "density_pairs",
        "viscosity_pairs", "surface_pair_tests", "surface_active_pairs",
        "objective_evaluations", "gradient_evaluations",
        "jacobian_products", "projected_directions", "line_search_trials",
        "accepted_inner_iterations", "outer_multiplier_updates",
        "box_plane_tests", "contact_reaction_values", "oracle_pair_tests",
        "finite_difference_evaluations", "topology_edges",
        "scalar_predicates", "portable_fields", "hash_derivations"};
    const auto values = work_values15(work);
    output << '{';
    for (std::size_t index = 0U; index < names.size(); ++index) {
        if (index != 0U) output << ',';
        emit_json_string15(output, names[index]);
        output << ':' << values[index];
    }
    output << '}';
}

void emit_work_seal15(std::ostream& output, const WorkSeal15& seal) {
    output << "{\"exact\":";
    emit_bool15(output, seal.exact);
    output << ",\"expected_root\":";
    emit_json_string15(output, seal.expected_root);
    output << ",\"actual_root\":";
    emit_json_string15(output, seal.actual_root);
    output << ",\"expected\":";
    emit_work15(output, seal.expected);
    output << ",\"actual\":";
    emit_work15(output, seal.actual);
    output << '}';
}

void emit_terms15(std::ostream& output, const TermMask15& terms) {
    output << "{\"density_constraint\":";
    emit_bool15(output, terms.density_constraint);
    output << ",\"normal_viscosity\":";
    emit_bool15(output, terms.normal_viscosity);
    output << ",\"tangential_viscosity\":";
    emit_bool15(output, terms.tangential_viscosity);
    output << ",\"surface\":";
    emit_bool15(output, terms.surface);
    output << '}';
}

void emit_profile15(std::ostream& output, const Profile15& profile,
    std::string_view root) {
    output << "{\"id\":";
    emit_json_string15(output, profile.id);
    output << ",\"root\":";
    emit_json_string15(output, root);
    const auto scalar = [&](std::string_view name, long double value) {
        output << ',';
        emit_json_string15(output, name);
        output << ':';
        emit_number15(output, value);
    };
    scalar("dt", profile.dt);
    scalar("spacing", profile.spacing);
    scalar("horizon", profile.horizon);
    scalar("surface_r0", profile.surface_r0);
    scalar("mass", profile.mass);
    scalar("rho0", profile.rho0);
    scalar("kernel_scale", profile.kernel_scale);
    scalar("beta", profile.beta);
    scalar("lambda_v", profile.lambda_v);
    scalar("mu_v", profile.mu_v);
    scalar("gamma", profile.gamma);
    output << ",\"gravity\":";
    emit_vec15(output, profile.gravity);
    output << ",\"basin\":";
    emit_vec15(output, profile.basin);
    output << ",\"ghost_layers\":" << profile.ghost_layers
           << ",\"maximum_dynamic_samples\":"
           << profile.maximum_dynamic_samples
           << ",\"maximum_neighbors\":" << profile.maximum_neighbors
           << ",\"maximum_outer_updates\":"
           << profile.maximum_outer_updates
           << ",\"maximum_accepted_inner_iterations\":"
           << profile.maximum_accepted_inner_iterations
           << ",\"maximum_line_search_backtracks\":"
           << profile.maximum_line_search_backtracks;
    scalar("armijo", profile.armijo);
    scalar("initial_alpha", profile.initial_alpha);
    scalar("line_search_shrink", profile.line_search_shrink);
    scalar("finite_difference_scale", profile.finite_difference_scale);
    output << ",\"gates\":[";
    const auto gates = gate_values15(profile.gates);
    for (std::size_t index = 0U; index < gates.size(); ++index) {
        if (index != 0U) output << ',';
        emit_number15(output, gates[index]);
    }
    output << ',';
    emit_number15(output, profile.gates.internal_force_closure);
    output << "]}";
}

void emit_dynamic15(std::ostream& output, const Dynamic15& item) {
    output << "{\"id\":" << item.id << ",\"reference\":";
    emit_vec15(output, item.reference);
    output << ",\"position\":";
    emit_vec15(output, item.position);
    output << ",\"velocity\":";
    emit_vec15(output, item.velocity);
    output << '}';
}

void emit_state15(std::ostream& output, const State15& state,
    bool physical_order) {
    output << "{\"dynamic\":[";
    std::vector<std::size_t> order;
    if (physical_order) {
        order.resize(state.dynamic.size());
        std::iota(order.begin(), order.end(), 0U);
    } else {
        order = stable_order15(state);
    }
    for (std::size_t ordered = 0U; ordered < order.size(); ++ordered) {
        if (ordered != 0U) output << ',';
        emit_dynamic15(output, state.dynamic[order[ordered]]);
    }
    output << "],\"ghosts\":[";
    for (std::size_t index = 0U; index < state.ghosts.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"id\":" << state.ghosts[index].id
               << ",\"position\":";
        emit_vec15(output, state.ghosts[index].position);
        output << '}';
    }
    output << "]}";
}

std::uint32_t binary32_bits15(long double value) {
    const float narrowed = static_cast<float>(value);
    std::uint32_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(narrowed),
        "binary32 size mismatch");
    std::memcpy(&bits, &narrowed, sizeof(bits));
    return bits;
}

void emit_vec_binary32_bits15(std::ostream& output, Vec3l value) {
    output << '[' << binary32_bits15(value.x) << ','
           << binary32_bits15(value.y) << ','
           << binary32_bits15(value.z) << ']';
}

void emit_base_binary32_bits15(std::ostream& output, const State15& state) {
    output << "{\"dynamic\":[";
    for (std::size_t index = 0U; index < state.dynamic.size(); ++index) {
        if (index != 0U) output << ',';
        const Dynamic15& item = state.dynamic[index];
        output << "{\"id\":" << item.id << ",\"reference\":";
        emit_vec_binary32_bits15(output, item.reference);
        output << ",\"position\":";
        emit_vec_binary32_bits15(output, item.position);
        output << ",\"velocity\":";
        emit_vec_binary32_bits15(output, item.velocity);
        output << '}';
    }
    output << "],\"ghosts\":[";
    for (std::size_t index = 0U; index < state.ghosts.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"id\":" << state.ghosts[index].id
               << ",\"position\":";
        emit_vec_binary32_bits15(output, state.ghosts[index].position);
        output << '}';
    }
    output << "]}";
}

void emit_input15(std::ostream& output, const Input15& input) {
    output << "{\"phase\":";
    emit_json_string15(output, input.phase);
    output << ",\"lane\":";
    emit_json_string15(output, input.lane);
    output << ",\"profile_root\":";
    emit_json_string15(output, input.profile_root);
    output << ",\"parent_fixture_root\":";
    emit_json_string15(output, input.parent_fixture_root);
    output << ",\"coordinate_tag\":";
    emit_json_string15(output, input.coordinate_tag);
    output << ",\"translation\":";
    emit_vec15(output, input.translation);
    output << ",\"gravity_mode\":";
    emit_json_string15(output, gravity_mode_name15(input.gravity_mode));
    output << ",\"boundary_mode\":";
    emit_json_string15(output, boundary_mode_name15(input.boundary_mode));
    output << ",\"terms\":";
    emit_terms15(output, input.terms);
    output << ",\"mutation\":";
    emit_json_string15(output, mutation_name15(input.mutation));
    output << ",\"base\":";
    emit_state15(output, input.base, true);
    output << ",\"base_binary32_bits\":";
    emit_base_binary32_bits15(output, input.base);
    output << ",\"root\":";
    emit_json_string15(output, input.root);
    output << '}';
}

void emit_graph15(std::ostream& output, const GraphObservables15& graph) {
    output << "{\"density_records\":[";
    for (std::size_t index = 0U; index < graph.density.size(); ++index) {
        if (index != 0U) output << ',';
        const DensityGraphRecord15& record = graph.density[index];
        output << "[" << record.owner << ',' << record.neighbor << ',';
        emit_bool15(output, record.ghost);
        output << ']';
    }
    output << "],\"surface_pairs\":[";
    for (std::size_t index = 0U; index < graph.surface.size(); ++index) {
        if (index != 0U) output << ',';
        output << '[' << graph.surface[index].first << ','
               << graph.surface[index].second << ']';
    }
    output << "],\"viscosity_pairs\":[";
    for (std::size_t index = 0U; index < graph.viscosity.size(); ++index) {
        if (index != 0U) output << ',';
        output << '[' << graph.viscosity[index].first << ','
               << graph.viscosity[index].second << ']';
    }
    output << "],\"maximum_degree\":" << graph.maximum_degree
           << ",\"capacity_overflow\":";
    emit_bool15(output, graph.capacity_overflow);
    output << ",\"invalid\":";
    emit_bool15(output, graph.invalid);
    output << '}';
}

void emit_id_vec_array15(std::ostream& output, const State15& state,
    const std::vector<Vec3l>& values) {
    const std::vector<std::size_t> order = stable_order15(state);
    const std::size_t count = std::min(order.size(), values.size());
    output << '[';
    for (std::size_t ordered = 0U; ordered < count; ++ordered) {
        if (ordered != 0U) output << ',';
        const std::size_t index = order[ordered];
        output << "{\"id\":" << state.dynamic[index].id
               << ",\"value\":";
        emit_vec15(output, values[index]);
        output << '}';
    }
    output << ']';
}

void emit_id_scalar_array15(std::ostream& output, const State15& state,
    const std::vector<long double>& values) {
    const std::vector<std::size_t> order = stable_order15(state);
    const std::size_t count = std::min(order.size(), values.size());
    output << '[';
    for (std::size_t ordered = 0U; ordered < count; ++ordered) {
        if (ordered != 0U) output << ',';
        const std::size_t index = order[ordered];
        output << "{\"id\":" << state.dynamic[index].id
               << ",\"value\":";
        emit_number15(output, values[index]);
        output << '}';
    }
    output << ']';
}

void emit_gate15(std::ostream& output, const StepGate15& gate) {
    output << "{\"finite\":";
    emit_bool15(output, gate.finite);
    output << ",\"nonnegative_multipliers\":";
    emit_bool15(output, gate.nonnegative_multipliers);
    output << ",\"density\":";
    emit_bool15(output, gate.density);
    output << ",\"kkt\":";
    emit_bool15(output, gate.kkt);
    output << ",\"complementarity\":";
    emit_bool15(output, gate.complementarity);
    output << ",\"multiplier_fixed_point\":";
    emit_bool15(output, gate.multiplier_fixed_point);
    output << ",\"active_signature\":";
    emit_bool15(output, gate.active_signature);
    output << ",\"containment\":";
    emit_bool15(output, gate.containment);
    output << ",\"capacity\":";
    emit_bool15(output, gate.capacity);
    output << ",\"trace\":";
    emit_bool15(output, gate.trace);
    output << ",\"identity\":";
    emit_bool15(output, gate.identity);
    output << ",\"density_oracle\":";
    emit_bool15(output, gate.density_oracle);
    output << ",\"manufactured_active_empty\":";
    emit_bool15(output, gate.manufactured_active_empty);
    output << ",\"accepted\":";
    emit_bool15(output, gate.accepted);
    output << '}';
}

void emit_step15(std::ostream& output, const FinalStep15& closed) {
    const Step15& step = closed.value;
    output << "{\"implementation\":";
    emit_json_string15(output, closed.implementation);
    output << ",\"one_based_step\":" << closed.one_based_step
           << ",\"input_root\":";
    emit_json_string15(output, step.input_root);
    output << ",\"previous_state_root\":";
    emit_json_string15(output, step.previous_state_root);
    output << ",\"state_root\":";
    emit_json_string15(output, step.state_root);
    output << ",\"pressure_root\":";
    emit_json_string15(output, step.pressure_root);
    output << ",\"contact_root\":";
    emit_json_string15(output, step.contact_root);
    output << ",\"work_root\":";
    emit_json_string15(output, step.work_root);
    output << ",\"result_root\":";
    emit_json_string15(output, step.result_root);
    output << ",\"typed_outcome\":";
    emit_json_string15(output, step.outcome);
    output << ",\"apparatus_valid\":";
    emit_bool15(output, step.apparatus_valid);
    output << ",\"work_ceiling\":";
    emit_bool15(output, step.work_ceiling);
    output << ",\"private_accepted\":";
    emit_bool15(output, step.accepted);
    output << ",\"transaction_committed\":";
    emit_bool15(output, closed.transaction_committed);
    output << ",\"private_commit_ready\":";
    emit_bool15(output, closed.private_commit_ready);
    output << ",\"root_closed\":";
    emit_bool15(output, closed.root_closed);
    output << ",\"failure_field\":";
    emit_json_string15(output, closed.failure_field);
    output << ",\"failure_index\":" << closed.failure_index;
    output << ",\"failure_class\":";
    emit_json_string15(output, closed.failure_class);
    output << ",\"gate\":";
    emit_gate15(output, closed.gate);
    output << ",\"density_cross\":{\"relative_l2\":";
    emit_number15(output, closed.density_cross.relative_l2);
    output << ",\"constraint_signature_exact\":";
    emit_bool15(output, closed.density_cross.constraint_signature_exact);
    output << ",\"pass\":";
    emit_bool15(output, closed.density_cross.pass);
    output << ",\"work\":";
    emit_work15(output, closed.density_cross.actual_work);
    output << ",\"expected_work\":";
    emit_work15(output, closed.density_cross.expected_work);
    output << '}';
    output << ",\"state\":";
    emit_state15(output, step.state, false);
    output << ",\"pressure\":[";
    const auto order = stable_order15(step.state);
    for (std::size_t ordered = 0U; ordered < order.size(); ++ordered) {
        if (ordered != 0U) output << ',';
        const std::size_t index = order[ordered];
        const long double pi = index < step.pi.size() ? step.pi[index] : 0.0L;
        output << "{\"id\":" << step.state.dynamic[index].id
               << ",\"pi_j\":";
        emit_number15(output, pi);
        output << ",\"pressure_pa\":";
        emit_number15(output, closed.options.profile.rho0
            / closed.options.profile.mass * pi);
        output << ",\"active\":";
        emit_bool15(output, pi > 0.0L);
        output << '}';
    }
    output << "],\"rho\":";
    emit_id_scalar_array15(output, step.state, step.rho);
    output << ",\"contact\":[";
    std::vector<Contact15> contact = step.contact;
    std::sort(contact.begin(), contact.end(), [](const Contact15& lhs,
                                               const Contact15& rhs) {
        return lhs.id < rhs.id;
    });
    for (std::size_t index = 0U; index < contact.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"id\":" << contact[index].id << ",\"mask\":"
               << static_cast<unsigned int>(contact[index].mask)
               << ",\"impulse\":";
        emit_vec15(output, contact[index].impulse);
        output << '}';
    }
    output << "],\"graph\":";
    emit_graph15(output, closed.graph);
    output << ",\"metrics\":{";
    output << "\"objective\":";
    emit_number15(output, step.metrics.objective);
    output << ",\"nonpressure_energy\":";
    emit_number15(output, step.metrics.nonpressure_energy);
    output << ",\"inertia_energy\":";
    emit_number15(output, step.metrics.inertia_energy);
    output << ",\"viscosity_energy\":";
    emit_number15(output, step.metrics.viscosity_energy);
    output << ",\"surface_energy\":";
    emit_number15(output, step.metrics.surface_energy);
    output << ",\"maximum_positive_density_strain\":";
    emit_number15(output, step.metrics.maximum_positive_density_strain);
    output << ",\"rms_positive_density_strain\":";
    emit_number15(output, step.metrics.rms_positive_density_strain);
    output << ",\"kkt_rms_m\":";
    emit_number15(output, step.metrics.kkt_rms_m);
    output << ",\"kkt_maximum_m\":";
    emit_number15(output, step.metrics.kkt_maximum_m);
    output << ",\"complementarity\":";
    emit_number15(output, step.metrics.complementarity);
    output << ",\"multiplier_fixed_point\":";
    emit_number15(output, step.metrics.multiplier_fixed_point);
    output << ",\"active_multiplier_count\":"
           << step.metrics.active_multiplier_count << '}';
    output << ",\"trace\":{\"outer_updates\":"
           << step.trace.outer_updates
           << ",\"accepted_inner_iterations\":"
           << step.trace.accepted_inner_iterations
           << ",\"backtracks\":[";
    for (std::size_t index = 0U; index < step.trace.backtracks.size(); ++index) {
        if (index != 0U) output << ',';
        output << step.trace.backtracks[index];
    }
    output << "],\"accepted_objectives\":[";
    for (std::size_t index = 0U;
         index < step.trace.accepted_objectives.size(); ++index) {
        if (index != 0U) output << ',';
        emit_number15(output, step.trace.accepted_objectives[index]);
    }
    output << "],\"outer_inner_offsets\":[";
    for (std::size_t index = 0U;
         index < step.trace.outer_inner_offsets.size(); ++index) {
        if (index != 0U) output << ',';
        output << step.trace.outer_inner_offsets[index];
    }
    output << "],\"evaluation_positions\":[";
    for (std::size_t evaluation = 0U;
         evaluation < step.trace.evaluation_positions.size(); ++evaluation) {
        if (evaluation != 0U) output << ',';
        emit_id_vec_array15(output, step.state,
            step.trace.evaluation_positions[evaluation]);
    }
    output << "],\"evaluation_multipliers\":[";
    for (std::size_t evaluation = 0U;
         evaluation < step.trace.evaluation_multipliers.size(); ++evaluation) {
        if (evaluation != 0U) output << ',';
        emit_id_scalar_array15(output, step.state,
            step.trace.evaluation_multipliers[evaluation]);
    }
    output << "],\"evaluation_modes\":[";
    for (std::size_t index = 0U;
         index < step.trace.evaluation_modes.size(); ++index) {
        if (index != 0U) output << ',';
        output << static_cast<unsigned int>(
            step.trace.evaluation_modes[index]);
    }
    output << "],\"evaluation_gradient\":[";
    for (std::size_t index = 0U;
         index < step.trace.evaluation_gradient.size(); ++index) {
        if (index != 0U) output << ',';
        output << static_cast<unsigned int>(
            step.trace.evaluation_gradient[index]);
    }
    output << "],\"reached_stages\":[";
    for (std::size_t index = 0U;
         index < step.trace.reached_stages.size(); ++index) {
        if (index != 0U) output << ',';
        emit_json_string15(output, step.trace.reached_stages[index]);
    }
    output << "]},\"active_multiplier_ids\":[";
    for (std::size_t index = 0U;
         index < step.active_multiplier_ids.size(); ++index) {
        if (index != 0U) output << ',';
        output << step.active_multiplier_ids[index];
    }
    output << "],\"kkt_gradient\":";
    emit_id_vec_array15(output, step.state, step.kkt_gradient);
    output << ",\"kkt_direction\":";
    emit_id_vec_array15(output, step.state, step.kkt_direction);
    output << ",\"forces\":{\"dynamic_pressure\":";
    emit_id_vec_array15(output, step.state, step.dynamic_pressure_force);
    output << ",\"ghost_pressure\":";
    emit_id_vec_array15(output, step.state, step.ghost_pressure_force);
    output << ",\"viscosity\":";
    emit_id_vec_array15(output, step.state, step.viscosity_force);
    output << ",\"surface\":";
    emit_id_vec_array15(output, step.state, step.surface_force);
    output << "},\"work\":";
    emit_work_seal15(output, closed.work);
    output << '}';
}

void emit_receipt15(std::ostream& output, const RootedReceipt15& receipt) {
    output << "{\"name\":";
    emit_json_string15(output, receipt.name);
    output << ",\"variant\":";
    emit_json_string15(output, receipt.variant);
    output << ",\"typed_outcome\":";
    emit_json_string15(output, receipt.typed_outcome);
    output << ",\"input_root\":";
    emit_json_string15(output, receipt.input_root);
    output << ",\"child_roots\":[";
    for (std::size_t index = 0U; index < receipt.child_roots.size(); ++index) {
        if (index != 0U) output << ',';
        emit_json_string15(output, receipt.child_roots[index]);
    }
    output << "],\"pass\":";
    emit_bool15(output, receipt.pass);
    output << ",\"apparatus_valid\":";
    emit_bool15(output, receipt.apparatus_valid);
    output << ",\"result_root\":";
    emit_json_string15(output, receipt.result_root);
    output << ",\"work\":";
    emit_work_seal15(output, receipt.work);
    output << '}';
}

void emit_comparison15(std::ostream& output, const Comparison15& value) {
    output << "{\"position_rms\":";
    emit_number15(output, value.position_rms);
    output << ",\"position_maximum\":";
    emit_number15(output, value.position_maximum);
    output << ",\"velocity_rms\":";
    emit_number15(output, value.velocity_rms);
    output << ",\"velocity_maximum\":";
    emit_number15(output, value.velocity_maximum);
    output << ",\"density_relative_l2\":";
    emit_number15(output, value.density_relative_l2);
    output << ",\"active_ids_exact\":";
    emit_bool15(output, value.active_ids_exact);
    output << ",\"contact_masks_exact\":";
    emit_bool15(output, value.contact_masks_exact);
    output << ",\"category_exact\":";
    emit_bool15(output, value.category_exact);
    output << ",\"trace_exact\":";
    emit_bool15(output, value.trace_exact);
    output << ",\"pass\":";
    emit_bool15(output, value.pass);
    output << '}';
}

void emit_trajectory_metrics15(std::ostream& output,
    const TrajectoryMetrics15& metrics) {
    output << "{\"position_rms\":";
    emit_number15(output, metrics.position_rms);
    output << ",\"position_maximum\":";
    emit_number15(output, metrics.position_maximum);
    output << ",\"velocity_rms\":";
    emit_number15(output, metrics.velocity_rms);
    output << ",\"velocity_maximum\":";
    emit_number15(output, metrics.velocity_maximum);
    output << ",\"momentum_residual\":";
    emit_number15(output, metrics.momentum_residual);
    output << ",\"energy_excess\":";
    emit_number15(output, metrics.energy_excess);
    output << ",\"pressure_force_closure\":";
    emit_number15(output, metrics.pressure_force_closure);
    output << ",\"viscosity_force_closure\":";
    emit_number15(output, metrics.viscosity_force_closure);
    output << ",\"surface_force_closure\":";
    emit_number15(output, metrics.surface_force_closure);
    output << ",\"components\":" << metrics.components
           << ",\"satellites\":" << metrics.satellites
           << ",\"inset_penetration\":";
    emit_number15(output, metrics.inset_penetration);
    output << ",\"top_contacts\":" << metrics.top_contacts
           << ",\"support\":[";
    for (std::size_t index = 0U; index < metrics.support.size(); ++index) {
        if (index != 0U) output << ',';
        output << metrics.support[index];
    }
    output << "],\"support_every_step\":";
    emit_bool15(output, metrics.support_every_step);
    output << ",\"first_failed_gate\":";
    emit_json_string15(output, metrics.first_failed_gate);
    output << ",\"pass\":";
    emit_bool15(output, metrics.pass);
    output << '}';
}

void emit_term_control15(std::ostream& output,
    const TermControl15& control) {
    output << "{\"receipt\":";
    emit_receipt15(output, control.receipt);
    output << ",\"inputs\":[";
    for (std::size_t index = 0U; index < control.inputs.size(); ++index) {
        if (index != 0U) output << ',';
        emit_input15(output, control.inputs[index]);
    }
    output << "],\"labels\":[";
    for (std::size_t index = 0U; index < control.labels.size(); ++index) {
        if (index != 0U) output << ',';
        emit_json_string15(output, control.labels[index]);
    }
    output << "],\"analytical\":[";
    for (std::size_t index = 0U; index < control.analytical.size(); ++index) {
        if (index != 0U) output << ',';
        emit_number15(output, control.analytical[index]);
    }
    output << "],\"observed\":[";
    for (std::size_t index = 0U; index < control.observed.size(); ++index) {
        if (index != 0U) output << ',';
        emit_number15(output, control.observed[index]);
    }
    output << "],\"steps\":[";
    for (std::size_t index = 0U; index < control.steps.size(); ++index) {
        if (index != 0U) output << ',';
        emit_step15(output, control.steps[index]);
    }
    output << "],\"semantic_roots\":[";
    for (std::size_t index = 0U;
         index < control.semantic_roots.size(); ++index) {
        if (index != 0U) output << ',';
        emit_json_string15(output, control.semantic_roots[index]);
    }
    output << "],\"candidate_gradient\":";
    if (!control.steps.empty()) {
        emit_id_vec_array15(output, control.steps.front().accepted_input,
            control.candidate_gradient);
    } else {
        output << "[]";
    }
    output << ",\"finite_difference_gradient\":";
    if (!control.steps.empty()) {
        emit_id_vec_array15(output, control.steps.front().accepted_input,
            control.finite_difference_gradient);
    } else {
        output << "[]";
    }
    output << ",\"gradient_root\":";
    emit_json_string15(output, control.gradient_root);
    output << '}';
}

void emit_observable_control15(std::ostream& output,
    const ObservableControl15& control) {
    output << "{\"receipt\":";
    emit_receipt15(output, control.receipt);
    output << ",\"inputs\":[";
    for (std::size_t index = 0U; index < control.inputs.size(); ++index) {
        if (index != 0U) output << ',';
        emit_input15(output, control.inputs[index]);
    }
    output << "],\"labels\":[";
    for (std::size_t index = 0U; index < control.labels.size(); ++index) {
        if (index != 0U) output << ',';
        emit_json_string15(output, control.labels[index]);
    }
    output << "],\"analytical\":[";
    for (std::size_t index = 0U;
         index < control.analytical.size(); ++index) {
        if (index != 0U) output << ',';
        emit_number15(output, control.analytical[index]);
    }
    output << "],\"observed\":[";
    for (std::size_t index = 0U; index < control.observed.size(); ++index) {
        if (index != 0U) output << ',';
        emit_number15(output, control.observed[index]);
    }
    output << "]}";
}

void emit_energy_control15(std::ostream& output,
    const EnergyControl15& control) {
    output << "{\"receipt\":";
    emit_receipt15(output, control.receipt);
    output << ",\"input\":";
    emit_input15(output, control.input);
    output << ",\"maximum_positive_excess\":";
    emit_number15(output, control.maximum_positive_excess);
    output << ",\"maximum_positive_excess_j\":";
    emit_number15(output, control.maximum_positive_excess_j);
    output << ",\"reversible_energy_drift\":";
    emit_number15(output, control.reversible_energy_drift);
    output << ",\"return_position_error\":";
    emit_number15(output, control.return_position_error);
    output << ",\"observable_complete\":";
    emit_bool15(output, control.observable_complete);
    output << ",\"observable_actual_work\":";
    emit_work15(output, control.observable_work);
    output << ",\"observable_expected_work\":";
    emit_work15(output, control.expected_observable_work);
    output << ",\"forward\":[";
    for (std::size_t index = 0U; index < control.forward.size(); ++index) {
        if (index != 0U) output << ',';
        emit_step15(output, control.forward[index]);
    }
    output << "],\"reverse\":[";
    for (std::size_t index = 0U; index < control.reverse.size(); ++index) {
        if (index != 0U) output << ',';
        emit_step15(output, control.reverse[index]);
    }
    output << "]}";
}

void emit_mutation_control15(std::ostream& output,
    const MutationControl15& control) {
    output << "{\"receipt\":";
    emit_receipt15(output, control.receipt);
    output << ",\"inputs\":[";
    for (std::size_t index = 0U; index < control.inputs.size(); ++index) {
        if (index != 0U) output << ',';
        emit_input15(output, control.inputs[index]);
    }
    output << "],\"expected_rejection_observed\":";
    emit_bool15(output, control.expected_rejection_observed);
    output << ",\"state_difference\":";
    emit_number15(output, control.state_difference);
    output << ",\"production_outcome\":";
    emit_json_string15(output, control.production_outcome);
    output << ",\"production_work_exact\":";
    emit_bool15(output, control.production_work_exact);
    output << ",\"roots_distinct\":";
    emit_bool15(output, control.roots_distinct);
    output << ",\"corrected\":";
    if (control.corrected.has_value()) {
        emit_step15(output, *control.corrected);
    } else {
        output << "null";
    }
    output << ",\"oracle\":";
    if (control.oracle.has_value()) {
        emit_step15(output, *control.oracle);
    } else {
        output << "null";
    }
    output << ",\"mutated\":";
    if (control.mutated.has_value()) {
        emit_step15(output, *control.mutated);
    } else {
        output << "null";
    }
    output << ",\"corrected_oracle\":";
    emit_comparison15(output, control.corrected_oracle);
    output << ",\"mutated_oracle\":";
    emit_comparison15(output, control.mutated_oracle);
    output << '}';
}

void emit_transaction15(std::ostream& output,
    const TransactionControl15& control) {
    output << "{\"receipt\":";
    emit_receipt15(output, control.receipt);
    output << ",\"input\":";
    emit_input15(output, control.input);
    output << ",\"prior_state_root\":";
    emit_json_string15(output, control.prior_state_root);
    output << ",\"prior_pressure_root\":";
    emit_json_string15(output, control.prior_pressure_root);
    output << ",\"normal_commit_state_root\":";
    emit_json_string15(output, control.normal_commit_state_root);
    output << ",\"normal_commit_pressure_root\":";
    emit_json_string15(output, control.normal_commit_pressure_root);
    output << ",\"post_rejection_state_root\":";
    emit_json_string15(output, control.post_rejection_state_root);
    output << ",\"post_rejection_pressure_root\":";
    emit_json_string15(output, control.post_rejection_pressure_root);
    output << ",\"verifier_rejection_state_root\":";
    emit_json_string15(output, control.verifier_rejection_state_root);
    output << ",\"verifier_rejection_pressure_root\":";
    emit_json_string15(output, control.verifier_rejection_pressure_root);
    output << ",\"verifier_actual_work_root\":";
    emit_json_string15(output, control.verifier_actual_work_root);
    output << ",\"verifier_expected_work_root\":";
    emit_json_string15(output, control.verifier_expected_work_root);
    output << ",\"verifier_actual_work\":";
    emit_work15(output, control.verifier_actual_work);
    output << ",\"verifier_expected_work\":";
    emit_work15(output, control.verifier_expected_work);
    output << ",\"private_scratch_records\":"
           << control.private_scratch_records
           << ",\"normal_post_scratch_records\":"
           << control.normal_post_scratch_records
           << ",\"forced_post_scratch_records\":"
           << control.forced_post_scratch_records
           << ",\"verifier_post_scratch_records\":"
           << control.verifier_post_scratch_records
           << ",\"final_work_verified\":";
    emit_bool15(output, control.final_work_verified);
    output << ",\"normal_post_scratch_empty\":";
    emit_bool15(output, control.normal_post_scratch_empty);
    output << ",\"forced_post_scratch_empty\":";
    emit_bool15(output, control.forced_post_scratch_empty);
    output << ",\"verifier_post_scratch_empty\":";
    emit_bool15(output, control.verifier_post_scratch_empty);
    output
           << ",\"normal_hook_fired\":";
    emit_bool15(output, control.normal_hook_fired);
    output << ",\"forced_hook_fired\":";
    emit_bool15(output, control.forced_hook_fired);
    output << ",\"verifier_hook_fired\":";
    emit_bool15(output, control.verifier_hook_fired);
    output << ",\"verifier_failure_observed\":";
    emit_bool15(output, control.verifier_failure_observed);
    output << ",\"normal_commit_exact\":";
    emit_bool15(output, control.normal_commit_exact);
    output << ",\"rollback_exact\":";
    emit_bool15(output, control.rollback_exact);
    output << ",\"verifier_rollback_exact\":";
    emit_bool15(output, control.verifier_rollback_exact);
    output << ",\"private_step\":";
    emit_step15(output, control.private_step);
    output << '}';
}

void emit_phase_a15(std::ostream& output, const PhaseA15& phase) {
    output << "{\"receipt\":";
    emit_receipt15(output, phase.receipt);
    output << ",\"work_ceiling\":";
    emit_bool15(output, phase.work_ceiling);
    output << ",\"physical_rejected\":";
    emit_bool15(output, phase.physical_rejected);
    output << ",\"retained\":";
    emit_observable_control15(output, phase.retained);
    output << ",\"term_controls\":[";
    for (std::size_t index = 0U; index < phase.terms.size(); ++index) {
        if (index != 0U) output << ',';
        emit_term_control15(output, phase.terms[index]);
    }
    output << "],\"representation_controls\":[";
    for (std::size_t index = 0U; index < phase.representation.size(); ++index) {
        if (index != 0U) output << ',';
        emit_observable_control15(output, phase.representation[index]);
    }
    output << "],\"energy_controls\":[";
    for (std::size_t index = 0U; index < phase.energy.size(); ++index) {
        if (index != 0U) output << ',';
        emit_energy_control15(output, phase.energy[index]);
    }
    output << "],\"mutations\":[";
    for (std::size_t index = 0U; index < phase.mutations.size(); ++index) {
        if (index != 0U) output << ',';
        emit_mutation_control15(output, phase.mutations[index]);
    }
    output << "],\"transaction\":";
    emit_transaction15(output, phase.transaction);
    output << '}';
}

void emit_mask15(std::ostream& output, const MaskEnvelope15& envelope) {
    if (!envelope.executed.has_value()) {
        output << "{\"receipt\":";
        emit_receipt15(output, envelope.receipt);
        output << ",\"executed\":null}";
        return;
    }
    const MaskReceipt15& mask = *envelope.executed;
    output << "{\"receipt\":";
    emit_receipt15(output, mask.receipt);
    output << ",\"canonical_input\":";
    emit_input15(output, mask.canonical_input);
    output << ",\"permuted_input\":";
    emit_input15(output, mask.permuted_input);
    output << ",\"candidate\":";
    emit_step15(output, mask.candidate);
    output << ",\"oracle\":";
    emit_step15(output, mask.oracle);
    output << ",\"permuted_candidate\":";
    emit_step15(output, mask.permuted_candidate);
    output << ",\"permuted_oracle\":";
    emit_step15(output, mask.permuted_oracle);
    output << ",\"candidate_oracle\":";
    emit_comparison15(output, mask.candidate_oracle);
    output << ",\"permuted_candidate_oracle\":";
    emit_comparison15(output, mask.permuted_candidate_oracle);
    output << ",\"permutation\":";
    emit_comparison15(output, mask.permutation);
    output << ",\"oracle_permutation\":";
    emit_comparison15(output, mask.oracle_permutation);
    output << ",\"semantic_permutation_exact\":";
    emit_bool15(output, mask.semantic_permutation_exact);
    output << ",\"candidate_physics\":";
    emit_trajectory_metrics15(output, mask.candidate_physics);
    output << ",\"oracle_physics\":";
    emit_trajectory_metrics15(output, mask.oracle_physics);
    output << ",\"permuted_candidate_physics\":";
    emit_trajectory_metrics15(output, mask.permuted_candidate_physics);
    output << ",\"permuted_oracle_physics\":";
    emit_trajectory_metrics15(output, mask.permuted_oracle_physics);
    output << ",\"route_first_physical_gate\":[";
    for (std::size_t index = 0U;
         index < mask.route_first_physical_gate.size(); ++index) {
        if (index != 0U) output << ',';
        emit_json_string15(output, mask.route_first_physical_gate[index]);
    }
    output << "],\"first_physical_gate\":";
    emit_json_string15(output, mask.first_physical_gate);
    output << ",\"physical_gate_exact\":";
    emit_bool15(output, mask.physical_gate_exact);
    output << ",\"work_ceiling\":";
    emit_bool15(output, mask.work_ceiling);
    output << ",\"physical_rejected\":";
    emit_bool15(output, mask.physical_rejected);
    output << ",\"observable_root\":";
    emit_json_string15(output, mask.observable_root);
    output << ",\"correspondence_actual_work\":";
    emit_work15(output, mask.correspondence_work);
    output << ",\"correspondence_expected_work\":";
    emit_work15(output, mask.expected_correspondence_work);
    output << '}';
}

void emit_path15(std::ostream& output, const TrajectoryPath15& path) {
    output << "{\"name\":";
    emit_json_string15(output, path.name);
    output << ",\"input\":";
    emit_input15(output, path.input);
    output << ",\"accepted\":[";
    for (std::size_t index = 0U; index < path.accepted.size(); ++index) {
        if (index != 0U) output << ',';
        emit_step15(output, path.accepted[index]);
    }
    output << "],\"rejected\":[";
    for (std::size_t index = 0U; index < path.rejected.size(); ++index) {
        if (index != 0U) output << ',';
        emit_step15(output, path.rejected[index]);
    }
    output << "],\"final_accepted\":";
    emit_state15(output, path.final_accepted, false);
    output << ",\"metrics\":";
    emit_trajectory_metrics15(output, path.metrics);
    output << ",\"outcome\":";
    emit_json_string15(output, path.outcome);
    output << ",\"trajectory_root\":";
    emit_json_string15(output, path.root);
    output << ",\"work\":";
    emit_work_seal15(output, path.work);
    output << '}';
}

void emit_phase_c15(std::ostream& output, const PhaseC15& phase) {
    output << "{\"receipt\":";
    emit_receipt15(output, phase.receipt);
    output << ",\"candidate\":";
    emit_path15(output, phase.candidate);
    output << ",\"oracle\":";
    emit_path15(output, phase.oracle);
    output << ",\"permuted\":";
    emit_path15(output, phase.permuted);
    output << ",\"candidate_oracle\":[";
    for (std::size_t index = 0U; index < phase.candidate_oracle.size(); ++index) {
        if (index != 0U) output << ',';
        emit_comparison15(output, phase.candidate_oracle[index]);
    }
    output << "],\"permutation\":[";
    for (std::size_t index = 0U; index < phase.permutation.size(); ++index) {
        if (index != 0U) output << ',';
        emit_comparison15(output, phase.permutation[index]);
    }
    output << "],\"semantic_exact\":[";
    for (std::size_t index = 0U; index < phase.semantic_exact.size(); ++index) {
        if (index != 0U) output << ',';
        emit_bool15(output, phase.semantic_exact[index]);
    }
    output << "],\"semantic_roots\":[";
    for (std::size_t index = 0U; index < phase.semantic_roots.size(); ++index) {
        if (index != 0U) output << ',';
        emit_json_string15(output, phase.semantic_roots[index]);
    }
    output << "],\"candidate_attempt_physics\":[";
    for (std::size_t index = 0U;
         index < phase.candidate_attempt_physics.size(); ++index) {
        if (index != 0U) output << ',';
        emit_trajectory_metrics15(output,
            phase.candidate_attempt_physics[index]);
    }
    output << "],\"oracle_attempt_physics\":[";
    for (std::size_t index = 0U;
         index < phase.oracle_attempt_physics.size(); ++index) {
        if (index != 0U) output << ',';
        emit_trajectory_metrics15(output, phase.oracle_attempt_physics[index]);
    }
    output << "],\"permuted_attempt_physics\":[";
    for (std::size_t index = 0U;
         index < phase.permuted_attempt_physics.size(); ++index) {
        if (index != 0U) output << ',';
        emit_trajectory_metrics15(output,
            phase.permuted_attempt_physics[index]);
    }
    output << "],\"attempt_physics_evaluated\":[";
    for (std::size_t index = 0U;
         index < phase.attempt_physics_evaluated.size(); ++index) {
        if (index != 0U) output << ',';
        emit_bool15(output, phase.attempt_physics_evaluated[index]);
    }
    output << "],\"physical_gate_exact\":[";
    for (std::size_t index = 0U;
         index < phase.physical_gate_exact.size(); ++index) {
        if (index != 0U) output << ',';
        emit_bool15(output, phase.physical_gate_exact[index]);
    }
    output << "],\"accepted_correspondence_count\":"
           << phase.candidate.accepted.size()
           << ",\"accepted_candidate_oracle_maxima\":";
    emit_comparison15(output, phase.candidate_oracle_maxima);
    output << ",\"accepted_permutation_maxima\":";
    emit_comparison15(output, phase.permutation_maxima);
    output << ",\"observable_root\":";
    emit_json_string15(output, phase.observable_root);
    output << ",\"correspondence_actual_work\":";
    emit_work15(output, phase.correspondence_work);
    output << ",\"correspondence_expected_work\":";
    emit_work15(output, phase.expected_correspondence_work);
    output << ",\"work_ceiling\":";
    emit_bool15(output, phase.work_ceiling);
    output << ",\"physical_rejected\":";
    emit_bool15(output, phase.physical_rejected);
    output << '}';
}

void emit_report15(std::ostream& output, const Report15& report) {
    output << std::setprecision(17) << "{\"schema\":";
    emit_json_string15(output, kSchema);
    output << ",\"status\":";
    emit_json_string15(output, report.finalization.status);
    output << ",\"invocation\":";
    emit_json_string15(output, kInvocation);
    output << ",\"contract_root\":";
    emit_json_string15(output, NCGP15_CONTRACT_ROOT);
    output << ",\"contract_sha\":";
    emit_json_string15(output, NCGP15_CONTRACT_SHA);
    output << ",\"contract_freeze_commit\":";
    emit_json_string15(output, kExpectedContractCommit);
    output << ",\"contract_freeze_tree\":";
    emit_json_string15(output, kExpectedContractTree);
    output << ",\"source_commit\":";
    emit_json_string15(output, NCGP15_SOURCE_COMMIT);
    output << ",\"source_tree\":";
    emit_json_string15(output, NCGP15_SOURCE_TREE);
    output << ",\"source_root\":";
    emit_json_string15(output, NCGP15_SOURCE_ROOT);
    output << ",\"binary_root\":";
    emit_json_string15(output, report.identity.binary.root);
    output << ",\"binary_bytes\":" << report.identity.binary.bytes
           << ",\"compiler_family\":";
    emit_json_string15(output, NCGP15_COMPILER_FAMILY);
    output << ",\"compiler_version\":";
    emit_json_string15(output, NCGP15_COMPILER_VERSION);
    output << ",\"compiler_flags\":";
    emit_json_string15(output, NCGP15_COMPILER_FLAGS);
    output << ",\"generated_compile_command_root\":";
    emit_json_string15(output, NCGP15_GENERATED_COMPILE_COMMAND_ROOT);
    output << ",\"generated_compile_command_material_hex\":";
    emit_json_string15(output,
        NCGP15_GENERATED_COMPILE_COMMAND_MATERIAL_HEX);
    output << ",\"first_failure_stage\":";
    emit_json_string15(output, report.finalization.first_failure_stage);
    output << ",\"first_failure_cause\":";
    emit_json_string15(output, report.finalization.first_failure_cause);
    output << ",\"first_failing_mask\":";
    emit_json_string15(output, report.finalization.first_failing_mask);
    output << ",\"identity\":{";
    output << "\"contract_valid\":";
    emit_bool15(output, report.identity.contract);
    output << ",\"source_valid\":";
    emit_bool15(output, report.identity.source);
    output << ",\"parent_valid\":";
    emit_bool15(output, report.identity.parent);
    output << ",\"fixture_valid\":";
    emit_bool15(output, report.identity.fixture);
    output << ",\"pass\":";
    emit_bool15(output, report.identity.pass);
    output << ",\"profile\":";
    emit_profile15(output, report.identity.profile,
        report.identity.profile_root);
    output << ",\"reconstructed_parent_fixture_root\":";
    emit_json_string15(output,
        report.identity.reconstructed_parent_fixture_root);
    output << ",\"tight_fixture\":";
    emit_state15(output, report.identity.tight, true);
    output << ",\"work\":";
    emit_work_seal15(output, report.identity.work);
    output << "},\"parent_replay\":{";
    output << "\"contract_file_sha\":";
    emit_json_string15(output, NCGP15_PARENT_CONTRACT_FILE_SHA);
    output << ",\"contract_root\":";
    emit_json_string15(output, NCGP15_PARENT_CONTRACT_ROOT);
    output << ",\"source_root\":";
    emit_json_string15(output, NCGP15_PARENT_SOURCE_ROOT);
    output << ",\"source_commit\":";
    emit_json_string15(output, NCGP15_PARENT_SOURCE_COMMIT);
    output << ",\"source_tree\":";
    emit_json_string15(output, NCGP15_PARENT_SOURCE_TREE);
    output << ",\"binary_root\":";
    emit_json_string15(output, NCGP15_PARENT_BINARY_ROOT);
    output << ",\"stdout_root\":";
    emit_json_string15(output, NCGP15_PARENT_STDOUT_ROOT);
    output << ",\"result_root\":";
    emit_json_string15(output, NCGP15_PARENT_RESULT_ROOT);
    output << ",\"tight_fixture_root\":";
    emit_json_string15(output, NCGP15_PARENT_TIGHT_FIXTURE_ROOT);
    output << ",\"tight_lane_root\":";
    emit_json_string15(output, NCGP15_PARENT_TIGHT_LANE_ROOT);
    output << "},\"phase_a\":";
    if (report.phase_a.has_value()) emit_phase_a15(output, *report.phase_a);
    else {
        output << "{\"receipt\":";
        emit_receipt15(output, report.phase_a_receipt);
        output << ",\"executed\":null}";
    }
    output << ",\"phase_b\":[";
    for (std::size_t index = 0U; index < report.masks.size(); ++index) {
        if (index != 0U) output << ',';
        emit_mask15(output, report.masks[index]);
    }
    output << "],\"phase_c\":";
    if (report.phase_c.has_value()) emit_phase_c15(output, *report.phase_c);
    else {
        output << "{\"receipt\":";
        emit_receipt15(output, report.phase_c_receipt);
        output << ",\"executed\":null}";
    }
    output << ",\"hypotheses\":{";
    output << "\"H15A\":";
    emit_bool15(output, report.finalization.h15a);
    output << ",\"H15B\":";
    emit_bool15(output, report.finalization.h15b);
    output << ",\"H15C\":";
    emit_bool15(output, report.finalization.h15c);
    output << ",\"H15D\":";
    emit_bool15(output, report.finalization.h15d);
    output << ",\"cause_not_unique\":";
    emit_bool15(output, report.finalization.cause_not_unique);
    output << "},\"total_work\":";
    emit_work_seal15(output, report.finalization.total_work);
    output << ",\"result_root\":";
    emit_json_string15(output, report.finalization.result_root);
    output << "}\n";
}

int run_main15() {
    const Report15 report = run_report15();
    emit_report15(std::cout, report);
    return report.finalization.exit_code;
}









} // namespace
} // namespace nextengine::nonlocal::ncgp15

int main(int argc, char** argv) {
    try {
        if (argc != 2 || std::string_view(argv[1])
                != "--unified-constrained-surface-viscosity") {
            std::cerr
                << "usage: nonlocal-corrected-cpu-unified-constrained "
                   "--unified-constrained-surface-viscosity\n";
            return 64;
        }
        return nextengine::nonlocal::ncgp15::run_main15();
    } catch (const std::exception& error) {
        std::cerr << "NCGP15 apparatus failure: " << error.what() << '\n';
        return 3;
    }
}
