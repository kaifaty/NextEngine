#pragma once

#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

namespace nextengine::nonlocal::gpu_full_step {

constexpr std::uint32_t kMaximumDynamicSamples = 50000U;
constexpr std::uint32_t kMaximumNeighbors = 256U;

struct Vec3d {
    double x = 0.0;
    double y = 0.0;
    double z = 0.0;
};

struct NonlocalGpuProfile {
    std::string id;
    double dt = 1.0 / 240.0;
    double spacing = 0.05;
    double horizon = 0.15;
    double mass = 0.125;
    double rest_density = 1000.0;
    double kernel_scale = 1.0;
    double kappa = 9196.875;
    double lambda = 2.5;
    double mu = 1.7;
    double gamma = 3.5;
    Vec3d gravity{0.0, 0.0, -9.81};
    Vec3d basin_extent{3.0, 2.5, 1.5};
    std::uint32_t ghost_layers = 3U;
    std::uint32_t maximum_dynamic_samples = kMaximumDynamicSamples;
    std::uint32_t maximum_neighbors = kMaximumNeighbors;
};

struct NonlocalGpuSample {
    std::uint32_t sample_id = 0U;
    Vec3d reference;
    Vec3d current;
    Vec3d velocity;
};

struct NonlocalGpuGhost {
    std::uint32_t sample_id = 0U;
    Vec3d position;
};

enum class NonlocalGpuFailure : std::uint32_t {
    None = 0U,
    InvalidProfile = 1U,
    InvalidState = 2U,
    DuplicateSampleId = 3U,
    CapacityExceeded = 4U,
    CellRangeExceeded = 5U,
    Nonfinite = 6U,
    WorkBudgetExceeded = 7U,
    DeviceFailure = 8U,
    PhysicsGateFailed = 9U,
};

enum class NonlocalGpuVariant : std::uint32_t {
    Corrected = 0U,
    StrictRadius = 1U,
    MissingKernelChain = 2U,
    HalfViscosity = 3U,
    WrongSurfaceSign = 4U,
    HvpSignFlip = 5U,
    CurrentReferenceSwap = 6U,
    OwnerOnlyPressure = 7U,
    DisableBoundary = 8U,
    PostFinalizeFailure = 9U,
    SurfaceF64 = 10U,
    CompensatedStateF32 = 11U,
    CompensatedOmitLow = 12U,
    CompensatedBrokenEft = 13U,
    CompensatedScaleF32 = 14U,
    CompensatedScaleHighOnlyGraph = 15U,
    CompensatedScaleStrictRadius = 16U,
    CompensatedScaleHighOnlyBoundary = 17U,
    CompensatedScalePostFinalizeFailure = 18U,
    CompensatedScalePressureF64 = 19U,
    CompensatedScaleOrdinaryPredictor = 20U,
};

enum class NonlocalGpuSolverProfile : std::uint32_t {
    Unpreconditioned = 0U,
    Jacobi = 1U,
};

enum class NonlocalGpuTraceKind : std::uint32_t {
    OuterStart = 0U,
    Diagonal = 1U,
    Inner = 2U,
    Trial = 3U,
    Terminal = 4U,
};

enum class NonlocalGpuTraceReason : std::uint32_t {
    None = 0U,
    GradientConverged = 1U,
    ForcingConverged = 2U,
    NegativeCurvature = 3U,
    TrustRadius = 4U,
    Accepted = 5U,
    Rejected = 6U,
    WorkCeiling = 7U,
    InvalidModel = 8U,
    Failure = 9U,
};

struct NonlocalGpuSolverTraceEvent {
    std::uint32_t sequence = 0U;
    std::uint32_t outer = 0U;
    std::uint32_t inner = 0U;
    std::uint32_t hvp_used = 0U;
    std::uint32_t active_pressure_centers = 0U;
    std::uint32_t directed_pairs = 0U;
    NonlocalGpuTraceKind kind = NonlocalGpuTraceKind::OuterStart;
    NonlocalGpuTraceReason reason = NonlocalGpuTraceReason::None;
    std::uint64_t inertia_floor_components = 0U;
    double radius_before = 0.0;
    double radius_after = 0.0;
    double gradient_norm = 0.0;
    double scaled_displacement_residual = 0.0;
    double diagonal_min_abs = 0.0;
    double diagonal_max_abs = 0.0;
    double initial_true_residual = 0.0;
    double initial_preconditioned_residual = 0.0;
    double true_residual = 0.0;
    double preconditioned_residual = 0.0;
    double forcing = 0.0;
    double curvature = 0.0;
    double alpha = 0.0;
    double beta = 0.0;
    double step_norm = 0.0;
    double predicted_reduction = 0.0;
    double actual_reduction = 0.0;
    double rho = 0.0;
};

struct NonlocalGpuWorkReceipt {
    std::uint64_t uploads = 0U;
    std::uint64_t graph_builds = 0U;
    std::uint64_t key_evaluations = 0U;
    std::uint64_t radix_sort_items = 0U;
    std::uint64_t cell_probes = 0U;
    std::uint64_t distance_predicates = 0U;
    std::uint64_t emitted_directed_pairs = 0U;
    std::uint64_t row_sort_items = 0U;
    std::uint64_t density_kernel_evaluations = 0U;
    std::uint64_t energy_pair_visits = 0U;
    std::uint64_t gradient_pair_visits = 0U;
    std::uint64_t hvp_pair_visits = 0U;
    std::uint64_t hvp_applications = 0U;
    std::uint64_t diagonal_probes = 0U;
    std::uint64_t reduction_values = 0U;
    std::uint64_t scalar_reductions = 0U;
    std::uint64_t vector_kernel_values = 0U;
    std::uint64_t boundary_intersections = 0U;
    std::uint64_t outer_trials = 0U;
    std::uint64_t accepted_trials = 0U;
    std::uint64_t rejected_trials = 0U;
    std::uint64_t radius_shrinks = 0U;
    std::uint64_t radius_expands = 0U;
    std::uint64_t projected_gradient_components = 0U;
    std::uint64_t contact_projections = 0U;
    std::uint64_t boundary_face_tests = 0U;
    std::uint64_t boundary_face_hits = 0U;
    std::uint64_t boundary_face_mask_xor = 0U;
    std::uint64_t state_updates = 0U;
    std::uint64_t host_to_device_bytes = 0U;
    std::uint64_t device_to_host_bytes = 0U;
    std::uint64_t compensated_input_components = 0U;
    std::uint64_t compensated_decomposition_components = 0U;
    std::uint64_t compensated_reconstruction_components = 0U;
    std::uint64_t compensated_canonical_checks = 0U;
    std::uint64_t compensated_difference_components = 0U;
    std::uint64_t compensated_inertia_components = 0U;
    std::uint64_t compensated_trial_eft_components = 0U;
    std::uint64_t compensated_transaction_components = 0U;
    std::uint64_t compensated_publish_components = 0U;
    std::uint64_t compensated_graph_quantizations = 0U;
    std::uint64_t compensated_boundary_origin_components = 0U;
    std::uint64_t compensated_contact_canonicalizations = 0U;
    std::uint64_t compensated_fault_injection_components = 0U;
    std::uint64_t compensated_rollback_components = 0U;
    std::uint64_t compensated_prediction_eft_components = 0U;
    std::uint64_t allocated_device_bytes = 0U;
};

struct NonlocalGpuTimings {
    float total_ms = 0.0F;
    float graph_ms = 0.0F;
    float density_energy_gradient_ms = 0.0F;
    float hvp_ms = 0.0F;
    float solver_control_ms = 0.0F;
    float boundary_integration_ms = 0.0F;
};

struct NonlocalGpuGraphResult {
    NonlocalGpuFailure failure = NonlocalGpuFailure::None;
    std::uint32_t dynamic_samples = 0U;
    std::uint32_t ghost_samples = 0U;
    std::uint32_t directed_pairs = 0U;
    std::uint32_t maximum_degree = 0U;
    std::uint32_t overflow_owner_id = 0U;
    std::uint32_t overflow_dynamic_neighbors = 0U;
    std::uint32_t overflow_ghost_neighbors = 0U;
    std::vector<std::uint32_t> owner_ids;
    std::vector<std::uint32_t> offsets;
    std::vector<std::uint32_t> neighbor_ids;
    NonlocalGpuWorkReceipt work;
    NonlocalGpuTimings timing;
};

struct NonlocalGpuEvaluationResult {
    NonlocalGpuFailure failure = NonlocalGpuFailure::None;
    double energy = 0.0;
    double gradient_norm = 0.0;
    std::uint32_t active_pressure_centers = 0U;
    std::uint32_t directed_pairs = 0U;
    std::uint32_t maximum_degree = 0U;
    std::vector<std::uint32_t> active_pressure_ids;
    std::vector<Vec3d> gradient;
    std::vector<Vec3d> hvp;
    std::vector<double> density;
    NonlocalGpuWorkReceipt work;
    NonlocalGpuTimings timing;
};

struct NonlocalGpuBoundaryProbeResult {
    NonlocalGpuFailure failure = NonlocalGpuFailure::None;
    std::vector<Vec3d> trial_high;
    std::vector<Vec3d> trial_low;
    std::vector<Vec3d> contact_impulse;
    std::uint64_t face_mask_xor = 0U;
    NonlocalGpuWorkReceipt work;
};

struct NonlocalGpuCompensatedStateSnapshot {
    NonlocalGpuFailure failure = NonlocalGpuFailure::None;
    std::vector<std::uint32_t> ids;
    std::vector<Vec3d> reference_high;
    std::vector<Vec3d> reference_low;
    std::vector<Vec3d> current_high;
    std::vector<Vec3d> current_low;
    std::vector<Vec3d> predicted_high;
    std::vector<Vec3d> predicted_low;
    std::vector<Vec3d> velocity_high;
    std::vector<Vec3d> velocity_low;
    NonlocalGpuWorkReceipt work;
};

struct NonlocalGpuPublicSnapshot {
    NonlocalGpuFailure failure = NonlocalGpuFailure::None;
    std::vector<NonlocalGpuSample> state;
    std::vector<double> density;
    std::vector<std::uint32_t> active_pressure_ids;
    NonlocalGpuWorkReceipt work;
};

struct NonlocalGpuStepResult {
    NonlocalGpuFailure failure = NonlocalGpuFailure::None;
    std::vector<NonlocalGpuSample> state;
    std::vector<double> density;
    double initial_energy = 0.0;
    double final_energy = 0.0;
    double gradient_norm = 0.0;
    double scaled_displacement_residual = 0.0;
    std::uint32_t active_pressure_centers = 0U;
    std::uint32_t maximum_directed_pairs = 0U;
    std::uint32_t maximum_degree = 0U;
    std::vector<std::uint32_t> active_pressure_ids;
    std::uint32_t hvp_budget = 0U;
    std::uint32_t hvp_used = 0U;
    std::uint32_t outer_trials = 0U;
    double maximum_penetration_m = 0.0;
    Vec3d boundary_impulse;
    std::uint64_t boundary_face_mask_xor = 0U;
    NonlocalGpuSolverProfile solver_profile =
        NonlocalGpuSolverProfile::Unpreconditioned;
    NonlocalGpuVariant variant = NonlocalGpuVariant::Corrected;
    NonlocalGpuWorkReceipt work;
    NonlocalGpuTimings timing;
    std::vector<NonlocalGpuSolverTraceEvent> trace;
};

class NonlocalGpuWorkspace {
public:
    explicit NonlocalGpuWorkspace(const NonlocalGpuProfile& profile);
    ~NonlocalGpuWorkspace();
    NonlocalGpuWorkspace(const NonlocalGpuWorkspace&) = delete;
    NonlocalGpuWorkspace& operator=(const NonlocalGpuWorkspace&) = delete;

    NonlocalGpuFailure upload(const std::vector<NonlocalGpuSample>& samples,
        const std::vector<NonlocalGpuGhost>& ghosts,
        bool prepare_compensated_state = false);

    NonlocalGpuGraphResult build_current_graph(NonlocalGpuVariant variant,
        bool capture_payload,
        bool measure);

    NonlocalGpuEvaluationResult evaluate(const std::vector<Vec3d>* direction,
        NonlocalGpuVariant variant,
        bool capture_payload,
        bool measure);

    NonlocalGpuBoundaryProbeResult probe_boundary(
        const std::vector<Vec3d>& proposal,
        NonlocalGpuVariant variant,
        bool capture_payload);

    NonlocalGpuCompensatedStateSnapshot capture_compensated_state();
    NonlocalGpuPublicSnapshot capture_public_snapshot();

    NonlocalGpuStepResult step(std::uint32_t total_hvp_budget,
        NonlocalGpuSolverProfile solver_profile,
        NonlocalGpuVariant variant,
        bool capture_state,
        bool measure,
        bool capture_trace = false);

    std::string environment_json() const;
    std::size_t allocated_device_bytes() const;

private:
    struct Impl;
    Impl* impl_ = nullptr;
};

NonlocalGpuProfile nonlocal_water_profile();
NonlocalGpuProfile nonlocal_water_corrected_profile();

NonlocalGpuFailure validate_nonlocal_input(
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts);

std::vector<NonlocalGpuSample> canonicalize_samples_binary32(
    const std::vector<NonlocalGpuSample>& samples);

std::vector<NonlocalGpuGhost> canonicalize_ghosts_binary32(
    const std::vector<NonlocalGpuGhost>& ghosts);

std::vector<NonlocalGpuSample> make_lattice_state(
    const NonlocalGpuProfile& profile,
    std::uint32_t nx,
    std::uint32_t ny,
    std::uint32_t nz,
    bool permuted,
    bool advected);

std::vector<NonlocalGpuGhost> make_basin_ghosts(
    const NonlocalGpuProfile& profile,
    std::uint32_t first_sample_id = 0x80000000U);

NonlocalGpuGraphResult build_reference_graph(
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts,
    bool strict_radius);

NonlocalGpuEvaluationResult evaluate_reference(
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts,
    const std::vector<Vec3d>* direction,
    NonlocalGpuVariant variant);

NonlocalGpuStepResult step_reference(
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts,
    std::uint32_t total_hvp_budget,
    NonlocalGpuVariant variant,
    bool capture_state);

std::string graph_semantic_root(const NonlocalGpuGraphResult& graph);
std::string work_semantic_root(const NonlocalGpuWorkReceipt& work);
std::string profile_semantic_root(const NonlocalGpuProfile& profile);
std::string input_semantic_root(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts);
std::string step_work_semantic_root(const NonlocalGpuProfile& profile,
    const NonlocalGpuStepResult& result);
std::string step_semantic_root(const NonlocalGpuProfile& profile,
    const std::string& input_root,
    const NonlocalGpuStepResult& result);
std::string solver_trace_semantic_root(
    const NonlocalGpuStepResult& result);
bool solver_trace_valid(const NonlocalGpuStepResult& result);

} // namespace nextengine::nonlocal::gpu_full_step
