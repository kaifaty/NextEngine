namespace {

constexpr const char* AL_R20_R63E_PARENT_SEMANTIC_SHA256 =
    "f9a5f0d807818eab61759b81d1b1df99265909c249fd8348308e614d06a45b8d";

std::string al_r20_r63e_projector_material_root(
    const ALR20ProjectorState& state) {
    std::ostringstream material;
    material << state.exact << ':' << state.ball_active << ':'
        << state.free_components << ':' << state.lower_components << ':'
        << state.upper_components << ':' << state.fixed_components << ':'
        << al_binary128_hex(state.eta) << ':'
        << al_binary128_hex(state.free_norm_squared) << ':' << state.root
        << '|';
    for (int item : state.mask) material << item << ',';
    material << '|' << al_r20_q_vec_root(state.value);
    return sha256_hex(material.str());
}

struct ALR20R63EVectorDecomposition {
    bool exact = false;
    std::size_t scalars = 0U;
    std::size_t free_components = 0U;
    std::size_t clamped_components = 0U;
    Binary128 radial_coefficient = static_cast<Binary128>(0.0);
    Binary128 source_norm_squared = static_cast<Binary128>(0.0);
    Binary128 clamped_norm_squared = static_cast<Binary128>(0.0);
    Binary128 free_norm_squared = static_cast<Binary128>(0.0);
    Binary128 radial_norm_squared = static_cast<Binary128>(0.0);
    Binary128 tangent_norm_squared = static_cast<Binary128>(0.0);
    Binary128 image_norm_squared = static_cast<Binary128>(0.0);
    Binary128 reconstruction_residual = static_cast<Binary128>(0.0);
    Binary128 reconstruction_bound = static_cast<Binary128>(0.0);
    Binary128 energy_residual = static_cast<Binary128>(0.0);
    Binary128 energy_bound = static_cast<Binary128>(0.0);
    Binary128 jvp_residual = static_cast<Binary128>(0.0);
    Binary128 jvp_bound = static_cast<Binary128>(0.0);
    std::vector<Binary128Vec3> source;
    std::vector<Binary128Vec3> clamped;
    std::vector<Binary128Vec3> free;
    std::vector<Binary128Vec3> radial;
    std::vector<Binary128Vec3> tangent;
    std::vector<Binary128Vec3> image;
    std::string root;
};

ALR20R63EVectorDecomposition al_r20_r63e_decompose(
    const ALR20ProjectorState& state,
    const std::vector<Binary128Vec3>& source) {
    ALR20R63EVectorDecomposition result;
    result.scalars = 3U * source.size();
    result.source = source;
    if (!state.exact || state.value.size() != source.size()
        || state.mask.size() != result.scalars) return result;
    result.clamped.resize(source.size());
    result.free.resize(source.size());
    result.radial.resize(source.size());
    result.tangent.resize(source.size());
    result.image.resize(source.size());
    bool exact = true;
    Binary128Accumulator radial_dot;
    for (std::size_t particle = 0U; particle < source.size(); ++particle)
        for (int axis = 0; axis < 3; ++axis) {
            const Binary128 value = al_r20_q_component(source[particle], axis);
            exact = exact && finiteq(value) != 0;
            const std::size_t scalar = 3U * particle
                + static_cast<std::size_t>(axis);
            if (state.mask[scalar] == 0) {
                al_r20_q_set_component(&result.free[particle], axis, value);
                radial_dot.add(
                    al_r20_q_component(state.value[particle], axis) * value);
                ++result.free_components;
            } else {
                al_r20_q_set_component(
                    &result.clamped[particle], axis, value);
                ++result.clamped_components;
            }
        }
    if (state.ball_active) {
        exact = exact
            && state.free_norm_squared > static_cast<Binary128>(0.0);
        if (exact)
            result.radial_coefficient =
                radial_dot.value() / state.free_norm_squared;
    }
    const Binary128 inverse = static_cast<Binary128>(1.0)
        / (static_cast<Binary128>(1.0) + state.eta);
    for (std::size_t particle = 0U; particle < source.size(); ++particle)
        for (int axis = 0; axis < 3; ++axis) {
            const std::size_t scalar = 3U * particle
                + static_cast<std::size_t>(axis);
            if (state.mask[scalar] != 0) continue;
            const Binary128 radial = state.ball_active
                ? al_r20_q_component(state.value[particle], axis)
                    * result.radial_coefficient
                : static_cast<Binary128>(0.0);
            const Binary128 tangent =
                al_r20_q_component(result.free[particle], axis) - radial;
            const Binary128 image = inverse * tangent;
            exact = exact && finiteq(radial) != 0 && finiteq(tangent) != 0
                && finiteq(image) != 0;
            al_r20_q_set_component(&result.radial[particle], axis, radial);
            al_r20_q_set_component(&result.tangent[particle], axis, tangent);
            al_r20_q_set_component(&result.image[particle], axis, image);
        }
    result.source_norm_squared =
        al_r20_projector_vector_dot(result.source, result.source);
    result.clamped_norm_squared =
        al_r20_projector_vector_dot(result.clamped, result.clamped);
    result.free_norm_squared =
        al_r20_projector_vector_dot(result.free, result.free);
    result.radial_norm_squared =
        al_r20_projector_vector_dot(result.radial, result.radial);
    result.tangent_norm_squared =
        al_r20_projector_vector_dot(result.tangent, result.tangent);
    result.image_norm_squared =
        al_r20_projector_vector_dot(result.image, result.image);
    Binary128 scale = static_cast<Binary128>(1.0);
    for (std::size_t particle = 0U; particle < source.size(); ++particle)
        for (int axis = 0; axis < 3; ++axis) {
            const Binary128 expected = al_r20_q_component(
                result.clamped[particle], axis)
                + al_r20_q_component(result.radial[particle], axis)
                + al_r20_q_component(result.tangent[particle], axis);
            result.reconstruction_residual = fmaxq(
                result.reconstruction_residual,
                fabsq(al_r20_q_component(source[particle], axis) - expected));
            scale = fmaxq(scale,
                fabsq(al_r20_q_component(source[particle], axis)));
        }
    result.reconstruction_bound = al_r20_q_gamma(
        64U * result.scalars + 4096U) * scale;
    result.energy_residual = fabsq(result.source_norm_squared
        - (result.clamped_norm_squared + result.radial_norm_squared
            + result.tangent_norm_squared));
    result.energy_bound = al_r20_q_gamma(
        128U * result.scalars + 8192U)
        * fmaxq(static_cast<Binary128>(1.0), result.source_norm_squared);
    const std::vector<Binary128Vec3> direct =
        al_r20_projector_jvp(state, source);
    for (std::size_t particle = 0U; particle < source.size(); ++particle)
        for (int axis = 0; axis < 3; ++axis)
            result.jvp_residual = fmaxq(result.jvp_residual,
                fabsq(al_r20_q_component(result.image[particle], axis)
                    - al_r20_q_component(direct[particle], axis)));
    result.jvp_bound = al_r20_q_gamma(
        128U * result.scalars + 8192U) * scale;
    result.exact = exact
        && result.free_components + result.clamped_components
            == result.scalars
        && result.reconstruction_residual <= result.reconstruction_bound
        && result.energy_residual <= result.energy_bound
        && result.jvp_residual <= result.jvp_bound;
    std::ostringstream material;
    material << result.exact << ':' << result.scalars << ':'
        << result.free_components << ':' << result.clamped_components << ':'
        << al_binary128_hex(result.radial_coefficient) << ':'
        << al_binary128_hex(result.source_norm_squared) << ':'
        << al_binary128_hex(result.clamped_norm_squared) << ':'
        << al_binary128_hex(result.free_norm_squared) << ':'
        << al_binary128_hex(result.radial_norm_squared) << ':'
        << al_binary128_hex(result.tangent_norm_squared) << ':'
        << al_binary128_hex(result.image_norm_squared) << ':'
        << al_binary128_hex(result.reconstruction_residual) << ':'
        << al_binary128_hex(result.reconstruction_bound) << ':'
        << al_binary128_hex(result.energy_residual) << ':'
        << al_binary128_hex(result.energy_bound) << ':'
        << al_binary128_hex(result.jvp_residual) << ':'
        << al_binary128_hex(result.jvp_bound) << ':'
        << al_r20_q_vec_root(result.source) << ':'
        << al_r20_q_vec_root(result.clamped) << ':'
        << al_r20_q_vec_root(result.free) << ':'
        << al_r20_q_vec_root(result.radial) << ':'
        << al_r20_q_vec_root(result.tangent) << ':'
        << al_r20_q_vec_root(result.image);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63EProjectorGrams {
    bool exact = false;
    bool direct_jvp_exact = false;
    bool full_matrix_exact = false;
    std::size_t rows = 0U;
    std::size_t scalars = 0U;
    std::size_t row_constructions = 0U;
    std::size_t directed_dots = 0U;
    std::size_t symmetrizations = 0U;
    std::vector<std::vector<Binary128Vec3>> source_rows;
    std::vector<Binary128> clamp;
    std::vector<Binary128> tangent;
    std::vector<Binary128> full;
    std::string clamp_root;
    std::string tangent_root;
    std::string full_root;
    std::string root;
};

ALR20R63EProjectorGrams al_r20_r63e_projector_grams(
    const ALR64Operator& op, const std::vector<std::size_t>& source_rows,
    std::size_t particle_count, const ALR20ProjectorState& projector,
    const std::vector<Binary128>& captured) {
    ALR20R63EProjectorGrams result;
    result.rows = source_rows.size();
    result.scalars = 3U * particle_count;
    if (result.rows == 0U || projector.value.size() != particle_count
        || projector.mask.size() != result.scalars
        || captured.size() != result.rows * result.rows) return result;
    std::vector<std::vector<Binary128Vec3>> clamp_rows;
    std::vector<std::vector<Binary128Vec3>> tangent_rows;
    std::vector<std::vector<Binary128Vec3>> full_rows;
    result.source_rows.reserve(result.rows);
    clamp_rows.reserve(result.rows);
    tangent_rows.reserve(result.rows);
    full_rows.reserve(result.rows);
    bool exact = projector.exact;
    for (std::size_t row : source_rows) {
        const std::vector<Binary128Vec3> source =
            al_r20_projector_row_vector(op, row, particle_count);
        const ALR20R63EVectorDecomposition decomposition =
            al_r20_r63e_decompose(projector, source);
        exact = exact && decomposition.exact;
        const std::vector<Binary128Vec3> direct =
            al_r20_projector_jvp(projector, source);
        bool direct_equal = direct.size() == decomposition.image.size();
        for (std::size_t particle = 0U;
             particle < direct.size() && direct_equal; ++particle)
            for (int axis = 0; axis < 3; ++axis)
                direct_equal = direct_equal
                    && al_r20_q_component(direct[particle], axis)
                        == al_r20_q_component(
                            decomposition.image[particle], axis);
        result.direct_jvp_exact = result.row_constructions == 0U
            ? direct_equal : result.direct_jvp_exact && direct_equal;
        result.source_rows.push_back(source);
        clamp_rows.push_back(decomposition.free);
        tangent_rows.push_back(decomposition.tangent);
        full_rows.push_back(decomposition.image);
        ++result.row_constructions;
    }
    std::vector<Binary128> raw_clamp(result.rows * result.rows);
    std::vector<Binary128> raw_tangent(result.rows * result.rows);
    std::vector<Binary128> raw_full(result.rows * result.rows);
    for (std::size_t row = 0U; row < result.rows; ++row)
        for (std::size_t column = 0U; column < result.rows; ++column) {
            raw_clamp[row * result.rows + column] =
                al_r20_projector_vector_dot(
                    result.source_rows[row], clamp_rows[column]);
            raw_tangent[row * result.rows + column] =
                al_r20_projector_vector_dot(
                    result.source_rows[row], tangent_rows[column]);
            raw_full[row * result.rows + column] =
                al_r20_projector_vector_dot(
                    result.source_rows[row], full_rows[column]);
            result.directed_dots += 3U;
        }
    result.clamp.resize(result.rows * result.rows);
    result.tangent.resize(result.rows * result.rows);
    result.full.resize(result.rows * result.rows);
    for (std::size_t row = 0U; row < result.rows; ++row)
        for (std::size_t column = 0U; column < result.rows; ++column) {
            const std::size_t index = row * result.rows + column;
            result.clamp[index] = static_cast<Binary128>(0.5)
                * (raw_clamp[index]
                    + raw_clamp[column * result.rows + row]);
            result.tangent[index] = static_cast<Binary128>(0.5)
                * (raw_tangent[index]
                    + raw_tangent[column * result.rows + row]);
            result.full[index] = static_cast<Binary128>(0.5)
                * (raw_full[index]
                    + raw_full[column * result.rows + row]);
            result.symmetrizations += 3U;
        }
    result.clamp_root = al_r20_q_scalar_root(result.clamp);
    result.tangent_root = al_r20_q_scalar_root(result.tangent);
    result.full_root = al_r20_q_scalar_root(result.full);
    result.full_matrix_exact = result.full == captured;
    result.exact = exact && result.direct_jvp_exact
        && result.full_matrix_exact
        && result.row_constructions == result.rows
        && result.directed_dots == 3U * result.rows * result.rows
        && result.symmetrizations == 3U * result.rows * result.rows;
    std::ostringstream material;
    material << result.exact << ':' << result.direct_jvp_exact << ':'
        << result.full_matrix_exact << ':' << result.rows << ':'
        << result.scalars << ':' << result.row_constructions << ':'
        << result.directed_dots << ':' << result.symmetrizations << ':'
        << result.clamp_root << ':' << result.tangent_root << ':'
        << result.full_root;
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63ERankProfile {
    bool exact = false;
    bool modular_full = false;
    ALR20R63DModularRank prime61;
    ALR20R63DModularRank prime31;
    ALR20R63DNormalized normalized;
    ALR20R63DBinary64Rank binary64;
    ALR20PivotedRank binary128;
    std::string root;
};

ALR20R63ERankProfile al_r20_r63e_rank_profile(
    const std::vector<Binary128>& matrix, std::size_t dimension) {
    ALR20R63ERankProfile result;
    result.prime61 = al_r20_r63d_modular_rank(
        matrix, dimension, dimension, AL_R20_R63D_PRIME_61);
    result.prime31 = al_r20_r63d_modular_rank(
        matrix, dimension, dimension, AL_R20_R63D_PRIME_31);
    result.modular_full = result.prime61.rank == dimension
        || result.prime31.rank == dimension;
    result.normalized = al_r20_r63d_normalize(matrix, dimension);
    if (result.normalized.exact) {
        result.binary64 = al_r20_r63d_binary64_rank(
            result.normalized.values, dimension);
        result.binary128 = al_r20_projector_pivoted_rank(
            result.normalized.values, dimension);
    }
    result.exact = result.prime61.exact && result.prime31.exact
        && result.modular_full && result.normalized.exact
        && result.binary64.exact && result.binary128.passed;
    result.root = sha256_hex(result.prime61.root + ':'
        + result.prime31.root + ':' + result.normalized.root + ':'
        + result.binary64.root + ':' + result.binary128.root);
    return result;
}

struct ALR20R63EWitness {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t scalars = 0U;
    std::size_t combination_terms = 0U;
    Binary128 gram_image_residual = static_cast<Binary128>(0.0);
    Binary128 gram_image_bound = static_cast<Binary128>(0.0);
    Binary128 gram_energy = static_cast<Binary128>(0.0);
    Binary128 tangent_energy = static_cast<Binary128>(0.0);
    Binary128 gram_energy_residual = static_cast<Binary128>(0.0);
    Binary128 gram_energy_bound = static_cast<Binary128>(0.0);
    std::vector<Binary128> alpha;
    std::vector<Binary128Vec3> source_combination;
    ALR20R63EVectorDecomposition decomposition;
    std::string root;
};

ALR20R63EWitness al_r20_r63e_witness(
    const ALR20R63EProjectorGrams& grams,
    const ALR20ProjectorState& projector,
    const ALR20PivotedRank& normalized_rank) {
    ALR20R63EWitness result;
    result.dimension = grams.rows;
    result.scalars = grams.scalars;
    if (!grams.exact || normalized_rank.rank + 1U != grams.rows
        || normalized_rank.witness.size() != grams.rows) return result;
    result.alpha.resize(grams.rows);
    bool exact = true;
    for (std::size_t row = 0U; row < grams.rows; ++row) {
        const Binary128 diagonal = grams.full[row * grams.rows + row];
        exact = exact && diagonal > static_cast<Binary128>(0.0)
            && finiteq(diagonal) != 0;
        if (!exact) break;
        result.alpha[row] = normalized_rank.witness[row] / sqrtq(diagonal);
        exact = exact && finiteq(result.alpha[row]) != 0;
    }
    std::vector<Binary128Accumulator> sums(grams.scalars);
    for (std::size_t row = 0U; row < grams.rows; ++row)
        for (std::size_t particle = 0U;
             particle < grams.source_rows[row].size(); ++particle)
            for (int axis = 0; axis < 3; ++axis) {
                const std::size_t scalar = 3U * particle
                    + static_cast<std::size_t>(axis);
                sums[scalar].add(result.alpha[row]
                    * al_r20_q_component(
                        grams.source_rows[row][particle], axis));
                ++result.combination_terms;
            }
    result.source_combination.resize(grams.scalars / 3U);
    for (std::size_t scalar = 0U; scalar < grams.scalars; ++scalar)
        al_r20_q_set_component(&result.source_combination[scalar / 3U],
            static_cast<int>(scalar % 3U), sums[scalar].value());
    result.decomposition = al_r20_r63e_decompose(
        projector, result.source_combination);

    std::vector<Binary128> gram_image(grams.rows);
    for (std::size_t row = 0U; row < grams.rows; ++row) {
        Binary128Accumulator image;
        for (std::size_t column = 0U; column < grams.rows; ++column)
            image.add(grams.full[row * grams.rows + column]
                * result.alpha[column]);
        gram_image[row] = image.value();
        const Binary128 direct = al_r20_projector_vector_dot(
            grams.source_rows[row], result.decomposition.image);
        result.gram_image_residual = fmaxq(result.gram_image_residual,
            fabsq(gram_image[row] - direct));
    }
    Binary128 scale = static_cast<Binary128>(1.0);
    for (Binary128 item : gram_image) scale = fmaxq(scale, fabsq(item));
    result.gram_image_bound = al_r20_q_gamma(
        64U * grams.rows * grams.rows + 65536U) * scale;
    Binary128Accumulator energy;
    for (std::size_t row = 0U; row < grams.rows; ++row)
        energy.add(result.alpha[row] * gram_image[row]);
    result.gram_energy = energy.value();
    result.tangent_energy = result.decomposition.tangent_norm_squared
        / (static_cast<Binary128>(1.0) + projector.eta);
    result.gram_energy_residual = fabsq(
        result.gram_energy - result.tangent_energy);
    result.gram_energy_bound = al_r20_q_gamma(
        128U * grams.rows * grams.rows + 131072U)
        * fmaxq(static_cast<Binary128>(1.0),
            fmaxq(fabsq(result.gram_energy), fabsq(result.tangent_energy)));
    result.exact = exact && result.decomposition.exact
        && result.combination_terms == grams.rows * grams.scalars
        && result.gram_image_residual <= result.gram_image_bound
        && result.gram_energy_residual <= result.gram_energy_bound;
    std::ostringstream material;
    material << result.exact << ':' << result.dimension << ':'
        << result.scalars << ':' << result.combination_terms << ':'
        << al_binary128_hex(result.gram_image_residual) << ':'
        << al_binary128_hex(result.gram_image_bound) << ':'
        << al_binary128_hex(result.gram_energy) << ':'
        << al_binary128_hex(result.tangent_energy) << ':'
        << al_binary128_hex(result.gram_energy_residual) << ':'
        << al_binary128_hex(result.gram_energy_bound) << ':'
        << al_r20_q_scalar_root(result.alpha) << ':'
        << al_r20_q_vec_root(result.source_combination) << ':'
        << result.decomposition.root;
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63e_classify(bool apparatus, bool ball_active,
    std::size_t clamp_rank, std::size_t tangent_rank,
    std::size_t full_rank, std::size_t dimension) {
    if (!apparatus) return "PROJECTOR_NULLSPACE_APPARATUS_REJECTED";
    if (clamp_rank < dimension)
        return "CLAMP_METRIC_NUMERICAL_RANK_LOSS";
    if (ball_active && tangent_rank < dimension)
        return "BALL_TANGENT_NUMERICAL_RANK_LOSS";
    if (tangent_rank == dimension && full_rank < dimension)
        return "PROJECTOR_SCALE_ARITHMETIC_BOUNDARY";
    return "PROJECTOR_COMPONENT_RANK_UNRESOLVED";
}

struct ALR20R63EControls {
    bool exact = false;
    bool decomposition = false;
    bool classifier = false;
    bool witness = false;
    bool mutation = false;
    std::string root;
};

ALR20R63EControls al_r20_r63e_controls() {
    ALR20R63EControls result;
    ALR20ProjectorState projector;
    projector.exact = true;
    projector.ball_active = true;
    projector.eta = static_cast<Binary128>(0.25);
    projector.free_norm_squared = static_cast<Binary128>(1.0);
    projector.free_components = 2U;
    projector.upper_components = 1U;
    projector.value = {{static_cast<Binary128>(1.0),
        static_cast<Binary128>(0.0), static_cast<Binary128>(0.0)}};
    projector.mask = {0, 1, 0};
    projector.root = sha256_hex("r63e-literal-projector");
    const std::vector<Binary128Vec3> direction{{
        static_cast<Binary128>(1.0), static_cast<Binary128>(2.0),
        static_cast<Binary128>(3.0)}};
    const ALR20R63EVectorDecomposition decomposition =
        al_r20_r63e_decompose(projector, direction);
    result.decomposition = decomposition.exact
        && decomposition.clamped_norm_squared == static_cast<Binary128>(4.0)
        && decomposition.radial_norm_squared == static_cast<Binary128>(1.0)
        && decomposition.tangent_norm_squared == static_cast<Binary128>(9.0)
        && fabsq(al_r20_q_component(decomposition.image[0U], 2)
                - static_cast<Binary128>(12.0)
                    / static_cast<Binary128>(5.0))
            <= al_r20_q_gamma(128U);

    result.classifier =
        al_r20_r63e_classify(true, true, 1U, 1U, 1U, 2U)
            == "CLAMP_METRIC_NUMERICAL_RANK_LOSS"
        && al_r20_r63e_classify(true, true, 2U, 1U, 1U, 2U)
            == "BALL_TANGENT_NUMERICAL_RANK_LOSS"
        && al_r20_r63e_classify(true, true, 2U, 2U, 1U, 2U)
            == "PROJECTOR_SCALE_ARITHMETIC_BOUNDARY";

    const std::vector<Binary128> normalized{
        static_cast<Binary128>(1.0), static_cast<Binary128>(1.0),
        static_cast<Binary128>(1.0), static_cast<Binary128>(1.0)};
    const ALR20PivotedRank rank =
        al_r20_projector_pivoted_rank(normalized, 2U);
    const std::vector<Binary128> diagonal{
        static_cast<Binary128>(4.0), static_cast<Binary128>(2.0),
        static_cast<Binary128>(2.0), static_cast<Binary128>(1.0)};
    Binary128Accumulator image0;
    Binary128Accumulator image1;
    if (rank.witness.size() == 2U) {
        const std::array<Binary128, 2U> alpha{
            rank.witness[0U] / static_cast<Binary128>(2.0),
            rank.witness[1U]};
        image0.add(diagonal[0U] * alpha[0U]);
        image0.add(diagonal[1U] * alpha[1U]);
        image1.add(diagonal[2U] * alpha[0U]);
        image1.add(diagonal[3U] * alpha[1U]);
    }
    result.witness = rank.passed && rank.rank == 1U
        && rank.witness.size() == 2U
        && image0.value() == static_cast<Binary128>(0.0)
        && image1.value() == static_cast<Binary128>(0.0);

    const std::string bound_root =
        al_r20_r63e_projector_material_root(projector);
    ALR20ProjectorState mutated_mask = projector;
    mutated_mask.mask[2U] = 1;
    ALR20ProjectorState mutated_value = projector;
    mutated_value.value[0U].x = nextafterq(
        mutated_value.value[0U].x, HUGE_VALQ);
    result.mutation = bound_root
            != al_r20_r63e_projector_material_root(mutated_mask)
        && bound_root
            != al_r20_r63e_projector_material_root(mutated_value);
    result.exact = result.decomposition && result.classifier
        && result.witness && result.mutation;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + decomposition.root + ':' + rank.root + ':' + bound_root + ':'
        + al_r20_r63e_projector_material_root(mutated_mask) + ':'
        + al_r20_r63e_projector_material_root(mutated_value));
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_projector_nullspace_controls() {
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
    const ALR20R63EControls controls = al_r20_r63e_controls();
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
    const ALR20ProjectorState& projector = factor_capture.projector;
    const bool projector_exact = projector.exact
        && projector.value.size() == 105U
        && projector.mask.size() == 315U
        && projector.free_components + projector.lower_components
            + projector.upper_components + projector.fixed_components == 315U
        && !projector.root.empty();
    const bool capture_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(oblique)
        && oblique.root == AL_R20_R60_CASE_SHA256
        && solution_capture.calls == 1U
        && solution_capture.dimension == AL_R20_R63_DIMENSION
        && factor_capture.calls == 125U
        && factor_capture.selected_calls == 1U
        && factor_capture.selected_column_solves == AL_R20_R63_DIMENSION
        && factor_capture.factor_passed
        && factor_capture.matrix_root == AL_R20_R63_MATRIX_SHA256
        && factor_capture.source_rows.size() == AL_R20_R63_DIMENSION
        && projector_exact && slope.calls == 0U && slope.inverse_audits == 0U;

    const ALR20R63EProjectorGrams grams = capture_exact
        ? al_r20_r63e_projector_grams(problem.op,
            factor_capture.source_rows, problem.target.size(),
            projector, factor_capture.matrix)
        : ALR20R63EProjectorGrams{};
    const ALR20R63ERankProfile clamp = grams.exact
        ? al_r20_r63e_rank_profile(grams.clamp, grams.rows)
        : ALR20R63ERankProfile{};
    const ALR20R63ERankProfile tangent = grams.exact
        ? al_r20_r63e_rank_profile(grams.tangent, grams.rows)
        : ALR20R63ERankProfile{};
    const ALR20R63ERankProfile full = grams.exact
        ? al_r20_r63e_rank_profile(grams.full, grams.rows)
        : ALR20R63ERankProfile{};
    const bool inherited_rank_exact = full.exact
        && full.binary64.rank == 101U && full.binary128.rank == 101U
        && full.binary128.witness.size() == 102U;
    const ALR20R63EWitness witness = inherited_rank_exact
        ? al_r20_r63e_witness(grams, projector, full.binary128)
        : ALR20R63EWitness{};

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool work_exact = grams.rows == 102U && grams.scalars == 315U
        && grams.row_constructions == 102U
        && grams.directed_dots == 31212U
        && grams.symmetrizations == 31212U
        && clamp.prime61.mapped_values == 10404U
        && clamp.prime31.mapped_values == 10404U
        && tangent.prime61.mapped_values == 10404U
        && tangent.prime31.mapped_values == 10404U
        && full.prime61.mapped_values == 10404U
        && full.prime31.mapped_values == 10404U
        && witness.combination_terms == 32130U;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_exact && capture_exact && grams.exact
        && clamp.exact && tangent.exact && full.exact
        && inherited_rank_exact && witness.exact
        && lifecycle_exact && work_exact;
    const std::string route = al_r20_r63e_classify(apparatus_exact,
        projector.ball_active, clamp.binary128.rank,
        tangent.binary128.rank, full.binary128.rank, 102U);
    const bool exact = apparatus_exact;

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63e-projector-nullspace.v1|"
        << AL_R20_R63E_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << oblique.root << ':' << factor_capture.matrix_root << ':'
        << controls.root << ':'
        << al_r20_r63e_projector_material_root(projector) << ':'
        << grams.root << ':' << clamp.root << ':' << tangent.root << ':'
        << full.root << ':' << witness.root;
    const std::string result_sha = sha256_hex(semantic.str());

    auto profile_json = [](std::ostringstream& report, const char* name,
                           const ALR20R63ERankProfile& profile) {
        report << ",\"" << name << "\":{\"exact\":"
            << (profile.exact ? "true" : "false")
            << ",\"modular_full\":"
            << (profile.modular_full ? "true" : "false")
            << ",\"rank_prime61\":" << profile.prime61.rank
            << ",\"rank_prime31\":" << profile.prime31.rank
            << ",\"rank_binary64\":" << profile.binary64.rank
            << ",\"threshold_binary64\":"
            << profile.binary64.threshold
            << ",\"rank_binary128\":" << profile.binary128.rank
            << ",\"threshold_binary128\":\""
            << al_binary128_text(profile.binary128.pivot_bound)
            << "\",\"rejected_binary128\":\""
            << al_binary128_text(profile.binary128.rejected_pivot)
            << "\",\"root\":\"" << profile.root << "\"}";
    };
    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63e_projector_nullspace.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63d_sha256\":\""
        << AL_R20_R63E_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls\":{\"exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"decomposition\":"
        << (controls.decomposition ? "true" : "false")
        << ",\"classifier\":"
        << (controls.classifier ? "true" : "false")
        << ",\"witness\":" << (controls.witness ? "true" : "false")
        << ",\"mutation\":" << (controls.mutation ? "true" : "false")
        << ",\"root\":\"" << controls.root << "\"}"
        << ",\"capture\":{\"exact\":"
        << (capture_exact ? "true" : "false")
        << ",\"observed_inverse_calls\":" << factor_capture.calls
        << ",\"selected_inverse_calls\":" << factor_capture.selected_calls
        << ",\"selected_columns\":"
        << factor_capture.selected_column_solves
        << ",\"matrix_root\":\"" << factor_capture.matrix_root << "\"}"
        << ",\"projector\":{\"exact\":"
        << (projector_exact ? "true" : "false")
        << ",\"ball_active\":"
        << (projector.ball_active ? "true" : "false")
        << ",\"eta\":\"" << al_binary128_text(projector.eta)
        << "\",\"free_components\":" << projector.free_components
        << ",\"lower_components\":" << projector.lower_components
        << ",\"upper_components\":" << projector.upper_components
        << ",\"fixed_components\":" << projector.fixed_components
        << ",\"root\":\"" << projector.root << "\"}"
        << ",\"grams\":{\"exact\":"
        << (grams.exact ? "true" : "false")
        << ",\"direct_jvp_exact\":"
        << (grams.direct_jvp_exact ? "true" : "false")
        << ",\"full_matrix_exact\":"
        << (grams.full_matrix_exact ? "true" : "false")
        << ",\"clamp_root\":\"" << grams.clamp_root
        << "\",\"tangent_root\":\"" << grams.tangent_root
        << "\",\"full_root\":\"" << grams.full_root << "\"}";
    profile_json(report, "clamp_profile", clamp);
    profile_json(report, "tangent_profile", tangent);
    profile_json(report, "full_profile", full);
    report << ",\"witness\":{\"exact\":"
        << (witness.exact ? "true" : "false")
        << ",\"combination_terms\":" << witness.combination_terms
        << ",\"source_norm_squared\":\""
        << al_binary128_text(witness.decomposition.source_norm_squared)
        << "\",\"clamped_norm_squared\":\""
        << al_binary128_text(witness.decomposition.clamped_norm_squared)
        << "\",\"radial_norm_squared\":\""
        << al_binary128_text(witness.decomposition.radial_norm_squared)
        << "\",\"tangent_norm_squared\":\""
        << al_binary128_text(witness.decomposition.tangent_norm_squared)
        << "\",\"image_norm_squared\":\""
        << al_binary128_text(witness.decomposition.image_norm_squared)
        << "\",\"reconstruction_residual\":\""
        << al_binary128_text(
            witness.decomposition.reconstruction_residual)
        << "\",\"gram_image_residual\":\""
        << al_binary128_text(witness.gram_image_residual)
        << "\",\"gram_energy\":\""
        << al_binary128_text(witness.gram_energy)
        << "\",\"tangent_energy\":\""
        << al_binary128_text(witness.tangent_energy)
        << "\",\"root\":\"" << witness.root << "\"}"
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"nnqp_rhs_solves\":0,\"rank_actions\":0,\"row_drops\":0"
        << ",\"center_iterations\":0,\"replacement_applies\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
