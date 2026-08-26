namespace {

constexpr const char* AL_R20_R63D_PARENT_SEMANTIC_SHA256 =
    "39ca735ea4c51a0ecf21cf13a1c5a3653a94361724da9ecc57e8a7d4fd854aa1";
constexpr std::uint64_t AL_R20_R63D_PRIME_61 =
    UINT64_C(2305843009213693951);
constexpr std::uint64_t AL_R20_R63D_PRIME_31 = UINT64_C(2147483647);
__extension__ typedef unsigned __int128 ALR20R63DUInt128;

std::uint64_t al_r20_r63d_mod_multiply(
    std::uint64_t lhs, std::uint64_t rhs, std::uint64_t prime) {
    return static_cast<std::uint64_t>(
        static_cast<ALR20R63DUInt128>(lhs) * rhs % prime);
}

std::uint64_t al_r20_r63d_mod_power(
    std::uint64_t base, std::uint64_t exponent, std::uint64_t prime) {
    std::uint64_t result = 1U;
    while (exponent != 0U) {
        if ((exponent & 1U) != 0U)
            result = al_r20_r63d_mod_multiply(result, base, prime);
        base = al_r20_r63d_mod_multiply(base, base, prime);
        exponent >>= 1U;
    }
    return result;
}

std::uint64_t al_r20_r63d_dyadic_mod(
    Binary128 value, std::uint64_t prime, bool* exact) {
    if (finiteq(value) == 0 || prime < 3U || (prime & 1U) == 0U) {
        *exact = false;
        return 0U;
    }
    if (value == static_cast<Binary128>(0.0)) return 0U;
    const bool negative = value < static_cast<Binary128>(0.0);
    int exponent = 0;
    const Binary128 fraction = frexpq(fabsq(value), &exponent);
    const Binary128 scaled = scalbnq(fraction, 113);
    const ALR20R63DUInt128 significand =
        static_cast<ALR20R63DUInt128>(scaled);
    if (static_cast<Binary128>(significand) != scaled) {
        *exact = false;
        return 0U;
    }
    std::uint64_t residue = static_cast<std::uint64_t>(significand % prime);
    const int shift = exponent - 113;
    const std::uint64_t power = shift >= 0
        ? al_r20_r63d_mod_power(2U, static_cast<std::uint64_t>(shift), prime)
        : al_r20_r63d_mod_power((prime + 1U) / 2U,
            static_cast<std::uint64_t>(-static_cast<std::int64_t>(shift)),
            prime);
    residue = al_r20_r63d_mod_multiply(residue, power, prime);
    return negative && residue != 0U ? prime - residue : residue;
}

struct ALR20R63DModularRank {
    bool exact = false;
    std::uint64_t prime = 0U;
    std::size_t rows = 0U;
    std::size_t columns = 0U;
    std::size_t rank = 0U;
    std::size_t mapped_values = 0U;
    std::size_t row_swaps = 0U;
    std::size_t inversions = 0U;
    std::size_t normalized_values = 0U;
    std::size_t eliminated_rows = 0U;
    std::size_t elimination_updates = 0U;
    std::vector<std::size_t> pivot_columns;
    std::string input_root;
    std::string root;
};

ALR20R63DModularRank al_r20_r63d_modular_rank(
    const std::vector<Binary128>& input, std::size_t rows,
    std::size_t columns, std::uint64_t prime) {
    ALR20R63DModularRank result;
    result.prime = prime;
    result.rows = rows;
    result.columns = columns;
    result.input_root = al_r20_q_scalar_root(input);
    if (rows == 0U || columns == 0U || input.size() != rows * columns)
        return result;
    bool mapping_exact = true;
    std::vector<std::uint64_t> matrix(input.size());
    for (std::size_t index = 0U; index < input.size(); ++index) {
        matrix[index] = al_r20_r63d_dyadic_mod(
            input[index], prime, &mapping_exact);
        ++result.mapped_values;
    }
    std::size_t rank = 0U;
    for (std::size_t column = 0U;
         column < columns && rank < rows; ++column) {
        std::size_t pivot = rank;
        while (pivot < rows && matrix[pivot * columns + column] == 0U)
            ++pivot;
        if (pivot == rows) continue;
        if (pivot != rank) {
            for (std::size_t item = 0U; item < columns; ++item)
                std::swap(matrix[rank * columns + item],
                    matrix[pivot * columns + item]);
            ++result.row_swaps;
        }
        const std::uint64_t inverse = al_r20_r63d_mod_power(
            matrix[rank * columns + column], prime - 2U, prime);
        ++result.inversions;
        for (std::size_t item = column; item < columns; ++item) {
            matrix[rank * columns + item] = al_r20_r63d_mod_multiply(
                matrix[rank * columns + item], inverse, prime);
            ++result.normalized_values;
        }
        for (std::size_t row = rank + 1U; row < rows; ++row) {
            const std::uint64_t factor = matrix[row * columns + column];
            if (factor == 0U) continue;
            ++result.eliminated_rows;
            for (std::size_t item = column; item < columns; ++item) {
                const std::uint64_t product = al_r20_r63d_mod_multiply(
                    factor, matrix[rank * columns + item], prime);
                std::uint64_t& target = matrix[row * columns + item];
                target = target >= product
                    ? target - product : prime - (product - target);
                ++result.elimination_updates;
            }
        }
        result.pivot_columns.push_back(column);
        ++rank;
    }
    result.rank = rank;
    result.exact = mapping_exact && result.mapped_values == input.size()
        && result.inversions == rank
        && result.pivot_columns.size() == rank;
    std::ostringstream material;
    material << result.exact << ':' << prime << ':' << rows << ':'
        << columns << ':' << rank << ':' << result.mapped_values << ':'
        << result.row_swaps << ':' << result.inversions << ':'
        << result.normalized_values << ':' << result.eliminated_rows << ':'
        << result.elimination_updates << ':' << result.input_root << '|';
    for (std::size_t column : result.pivot_columns)
        material << column << ',';
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63DSourceMatrix {
    bool exact = false;
    std::size_t rows = 0U;
    std::size_t columns = 0U;
    std::size_t sparse_entries = 0U;
    std::size_t scalar_updates = 0U;
    std::size_t duplicate_writes = 0U;
    std::size_t nonzero_values = 0U;
    std::size_t duplicate_rows = 0U;
    std::vector<Binary128> values;
    std::string row_root;
    std::string value_root;
    std::string root;
};

ALR20R63DSourceMatrix al_r20_r63d_source_matrix(
    const ALR64Operator& op, const std::vector<std::size_t>& source_rows,
    std::size_t particle_count) {
    ALR20R63DSourceMatrix result;
    result.rows = source_rows.size();
    result.columns = 3U * particle_count;
    if (result.rows == 0U || particle_count == 0U
        || op.entry_offsets.empty()) return result;
    std::ostringstream row_material;
    std::vector<bool> selected(op.entry_offsets.size() - 1U, false);
    bool exact = true;
    result.values.reserve(result.rows * result.columns);
    for (std::size_t row : source_rows) {
        row_material << row << ',';
        if (row + 1U >= op.entry_offsets.size() || selected[row]) {
            exact = false;
            break;
        }
        selected[row] = true;
        std::vector<bool> written(result.columns, false);
        for (std::size_t entry = op.entry_offsets[row];
             entry < op.entry_offsets[row + 1U]; ++entry) {
            if (op.entries[entry].particle >= particle_count) {
                exact = false;
                break;
            }
            ++result.sparse_entries;
            for (std::size_t axis = 0U; axis < 3U; ++axis) {
                const std::size_t coordinate =
                    3U * op.entries[entry].particle + axis;
                if (written[coordinate]) ++result.duplicate_writes;
                written[coordinate] = true;
                ++result.scalar_updates;
            }
        }
        if (!exact) break;
        const std::vector<Binary128Vec3> vector =
            al_r20_projector_row_vector(op, row, particle_count);
        for (const Binary128Vec3& value : vector) {
            const std::array<Binary128, 3U> components{
                value.x, value.y, value.z};
            for (Binary128 component : components) {
                exact = exact && finiteq(component) != 0;
                if (component != static_cast<Binary128>(0.0))
                    ++result.nonzero_values;
                result.values.push_back(component);
            }
        }
    }
    if (exact && result.values.size() == result.rows * result.columns) {
        for (std::size_t lhs = 0U; lhs < result.rows; ++lhs)
            for (std::size_t rhs = lhs + 1U; rhs < result.rows; ++rhs) {
                bool equal = true;
                for (std::size_t column = 0U;
                     column < result.columns; ++column)
                    equal = equal && result.values[lhs * result.columns + column]
                        == result.values[rhs * result.columns + column];
                if (equal) ++result.duplicate_rows;
            }
    }
    result.row_root = sha256_hex(row_material.str());
    result.value_root = al_r20_q_scalar_root(result.values);
    result.exact = exact
        && result.values.size() == result.rows * result.columns;
    std::ostringstream material;
    material << result.exact << ':' << result.rows << ':' << result.columns
        << ':' << result.sparse_entries << ':' << result.scalar_updates << ':'
        << result.duplicate_writes << ':' << result.nonzero_values << ':'
        << result.duplicate_rows << ':' << result.row_root << ':'
        << result.value_root;
    result.root = sha256_hex(material.str());
    return result;
}

std::vector<Binary128> al_r20_r63d_gram(
    const std::vector<Binary128>& rows, std::size_t row_count,
    std::size_t columns) {
    std::vector<Binary128> result(
        row_count * row_count, static_cast<Binary128>(0.0));
    if (rows.size() != row_count * columns) return {};
    for (std::size_t row = 0U; row < row_count; ++row)
        for (std::size_t column = row; column < row_count; ++column) {
            Binary128Accumulator dot;
            for (std::size_t inner = 0U; inner < columns; ++inner)
                dot.add(rows[row * columns + inner]
                    * rows[column * columns + inner]);
            result[row * row_count + column] = dot.value();
            result[column * row_count + row] = dot.value();
        }
    return result;
}

struct ALR20R63DNormalized {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t square_roots = 0U;
    std::size_t divisions = 0U;
    std::vector<Binary128> values;
    std::string input_root;
    std::string root;
};

ALR20R63DNormalized al_r20_r63d_normalize(
    const std::vector<Binary128>& matrix, std::size_t dimension) {
    ALR20R63DNormalized result;
    result.dimension = dimension;
    result.input_root = al_r20_q_scalar_root(matrix);
    if (dimension == 0U || matrix.size() != dimension * dimension)
        return result;
    std::vector<Binary128> scale(dimension);
    bool exact = true;
    for (std::size_t row = 0U; row < dimension; ++row) {
        const Binary128 diagonal = matrix[row * dimension + row];
        exact = exact && finiteq(diagonal) != 0
            && diagonal > static_cast<Binary128>(0.0);
        if (!exact) break;
        scale[row] = sqrtq(diagonal);
        exact = finiteq(scale[row]) != 0
            && scale[row] > static_cast<Binary128>(0.0);
        ++result.square_roots;
    }
    result.values.assign(matrix.size(), static_cast<Binary128>(0.0));
    for (std::size_t row = 0U; row < dimension && exact; ++row)
        for (std::size_t column = row; column < dimension; ++column) {
            if (matrix[row * dimension + column]
                != matrix[column * dimension + row]) {
                exact = false;
                break;
            }
            const Binary128 value = matrix[row * dimension + column]
                / scale[row] / scale[column];
            exact = finiteq(value) != 0;
            if (!exact) break;
            result.values[row * dimension + column] = value;
            result.values[column * dimension + row] = value;
            ++result.divisions;
        }
    result.exact = exact && result.square_roots == dimension
        && result.divisions == dimension * (dimension + 1U) / 2U;
    std::ostringstream material;
    material << result.exact << ':' << dimension << ':'
        << result.square_roots << ':' << result.divisions << ':'
        << result.input_root << ':' << al_r20_q_scalar_root(result.values);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63DBinary64Rank {
    bool exact = false;
    bool negative_pivot = false;
    std::size_t dimension = 0U;
    std::size_t rank = 0U;
    std::size_t row_column_swaps = 0U;
    std::size_t square_roots = 0U;
    std::size_t divisions = 0U;
    std::size_t fmas = 0U;
    double scale = 0.0;
    double threshold = 0.0;
    double minimum_positive_pivot = 0.0;
    double rejected_pivot = 0.0;
    std::vector<std::size_t> permutation;
    std::string projected_root;
    std::string root;
};

ALR20R63DBinary64Rank al_r20_r63d_binary64_rank(
    const std::vector<Binary128>& input, std::size_t dimension) {
    ALR20R63DBinary64Rank result;
    result.dimension = dimension;
    result.permutation.resize(dimension);
    std::iota(result.permutation.begin(), result.permutation.end(), 0U);
    if (dimension == 0U || input.size() != dimension * dimension)
        return result;
    std::vector<double> schur(input.size());
    bool exact = true;
    for (std::size_t index = 0U; index < input.size(); ++index) {
        schur[index] = static_cast<double>(input[index]);
        exact = exact && std::isfinite(schur[index]);
    }
    result.projected_root = al_r20_double_root(schur);
    for (std::size_t row = 0U; row < dimension; ++row)
        result.scale = std::max(result.scale,
            schur[row * dimension + row]);
    result.threshold = static_cast<double>(dimension)
        * std::ldexp(1.0, -53) * result.scale;
    std::vector<double> lower(input.size(), 0.0);
    result.minimum_positive_pivot =
        std::numeric_limits<double>::infinity();
    for (std::size_t column = 0U; column < dimension && exact; ++column) {
        std::size_t pivot_index = column;
        double pivot = schur[column * dimension + column];
        for (std::size_t candidate = column + 1U;
             candidate < dimension; ++candidate)
            if (schur[candidate * dimension + candidate] > pivot) {
                pivot = schur[candidate * dimension + candidate];
                pivot_index = candidate;
            }
        if (pivot <= result.threshold) {
            result.rejected_pivot = pivot;
            result.negative_pivot = pivot < -result.threshold;
            break;
        }
        if (pivot_index != column) {
            for (std::size_t item = 0U; item < dimension; ++item)
                std::swap(schur[column * dimension + item],
                    schur[pivot_index * dimension + item]);
            for (std::size_t item = 0U; item < dimension; ++item)
                std::swap(schur[item * dimension + column],
                    schur[item * dimension + pivot_index]);
            for (std::size_t item = 0U; item < column; ++item)
                std::swap(lower[column * dimension + item],
                    lower[pivot_index * dimension + item]);
            std::swap(result.permutation[column],
                result.permutation[pivot_index]);
            ++result.row_column_swaps;
            pivot = schur[column * dimension + column];
        }
        const double diagonal = std::sqrt(pivot);
        exact = std::isfinite(diagonal) && diagonal > 0.0;
        if (!exact) break;
        lower[column * dimension + column] = diagonal;
        ++result.square_roots;
        result.minimum_positive_pivot = std::min(
            result.minimum_positive_pivot, pivot);
        for (std::size_t row = column + 1U; row < dimension; ++row) {
            lower[row * dimension + column] =
                schur[row * dimension + column] / diagonal;
            exact = exact && std::isfinite(
                lower[row * dimension + column]);
            ++result.divisions;
        }
        for (std::size_t row = column + 1U; row < dimension; ++row)
            for (std::size_t other = row; other < dimension; ++other) {
                const double value = std::fma(
                    -lower[other * dimension + column],
                    lower[row * dimension + column],
                    schur[other * dimension + row]);
                exact = exact && std::isfinite(value);
                schur[other * dimension + row] = value;
                schur[row * dimension + other] = value;
                ++result.fmas;
            }
        ++result.rank;
    }
    if (result.rank == dimension) result.rejected_pivot = 0.0;
    if (result.rank == 0U)
        result.minimum_positive_pivot = 0.0;
    result.exact = exact && std::isfinite(result.scale)
        && std::isfinite(result.threshold) && !result.negative_pivot;
    std::ostringstream material;
    material << result.exact << ':' << result.negative_pivot << ':'
        << dimension << ':' << result.rank << ':'
        << result.row_column_swaps << ':' << result.square_roots << ':'
        << result.divisions << ':' << result.fmas << ':'
        << binary64_bits(result.scale) << ':'
        << binary64_bits(result.threshold) << ':'
        << binary64_bits(result.minimum_positive_pivot) << ':'
        << binary64_bits(result.rejected_pivot) << ':'
        << result.projected_root << '|';
    for (std::size_t item : result.permutation) material << item << ',';
    result.root = sha256_hex(material.str());
    return result;
}

std::string al_r20_r63d_classify(
    bool apparatus, std::size_t duplicate_rows,
    bool source_modular_full, bool projected_modular_full,
    std::size_t source_rank64, std::size_t projected_rank64,
    std::size_t dimension) {
    if (!apparatus) return "ACTIVE_ROW_RANK_APPARATUS_REJECTED";
    if (duplicate_rows != 0U) return "SOURCE_EXACT_DUPLICATE";
    if (!source_modular_full) return "SOURCE_MODULAR_RANK_INCONCLUSIVE";
    if (!projected_modular_full)
        return "PROJECTED_MODULAR_RANK_INCONCLUSIVE";
    if (source_rank64 < dimension)
        return "SOURCE_ROW_NUMERICAL_NEAR_DEPENDENCE";
    if (projected_rank64 < dimension)
        return "PROJECTOR_METRIC_NUMERICAL_RANK_LOSS";
    return "FULL_RANK_INVERSE_CONDITIONING_BOUNDARY";
}

struct ALR20R63DControls {
    bool exact = false;
    bool mapping = false;
    bool modular = false;
    bool pivot = false;
    bool classification = false;
    bool mutation = false;
    std::string root;
};

ALR20R63DControls al_r20_r63d_controls() {
    ALR20R63DControls result;
    result.mapping = true;
    for (std::uint64_t prime :
         {AL_R20_R63D_PRIME_61, AL_R20_R63D_PRIME_31}) {
        bool exact = true;
        const std::uint64_t zero = al_r20_r63d_dyadic_mod(
            static_cast<Binary128>(0.0), prime, &exact);
        const std::uint64_t half = al_r20_r63d_dyadic_mod(
            static_cast<Binary128>(0.5), prime, &exact);
        const std::uint64_t quarter = al_r20_r63d_dyadic_mod(
            static_cast<Binary128>(0.25), prime, &exact);
        const std::uint64_t negative = al_r20_r63d_dyadic_mod(
            static_cast<Binary128>(-0.5), prime, &exact);
        const std::uint64_t minimum_normal = al_r20_r63d_dyadic_mod(
            ldexpq(static_cast<Binary128>(1.0), -16382), prime, &exact);
        const std::uint64_t minimum_subnormal = al_r20_r63d_dyadic_mod(
            ldexpq(static_cast<Binary128>(1.0), -16494), prime, &exact);
        result.mapping = result.mapping && exact && zero == 0U
            && al_r20_r63d_mod_multiply(half, 2U, prime) == 1U
            && al_r20_r63d_mod_multiply(quarter, 4U, prime) == 1U
            && negative == prime - half
            && al_r20_r63d_mod_multiply(minimum_normal,
                al_r20_r63d_mod_power(2U, 16382U, prime), prime) == 1U
            && al_r20_r63d_mod_multiply(minimum_subnormal,
                al_r20_r63d_mod_power(2U, 16494U, prime), prime) == 1U;
    }

    const std::vector<Binary128> full{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.0),
        static_cast<Binary128>(0.5), static_cast<Binary128>(0.0),
        static_cast<Binary128>(1.0), static_cast<Binary128>(-0.25)};
    const std::vector<Binary128> duplicate{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.5),
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.5)};
    const ALR20R63DModularRank full61 = al_r20_r63d_modular_rank(
        full, 2U, 3U, AL_R20_R63D_PRIME_61);
    const ALR20R63DModularRank full31 = al_r20_r63d_modular_rank(
        full, 2U, 3U, AL_R20_R63D_PRIME_31);
    const ALR20R63DModularRank duplicate61 = al_r20_r63d_modular_rank(
        duplicate, 2U, 2U, AL_R20_R63D_PRIME_61);
    result.modular = full61.exact && full31.exact && duplicate61.exact
        && full61.rank == 2U && full31.rank == 2U
        && duplicate61.rank == 1U;

    const std::vector<Binary128> positive{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.25),
        static_cast<Binary128>(0.25), static_cast<Binary128>(1.0)};
    const std::vector<Binary128> rank_one{
        static_cast<Binary128>(1.0), static_cast<Binary128>(1.0),
        static_cast<Binary128>(1.0), static_cast<Binary128>(1.0)};
    const ALR20R63DBinary64Rank positive64 =
        al_r20_r63d_binary64_rank(positive, 2U);
    const ALR20R63DBinary64Rank rank_one64 =
        al_r20_r63d_binary64_rank(rank_one, 2U);
    const ALR20PivotedRank positive128 =
        al_r20_projector_pivoted_rank(positive, 2U);
    const ALR20PivotedRank rank_one128 =
        al_r20_projector_pivoted_rank(rank_one, 2U);
    result.pivot = positive64.exact && rank_one64.exact
        && positive64.rank == 2U && rank_one64.rank == 1U
        && positive128.passed && rank_one128.passed
        && positive128.rank == 2U && rank_one128.rank == 1U;

    result.classification =
        al_r20_r63d_classify(true, 0U, true, true, 1U, 1U, 2U)
            == "SOURCE_ROW_NUMERICAL_NEAR_DEPENDENCE"
        && al_r20_r63d_classify(true, 0U, true, true, 2U, 1U, 2U)
            == "PROJECTOR_METRIC_NUMERICAL_RANK_LOSS"
        && al_r20_r63d_classify(true, 0U, true, true, 2U, 2U, 2U)
            == "FULL_RANK_INVERSE_CONDITIONING_BOUNDARY"
        && al_r20_r63d_classify(true, 0U, false, false, 0U, 0U, 2U)
            == "SOURCE_MODULAR_RANK_INCONCLUSIVE";

    const std::vector<Binary128> bound{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.5)};
    std::vector<Binary128> mutated = bound;
    mutated[1U] = nextafterq(mutated[1U], HUGE_VALQ);
    const std::vector<std::size_t> rows{1U, 2U};
    const std::vector<std::size_t> mutated_rows{1U, 3U};
    std::ostringstream row_root;
    std::ostringstream mutated_row_root;
    for (std::size_t row : rows) row_root << row << ',';
    for (std::size_t row : mutated_rows) mutated_row_root << row << ',';
    result.mutation = al_r20_q_scalar_root(bound)
            != al_r20_q_scalar_root(mutated)
        && sha256_hex(row_root.str()) != sha256_hex(mutated_row_root.str());

    result.exact = result.mapping && result.modular && result.pivot
        && result.classification && result.mutation;
    std::ostringstream material;
    material << result.exact << ':' << result.mapping << ':'
        << result.modular << ':' << result.pivot << ':'
        << result.classification << ':' << result.mutation << ':'
        << full61.root << ':' << full31.root << ':' << duplicate61.root
        << ':' << positive64.root << ':' << rank_one64.root << ':'
        << positive128.root << ':' << rank_one128.root;
    result.root = sha256_hex(material.str());
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_active_row_rank_controls() {
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
    const ALR20R63DControls controls = al_r20_r63d_controls();
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
        && al_r20_slope_error_refiner == nullptr
        && al_r20_r55_slope_trace == nullptr
        && al_r20_direct_solution_capture == nullptr;
    const bool capture_exact = parent_exact && hook_slot_clear
        && hook_slot_released && solver_executions == 1U
        && al_r20_r35_case_observation_exact(oblique)
        && oblique.root == AL_R20_R60_CASE_SHA256
        && solution_capture.calls == 1U
        && solution_capture.dimension == AL_R20_R63_DIMENSION
        && solution_capture.material_root == AL_R20_R63_MATERIAL_SHA256
        && factor_capture.selected_calls == 1U
        && factor_capture.selected_column_solves == AL_R20_R63_DIMENSION
        && factor_capture.factor_passed
        && factor_capture.matrix_root == AL_R20_R63_MATRIX_SHA256
        && factor_capture.source_rows.size() == AL_R20_R63_DIMENSION
        && factor_capture.matrix.size()
            == AL_R20_R63_DIMENSION * AL_R20_R63_DIMENSION
        && slope.calls == 0U && slope.inverse_audits == 0U;

    const ALR20R63DSourceMatrix source = capture_exact
        ? al_r20_r63d_source_matrix(problem.op, factor_capture.source_rows,
            problem.target.size())
        : ALR20R63DSourceMatrix{};
    const std::vector<Binary128> source_gram = source.exact
        ? al_r20_r63d_gram(source.values, source.rows, source.columns)
        : std::vector<Binary128>{};
    const bool gram_exact = source.exact
        && source_gram.size() == source.rows * source.rows;

    const ALR20R63DModularRank source61 = source.exact
        ? al_r20_r63d_modular_rank(source.values, source.rows,
            source.columns, AL_R20_R63D_PRIME_61)
        : ALR20R63DModularRank{};
    const ALR20R63DModularRank source31 = source.exact
        ? al_r20_r63d_modular_rank(source.values, source.rows,
            source.columns, AL_R20_R63D_PRIME_31)
        : ALR20R63DModularRank{};
    const ALR20R63DModularRank projected61 = capture_exact
        ? al_r20_r63d_modular_rank(factor_capture.matrix,
            AL_R20_R63_DIMENSION, AL_R20_R63_DIMENSION,
            AL_R20_R63D_PRIME_61)
        : ALR20R63DModularRank{};
    const ALR20R63DModularRank projected31 = capture_exact
        ? al_r20_r63d_modular_rank(factor_capture.matrix,
            AL_R20_R63_DIMENSION, AL_R20_R63_DIMENSION,
            AL_R20_R63D_PRIME_31)
        : ALR20R63DModularRank{};
    const bool modular_exact = source61.exact && source31.exact
        && projected61.exact && projected31.exact;
    const bool source_modular_full = source61.rank == source.rows
        || source31.rank == source.rows;
    const bool projected_modular_full =
        projected61.rank == AL_R20_R63_DIMENSION
        || projected31.rank == AL_R20_R63_DIMENSION;

    const ALR20R63DNormalized source_normalized = gram_exact
        ? al_r20_r63d_normalize(source_gram, source.rows)
        : ALR20R63DNormalized{};
    const ALR20R63DNormalized projected_normalized = capture_exact
        ? al_r20_r63d_normalize(
            factor_capture.matrix, AL_R20_R63_DIMENSION)
        : ALR20R63DNormalized{};
    const ALR20R63DBinary64Rank source64 = source_normalized.exact
        ? al_r20_r63d_binary64_rank(
            source_normalized.values, source.rows)
        : ALR20R63DBinary64Rank{};
    const ALR20R63DBinary64Rank projected64 = projected_normalized.exact
        ? al_r20_r63d_binary64_rank(
            projected_normalized.values, AL_R20_R63_DIMENSION)
        : ALR20R63DBinary64Rank{};
    const ALR20PivotedRank source128 = source_normalized.exact
        ? al_r20_projector_pivoted_rank(
            source_normalized.values, source.rows)
        : ALR20PivotedRank{};
    const ALR20PivotedRank projected128 = projected_normalized.exact
        ? al_r20_projector_pivoted_rank(
            projected_normalized.values, AL_R20_R63_DIMENSION)
        : ALR20PivotedRank{};
    const bool pivot_exact = source_normalized.exact
        && projected_normalized.exact && source64.exact
        && projected64.exact && source128.passed && projected128.passed;

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const bool work_exact = source.rows == 102U && source.columns == 315U
        && source61.mapped_values == 32130U
        && source31.mapped_values == 32130U
        && projected61.mapped_values == 10404U
        && projected31.mapped_values == 10404U
        && source_normalized.square_roots == 102U
        && projected_normalized.square_roots == 102U
        && source_normalized.divisions == 5253U
        && projected_normalized.divisions == 5253U;
    const bool apparatus_exact = profile_exact && controls_exact
        && parent_exact && capture_exact && source.exact && gram_exact
        && modular_exact && pivot_exact && lifecycle_exact && work_exact;
    const std::string route = al_r20_r63d_classify(apparatus_exact,
        source.duplicate_rows, source_modular_full,
        projected_modular_full, source64.rank, projected64.rank,
        AL_R20_R63_DIMENSION);
    const bool exact = apparatus_exact;

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63d-active-row-rank.v1|"
        << AL_R20_R63D_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << oblique.root << ':' << factor_capture.matrix_root << ':'
        << controls.root << ':' << source.root << ':'
        << al_r20_q_scalar_root(source_gram) << ':'
        << source61.root << ':' << source31.root << ':'
        << projected61.root << ':' << projected31.root << ':'
        << source_normalized.root << ':' << projected_normalized.root << ':'
        << source64.root << ':' << projected64.root << ':'
        << source128.root << ':' << projected128.root;
    const std::string result_sha = sha256_hex(semantic.str());

    auto modular_json = [](std::ostringstream& report, const char* name,
                           const ALR20R63DModularRank& rank) {
        report << ",\"" << name << "\":{\"exact\":"
            << (rank.exact ? "true" : "false")
            << ",\"prime\":" << rank.prime
            << ",\"rows\":" << rank.rows
            << ",\"columns\":" << rank.columns
            << ",\"rank\":" << rank.rank
            << ",\"mapped_values\":" << rank.mapped_values
            << ",\"row_swaps\":" << rank.row_swaps
            << ",\"inversions\":" << rank.inversions
            << ",\"eliminated_rows\":" << rank.eliminated_rows
            << ",\"updates\":" << rank.elimination_updates
            << ",\"root\":\"" << rank.root << "\"}";
    };
    auto rank64_json = [](std::ostringstream& report, const char* name,
                          const ALR20R63DBinary64Rank& rank) {
        report << ",\"" << name << "\":{\"exact\":"
            << (rank.exact ? "true" : "false")
            << ",\"rank\":" << rank.rank
            << ",\"threshold\":" << rank.threshold
            << ",\"minimum_positive_pivot\":"
            << rank.minimum_positive_pivot
            << ",\"rejected_pivot\":" << rank.rejected_pivot
            << ",\"swaps\":" << rank.row_column_swaps
            << ",\"square_roots\":" << rank.square_roots
            << ",\"divisions\":" << rank.divisions
            << ",\"fmas\":" << rank.fmas
            << ",\"root\":\"" << rank.root << "\"}";
    };
    auto rank128_json = [](std::ostringstream& report, const char* name,
                           const ALR20PivotedRank& rank) {
        report << ",\"" << name << "\":{\"passed\":"
            << (rank.passed ? "true" : "false")
            << ",\"rank\":" << rank.rank
            << ",\"threshold\":\""
            << al_binary128_text(rank.pivot_bound)
            << "\",\"minimum_positive_pivot\":\""
            << al_binary128_text(rank.minimum_positive_pivot)
            << "\",\"rejected_pivot\":\""
            << al_binary128_text(rank.rejected_pivot)
            << "\",\"root\":\"" << rank.root << "\"}";
    };

    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63d_active_row_rank.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63c_sha256\":\""
        << AL_R20_R63D_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls\":{\"exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"mapping\":" << (controls.mapping ? "true" : "false")
        << ",\"modular\":" << (controls.modular ? "true" : "false")
        << ",\"pivot\":" << (controls.pivot ? "true" : "false")
        << ",\"classification\":"
        << (controls.classification ? "true" : "false")
        << ",\"mutation\":" << (controls.mutation ? "true" : "false")
        << ",\"root\":\"" << controls.root << "\"}"
        << ",\"capture\":{\"exact\":"
        << (capture_exact ? "true" : "false")
        << ",\"observed_inverse_calls\":" << factor_capture.calls
        << ",\"selected_inverse_calls\":" << factor_capture.selected_calls
        << ",\"selected_column_solves\":"
        << factor_capture.selected_column_solves
        << ",\"source_rows\":" << factor_capture.source_rows.size()
        << ",\"matrix_root\":\"" << factor_capture.matrix_root << "\"}"
        << ",\"source\":{\"exact\":"
        << (source.exact ? "true" : "false")
        << ",\"rows\":" << source.rows
        << ",\"columns\":" << source.columns
        << ",\"sparse_entries\":" << source.sparse_entries
        << ",\"scalar_updates\":" << source.scalar_updates
        << ",\"duplicate_writes\":" << source.duplicate_writes
        << ",\"nonzero_values\":" << source.nonzero_values
        << ",\"duplicate_rows\":" << source.duplicate_rows
        << ",\"row_root\":\"" << source.row_root
        << "\",\"value_root\":\"" << source.value_root << "\"}"
        << ",\"source_modular_full\":"
        << (source_modular_full ? "true" : "false")
        << ",\"projected_modular_full\":"
        << (projected_modular_full ? "true" : "false");
    modular_json(report, "source_prime61", source61);
    modular_json(report, "source_prime31", source31);
    modular_json(report, "projected_prime61", projected61);
    modular_json(report, "projected_prime31", projected31);
    rank64_json(report, "source_binary64", source64);
    rank64_json(report, "projected_binary64", projected64);
    rank128_json(report, "source_binary128", source128);
    rank128_json(report, "projected_binary128", projected128);
    report << ",\"normalization\":{\"source_exact\":"
        << (source_normalized.exact ? "true" : "false")
        << ",\"projected_exact\":"
        << (projected_normalized.exact ? "true" : "false")
        << ",\"source_root\":\"" << source_normalized.root
        << "\",\"projected_root\":\"" << projected_normalized.root << "\"}"
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"nnqp_rhs_solves\":0,\"row_drops\":0"
        << ",\"center_iterations\":0,\"replacement_applies\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
