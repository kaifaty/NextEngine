namespace {

constexpr const char* AL_R20_R63Z_PARENT_SEMANTIC_SHA256 =
    "ad3d84abaecc5142f05d369d0e653533c5e152cac2a9112efb2ad8277d70fb9d";
constexpr const char* AL_R20_R63Z_PARENT_STDOUT_SHA256 =
    "7427ecdb3c4c7c95900bc6046545d298a674be42865936bad543c7e0191f4372";
constexpr const char* AL_R20_R63Z_TANGENT_SHA256 =
    "114a73ea34534ae25c0ea33a708944a434987374e18b703df542de8130aa73f1";
constexpr std::size_t AL_R20_R63Z_DIMENSION = 102U;
constexpr std::size_t AL_R20_R63Z_WIDTH = 2U;
constexpr std::size_t AL_R20_R63Z_TANGENT_NONZEROS = 17748U;
constexpr std::size_t AL_R20_R63Z_TANGENT_CAPACITY = 32130U;

struct ALR20R63ZIdentity {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t width = 0U;
    std::string parent_semantic;
    std::string common_root;
    std::string tangent_root;
    std::string order;
    std::string root;
};

std::string al_r20_r63z_identity_root(const ALR20R63ZIdentity& identity) {
    std::ostringstream material;
    material << identity.dimension << ':' << identity.width << ':'
        << identity.parent_semantic << ':' << identity.common_root << ':'
        << identity.tangent_root << ':' << identity.order;
    return sha256_hex(material.str());
}

ALR20R63ZIdentity al_r20_r63z_identity(
    std::size_t dimension, const std::string& parent_semantic,
    const std::string& common_root, const std::string& tangent_root) {
    ALR20R63ZIdentity result;
    result.dimension = dimension;
    result.width = AL_R20_R63Z_WIDTH;
    result.parent_semantic = parent_semantic;
    result.common_root = common_root;
    result.tangent_root = tangent_root;
    result.order = "row,column,matrix_part,candidate_part";
    result.root = al_r20_r63z_identity_root(result);
    result.exact = dimension > 0U && !parent_semantic.empty()
        && !common_root.empty() && !tangent_root.empty();
    return result;
}

bool al_r20_r63z_identity_valid(const ALR20R63ZIdentity& identity) {
    return identity.exact && identity.dimension > 0U
        && identity.width == AL_R20_R63Z_WIDTH
        && identity.order == "row,column,matrix_part,candidate_part"
        && identity.root == al_r20_r63z_identity_root(identity);
}

struct ALR20R63ZArtifact {
    bool exact = false;
    bool contained = false;
    ALR20R63ZIdentity identity;
    std::size_t entries = 0U;
    std::size_t containments = 0U;
    std::size_t nonzero_lows = 0U;
    std::vector<double> components;
    std::vector<double> radius;
    double maximum_radius = 0.0;
    std::string component_root;
    std::string radius_root;
    std::string root;
};

std::string al_r20_r63z_artifact_root(const ALR20R63ZArtifact& artifact) {
    std::ostringstream material;
    material << artifact.identity.root << ':' << artifact.entries << ':'
        << artifact.containments << ':' << artifact.nonzero_lows << ':'
        << binary64_bits(artifact.maximum_radius) << ':'
        << artifact.component_root << ':' << artifact.radius_root;
    return sha256_hex(material.str());
}

bool al_r20_r63z_artifact_valid(const ALR20R63ZArtifact& artifact) {
    const std::size_t dimension = artifact.identity.dimension;
    if (!artifact.exact || !artifact.contained
        || !al_r20_r63z_identity_valid(artifact.identity)
        || artifact.entries != dimension * dimension
        || artifact.containments != artifact.entries
        || artifact.components.size()
            != artifact.entries * artifact.identity.width
        || artifact.radius.size() != artifact.entries
        || artifact.component_root != al_r20_double_root(artifact.components)
        || artifact.radius_root != al_r20_double_root(artifact.radius)
        || artifact.root != al_r20_r63z_artifact_root(artifact)
        || !al_r20_r63y_normal_or_zero(artifact.maximum_radius)) return false;
    for (double value : artifact.components)
        if (!al_r20_r63y_normal_or_zero(value)) return false;
    for (double value : artifact.radius)
        if (value < 0.0 || !al_r20_r63y_normal_or_zero(value)) return false;
    return true;
}

ALR20R63ZArtifact al_r20_r63z_artifact(
    const ALR20R63QExactMatrix& matrix,
    const ALR20R63ZIdentity& identity) {
    ALR20R63ZArtifact result;
    result.identity = identity;
    const std::size_t dimension = identity.dimension;
    if (!matrix.exact || matrix.rows != dimension
        || matrix.oracle_numerator.size() != dimension * dimension
        || al_r20_r63p_sign(matrix.d) <= 0
        || !al_r20_r63z_identity_valid(identity)
        || matrix.root != identity.common_root) return result;
    bool exact = true;
    result.radius.resize(dimension * dimension);
    result.components.reserve(
        dimension * dimension * identity.width);
    for (std::size_t index = 0U;
         index < dimension * dimension; ++index) {
        const ALR20R63YExpansion expansion = al_r20_r63y_project(
            matrix.oracle_numerator[index], matrix.d, identity.width);
        exact = exact && expansion.exact;
        result.components.insert(result.components.end(),
            expansion.components.begin(), expansion.components.end());
        result.radius[index] = expansion.radius;
        result.maximum_radius = std::max(
            result.maximum_radius, expansion.radius);
        result.containments += expansion.exact ? 1U : 0U;
        result.nonzero_lows += expansion.nonzero_low ? 1U : 0U;
        ++result.entries;
    }
    result.contained = exact
        && result.containments == dimension * dimension;
    result.component_root = al_r20_double_root(result.components);
    result.radius_root = al_r20_double_root(result.radius);
    result.exact = result.contained
        && result.components.size()
            == dimension * dimension * identity.width
        && al_r20_r63y_normal_or_zero(result.maximum_radius);
    result.root = al_r20_r63z_artifact_root(result);
    result.exact = result.exact && al_r20_r63z_artifact_valid(result);
    return result;
}

struct ALR20R63ZCandidate {
    bool exact = false;
    ALR20R63ZIdentity identity;
    std::string artifact_root;
    std::string lane;
    std::size_t state = 0U;
    ALR20R63YSolution solution;
    std::string root;
};

std::string al_r20_r63z_solution_root(const ALR20R63YSolution& solution) {
    std::ostringstream material;
    material << solution.exact << ':' << solution.width << ':'
        << solution.dimension << ':' << solution.containments << ':'
        << solution.nonzero_lows << ':'
        << al_r20_double_root(solution.components) << ':'
        << al_r20_double_root(solution.radius);
    return sha256_hex(material.str());
}

std::string al_r20_r63z_candidate_root(
    const ALR20R63ZCandidate& candidate) {
    return sha256_hex(candidate.identity.root + ":" + candidate.lane + ":"
        + std::to_string(candidate.state) + ":" + candidate.artifact_root
        + ":" + candidate.solution.root);
}

bool al_r20_r63z_candidate_valid(const ALR20R63ZCandidate& candidate) {
    const std::size_t dimension = candidate.identity.dimension;
    if (!candidate.exact || !al_r20_r63z_identity_valid(candidate.identity)
        || candidate.lane.empty()
        || candidate.artifact_root.empty()
        || !candidate.solution.exact
        || candidate.solution.width != candidate.identity.width
        || candidate.solution.dimension != dimension
        || candidate.solution.components.size()
            != dimension * candidate.identity.width
        || candidate.solution.radius.size() != dimension
        || candidate.solution.root
            != al_r20_r63z_solution_root(candidate.solution)
        || candidate.root != al_r20_r63z_candidate_root(candidate))
        return false;
    for (double value : candidate.solution.components)
        if (!al_r20_r63y_normal_or_zero(value)) return false;
    for (double value : candidate.solution.radius)
        if (value < 0.0 || !al_r20_r63y_normal_or_zero(value)) return false;
    return true;
}

ALR20R63ZCandidate al_r20_r63z_candidate(
    const ALR20R63ZArtifact& artifact, const std::string& lane,
    std::size_t state, const std::vector<Binary128>& source) {
    ALR20R63ZCandidate result;
    result.identity = artifact.identity;
    result.artifact_root = artifact.root;
    result.lane = lane;
    result.state = state;
    result.solution = al_r20_r63y_solution(
        source, artifact.identity.width);
    result.exact = al_r20_r63z_artifact_valid(artifact)
        && !lane.empty() && result.solution.exact;
    result.root = al_r20_r63z_candidate_root(result);
    result.exact = result.exact && al_r20_r63z_candidate_valid(result);
    return result;
}

struct ALR20R63ZProduct {
    bool exact = false;
    bool identity_rejected = false;
    std::size_t dimension = 0U;
    std::size_t width = 0U;
    std::size_t dots = 0U;
    std::size_t component_products = 0U;
    std::size_t radius_triplets = 0U;
    std::vector<double> center;
    std::vector<double> radius;
    double maximum_radius = 0.0;
    std::string identity_root;
    std::string candidate_root;
    std::string dot_root;
    std::string root;
};

ALR20R63ZProduct al_r20_r63z_product(
    const ALR20R63ZArtifact& artifact,
    const ALR20R63ZCandidate& candidate) {
    ALR20R63ZProduct result;
    result.dimension = artifact.identity.dimension;
    result.width = artifact.identity.width;
    result.identity_root = artifact.identity.root;
    result.candidate_root = candidate.root;
    if (!al_r20_r63z_artifact_valid(artifact)
        || !al_r20_r63z_candidate_valid(candidate)
        || artifact.identity.root != candidate.identity.root
        || artifact.root != candidate.artifact_root) {
        result.identity_rejected = true;
        return result;
    }
    const std::size_t dimension = result.dimension;
    const std::size_t width = result.width;
    const std::size_t dot_count = dimension * width * width;
    bool exact = true;
    result.center.resize(dimension);
    result.radius.resize(dimension);
    std::ostringstream dot_material;
    for (std::size_t row = 0U; row < dimension; ++row) {
        const ALR20R63Dot2Err dot = al_r20_r63_dot2err(
            dot_count,
            [&](std::size_t index) {
                std::size_t flat = index;
                const std::size_t xpart = flat % width;
                flat /= width;
                const std::size_t hpart = flat % width;
                const std::size_t column = flat / width;
                (void)xpart;
                return artifact.components[
                    (row * dimension + column) * width + hpart];
            },
            [&](std::size_t index) {
                std::size_t flat = index;
                const std::size_t xpart = flat % width;
                flat /= width;
                flat /= width;
                const std::size_t column = flat;
                return candidate.solution.components[
                    column * width + xpart];
            });
        ++result.dots;
        result.component_products += dot_count;
        exact = exact && dot.exact;
        double radius = dot.error;
        for (std::size_t column = 0U; column < dimension; ++column) {
            double h_abs = 0.0;
            double x_abs = 0.0;
            for (std::size_t part = 0U; part < width; ++part) {
                h_abs = al_r20_r63y_up_add(h_abs,
                    std::abs(artifact.components[
                        (row * dimension + column) * width + part]));
                x_abs = al_r20_r63y_up_add(x_abs,
                    std::abs(candidate.solution.components[
                        column * width + part]));
            }
            const double hr = artifact.radius[row * dimension + column];
            const double xr = candidate.solution.radius[column];
            radius = al_r20_r63y_up_add(radius,
                al_r20_r63y_up_multiply(h_abs, xr));
            radius = al_r20_r63y_up_add(radius,
                al_r20_r63y_up_multiply(hr, x_abs));
            radius = al_r20_r63y_up_add(radius,
                al_r20_r63y_up_multiply(hr, xr));
            result.radius_triplets += 1U;
        }
        result.center[row] = dot.result;
        result.radius[row] = radius;
        result.maximum_radius = std::max(result.maximum_radius, radius);
        exact = exact && radius >= 0.0
            && al_r20_r63y_normal_or_zero(dot.result)
            && al_r20_r63y_normal_or_zero(radius);
        dot_material << dot.root << ':' << binary64_bits(radius) << ';';
    }
    result.dot_root = sha256_hex(dot_material.str());
    result.exact = exact && result.dots == dimension
        && result.component_products == dimension * dot_count
        && result.radius_triplets == dimension * dimension
        && al_r20_r63y_normal_or_zero(result.maximum_radius);
    std::ostringstream material;
    material << result.exact << ':' << result.identity_rejected << ':'
        << result.dimension << ':' << result.width << ':' << result.dots
        << ':' << result.component_products << ':'
        << result.radius_triplets << ':'
        << binary64_bits(result.maximum_radius) << ':'
        << result.identity_root << ':' << result.candidate_root << ':'
        << result.dot_root << ':' << al_r20_double_root(result.center) << ':'
        << al_r20_double_root(result.radius);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63ZOracle {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t products = 0U;
    ALR20R37Dyadic d;
    std::vector<ALR20R37Dyadic> numerator;
    std::string root;
};

ALR20R63ZOracle al_r20_r63z_oracle(
    const ALR20R63QExactMatrix& matrix,
    const std::vector<Binary128>& solution) {
    ALR20R63ZOracle result;
    result.dimension = solution.size();
    result.d = matrix.d;
    const std::size_t dimension = result.dimension;
    if (!matrix.exact || matrix.rows != dimension || dimension == 0U
        || matrix.oracle_numerator.size() != dimension * dimension)
        return result;
    std::vector<ALR20R37Dyadic> exact_solution(dimension);
    bool conversions_exact = true;
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R37Conversion conversion =
            al_r20_r37_from_binary128(solution[index]);
        conversions_exact = conversions_exact && conversion.exact;
        exact_solution[index] = conversion.value;
    }
    result.numerator.resize(dimension);
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < dimension; ++column) {
            result.numerator[row] = al_r20_r37_add(result.numerator[row],
                al_r20_r37_multiply(
                    matrix.oracle_numerator[row * dimension + column],
                    exact_solution[column]));
            ++result.products;
        }
    result.exact = conversions_exact
        && result.products == dimension * dimension;
    result.root = sha256_hex(al_r20_r63q_vector_root(result.numerator) + ":"
        + al_r20_r37_dyadic_text(result.d));
    return result;
}

struct ALR20R63ZContainment {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t entries = 0U;
    std::string finite_root;
    std::string root;
};

ALR20R63ZContainment al_r20_r63z_containment(
    const ALR20R63ZProduct& finite,
    const ALR20R63ZOracle& oracle) {
    ALR20R63ZContainment result;
    result.dimension = finite.dimension;
    result.finite_root = finite.root;
    if (!finite.exact || !oracle.exact
        || oracle.dimension != result.dimension
        || finite.center.size() != result.dimension
        || finite.radius.size() != result.dimension
        || oracle.numerator.size() != result.dimension) return result;
    std::ostringstream material;
    for (std::size_t index = 0U; index < result.dimension; ++index) {
        const bool contained = al_r20_r63w_contains_fraction(
            static_cast<Binary128>(finite.center[index]),
            static_cast<Binary128>(finite.radius[index]),
            oracle.numerator[index], oracle.d);
        result.entries += contained ? 1U : 0U;
        material << index << ':' << contained << ';';
    }
    result.exact = result.entries == result.dimension;
    material << '|' << result.exact << ':' << result.dimension << ':'
        << result.entries << ':' << result.finite_root << ':' << oracle.root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63ZLane {
    bool exact = false;
    bool contained = false;
    std::size_t dimension = 0U;
    std::size_t products = 0U;
    std::size_t containments = 0U;
    std::vector<ALR20R63ZCandidate> candidates;
    std::vector<ALR20R63ZProduct> finite;
    std::vector<ALR20R63ZOracle> oracle;
    std::vector<ALR20R63ZContainment> audits;
    std::string root;
};

ALR20R63ZLane al_r20_r63z_lane(
    const ALR20R63ZArtifact& artifact, const std::string& lane,
    const std::vector<std::vector<Binary128>>& states,
    const ALR20R63QExactMatrix& matrix) {
    ALR20R63ZLane result;
    result.dimension = artifact.identity.dimension;
    if (states.size() != 3U || lane.empty()) return result;
    bool exact = true;
    bool contained = true;
    for (std::size_t state = 0U; state < 3U; ++state) {
        ALR20R63ZCandidate candidate = al_r20_r63z_candidate(
            artifact, lane, state, states[state]);
        ALR20R63ZProduct finite = al_r20_r63z_product(
            artifact, candidate);
        ALR20R63ZOracle oracle = al_r20_r63z_oracle(
            matrix, states[state]);
        ALR20R63ZContainment audit = al_r20_r63z_containment(
            finite, oracle);
        exact = exact && candidate.exact && finite.exact && oracle.exact;
        contained = contained && audit.exact;
        result.containments += audit.entries;
        ++result.products;
        result.candidates.push_back(std::move(candidate));
        result.finite.push_back(std::move(finite));
        result.oracle.push_back(std::move(oracle));
        result.audits.push_back(std::move(audit));
    }
    result.contained = contained
        && result.containments == 3U * result.dimension;
    result.exact = exact && result.contained && result.products == 3U;
    std::ostringstream material;
    material << result.exact << ':' << result.contained << ':'
        << result.dimension << ':' << result.products << ':'
        << result.containments << '|';
    for (std::size_t state = 0U; state < 3U; ++state)
        material << result.candidates[state].root << ':'
            << result.finite[state].root << ':' << result.oracle[state].root
            << ':' << result.audits[state].root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63z_classify(bool apparatus, bool identity,
    bool containment, bool retained, bool exported) {
    if (!apparatus) return "BLOCK_OPERATOR_APPARATUS_REJECTED";
    if (!identity) return "BLOCK_OPERATOR_IDENTITY_REJECTED";
    if (!containment) return "BLOCK_OPERATOR_CONTAINMENT_REJECTED";
    if (!retained) return "RETAINED_BLOCK_OPERATOR_REJECTED";
    if (!exported) return "EXPORTED_BLOCK_OPERATOR_REJECTED";
    return "TWOFOLD_BLOCK_DENSE_OPERATOR_CANDIDATE";
}

struct ALR20R63ZControls {
    bool exact = false;
    bool identity = false;
    bool product = false;
    bool cancellation = false;
    bool coefficient_radius = false;
    bool candidate_radius = false;
    bool dropped_radii = false;
    bool orientation = false;
    bool invalid = false;
    bool overflow = false;
    bool stale = false;
    bool mutation = false;
    bool oracle_independence = false;
    bool classifier = false;
    std::string root;
};

struct ALR20R63ZLastResult {
    bool exact = false;
    std::string route;
    std::string semantic;
    std::string artifact_root;
    std::string identity_root;
};

ALR20R63ZLastResult al_r20_r63z_last_result;

ALR20R63QExactMatrix al_r20_r63z_control_matrix(
    const std::vector<ALR20R37Dyadic>& numerator,
    const ALR20R37Dyadic& d, std::size_t dimension) {
    ALR20R63QExactMatrix result;
    result.exact = numerator.size() == dimension * dimension
        && al_r20_r63p_sign(d) > 0;
    result.rows = dimension;
    result.columns = dimension;
    result.d = d;
    result.oracle_numerator = numerator;
    result.oracle_root = al_r20_r63q_vector_root(numerator);
    result.root = sha256_hex(result.oracle_root + ":"
        + al_r20_r37_dyadic_text(d) + ":" + std::to_string(dimension));
    return result;
}

ALR20R63ZControls al_r20_r63z_controls() {
    ALR20R63ZControls result;
    const ALR20R37Dyadic one = al_r20_r63p_control_dyadic(1);
    const ALR20R37Dyadic three = al_r20_r63p_control_dyadic(3);
    const ALR20R63QExactMatrix scalar_matrix =
        al_r20_r63z_control_matrix({one}, three, 1U);
    const ALR20R63ZIdentity identity = al_r20_r63z_identity(
        1U, "control-parent", scalar_matrix.root, "control-tangent");
    const ALR20R63ZArtifact artifact = al_r20_r63z_artifact(
        scalar_matrix, identity);
    const std::vector<Binary128> scalar_solution{
        static_cast<Binary128>(1.0) / static_cast<Binary128>(3.0)};
    const ALR20R63ZCandidate candidate = al_r20_r63z_candidate(
        artifact, "control", 0U, scalar_solution);
    const ALR20R63ZProduct product = al_r20_r63z_product(
        artifact, candidate);
    const ALR20R63ZOracle oracle = al_r20_r63z_oracle(
        scalar_matrix, scalar_solution);
    const ALR20R63ZContainment containment = al_r20_r63z_containment(
        product, oracle);
    result.identity = al_r20_r63z_identity_valid(identity)
        && artifact.identity.root == candidate.identity.root;
    result.product = artifact.exact && candidate.exact && product.exact
        && oracle.exact && containment.exact;
    result.coefficient_radius = artifact.maximum_radius > 0.0;
    result.candidate_radius = candidate.solution.radius.size() == 1U
        && candidate.solution.radius[0U] > 0.0;
    const ALR20R63YExpansion coefficient = al_r20_r63y_project(
        one, three, 2U);
    ALR20R63YExpansion dropped_coefficient = coefficient;
    dropped_coefficient.radius = 0.0;
    const ALR20R37Conversion scalar_exact =
        al_r20_r37_from_binary128(scalar_solution[0U]);
    const ALR20R63YExpansion projected_candidate = scalar_exact.exact
        ? al_r20_r63y_project(scalar_exact.value, one, 2U)
        : ALR20R63YExpansion{};
    ALR20R63YExpansion dropped_candidate = projected_candidate;
    dropped_candidate.radius = 0.0;
    result.dropped_radii = coefficient.exact && projected_candidate.exact
        && !al_r20_r63y_expansion_contains(
            dropped_coefficient, one, three)
        && !al_r20_r63y_expansion_contains(
            dropped_candidate, scalar_exact.value, one);

    const ALR20R37Dyadic minus_three =
        al_r20_r63p_control_dyadic(-3);
    const ALR20R63QExactMatrix cancellation_matrix =
        al_r20_r63z_control_matrix(
            {three, minus_three, three, three}, three, 2U);
    const ALR20R63ZIdentity cancellation_identity = al_r20_r63z_identity(
        2U, "control-parent", cancellation_matrix.root,
        "control-tangent");
    const ALR20R63ZArtifact cancellation_artifact =
        al_r20_r63z_artifact(cancellation_matrix, cancellation_identity);
    const std::vector<Binary128> cancellation_source{
        static_cast<Binary128>(1.0),
        nextafterq(static_cast<Binary128>(1.0), HUGE_VALQ)};
    const ALR20R63ZCandidate cancellation_candidate =
        al_r20_r63z_candidate(cancellation_artifact, "cancel", 0U,
            cancellation_source);
    const ALR20R63ZProduct cancellation_product = al_r20_r63z_product(
        cancellation_artifact, cancellation_candidate);
    const ALR20R63ZOracle cancellation_oracle = al_r20_r63z_oracle(
        cancellation_matrix, cancellation_source);
    const ALR20R63ZContainment cancellation_containment =
        al_r20_r63z_containment(
            cancellation_product, cancellation_oracle);
    result.cancellation = cancellation_product.exact
        && cancellation_containment.exact
        && std::abs(cancellation_product.center[0U])
            < std::abs(cancellation_product.center[1U]);

    const ALR20R63QExactMatrix transposed_matrix =
        al_r20_r63z_control_matrix(
            {three, three, minus_three, three}, three, 2U);
    const ALR20R63ZIdentity transposed_identity = al_r20_r63z_identity(
        2U, "control-parent", transposed_matrix.root, "control-tangent");
    const ALR20R63ZArtifact transposed_artifact =
        al_r20_r63z_artifact(transposed_matrix, transposed_identity);
    const ALR20R63ZCandidate transposed_candidate = al_r20_r63z_candidate(
        transposed_artifact, "cancel", 0U, cancellation_source);
    const ALR20R63ZProduct transposed_product = al_r20_r63z_product(
        transposed_artifact, transposed_candidate);
    result.orientation = transposed_artifact.exact
        && transposed_product.exact
        && transposed_artifact.root != cancellation_artifact.root
        && transposed_product.root != cancellation_product.root;

    ALR20R63ZArtifact invalid_artifact = artifact;
    invalid_artifact.radius[0U] = -1.0;
    invalid_artifact.radius_root = al_r20_double_root(invalid_artifact.radius);
    invalid_artifact.root = al_r20_r63z_artifact_root(invalid_artifact);
    const ALR20R63ZProduct invalid_product = al_r20_r63z_product(
        invalid_artifact, candidate);
    auto artifact_component_case = [&](double value) {
        ALR20R63ZArtifact changed = artifact;
        changed.components[0U] = value;
        changed.component_root = al_r20_double_root(changed.components);
        changed.root = al_r20_r63z_artifact_root(changed);
        return changed;
    };
    const ALR20R63ZArtifact subnormal_artifact = artifact_component_case(
        std::numeric_limits<double>::denorm_min());
    const ALR20R63ZArtifact infinite_artifact = artifact_component_case(
        std::numeric_limits<double>::infinity());
    const ALR20R63ZArtifact nan_artifact = artifact_component_case(
        std::numeric_limits<double>::quiet_NaN());
    const ALR20R63ZProduct subnormal_product = al_r20_r63z_product(
        subnormal_artifact, candidate);
    const ALR20R63ZProduct infinite_product = al_r20_r63z_product(
        infinite_artifact, candidate);
    const ALR20R63ZProduct nan_product = al_r20_r63z_product(
        nan_artifact, candidate);
    ALR20R63ZCandidate negative_candidate = candidate;
    negative_candidate.solution.radius[0U] = -1.0;
    negative_candidate.solution.root = al_r20_r63z_solution_root(
        negative_candidate.solution);
    negative_candidate.root = al_r20_r63z_candidate_root(
        negative_candidate);
    const ALR20R63ZProduct negative_candidate_product =
        al_r20_r63z_product(artifact, negative_candidate);
    auto rejected_without_dots = [](const ALR20R63ZProduct& value) {
        return value.identity_rejected && value.dots == 0U;
    };
    result.invalid = rejected_without_dots(invalid_product)
        && rejected_without_dots(subnormal_product)
        && rejected_without_dots(infinite_product)
        && rejected_without_dots(nan_product)
        && rejected_without_dots(negative_candidate_product);

    ALR20R63ZArtifact overflow_artifact = artifact;
    overflow_artifact.components[0U] =
        std::numeric_limits<double>::max();
    overflow_artifact.components[1U] = 0.0;
    overflow_artifact.radius[0U] = 0.0;
    overflow_artifact.nonzero_lows = 0U;
    overflow_artifact.maximum_radius = 0.0;
    overflow_artifact.component_root = al_r20_double_root(
        overflow_artifact.components);
    overflow_artifact.radius_root = al_r20_double_root(
        overflow_artifact.radius);
    overflow_artifact.root = al_r20_r63z_artifact_root(
        overflow_artifact);
    const ALR20R63ZCandidate overflow_candidate = al_r20_r63z_candidate(
        overflow_artifact, "overflow", 0U,
        {static_cast<Binary128>(std::numeric_limits<double>::max())});
    const ALR20R63ZProduct overflow_product = al_r20_r63z_product(
        overflow_artifact, overflow_candidate);
    result.overflow = !overflow_product.exact
        && !overflow_product.identity_rejected
        && overflow_product.dots == 1U;

    auto stale_identity_product = [&](auto mutate) {
        ALR20R63ZCandidate changed = candidate;
        mutate(changed.identity);
        changed.identity.root = al_r20_r63z_identity_root(changed.identity);
        changed.root = al_r20_r63z_candidate_root(changed);
        return al_r20_r63z_product(artifact, changed);
    };
    const ALR20R63ZProduct stale_face = stale_identity_product(
        [](ALR20R63ZIdentity& value) {
            value.parent_semantic = "stale-face-parent";
        });
    const ALR20R63ZProduct stale_common = stale_identity_product(
        [](ALR20R63ZIdentity& value) {
            value.common_root = "stale-common";
        });
    const ALR20R63ZProduct stale_tangent = stale_identity_product(
        [](ALR20R63ZIdentity& value) {
            value.tangent_root = "stale-tangent";
        });
    const ALR20R63ZProduct stale_width = stale_identity_product(
        [](ALR20R63ZIdentity& value) { value.width = 3U; });
    const ALR20R63ZProduct stale_order = stale_identity_product(
        [](ALR20R63ZIdentity& value) { value.order = "stale-order"; });
    ALR20R63ZCandidate stale_lane_candidate = candidate;
    stale_lane_candidate.lane = "stale-lane";
    const ALR20R63ZProduct stale_lane = al_r20_r63z_product(
        artifact, stale_lane_candidate);
    ALR20R63ZCandidate stale_state_candidate = candidate;
    stale_state_candidate.state = 1U;
    const ALR20R63ZProduct stale_state = al_r20_r63z_product(
        artifact, stale_state_candidate);
    result.stale = rejected_without_dots(stale_face)
        && rejected_without_dots(stale_common)
        && rejected_without_dots(stale_tangent)
        && rejected_without_dots(stale_width)
        && rejected_without_dots(stale_order)
        && rejected_without_dots(stale_lane)
        && rejected_without_dots(stale_state);

    ALR20R63ZArtifact mutated_high = artifact;
    mutated_high.components[0U] = std::nextafter(
        mutated_high.components[0U],
        std::numeric_limits<double>::infinity());
    ALR20R63ZArtifact mutated_low = artifact;
    mutated_low.components[1U] = std::nextafter(
        mutated_low.components[1U],
        std::numeric_limits<double>::infinity());
    ALR20R63ZArtifact mutated_radius = artifact;
    mutated_radius.radius[0U] = std::nextafter(
        mutated_radius.radius[0U],
        std::numeric_limits<double>::infinity());
    ALR20R63ZArtifact mutated_root = artifact;
    mutated_root.root = "mutated-root";
    const ALR20R63ZProduct mutation_high = al_r20_r63z_product(
        mutated_high, candidate);
    const ALR20R63ZProduct mutation_low = al_r20_r63z_product(
        mutated_low, candidate);
    const ALR20R63ZProduct mutation_radius = al_r20_r63z_product(
        mutated_radius, candidate);
    const ALR20R63ZProduct mutation_root = al_r20_r63z_product(
        mutated_root, candidate);
    ALR20R63ZArtifact resealed_artifact = artifact;
    resealed_artifact.radius[0U] = std::nextafter(
        resealed_artifact.radius[0U],
        std::numeric_limits<double>::infinity());
    resealed_artifact.maximum_radius = resealed_artifact.radius[0U];
    resealed_artifact.radius_root = al_r20_double_root(
        resealed_artifact.radius);
    resealed_artifact.root = al_r20_r63z_artifact_root(resealed_artifact);
    const ALR20R63ZProduct resealed_mutation = al_r20_r63z_product(
        resealed_artifact, candidate);
    result.mutation = rejected_without_dots(mutation_high)
        && rejected_without_dots(mutation_low)
        && rejected_without_dots(mutation_radius)
        && rejected_without_dots(mutation_root)
        && rejected_without_dots(resealed_mutation);

    ALR20R63ZOracle mutated_oracle = oracle;
    mutated_oracle.numerator[0U] = al_r20_r37_add(
        mutated_oracle.numerator[0U], al_r20_r63p_control_dyadic(16));
    mutated_oracle.root = sha256_hex(
        al_r20_r63q_vector_root(mutated_oracle.numerator) + ":"
        + al_r20_r37_dyadic_text(mutated_oracle.d));
    const ALR20R63ZContainment rejected_oracle =
        al_r20_r63z_containment(product, mutated_oracle);
    const ALR20R63ZProduct repeated = al_r20_r63z_product(
        artifact, candidate);
    result.oracle_independence = !rejected_oracle.exact
        && repeated.root == product.root;
    result.classifier =
        al_r20_r63z_classify(false, false, false, false, false)
            == "BLOCK_OPERATOR_APPARATUS_REJECTED"
        && al_r20_r63z_classify(true, false, false, false, false)
            == "BLOCK_OPERATOR_IDENTITY_REJECTED"
        && al_r20_r63z_classify(true, true, false, false, false)
            == "BLOCK_OPERATOR_CONTAINMENT_REJECTED"
        && al_r20_r63z_classify(true, true, true, false, true)
            == "RETAINED_BLOCK_OPERATOR_REJECTED"
        && al_r20_r63z_classify(true, true, true, true, false)
            == "EXPORTED_BLOCK_OPERATOR_REJECTED"
        && al_r20_r63z_classify(true, true, true, true, true)
            == "TWOFOLD_BLOCK_DENSE_OPERATOR_CANDIDATE";
    result.exact = result.identity && result.product
        && result.cancellation && result.coefficient_radius
        && result.candidate_radius && result.dropped_radii
        && result.orientation && result.invalid && result.overflow
        && result.stale
        && result.mutation && result.oracle_independence
        && result.classifier;
    std::ostringstream material;
    material << result.exact << ':' << result.identity << ':'
        << result.product << ':' << result.cancellation << ':'
        << result.coefficient_radius << ':' << result.candidate_radius << ':'
        << result.dropped_radii << ':' << result.orientation << ':'
        << result.invalid << ':' << result.overflow << ':' << result.stale
        << ':' << result.mutation
        << ':' << result.oracle_independence << ':' << result.classifier
        << ':' << identity.root << ':' << artifact.root << ':'
        << candidate.root << ':' << product.root << ':' << containment.root
        << ':' << cancellation_artifact.root << ':'
        << cancellation_product.root << ':' << transposed_artifact.root << ':'
        << transposed_product.root << ':'
        << al_r20_double_root(subnormal_artifact.components) << ':'
        << al_r20_double_root(infinite_artifact.components) << ':'
        << al_r20_double_root(nan_artifact.components) << ':'
        << overflow_product.root << ':'
        << al_r20_double_root(mutated_high.components) << ':'
        << al_r20_double_root(mutated_low.components) << ':'
        << al_r20_double_root(mutated_radius.radius) << ':'
        << resealed_artifact.root << ':' << rejected_oracle.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_active_block_operator_controls() {
    al_r20_r63z_last_result = ALR20R63ZLastResult{};
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
    const ALR20R63ZControls controls = al_r20_r63z_controls();
    const SplitBoundaryReport parent =
        run_al_generalization_v5_portable_expansion_affine_controls();
    const ALR20R63YLastResult parent_result = al_r20_r63y_last_result;
    const ALR20R63XFixture fixture = al_r20_r63x_last_fixture;
    const bool parent_exact = parent.passed && parent_result.exact
        && parent_result.selected_width == 2U
        && parent_result.route == "TWOFOLD_AFFINE_IMAGE_CANDIDATE"
        && parent_result.semantic == AL_R20_R63Z_PARENT_SEMANTIC_SHA256
        && fixture.exact && fixture.matrix.exact
        && fixture.matrix.rows == AL_R20_R63Z_DIMENSION;
    const ALR20R63ZIdentity identity = parent_exact
        ? al_r20_r63z_identity(AL_R20_R63Z_DIMENSION,
            parent_result.semantic, fixture.matrix.root,
            AL_R20_R63Z_TANGENT_SHA256)
        : ALR20R63ZIdentity{};
    const ALR20R63ZArtifact artifact = parent_exact
        ? al_r20_r63z_artifact(fixture.matrix, identity)
        : ALR20R63ZArtifact{};
    const ALR20R63ZLane retained = artifact.exact
        ? al_r20_r63z_lane(artifact, "retained",
            fixture.wide_solutions, fixture.matrix)
        : ALR20R63ZLane{};
    const ALR20R63ZLane exported = artifact.exact
        ? al_r20_r63z_lane(artifact, "exported",
            fixture.exported_solutions, fixture.matrix)
        : ALR20R63ZLane{};
    const bool identity_exact = al_r20_r63z_identity_valid(identity)
        && artifact.identity.root == identity.root;
    const std::size_t containments = retained.containments
        + exported.containments;
    const bool containment_exact = retained.contained && exported.contained
        && containments == 612U;
    auto lane_work_exact = [](const ALR20R63ZLane& lane) {
        if (!lane.exact || lane.finite.size() != 3U
            || lane.oracle.size() != 3U) return false;
        std::size_t component_products = 0U;
        std::size_t radius_triplets = 0U;
        std::size_t oracle_products = 0U;
        for (std::size_t state = 0U; state < 3U; ++state) {
            component_products += lane.finite[state].component_products;
            radius_triplets += lane.finite[state].radius_triplets;
            oracle_products += lane.oracle[state].products;
        }
        return component_products == 124848U
            && radius_triplets == 31212U
            && oracle_products == 31212U;
    };
    const bool work_exact = artifact.entries == 10404U
        && retained.products == 3U && exported.products == 3U
        && lane_work_exact(retained) && lane_work_exact(exported);
    const bool apparatus_exact = profile_exact && controls.exact
        && parent_exact && artifact.exact && work_exact;
    const std::string route = al_r20_r63z_classify(apparatus_exact,
        identity_exact, containment_exact, retained.exact, exported.exact);
    const bool exact = apparatus_exact && identity_exact
        && containment_exact && retained.exact && exported.exact;
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63z-active-block-operator.v1|"
        << parent_result.semantic << ':' << route << ':' << controls.root
        << ':' << identity.root << ':' << artifact.root << ':'
        << retained.root << ':' << exported.root;
    const std::string result_sha = sha256_hex(semantic.str());
    al_r20_r63z_last_result.exact = exact;
    al_r20_r63z_last_result.route = route;
    al_r20_r63z_last_result.semantic = result_sha;
    al_r20_r63z_last_result.artifact_root = artifact.root;
    al_r20_r63z_last_result.identity_root = identity.root;
    auto lane_json = [](const char* name, const ALR20R63ZLane& lane) {
        std::ostringstream stream;
        stream << std::setprecision(17)
            << "\"" << name << "\":{\"exact\":"
            << (lane.exact ? "true" : "false")
            << ",\"contained\":" << (lane.contained ? "true" : "false")
            << ",\"states\":[";
        for (std::size_t state = 0U; state < lane.finite.size(); ++state) {
            if (state != 0U) stream << ',';
            stream << "{\"state\":" << state
                << ",\"maximum_radius\":"
                << lane.finite[state].maximum_radius
                << ",\"contained\":"
                << (lane.audits[state].exact ? "true" : "false")
                << ",\"root\":\"" << lane.finite[state].root << "\"}";
        }
        stream << "],\"containments\":" << lane.containments
            << ",\"root\":\"" << lane.root << "\"}";
        return stream.str();
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63z_active_block_operator.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63y_sha256\":\"" << parent_result.semantic << "\""
        << ",\"parent_stdout_sha256\":\""
        << AL_R20_R63Z_PARENT_STDOUT_SHA256 << "\""
        << ",\"controls_exact\":"
        << (controls.exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"controls\":{\"identity\":"
        << (controls.identity ? "true" : "false")
        << ",\"product\":" << (controls.product ? "true" : "false")
        << ",\"cancellation\":"
        << (controls.cancellation ? "true" : "false")
        << ",\"coefficient_radius\":"
        << (controls.coefficient_radius ? "true" : "false")
        << ",\"candidate_radius\":"
        << (controls.candidate_radius ? "true" : "false")
        << ",\"dropped_radii\":"
        << (controls.dropped_radii ? "true" : "false")
        << ",\"orientation\":"
        << (controls.orientation ? "true" : "false")
        << ",\"invalid\":" << (controls.invalid ? "true" : "false")
        << ",\"overflow\":"
        << (controls.overflow ? "true" : "false")
        << ",\"stale\":" << (controls.stale ? "true" : "false")
        << ",\"mutation\":"
        << (controls.mutation ? "true" : "false")
        << ",\"oracle_independence\":"
        << (controls.oracle_independence ? "true" : "false")
        << ",\"classifier\":"
        << (controls.classifier ? "true" : "false") << '}'
        << ",\"identity\":{\"exact\":"
        << (identity_exact ? "true" : "false")
        << ",\"root\":\"" << identity.root << "\"}"
        << ",\"artifact\":{\"exact\":"
        << (artifact.exact ? "true" : "false")
        << ",\"entries\":" << artifact.entries
        << ",\"nonzero_lows\":" << artifact.nonzero_lows
        << ",\"maximum_radius\":" << artifact.maximum_radius
        << ",\"root\":\"" << artifact.root << "\"}"
        << ",\"structure\":{\"tangent_nonzeros\":"
        << AL_R20_R63Z_TANGENT_NONZEROS
        << ",\"tangent_capacity\":" << AL_R20_R63Z_TANGENT_CAPACITY
        << ",\"common_capacity\":10404"
        << ",\"two_stage_coefficient_visits\":35496"
        << ",\"block_coefficient_visits\":10404"
        << ",\"visit_ratio\":3.4117647058823528}"
        << ",\"lanes\":{" << lane_json("retained", retained) << ','
        << lane_json("exported", exported) << '}'
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"identity_exact\":"
        << (identity_exact ? "true" : "false")
        << ",\"containment_exact\":"
        << (containment_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"finite_products\":612"
        << ",\"finite_component_products\":249696"
        << ",\"radius_triplets\":62424"
        << ",\"exact_oracle_products\":62424"
        << ",\"exact_containments\":" << containments
        << ",\"candidate_updates\":0,\"pcg_scalars\":0"
        << ",\"preconditioner_solves\":0,\"sparse_builds\":0"
        << ",\"exact_oracle_classification_uses\":0"
        << ",\"timing_admitted\":false,\"runtime_authority\":false"
        << ",\"gpu_authority\":false,\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
