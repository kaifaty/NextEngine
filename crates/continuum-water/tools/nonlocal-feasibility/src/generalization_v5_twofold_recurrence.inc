namespace {

constexpr const char* AL_R20_R63ZB_PARENT_SEMANTIC_SHA256 =
    "797cfbf4d6a96e76e7d099096711aad3468f2ab10cb2ada854168e621837124b";
constexpr const char* AL_R20_R63ZB_PARENT_STDOUT_SHA256 =
    "ed2511615eb0b488b4337133860971f86a0d7ad48c757db49c5fcda32b325f72";
constexpr const char* AL_R20_R63ZB_PARENT_IDENTITY_SHA256 =
    "78ce6fc93f9dc5d2f979eb1d401d9ee883c7d469571e16799baf73cce2700dc1";
constexpr const char* AL_R20_R63ZB_FIXTURE_SHA256 =
    "05e520b948d93029d18f6b2b34e87b356ab2dbd2f0a354b55a10f3920fbc1c8e";
constexpr const char* AL_R20_R63ZB_OPERATOR_SEMANTIC_SHA256 =
    "e27ee8616c1a34a1a224f65f32d5cebe1cd6dac973ccc2ae89c3fbad8343e4be";
constexpr const char* AL_R20_R63ZB_OPERATOR_ARTIFACT_SHA256 =
    "38cd1871eaf35c2153b469f4e98bba15754baa3853ceb76fdecc40d2d0146de1";
constexpr const char* AL_R20_R63ZB_VERIFIER_SEMANTIC_SHA256 =
    "ad3d84abaecc5142f05d369d0e653533c5e152cac2a9112efb2ad8277d70fb9d";
constexpr const char* AL_R20_R63ZB_VERIFIER_PROFILE_SHA256 =
    "b1c430444e04e23d282874ae08130ea27d5eedb951d551151cc09423ca90a291";
constexpr const char* AL_R20_R63ZB_SIGN_SHA256 =
    "89b2908b21369cf77a376287ed8142026827815c45a59aa70d2e831aac406094";
constexpr std::size_t AL_R20_R63ZB_DIMENSION = 102U;

bool al_r20_r63zb_operand(const ALR20R63ZAScalar& value) {
    return al_r20_r63za_canonical(value)
        && binary64_bits(value.radius) == binary64_bits(0.0);
}

ALR20R63ZAScalar al_r20_r63zb_center(
    const ALR20R63ZAScalar& value) {
    if (!al_r20_r63za_canonical(value)) return ALR20R63ZAScalar{};
    return al_r20_r63za_seal(value.high, value.low, 0.0);
}

ALR20R63ZAScalar al_r20_r63zb_project(Binary128 value) {
    return al_r20_r63zb_center(al_r20_r63za_project(value));
}

ALR20R63ZAScalar al_r20_r63zb_add(
    const ALR20R63ZAScalar& left, const ALR20R63ZAScalar& right,
    double right_sign = 1.0) {
    if (!al_r20_r63zb_operand(left) || !al_r20_r63zb_operand(right))
        return ALR20R63ZAScalar{};
    return al_r20_r63zb_center(
        al_r20_r63za_add(left, right, right_sign));
}

ALR20R63ZAScalar al_r20_r63zb_multiply(
    const ALR20R63ZAScalar& left, const ALR20R63ZAScalar& right) {
    if (!al_r20_r63zb_operand(left) || !al_r20_r63zb_operand(right))
        return ALR20R63ZAScalar{};
    return al_r20_r63zb_center(al_r20_r63za_multiply(left, right));
}

ALR20R63ZAScalar al_r20_r63zb_divide(
    const ALR20R63ZAScalar& numerator,
    const ALR20R63ZAScalar& denominator) {
    if (!al_r20_r63zb_operand(numerator)
        || !al_r20_r63zb_operand(denominator))
        return ALR20R63ZAScalar{};
    return al_r20_r63zb_center(
        al_r20_r63za_divide(numerator, denominator));
}

bool al_r20_r63zb_positive(const ALR20R63ZAScalar& value) {
    return al_r20_r63zb_operand(value) && value.high > 0.0;
}

std::string al_r20_r63zb_vector_root(
    const std::vector<ALR20R63ZAScalar>& values) {
    std::ostringstream material;
    material << values.size() << '|';
    for (const ALR20R63ZAScalar& value : values)
        material << value.root << ';';
    return sha256_hex(material.str());
}

bool al_r20_r63zb_vector_valid(
    const std::vector<ALR20R63ZAScalar>& values,
    std::size_t dimension) {
    return values.size() == dimension
        && std::all_of(values.begin(), values.end(),
            [](const ALR20R63ZAScalar& value) {
                return al_r20_r63zb_operand(value);
            });
}

struct ALR20R63ZBSolve {
    bool exact = false;
    std::string role;
    std::size_t dimension = 0U;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::string factor_root;
    std::string permutation_root;
    std::string inverse_scale_root;
    std::string source_root;
    std::vector<ALR20R63ZAScalar> intermediate;
    std::vector<ALR20R63ZAScalar> solution;
    std::string root;
};

ALR20R63ZBSolve al_r20_r63zb_solve(
    const std::string& role, const std::vector<double>& upper,
    const std::vector<std::size_t>& permutation,
    const ALR20R63ZAScalar& inverse_scale,
    const std::vector<ALR20R63ZAScalar>& source) {
    ALR20R63ZBSolve result;
    result.role = role;
    result.dimension = source.size();
    result.factor_root = al_r20_double_root(upper);
    result.permutation_root = al_r20_r63g_index_root(permutation);
    result.inverse_scale_root = inverse_scale.root;
    result.source_root = al_r20_r63zb_vector_root(source);
    const std::size_t dimension = result.dimension;
    if (role.empty() || dimension == 0U
        || upper.size() != dimension * dimension
        || !al_r20_r63g_permutation_exact(permutation, dimension)
        || !al_r20_r63zb_operand(inverse_scale)
        || !al_r20_r63zb_vector_valid(source, dimension)) return result;
    for (double value : upper)
        if (!al_r20_r63y_normal_or_zero(value)) return result;
    result.intermediate.resize(dimension);
    std::vector<ALR20R63ZAScalar> permuted_solution(dimension);
    result.solution.resize(dimension);
    bool exact = true;
    for (std::size_t row = 0U; row < dimension && exact; ++row) {
        ALR20R63ZAScalar numerator = al_r20_r63zb_multiply(
            source[permutation[row]], inverse_scale);
        for (std::size_t column = 0U; column < row; ++column) {
            const ALR20R63ZAScalar term = al_r20_r63zb_multiply(
                al_r20_r63za_exact_double(
                    upper[column * dimension + row]),
                result.intermediate[column]);
            numerator = al_r20_r63zb_add(numerator, term, -1.0);
            ++result.forward_terms;
        }
        result.intermediate[row] = al_r20_r63zb_divide(numerator,
            al_r20_r63za_exact_double(upper[row * dimension + row]));
        exact = al_r20_r63zb_operand(result.intermediate[row]);
        ++result.divisions;
    }
    for (std::size_t reverse = dimension; reverse-- > 0U && exact;) {
        ALR20R63ZAScalar numerator = result.intermediate[reverse];
        for (std::size_t column = reverse + 1U;
             column < dimension; ++column) {
            const ALR20R63ZAScalar term = al_r20_r63zb_multiply(
                al_r20_r63za_exact_double(
                    upper[reverse * dimension + column]),
                permuted_solution[column]);
            numerator = al_r20_r63zb_add(numerator, term, -1.0);
            ++result.backward_terms;
        }
        permuted_solution[reverse] = al_r20_r63zb_divide(numerator,
            al_r20_r63za_exact_double(
                upper[reverse * dimension + reverse]));
        exact = al_r20_r63zb_operand(permuted_solution[reverse]);
        ++result.divisions;
    }
    for (std::size_t index = 0U; index < dimension; ++index)
        result.solution[permutation[index]] = permuted_solution[index];
    result.exact = exact
        && al_r20_r63zb_vector_valid(result.intermediate, dimension)
        && al_r20_r63zb_vector_valid(result.solution, dimension)
        && result.forward_terms == dimension * (dimension - 1U) / 2U
        && result.backward_terms == dimension * (dimension - 1U) / 2U
        && result.divisions == 2U * dimension;
    std::ostringstream material;
    material << result.exact << ':' << result.role << ':'
        << result.dimension << ':' << result.forward_terms << ':'
        << result.backward_terms << ':' << result.divisions << ':'
        << result.factor_root << ':' << result.permutation_root << ':'
        << result.inverse_scale_root << ':' << result.source_root << ':'
        << al_r20_r63zb_vector_root(result.intermediate) << ':'
        << al_r20_r63zb_vector_root(result.solution);
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63zb_profile_root(
    const ALR20R63YProfile& profile) {
    std::ostringstream material;
    material << profile.exact << ':' << profile.contained << ':'
        << profile.contractive << ':' << profile.width << ':'
        << profile.dimension << ':' << profile.vector_containments << ':'
        << profile.matrix_containments << ':'
        << profile.nonzero_vector_lows << ':'
        << profile.nonzero_matrix_lows << ':'
        << binary64_bits(profile.rho_upper) << ':'
        << binary64_bits(profile.maximum_vector_radius) << ':'
        << binary64_bits(profile.maximum_matrix_radius) << ':'
        << profile.exact_root << ':' << profile.component_root << ':'
        << profile.radius_root;
    return sha256_hex(material.str());
}

bool al_r20_r63zb_profile_valid(const ALR20R63YProfile& profile) {
    if (!profile.exact || !profile.contained || !profile.contractive
        || profile.width != 2U
        || profile.dimension != AL_R20_R63ZB_DIMENSION
        || profile.vector_containments != profile.dimension
        || profile.matrix_containments
            != profile.dimension * profile.dimension
        || profile.vector_components.size()
            != profile.dimension * profile.width
        || profile.vector_radius.size() != profile.dimension
        || profile.matrix_components.size()
            != profile.dimension * profile.dimension * profile.width
        || profile.matrix_radius.size()
            != profile.dimension * profile.dimension
        || profile.component_root != sha256_hex(
            al_r20_double_root(profile.vector_components) + ":"
            + al_r20_double_root(profile.matrix_components))
        || profile.radius_root != sha256_hex(
            al_r20_double_root(profile.vector_radius) + ":"
            + al_r20_double_root(profile.matrix_radius))
        || profile.root != al_r20_r63zb_profile_root(profile)
        || profile.root != AL_R20_R63ZB_VERIFIER_PROFILE_SHA256)
        return false;
    for (double value : profile.vector_components)
        if (!al_r20_r63y_normal_or_zero(value)) return false;
    for (double value : profile.matrix_components)
        if (!al_r20_r63y_normal_or_zero(value)) return false;
    for (double value : profile.vector_radius)
        if (value < 0.0 || !al_r20_r63y_normal_or_zero(value)) return false;
    for (double value : profile.matrix_radius)
        if (value < 0.0 || !al_r20_r63y_normal_or_zero(value)) return false;
    return profile.rho_upper >= 0.0 && profile.rho_upper < 1.0;
}

bool al_r20_r63zb_fixture_valid(
    const ALR20R63SConsumptionFixture& fixture) {
    if (!fixture.exact || fixture.dimension != AL_R20_R63ZB_DIMENSION
        || al_r20_double_root(fixture.exported_upper)
            != AL_R20_R63ZA_EXPORTED_FACTOR_SHA256
        || al_r20_r63g_index_root(fixture.permutation)
            != AL_R20_R63ZA_PERMUTATION_SHA256
        || !(fixture.inverse_scale > static_cast<Binary128>(0.0))
        || finiteq(fixture.inverse_scale) == 0
        || fixture.sources.size() != 3U || fixture.references.size() != 3U
        || fixture.exported_lane_root
            != AL_R20_R63ZA_EXPORTED_LANE_SHA256
        || fixture.root != al_r20_r63s_consumption_fixture_root(fixture)
        || fixture.root != AL_R20_R63ZB_FIXTURE_SHA256) return false;
    for (std::size_t index = 0U; index < 3U; ++index)
        if (fixture.sources[index].size() != fixture.dimension
            || !fixture.references[index].exact
            || fixture.references[index].dimension != fixture.dimension)
            return false;
    return true;
}

struct ALR20R63ZBProduct {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t entries = 0U;
    std::vector<ALR20R63ZAScalar> value;
    std::string artifact_root;
    std::string input_root;
    std::string root;
};

ALR20R63ZBProduct al_r20_r63zb_product(
    const ALR20R63ZArtifact& artifact,
    const std::vector<ALR20R63ZAScalar>& input) {
    ALR20R63ZBProduct result;
    result.dimension = artifact.identity.dimension;
    result.artifact_root = artifact.root;
    result.input_root = al_r20_r63zb_vector_root(input);
    const std::size_t dimension = result.dimension;
    if (!al_r20_r63z_artifact_valid(artifact)
        || artifact.identity.width != 2U
        || !al_r20_r63zb_vector_valid(input, dimension)) return result;
    result.value.resize(dimension);
    bool exact = true;
    for (std::size_t row = 0U; row < dimension && exact; ++row) {
        ALR20R63ZAScalar sum = al_r20_r63za_exact_double(0.0);
        for (std::size_t column = 0U; column < dimension; ++column) {
            const std::size_t offset =
                (row * dimension + column) * 2U;
            const ALR20R63ZAScalar coefficient = al_r20_r63za_seal(
                artifact.components[offset],
                artifact.components[offset + 1U], 0.0);
            const ALR20R63ZAScalar term = al_r20_r63zb_multiply(
                coefficient, input[column]);
            sum = al_r20_r63zb_add(sum, term);
            exact = exact && al_r20_r63zb_operand(coefficient)
                && al_r20_r63zb_operand(term)
                && al_r20_r63zb_operand(sum);
            ++result.entries;
        }
        result.value[row] = sum;
    }
    result.exact = exact
        && result.entries == dimension * dimension
        && al_r20_r63zb_vector_valid(result.value, dimension);
    std::ostringstream material;
    material << result.exact << ':' << result.dimension << ':'
        << result.entries << ':' << result.artifact_root << ':'
        << result.input_root << ':'
        << al_r20_r63zb_vector_root(result.value);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63ZBDot {
    bool exact = false;
    std::size_t terms = 0U;
    ALR20R63ZAScalar value;
    std::string left_root;
    std::string right_root;
    std::string root;
};

ALR20R63ZBDot al_r20_r63zb_dot(
    const std::vector<ALR20R63ZAScalar>& left,
    const std::vector<ALR20R63ZAScalar>& right) {
    ALR20R63ZBDot result;
    result.left_root = al_r20_r63zb_vector_root(left);
    result.right_root = al_r20_r63zb_vector_root(right);
    if (left.empty() || !al_r20_r63zb_vector_valid(left, left.size())
        || !al_r20_r63zb_vector_valid(right, left.size())) return result;
    ALR20R63ZAScalar sum = al_r20_r63za_exact_double(0.0);
    bool exact = true;
    for (std::size_t index = 0U; index < left.size(); ++index) {
        const ALR20R63ZAScalar term = al_r20_r63zb_multiply(
            left[index], right[index]);
        sum = al_r20_r63zb_add(sum, term);
        exact = exact && al_r20_r63zb_operand(term)
            && al_r20_r63zb_operand(sum);
        ++result.terms;
    }
    result.value = sum;
    result.exact = exact && result.terms == left.size()
        && al_r20_r63zb_operand(result.value);
    std::ostringstream material;
    material << result.exact << ':' << result.terms << ':'
        << result.left_root << ':' << result.right_root << ':'
        << result.value.root;
    result.root = sha256_hex(material.str());
    return result;
}

std::vector<ALR20R63ZAScalar> al_r20_r63zb_update(
    const std::vector<ALR20R63ZAScalar>& base,
    const ALR20R63ZAScalar& scale,
    const std::vector<ALR20R63ZAScalar>& direction,
    double direction_sign = 1.0) {
    if (base.empty() || !al_r20_r63zb_operand(scale)
        || (direction_sign != 1.0 && direction_sign != -1.0)
        || !al_r20_r63zb_vector_valid(base, base.size())
        || !al_r20_r63zb_vector_valid(direction, base.size())) return {};
    std::vector<ALR20R63ZAScalar> result(base.size());
    for (std::size_t index = 0U; index < base.size(); ++index) {
        const ALR20R63ZAScalar term = al_r20_r63zb_multiply(
            scale, direction[index]);
        result[index] = al_r20_r63zb_add(
            base[index], term, direction_sign);
        if (!al_r20_r63zb_operand(result[index])) return {};
    }
    return result;
}

ALR20R63YSolution al_r20_r63zb_solution(
    const std::vector<ALR20R63ZAScalar>& values) {
    ALR20R63YSolution result;
    result.width = 2U;
    result.dimension = values.size();
    result.radius.assign(result.dimension, 0.0);
    bool exact = result.dimension > 0U;
    for (const ALR20R63ZAScalar& value : values) {
        exact = exact && al_r20_r63zb_operand(value);
        result.components.push_back(value.high);
        result.components.push_back(value.low);
        result.containments += al_r20_r63zb_operand(value) ? 1U : 0U;
        result.nonzero_lows += value.low != 0.0 ? 1U : 0U;
    }
    result.exact = exact && result.containments == result.dimension
        && result.components.size() == 2U * result.dimension;
    std::ostringstream material;
    material << result.exact << ':' << result.width << ':'
        << result.dimension << ':' << result.containments << ':'
        << result.nonzero_lows << ':'
        << al_r20_double_root(result.components) << ':'
        << al_r20_double_root(result.radius);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63ZBIdentity {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t width = 0U;
    std::string parent_semantic;
    std::string parent_stdout;
    std::string parent_identity_root;
    std::string fixture_root;
    std::string operator_semantic;
    std::string operator_root;
    std::string verifier_semantic;
    std::string verifier_profile_root;
    std::string factor_root;
    std::string permutation_root;
    std::string inverse_scale;
    std::string rhs_root;
    std::string lane_root;
    std::string order;
    std::string root;
};

std::string al_r20_r63zb_identity_root(
    const ALR20R63ZBIdentity& identity) {
    std::ostringstream material;
    material << identity.dimension << ':' << identity.width << ':'
        << identity.parent_semantic << ':' << identity.parent_stdout << ':'
        << identity.parent_identity_root << ':' << identity.fixture_root << ':'
        << identity.operator_semantic << ':' << identity.operator_root << ':'
        << identity.verifier_semantic << ':'
        << identity.verifier_profile_root << ':' << identity.factor_root << ':'
        << identity.permutation_root << ':' << identity.inverse_scale << ':'
        << identity.rhs_root << ':'
        << identity.lane_root << ':' << identity.order;
    return sha256_hex(material.str());
}

ALR20R63ZBIdentity al_r20_r63zb_identity(
    const std::string& parent_stdout,
    const ALR20R63ZALastResult& parent,
    const ALR20R63ZLastResult& operator_parent,
    const ALR20R63YLastResult& verifier_parent) {
    ALR20R63ZBIdentity result;
    result.dimension = parent.fixture.dimension;
    result.width = 2U;
    result.parent_semantic = parent.semantic;
    result.parent_stdout = parent_stdout;
    result.parent_identity_root = parent.identity.root;
    result.fixture_root = parent.fixture.root;
    result.operator_semantic = operator_parent.semantic;
    result.operator_root = operator_parent.artifact.root;
    result.verifier_semantic = verifier_parent.semantic;
    result.verifier_profile_root = verifier_parent.selected_profile.root;
    result.factor_root = al_r20_double_root(
        parent.fixture.exported_upper);
    result.permutation_root = al_r20_r63g_index_root(
        parent.fixture.permutation);
    result.inverse_scale = al_binary128_hex(
        parent.fixture.inverse_scale);
    if (!parent.fixture.sources.empty())
        result.rhs_root = al_r20_q_scalar_root(parent.fixture.sources[0U]);
    result.lane_root = parent.fixture.exported_lane_root;
    result.order =
        "x0,Hx0,r0,z0,p0,rho0;iteration1:Hp,den,alpha,x,r,z,rho,beta,p;"
        "iteration2:Hp,den,alpha,x,r;verify:x0,x1,x2";
    result.root = al_r20_r63zb_identity_root(result);
    result.exact = true;
    return result;
}

bool al_r20_r63zb_identity_valid(
    const ALR20R63ZBIdentity& identity,
    const ALR20R63ZALastResult& parent,
    const ALR20R63ZLastResult& operator_parent,
    const ALR20R63YLastResult& verifier_parent) {
    return identity.exact && parent.exact
        && parent.semantic == AL_R20_R63ZB_PARENT_SEMANTIC_SHA256
        && al_r20_r63za_identity_valid(parent.identity)
        && al_r20_r63zb_fixture_valid(parent.fixture)
        && operator_parent.exact
        && operator_parent.semantic
            == AL_R20_R63ZB_OPERATOR_SEMANTIC_SHA256
        && al_r20_r63z_artifact_valid(operator_parent.artifact)
        && verifier_parent.exact && verifier_parent.selected_width == 2U
        && verifier_parent.semantic
            == AL_R20_R63ZB_VERIFIER_SEMANTIC_SHA256
        && verifier_parent.twofold_profile_root
            == AL_R20_R63ZB_VERIFIER_PROFILE_SHA256
        && al_r20_r63zb_profile_valid(verifier_parent.selected_profile)
        && identity.dimension == AL_R20_R63ZB_DIMENSION
        && identity.width == 2U
        && identity.parent_semantic == AL_R20_R63ZB_PARENT_SEMANTIC_SHA256
        && identity.parent_stdout == AL_R20_R63ZB_PARENT_STDOUT_SHA256
        && identity.parent_identity_root
            == AL_R20_R63ZB_PARENT_IDENTITY_SHA256
        && identity.parent_identity_root == parent.identity.root
        && identity.fixture_root == AL_R20_R63ZB_FIXTURE_SHA256
        && identity.fixture_root == parent.fixture.root
        && identity.operator_semantic
            == AL_R20_R63ZB_OPERATOR_SEMANTIC_SHA256
        && identity.operator_semantic == operator_parent.semantic
        && identity.operator_root == AL_R20_R63ZB_OPERATOR_ARTIFACT_SHA256
        && identity.operator_root == operator_parent.artifact.root
        && identity.verifier_semantic
            == AL_R20_R63ZB_VERIFIER_SEMANTIC_SHA256
        && identity.verifier_semantic == verifier_parent.semantic
        && identity.verifier_profile_root
            == AL_R20_R63ZB_VERIFIER_PROFILE_SHA256
        && identity.verifier_profile_root
            == verifier_parent.selected_profile.root
        && identity.factor_root
            == al_r20_double_root(parent.fixture.exported_upper)
        && identity.factor_root == AL_R20_R63ZA_EXPORTED_FACTOR_SHA256
        && identity.permutation_root
            == al_r20_r63g_index_root(parent.fixture.permutation)
        && identity.permutation_root == AL_R20_R63ZA_PERMUTATION_SHA256
        && identity.inverse_scale
            == al_binary128_hex(parent.fixture.inverse_scale)
        && parent.fixture.sources.size() == 3U
        && identity.rhs_root
            == al_r20_q_scalar_root(parent.fixture.sources[0U])
        && identity.lane_root == parent.fixture.exported_lane_root
        && identity.lane_root == AL_R20_R63ZA_EXPORTED_LANE_SHA256
        && identity.order
            == "x0,Hx0,r0,z0,p0,rho0;iteration1:Hp,den,alpha,x,r,z,rho,beta,p;"
               "iteration2:Hp,den,alpha,x,r;verify:x0,x1,x2"
        && identity.root == al_r20_r63zb_identity_root(identity);
}

struct ALR20R63ZBState {
    bool exact = false;
    std::size_t index = 0U;
    std::vector<ALR20R63ZAScalar> solution;
    std::vector<ALR20R63ZAScalar> residual;
    std::vector<ALR20R63ZAScalar> direction;
    std::vector<ALR20R63ZAScalar> preconditioned;
    std::string operator_root;
    std::string solve_root;
    ALR20R63ZAScalar rho;
    ALR20R63ZAScalar denominator;
    ALR20R63ZAScalar alpha;
    ALR20R63ZAScalar beta;
    std::string root;
};

std::string al_r20_r63zb_state_root(const ALR20R63ZBState& state) {
    std::ostringstream material;
    material << state.index << ':'
        << al_r20_r63zb_vector_root(state.solution) << ':'
        << al_r20_r63zb_vector_root(state.residual) << ':'
        << al_r20_r63zb_vector_root(state.direction) << ':'
        << al_r20_r63zb_vector_root(state.preconditioned) << ':'
        << state.operator_root << ':' << state.solve_root << ':'
        << state.rho.root << ':' << state.denominator.root << ':'
        << state.alpha.root << ':' << state.beta.root;
    return sha256_hex(material.str());
}

ALR20R63ZBState al_r20_r63zb_state(
    std::size_t index,
    const std::vector<ALR20R63ZAScalar>& solution,
    const std::vector<ALR20R63ZAScalar>& residual,
    const std::vector<ALR20R63ZAScalar>& direction,
    const std::vector<ALR20R63ZAScalar>& preconditioned,
    const std::string& operator_root, const std::string& solve_root,
    const ALR20R63ZAScalar& rho,
    const ALR20R63ZAScalar& denominator,
    const ALR20R63ZAScalar& alpha,
    const ALR20R63ZAScalar& beta) {
    ALR20R63ZBState result;
    result.index = index;
    result.solution = solution;
    result.residual = residual;
    result.direction = direction;
    result.preconditioned = preconditioned;
    result.operator_root = operator_root;
    result.solve_root = solve_root;
    result.rho = rho;
    result.denominator = denominator;
    result.alpha = alpha;
    result.beta = beta;
    const std::size_t dimension = solution.size();
    result.exact = dimension > 0U
        && al_r20_r63zb_vector_valid(solution, dimension)
        && al_r20_r63zb_vector_valid(residual, dimension)
        && al_r20_r63zb_vector_valid(direction, dimension)
        && al_r20_r63zb_vector_valid(preconditioned, dimension)
        && !operator_root.empty() && !solve_root.empty()
        && al_r20_r63zb_operand(rho)
        && al_r20_r63zb_operand(denominator)
        && al_r20_r63zb_operand(alpha)
        && al_r20_r63zb_operand(beta);
    result.root = al_r20_r63zb_state_root(result);
    return result;
}

bool al_r20_r63zb_state_valid(const ALR20R63ZBState& state) {
    const std::size_t dimension = state.solution.size();
    return state.exact && dimension > 0U
        && al_r20_r63zb_vector_valid(state.solution, dimension)
        && al_r20_r63zb_vector_valid(state.residual, dimension)
        && al_r20_r63zb_vector_valid(state.direction, dimension)
        && al_r20_r63zb_vector_valid(state.preconditioned, dimension)
        && al_r20_r63zb_operand(state.rho)
        && al_r20_r63zb_operand(state.denominator)
        && al_r20_r63zb_operand(state.alpha)
        && al_r20_r63zb_operand(state.beta)
        && state.root == al_r20_r63zb_state_root(state);
}

struct ALR20R63ZBWork {
    std::size_t certificates = 0U;
    std::size_t operator_products = 0U;
    std::size_t operator_entries = 0U;
    std::size_t factor_solves = 0U;
    std::size_t factor_terms = 0U;
    std::size_t factor_divisions = 0U;
    std::size_t rho_dots = 0U;
    std::size_t denominator_dots = 0U;
    std::size_t dot_terms = 0U;
    std::size_t scalar_divisions = 0U;
    std::size_t solution_updates = 0U;
    std::size_t residual_updates = 0U;
    std::size_t direction_updates = 0U;
    std::size_t adaptive_stops = 0U;
};

bool al_r20_r63zb_work_exact(const ALR20R63ZBWork& work) {
    return work.certificates == 3U
        && work.operator_products == 3U
        && work.operator_entries == 31212U
        && work.factor_solves == 3U
        && work.factor_terms == 30906U
        && work.factor_divisions == 612U
        && work.rho_dots == 2U
        && work.denominator_dots == 2U
        && work.dot_terms == 408U
        && work.scalar_divisions == 3U
        && work.solution_updates == 204U
        && work.residual_updates == 204U
        && work.direction_updates == 102U
        && work.adaptive_stops == 0U;
}

std::string al_r20_r63zb_classify(bool apparatus, bool identity,
    bool arithmetic, bool positivity, bool verifier, bool ladder) {
    if (!apparatus) return "TWOFOLD_RECURRENCE_APPARATUS_REJECTED";
    if (!identity) return "TWOFOLD_RECURRENCE_IDENTITY_REJECTED";
    if (!arithmetic) return "TWOFOLD_RECURRENCE_ARITHMETIC_REJECTED";
    if (!positivity) return "TWOFOLD_RECURRENCE_POSITIVITY_REJECTED";
    if (!verifier) return "TWOFOLD_RECURRENCE_VERIFIER_REJECTED";
    if (!ladder) return "TWOFOLD_RECURRENCE_LADDER_REJECTED";
    return "TWOFOLD_EXPORTED_STATE2_CANDIDATE";
}

bool al_r20_r63zb_ladder(
    const std::vector<ALR20R63YCertificate>& certificates) {
    return certificates.size() == 3U
        && certificates[0U].exact && !certificates[0U].passed
        && certificates[0U].unresolved > 0U
        && certificates[1U].exact && !certificates[1U].passed
        && certificates[1U].unresolved > 0U
        && certificates[2U].exact && certificates[2U].passed
        && certificates[2U].positive == 24U
        && certificates[2U].negative == 78U
        && certificates[2U].unresolved == 0U
        && certificates[2U].sign_root == AL_R20_R63ZB_SIGN_SHA256;
}

struct ALR20R63ZBTransaction {
    bool exact = false;
    bool complete = false;
    bool arithmetic_prefix_exact = false;
    bool positivity_exact = false;
    std::size_t sealed_states = 0U;
    std::string failure_stage;
    ALR20R63ZBWork work;
    std::vector<ALR20R63ZBState> states;
    std::string root;
};

template <typename ProductFunction>
ALR20R63ZBTransaction al_r20_r63zb_transaction_with_product(
    const ALR20R63ZArtifact& artifact,
    const std::vector<double>& upper,
    const std::vector<std::size_t>& permutation,
    const ALR20R63ZAScalar& inverse_scale,
    const std::vector<ALR20R63ZAScalar>& rhs,
    ProductFunction&& apply_product) {
    ALR20R63ZBTransaction result;
    const std::size_t dimension = rhs.size();
    result.arithmetic_prefix_exact = al_r20_r63z_artifact_valid(artifact)
        && artifact.identity.dimension == dimension
        && upper.size() == dimension * dimension
        && al_r20_r63g_permutation_exact(permutation, dimension)
        && al_r20_r63zb_operand(inverse_scale)
        && al_r20_r63zb_vector_valid(rhs, dimension);
    result.positivity_exact = result.arithmetic_prefix_exact;
    if (!result.arithmetic_prefix_exact) {
        result.failure_stage = "input";
    } else {
        const ALR20R63ZBSolve start = al_r20_r63zb_solve(
            "start", upper, permutation, inverse_scale, rhs);
        ++result.work.factor_solves;
        result.work.factor_terms += start.forward_terms
            + start.backward_terms;
        result.work.factor_divisions += start.divisions;
        const ALR20R63ZBProduct initial_product =
            apply_product(start.solution);
        ++result.work.operator_products;
        result.work.operator_entries += initial_product.entries;
        const std::vector<ALR20R63ZAScalar> r0 = al_r20_r63zb_update(rhs,
            al_r20_r63za_exact_double(1.0), initial_product.value, -1.0);
        const ALR20R63ZBSolve z0_solve = al_r20_r63zb_solve(
            "initial_residual", upper, permutation, inverse_scale, r0);
        ++result.work.factor_solves;
        result.work.factor_terms += z0_solve.forward_terms
            + z0_solve.backward_terms;
        result.work.factor_divisions += z0_solve.divisions;
        const ALR20R63ZBDot rho0 = al_r20_r63zb_dot(
            r0, z0_solve.solution);
        ++result.work.rho_dots;
        result.work.dot_terms += rho0.terms;
        result.arithmetic_prefix_exact = start.exact
            && initial_product.exact && z0_solve.exact && rho0.exact
            && al_r20_r63zb_vector_valid(r0, dimension);
        result.positivity_exact = result.arithmetic_prefix_exact
            && al_r20_r63zb_positive(rho0.value);
        if (!result.arithmetic_prefix_exact) {
            result.failure_stage = "initial_arithmetic";
        } else if (!result.positivity_exact) {
            result.failure_stage = "initial_rho";
        } else {
            const std::vector<ALR20R63ZAScalar> x0 = start.solution;
            const std::vector<ALR20R63ZAScalar> z0 = z0_solve.solution;
            const std::vector<ALR20R63ZAScalar> p0 = z0;
            const ALR20R63ZBProduct hp0 = apply_product(p0);
            ++result.work.operator_products;
            result.work.operator_entries += hp0.entries;
            const ALR20R63ZBDot denominator0 = al_r20_r63zb_dot(
                p0, hp0.value);
            ++result.work.denominator_dots;
            result.work.dot_terms += denominator0.terms;
            result.arithmetic_prefix_exact = hp0.exact
                && denominator0.exact;
            result.positivity_exact = result.arithmetic_prefix_exact
                && al_r20_r63zb_positive(denominator0.value);
            if (!result.arithmetic_prefix_exact) {
                result.failure_stage = "iteration1_arithmetic";
            } else if (!result.positivity_exact) {
                result.failure_stage = "iteration1_denominator";
            } else {
                const ALR20R63ZAScalar alpha0 = al_r20_r63zb_divide(
                    rho0.value, denominator0.value);
                ++result.work.scalar_divisions;
                const std::vector<ALR20R63ZAScalar> x1 =
                    al_r20_r63zb_update(x0, alpha0, p0);
                const std::vector<ALR20R63ZAScalar> r1 =
                    al_r20_r63zb_update(r0, alpha0, hp0.value, -1.0);
                result.work.solution_updates += dimension;
                result.work.residual_updates += dimension;
                result.arithmetic_prefix_exact =
                    al_r20_r63zb_operand(alpha0)
                    && al_r20_r63zb_vector_valid(x1, dimension)
                    && al_r20_r63zb_vector_valid(r1, dimension);
                if (!result.arithmetic_prefix_exact) {
                    result.failure_stage = "iteration1_update";
                } else {
                    const ALR20R63ZBSolve z1_solve = al_r20_r63zb_solve(
                        "iteration1_residual", upper, permutation,
                        inverse_scale, r1);
                    ++result.work.factor_solves;
                    result.work.factor_terms += z1_solve.forward_terms
                        + z1_solve.backward_terms;
                    result.work.factor_divisions += z1_solve.divisions;
                    const ALR20R63ZBDot rho1 = al_r20_r63zb_dot(
                        r1, z1_solve.solution);
                    ++result.work.rho_dots;
                    result.work.dot_terms += rho1.terms;
                    result.arithmetic_prefix_exact = z1_solve.exact
                        && rho1.exact;
                    result.positivity_exact =
                        result.arithmetic_prefix_exact
                        && al_r20_r63zb_positive(rho1.value);
                    if (!result.arithmetic_prefix_exact) {
                        result.failure_stage = "iteration1_preconditioner";
                    } else if (!result.positivity_exact) {
                        result.failure_stage = "iteration1_rho";
                    } else {
                        const ALR20R63ZAScalar beta1 = al_r20_r63zb_divide(
                            rho1.value, rho0.value);
                        ++result.work.scalar_divisions;
                        const std::vector<ALR20R63ZAScalar> p1 =
                            al_r20_r63zb_update(
                                z1_solve.solution, beta1, p0);
                        result.work.direction_updates += dimension;
                        result.arithmetic_prefix_exact =
                            al_r20_r63zb_operand(beta1)
                            && al_r20_r63zb_vector_valid(p1, dimension);
                        if (!result.arithmetic_prefix_exact) {
                            result.failure_stage = "direction_update";
                        } else {
                            const ALR20R63ZBProduct hp1 =
                                apply_product(p1);
                            ++result.work.operator_products;
                            result.work.operator_entries += hp1.entries;
                            const ALR20R63ZBDot denominator1 =
                                al_r20_r63zb_dot(p1, hp1.value);
                            ++result.work.denominator_dots;
                            result.work.dot_terms += denominator1.terms;
                            result.arithmetic_prefix_exact = hp1.exact
                                && denominator1.exact;
                            result.positivity_exact =
                                result.arithmetic_prefix_exact
                                && al_r20_r63zb_positive(
                                    denominator1.value);
                            if (!result.arithmetic_prefix_exact) {
                                result.failure_stage =
                                    "iteration2_arithmetic";
                            } else if (!result.positivity_exact) {
                                result.failure_stage =
                                    "iteration2_denominator";
                            } else {
                                const ALR20R63ZAScalar alpha1 =
                                    al_r20_r63zb_divide(
                                        rho1.value, denominator1.value);
                                ++result.work.scalar_divisions;
                                const std::vector<ALR20R63ZAScalar> x2 =
                                    al_r20_r63zb_update(x1, alpha1, p1);
                                const std::vector<ALR20R63ZAScalar> r2 =
                                    al_r20_r63zb_update(
                                        r1, alpha1, hp1.value, -1.0);
                                result.work.solution_updates += dimension;
                                result.work.residual_updates += dimension;
                                result.arithmetic_prefix_exact =
                                    al_r20_r63zb_operand(alpha1)
                                    && al_r20_r63zb_vector_valid(
                                        x2, dimension)
                                    && al_r20_r63zb_vector_valid(
                                        r2, dimension);
                                if (!result.arithmetic_prefix_exact) {
                                    result.failure_stage =
                                        "iteration2_update";
                                } else {
                                    const ALR20R63ZAScalar zero =
                                        al_r20_r63za_exact_double(0.0);
                                    result.states.push_back(
                                        al_r20_r63zb_state(0U, x0, r0,
                                            p0, z0, initial_product.root,
                                            z0_solve.root, rho0.value,
                                            zero, zero, zero));
                                    result.states.push_back(
                                        al_r20_r63zb_state(1U, x1, r1,
                                            p1, z1_solve.solution, hp0.root,
                                            z1_solve.root, rho1.value,
                                            denominator0.value, alpha0,
                                            beta1));
                                    result.states.push_back(
                                        al_r20_r63zb_state(2U, x2, r2,
                                            p1, z1_solve.solution, hp1.root,
                                            z1_solve.root, rho1.value,
                                            denominator1.value, alpha1,
                                            beta1));
                                    result.complete =
                                        std::all_of(result.states.begin(),
                                            result.states.end(),
                                            [](const ALR20R63ZBState& state) {
                                                return al_r20_r63zb_state_valid(
                                                    state);
                                            });
                                    result.sealed_states =
                                        result.complete
                                        ? result.states.size() : 0U;
                                    if (!result.complete)
                                        result.failure_stage = "state_seal";
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    result.exact = result.complete
        || (result.arithmetic_prefix_exact && !result.positivity_exact
            && !result.failure_stage.empty()
            && result.states.empty() && result.sealed_states == 0U);
    std::ostringstream material;
    material << result.exact << ':' << result.complete << ':'
        << result.arithmetic_prefix_exact << ':' << result.positivity_exact
        << ':' << result.sealed_states << ':' << result.failure_stage << ':'
        << artifact.root << ':' << al_r20_double_root(upper) << ':'
        << al_r20_r63g_index_root(permutation) << ':'
        << inverse_scale.root << ':' << al_r20_r63zb_vector_root(rhs) << ':'
        << result.work.operator_products << ':' << result.work.factor_solves
        << ':' << result.work.rho_dots << ':'
        << result.work.denominator_dots << ':'
        << result.work.solution_updates << ':'
        << result.work.residual_updates << ':'
        << result.work.direction_updates << '|';
    for (const ALR20R63ZBState& state : result.states)
        material << state.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

ALR20R63ZBTransaction al_r20_r63zb_transaction(
    const ALR20R63ZArtifact& artifact,
    const std::vector<double>& upper,
    const std::vector<std::size_t>& permutation,
    const ALR20R63ZAScalar& inverse_scale,
    const std::vector<ALR20R63ZAScalar>& rhs) {
    return al_r20_r63zb_transaction_with_product(artifact, upper,
        permutation, inverse_scale, rhs,
        [&](const std::vector<ALR20R63ZAScalar>& input) {
            return al_r20_r63zb_product(artifact, input);
        });
}

std::string al_r20_r63zb_transaction_route(bool apparatus, bool identity,
    const ALR20R63ZBTransaction& transaction, bool complete_arithmetic,
    bool verifier, bool ladder) {
    const bool route_arithmetic = complete_arithmetic
        || (transaction.exact && !transaction.complete
            && transaction.arithmetic_prefix_exact
            && !transaction.positivity_exact);
    return al_r20_r63zb_classify(apparatus, identity, route_arithmetic,
        transaction.positivity_exact, verifier, ladder);
}

struct ALR20R63ZBControlPcg {
    bool exact = false;
    bool positive = false;
    bool reference = false;
    bool two_updates = false;
    std::vector<ALR20R63ZAScalar> solution;
    std::vector<ALR20R63ZAScalar> residual;
    std::vector<ALR20R63ZAScalar> direction;
    std::vector<ALR20R63ZAScalar> preconditioned;
    ALR20R63ZAScalar rho;
    std::string root;
};

ALR20R63ZArtifact al_r20_r63zb_control_artifact() {
    ALR20R63ZArtifact result;
    result.identity = al_r20_r63z_identity(
        2U, "r63zb-control-parent", "r63zb-control-common",
        "r63zb-control-tangent");
    result.entries = 4U;
    result.containments = 4U;
    result.components = {4.0, 0.0, 1.0, 0.0,
                         1.0, 0.0, 3.0, 0.0};
    result.radius.assign(4U, 0.0);
    result.maximum_radius = 0.0;
    result.component_root = al_r20_double_root(result.components);
    result.radius_root = al_r20_double_root(result.radius);
    result.contained = true;
    result.exact = true;
    result.root = al_r20_r63z_artifact_root(result);
    result.exact = al_r20_r63z_artifact_valid(result);
    return result;
}

ALR20R63ZArtifact al_r20_r63zb_diagonal_control_artifact(
    double diagonal, const std::string& role) {
    ALR20R63ZArtifact result;
    result.identity = al_r20_r63z_identity(
        2U, "r63zb-" + role + "-parent",
        "r63zb-" + role + "-common",
        "r63zb-" + role + "-tangent");
    result.entries = 4U;
    result.containments = 4U;
    result.components = {diagonal, 0.0, 0.0, 0.0,
                         0.0, 0.0, diagonal, 0.0};
    result.radius.assign(4U, 0.0);
    result.maximum_radius = 0.0;
    result.component_root = al_r20_double_root(result.components);
    result.radius_root = al_r20_double_root(result.radius);
    result.contained = true;
    result.exact = true;
    result.root = al_r20_r63z_artifact_root(result);
    result.exact = al_r20_r63z_artifact_valid(result);
    return result;
}

ALR20R63ZBControlPcg al_r20_r63zb_control_pcg() {
    ALR20R63ZBControlPcg result;
    const ALR20R63ZArtifact artifact = al_r20_r63zb_control_artifact();
    const std::vector<double> upper{1.0, 0.0, 0.0, 1.0};
    const std::vector<std::size_t> permutation{0U, 1U};
    const ALR20R63ZAScalar one = al_r20_r63za_exact_double(1.0);
    const ALR20R63ZAScalar zero = al_r20_r63za_exact_double(0.0);
    const std::vector<ALR20R63ZAScalar> rhs{
        al_r20_r63za_exact_double(1.0),
        al_r20_r63za_exact_double(2.0)};
    std::vector<ALR20R63ZAScalar> solution{zero, zero};
    std::vector<ALR20R63ZAScalar> residual = rhs;
    ALR20R63ZBSolve solve = al_r20_r63zb_solve(
        "control-r0", upper, permutation, one, residual);
    std::vector<ALR20R63ZAScalar> preconditioned = solve.solution;
    std::vector<ALR20R63ZAScalar> direction = preconditioned;
    ALR20R63ZBDot rho = al_r20_r63zb_dot(
        residual, preconditioned);
    bool exact = artifact.exact && solve.exact && rho.exact;
    bool positive = al_r20_r63zb_positive(rho.value);
    std::size_t updates = 0U;
    for (std::size_t iteration = 0U;
         iteration < 2U && exact && positive; ++iteration) {
        const ALR20R63ZBProduct product = al_r20_r63zb_product(
            artifact, direction);
        const ALR20R63ZBDot denominator = al_r20_r63zb_dot(
            direction, product.value);
        const ALR20R63ZAScalar alpha = al_r20_r63zb_divide(
            rho.value, denominator.value);
        positive = positive && al_r20_r63zb_positive(denominator.value);
        exact = exact && product.exact && denominator.exact
            && al_r20_r63zb_operand(alpha);
        solution = al_r20_r63zb_update(
            solution, alpha, direction);
        residual = al_r20_r63zb_update(
            residual, alpha, product.value, -1.0);
        updates += 2U;
        exact = exact && al_r20_r63zb_vector_valid(solution, 2U)
            && al_r20_r63zb_vector_valid(residual, 2U);
        if (iteration == 0U && exact) {
            solve = al_r20_r63zb_solve(
                "control-r1", upper, permutation, one, residual);
            preconditioned = solve.solution;
            const ALR20R63ZBDot rho_new = al_r20_r63zb_dot(
                residual, preconditioned);
            const ALR20R63ZAScalar beta = al_r20_r63zb_divide(
                rho_new.value, rho.value);
            direction = al_r20_r63zb_update(
                preconditioned, beta, direction);
            positive = positive && al_r20_r63zb_positive(rho_new.value);
            exact = exact && solve.exact && rho_new.exact
                && al_r20_r63zb_operand(beta)
                && al_r20_r63zb_vector_valid(direction, 2U);
            rho = rho_new;
            ++updates;
        }
    }
    const Binary128 tolerance = ldexpq(static_cast<Binary128>(1.0), -90);
    const std::array<Binary128, 2U> reference{
        static_cast<Binary128>(1.0) / static_cast<Binary128>(11.0),
        static_cast<Binary128>(7.0) / static_cast<Binary128>(11.0)};
    bool reference_exact = exact;
    for (std::size_t index = 0U; index < 2U; ++index) {
        const Binary128 center =
            static_cast<Binary128>(solution[index].high)
            + static_cast<Binary128>(solution[index].low);
        reference_exact = reference_exact
            && fabsq(center - reference[index]) <= tolerance;
    }
    result.solution = solution;
    result.residual = residual;
    result.direction = direction;
    result.preconditioned = preconditioned;
    result.rho = rho.value;
    result.positive = positive;
    result.reference = reference_exact;
    result.two_updates = updates == 5U;
    result.exact = exact && result.positive && result.reference
        && result.two_updates;
    std::ostringstream material;
    material << result.exact << ':' << result.positive << ':'
        << result.reference << ':' << result.two_updates << ':'
        << artifact.root << ':'
        << al_r20_r63zb_vector_root(result.solution) << ':'
        << al_r20_r63zb_vector_root(result.residual) << ':'
        << al_r20_r63zb_vector_root(result.direction) << ':'
        << result.rho.root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63ZBControls {
    bool exact = false;
    bool apparatus = false;
    bool identity = false;
    bool small_spd = false;
    bool cancellation = false;
    bool arithmetic_route = false;
    bool nonpositive_rho = false;
    bool nonpositive_denominator = false;
    bool division = false;
    bool invalid = false;
    bool overflow = false;
    bool normalization = false;
    bool forbidden_roles = false;
    bool sealed_parents = false;
    bool stale = false;
    bool mutation = false;
    bool ladder = false;
    bool classifier = false;
    std::string root;
};

ALR20R63ZBControls al_r20_r63zb_controls(
    const ALR20R63ZBIdentity& identity,
    const ALR20R63ZALastResult& parent,
    const ALR20R63ZLastResult& operator_parent,
    const ALR20R63YLastResult& verifier_parent) {
    ALR20R63ZBControls result;
    const ALR20R63ZBControlPcg control = al_r20_r63zb_control_pcg();
    result.small_spd = control.exact;
    const ALR20R63ZAScalar one = al_r20_r63za_exact_double(1.0);
    const ALR20R63ZAScalar tiny = al_r20_r63za_exact_double(
        std::ldexp(1.0, -54));
    const ALR20R63ZAScalar cancellation = al_r20_r63zb_add(one, tiny);
    result.cancellation = al_r20_r63zb_operand(cancellation)
        && cancellation.low != 0.0;
    const ALR20R63ZAScalar zero = al_r20_r63za_exact_double(0.0);
    const std::vector<double> identity_upper{1.0, 0.0, 0.0, 1.0};
    const std::vector<std::size_t> identity_permutation{0U, 1U};
    const std::vector<ALR20R63ZAScalar> positivity_rhs{
        al_r20_r63za_exact_double(1.0),
        al_r20_r63za_exact_double(2.0)};
    const ALR20R63ZBTransaction zero_rho = al_r20_r63zb_transaction(
        al_r20_r63zb_diagonal_control_artifact(1.0, "zero-rho"),
        identity_upper, identity_permutation, one, positivity_rhs);
    const ALR20R63ZBTransaction negative_denominator =
        al_r20_r63zb_transaction(
            al_r20_r63zb_diagonal_control_artifact(
                -1.0, "negative-denominator"),
            identity_upper, identity_permutation, one, positivity_rhs);
    const ALR20R63ZBTransaction invalid_input = al_r20_r63zb_transaction(
        al_r20_r63zb_diagonal_control_artifact(1.0, "invalid-input"),
        identity_upper, identity_permutation, one,
        {al_r20_r63za_exact_double(1.0)});
    const std::string invalid_input_route =
        al_r20_r63zb_transaction_route(
            true, true, invalid_input, false, false, false);
    const std::string zero_rho_route = al_r20_r63zb_transaction_route(
        true, true, zero_rho, false, false, false);
    const std::string negative_denominator_route =
        al_r20_r63zb_transaction_route(
            true, true, negative_denominator, false, false, false);
    result.nonpositive_rho = zero_rho.exact && !zero_rho.complete
        && zero_rho.failure_stage == "initial_rho"
        && zero_rho.work.solution_updates == 0U
        && zero_rho.work.residual_updates == 0U
        && zero_rho.work.direction_updates == 0U
        && zero_rho.states.empty() && zero_rho.sealed_states == 0U
        && zero_rho_route == "TWOFOLD_RECURRENCE_POSITIVITY_REJECTED";
    result.nonpositive_denominator = negative_denominator.exact
        && !negative_denominator.complete
        && negative_denominator.failure_stage == "iteration1_denominator"
        && negative_denominator.work.solution_updates == 0U
        && negative_denominator.work.residual_updates == 0U
        && negative_denominator.work.direction_updates == 0U
        && negative_denominator.states.empty()
        && negative_denominator.sealed_states == 0U
        && negative_denominator_route
            == "TWOFOLD_RECURRENCE_POSITIVITY_REJECTED";
    result.arithmetic_route = !invalid_input.exact
        && !invalid_input.complete && invalid_input.failure_stage == "input"
        && invalid_input.work.operator_products == 0U
        && invalid_input.work.factor_solves == 0U
        && invalid_input.states.empty() && invalid_input.sealed_states == 0U
        && invalid_input_route == "TWOFOLD_RECURRENCE_ARITHMETIC_REJECTED";
    const ALR20R63ZAScalar third = al_r20_r63zb_divide(
        one, al_r20_r63za_exact_double(3.0));
    result.division = al_r20_r63zb_operand(third)
        && third.low != 0.0;
    const ALR20R63ZAScalar subnormal = al_r20_r63za_exact_double(
        std::numeric_limits<double>::denorm_min());
    const ALR20R63ZAScalar infinite = al_r20_r63za_exact_double(
        std::numeric_limits<double>::infinity());
    const ALR20R63ZAScalar nan = al_r20_r63za_exact_double(
        std::numeric_limits<double>::quiet_NaN());
    result.invalid = !subnormal.exact && !infinite.exact && !nan.exact;
    result.overflow = !al_r20_r63zb_multiply(
        al_r20_r63za_exact_double(std::numeric_limits<double>::max()),
        al_r20_r63za_exact_double(std::numeric_limits<double>::max())).exact;
    ALR20R63ZAScalar nonnormalized = one;
    nonnormalized.low = 1.0;
    nonnormalized.root = al_r20_r63za_scalar_root(nonnormalized);
    result.normalization = !al_r20_r63zb_operand(nonnormalized);
    ALR20R63ZAScalar audit_radius = one;
    audit_radius.radius = 1.0;
    audit_radius.root = al_r20_r63za_scalar_root(audit_radius);
    result.forbidden_roles = al_r20_r63za_canonical(audit_radius)
        && !al_r20_r63zb_operand(audit_radius)
        && !al_r20_r63zb_add(one, audit_radius).exact
        && !al_r20_r63zb_multiply(one, audit_radius).exact;
    if (!parent.fixture.exported_upper.empty()
        && !parent.fixture.sources.empty()
        && !parent.fixture.sources[0U].empty()
        && !verifier_parent.selected_profile.vector_components.empty()
        && !operator_parent.artifact.components.empty()) {
    ALR20R63ZALastResult factor_mutation = parent;
    factor_mutation.fixture.exported_upper[0U] = std::nextafter(
        factor_mutation.fixture.exported_upper[0U],
        std::numeric_limits<double>::infinity());
    factor_mutation.fixture.root = al_r20_r63s_consumption_fixture_root(
        factor_mutation.fixture);
    const ALR20R63ZBIdentity factor_identity = al_r20_r63zb_identity(
        AL_R20_R63ZB_PARENT_STDOUT_SHA256, factor_mutation,
        operator_parent, verifier_parent);
    ALR20R63ZALastResult scale_mutation = parent;
    scale_mutation.fixture.inverse_scale = nextafterq(
        scale_mutation.fixture.inverse_scale, HUGE_VALQ);
    scale_mutation.fixture.root = al_r20_r63s_consumption_fixture_root(
        scale_mutation.fixture);
    const ALR20R63ZBIdentity scale_identity = al_r20_r63zb_identity(
        AL_R20_R63ZB_PARENT_STDOUT_SHA256, scale_mutation,
        operator_parent, verifier_parent);
    ALR20R63ZALastResult rhs_mutation = parent;
    rhs_mutation.fixture.sources[0U][0U] += static_cast<Binary128>(1.0);
    rhs_mutation.fixture.root = al_r20_r63s_consumption_fixture_root(
        rhs_mutation.fixture);
    const ALR20R63ZBIdentity rhs_identity = al_r20_r63zb_identity(
        AL_R20_R63ZB_PARENT_STDOUT_SHA256, rhs_mutation,
        operator_parent, verifier_parent);
    ALR20R63YLastResult profile_mutation = verifier_parent;
    profile_mutation.selected_profile.vector_components[0U] = std::nextafter(
        profile_mutation.selected_profile.vector_components[0U],
        std::numeric_limits<double>::infinity());
    profile_mutation.selected_profile.component_root = sha256_hex(
        al_r20_double_root(
            profile_mutation.selected_profile.vector_components) + ":"
        + al_r20_double_root(
            profile_mutation.selected_profile.matrix_components));
    profile_mutation.selected_profile.root = al_r20_r63zb_profile_root(
        profile_mutation.selected_profile);
    profile_mutation.twofold_profile_root =
        profile_mutation.selected_profile.root;
    const ALR20R63ZBIdentity profile_identity = al_r20_r63zb_identity(
        AL_R20_R63ZB_PARENT_STDOUT_SHA256, parent,
        operator_parent, profile_mutation);
    ALR20R63ZLastResult operator_mutation = operator_parent;
    operator_mutation.artifact.components[0U] = std::nextafter(
        operator_mutation.artifact.components[0U],
        std::numeric_limits<double>::infinity());
    operator_mutation.artifact.component_root = al_r20_double_root(
        operator_mutation.artifact.components);
    operator_mutation.artifact.root = al_r20_r63z_artifact_root(
        operator_mutation.artifact);
    const ALR20R63ZBIdentity operator_identity = al_r20_r63zb_identity(
        AL_R20_R63ZB_PARENT_STDOUT_SHA256, parent,
        operator_mutation, verifier_parent);
    result.sealed_parents = al_r20_r63zb_fixture_valid(parent.fixture)
        && al_r20_r63zb_profile_valid(verifier_parent.selected_profile)
        && !al_r20_r63zb_identity_valid(factor_identity, factor_mutation,
            operator_parent, verifier_parent)
        && !al_r20_r63zb_identity_valid(scale_identity, scale_mutation,
            operator_parent, verifier_parent)
        && !al_r20_r63zb_identity_valid(rhs_identity, rhs_mutation,
            operator_parent, verifier_parent)
        && !al_r20_r63zb_identity_valid(profile_identity, parent,
            operator_parent, profile_mutation)
        && !al_r20_r63zb_identity_valid(operator_identity, parent,
            operator_mutation, verifier_parent);
    }
    ALR20R63ZBIdentity stale = identity;
    stale.order = "stale-order";
    result.stale = al_r20_r63zb_identity_valid(identity, parent,
        operator_parent, verifier_parent)
        && !al_r20_r63zb_identity_valid(stale, parent,
            operator_parent, verifier_parent)
        && al_r20_r63zb_transaction_route(true, false,
            ALR20R63ZBTransaction{}, false, false, false)
            == "TWOFOLD_RECURRENCE_IDENTITY_REJECTED";
    std::vector<ALR20R63ZAScalar> mutated_vector = control.solution;
    if (!mutated_vector.empty())
        mutated_vector[0U].high = std::nextafter(
            mutated_vector[0U].high,
            std::numeric_limits<double>::infinity());
    ALR20R63ZAScalar mutated_scalar = control.rho;
    mutated_scalar.low = std::nextafter(
        mutated_scalar.low, std::numeric_limits<double>::infinity());
    ALR20R63ZBState state = al_r20_r63zb_state(2U,
        control.solution, control.residual, control.direction,
        control.preconditioned, "control-product", "control-solve",
        control.rho, one, one, zero);
    ALR20R63ZBState mutated_state = state;
    mutated_state.index = 1U;
    result.mutation = !al_r20_r63zb_vector_valid(
            mutated_vector, control.solution.size())
        && !al_r20_r63zb_operand(mutated_scalar)
        && al_r20_r63zb_state_valid(state)
        && !al_r20_r63zb_state_valid(mutated_state);
    std::vector<ALR20R63YCertificate> ladder(3U);
    ladder[0U].exact = true;
    ladder[0U].unresolved = 1U;
    ladder[1U].exact = true;
    ladder[1U].unresolved = 1U;
    ladder[2U].exact = true;
    ladder[2U].passed = true;
    ladder[2U].positive = 24U;
    ladder[2U].negative = 78U;
    ladder[2U].sign_root = AL_R20_R63ZB_SIGN_SHA256;
    const bool expected_ladder = al_r20_r63zb_ladder(ladder);
    ladder[0U].passed = true;
    const bool state0_pass_rejected = !al_r20_r63zb_ladder(ladder);
    ladder[0U].passed = false;
    ladder[1U].passed = true;
    const bool state1_pass_rejected = !al_r20_r63zb_ladder(ladder);
    ladder[1U].passed = false;
    ladder[2U].passed = false;
    const bool state2_reject_rejected = !al_r20_r63zb_ladder(ladder);
    result.ladder = expected_ladder && state0_pass_rejected
        && state1_pass_rejected && state2_reject_rejected;
    result.classifier =
        al_r20_r63zb_classify(false, false, false, false, false, false)
            == "TWOFOLD_RECURRENCE_APPARATUS_REJECTED"
        && al_r20_r63zb_classify(true, false, false, false, false, false)
            == "TWOFOLD_RECURRENCE_IDENTITY_REJECTED"
        && al_r20_r63zb_classify(true, true, false, false, false, false)
            == "TWOFOLD_RECURRENCE_ARITHMETIC_REJECTED"
        && al_r20_r63zb_classify(true, true, true, false, false, false)
            == "TWOFOLD_RECURRENCE_POSITIVITY_REJECTED"
        && al_r20_r63zb_classify(true, true, true, true, false, false)
            == "TWOFOLD_RECURRENCE_VERIFIER_REJECTED"
        && al_r20_r63zb_classify(true, true, true, true, true, false)
            == "TWOFOLD_RECURRENCE_LADDER_REJECTED"
        && al_r20_r63zb_classify(true, true, true, true, true, true)
            == "TWOFOLD_EXPORTED_STATE2_CANDIDATE";
    result.apparatus = result.small_spd && result.cancellation
        && result.arithmetic_route
        && result.nonpositive_rho && result.nonpositive_denominator
        && result.division && result.invalid && result.overflow
        && result.normalization && result.forbidden_roles
        && result.mutation && result.ladder && result.classifier;
    result.identity = result.sealed_parents && result.stale;
    result.exact = result.apparatus && result.identity;
    std::ostringstream material;
    material << result.exact << ':' << result.apparatus << ':'
        << result.identity << ':' << result.small_spd << ':'
        << result.cancellation << ':' << result.arithmetic_route << ':'
        << result.nonpositive_rho << ':'
        << result.nonpositive_denominator << ':' << result.division << ':'
        << result.invalid << ':' << result.overflow << ':'
        << result.normalization << ':' << result.forbidden_roles << ':'
        << result.sealed_parents << ':'
        << result.stale << ':' << result.mutation << ':'
        << result.ladder << ':'
        << result.classifier << ':' << control.root << ':'
        << cancellation.root << ':' << third.root << ':' << identity.root
        << ':' << zero_rho.root << ':' << negative_denominator.root << ':'
        << invalid_input.root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63ZBComparator {
    bool exact = false;
    bool ladder = false;
    std::size_t operator_products = 0U;
    std::size_t factor_solves = 0U;
    std::size_t scalar_dots = 0U;
    std::vector<std::vector<Binary128>> states;
    std::vector<ALR20R63YCertificate> certificates;
    std::array<Binary128, 3U> maximum_finite_difference{};
    std::array<Binary128, 3U> maximum_parent_difference{};
    std::string root;
};

ALR20R63ZBComparator al_r20_r63zb_binary128_comparator(
    const ALR20R63ZArtifact& artifact,
    const std::vector<ALR20R63ZAScalar>& finite_rhs,
    const ALR20R63ZAScalar& finite_inverse_scale,
    const std::vector<double>& upper,
    const std::vector<std::size_t>& permutation,
    const ALR20R63YProfile& profile,
    const std::vector<ALR20R63ZBState>& finite_states,
    const std::vector<std::vector<Binary128>>& parent_states) {
    ALR20R63ZBComparator result;
    const std::size_t dimension = artifact.identity.dimension;
    if (!al_r20_r63z_artifact_valid(artifact)
        || !al_r20_r63zb_vector_valid(finite_rhs, dimension)
        || !al_r20_r63zb_operand(finite_inverse_scale)
        || upper.size() != dimension * dimension
        || !al_r20_r63g_permutation_exact(permutation, dimension)
        || !profile.exact || finite_states.size() != 3U
        || parent_states.size() != 3U) return result;
    std::vector<Binary128> matrix(dimension * dimension);
    for (std::size_t index = 0U; index < matrix.size(); ++index)
        matrix[index] = static_cast<Binary128>(artifact.components[2U * index])
            + static_cast<Binary128>(artifact.components[2U * index + 1U]);
    std::vector<Binary128> rhs(dimension);
    for (std::size_t index = 0U; index < dimension; ++index)
        rhs[index] = static_cast<Binary128>(finite_rhs[index].high)
            + static_cast<Binary128>(finite_rhs[index].low);
    const Binary128 inverse_scale =
        static_cast<Binary128>(finite_inverse_scale.high)
        + static_cast<Binary128>(finite_inverse_scale.low);
    const std::vector<Binary128> upper_q = al_r20_r63g_q_from_double(upper);
    ALR20R63IQSolve solve = al_r20_r63i_q_solve(
        upper_q, permutation, rhs, inverse_scale);
    ++result.factor_solves;
    std::vector<Binary128> solution = solve.solution;
    result.states.push_back(solution);
    ALR20R63MProduct product = al_r20_r63m_product(matrix, solution);
    ++result.operator_products;
    std::vector<Binary128> residual(dimension);
    bool exact = solve.exact && product.exact;
    for (std::size_t index = 0U; index < dimension; ++index) {
        const ALR20R38Dot2 dot = al_r20_r38_dot2(
            {rhs[index], -product.value[index]},
            {static_cast<Binary128>(1.0), static_cast<Binary128>(1.0)});
        residual[index] = dot.value;
        exact = exact && dot.exact && dot.no_underflow;
    }
    solve = al_r20_r63i_q_solve(
        upper_q, permutation, residual, inverse_scale);
    ++result.factor_solves;
    std::vector<Binary128> preconditioned = solve.solution;
    std::vector<Binary128> direction = preconditioned;
    ALR20R63MScalar rho = al_r20_r63m_scalar(
        residual, preconditioned);
    ++result.scalar_dots;
    exact = exact && solve.exact && rho.exact && rho.positive;
    for (std::size_t iteration = 0U;
         iteration < 2U && exact; ++iteration) {
        product = al_r20_r63m_product(matrix, direction);
        ++result.operator_products;
        const ALR20R63MScalar denominator = al_r20_r63m_scalar(
            direction, product.value);
        ++result.scalar_dots;
        exact = exact && product.exact
            && denominator.exact && denominator.positive;
        const Binary128 alpha = rho.value / denominator.value;
        std::vector<Binary128> updated_solution(dimension);
        std::vector<Binary128> updated_residual(dimension);
        for (std::size_t index = 0U; index < dimension; ++index) {
            const ALR20R38Dot2 x_dot = al_r20_r38_dot2(
                {solution[index], alpha},
                {static_cast<Binary128>(1.0), direction[index]});
            const ALR20R38Dot2 r_dot = al_r20_r38_dot2(
                {residual[index], -alpha},
                {static_cast<Binary128>(1.0), product.value[index]});
            updated_solution[index] = x_dot.value;
            updated_residual[index] = r_dot.value;
            exact = exact && x_dot.exact && r_dot.exact
                && x_dot.no_underflow && r_dot.no_underflow;
        }
        solution = std::move(updated_solution);
        residual = std::move(updated_residual);
        result.states.push_back(solution);
        if (iteration == 0U && exact) {
            solve = al_r20_r63i_q_solve(
                upper_q, permutation, residual, inverse_scale);
            ++result.factor_solves;
            preconditioned = solve.solution;
            const ALR20R63MScalar rho_new = al_r20_r63m_scalar(
                residual, preconditioned);
            ++result.scalar_dots;
            const Binary128 beta = rho_new.value / rho.value;
            std::vector<Binary128> updated_direction(dimension);
            for (std::size_t index = 0U; index < dimension; ++index) {
                const ALR20R38Dot2 dot = al_r20_r38_dot2(
                    {preconditioned[index], beta},
                    {static_cast<Binary128>(1.0), direction[index]});
                updated_direction[index] = dot.value;
                exact = exact && dot.exact && dot.no_underflow;
            }
            exact = exact && solve.exact
                && rho_new.exact && rho_new.positive;
            direction = std::move(updated_direction);
            rho = rho_new;
        }
    }
    if (exact && result.states.size() == 3U) {
        for (std::size_t state = 0U; state < 3U; ++state) {
            const ALR20R63YSolution candidate = al_r20_r63y_solution(
                result.states[state], 2U);
            result.certificates.push_back(al_r20_r63y_certificate(
                profile, candidate));
            for (std::size_t index = 0U; index < dimension; ++index) {
                const Binary128 finite =
                    static_cast<Binary128>(
                        finite_states[state].solution[index].high)
                    + static_cast<Binary128>(
                        finite_states[state].solution[index].low);
                result.maximum_finite_difference[state] = fmaxq(
                    result.maximum_finite_difference[state],
                    fabsq(result.states[state][index] - finite));
                result.maximum_parent_difference[state] = fmaxq(
                    result.maximum_parent_difference[state],
                    fabsq(result.states[state][index]
                        - parent_states[state][index]));
            }
        }
    }
    result.ladder = al_r20_r63zb_ladder(result.certificates);
    result.exact = exact && result.states.size() == 3U
        && result.certificates.size() == 3U
        && std::all_of(result.certificates.begin(), result.certificates.end(),
            [](const ALR20R63YCertificate& certificate) {
                return certificate.exact;
            })
        && result.operator_products == 3U
        && result.factor_solves == 3U
        && result.scalar_dots == 4U;
    std::ostringstream material;
    material << result.exact << ':' << result.ladder << ':'
        << result.operator_products << ':' << result.factor_solves << ':'
        << result.scalar_dots << '|';
    for (std::size_t state = 0U; state < result.states.size(); ++state)
        material << al_r20_q_scalar_root(result.states[state]) << ':'
            << result.certificates[state].root << ':'
            << al_binary128_hex(result.maximum_finite_difference[state])
            << ':'
            << al_binary128_hex(result.maximum_parent_difference[state])
            << ';';
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_twofold_recurrence_controls() {
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
    const SplitBoundaryReport parent =
        run_al_generalization_v5_finite_factor_consumption_controls();
    const std::string parent_stdout = sha256_hex(parent.json + "\n");
    const ALR20R63ZALastResult parent_result = al_r20_r63za_last_result;
    const ALR20R63ZLastResult operator_parent = al_r20_r63z_last_result;
    const ALR20R63YLastResult verifier_parent = al_r20_r63y_last_result;
    const bool parent_exact = parent.passed && parent_result.exact
        && parent_result.route
            == "TWOFOLD_EXPORTED_FACTOR_CONSUMPTION_CANDIDATE"
        && parent_result.semantic == AL_R20_R63ZB_PARENT_SEMANTIC_SHA256
        && parent_stdout == AL_R20_R63ZB_PARENT_STDOUT_SHA256
        && al_r20_r63za_identity_valid(parent_result.identity)
        && parent_result.identity.root
            == AL_R20_R63ZB_PARENT_IDENTITY_SHA256
        && al_r20_r63zb_fixture_valid(parent_result.fixture)
        && operator_parent.exact
        && operator_parent.semantic
            == AL_R20_R63ZB_OPERATOR_SEMANTIC_SHA256
        && al_r20_r63z_artifact_valid(operator_parent.artifact)
        && operator_parent.artifact.root
            == AL_R20_R63ZB_OPERATOR_ARTIFACT_SHA256
        && verifier_parent.exact && verifier_parent.selected_width == 2U
        && verifier_parent.semantic
            == AL_R20_R63ZB_VERIFIER_SEMANTIC_SHA256
        && verifier_parent.twofold_profile_root
            == AL_R20_R63ZB_VERIFIER_PROFILE_SHA256
        && al_r20_r63zb_profile_valid(
            verifier_parent.selected_profile);
    const ALR20R63ZBIdentity identity = al_r20_r63zb_identity(
        parent_stdout, parent_result, operator_parent, verifier_parent);
    const bool identity_payload_exact = parent_exact
        && al_r20_r63zb_identity_valid(identity, parent_result,
            operator_parent, verifier_parent);
    const ALR20R63ZBControls controls = al_r20_r63zb_controls(
        identity, parent_result, operator_parent, verifier_parent);
    const bool identity_exact = identity_payload_exact && controls.identity;

    std::vector<ALR20R63ZAScalar> rhs;
    ALR20R63ZAScalar inverse_scale;
    if (identity_exact) {
        rhs.reserve(AL_R20_R63ZB_DIMENSION);
        for (Binary128 value : parent_result.fixture.sources[0U])
            rhs.push_back(al_r20_r63zb_project(value));
        inverse_scale = al_r20_r63zb_project(
            parent_result.fixture.inverse_scale);
    }
    const ALR20R63ZBTransaction transaction = identity_exact
        ? al_r20_r63zb_transaction(operator_parent.artifact,
            parent_result.fixture.exported_upper,
            parent_result.fixture.permutation, inverse_scale, rhs)
        : ALR20R63ZBTransaction{};
    ALR20R63ZBWork work = transaction.work;
    const std::vector<ALR20R63ZBState>& states = transaction.states;
    const bool recurrence_exact = transaction.complete;
    const bool positivity_exact = transaction.positivity_exact;
    std::vector<ALR20R63YSolution> solutions;
    std::vector<ALR20R63YCertificate> certificates;
    if (recurrence_exact) {
        for (const ALR20R63ZBState& state : states) {
            solutions.push_back(al_r20_r63zb_solution(state.solution));
            certificates.push_back(al_r20_r63y_certificate(
                verifier_parent.selected_profile, solutions.back()));
            ++work.certificates;
        }
    }
    const bool verifier_exact = certificates.size() == 3U
        && std::all_of(certificates.begin(), certificates.end(),
            [](const ALR20R63YCertificate& certificate) {
                return certificate.exact
                    && (!certificate.passed
                        || certificate.sign_root
                            == AL_R20_R63ZB_SIGN_SHA256);
            });
    const bool ladder_exact = verifier_exact
        && al_r20_r63zb_ladder(certificates);
    const ALR20R63ZBComparator comparator = recurrence_exact
        ? al_r20_r63zb_binary128_comparator(operator_parent.artifact,
            rhs, inverse_scale, parent_result.fixture.exported_upper,
            parent_result.fixture.permutation,
            verifier_parent.selected_profile, states,
            al_r20_r63x_last_fixture.exported_solutions)
        : ALR20R63ZBComparator{};
    std::vector<std::vector<Binary128>> mutated_parent_states =
        al_r20_r63x_last_fixture.exported_solutions;
    const bool comparator_mutation_input =
        mutated_parent_states.size() == 3U
        && !mutated_parent_states[0U].empty();
    if (comparator_mutation_input)
        mutated_parent_states[0U][0U] += static_cast<Binary128>(1.0);
    const ALR20R63ZBComparator mutated_comparator =
        recurrence_exact && comparator_mutation_input
        ? al_r20_r63zb_binary128_comparator(operator_parent.artifact,
            rhs, inverse_scale, parent_result.fixture.exported_upper,
            parent_result.fixture.permutation,
            verifier_parent.selected_profile, states,
            mutated_parent_states)
        : ALR20R63ZBComparator{};
    bool comparator_outputs_unchanged = comparator.exact
        && mutated_comparator.exact
        && comparator.ladder == mutated_comparator.ladder
        && comparator.states.size() == mutated_comparator.states.size()
        && comparator.certificates.size()
            == mutated_comparator.certificates.size();
    bool comparator_parent_difference_changed = false;
    for (std::size_t state = 0U;
         state < comparator.states.size() && comparator_outputs_unchanged;
         ++state) {
        comparator_outputs_unchanged =
            al_r20_q_scalar_root(comparator.states[state])
                == al_r20_q_scalar_root(mutated_comparator.states[state])
            && comparator.certificates[state].root
                == mutated_comparator.certificates[state].root
            && comparator.maximum_finite_difference[state]
                == mutated_comparator.maximum_finite_difference[state];
        comparator_parent_difference_changed =
            comparator_parent_difference_changed
            || comparator.maximum_parent_difference[state]
                != mutated_comparator.maximum_parent_difference[state];
    }
    const bool comparator_exclusion_exact = comparator_mutation_input
        && comparator_outputs_unchanged
        && comparator_parent_difference_changed
        && comparator.root != mutated_comparator.root;
    const bool work_exact = al_r20_r63zb_work_exact(work);
    const bool apparatus_exact = profile_exact && controls.apparatus;
    const bool arithmetic_exact = recurrence_exact && transaction.exact
        && work_exact;
    const std::string route = al_r20_r63zb_transaction_route(
        apparatus_exact, identity_exact, transaction, arithmetic_exact,
        verifier_exact, ladder_exact);
    const bool exact = apparatus_exact && identity_exact
        && arithmetic_exact && positivity_exact
        && verifier_exact && ladder_exact && comparator_exclusion_exact;
    std::ostringstream state_material;
    for (const ALR20R63ZBState& state : states)
        state_material << state.root << ';';
    std::ostringstream certificate_material;
    for (const ALR20R63YCertificate& certificate : certificates)
        certificate_material << certificate.root << ';';
    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63zb-twofold-recurrence.v1|"
        << parent_result.semantic << ':' << route << ':' << controls.root
        << ':' << identity.root << ':' << transaction.root << ':'
        << state_material.str() << ':'
        << certificate_material.str() << ':' << work.certificates << ':'
        << work.operator_products << ':' << work.operator_entries << ':'
        << work.factor_solves << ':' << work.factor_terms << ':'
        << work.factor_divisions << ':' << work.rho_dots << ':'
        << work.denominator_dots << ':' << work.dot_terms << ':'
        << work.scalar_divisions << ':' << work.solution_updates << ':'
        << work.residual_updates << ':' << work.direction_updates << ':'
        << work.adaptive_stops;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63zb_twofold_recurrence.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63za_sha256\":\"" << parent_result.semantic << "\""
        << ",\"parent_stdout_sha256\":\"" << parent_stdout << "\""
        << ",\"identity_exact\":"
        << (identity_exact ? "true" : "false")
        << ",\"identity_root\":\"" << identity.root << "\""
        << ",\"transaction_exact\":"
        << (transaction.exact ? "true" : "false")
        << ",\"transaction_complete\":"
        << (transaction.complete ? "true" : "false")
        << ",\"transaction_failure_stage\":\""
        << transaction.failure_stage << "\""
        << ",\"transaction_root\":\"" << transaction.root << "\""
        << ",\"controls_exact\":"
        << (controls.exact ? "true" : "false")
        << ",\"controls_root\":\"" << controls.root << "\""
        << ",\"controls\":{\"apparatus\":"
        << (controls.apparatus ? "true" : "false")
        << ",\"identity\":"
        << (controls.identity ? "true" : "false")
        << ",\"small_spd\":"
        << (controls.small_spd ? "true" : "false")
        << ",\"cancellation\":"
        << (controls.cancellation ? "true" : "false")
        << ",\"arithmetic_route\":"
        << (controls.arithmetic_route ? "true" : "false")
        << ",\"nonpositive_rho\":"
        << (controls.nonpositive_rho ? "true" : "false")
        << ",\"nonpositive_denominator\":"
        << (controls.nonpositive_denominator ? "true" : "false")
        << ",\"division\":" << (controls.division ? "true" : "false")
        << ",\"invalid\":" << (controls.invalid ? "true" : "false")
        << ",\"overflow\":" << (controls.overflow ? "true" : "false")
        << ",\"normalization\":"
        << (controls.normalization ? "true" : "false")
        << ",\"forbidden_roles\":"
        << (controls.forbidden_roles ? "true" : "false")
        << ",\"sealed_parents\":"
        << (controls.sealed_parents ? "true" : "false")
        << ",\"stale\":" << (controls.stale ? "true" : "false")
        << ",\"mutation\":" << (controls.mutation ? "true" : "false")
        << ",\"ladder\":" << (controls.ladder ? "true" : "false")
        << ",\"classifier\":"
        << (controls.classifier ? "true" : "false") << '}';
    report << ",\"states\":[";
    for (std::size_t index = 0U; index < states.size(); ++index) {
        if (index != 0U) report << ',';
        report << "{\"state\":" << states[index].index
            << ",\"root\":\"" << states[index].root << "\""
            << ",\"solution_root\":\""
            << al_r20_r63zb_vector_root(states[index].solution) << "\""
            << ",\"residual_root\":\""
            << al_r20_r63zb_vector_root(states[index].residual) << "\"}";
    }
    report << "] ,\"certificates\":[";
    for (std::size_t index = 0U; index < certificates.size(); ++index) {
        if (index != 0U) report << ',';
        report << "{\"state\":" << index
            << ",\"passed\":"
            << (certificates[index].passed ? "true" : "false")
            << ",\"positive\":" << certificates[index].positive
            << ",\"negative\":" << certificates[index].negative
            << ",\"unresolved\":" << certificates[index].unresolved
            << ",\"error_upper\":" << certificates[index].error_upper
            << ",\"sign_root\":\"" << certificates[index].sign_root
            << "\",\"root\":\"" << certificates[index].root << "\"}";
    }
    report << "]"
        << ",\"binary128_recurrence_comparator\":{\"exact\":"
        << (comparator.exact ? "true" : "false")
        << ",\"ladder\":" << (comparator.ladder ? "true" : "false")
        << ",\"operator_products\":" << comparator.operator_products
        << ",\"factor_solves\":" << comparator.factor_solves
        << ",\"scalar_dots\":" << comparator.scalar_dots
        << ",\"states\":[";
    for (std::size_t index = 0U;
         index < comparator.certificates.size(); ++index) {
        if (index != 0U) report << ',';
        report << "{\"state\":" << index
            << ",\"passed\":"
            << (comparator.certificates[index].passed ? "true" : "false")
            << ",\"positive\":" << comparator.certificates[index].positive
            << ",\"negative\":" << comparator.certificates[index].negative
            << ",\"unresolved\":"
            << comparator.certificates[index].unresolved
            << ",\"error_upper\":"
            << comparator.certificates[index].error_upper
            << ",\"finite_difference\":\""
            << al_binary128_text(
                comparator.maximum_finite_difference[index])
            << "\",\"parent_difference\":\""
            << al_binary128_text(
                comparator.maximum_parent_difference[index])
            << "\"}";
    }
    report << "],\"shared_r63y_verifier\":true"
        << ",\"independent_end_to_end\":false"
        << ",\"root\":\"" << comparator.root << "\"}"
        << ",\"comparator_exclusion_exact\":"
        << (comparator_exclusion_exact ? "true" : "false")
        << ",\"mutated_comparator_root\":\""
        << mutated_comparator.root << "\""
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"arithmetic_exact\":"
        << (arithmetic_exact ? "true" : "false")
        << ",\"positivity_exact\":"
        << (positivity_exact ? "true" : "false")
        << ",\"verifier_exact\":"
        << (verifier_exact ? "true" : "false")
        << ",\"ladder_exact\":"
        << (ladder_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"work\":{\"certificates\":" << work.certificates
        << ",\"operator_products\":" << work.operator_products
        << ",\"operator_entries\":" << work.operator_entries
        << ",\"factor_solves\":" << work.factor_solves
        << ",\"factor_terms\":" << work.factor_terms
        << ",\"factor_divisions\":" << work.factor_divisions
        << ",\"rho_dots\":" << work.rho_dots
        << ",\"denominator_dots\":" << work.denominator_dots
        << ",\"dot_terms\":" << work.dot_terms
        << ",\"scalar_divisions\":" << work.scalar_divisions
        << ",\"solution_updates\":" << work.solution_updates
        << ",\"residual_updates\":" << work.residual_updates
        << ",\"direction_updates\":" << work.direction_updates
        << ",\"adaptive_stops\":" << work.adaptive_stops << '}'
        << ",\"exact_or_binary128_candidate_inputs\":0"
        << ",\"audit_radius_recurrence_inputs\":0"
        << ",\"adaptive_stop_uses\":0"
        << ",\"timing_admitted\":false"
        << ",\"runtime_authority\":false"
        << ",\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
