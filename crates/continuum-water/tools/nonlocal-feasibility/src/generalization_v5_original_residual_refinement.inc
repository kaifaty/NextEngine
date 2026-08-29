namespace {

constexpr const char* AL_R20_R63J_PARENT_SEMANTIC_SHA256 =
    "4055d0b043bd14c7c65a28f052bef02d39a55bc9625a9c2d861f41fdf0466415";
constexpr const char* AL_R20_R63J_WIDE_SOLVE_SHA256 =
    "e76e676b410b0a315013d94645e78af598497f1ea8f074c3e16e1bcdaa57c4a2";
constexpr const char* AL_R20_R63J_EXPORT_WIDE_SOLVE_SHA256 =
    "c9e3c16966c9ed31d513af678ae7edec369448d49d600ea301230daa9319db5f";
constexpr const char* AL_R20_R63J_STRICT_SOLVE_SHA256 =
    "a15db84826bb3ace61015b14e855910818f2b0c41ef7816899c031dfbdc1da79";
constexpr const char* AL_R20_R63J_WIDE_CERTIFICATE_SHA256 =
    "04767e5592f87f615a28d6b7d1a1a693f50bad534a1bb1fec91a94c8db2d23bb";
constexpr const char* AL_R20_R63J_EXPORT_WIDE_CERTIFICATE_SHA256 =
    "bda9f523326080e9025de277cc89916ed9e80041a6ad1968847a48bd6b90c64b";
constexpr const char* AL_R20_R63J_STRICT_CERTIFICATE_SHA256 =
    "190e71c0327119b093c6caa97c3ea0c60f96a479b67488a76dd12c2af0208a21";
constexpr const char* AL_R20_R63J_REFERENCE_SHA256 =
    "454dfcc38e41a82cbd81e9b50fcdad2435aaf5d3c2a5512e3fddc0c4604cd935";
constexpr std::array<std::size_t, 6U> AL_R20_R63J_DEPTHS{
    4U, 8U, 9U, 10U, 12U, 16U};

bool al_r20_r63j_checkpoint_depth(std::size_t depth) {
    return std::find(AL_R20_R63J_DEPTHS.begin(),
        AL_R20_R63J_DEPTHS.end(), depth) != AL_R20_R63J_DEPTHS.end();
}

struct ALR20R63JDefect {
    bool exact = false;
    bool no_underflow = false;
    bool contractive = false;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    std::size_t worst_row = 0U;
    Binary128 rho_bound = static_cast<Binary128>(0.0);
    Binary128 maximum_bound = static_cast<Binary128>(0.0);
    std::vector<Binary128> value;
    std::vector<Binary128> bound;
    std::string root;
};

ALR20R63JDefect al_r20_r63j_defect(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse) {
    ALR20R63JDefect result;
    if (matrix.empty() || matrix.size() != inverse.size()) return result;
    result.dimension = static_cast<std::size_t>(
        sqrt(static_cast<double>(matrix.size())));
    const std::size_t dimension = result.dimension;
    if (dimension * dimension != matrix.size()) return result;
    result.value.resize(matrix.size());
    result.bound.resize(matrix.size());
    bool exact = true;
    bool no_underflow = true;
    for (std::size_t row = 0U; row < dimension; ++row) {
        Binary128 row_bound = static_cast<Binary128>(0.0);
        for (std::size_t column = 0U; column < dimension; ++column) {
            std::vector<Binary128> left(dimension + 1U);
            std::vector<Binary128> right(dimension + 1U);
            left[0U] = row == column
                ? static_cast<Binary128>(1.0)
                : static_cast<Binary128>(0.0);
            right[0U] = static_cast<Binary128>(1.0);
            for (std::size_t inner = 0U; inner < dimension; ++inner) {
                left[inner + 1U] = inverse[row * dimension + inner];
                right[inner + 1U] = -matrix[inner * dimension + column];
            }
            const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
            const std::size_t index = row * dimension + column;
            result.value[index] = dot.value;
            result.bound[index] = dot.bound;
            exact = exact && dot.exact && finiteq(dot.value) != 0
                && finiteq(dot.bound) != 0;
            no_underflow = no_underflow && dot.no_underflow;
            result.maximum_bound = fmaxq(result.maximum_bound, dot.bound);
            row_bound = al_r20_r38_up_add(row_bound,
                al_r20_r38_up_add(fabsq(dot.value), dot.bound));
            ++result.dots;
        }
        if (row == 0U || row_bound > result.rho_bound) {
            result.rho_bound = row_bound;
            result.worst_row = row;
        }
    }
    result.no_underflow = no_underflow;
    result.contractive = result.rho_bound
        < static_cast<Binary128>(1.0);
    result.exact = exact && no_underflow
        && result.dots == dimension * dimension
        && finiteq(result.rho_bound) != 0
        && finiteq(result.maximum_bound) != 0;
    std::ostringstream material;
    material << result.exact << ':' << result.no_underflow << ':'
        << result.contractive << ':' << result.dimension << ':'
        << result.dots << ':' << result.worst_row << ':'
        << al_binary128_hex(result.rho_bound) << ':'
        << al_binary128_hex(result.maximum_bound) << ':'
        << al_r20_q_scalar_root(result.value) << ':'
        << al_r20_q_scalar_root(result.bound);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63JCheckpoint {
    bool exact = false;
    std::size_t depth = 0U;
    std::size_t center_dots = 0U;
    Binary128 maximum_center_bound = static_cast<Binary128>(0.0);
    std::vector<Binary128> center;
    std::vector<Binary128> center_bound;
    ALR20R63ISignCertificate certificate;
    std::string root;
};

struct ALR20R63JLane {
    bool exact = false;
    bool passed = false;
    bool no_underflow = false;
    std::size_t dimension = 0U;
    std::size_t recurrence_dots = 0U;
    std::size_t center_dots = 0U;
    std::size_t certificate_residual_dots = 0U;
    std::size_t certificate_image_dots = 0U;
    std::size_t first_passing_depth = 0U;
    std::vector<Binary128> correction;
    std::vector<ALR20R63JCheckpoint> checkpoints;
    std::string root;
};

ALR20R63JLane al_r20_r63j_lane(
    const std::vector<Binary128>& matrix,
    const std::vector<Binary128>& inverse,
    const std::vector<Binary128>& rhs,
    const std::vector<Binary128>& initial,
    const ALR20R63ISignCertificate& baseline,
    const ALR20R63JDefect& defect,
    const std::vector<int>& reference_signs,
    const std::vector<Binary128>& reference_solution) {
    ALR20R63JLane result;
    result.dimension = rhs.size();
    const std::size_t dimension = result.dimension;
    if (dimension == 0U || matrix.size() != dimension * dimension
        || inverse.size() != dimension * dimension
        || initial.size() != dimension || !baseline.exact
        || baseline.passed || baseline.unresolved == 0U
        || baseline.image.size() != dimension
        || !defect.exact || !defect.contractive
        || defect.dimension != dimension
        || reference_signs.size() != dimension
        || reference_solution.size() != dimension) return result;
    result.correction.assign(dimension, static_cast<Binary128>(0.0));
    bool exact = true;
    bool no_underflow = true;
    for (std::size_t depth = 1U; depth <= 16U; ++depth) {
        std::vector<Binary128> next(dimension);
        for (std::size_t row = 0U; row < dimension; ++row) {
            std::vector<Binary128> left(dimension + 1U);
            std::vector<Binary128> right(dimension + 1U);
            left[0U] = baseline.image[row];
            right[0U] = static_cast<Binary128>(1.0);
            for (std::size_t column = 0U; column < dimension; ++column) {
                left[column + 1U] = defect.value[row * dimension + column];
                right[column + 1U] = result.correction[column];
            }
            const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
            next[row] = dot.value;
            exact = exact && dot.exact && finiteq(dot.value) != 0
                && finiteq(dot.bound) != 0;
            no_underflow = no_underflow && dot.no_underflow;
            ++result.recurrence_dots;
        }
        result.correction = std::move(next);
        if (!al_r20_r63j_checkpoint_depth(depth)) continue;
        ALR20R63JCheckpoint checkpoint;
        checkpoint.depth = depth;
        checkpoint.center.resize(dimension);
        checkpoint.center_bound.resize(dimension);
        bool center_exact = true;
        for (std::size_t row = 0U; row < dimension; ++row) {
            const std::vector<Binary128> left{
                initial[row], result.correction[row]};
            const std::vector<Binary128> right{
                static_cast<Binary128>(1.0),
                static_cast<Binary128>(1.0)};
            const ALR20R38Dot2 dot = al_r20_r38_dot2(left, right);
            checkpoint.center[row] = dot.value;
            checkpoint.center_bound[row] = dot.bound;
            center_exact = center_exact && dot.exact
                && finiteq(dot.value) != 0 && finiteq(dot.bound) != 0;
            no_underflow = no_underflow && dot.no_underflow;
            checkpoint.maximum_center_bound = fmaxq(
                checkpoint.maximum_center_bound, dot.bound);
            ++checkpoint.center_dots;
            ++result.center_dots;
        }
        checkpoint.certificate = center_exact
            ? al_r20_r63i_sign_certificate(matrix, inverse, rhs,
                checkpoint.center, reference_signs, defect.rho_bound,
                reference_solution)
            : ALR20R63ISignCertificate{};
        result.certificate_residual_dots +=
            checkpoint.certificate.residual_dots;
        result.certificate_image_dots +=
            checkpoint.certificate.image_dots;
        checkpoint.exact = center_exact
            && checkpoint.center_dots == dimension
            && checkpoint.certificate.exact;
        if (checkpoint.exact && checkpoint.certificate.passed
            && result.first_passing_depth == 0U)
            result.first_passing_depth = depth;
        std::ostringstream checkpoint_material;
        checkpoint_material << checkpoint.exact << ':' << checkpoint.depth
            << ':' << checkpoint.center_dots << ':'
            << al_binary128_hex(checkpoint.maximum_center_bound) << ':'
            << al_r20_q_scalar_root(checkpoint.center) << ':'
            << al_r20_q_scalar_root(checkpoint.center_bound) << ':'
            << checkpoint.certificate.root;
        checkpoint.root = sha256_hex(checkpoint_material.str());
        exact = exact && checkpoint.exact;
        result.checkpoints.push_back(std::move(checkpoint));
    }
    result.no_underflow = no_underflow;
    result.passed = result.first_passing_depth != 0U;
    result.exact = exact && no_underflow
        && result.recurrence_dots == 16U * dimension
        && result.checkpoints.size() == AL_R20_R63J_DEPTHS.size()
        && result.center_dots == AL_R20_R63J_DEPTHS.size() * dimension
        && result.certificate_residual_dots
            == AL_R20_R63J_DEPTHS.size() * dimension
        && result.certificate_image_dots
            == AL_R20_R63J_DEPTHS.size() * dimension;
    std::ostringstream material;
    material << result.exact << ':' << result.passed << ':'
        << result.no_underflow << ':' << result.dimension << ':'
        << result.recurrence_dots << ':' << result.center_dots << ':'
        << result.certificate_residual_dots << ':'
        << result.certificate_image_dots << ':'
        << result.first_passing_depth << ':'
        << al_r20_q_scalar_root(result.correction) << '|';
    for (const ALR20R63JCheckpoint& checkpoint : result.checkpoints)
        material << checkpoint.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63j_classify(bool apparatus, bool wide,
    bool export_wide, bool strict_binary64) {
    if (!apparatus)
        return "ORIGINAL_RESIDUAL_REFINEMENT_APPARATUS_REJECTED";
    if (!wide) return "WIDE_ORIGINAL_RESIDUAL_REFINEMENT_REJECTED";
    if (!export_wide) return "EXPORTED_FACTOR_WIDE_REFINEMENT_REJECTED";
    if (!strict_binary64)
        return "EXPORTED_FACTOR_BINARY64_REFINEMENT_REJECTED";
    return "EXPORTED_FACTOR_REFINED_RHS_CANDIDATE";
}

struct ALR20R63JControls {
    bool exact = false;
    bool ladder = false;
    bool defect = false;
    bool recurrence = false;
    bool crossing = false;
    bool noncontractive = false;
    bool mutation = false;
    bool classifier = false;
    std::string root;
};

ALR20R63JControls al_r20_r63j_controls() {
    ALR20R63JControls result;
    result.ladder = AL_R20_R63J_DEPTHS
        == std::array<std::size_t, 6U>{4U, 8U, 9U, 10U, 12U, 16U};
    const std::vector<Binary128> matrix{static_cast<Binary128>(1.0)};
    const std::vector<Binary128> inverse{static_cast<Binary128>(0.75)};
    const std::vector<Binary128> rhs{static_cast<Binary128>(1.0)};
    const std::vector<Binary128> initial{static_cast<Binary128>(-3.0)};
    const std::vector<Binary128> reference{static_cast<Binary128>(1.0)};
    const std::vector<int> signs{1};
    const ALR20R63JDefect defect = al_r20_r63j_defect(matrix, inverse);
    const ALR20R63ISignCertificate baseline =
        al_r20_r63i_sign_certificate(matrix, inverse, rhs, initial, signs,
            defect.rho_bound, reference);
    const ALR20R63JLane lane = al_r20_r63j_lane(matrix, inverse, rhs,
        initial, baseline, defect, signs, reference);
    result.defect = defect.exact && defect.contractive
        && defect.dimension == 1U && defect.dots == 1U;
    result.recurrence = lane.exact && lane.passed
        && lane.checkpoints.size() == 6U;
    result.crossing = !baseline.passed && baseline.unresolved == 1U
        && lane.first_passing_depth == 4U
        && lane.checkpoints.front().certificate.positive == 1U;
    const ALR20R63JDefect noncontractive = al_r20_r63j_defect(
        matrix, {static_cast<Binary128>(0.0)});
    result.noncontractive = noncontractive.exact
        && !noncontractive.contractive;
    std::vector<Binary128> mutated = lane.correction;
    mutated[0U] = nextafterq(mutated[0U], HUGE_VALQ);
    result.mutation = al_r20_q_scalar_root(mutated)
        != al_r20_q_scalar_root(lane.correction);
    result.classifier =
        al_r20_r63j_classify(false, true, true, true)
            == "ORIGINAL_RESIDUAL_REFINEMENT_APPARATUS_REJECTED"
        && al_r20_r63j_classify(true, false, true, true)
            == "WIDE_ORIGINAL_RESIDUAL_REFINEMENT_REJECTED"
        && al_r20_r63j_classify(true, true, false, true)
            == "EXPORTED_FACTOR_WIDE_REFINEMENT_REJECTED"
        && al_r20_r63j_classify(true, true, true, false)
            == "EXPORTED_FACTOR_BINARY64_REFINEMENT_REJECTED"
        && al_r20_r63j_classify(true, true, true, true)
            == "EXPORTED_FACTOR_REFINED_RHS_CANDIDATE";
    result.exact = result.ladder && result.defect && result.recurrence
        && result.crossing && result.noncontractive && result.mutation
        && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + defect.root + ':' + baseline.root + ':' + lane.root + ':'
        + al_r20_q_scalar_root(mutated));
    return result;
}

} // namespace

SplitBoundaryReport
run_al_generalization_v5_original_residual_refinement_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__) && !defined(__FAST_MATH__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT128_MANT_DIG == 113
        && sizeof(double) == 8U && FLT_RADIX == 2 && DBL_MANT_DIG == 53
        && FLT_EVAL_METHOD == 0 && std::numeric_limits<double>::is_iec559
        && std::numeric_limits<double>::has_denorm == std::denorm_present
        && std::fegetround() == FE_TONEAREST;
    const ALR20R63JControls controls = al_r20_r63j_controls();
    const bool controls_exact = profile_exact && controls.exact;

    std::vector<ALR20ManifestCase> cases = al_r20_v5_manifest_cases();
    bool parent_exact = controls_exact && cases.size() == 6U
        && al_r20_v5_manifest_root(cases) == AL_R20_V5_MANIFEST_SHA256;
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    ALR20ProblemData problem;
    bool problem_materialized = false;
    if (parent_exact) {
        problem = al_r20_materialize_problem(
            cases[3U], &sparse_work, &static_work, &adjacency_work);
        problem_materialized = true;
        parent_exact = problem.exact && problem.source_positive == 0U
            && problem.target.size() == 105U
            && problem.problem_root == AL_R20_R60_PROBLEM_SHA256;
    }

    ALR20R60Capture solution_capture;
    ALR20VerifiedInverseCapture factor_capture;
    factor_capture.selected_matrix_root = AL_R20_R63_MATRIX_SHA256;
    ALR20R55SlopeTrace slope;
    ALR20R35CaseAudit oblique;
    std::size_t solver_executions = 0U;
    const bool hook_slot_clear = al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_verified_inverse_projector_context == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr;
    if (parent_exact && hook_slot_clear) {
        al_r20_r60_capture = &solution_capture;
        al_r20_verified_solution_refiner = &al_r20_r60_capture_refiner;
        al_r20_verified_inverse_capture = &factor_capture;
        al_r20_r55_slope_trace = &slope;
        al_r20_slope_error_refiner = &al_r20_r55_slope_refiner;
        oblique = al_r20_r35_case(problem);
        al_r20_slope_error_refiner = nullptr;
        al_r20_r55_slope_trace = nullptr;
        al_r20_verified_inverse_capture = nullptr;
        al_r20_verified_solution_refiner = nullptr;
        al_r20_r60_capture = nullptr;
        solver_executions = 1U;
    }
    const bool hook_slot_released =
        al_r20_verified_solution_refiner == nullptr
        && al_r20_r60_capture == nullptr
        && al_r20_verified_inverse_capture == nullptr
        && al_r20_verified_inverse_projector_context == nullptr
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && al_r20_direct_solution_capture == nullptr;
    const bool capture_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(oblique)
        && oblique.root == AL_R20_R60_CASE_SHA256
        && solution_capture.calls == 1U
        && factor_capture.calls == 125U
        && factor_capture.selected_calls == 1U
        && factor_capture.selected_column_solves == 102U
        && factor_capture.matrix_root == AL_R20_R63_MATRIX_SHA256
        && al_r20_q_scalar_root(factor_capture.inverse)
            == AL_R20_R63I_INVERSE_SHA256
        && al_r20_q_scalar_root(factor_capture.rhs)
            == AL_R20_R63I_RHS_SHA256
        && al_r20_q_scalar_root(factor_capture.solution)
            == AL_R20_R63I_SOLUTION_SHA256
        && factor_capture.rhs.size() == 102U
        && factor_capture.solution.size() == 102U
        && factor_capture.projector.exact
        && slope.calls == 0U && slope.inverse_audits == 0U;

    const ALR20R63EProjectorGrams grams = capture_exact
        ? al_r20_r63e_projector_grams(problem.op,
            factor_capture.source_rows, problem.target.size(),
            factor_capture.projector, factor_capture.matrix)
        : ALR20R63EProjectorGrams{};
    const ALR20R63ERankProfile full_profile = grams.exact
        ? al_r20_r63e_rank_profile(grams.full, grams.rows)
        : ALR20R63ERankProfile{};
    const ALR20R63FTangentMatrix tangent = grams.exact
        ? al_r20_r63f_tangent_matrix(grams, factor_capture.projector,
            factor_capture.source_rows)
        : ALR20R63FTangentMatrix{};
    const std::vector<std::size_t> permutation = full_profile.exact
        ? full_profile.binary128.permutation
        : std::vector<std::size_t>{};
    const bool permutation_exact = al_r20_r63g_permutation_exact(
        permutation, tangent.rows);

    std::vector<Binary128> transposed;
    std::vector<double> permuted_d;
    if (tangent.exact && permutation_exact) {
        transposed.resize(tangent.columns * tangent.rows);
        permuted_d.resize(transposed.size());
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column)
                transposed[scalar * tangent.rows + column] =
                    tangent.values[column * tangent.columns + scalar];
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column)
                permuted_d[scalar * tangent.rows + column] =
                    static_cast<double>(transposed[
                        scalar * tangent.rows + permutation[column]]);
    }
    const ALR20R63GDFactor parent_d_factor = !permuted_d.empty()
        ? al_r20_r63g_d_factor(
            permuted_d, tangent.columns, tangent.rows)
        : ALR20R63GDFactor{};
    const std::vector<Binary128> stored_permuted =
        al_r20_r63g_q_from_double(parent_d_factor.input);
    const ALR20R63GQFactor wide_factor = !stored_permuted.empty()
        ? al_r20_r63g_q_factor(
            stored_permuted, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    std::vector<double> exported_r_d;
    if (wide_factor.exact) {
        exported_r_d.reserve(wide_factor.r.size());
        for (Binary128 value : wide_factor.r)
            exported_r_d.push_back(static_cast<double>(value));
    }
    const std::vector<Binary128> exported_r =
        al_r20_r63g_q_from_double(exported_r_d);
    const bool factor_exact = capture_exact && grams.exact
        && full_profile.exact && tangent.exact && permutation_exact
        && parent_d_factor.exact && wide_factor.exact
        && wide_factor.root == AL_R20_R63I_WIDE_FACTOR_SHA256
        && al_r20_double_root(exported_r_d)
            == AL_R20_R63I_EXPORT_R_SHA256;

    const ALR20R42PracticalCertificate reference_certificate = capture_exact
        ? al_r20_r42_practical_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            factor_capture.solution, 4U, false)
        : ALR20R42PracticalCertificate{};
    std::vector<int> reference_signs(102U, 0);
    bool reference_signs_exact = reference_certificate.passed
        && reference_certificate.solution.size() == 102U;
    for (std::size_t index = 0U;
         index < reference_signs.size() && reference_signs_exact; ++index) {
        if (reference_certificate.solution[index]
                - reference_certificate.error
            > static_cast<Binary128>(0.0))
            reference_signs[index] = 1;
        else if (reference_certificate.solution[index]
                + reference_certificate.error
            < static_cast<Binary128>(0.0))
            reference_signs[index] = -1;
        else
            reference_signs_exact = false;
    }
    const bool reference_exact = reference_signs_exact
        && reference_certificate.exact && reference_certificate.passed
        && reference_certificate.contractive
        && reference_certificate.positive == 24U
        && reference_certificate.negative == 78U
        && reference_certificate.unresolved == 0U
        && reference_certificate.root == AL_R20_R63J_REFERENCE_SHA256;

    const Binary128 inverse_scale = static_cast<Binary128>(1.0)
        + factor_capture.projector.eta;
    const ALR20R63IQSolve wide_solve = factor_exact
        ? al_r20_r63i_q_solve(wide_factor.r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IQSolve export_wide_solve = factor_exact
        ? al_r20_r63i_q_solve(exported_r, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IQSolve{};
    const ALR20R63IDSolve strict_solve = factor_exact
        ? al_r20_r63i_d_solve(exported_r_d, permutation,
            factor_capture.rhs, inverse_scale)
        : ALR20R63IDSolve{};
    const std::vector<Binary128> strict_solution =
        al_r20_r63g_q_from_double(strict_solve.solution);
    const ALR20R63ISignCertificate wide_baseline =
        wide_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            wide_solve.solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};
    const ALR20R63ISignCertificate export_wide_baseline =
        export_wide_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            export_wide_solve.solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};
    const ALR20R63ISignCertificate strict_baseline =
        strict_solve.exact && reference_exact
        ? al_r20_r63i_sign_certificate(factor_capture.matrix,
            factor_capture.inverse, factor_capture.rhs,
            strict_solution, reference_signs,
            reference_certificate.rho_bound,
            reference_certificate.solution)
        : ALR20R63ISignCertificate{};
    const bool parent_r63i_exact = factor_exact && reference_exact
        && wide_solve.exact && export_wide_solve.exact && strict_solve.exact
        && wide_solve.root == AL_R20_R63J_WIDE_SOLVE_SHA256
        && export_wide_solve.root == AL_R20_R63J_EXPORT_WIDE_SOLVE_SHA256
        && strict_solve.root == AL_R20_R63J_STRICT_SOLVE_SHA256
        && wide_baseline.exact && !wide_baseline.passed
        && export_wide_baseline.exact && !export_wide_baseline.passed
        && strict_baseline.exact && !strict_baseline.passed
        && wide_baseline.unresolved == 66U
        && export_wide_baseline.unresolved == 66U
        && strict_baseline.unresolved == 66U
        && wide_baseline.root == AL_R20_R63J_WIDE_CERTIFICATE_SHA256
        && export_wide_baseline.root
            == AL_R20_R63J_EXPORT_WIDE_CERTIFICATE_SHA256
        && strict_baseline.root == AL_R20_R63J_STRICT_CERTIFICATE_SHA256;

    const ALR20R63JDefect defect = parent_r63i_exact
        ? al_r20_r63j_defect(
            factor_capture.matrix, factor_capture.inverse)
        : ALR20R63JDefect{};
    const bool defect_exact = defect.exact && defect.contractive
        && defect.dimension == 102U && defect.dots == 10404U
        && defect.rho_bound == reference_certificate.rho_bound;
    const ALR20R63JLane wide_lane = defect_exact
        ? al_r20_r63j_lane(factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, wide_solve.solution, wide_baseline, defect,
            reference_signs, reference_certificate.solution)
        : ALR20R63JLane{};
    const ALR20R63JLane export_wide_lane = defect_exact
        ? al_r20_r63j_lane(factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, export_wide_solve.solution,
            export_wide_baseline, defect, reference_signs,
            reference_certificate.solution)
        : ALR20R63JLane{};
    const ALR20R63JLane strict_lane = defect_exact
        ? al_r20_r63j_lane(factor_capture.matrix, factor_capture.inverse,
            factor_capture.rhs, strict_solution, strict_baseline, defect,
            reference_signs, reference_certificate.solution)
        : ALR20R63JLane{};

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    auto lane_work_exact = [](const ALR20R63JLane& lane) {
        return lane.recurrence_dots == 1632U
            && lane.center_dots == 612U
            && lane.certificate_residual_dots == 612U
            && lane.certificate_image_dots == 612U;
    };
    const bool work_exact = defect.dots == 10404U
        && wide_solve.forward_terms == 5151U
        && wide_solve.backward_terms == 5151U
        && export_wide_solve.forward_terms == 5151U
        && export_wide_solve.backward_terms == 5151U
        && strict_solve.forward_terms == 5151U
        && strict_solve.backward_terms == 5151U
        && lane_work_exact(wide_lane)
        && lane_work_exact(export_wide_lane)
        && lane_work_exact(strict_lane);
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_r63i_exact && defect_exact
        && wide_lane.exact && export_wide_lane.exact && strict_lane.exact
        && work_exact && lifecycle_exact;
    const std::string route = al_r20_r63j_classify(apparatus_exact,
        wide_lane.passed, export_wide_lane.passed, strict_lane.passed);
    const bool exact = apparatus_exact;

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63j-original-residual-refinement.v1|"
        << AL_R20_R63J_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << controls.root << ':' << defect.root << ':'
        << wide_solve.root << ':' << export_wide_solve.root << ':'
        << strict_solve.root << ':' << wide_baseline.root << ':'
        << export_wide_baseline.root << ':' << strict_baseline.root << ':'
        << wide_lane.root << ':' << export_wide_lane.root << ':'
        << strict_lane.root;
    const std::string result_sha = sha256_hex(semantic.str());

    auto lane_json = [](const char* name, const ALR20R63JLane& lane) {
        std::ostringstream stream;
        stream << "\"" << name << "\":{\"exact\":"
            << (lane.exact ? "true" : "false")
            << ",\"passed\":" << (lane.passed ? "true" : "false")
            << ",\"first_passing_depth\":" << lane.first_passing_depth
            << ",\"checkpoints\":[";
        for (std::size_t index = 0U; index < lane.checkpoints.size(); ++index) {
            if (index != 0U) stream << ',';
            const ALR20R63JCheckpoint& checkpoint = lane.checkpoints[index];
            const ALR20R63ISignCertificate& certificate =
                checkpoint.certificate;
            stream << "{\"depth\":" << checkpoint.depth
                << ",\"exact\":"
                << (checkpoint.exact ? "true" : "false")
                << ",\"passed\":"
                << (certificate.passed ? "true" : "false")
                << ",\"error\":\""
                << al_binary128_text(certificate.error)
                << "\",\"positive\":" << certificate.positive
                << ",\"negative\":" << certificate.negative
                << ",\"unresolved\":" << certificate.unresolved
                << ",\"minimum_separation\":\""
                << al_binary128_text(certificate.minimum_separation)
                << "\",\"maximum_reference_difference\":\""
                << al_binary128_text(
                    certificate.maximum_reference_difference)
                << "\",\"root\":\"" << checkpoint.root << "\"}";
        }
        stream << "],\"root\":\"" << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63j_original_residual_refinement.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63i_sha256\":\""
        << AL_R20_R63J_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls\":{\"exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"ladder\":" << (controls.ladder ? "true" : "false")
        << ",\"defect\":" << (controls.defect ? "true" : "false")
        << ",\"recurrence\":"
        << (controls.recurrence ? "true" : "false")
        << ",\"crossing\":" << (controls.crossing ? "true" : "false")
        << ",\"noncontractive\":"
        << (controls.noncontractive ? "true" : "false")
        << ",\"mutation\":" << (controls.mutation ? "true" : "false")
        << ",\"classifier\":"
        << (controls.classifier ? "true" : "false")
        << ",\"root\":\"" << controls.root << "\"}"
        << ",\"parent_r63i_exact\":"
        << (parent_r63i_exact ? "true" : "false")
        << ",\"defect\":{\"exact\":"
        << (defect_exact ? "true" : "false")
        << ",\"rho_bound\":\"" << al_binary128_text(defect.rho_bound)
        << "\",\"worst_row\":" << defect.worst_row
        << ",\"maximum_bound\":\""
        << al_binary128_text(defect.maximum_bound)
        << "\",\"root\":\"" << defect.root << "\"}"
        << ",\"lanes\":{" << lane_json("wide", wide_lane) << ','
        << lane_json("export_wide", export_wide_lane) << ','
        << lane_json("strict_binary64", strict_lane) << '}'
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"initial_rhs_solves\":3,\"centered_refinements\":3"
        << ",\"checkpoint_certificates\":18"
        << ",\"new_inverse_constructions\":0,\"new_factorizations\":0"
        << ",\"gmres_solves\":0,\"lsqr_solves\":0,\"svd_factorizations\":0"
        << ",\"rank_actions\":0,\"row_drops\":0,\"regularizations\":0"
        << ",\"replacement_applies\":0,\"following_transitions\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
