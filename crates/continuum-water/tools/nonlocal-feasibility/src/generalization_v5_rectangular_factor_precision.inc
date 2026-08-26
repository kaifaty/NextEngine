namespace {

constexpr const char* AL_R20_R63G_PARENT_SEMANTIC_SHA256 =
    "048601d394e75ae42fc84554cc08b1f3012817824ebb994e990e1b25dd7e9acf";

std::string al_r20_r63g_index_root(
    const std::vector<std::size_t>& values) {
    std::ostringstream material;
    material << values.size();
    for (std::size_t value : values) material << ':' << value;
    return sha256_hex(material.str());
}

bool al_r20_r63g_permutation_exact(
    const std::vector<std::size_t>& permutation, std::size_t size) {
    if (permutation.size() != size) return false;
    std::vector<bool> seen(size, false);
    for (std::size_t value : permutation) {
        if (value >= size || seen[value]) return false;
        seen[value] = true;
    }
    return std::all_of(seen.begin(), seen.end(), [](bool value) {
        return value;
    });
}

Binary128 al_r20_r63g_q_norm(
    const std::vector<Binary128>& matrix, std::size_t rows,
    std::size_t columns, std::size_t column, std::size_t first_row,
    std::size_t* terms) {
    Binary128 scale = static_cast<Binary128>(0.0);
    Binary128 sum = static_cast<Binary128>(1.0);
    for (std::size_t row = first_row; row < rows; ++row) {
        const Binary128 value = fabsq(matrix[row * columns + column]);
        if (value != static_cast<Binary128>(0.0)) {
            if (scale < value) {
                const Binary128 ratio = scale / value;
                sum = static_cast<Binary128>(1.0) + sum * ratio * ratio;
                scale = value;
            } else {
                const Binary128 ratio = value / scale;
                sum += ratio * ratio;
            }
        }
        ++(*terms);
    }
    return scale == static_cast<Binary128>(0.0)
        ? static_cast<Binary128>(0.0) : scale * sqrtq(sum);
}

double al_r20_r63g_d_norm(
    const std::vector<double>& matrix, std::size_t rows,
    std::size_t columns, std::size_t column, std::size_t first_row,
    std::size_t* terms) {
    double scale = 0.0;
    double sum = 1.0;
    for (std::size_t row = first_row; row < rows; ++row) {
        const double value = std::abs(matrix[row * columns + column]);
        if (value != 0.0) {
            if (scale < value) {
                const double ratio = scale / value;
                sum = 1.0 + sum * ratio * ratio;
                scale = value;
            } else {
                const double ratio = value / scale;
                sum += ratio * ratio;
            }
        }
        ++(*terms);
    }
    return scale == 0.0 ? 0.0 : scale * std::sqrt(sum);
}

struct ALR20R63GQFactor {
    bool exact = false;
    std::size_t rows = 0U;
    std::size_t columns = 0U;
    std::size_t nonzero_diagonal = 0U;
    std::size_t norm_terms = 0U;
    std::size_t reflector_products = 0U;
    std::size_t q_products = 0U;
    Binary128 minimum_abs_diagonal = HUGE_VALQ;
    Binary128 maximum_abs_diagonal = static_cast<Binary128>(0.0);
    std::vector<Binary128> input;
    std::vector<Binary128> reflectors;
    std::vector<Binary128> tau;
    std::vector<Binary128> q;
    std::vector<Binary128> r;
    std::string root;
};

ALR20R63GQFactor al_r20_r63g_q_factor(
    const std::vector<Binary128>& input, std::size_t rows,
    std::size_t columns) {
    ALR20R63GQFactor result;
    result.rows = rows;
    result.columns = columns;
    result.input = input;
    if (rows < columns || input.size() != rows * columns) return result;
    std::vector<Binary128> work = input;
    result.reflectors.assign(rows * columns,
        static_cast<Binary128>(0.0));
    result.tau.assign(columns, static_cast<Binary128>(0.0));
    result.r.assign(columns * columns, static_cast<Binary128>(0.0));
    bool finite = true;
    for (std::size_t column = 0U; column < columns; ++column) {
        const Binary128 norm = al_r20_r63g_q_norm(work, rows, columns,
            column, column, &result.norm_terms);
        const Binary128 first = work[column * columns + column];
        const Binary128 beta = first >= static_cast<Binary128>(0.0)
            ? -norm : norm;
        const Binary128 head = first - beta;
        finite = finite && finiteq(norm) != 0 && finiteq(beta) != 0
            && finiteq(head) != 0
            && norm > static_cast<Binary128>(0.0)
            && head != static_cast<Binary128>(0.0)
            && beta != static_cast<Binary128>(0.0);
        if (!finite) break;
        result.reflectors[column * columns + column] =
            static_cast<Binary128>(1.0);
        for (std::size_t row = column + 1U; row < rows; ++row) {
            result.reflectors[row * columns + column] =
                work[row * columns + column] / head;
            finite = finite && finiteq(
                result.reflectors[row * columns + column]) != 0;
        }
        result.tau[column] = (beta - first) / beta;
        finite = finite && finiteq(result.tau[column]) != 0;
        work[column * columns + column] = beta;
        for (std::size_t other = column + 1U;
             other < columns; ++other) {
            Binary128Accumulator dot;
            for (std::size_t row = column; row < rows; ++row) {
                dot.add(result.reflectors[row * columns + column]
                    * work[row * columns + other]);
                ++result.reflector_products;
            }
            const Binary128 update = result.tau[column] * dot.value();
            for (std::size_t row = column; row < rows; ++row) {
                work[row * columns + other] -=
                    result.reflectors[row * columns + column] * update;
                ++result.reflector_products;
                finite = finite
                    && finiteq(work[row * columns + other]) != 0;
            }
        }
        for (std::size_t other = column; other < columns; ++other)
            result.r[column * columns + other] =
                work[column * columns + other];
        result.minimum_abs_diagonal = fminq(result.minimum_abs_diagonal,
            fabsq(beta));
        result.maximum_abs_diagonal = fmaxq(result.maximum_abs_diagonal,
            fabsq(beta));
        ++result.nonzero_diagonal;
    }
    result.q.assign(rows * columns, static_cast<Binary128>(0.0));
    for (std::size_t index = 0U; index < columns; ++index)
        result.q[index * columns + index] = static_cast<Binary128>(1.0);
    if (finite && result.nonzero_diagonal == columns) {
        for (std::size_t reverse = columns; reverse-- > 0U;)
            for (std::size_t q_column = 0U;
                 q_column < columns; ++q_column) {
                Binary128Accumulator dot;
                for (std::size_t row = reverse; row < rows; ++row) {
                    dot.add(result.reflectors[row * columns + reverse]
                        * result.q[row * columns + q_column]);
                    ++result.q_products;
                }
                const Binary128 update = result.tau[reverse] * dot.value();
                for (std::size_t row = reverse; row < rows; ++row) {
                    result.q[row * columns + q_column] -=
                        result.reflectors[row * columns + reverse] * update;
                    ++result.q_products;
                    finite = finite
                        && finiteq(result.q[row * columns + q_column]) != 0;
                }
            }
    }
    result.exact = finite && result.nonzero_diagonal == columns;
    std::ostringstream material;
    material << result.exact << ':' << result.rows << ':' << result.columns
        << ':' << result.nonzero_diagonal << ':' << result.norm_terms << ':'
        << result.reflector_products << ':' << result.q_products << ':'
        << al_binary128_hex(result.minimum_abs_diagonal) << ':'
        << al_binary128_hex(result.maximum_abs_diagonal) << ':'
        << al_r20_q_scalar_root(result.input) << ':'
        << al_r20_q_scalar_root(result.reflectors) << ':'
        << al_r20_q_scalar_root(result.tau) << ':'
        << al_r20_q_scalar_root(result.q) << ':'
        << al_r20_q_scalar_root(result.r);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63GDFactor {
    bool exact = false;
    std::size_t rows = 0U;
    std::size_t columns = 0U;
    std::size_t nonzero_diagonal = 0U;
    std::size_t norm_terms = 0U;
    std::size_t reflector_products = 0U;
    std::size_t q_products = 0U;
    double minimum_abs_diagonal = std::numeric_limits<double>::infinity();
    double maximum_abs_diagonal = 0.0;
    std::vector<double> input;
    std::vector<double> reflectors;
    std::vector<double> tau;
    std::vector<double> q;
    std::vector<double> r;
    std::string root;
};

ALR20R63GDFactor al_r20_r63g_d_factor(
    const std::vector<double>& input, std::size_t rows,
    std::size_t columns) {
    ALR20R63GDFactor result;
    result.rows = rows;
    result.columns = columns;
    result.input = input;
    if (rows < columns || input.size() != rows * columns) return result;
    std::vector<double> work = input;
    result.reflectors.assign(rows * columns, 0.0);
    result.tau.assign(columns, 0.0);
    result.r.assign(columns * columns, 0.0);
    bool finite = true;
    for (std::size_t column = 0U; column < columns; ++column) {
        const double norm = al_r20_r63g_d_norm(work, rows, columns,
            column, column, &result.norm_terms);
        const double first = work[column * columns + column];
        const double beta = first >= 0.0 ? -norm : norm;
        const double head = first - beta;
        finite = finite && std::isfinite(norm) && std::isfinite(beta)
            && std::isfinite(head) && norm > 0.0 && head != 0.0
            && beta != 0.0;
        if (!finite) break;
        result.reflectors[column * columns + column] = 1.0;
        for (std::size_t row = column + 1U; row < rows; ++row) {
            result.reflectors[row * columns + column] =
                work[row * columns + column] / head;
            finite = finite && std::isfinite(
                result.reflectors[row * columns + column]);
        }
        result.tau[column] = (beta - first) / beta;
        finite = finite && std::isfinite(result.tau[column]);
        work[column * columns + column] = beta;
        for (std::size_t other = column + 1U;
             other < columns; ++other) {
            double dot = 0.0;
            for (std::size_t row = column; row < rows; ++row) {
                dot += result.reflectors[row * columns + column]
                    * work[row * columns + other];
                ++result.reflector_products;
            }
            const double update = result.tau[column] * dot;
            for (std::size_t row = column; row < rows; ++row) {
                work[row * columns + other] -=
                    result.reflectors[row * columns + column] * update;
                ++result.reflector_products;
                finite = finite
                    && std::isfinite(work[row * columns + other]);
            }
        }
        for (std::size_t other = column; other < columns; ++other)
            result.r[column * columns + other] =
                work[column * columns + other];
        result.minimum_abs_diagonal = std::min(
            result.minimum_abs_diagonal, std::abs(beta));
        result.maximum_abs_diagonal = std::max(
            result.maximum_abs_diagonal, std::abs(beta));
        ++result.nonzero_diagonal;
    }
    result.q.assign(rows * columns, 0.0);
    for (std::size_t index = 0U; index < columns; ++index)
        result.q[index * columns + index] = 1.0;
    if (finite && result.nonzero_diagonal == columns) {
        for (std::size_t reverse = columns; reverse-- > 0U;)
            for (std::size_t q_column = 0U;
                 q_column < columns; ++q_column) {
                double dot = 0.0;
                for (std::size_t row = reverse; row < rows; ++row) {
                    dot += result.reflectors[row * columns + reverse]
                        * result.q[row * columns + q_column];
                    ++result.q_products;
                }
                const double update = result.tau[reverse] * dot;
                for (std::size_t row = reverse; row < rows; ++row) {
                    result.q[row * columns + q_column] -=
                        result.reflectors[row * columns + reverse] * update;
                    ++result.q_products;
                    finite = finite
                        && std::isfinite(result.q[row * columns + q_column]);
                }
            }
    }
    result.exact = finite && result.nonzero_diagonal == columns;
    std::ostringstream material;
    material << result.exact << ':' << result.rows << ':' << result.columns
        << ':' << result.nonzero_diagonal << ':' << result.norm_terms << ':'
        << result.reflector_products << ':' << result.q_products << ':'
        << binary64_bits(result.minimum_abs_diagonal) << ':'
        << binary64_bits(result.maximum_abs_diagonal) << ':'
        << al_r20_double_root(result.input) << ':'
        << al_r20_double_root(result.reflectors) << ':'
        << al_r20_double_root(result.tau) << ':'
        << al_r20_double_root(result.q) << ':'
        << al_r20_double_root(result.r);
    result.root = sha256_hex(material.str());
    return result;
}

std::vector<Binary128> al_r20_r63g_q_from_double(
    const std::vector<double>& input) {
    std::vector<Binary128> result;
    result.reserve(input.size());
    for (double value : input) result.push_back(
        static_cast<Binary128>(value));
    return result;
}

struct ALR20R63GFactorAudit {
    bool exact = false;
    bool finite = false;
    std::size_t orthogonality_entries = 0U;
    std::size_t reconstruction_entries = 0U;
    std::size_t gram_entries = 0U;
    std::size_t orthogonality_products = 0U;
    std::size_t reconstruction_products = 0U;
    std::size_t gram_products = 0U;
    std::size_t orthogonality_contained = 0U;
    std::size_t reconstruction_contained = 0U;
    std::size_t gram_contained = 0U;
    Binary128 orthogonality_residual = static_cast<Binary128>(0.0);
    Binary128 reconstruction_residual = static_cast<Binary128>(0.0);
    Binary128 gram_residual = static_cast<Binary128>(0.0);
    Binary128 orthogonality_bound = static_cast<Binary128>(0.0);
    Binary128 reconstruction_bound = static_cast<Binary128>(0.0);
    Binary128 gram_bound = static_cast<Binary128>(0.0);
    std::string root;
};

ALR20R63GFactorAudit al_r20_r63g_factor_audit(
    const std::vector<Binary128>& input,
    const std::vector<Binary128>& q,
    const std::vector<Binary128>& r,
    const std::vector<Binary128>& gram,
    const std::vector<std::size_t>& permutation,
    std::size_t rows, std::size_t columns, bool require_containment) {
    ALR20R63GFactorAudit result;
    if (input.size() != rows * columns || q.size() != rows * columns
        || r.size() != columns * columns
        || gram.size() != columns * columns
        || !al_r20_r63g_permutation_exact(permutation, columns)) return result;
    const Binary128 base_bound = al_r20_q_gamma(
        2048U * rows * columns + 131072U);
    bool finite = true;
    for (std::size_t first = 0U; first < columns; ++first)
        for (std::size_t second = 0U; second < columns; ++second) {
            Binary128Accumulator dot;
            Binary128 absolute = static_cast<Binary128>(0.0);
            for (std::size_t row = 0U; row < rows; ++row) {
                const Binary128 product = q[row * columns + first]
                    * q[row * columns + second];
                dot.add(product);
                absolute += fabsq(product);
                ++result.orthogonality_products;
            }
            const Binary128 expected = first == second
                ? static_cast<Binary128>(1.0)
                : static_cast<Binary128>(0.0);
            const Binary128 residual = fabsq(dot.value() - expected);
            const Binary128 bound = base_bound * fmaxq(
                static_cast<Binary128>(1.0), absolute);
            result.orthogonality_residual = fmaxq(
                result.orthogonality_residual, residual);
            result.orthogonality_bound = fmaxq(
                result.orthogonality_bound, bound);
            if (residual <= bound) ++result.orthogonality_contained;
            finite = finite && finiteq(dot.value()) != 0
                && finiteq(residual) != 0 && finiteq(bound) != 0;
            ++result.orthogonality_entries;
        }
    for (std::size_t row = 0U; row < rows; ++row)
        for (std::size_t column = 0U; column < columns; ++column) {
            Binary128Accumulator dot;
            Binary128 absolute = static_cast<Binary128>(0.0);
            for (std::size_t inner = 0U; inner < columns; ++inner) {
                const Binary128 product = q[row * columns + inner]
                    * r[inner * columns + column];
                dot.add(product);
                absolute += fabsq(product);
                ++result.reconstruction_products;
            }
            const Binary128 residual = fabsq(
                dot.value() - input[row * columns + column]);
            const Binary128 bound = base_bound * fmaxq(
                static_cast<Binary128>(1.0), absolute);
            result.reconstruction_residual = fmaxq(
                result.reconstruction_residual, residual);
            result.reconstruction_bound = fmaxq(
                result.reconstruction_bound, bound);
            if (residual <= bound) ++result.reconstruction_contained;
            finite = finite && finiteq(dot.value()) != 0
                && finiteq(residual) != 0 && finiteq(bound) != 0;
            ++result.reconstruction_entries;
        }
    for (std::size_t first = 0U; first < columns; ++first)
        for (std::size_t second = 0U; second < columns; ++second) {
            Binary128Accumulator dot;
            Binary128 absolute = static_cast<Binary128>(0.0);
            for (std::size_t inner = 0U; inner < columns; ++inner) {
                const Binary128 product = r[inner * columns + first]
                    * r[inner * columns + second];
                dot.add(product);
                absolute += fabsq(product);
                ++result.gram_products;
            }
            const Binary128 expected = gram[
                permutation[first] * columns + permutation[second]];
            const Binary128 residual = fabsq(dot.value() - expected);
            const Binary128 bound = base_bound * fmaxq(
                static_cast<Binary128>(1.0), absolute);
            result.gram_residual = fmaxq(result.gram_residual, residual);
            result.gram_bound = fmaxq(result.gram_bound, bound);
            if (residual <= bound) ++result.gram_contained;
            finite = finite && finiteq(dot.value()) != 0
                && finiteq(residual) != 0 && finiteq(bound) != 0;
            ++result.gram_entries;
        }
    result.finite = finite;
    const bool contained = result.orthogonality_contained
            == result.orthogonality_entries
        && result.reconstruction_contained == result.reconstruction_entries
        && result.gram_contained == result.gram_entries;
    result.exact = finite && (!require_containment || contained);
    std::ostringstream material;
    material << result.exact << ':' << result.finite << ':'
        << result.orthogonality_entries << ':'
        << result.reconstruction_entries << ':' << result.gram_entries << ':'
        << result.orthogonality_products << ':'
        << result.reconstruction_products << ':' << result.gram_products << ':'
        << result.orthogonality_contained << ':'
        << result.reconstruction_contained << ':' << result.gram_contained
        << ':' << al_binary128_hex(result.orthogonality_residual) << ':'
        << al_binary128_hex(result.reconstruction_residual) << ':'
        << al_binary128_hex(result.gram_residual) << ':'
        << al_binary128_hex(result.orthogonality_bound) << ':'
        << al_binary128_hex(result.reconstruction_bound) << ':'
        << al_binary128_hex(result.gram_bound);
    result.root = sha256_hex(material.str());
    return result;
}

struct ALR20R63GSignal {
    bool exact = false;
    bool survives = false;
    std::size_t scalars = 0U;
    Binary128 norm_squared = static_cast<Binary128>(0.0);
    Binary128 error_squared = static_cast<Binary128>(0.0);
    Binary128 reference_norm_squared = static_cast<Binary128>(0.0);
    Binary128 maximum_residual = static_cast<Binary128>(0.0);
    std::vector<Binary128> values;
    std::vector<Binary128> error;
    std::string root;
};

ALR20R63GSignal al_r20_r63g_signal(
    const std::vector<Binary128>& values,
    const std::vector<Binary128>& reference) {
    ALR20R63GSignal result;
    result.scalars = values.size();
    result.values = values;
    if (values.size() != reference.size() || values.empty()) return result;
    result.error.resize(values.size());
    Binary128Accumulator norm;
    Binary128Accumulator error;
    Binary128Accumulator reference_norm;
    bool finite = true;
    for (std::size_t index = 0U; index < values.size(); ++index) {
        result.error[index] = values[index] - reference[index];
        norm.add(values[index] * values[index]);
        error.add(result.error[index] * result.error[index]);
        reference_norm.add(reference[index] * reference[index]);
        result.maximum_residual = fmaxq(result.maximum_residual,
            fabsq(result.error[index]));
        finite = finite && finiteq(values[index]) != 0
            && finiteq(result.error[index]) != 0;
    }
    result.norm_squared = norm.value();
    result.error_squared = error.value();
    result.reference_norm_squared = reference_norm.value();
    result.survives = result.error_squared < result.reference_norm_squared;
    result.exact = finite && result.reference_norm_squared
        > static_cast<Binary128>(0.0);
    std::ostringstream material;
    material << result.exact << ':' << result.survives << ':'
        << result.scalars << ':' << al_binary128_hex(result.norm_squared)
        << ':' << al_binary128_hex(result.error_squared) << ':'
        << al_binary128_hex(result.reference_norm_squared) << ':'
        << al_binary128_hex(result.maximum_residual) << ':'
        << al_r20_q_scalar_root(result.values) << ':'
        << al_r20_q_scalar_root(result.error) << ':'
        << al_r20_q_scalar_root(reference);
    result.root = sha256_hex(material.str());
    return result;
}

std::vector<Binary128> al_r20_r63g_gram(
    const std::vector<Binary128>& input, std::size_t rows,
    std::size_t columns) {
    if (input.size() != rows * columns) return {};
    std::vector<Binary128> result(columns * columns);
    for (std::size_t first = 0U; first < columns; ++first)
        for (std::size_t second = 0U; second < columns; ++second) {
            Binary128Accumulator dot;
            for (std::size_t row = 0U; row < rows; ++row)
                dot.add(input[row * columns + first]
                    * input[row * columns + second]);
            result[first * columns + second] = dot.value();
        }
    return result;
}

std::vector<Binary128> al_r20_r63g_direct_image(
    const std::vector<Binary128>& matrix, std::size_t rows,
    std::size_t columns, const std::vector<Binary128>& alpha,
    std::size_t* products) {
    std::vector<Binary128> result(rows);
    if (matrix.size() != rows * columns || alpha.size() != columns)
        return {};
    for (std::size_t row = 0U; row < rows; ++row) {
        Binary128Accumulator dot;
        for (std::size_t column = 0U; column < columns; ++column) {
            dot.add(matrix[row * columns + column] * alpha[column]);
            ++(*products);
        }
        result[row] = dot.value();
    }
    return result;
}

std::vector<Binary128> al_r20_r63g_factor_image(
    const std::vector<Binary128>& q,
    const std::vector<Binary128>& r,
    std::size_t rows, std::size_t columns,
    const std::vector<std::size_t>& permutation,
    const std::vector<Binary128>& alpha, std::size_t* products) {
    if (q.size() != rows * columns || r.size() != columns * columns
        || alpha.size() != columns
        || !al_r20_r63g_permutation_exact(permutation, columns)) return {};
    std::vector<Binary128> permuted(columns);
    for (std::size_t index = 0U; index < columns; ++index)
        permuted[index] = alpha[permutation[index]];
    std::vector<Binary128> intermediate(columns);
    for (std::size_t row = 0U; row < columns; ++row) {
        Binary128Accumulator dot;
        for (std::size_t column = 0U; column < columns; ++column) {
            dot.add(r[row * columns + column] * permuted[column]);
            ++(*products);
        }
        intermediate[row] = dot.value();
    }
    std::vector<Binary128> result(rows);
    for (std::size_t row = 0U; row < rows; ++row) {
        Binary128Accumulator dot;
        for (std::size_t column = 0U; column < columns; ++column) {
            dot.add(q[row * columns + column] * intermediate[column]);
            ++(*products);
        }
        result[row] = dot.value();
    }
    return result;
}

std::string al_r20_r63g_classify(bool apparatus, bool reference,
    bool storage_survives, bool factor_survives) {
    if (!apparatus) return "RECTANGULAR_FACTOR_APPARATUS_REJECTED";
    if (!reference) return "RECTANGULAR_BINARY128_REFERENCE_REJECTED";
    if (!storage_survives) return "BINARY64_RECTANGULAR_STORAGE_FLOOR";
    if (!factor_survives) return "BINARY64_RECTANGULAR_FACTOR_FLOOR";
    return "BINARY64_NO_DROP_FACTOR_CANDIDATE";
}

struct ALR20R63GControls {
    bool exact = false;
    bool factors = false;
    bool near_dependent = false;
    bool duplicate = false;
    bool permutation = false;
    bool signal = false;
    bool mutation = false;
    bool shrunk = false;
    bool classifier = false;
    std::string root;
};

ALR20R63GControls al_r20_r63g_controls() {
    ALR20R63GControls result;
    const std::vector<Binary128> tall{
        static_cast<Binary128>(1.0), static_cast<Binary128>(0.5),
            static_cast<Binary128>(0.0),
        static_cast<Binary128>(0.0), static_cast<Binary128>(1.0),
            static_cast<Binary128>(0.25),
        static_cast<Binary128>(0.5), static_cast<Binary128>(0.0),
            static_cast<Binary128>(1.0),
        static_cast<Binary128>(0.25), static_cast<Binary128>(-0.5),
            static_cast<Binary128>(0.5)};
    std::vector<double> tall_d;
    for (Binary128 value : tall) tall_d.push_back(static_cast<double>(value));
    const ALR20R63GQFactor tall_q = al_r20_r63g_q_factor(tall, 4U, 3U);
    const ALR20R63GDFactor tall_f = al_r20_r63g_d_factor(tall_d, 4U, 3U);
    const std::vector<Binary128> tall_gram =
        al_r20_r63g_gram(tall, 4U, 3U);
    const std::vector<std::size_t> identity{0U, 1U, 2U};
    const ALR20R63GFactorAudit tall_q_audit = al_r20_r63g_factor_audit(
        tall, tall_q.q, tall_q.r, tall_gram, identity, 4U, 3U, true);
    const ALR20R63GFactorAudit tall_d_audit = al_r20_r63g_factor_audit(
        al_r20_r63g_q_from_double(tall_f.input),
        al_r20_r63g_q_from_double(tall_f.q),
        al_r20_r63g_q_from_double(tall_f.r), tall_gram, identity,
        4U, 3U, false);
    const Binary128 double_control_bound = ldexpq(
        static_cast<Binary128>(65536.0), -52);
    result.factors = tall_q.exact && tall_f.exact && tall_q_audit.exact
        && tall_d_audit.finite
        && tall_d_audit.orthogonality_residual <= double_control_bound
        && tall_d_audit.reconstruction_residual <= double_control_bound
        && tall_d_audit.gram_residual <= double_control_bound;

    const Binary128 tiny = ldexpq(static_cast<Binary128>(1.0), -40);
    const std::vector<Binary128> near{
        static_cast<Binary128>(1.0), static_cast<Binary128>(1.0),
        static_cast<Binary128>(0.0), tiny,
        static_cast<Binary128>(0.0), static_cast<Binary128>(0.0)};
    std::vector<double> near_d;
    for (Binary128 value : near) near_d.push_back(static_cast<double>(value));
    result.near_dependent = al_r20_r63g_q_factor(near, 3U, 2U).exact
        && al_r20_r63g_d_factor(near_d, 3U, 2U).exact;
    const std::vector<Binary128> duplicate{
        static_cast<Binary128>(1.0), static_cast<Binary128>(1.0),
        static_cast<Binary128>(0.0), static_cast<Binary128>(0.0),
        static_cast<Binary128>(0.0), static_cast<Binary128>(0.0)};
    std::vector<double> duplicate_d;
    for (Binary128 value : duplicate)
        duplicate_d.push_back(static_cast<double>(value));
    result.duplicate = !al_r20_r63g_q_factor(duplicate, 3U, 2U).exact
        && !al_r20_r63g_d_factor(duplicate_d, 3U, 2U).exact;

    const std::vector<std::size_t> permutation{2U, 0U, 1U};
    std::vector<Binary128> permuted_tall(tall.size());
    for (std::size_t row = 0U; row < 4U; ++row)
        for (std::size_t column = 0U; column < 3U; ++column)
            permuted_tall[row * 3U + column] =
                tall[row * 3U + permutation[column]];
    const ALR20R63GQFactor permuted_factor =
        al_r20_r63g_q_factor(permuted_tall, 4U, 3U);
    const ALR20R63GFactorAudit permuted_audit =
        al_r20_r63g_factor_audit(permuted_tall, permuted_factor.q,
            permuted_factor.r, tall_gram, permutation, 4U, 3U, true);
    result.permutation = al_r20_r63g_permutation_exact(permutation, 3U)
        && !al_r20_r63g_permutation_exact({2U, 0U, 0U}, 3U)
        && permuted_factor.exact && permuted_audit.exact;
    const std::vector<Binary128> reference{
        tiny, static_cast<Binary128>(0.0)};
    const ALR20R63GSignal inside = al_r20_r63g_signal(
        {tiny + tiny / static_cast<Binary128>(2.0),
            static_cast<Binary128>(0.0)}, reference);
    const ALR20R63GSignal outside = al_r20_r63g_signal(
        {static_cast<Binary128>(3.0) * tiny,
            static_cast<Binary128>(0.0)}, reference);
    result.signal = inside.exact && inside.survives
        && outside.exact && !outside.survives;
    std::vector<Binary128> mutated = tall;
    mutated[3U] = nextafterq(mutated[3U], HUGE_VALQ);
    result.mutation = al_r20_q_scalar_root(tall)
        != al_r20_q_scalar_root(mutated);
    const Binary128 bound = ldexpq(static_cast<Binary128>(1.0), -100);
    const Binary128 shrunk = nextafterq(bound,
        static_cast<Binary128>(0.0));
    result.shrunk = shrunk < bound && !(bound <= shrunk);
    result.classifier =
        al_r20_r63g_classify(true, false, true, true)
            == "RECTANGULAR_BINARY128_REFERENCE_REJECTED"
        && al_r20_r63g_classify(true, true, false, true)
            == "BINARY64_RECTANGULAR_STORAGE_FLOOR"
        && al_r20_r63g_classify(true, true, true, false)
            == "BINARY64_RECTANGULAR_FACTOR_FLOOR"
        && al_r20_r63g_classify(true, true, true, true)
            == "BINARY64_NO_DROP_FACTOR_CANDIDATE";
    result.exact = result.factors && result.near_dependent
        && result.duplicate && result.permutation && result.signal
        && result.mutation && result.shrunk && result.classifier;
    result.root = sha256_hex(std::to_string(result.exact) + ':'
        + tall_q.root + ':' + tall_f.root + ':' + tall_q_audit.root + ':'
        + tall_d_audit.root + ':' + permuted_factor.root + ':'
        + permuted_audit.root + ':'
        + al_r20_q_scalar_root(near) + ':'
        + al_r20_q_scalar_root(duplicate) + ':'
        + al_r20_r63g_index_root(permutation) + ':' + inside.root + ':'
        + outside.root + ':' + al_r20_q_scalar_root(mutated));
    return result;
}

} // namespace

SplitBoundaryReport run_al_generalization_v5_rectangular_factor_precision_controls() {
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
    const ALR20R63GControls controls = al_r20_r63g_controls();
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
        && factor_capture.source_rows.size() == 102U
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
    const ALR20R63EWitness inherited = full_profile.exact
        ? al_r20_r63e_witness(
            grams, factor_capture.projector, full_profile.binary128)
        : ALR20R63EWitness{};
    const ALR20R63FTangentMatrix tangent = grams.exact
        ? al_r20_r63f_tangent_matrix(grams, factor_capture.projector,
            factor_capture.source_rows)
        : ALR20R63FTangentMatrix{};
    const ALR20R63FWitnessImage inherited_image = tangent.exact
        && inherited.exact
        ? al_r20_r63f_witness_image(tangent, inherited.alpha,
            inherited.decomposition.tangent)
        : ALR20R63FWitnessImage{};
    const std::vector<std::size_t> permutation = full_profile.exact
        ? full_profile.binary128.permutation
        : std::vector<std::size_t>{};
    const bool permutation_exact = al_r20_r63g_permutation_exact(
        permutation, tangent.rows);

    std::vector<Binary128> transposed;
    std::vector<Binary128> permuted_q;
    std::vector<double> permuted_d;
    if (tangent.exact && permutation_exact) {
        transposed.resize(tangent.columns * tangent.rows);
        permuted_q.resize(transposed.size());
        permuted_d.resize(transposed.size());
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column)
                transposed[scalar * tangent.rows + column] =
                    tangent.values[column * tangent.columns + scalar];
        for (std::size_t scalar = 0U; scalar < tangent.columns; ++scalar)
            for (std::size_t column = 0U; column < tangent.rows; ++column) {
                permuted_q[scalar * tangent.rows + column] =
                    transposed[scalar * tangent.rows + permutation[column]];
                permuted_d[scalar * tangent.rows + column] =
                    static_cast<double>(
                        permuted_q[scalar * tangent.rows + column]);
            }
    }
    const ALR20R63GQFactor q_factor = !permuted_q.empty()
        ? al_r20_r63g_q_factor(
            permuted_q, tangent.columns, tangent.rows)
        : ALR20R63GQFactor{};
    const ALR20R63GDFactor d_factor = !permuted_d.empty()
        ? al_r20_r63g_d_factor(
            permuted_d, tangent.columns, tangent.rows)
        : ALR20R63GDFactor{};
    const ALR20R63GFactorAudit q_audit = q_factor.exact
        ? al_r20_r63g_factor_audit(permuted_q, q_factor.q, q_factor.r,
            grams.tangent, permutation, tangent.columns, tangent.rows, true)
        : ALR20R63GFactorAudit{};
    const ALR20R63GFactorAudit d_audit = d_factor.exact
        ? al_r20_r63g_factor_audit(
            al_r20_r63g_q_from_double(d_factor.input),
            al_r20_r63g_q_from_double(d_factor.q),
            al_r20_r63g_q_from_double(d_factor.r), grams.tangent,
            permutation, tangent.columns, tangent.rows, false)
        : ALR20R63GFactorAudit{};

    std::vector<Binary128> reference;
    if (inherited_image.exact) {
        reference.reserve(inherited_image.direct.size() * 3U);
        for (const Binary128Vec3& value : inherited_image.direct)
            for (int axis = 0; axis < 3; ++axis)
                reference.push_back(al_r20_q_component(value, axis));
    }
    std::size_t direct_products = 0U;
    std::size_t rounded_alpha_products = 0U;
    std::size_t q_factor_products = 0U;
    std::size_t d_factor_products = 0U;
    std::vector<Binary128> rounded_matrix = transposed;
    for (Binary128& value : rounded_matrix)
        value = static_cast<Binary128>(static_cast<double>(value));
    std::vector<Binary128> rounded_alpha = inherited.alpha;
    for (Binary128& value : rounded_alpha)
        value = static_cast<Binary128>(static_cast<double>(value));
    const ALR20R63GSignal storage_signal = !reference.empty()
        ? al_r20_r63g_signal(al_r20_r63g_direct_image(rounded_matrix,
            tangent.columns, tangent.rows, inherited.alpha,
            &direct_products), reference)
        : ALR20R63GSignal{};
    const ALR20R63GSignal alpha_signal = !reference.empty()
        ? al_r20_r63g_signal(al_r20_r63g_direct_image(transposed,
            tangent.columns, tangent.rows, rounded_alpha,
            &rounded_alpha_products), reference)
        : ALR20R63GSignal{};
    const ALR20R63GSignal q_factor_signal = q_factor.exact
        && !reference.empty()
        ? al_r20_r63g_signal(al_r20_r63g_factor_image(q_factor.q,
            q_factor.r, tangent.columns, tangent.rows, permutation,
            inherited.alpha, &q_factor_products), reference)
        : ALR20R63GSignal{};
    const ALR20R63GSignal d_factor_signal = d_factor.exact
        && !reference.empty()
        ? al_r20_r63g_signal(al_r20_r63g_factor_image(
            al_r20_r63g_q_from_double(d_factor.q),
            al_r20_r63g_q_from_double(d_factor.r), tangent.columns,
            tangent.rows, permutation, inherited.alpha,
            &d_factor_products), reference)
        : ALR20R63GSignal{};

    if (problem_materialized)
        release_al_normalized_workspace(problem.workspace, &sparse_work);
    const bool lifecycle_exact = sparse_work.workspace_releases == 1U
        && sparse_work.live_workspaces == 0U
        && !sparse_work.lifecycle_underflow;
    const std::size_t expected_norm_terms = 26979U;
    const std::size_t expected_orthogonality_products = 3277260U;
    const std::size_t expected_reconstruction_products = 3277260U;
    const std::size_t expected_gram_products = 1061208U;
    const bool work_exact = tangent.rows == 102U
        && tangent.columns == 315U
        && q_factor.norm_terms == expected_norm_terms
        && d_factor.norm_terms == expected_norm_terms
        && q_factor.reflector_products == d_factor.reflector_products
        && q_factor.q_products == d_factor.q_products
        && q_audit.orthogonality_products
            == expected_orthogonality_products
        && q_audit.reconstruction_products
            == expected_reconstruction_products
        && q_audit.gram_products == expected_gram_products
        && d_audit.orthogonality_products
            == expected_orthogonality_products
        && d_audit.reconstruction_products
            == expected_reconstruction_products
        && d_audit.gram_products == expected_gram_products
        && direct_products == 32130U
        && rounded_alpha_products == 32130U
        && q_factor_products == 42534U
        && d_factor_products == 42534U;
    const bool reference_exact = q_factor.exact && q_audit.exact
        && q_factor_signal.exact && q_factor_signal.survives;
    const bool apparatus_exact = profile_exact && controls_exact
        && capture_exact && grams.exact && full_profile.exact
        && full_profile.binary128.rank == 101U && inherited.exact
        && tangent.exact && inherited_image.exact && permutation_exact
        && transposed.size() == 32130U && q_factor.exact && d_factor.exact
        && q_audit.exact && d_audit.exact && storage_signal.exact
        && alpha_signal.exact && q_factor_signal.exact
        && d_factor_signal.exact && lifecycle_exact && work_exact;
    const std::string route = al_r20_r63g_classify(apparatus_exact,
        reference_exact, storage_signal.survives, d_factor_signal.survives);
    const bool exact = apparatus_exact && reference_exact;

    std::ostringstream semantic;
    semantic
        << "nextengine.nonlocal.nsr3b4e2d7r20r63g-rectangular-factor-precision.v1|"
        << AL_R20_R63G_PARENT_SEMANTIC_SHA256 << ':' << route << ':'
        << oblique.root << ':' << factor_capture.matrix_root << ':'
        << controls.root << ':' << tangent.root << ':'
        << al_r20_r63g_index_root(permutation) << ':' << q_factor.root << ':'
        << d_factor.root << ':' << q_audit.root << ':' << d_audit.root << ':'
        << storage_signal.root << ':' << alpha_signal.root << ':'
        << q_factor_signal.root << ':' << d_factor_signal.root;
    const std::string result_sha = sha256_hex(semantic.str());

    std::ostringstream report;
    report << std::setprecision(17)
        << "{\"schema\":\"nextengine.nonlocal.nsr3b4e2d7r20r63g_rectangular_factor_precision.v1\""
        << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
        << ",\"route\":\"" << route << "\""
        << ",\"parent_r63f_sha256\":\""
        << AL_R20_R63G_PARENT_SEMANTIC_SHA256 << "\""
        << ",\"profile_exact\":" << (profile_exact ? "true" : "false")
        << ",\"controls\":{\"exact\":"
        << (controls_exact ? "true" : "false")
        << ",\"factors\":" << (controls.factors ? "true" : "false")
        << ",\"near_dependent\":"
        << (controls.near_dependent ? "true" : "false")
        << ",\"duplicate\":" << (controls.duplicate ? "true" : "false")
        << ",\"permutation\":"
        << (controls.permutation ? "true" : "false")
        << ",\"signal\":" << (controls.signal ? "true" : "false")
        << ",\"mutation\":" << (controls.mutation ? "true" : "false")
        << ",\"shrunk\":" << (controls.shrunk ? "true" : "false")
        << ",\"classifier\":"
        << (controls.classifier ? "true" : "false")
        << ",\"root\":\"" << controls.root << "\"}"
        << ",\"capture\":{\"exact\":"
        << (capture_exact ? "true" : "false")
        << ",\"matrix_root\":\"" << factor_capture.matrix_root << "\"}"
        << ",\"operator\":{\"rows\":315,\"columns\":102"
        << ",\"permutation_exact\":"
        << (permutation_exact ? "true" : "false")
        << ",\"permutation_root\":\""
        << al_r20_r63g_index_root(permutation) << "\""
        << ",\"transpose_root\":\""
        << al_r20_q_scalar_root(transposed) << "\"}"
        << ",\"binary128_factor\":{\"exact\":"
        << (q_factor.exact ? "true" : "false")
        << ",\"nonzero_diagonal\":" << q_factor.nonzero_diagonal
        << ",\"minimum_abs_diagonal\":\""
        << al_binary128_text(q_factor.minimum_abs_diagonal) << "\""
        << ",\"maximum_abs_diagonal\":\""
        << al_binary128_text(q_factor.maximum_abs_diagonal) << "\""
        << ",\"root\":\"" << q_factor.root << "\"}"
        << ",\"binary64_factor\":{\"exact\":"
        << (d_factor.exact ? "true" : "false")
        << ",\"nonzero_diagonal\":" << d_factor.nonzero_diagonal
        << ",\"minimum_abs_diagonal\":"
        << d_factor.minimum_abs_diagonal
        << ",\"maximum_abs_diagonal\":"
        << d_factor.maximum_abs_diagonal
        << ",\"root\":\"" << d_factor.root << "\"}"
        << ",\"binary128_audit\":{\"exact\":"
        << (q_audit.exact ? "true" : "false")
        << ",\"orthogonality_residual\":\""
        << al_binary128_text(q_audit.orthogonality_residual)
        << "\",\"reconstruction_residual\":\""
        << al_binary128_text(q_audit.reconstruction_residual)
        << "\",\"gram_residual\":\""
        << al_binary128_text(q_audit.gram_residual)
        << "\",\"root\":\"" << q_audit.root << "\"}"
        << ",\"binary64_audit\":{\"finite\":"
        << (d_audit.finite ? "true" : "false")
        << ",\"orthogonality_residual\":\""
        << al_binary128_text(d_audit.orthogonality_residual)
        << "\",\"reconstruction_residual\":\""
        << al_binary128_text(d_audit.reconstruction_residual)
        << "\",\"gram_residual\":\""
        << al_binary128_text(d_audit.gram_residual)
        << "\",\"root\":\"" << d_audit.root << "\"}"
        << ",\"signals\":{\"reference_norm_squared\":\""
        << al_binary128_text(q_factor_signal.reference_norm_squared) << "\""
        << ",\"storage_survives\":"
        << (storage_signal.survives ? "true" : "false")
        << ",\"storage_error_squared\":\""
        << al_binary128_text(storage_signal.error_squared) << "\""
        << ",\"rounded_alpha_survives\":"
        << (alpha_signal.survives ? "true" : "false")
        << ",\"rounded_alpha_error_squared\":\""
        << al_binary128_text(alpha_signal.error_squared) << "\""
        << ",\"binary128_factor_survives\":"
        << (q_factor_signal.survives ? "true" : "false")
        << ",\"binary128_factor_error_squared\":\""
        << al_binary128_text(q_factor_signal.error_squared) << "\""
        << ",\"binary64_factor_survives\":"
        << (d_factor_signal.survives ? "true" : "false")
        << ",\"binary64_factor_error_squared\":\""
        << al_binary128_text(d_factor_signal.error_squared) << "\"}"
        << ",\"reference_exact\":"
        << (reference_exact ? "true" : "false")
        << ",\"apparatus_exact\":"
        << (apparatus_exact ? "true" : "false")
        << ",\"work_exact\":" << (work_exact ? "true" : "false")
        << ",\"lifecycle_exact\":"
        << (lifecycle_exact ? "true" : "false")
        << ",\"solver_executions\":" << solver_executions
        << ",\"qr_factorizations\":2,\"nnqp_rhs_solves\":0"
        << ",\"lsqr_solves\":0,\"svd_factorizations\":0"
        << ",\"rank_actions\":0,\"row_drops\":0,\"regularizations\":0"
        << ",\"center_iterations\":0,\"replacement_applies\":0"
        << ",\"state_updates\":0,\"timing_admitted\":false"
        << ",\"runtime_authority\":false,\"gpu_authority\":false"
        << ",\"production_authority\":false"
        << ",\"result_sha256\":\"" << result_sha << "\"}";
    return {exact, report.str()};
}
