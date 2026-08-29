namespace {

constexpr const char* AL_R20_MPADMM_SHA256 =
    "cd094b7fe5cb7ca5a713f60185493250fb9d66ed3221038eb1c8a0f0a25db486";
constexpr std::size_t AL_R20_PROJECTOR_PROBES = 6U;

Binary128 al_r20_projector_vector_dot(
    const std::vector<Binary128Vec3>& lhs,
    const std::vector<Binary128Vec3>& rhs) {
    Binary128Accumulator sum;
    for (std::size_t index = 0U; index < lhs.size(); ++index)
        sum.add(al_r20_q_dot(lhs[index], rhs[index]));
    return sum.value();
}

Binary128 al_r20_projector_vector_norm(
    const std::vector<Binary128Vec3>& value) {
    return sqrtq(fmaxq(static_cast<Binary128>(0.0),
        al_r20_projector_vector_dot(value, value)));
}

std::vector<Binary128Vec3> al_r20_projector_promote_bounds(
    const std::vector<Vec3>& values) {
    return al_r20_q_promote(values);
}

int al_r20_projector_mask_component(
    const std::vector<int>& mask, std::size_t particle, int axis) {
    return mask[3U * particle + static_cast<std::size_t>(axis)];
}

struct ALR20ProjectorState {
    bool exact = false;
    bool finite = false;
    bool ball_active = false;
    Binary128 eta = static_cast<Binary128>(0.0);
    Binary128 norm_squared = static_cast<Binary128>(0.0);
    Binary128 free_norm_squared = static_cast<Binary128>(0.0);
    Binary128 remaining_radius_squared = static_cast<Binary128>(0.0);
    Binary128 kkt_residual = static_cast<Binary128>(0.0);
    Binary128 kkt_bound = static_cast<Binary128>(0.0);
    Binary128 minimum_free_margin = static_cast<Binary128>(0.0);
    Binary128 minimum_clamped_margin = static_cast<Binary128>(0.0);
    std::size_t free_components = 0U;
    std::size_t lower_components = 0U;
    std::size_t upper_components = 0U;
    std::size_t fixed_components = 0U;
    std::vector<Binary128Vec3> value;
    std::vector<int> mask;
    std::string root;
};

ALR20ProjectorState al_r20_projector_state(
    const std::vector<Binary128Vec3>& input,
    const std::vector<Binary128Vec3>& lower,
    const std::vector<Binary128Vec3>& upper,
    Binary128 radius) {
    ALR20ProjectorState result;
    const std::size_t particles = input.size();
    if (lower.size() != particles || upper.size() != particles
        || !(radius > static_cast<Binary128>(0.0))) return result;
    result.value.resize(particles);
    result.mask.assign(3U * particles, 0);
    const Binary128 radius_squared = radius * radius;
    const auto norm_at = [&](Binary128 eta) {
        Binary128Accumulator sum;
        const Binary128 divisor = static_cast<Binary128>(1.0) + eta;
        for (std::size_t particle = 0U; particle < particles; ++particle)
            for (int axis = 0; axis < 3; ++axis) {
                const Binary128 item = al_r20_q_clamp(
                    al_r20_q_component(input[particle], axis) / divisor,
                    al_r20_q_component(lower[particle], axis),
                    al_r20_q_component(upper[particle], axis));
                sum.add(item * item);
            }
        return sum.value();
    };

    result.eta = static_cast<Binary128>(0.0);
    const Binary128 box_norm_squared = norm_at(result.eta);
    bool bracketed = true;
    if (box_norm_squared > radius_squared) {
        result.ball_active = true;
        Binary128 low = static_cast<Binary128>(0.0);
        Binary128 high = static_cast<Binary128>(1.0);
        std::size_t expansions = 0U;
        while (norm_at(high) > radius_squared && expansions < 512U) {
            high *= static_cast<Binary128>(2.0);
            ++expansions;
        }
        bracketed = norm_at(high) <= radius_squared;
        for (std::size_t iteration = 0U;
             iteration < 256U && bracketed; ++iteration) {
            const Binary128 middle = static_cast<Binary128>(0.5) * (low + high);
            if (middle == low || middle == high) break;
            if (norm_at(middle) > radius_squared) low = middle;
            else high = middle;
        }
        result.eta = high;
    }

    const Binary128 divisor = static_cast<Binary128>(1.0) + result.eta;
    Binary128Accumulator norm_sum;
    Binary128Accumulator free_norm_sum;
    Binary128Accumulator clamped_norm_sum;
    Binary128 scale = fmaxq(static_cast<Binary128>(1.0), radius);
    Binary128 free_margin = HUGE_VALQ;
    Binary128 clamped_margin = HUGE_VALQ;
    for (std::size_t particle = 0U; particle < particles; ++particle)
        for (int axis = 0; axis < 3; ++axis) {
            const Binary128 z = al_r20_q_component(input[particle], axis);
            const Binary128 lo = al_r20_q_component(lower[particle], axis);
            const Binary128 hi = al_r20_q_component(upper[particle], axis);
            if (!(lo <= hi) || finiteq(z) == 0 || finiteq(lo) == 0
                || finiteq(hi) == 0) bracketed = false;
            const Binary128 y = al_r20_q_clamp(z / divisor, lo, hi);
            al_r20_q_set_component(&result.value[particle], axis, y);
            const std::size_t scalar = 3U * particle
                + static_cast<std::size_t>(axis);
            if (lo == hi) {
                result.mask[scalar] = 2;
                ++result.fixed_components;
                clamped_norm_sum.add(y * y);
            } else if (y == lo) {
                result.mask[scalar] = -1;
                ++result.lower_components;
                clamped_norm_sum.add(y * y);
                clamped_margin = fminq(clamped_margin, divisor * lo - z);
            } else if (y == hi) {
                result.mask[scalar] = 1;
                ++result.upper_components;
                clamped_norm_sum.add(y * y);
                clamped_margin = fminq(clamped_margin, z - divisor * hi);
            } else {
                ++result.free_components;
                free_norm_sum.add(y * y);
                free_margin = fminq(free_margin,
                    fminq(y - lo, hi - y));
            }
            norm_sum.add(y * y);
            scale = fmaxq(scale,
                fmaxq(fabsq(z), fmaxq(fabsq(lo), fabsq(hi))));
        }
    result.norm_squared = norm_sum.value();
    result.free_norm_squared = free_norm_sum.value();
    result.remaining_radius_squared = radius_squared - clamped_norm_sum.value();
    result.minimum_free_margin = result.free_components == 0U
        ? static_cast<Binary128>(0.0) : free_margin;
    result.minimum_clamped_margin =
        result.lower_components + result.upper_components == 0U
        ? static_cast<Binary128>(0.0) : clamped_margin;
    result.kkt_bound = al_r20_q_gamma(128U * particles + 4096U)
        * scale * scale;
    Binary128 residual = fmaxq(static_cast<Binary128>(0.0),
        result.norm_squared - radius_squared);
    if (result.ball_active)
        residual = fmaxq(residual,
            fabsq(result.norm_squared - radius_squared));
    for (std::size_t particle = 0U; particle < particles; ++particle)
        for (int axis = 0; axis < 3; ++axis) {
            const Binary128 z = al_r20_q_component(input[particle], axis);
            const Binary128 y = al_r20_q_component(result.value[particle], axis);
            const int mask = al_r20_projector_mask_component(
                result.mask, particle, axis);
            const Binary128 relation = z - divisor * y;
            if (mask == 0) residual = fmaxq(residual, fabsq(relation));
            else if (mask == -1)
                residual = fmaxq(residual,
                    fmaxq(static_cast<Binary128>(0.0), relation));
            else if (mask == 1)
                residual = fmaxq(residual,
                    fmaxq(static_cast<Binary128>(0.0), -relation));
        }
    result.kkt_residual = residual;
    result.finite = bracketed && finiteq(result.eta) != 0
        && finiteq(result.norm_squared) != 0
        && finiteq(result.free_norm_squared) != 0
        && finiteq(result.remaining_radius_squared) != 0
        && finiteq(result.kkt_residual) != 0
        && finiteq(result.kkt_bound) != 0;
    const bool derivative_regular = !result.ball_active
        || result.free_norm_squared > result.kkt_bound;
    result.exact = result.finite && derivative_regular
        && result.eta >= static_cast<Binary128>(0.0)
        && result.kkt_residual <= result.kkt_bound;
    std::ostringstream material;
    material << result.exact << ':' << result.ball_active << ':'
             << al_binary128_hex(result.eta) << ':'
             << al_binary128_hex(result.norm_squared) << ':'
             << al_binary128_hex(result.free_norm_squared) << ':'
             << al_binary128_hex(result.remaining_radius_squared) << ':'
             << al_binary128_hex(result.kkt_residual) << ':'
             << al_binary128_hex(result.kkt_bound) << '|';
    for (int item : result.mask) material << item << ',';
    material << '|' << al_r20_q_vec_root(result.value);
    result.root = sha256_hex(material.str());
    return result;
}

std::vector<Binary128Vec3> al_r20_projector_jvp(
    const ALR20ProjectorState& state,
    const std::vector<Binary128Vec3>& direction) {
    std::vector<Binary128Vec3> result(direction.size());
    Binary128 radial = static_cast<Binary128>(0.0);
    if (state.ball_active) {
        Binary128Accumulator sum;
        for (std::size_t particle = 0U; particle < direction.size(); ++particle)
            for (int axis = 0; axis < 3; ++axis)
                if (al_r20_projector_mask_component(
                        state.mask, particle, axis) == 0)
                    sum.add(al_r20_q_component(state.value[particle], axis)
                        * al_r20_q_component(direction[particle], axis));
        radial = sum.value() / state.free_norm_squared;
    }
    const Binary128 inverse = static_cast<Binary128>(1.0)
        / (static_cast<Binary128>(1.0) + state.eta);
    for (std::size_t particle = 0U; particle < direction.size(); ++particle)
        for (int axis = 0; axis < 3; ++axis)
            if (al_r20_projector_mask_component(
                    state.mask, particle, axis) == 0) {
                Binary128 value = al_r20_q_component(direction[particle], axis);
                if (state.ball_active)
                    value -= al_r20_q_component(state.value[particle], axis)
                        * radial;
                al_r20_q_set_component(&result[particle], axis,
                    inverse * value);
            }
    return result;
}

std::vector<Binary128Vec3> al_r20_projector_probe_direction(
    const ALR20ProjectorState& state, std::size_t probe) {
    std::vector<Binary128Vec3> result(state.value.size());
    if (!state.ball_active) {
        std::vector<std::size_t> free;
        for (std::size_t scalar = 0U; scalar < state.mask.size(); ++scalar)
            if (state.mask[scalar] == 0) free.push_back(scalar);
        if (!free.empty()) {
            const std::size_t scalar = free[probe % free.size()];
            al_r20_q_set_component(&result[scalar / 3U],
                static_cast<int>(scalar % 3U),
                static_cast<Binary128>(1.0));
        }
        return result;
    }
    for (std::size_t particle = 0U; particle < result.size(); ++particle)
        for (int axis = 0; axis < 3; ++axis) {
            const std::size_t scalar = 3U * particle
                + static_cast<std::size_t>(axis);
            if (state.mask[scalar] != 0) continue;
            const std::size_t code = ((scalar + 3U) * (probe + 5U)
                + 7U * probe * probe + 11U) % 29U;
            al_r20_q_set_component(&result[particle], axis,
                static_cast<Binary128>(static_cast<int>(code) - 14));
        }
    if (state.ball_active) {
        Binary128Accumulator dot;
        for (std::size_t particle = 0U; particle < result.size(); ++particle)
            for (int axis = 0; axis < 3; ++axis)
                if (al_r20_projector_mask_component(
                        state.mask, particle, axis) == 0)
                    dot.add(al_r20_q_component(state.value[particle], axis)
                        * al_r20_q_component(result[particle], axis));
        const Binary128 radial = dot.value() / state.free_norm_squared;
        for (std::size_t particle = 0U; particle < result.size(); ++particle)
            for (int axis = 0; axis < 3; ++axis)
                if (al_r20_projector_mask_component(
                        state.mask, particle, axis) == 0)
                    al_r20_q_set_component(&result[particle], axis,
                        al_r20_q_component(result[particle], axis)
                        - radial * al_r20_q_component(
                            state.value[particle], axis));
    }
    Binary128 norm = al_r20_projector_vector_norm(result);
    if (!(norm > static_cast<Binary128>(0.0))) {
        std::size_t first = state.mask.size();
        std::size_t second = state.mask.size();
        for (std::size_t scalar = 0U; scalar < state.mask.size(); ++scalar)
            if (state.mask[scalar] == 0) {
                if (first == state.mask.size()) first = scalar;
                else { second = scalar; break; }
            }
        if (!state.ball_active && first < state.mask.size()) {
            al_r20_q_set_component(&result[first / 3U],
                static_cast<int>(first % 3U), static_cast<Binary128>(1.0));
        } else if (second < state.mask.size()) {
            const Binary128 first_value = al_r20_q_component(
                state.value[first / 3U], static_cast<int>(first % 3U));
            const Binary128 second_value = al_r20_q_component(
                state.value[second / 3U], static_cast<int>(second % 3U));
            al_r20_q_set_component(&result[first / 3U],
                static_cast<int>(first % 3U), second_value);
            al_r20_q_set_component(&result[second / 3U],
                static_cast<int>(second % 3U), -first_value);
        }
        norm = al_r20_projector_vector_norm(result);
    }
    if (norm > static_cast<Binary128>(0.0)) {
        const Binary128 inverse = static_cast<Binary128>(1.0) / norm;
        for (Binary128Vec3& item : result)
            item = al_r20_q_scale(inverse, item);
    }
    return result;
}

struct ALR20DerivativeProbe {
    bool passed = false;
    bool mask_stable = false;
    Binary128 tangent_residual = static_cast<Binary128>(0.0);
    Binary128 error_20 = static_cast<Binary128>(0.0);
    Binary128 error_24 = static_cast<Binary128>(0.0);
    Binary128 agreement_bound = static_cast<Binary128>(0.0);
    Binary128 symmetry_residual = static_cast<Binary128>(0.0);
    Binary128 psd_value = static_cast<Binary128>(0.0);
    Binary128 norm_excess = static_cast<Binary128>(0.0);
    std::string root;
};

ALR20DerivativeProbe al_r20_projector_probe(
    const ALR20ProjectorState& state,
    const std::vector<Binary128Vec3>& input,
    const std::vector<Binary128Vec3>& lower,
    const std::vector<Binary128Vec3>& upper,
    Binary128 radius, std::size_t probe) {
    ALR20DerivativeProbe result;
    const std::vector<Binary128Vec3> direction =
        al_r20_projector_probe_direction(state, probe);
    const std::vector<Binary128Vec3> image =
        al_r20_projector_jvp(state, direction);
    const Binary128 direction_norm = al_r20_projector_vector_norm(direction);
    const Binary128 image_norm = al_r20_projector_vector_norm(image);
    if (state.ball_active) {
        Binary128Accumulator tangent;
        for (std::size_t particle = 0U; particle < direction.size(); ++particle)
            for (int axis = 0; axis < 3; ++axis)
                if (al_r20_projector_mask_component(
                        state.mask, particle, axis) == 0)
                    tangent.add(al_r20_q_component(state.value[particle], axis)
                        * al_r20_q_component(direction[particle], axis));
        result.tangent_residual = fabsq(tangent.value());
    }
    const std::array<int, 2U> powers{{20, 24}};
    std::array<Binary128, 2U> errors{};
    bool exact = direction_norm > static_cast<Binary128>(0.0);
    result.mask_stable = exact;
    for (std::size_t level = 0U; level < powers.size(); ++level) {
        const Binary128 epsilon = ldexpq(
            static_cast<Binary128>(1.0), -powers[level]);
        std::vector<Binary128Vec3> plus(input.size());
        std::vector<Binary128Vec3> minus(input.size());
        for (std::size_t particle = 0U; particle < input.size(); ++particle) {
            plus[particle] = al_r20_q_add(input[particle],
                al_r20_q_scale(epsilon, direction[particle]));
            minus[particle] = al_r20_q_add(input[particle],
                al_r20_q_scale(-epsilon, direction[particle]));
        }
        const ALR20ProjectorState plus_state = al_r20_projector_state(
            plus, lower, upper, radius);
        const ALR20ProjectorState minus_state = al_r20_projector_state(
            minus, lower, upper, radius);
        const bool masks_equal = plus_state.mask == state.mask
            && minus_state.mask == state.mask
            && plus_state.ball_active == state.ball_active
            && minus_state.ball_active == state.ball_active;
        result.mask_stable = result.mask_stable && masks_equal;
        exact = exact && plus_state.exact && minus_state.exact && masks_equal;
        const Binary128 inverse = static_cast<Binary128>(0.5) / epsilon;
        Binary128 error = static_cast<Binary128>(0.0);
        for (std::size_t particle = 0U; particle < input.size(); ++particle) {
            const Binary128Vec3 central = al_r20_q_scale(inverse,
                plus_state.value[particle] - minus_state.value[particle]);
            error = fmaxq(error,
                al_r20_q_absmax(central - image[particle]));
        }
        errors[level] = error;
    }
    result.error_20 = errors[0];
    result.error_24 = errors[1];
    const Binary128 epsilon_24 = ldexpq(static_cast<Binary128>(1.0), -24);
    const Binary128 curvature_scale = static_cast<Binary128>(256.0)
        / fmaxq(radius * radius, ldexpq(static_cast<Binary128>(1.0), -100));
    const Binary128 numeric_bound = al_r20_q_gamma(
        256U * input.size() + 8192U) / epsilon_24;
    result.agreement_bound = curvature_scale * epsilon_24 * epsilon_24
        + numeric_bound;
    const std::vector<Binary128Vec3> companion =
        al_r20_projector_probe_direction(
            state, (probe + 1U) % AL_R20_PROJECTOR_PROBES);
    const std::vector<Binary128Vec3> companion_image =
        al_r20_projector_jvp(state, companion);
    result.symmetry_residual = fabsq(
        al_r20_projector_vector_dot(direction, companion_image)
        - al_r20_projector_vector_dot(companion, image));
    result.psd_value = al_r20_projector_vector_dot(direction, image);
    result.norm_excess = fmaxq(static_cast<Binary128>(0.0),
        image_norm - direction_norm);
    const Binary128 algebra_bound = al_r20_q_gamma(
        128U * input.size() + 4096U);
    const Binary128 tangent_bound = algebra_bound
        * fmaxq(static_cast<Binary128>(1.0),
            sqrtq(fmaxq(static_cast<Binary128>(0.0),
                state.free_norm_squared)));
    const bool contracted = result.error_24
        <= static_cast<Binary128>(0.25) * result.error_20 + numeric_bound;
    result.passed = exact && result.error_24 <= result.agreement_bound
        && contracted && result.tangent_residual <= tangent_bound
        && result.symmetry_residual <= algebra_bound
        && result.psd_value >= -algebra_bound
        && result.norm_excess <= algebra_bound;
    std::ostringstream material;
    material << result.passed << ':' << result.mask_stable << ':'
             << al_binary128_hex(result.tangent_residual) << ':'
             << al_binary128_hex(result.error_20) << ':'
             << al_binary128_hex(result.error_24) << ':'
             << al_binary128_hex(result.agreement_bound) << ':'
             << al_binary128_hex(result.symmetry_residual) << ':'
             << al_binary128_hex(result.psd_value) << ':'
             << al_binary128_hex(result.norm_excess) << ':'
             << al_r20_q_vec_root(direction) << ':'
             << al_r20_q_vec_root(image);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20PivotedRank {
    bool passed = false;
    bool negative_pivot = false;
    std::size_t dimension = 0U;
    std::size_t rank = 0U;
    Binary128 scale = static_cast<Binary128>(0.0);
    Binary128 pivot_bound = static_cast<Binary128>(0.0);
    Binary128 minimum_positive_pivot = static_cast<Binary128>(0.0);
    Binary128 rejected_pivot = static_cast<Binary128>(0.0);
    Binary128 reconstruction_residual = static_cast<Binary128>(0.0);
    Binary128 reconstruction_bound = static_cast<Binary128>(0.0);
    Binary128 dependency_residual = static_cast<Binary128>(0.0);
    std::vector<std::size_t> permutation;
    std::vector<Binary128> witness;
    std::string dependency_root;
    std::string root;
};

ALR20PivotedRank al_r20_projector_pivoted_rank(
    const std::vector<Binary128>& matrix, std::size_t dimension) {
    ALR20PivotedRank result;
    result.dimension = dimension;
    result.permutation.resize(dimension);
    std::iota(result.permutation.begin(), result.permutation.end(), 0U);
    if (matrix.size() != dimension * dimension) return result;
    if (dimension == 0U) {
        result.passed = true;
        result.dependency_root = sha256_hex("empty");
        result.root = result.dependency_root;
        return result;
    }
    std::vector<Binary128> schur = matrix;
    std::vector<Binary128> lower(dimension * dimension,
        static_cast<Binary128>(0.0));
    for (Binary128 value : matrix) {
        if (finiteq(value) == 0) return result;
        result.scale = fmaxq(result.scale, fabsq(value));
    }
    result.scale = fmaxq(result.scale,
        ldexpq(static_cast<Binary128>(1.0), -16382));
    result.pivot_bound = al_r20_q_gamma(
        512U * dimension * dimension + 8192U) * result.scale;
    result.reconstruction_bound = static_cast<Binary128>(64.0)
        * result.pivot_bound;
    result.minimum_positive_pivot = HUGE_VALQ;
    std::size_t rank = 0U;
    for (std::size_t column = 0U; column < dimension; ++column) {
        std::size_t pivot_index = column;
        Binary128 pivot = schur[column * dimension + column];
        for (std::size_t candidate = column + 1U;
             candidate < dimension; ++candidate)
            if (schur[candidate * dimension + candidate] > pivot) {
                pivot = schur[candidate * dimension + candidate];
                pivot_index = candidate;
            }
        if (pivot <= result.pivot_bound) {
            result.rejected_pivot = pivot;
            if (pivot < -result.pivot_bound) result.negative_pivot = true;
            break;
        }
        if (pivot_index != column) {
            for (std::size_t index = 0U; index < dimension; ++index)
                std::swap(schur[column * dimension + index],
                    schur[pivot_index * dimension + index]);
            for (std::size_t index = 0U; index < dimension; ++index)
                std::swap(schur[index * dimension + column],
                    schur[index * dimension + pivot_index]);
            for (std::size_t index = 0U; index < column; ++index)
                std::swap(lower[column * dimension + index],
                    lower[pivot_index * dimension + index]);
            std::swap(result.permutation[column],
                result.permutation[pivot_index]);
            pivot = schur[column * dimension + column];
        }
        lower[column * dimension + column] = sqrtq(pivot);
        result.minimum_positive_pivot = fminq(
            result.minimum_positive_pivot, pivot);
        for (std::size_t row = column + 1U; row < dimension; ++row)
            lower[row * dimension + column] =
                schur[row * dimension + column]
                / lower[column * dimension + column];
        for (std::size_t row = column + 1U; row < dimension; ++row)
            for (std::size_t other = row; other < dimension; ++other) {
                const Binary128 updated = schur[other * dimension + row]
                    - lower[other * dimension + column]
                    * lower[row * dimension + column];
                schur[other * dimension + row] = updated;
                schur[row * dimension + other] = updated;
            }
        ++rank;
    }
    result.rank = rank;
    if (rank == dimension)
        result.rejected_pivot = static_cast<Binary128>(0.0);
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < dimension; ++column) {
            Binary128Accumulator reconstructed;
            for (std::size_t inner = 0U; inner < rank; ++inner)
                reconstructed.add(lower[row * dimension + inner]
                    * lower[column * dimension + inner]);
            const Binary128 expected = matrix[
                result.permutation[row] * dimension
                + result.permutation[column]];
            result.reconstruction_residual = fmaxq(
                result.reconstruction_residual,
                fabsq(reconstructed.value() - expected));
        }
    std::vector<Binary128> witness(dimension,
        static_cast<Binary128>(0.0));
    if (rank < dimension) {
        const std::size_t trailing = rank;
        std::vector<Binary128> alpha(rank,
            static_cast<Binary128>(0.0));
        for (std::size_t reverse = rank; reverse-- > 0U;) {
            Binary128Accumulator sum;
            for (std::size_t row = reverse + 1U; row < rank; ++row)
                sum.add(lower[row * dimension + reverse] * alpha[row]);
            alpha[reverse] = (lower[trailing * dimension + reverse]
                - sum.value()) / lower[reverse * dimension + reverse];
        }
        witness[result.permutation[trailing]] = static_cast<Binary128>(1.0);
        for (std::size_t row = 0U; row < rank; ++row)
            witness[result.permutation[row]] = -alpha[row];
        for (std::size_t row = 0U; row < dimension; ++row) {
            Binary128Accumulator image;
            for (std::size_t column = 0U; column < dimension; ++column)
                image.add(matrix[row * dimension + column] * witness[column]);
            result.dependency_residual = fmaxq(
                result.dependency_residual, fabsq(image.value()));
        }
    }
    result.dependency_root = al_r20_q_scalar_root(witness);
    result.witness = witness;
    Binary128 witness_scale = static_cast<Binary128>(1.0);
    for (Binary128 item : witness)
        witness_scale += fabsq(item);
    const bool dependency_exact = rank == dimension
        || result.dependency_residual
            <= result.reconstruction_bound * witness_scale;
    result.passed = !result.negative_pivot
        && result.reconstruction_residual <= result.reconstruction_bound
        && dependency_exact;
    std::ostringstream material;
    material << result.passed << ':' << result.negative_pivot << ':'
             << dimension << ':' << rank << ':'
             << al_binary128_hex(result.scale) << ':'
             << al_binary128_hex(result.pivot_bound) << ':'
             << al_binary128_hex(result.minimum_positive_pivot) << ':'
             << al_binary128_hex(result.rejected_pivot) << ':'
             << al_binary128_hex(result.reconstruction_residual) << ':'
             << al_binary128_hex(result.reconstruction_bound) << ':'
             << al_binary128_hex(result.dependency_residual) << ':'
             << result.dependency_root << '|';
    for (std::size_t item : result.permutation) material << item << ',';
    result.root = sha256_hex(material.str());
    return result;
}

std::vector<Binary128Vec3> al_r20_projector_row_vector(
    const ALR64Operator& op, std::size_t row,
    std::size_t particle_count) {
    std::vector<Binary128Vec3> result(particle_count);
    for (std::size_t index = op.entry_offsets[row];
         index < op.entry_offsets[row + 1U]; ++index) {
        const ALR64Entry& entry = op.entries[index];
        result[entry.particle] = al_r20_q_add(result[entry.particle],
            al_binary128_vec3(entry.coefficient));
    }
    return result;
}

struct ALR20ProjectorCase {
    bool exact = false;
    bool derivative_exact = false;
    bool face_exact = false;
    std::size_t positive_rows = 0U;
    std::size_t near_zero_rows = 0U;
    Binary128 maximum_probe_error_20 = static_cast<Binary128>(0.0);
    Binary128 maximum_probe_error_24 = static_cast<Binary128>(0.0);
    Binary128 face_symmetry_residual = static_cast<Binary128>(0.0);
    Binary128 face_symmetry_bound = static_cast<Binary128>(0.0);
    ALR20ProjectorState projector;
    std::vector<ALR20DerivativeProbe> probes;
    ALR20PivotedRank face_rank;
    std::string active_rows_root;
    std::string face_matrix_root;
    std::string root;
};

ALR20ProjectorCase al_r20_projector_case(
    const ALR20ProblemData& problem) {
    ALR20ProjectorCase result;
    const std::vector<Binary128Vec3> input = al_r20_q_promote(problem.target);
    const std::vector<Binary128Vec3> lower =
        al_r20_projector_promote_bounds(problem.lower);
    const std::vector<Binary128Vec3> upper =
        al_r20_projector_promote_bounds(problem.upper);
    const Binary128 radius = static_cast<Binary128>(AL_R20_TRUST_RADIUS);
    result.projector = al_r20_projector_state(input, lower, upper, radius);
    result.derivative_exact = result.projector.exact;
    for (std::size_t probe = 0U;
         probe < AL_R20_PROJECTOR_PROBES; ++probe) {
        ALR20DerivativeProbe audit = al_r20_projector_probe(
            result.projector, input, lower, upper, radius, probe);
        result.maximum_probe_error_20 = fmaxq(
            result.maximum_probe_error_20, audit.error_20);
        result.maximum_probe_error_24 = fmaxq(
            result.maximum_probe_error_24, audit.error_24);
        result.derivative_exact = result.derivative_exact && audit.passed;
        result.probes.push_back(std::move(audit));
    }

    std::vector<Binary128> action_bound;
    const std::vector<Binary128> raw = al_r20_q_action(problem.op,
        result.projector.value, problem.workspace.support.constraint,
        &action_bound);
    std::vector<std::size_t> active_rows;
    for (std::size_t row = 0U; row < raw.size(); ++row) {
        if (raw[row] > action_bound[row]) active_rows.push_back(row);
        else if (fabsq(raw[row]) <= action_bound[row]) ++result.near_zero_rows;
    }
    result.positive_rows = active_rows.size();
    std::ostringstream active_material;
    for (std::size_t row : active_rows) active_material << row << ',';
    result.active_rows_root = sha256_hex(active_material.str());

    const std::size_t dimension = active_rows.size();
    std::vector<std::vector<Binary128Vec3>> rows;
    std::vector<std::vector<Binary128Vec3>> images;
    rows.reserve(dimension);
    images.reserve(dimension);
    for (std::size_t row : active_rows) {
        rows.push_back(al_r20_projector_row_vector(
            problem.op, row, input.size()));
        images.push_back(al_r20_projector_jvp(
            result.projector, rows.back()));
    }
    std::vector<Binary128> matrix(dimension * dimension,
        static_cast<Binary128>(0.0));
    Binary128 matrix_scale = static_cast<Binary128>(0.0);
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < dimension; ++column) {
            matrix[row * dimension + column] =
                al_r20_projector_vector_dot(rows[row], images[column]);
            matrix_scale = fmaxq(matrix_scale,
                fabsq(matrix[row * dimension + column]));
        }
    result.face_symmetry_bound = al_r20_q_gamma(
        128U * input.size() + 8192U)
        * fmaxq(matrix_scale,
            ldexpq(static_cast<Binary128>(1.0), -16382));
    std::vector<Binary128> symmetric = matrix;
    for (std::size_t row = 0U; row < dimension; ++row)
        for (std::size_t column = 0U; column < row; ++column) {
            result.face_symmetry_residual = fmaxq(
                result.face_symmetry_residual,
                fabsq(matrix[row * dimension + column]
                    - matrix[column * dimension + row]));
            const Binary128 average = static_cast<Binary128>(0.5)
                * (matrix[row * dimension + column]
                    + matrix[column * dimension + row]);
            symmetric[row * dimension + column] = average;
            symmetric[column * dimension + row] = average;
        }
    result.face_rank = al_r20_projector_pivoted_rank(symmetric, dimension);
    result.face_matrix_root = al_r20_q_scalar_root(matrix);
    result.face_exact = result.derivative_exact
        && result.face_symmetry_residual <= result.face_symmetry_bound
        && result.face_rank.passed;
    result.exact = result.projector.exact && result.derivative_exact
        && result.face_exact;
    std::ostringstream material;
    material << result.exact << ':' << result.derivative_exact << ':'
             << result.face_exact << ':' << result.positive_rows << ':'
             << result.near_zero_rows << ':'
             << al_binary128_hex(result.maximum_probe_error_20) << ':'
             << al_binary128_hex(result.maximum_probe_error_24) << ':'
             << al_binary128_hex(result.face_symmetry_residual) << ':'
             << al_binary128_hex(result.face_symmetry_bound) << ':'
             << result.projector.root << ':' << result.active_rows_root << ':'
             << result.face_matrix_root << ':' << result.face_rank.root << '|';
    for (const ALR20DerivativeProbe& probe : result.probes)
        material << probe.root << ';';
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20ProjectorControls {
    bool inactive_ball = false;
    bool active_ball_tangent = false;
    bool clamps = false;
    bool duplicate_rank = false;
    bool negative_pivot = false;
    std::string root;
};

ALR20ProjectorControls al_r20_projector_controls() {
    ALR20ProjectorControls result;
    const Binary128 zero = static_cast<Binary128>(0.0);
    const Binary128 one = static_cast<Binary128>(1.0);
    const std::vector<Binary128Vec3> lower{{-one, -one, -one}};
    const std::vector<Binary128Vec3> upper{{one, one, one}};
    const ALR20ProjectorState inactive = al_r20_projector_state(
        {{static_cast<Binary128>(0.25), static_cast<Binary128>(-0.5),
            static_cast<Binary128>(0.75)}}, lower, upper,
        static_cast<Binary128>(2.0));
    const std::vector<Binary128Vec3> inactive_direction{{one, -one, one}};
    const std::vector<Binary128Vec3> inactive_image =
        al_r20_projector_jvp(inactive, inactive_direction);
    result.inactive_ball = inactive.exact && !inactive.ball_active
        && al_r20_q_absmax(inactive_image[0] - inactive_direction[0]) == zero;

    const std::vector<Binary128Vec3> wide_lower{{
        static_cast<Binary128>(-10.0), static_cast<Binary128>(-10.0),
        static_cast<Binary128>(-10.0)}};
    const std::vector<Binary128Vec3> wide_upper{{
        static_cast<Binary128>(10.0), static_cast<Binary128>(10.0),
        static_cast<Binary128>(10.0)}};
    const ALR20ProjectorState active = al_r20_projector_state(
        {{static_cast<Binary128>(2.0), one, zero}},
        wide_lower, wide_upper, one);
    const std::vector<Binary128Vec3> tangent{{-one,
        static_cast<Binary128>(2.0), zero}};
    const std::vector<Binary128Vec3> tangent_image =
        al_r20_projector_jvp(active, tangent);
    const Binary128 tangent_scale = one / sqrtq(static_cast<Binary128>(5.0));
    const Binary128 control_bound = al_r20_q_gamma(8192U);
    result.active_ball_tangent = active.exact && active.ball_active
        && al_r20_q_absmax(tangent_image[0]
            - al_r20_q_scale(tangent_scale, tangent[0])) <= control_bound;

    const ALR20ProjectorState clamped = al_r20_projector_state(
        {{static_cast<Binary128>(-2.0), static_cast<Binary128>(0.25),
            static_cast<Binary128>(3.0)}}, lower, upper,
        static_cast<Binary128>(10.0));
    const std::vector<Binary128Vec3> clamp_image =
        al_r20_projector_jvp(clamped, {{one, one, one}});
    result.clamps = clamped.exact && !clamped.ball_active
        && clamped.mask == std::vector<int>({-1, 0, 1})
        && al_r20_q_absmax(clamp_image[0]
            - Binary128Vec3{zero, one, zero}) == zero;

    const ALR20PivotedRank duplicate = al_r20_projector_pivoted_rank(
        {one, one, one, one}, 2U);
    result.duplicate_rank = duplicate.passed
        && duplicate.rank == 1U
        && duplicate.dependency_residual <= duplicate.reconstruction_bound
            * static_cast<Binary128>(2.0);
    const ALR20PivotedRank negative = al_r20_projector_pivoted_rank(
        {one, static_cast<Binary128>(2.0),
         static_cast<Binary128>(2.0), one}, 2U);
    result.negative_pivot = !negative.passed && negative.negative_pivot;
    std::ostringstream material;
    material << result.inactive_ball << ':' << result.active_ball_tangent
             << ':' << result.clamps << ':' << result.duplicate_rank << ':'
             << result.negative_pivot << ':' << inactive.root << ':'
             << active.root << ':' << clamped.root << ':'
             << duplicate.root << ':' << negative.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_projector_derivative_controls() {
#if defined(__linux__) && defined(__x86_64__) && defined(__GNUC__) \
    && defined(__SIZEOF_FLOAT128__)
    constexpr bool frozen_platform = true;
#else
    constexpr bool frozen_platform = false;
#endif
    const bool profile_exact = frozen_platform
        && sizeof(Binary128) == 16U && FLT_RADIX == 2
        && FLT128_MANT_DIG == 113;
    const ALR20ProjectorControls controls = al_r20_projector_controls();
    const bool controls_exact = controls.inactive_ball
        && controls.active_ball_tangent && controls.clamps
        && controls.duplicate_rank && controls.negative_pivot;
    std::vector<ALR20ManifestCase> cases = al_r20_manifest_cases();
    const bool parent_exact = profile_exact && controls_exact
        && al_r20_manifest_semantic_root(cases) == AL_R20_MANIFEST_SHA256
        && cases.size() == AL_R20_PROBLEM_SHA256.size();
    ALSparseWorkTrace sparse_work;
    StaticSupportWorkTrace static_work;
    FlatAdjacencyWorkTrace adjacency_work;
    std::vector<ALR20ProblemData> problems;
    std::vector<ALR20ProjectorCase> diagnostics;
    bool exact = parent_exact;
    bool derivative_exact = parent_exact;
    bool face_exact = parent_exact;
    bool representative_selection = false;
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        ALR20ProblemData problem = al_r20_materialize_problem(
            cases[index], &sparse_work, &static_work, &adjacency_work);
        const bool problem_exact = problem.exact
            && index < AL_R20_PROBLEM_SHA256.size()
            && problem.problem_root == AL_R20_PROBLEM_SHA256[index]
            && problem.source_positive == 0U;
        ALR20ProjectorCase diagnostic = problem_exact
            ? al_r20_projector_case(problem) : ALR20ProjectorCase{};
        exact = exact && problem_exact && diagnostic.exact;
        derivative_exact = derivative_exact
            && problem_exact && diagnostic.derivative_exact;
        face_exact = face_exact && problem_exact && diagnostic.face_exact;
        if (diagnostic.positive_rows > 0U
            && diagnostic.face_rank.rank < diagnostic.positive_rows)
            representative_selection = true;
        diagnostics.push_back(std::move(diagnostic));
        problems.push_back(std::move(problem));
    }
    for (ALR20ProblemData& problem : problems)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == cases.size()
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    exact = exact && lifecycle_exact;
    const std::string route = !parent_exact
        ? "PROJECTOR_DERIVATIVE_PARENT_REJECTED"
        : (!derivative_exact ? "PROJECTOR_DERIVATIVE_REJECTED"
            : (!face_exact || !lifecycle_exact ? "FACE_HESSIAN_REJECTED"
                : (representative_selection
                    ? "FACE_HESSIAN_REPRESENTATIVE_SELECTION_REQUIRED"
                    : "FACE_HESSIAN_NONSINGULAR_CANDIDATE")));
    std::ostringstream semantic;
    semantic << "nextengine.nonlocal.nsr3b4e2d7r20r4-projector-derivative.v1|"
             << AL_R20_MPADMM_SHA256 << ':' << route << ':'
             << controls.root << '|';
    for (const ALR20ProjectorCase& value : diagnostics)
        semantic << value.root << ';';
    const std::string result_sha = sha256_hex(semantic.str());
    std::ostringstream report;
    report << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r4_projector_derivative.v1\""
           << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
           << ",\"route\":\"" << route << "\""
           << ",\"parent_mpadmm_sha256\":\"" << AL_R20_MPADMM_SHA256
           << "\",\"profile_exact\":" << (profile_exact ? "true" : "false")
           << ",\"controls_exact\":" << (controls_exact ? "true" : "false")
           << ",\"controls_root\":\"" << controls.root << "\""
           << ",\"lifecycle_exact\":" << (lifecycle_exact ? "true" : "false")
           << ",\"representative_selection\":"
           << (representative_selection ? "true" : "false")
           << ",\"cases\":[";
    for (std::size_t index = 0U; index < cases.size(); ++index) {
        if (index != 0U) report << ',';
        const ALR20ProjectorCase& value = diagnostics[index];
        report << "{\"name\":\"" << cases[index].fixture.name
               << "\",\"role\":\"" << cases[index].role
               << "\",\"exact\":" << (value.exact ? "true" : "false")
               << ",\"ball_active\":"
               << (value.projector.ball_active ? "true" : "false")
               << ",\"eta\":\"" << al_binary128_text(value.projector.eta)
               << "\",\"free_components\":" << value.projector.free_components
               << ",\"lower_components\":" << value.projector.lower_components
               << ",\"upper_components\":" << value.projector.upper_components
               << ",\"fixed_components\":" << value.projector.fixed_components
               << ",\"projector_kkt\":\""
               << al_binary128_text(value.projector.kkt_residual)
               << "\",\"projector_kkt_bound\":\""
               << al_binary128_text(value.projector.kkt_bound)
               << "\",\"minimum_free_margin\":\""
               << al_binary128_text(value.projector.minimum_free_margin)
               << "\",\"minimum_clamped_margin\":\""
               << al_binary128_text(value.projector.minimum_clamped_margin)
               << "\",\"probe_count\":" << value.probes.size()
               << ",\"probe_error_20\":\""
               << al_binary128_text(value.maximum_probe_error_20)
               << "\",\"probe_error_24\":\""
               << al_binary128_text(value.maximum_probe_error_24)
               << "\",\"positive_rows\":" << value.positive_rows
               << ",\"near_zero_rows\":" << value.near_zero_rows
               << ",\"face_symmetry\":\""
               << al_binary128_text(value.face_symmetry_residual)
               << "\",\"face_symmetry_bound\":\""
               << al_binary128_text(value.face_symmetry_bound)
               << "\",\"face_rank\":" << value.face_rank.rank
               << ",\"face_nullity\":"
               << (value.face_rank.dimension - value.face_rank.rank)
               << ",\"minimum_positive_pivot\":\""
               << al_binary128_text(value.face_rank.minimum_positive_pivot)
               << "\",\"rejected_pivot\":\""
               << al_binary128_text(value.face_rank.rejected_pivot)
               << "\",\"pivot_bound\":\""
               << al_binary128_text(value.face_rank.pivot_bound)
               << "\",\"reconstruction_residual\":\""
               << al_binary128_text(value.face_rank.reconstruction_residual)
               << "\",\"dependency_residual\":\""
               << al_binary128_text(value.face_rank.dependency_residual)
               << "\",\"dependency_root\":\""
               << value.face_rank.dependency_root
               << "\",\"active_rows_root\":\"" << value.active_rows_root
               << "\",\"face_matrix_root\":\"" << value.face_matrix_root
               << "\",\"root\":\"" << value.root << "\"}";
    }
    report << "]"
              ",\"projector_evaluations_per_case\":25"
              ",\"derivative_probes_per_case\":6"
              ",\"newton_iterations\":0,\"multiplier_updates\":0"
              ",\"line_search_trials\":0,\"old_oracle_reruns\":0"
              ",\"generalization_authority\":false"
              ",\"timing_admitted\":false,\"runtime_authority\":false"
              ",\"production_authority\":false,\"result_sha256\":\""
           << result_sha << "\"}";
    return {exact, report.str()};
}
