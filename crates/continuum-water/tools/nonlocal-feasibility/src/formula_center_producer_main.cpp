#define NEXTENGINE_R63ZG_EMBEDDED
#include "formula_certificate_detail_main.cpp"
#undef NEXTENGINE_R63ZG_EMBEDDED

#include <boost/multiprecision/cpp_int.hpp>

#include <array>
#include <climits>

namespace {

using boost::multiprecision::cpp_int;
using nextengine::nonlocal::fcr::FormulaProbeProfile;
using nextengine::nonlocal::fcr::FormulaProbeSolutionExpansion;

constexpr const char* R63ZG_RESULT_SHA256 =
    "989886a4707e03b76eea48ad92bfcbc1721eed9ba679edfed342079f01c6a5e5";
constexpr const char* R63ZG_STDOUT_SHA256 =
    "ca2a0f80b692a466aa97b4725fcc0ac3f553db95bb7ddf72494a754a7e2029c9";
constexpr const char* R63ZG_DETAIL_WORK_SHA256 =
    "5ce8dd9c0a6b2bbdf7b5cc00ec22775d91b5f2131525aecfbd6b31b4e5edd0ed";
constexpr const char* R63ZG_OBSERVATIONS_SHA256 =
    "0d506330c92bd05d4a48c3a6c76b3950420ee236a11798d021c333b863ff0804";
constexpr const char* R63ZG_CONTROLS_SHA256 =
    "c3d2d81f6aa4a516990c57fc64551d9f1a49a2cfbd45a270c31a1ff0cbe831e8";
constexpr const char* R63ZG_TRANSACTION_SHA256 =
    "4e322f5131d7d3bdc7ea19f96136582f53cb078036950321fab1476de570c2e4";

std::string solution_expansion_root(
    const FormulaProbeSolutionExpansion& value) {
    std::ostringstream material;
    material << value.exact << ':' << value.width << ':' << value.dimension
        << ':' << value.containments << ':' << value.nonzero_lows << ':'
        << value.source_solution_root << ':' << value.component_root << ':'
        << value.radius_root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

std::string internal_solution_root(
    const FormulaProbeSolutionExpansion& value) {
    std::ostringstream material;
    material << value.exact << ':' << value.width << ':' << value.dimension
        << ':' << value.containments << ':' << value.nonzero_lows << ':'
        << value.component_root << ':' << value.radius_root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

bool solution_expansion_valid(const FormulaProbeSolutionExpansion& value) {
    using nextengine::nonlocal::fcr::formula_probe_binary64_vector_root;
    bool finite = true;
    for (double component : value.components)
        finite = finite && std::isfinite(component);
    for (double radius : value.radius)
        finite = finite && std::isfinite(radius) && radius >= 0.0;
    return value.exact && value.width == 2U && value.dimension == DIMENSION
        && value.containments == value.dimension
        && value.components.size() == value.dimension * value.width
        && value.radius.size() == value.dimension && finite
        && value.component_root
            == formula_probe_binary64_vector_root(value.components)
        && value.radius_root
            == formula_probe_binary64_vector_root(value.radius)
        && value.root == solution_expansion_root(value);
}

void reseal_solution_expansion(FormulaProbeSolutionExpansion& value) {
    using nextengine::nonlocal::fcr::formula_probe_binary64_vector_root;
    value.component_root = formula_probe_binary64_vector_root(value.components);
    value.radius_root = formula_probe_binary64_vector_root(value.radius);
    value.root = solution_expansion_root(value);
}

std::string profile_root(const FormulaProbeProfile& value) {
    std::ostringstream material;
    material << value.exact << ':' << value.contained << ':'
        << value.contractive << ':' << value.width << ':' << value.dimension
        << ':' << value.vector_containments << ':'
        << value.matrix_containments << ':' << value.nonzero_vector_lows
        << ':' << value.nonzero_matrix_lows << ':'
        << double_bits(value.rho_upper) << ':'
        << double_bits(value.maximum_vector_radius) << ':'
        << double_bits(value.maximum_matrix_radius) << ':' << value.exact_root
        << ':' << value.component_root << ':' << value.radius_root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

bool profile_valid(const FormulaProbeProfile& value) {
    using nextengine::nonlocal::fcr::formula_probe_binary64_vector_root;
    const std::string component_root = nextengine::nonlocal::sha256_hex(
        formula_probe_binary64_vector_root(value.vector_components) + ":"
        + formula_probe_binary64_vector_root(value.matrix_components));
    const std::string radius_root = nextengine::nonlocal::sha256_hex(
        formula_probe_binary64_vector_root(value.vector_radius) + ":"
        + formula_probe_binary64_vector_root(value.matrix_radius));
    bool finite = true;
    for (double component : value.vector_components)
        finite = finite && std::isfinite(component);
    for (double component : value.matrix_components)
        finite = finite && std::isfinite(component);
    return value.exact && value.contained && value.contractive
        && value.width == 2U && value.dimension == DIMENSION
        && value.vector_components.size() == DIMENSION * 2U
        && value.matrix_components.size() == DIMENSION * DIMENSION * 2U
        && value.vector_radius.size() == DIMENSION
        && value.matrix_radius.size() == DIMENSION * DIMENSION && finite
        && value.component_root == component_root
        && value.radius_root == radius_root && value.root == profile_root(value);
}

void reseal_profile(FormulaProbeProfile& value) {
    using nextengine::nonlocal::fcr::formula_probe_binary64_vector_root;
    value.component_root = nextengine::nonlocal::sha256_hex(
        formula_probe_binary64_vector_root(value.vector_components) + ":"
        + formula_probe_binary64_vector_root(value.matrix_components));
    value.root = profile_root(value);
}

struct ExactWork {
    std::size_t solution_expansions = 0U;
    std::size_t copied_solution_components = 0U;
    std::size_t copied_solution_radii = 0U;
    std::size_t vector_decodes = 0U;
    std::size_t matrix_decodes = 0U;
    std::size_t tangent_solution_decodes = 0U;
    std::size_t common_solution_decodes = 0U;
    std::size_t center_rows = 0U;
    std::size_t center_products = 0U;
    std::size_t center_containments = 0U;
    std::size_t solution_component_differences = 0U;
    std::size_t transport_rows = 0U;
    std::size_t transport_products = 0U;
    std::size_t transport_equalities = 0U;
    std::size_t maximum_row_scans = 0U;
    std::size_t maximum_row_contributions = 0U;
    std::size_t exact_additions = 0U;
    std::size_t exact_multiplications = 0U;
    std::size_t normalizations = 0U;
    std::size_t alignment_bits = 0U;
    std::size_t candidate_updates = 0U;
    std::size_t operator_updates = 0U;
    std::size_t solver_updates = 0U;
    std::size_t certificate_updates = 0U;
};

std::string exact_work_root(const ExactWork& work) {
    std::ostringstream material;
    material << work.solution_expansions << ':'
        << work.copied_solution_components << ':'
        << work.copied_solution_radii << ':' << work.vector_decodes << ':'
        << work.matrix_decodes << ':' << work.tangent_solution_decodes << ':'
        << work.common_solution_decodes << ':' << work.center_rows << ':'
        << work.center_products << ':' << work.center_containments << ':'
        << work.solution_component_differences << ':' << work.transport_rows
        << ':' << work.transport_products << ':' << work.transport_equalities
        << ':' << work.maximum_row_scans << ':'
        << work.maximum_row_contributions << ':' << work.exact_additions << ':'
        << work.exact_multiplications << ':' << work.normalizations << ':'
        << work.alignment_bits << ':' << work.candidate_updates << ':'
        << work.operator_updates << ':' << work.solver_updates << ':'
        << work.certificate_updates;
    return nextengine::nonlocal::sha256_hex(material.str());
}

bool exact_work_valid(const ExactWork& work) {
    return work.solution_expansions == 2U
        && work.copied_solution_components == 408U
        && work.copied_solution_radii == 204U
        && work.vector_decodes == 204U && work.matrix_decodes == 20808U
        && work.tangent_solution_decodes == 204U
        && work.common_solution_decodes == 204U
        && work.center_rows == 204U && work.center_products == 83232U
        && work.center_containments == 204U
        && work.solution_component_differences == 204U
        && work.transport_rows == 102U && work.transport_products == 41616U
        && work.transport_equalities == 102U
        && work.maximum_row_scans == 204U
        && work.maximum_row_contributions == 102U
        && work.candidate_updates == 0U && work.operator_updates == 0U
        && work.solver_updates == 0U && work.certificate_updates == 0U;
}

struct Dyadic {
    cpp_int integer = 0;
    int exponent = 0;
};

bool checked_exponent_add(int left, int right, int& result) {
    if ((right > 0 && left > INT_MAX - right)
        || (right < 0 && left < INT_MIN - right))
        return false;
    result = left + right;
    return true;
}

bool normalize(Dyadic& value, ExactWork* work) {
    if (work != nullptr) ++work->normalizations;
    if (value.integer == 0) {
        value.exponent = 0;
        return true;
    }
    while (value.integer % 2 == 0) {
        if (value.exponent == INT_MAX) return false;
        value.integer /= 2;
        ++value.exponent;
    }
    return true;
}

bool decode_binary64(double source, Dyadic& result, ExactWork* work) {
    const std::uint64_t bits = double_bits(source);
    const std::uint64_t exponent_bits = (bits >> 52U) & 0x7ffU;
    const std::uint64_t fraction = bits & ((UINT64_C(1) << 52U) - 1U);
    if (exponent_bits == 0x7ffU) return false;
    if (exponent_bits == 0U && fraction == 0U) {
        result = {};
        return normalize(result, work);
    }
    result.integer = exponent_bits == 0U
        ? cpp_int(fraction)
        : cpp_int((UINT64_C(1) << 52U) | fraction);
    if ((bits >> 63U) != 0U) result.integer = -result.integer;
    result.exponent = exponent_bits == 0U
        ? -1074 : static_cast<int>(exponent_bits) - 1023 - 52;
    return normalize(result, work);
}

bool add_dyadic(const Dyadic& left, const Dyadic& right, Dyadic& result,
    ExactWork* work) {
    const int exponent = std::min(left.exponent, right.exponent);
    const unsigned left_shift = static_cast<unsigned>(left.exponent - exponent);
    const unsigned right_shift =
        static_cast<unsigned>(right.exponent - exponent);
    result.integer = (left.integer << left_shift)
        + (right.integer << right_shift);
    result.exponent = exponent;
    if (work != nullptr) {
        ++work->exact_additions;
        work->alignment_bits += left_shift + right_shift;
    }
    return normalize(result, work);
}

bool negate_dyadic(const Dyadic& source, Dyadic& result) {
    result = source;
    result.integer = -result.integer;
    return true;
}

bool subtract_dyadic(const Dyadic& left, const Dyadic& right, Dyadic& result,
    ExactWork* work) {
    Dyadic negative;
    negate_dyadic(right, negative);
    return add_dyadic(left, negative, result, work);
}

bool multiply_dyadic(const Dyadic& left, const Dyadic& right, Dyadic& result,
    ExactWork* work) {
    result.integer = left.integer * right.integer;
    if (!checked_exponent_add(left.exponent, right.exponent, result.exponent))
        return false;
    if (work != nullptr) ++work->exact_multiplications;
    return normalize(result, work);
}

int compare_dyadic(const Dyadic& left, const Dyadic& right) {
    const int exponent = std::min(left.exponent, right.exponent);
    const cpp_int lhs = left.integer
        << static_cast<unsigned>(left.exponent - exponent);
    const cpp_int rhs = right.integer
        << static_cast<unsigned>(right.exponent - exponent);
    return lhs < rhs ? -1 : lhs > rhs ? 1 : 0;
}

int compare_absolute(const Dyadic& left, const Dyadic& right) {
    Dyadic lhs = left;
    Dyadic rhs = right;
    if (lhs.integer < 0) lhs.integer = -lhs.integer;
    if (rhs.integer < 0) rhs.integer = -rhs.integer;
    return compare_dyadic(lhs, rhs);
}

std::string dyadic_text(const Dyadic& value) {
    return value.integer.str() + "p" + std::to_string(value.exponent);
}

std::string dyadic_vector_root(const std::vector<Dyadic>& values) {
    std::ostringstream material;
    material << values.size() << '|';
    for (const Dyadic& value : values) material << dyadic_text(value) << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

struct CenterAudit {
    bool exact = false;
    bool complete = false;
    std::string failure;
    ExactWork work;
    std::array<std::vector<Dyadic>, 2U> vector_terms;
    std::array<std::vector<Dyadic>, 2U> product_terms;
    std::array<std::vector<Dyadic>, 2U> centers;
    std::array<std::vector<bool>, 2U> containments;
    std::vector<Dyadic> displacement;
    std::vector<Dyadic> delta_residual;
    std::vector<Dyadic> transport;
    std::vector<Dyadic> maximum_transport_contributions;
    std::size_t closed_transport_rows = 0U;
    std::size_t exact_maximum_row = 0U;
    std::size_t finite_maximum_row = 0U;
    Dyadic common_maximum_lower;
    Dyadic common_maximum_upper;
    std::string input_root;
    std::string center_root;
    std::string containment_root;
    std::string transport_root;
    std::string maximum_row_root;
    std::string root;
};

std::string center_audit_root(const CenterAudit& value) {
    std::ostringstream material;
    material << value.exact << ':' << value.complete << ':' << value.failure
        << ':' << value.input_root << ':' << exact_work_root(value.work) << ':'
        << value.center_root << ':' << value.containment_root << ':'
        << value.transport_root << ':' << value.exact_maximum_row << ':'
        << value.finite_maximum_row << ':' << value.closed_transport_rows
        << ':' << value.maximum_row_root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

enum class TransportFault { None, ReverseSign, TransposeMatrix };

CenterAudit run_center_audit(const FormulaProbeProfile& profile,
    const FormulaProbeSolutionExpansion& tangent,
    const FormulaProbeSolutionExpansion& common,
    const std::array<FormulaProbeCertificateDetail, 2U>& details,
    TransportFault transport_fault = TransportFault::None) {
    CenterAudit result;
    const auto fail = [&](const char* stage) {
        result.failure = stage;
        result.root = center_audit_root(result);
        return result;
    };
    if (!profile_valid(profile) || !solution_expansion_valid(tangent)
        || !solution_expansion_valid(common) || !detail_valid(details[0U])
        || !detail_valid(details[1U]))
        return fail("identity");

    result.work.solution_expansions = 2U;
    result.work.copied_solution_components =
        tangent.components.size() + common.components.size();
    result.work.copied_solution_radii =
        tangent.radius.size() + common.radius.size();
    result.input_root = nextengine::nonlocal::sha256_hex(profile.root + ":"
        + tangent.root + ":" + common.root + ":" + details[0U].root + ":"
        + details[1U].root);

    std::vector<Dyadic> vector(profile.vector_components.size());
    std::vector<Dyadic> matrix(profile.matrix_components.size());
    std::array<std::vector<Dyadic>, 2U> solution{
        std::vector<Dyadic>(tangent.components.size()),
        std::vector<Dyadic>(common.components.size())};
    for (std::size_t index = 0U; index < vector.size(); ++index) {
        if (!decode_binary64(profile.vector_components[index], vector[index],
                &result.work))
            return fail("vector_decode");
        ++result.work.vector_decodes;
    }
    for (std::size_t index = 0U; index < matrix.size(); ++index) {
        if (!decode_binary64(profile.matrix_components[index], matrix[index],
                &result.work))
            return fail("matrix_decode");
        ++result.work.matrix_decodes;
    }
    const std::array<const FormulaProbeSolutionExpansion*, 2U> expansions{
        &tangent, &common};
    for (std::size_t endpoint = 0U; endpoint < 2U; ++endpoint) {
        for (std::size_t index = 0U;
             index < expansions[endpoint]->components.size(); ++index) {
            if (!decode_binary64(expansions[endpoint]->components[index],
                    solution[endpoint][index], &result.work))
                return fail("solution_decode");
            if (endpoint == 0U) ++result.work.tangent_solution_decodes;
            else ++result.work.common_solution_decodes;
        }
    }

    std::array<std::vector<std::vector<Dyadic>>, 2U> contributions;
    for (std::size_t endpoint = 0U; endpoint < 2U; ++endpoint) {
        result.centers[endpoint].reserve(DIMENSION);
        result.vector_terms[endpoint].reserve(DIMENSION);
        result.product_terms[endpoint].reserve(DIMENSION);
        result.containments[endpoint].reserve(DIMENSION);
        contributions[endpoint].resize(DIMENSION);
        for (std::size_t row = 0U; row < DIMENSION; ++row) {
            Dyadic vector_sum;
            if (!add_dyadic(vector[row * 2U], vector[row * 2U + 1U],
                    vector_sum, &result.work))
                return fail("vector_sum");
            Dyadic product_sum;
            contributions[endpoint][row].reserve(DIMENSION);
            for (std::size_t column = 0U; column < DIMENSION; ++column) {
                Dyadic column_sum;
                for (std::size_t matrix_part = 0U; matrix_part < 2U;
                     ++matrix_part) {
                    for (std::size_t solution_part = 0U; solution_part < 2U;
                         ++solution_part) {
                        const std::size_t matrix_index =
                            (row * DIMENSION + column) * 2U + matrix_part;
                        const std::size_t solution_index =
                            column * 2U + solution_part;
                        Dyadic term;
                        if (!multiply_dyadic(matrix[matrix_index],
                                solution[endpoint][solution_index], term,
                                &result.work)
                            || !add_dyadic(column_sum, term, column_sum,
                                &result.work))
                            return fail("center_product");
                        ++result.work.center_products;
                    }
                }
                contributions[endpoint][row].push_back(column_sum);
                if (!add_dyadic(product_sum, column_sum, product_sum,
                        &result.work))
                    return fail("center_accumulation");
            }
            Dyadic center;
            if (!subtract_dyadic(vector_sum, product_sum, center,
                    &result.work))
                return fail("center_subtraction");
            result.vector_terms[endpoint].push_back(vector_sum);
            result.product_terms[endpoint].push_back(product_sum);
            result.centers[endpoint].push_back(center);
            ++result.work.center_rows;

            Dyadic image_center;
            Dyadic image_radius;
            Dyadic lower;
            Dyadic upper;
            if (!decode_binary64(details[endpoint].image_center[row],
                    image_center, &result.work)
                || !decode_binary64(details[endpoint].image_radius[row],
                    image_radius, &result.work)
                || !subtract_dyadic(image_center, image_radius, lower,
                    &result.work)
                || !add_dyadic(image_center, image_radius, upper,
                    &result.work))
                return fail("interval_decode");
            result.containments[endpoint].push_back(
                compare_dyadic(lower, center) <= 0
                && compare_dyadic(center, upper) <= 0);
            ++result.work.center_containments;
        }
    }

    result.exact_maximum_row = 0U;
    result.finite_maximum_row = 0U;
    for (std::size_t row = 0U; row < DIMENSION; ++row) {
        if (row != 0U && compare_absolute(result.centers[1U][row],
                result.centers[1U][result.exact_maximum_row]) > 0)
            result.exact_maximum_row = row;
        ++result.work.maximum_row_scans;
    }
    for (std::size_t row = 0U; row < DIMENSION; ++row) {
        if (row != 0U
            && std::abs(details[1U].image_center[row])
                > std::abs(details[1U].image_center[
                    result.finite_maximum_row]))
            result.finite_maximum_row = row;
        ++result.work.maximum_row_scans;
    }

    result.displacement.resize(DIMENSION * 2U);
    for (std::size_t index = 0U; index < result.displacement.size(); ++index) {
        if (!subtract_dyadic(solution[1U][index], solution[0U][index],
                result.displacement[index], &result.work))
            return fail("solution_displacement");
        ++result.work.solution_component_differences;
    }
    result.delta_residual.reserve(DIMENSION);
    result.transport.reserve(DIMENSION);
    for (std::size_t row = 0U; row < DIMENSION; ++row) {
        Dyadic delta;
        if (!subtract_dyadic(result.centers[1U][row],
                result.centers[0U][row], delta, &result.work))
            return fail("residual_displacement");
        result.delta_residual.push_back(delta);
        Dyadic transported;
        for (std::size_t column = 0U; column < DIMENSION; ++column) {
            Dyadic column_transport;
            for (std::size_t matrix_part = 0U; matrix_part < 2U;
                 ++matrix_part) {
                for (std::size_t solution_part = 0U; solution_part < 2U;
                     ++solution_part) {
                    const std::size_t source_row =
                        transport_fault == TransportFault::TransposeMatrix
                        ? column : row;
                    const std::size_t source_column =
                        transport_fault == TransportFault::TransposeMatrix
                        ? row : column;
                    const std::size_t matrix_index =
                        (source_row * DIMENSION + source_column) * 2U
                        + matrix_part;
                    const std::size_t solution_index =
                        column * 2U + solution_part;
                    Dyadic term;
                    if (!multiply_dyadic(matrix[matrix_index],
                            result.displacement[solution_index], term,
                            &result.work)
                        || !add_dyadic(column_transport, term,
                            column_transport,
                            &result.work))
                        return fail("transport_product");
                    ++result.work.transport_products;
                }
            }
            if (transport_fault != TransportFault::ReverseSign)
                column_transport.integer = -column_transport.integer;
            if (!add_dyadic(transported, column_transport, transported,
                    &result.work))
                return fail("transport_accumulation");
            if (row == result.exact_maximum_row)
                result.maximum_transport_contributions.push_back(
                    column_transport);
        }
        result.transport.push_back(transported);
        ++result.work.transport_rows;
        ++result.work.transport_equalities;
        if (compare_dyadic(delta, transported) == 0)
            ++result.closed_transport_rows;
    }

    Dyadic common_image_center;
    Dyadic common_image_radius;
    const std::size_t maximum = result.exact_maximum_row;
    if (!decode_binary64(details[1U].image_center[maximum],
            common_image_center, &result.work)
        || !decode_binary64(details[1U].image_radius[maximum],
            common_image_radius, &result.work)
        || !subtract_dyadic(common_image_center, common_image_radius,
            result.common_maximum_lower, &result.work)
        || !add_dyadic(common_image_center, common_image_radius,
            result.common_maximum_upper, &result.work))
        return fail("maximum_interval");
    std::ostringstream contribution_material;
    if (result.maximum_transport_contributions.size() != DIMENSION)
        return fail("maximum_contributions");
    for (std::size_t column = 0U; column < DIMENSION; ++column) {
        contribution_material
            << dyadic_text(contributions[0U][maximum][column]) << ':'
            << dyadic_text(contributions[1U][maximum][column]) << ':'
            << dyadic_text(result.maximum_transport_contributions[column])
            << ';';
        ++result.work.maximum_row_contributions;
    }
    const std::string contribution_root = nextengine::nonlocal::sha256_hex(
        contribution_material.str());
    result.center_root = nextengine::nonlocal::sha256_hex(
        dyadic_vector_root(result.centers[0U]) + ":"
        + dyadic_vector_root(result.centers[1U]));
    std::ostringstream containment_material;
    for (const auto& endpoint : result.containments)
        for (bool contained : endpoint)
            containment_material << contained;
    result.containment_root = nextengine::nonlocal::sha256_hex(
        containment_material.str());
    result.transport_root = nextengine::nonlocal::sha256_hex(
        dyadic_vector_root(result.displacement) + ":"
        + dyadic_vector_root(result.delta_residual) + ":"
        + dyadic_vector_root(result.transport));
    result.maximum_row_root = nextengine::nonlocal::sha256_hex(
        std::to_string(maximum) + ":"
        + dyadic_text(result.vector_terms[1U][maximum]) + ":"
        + dyadic_text(result.product_terms[0U][maximum]) + ":"
        + dyadic_text(result.product_terms[1U][maximum]) + ":"
        + dyadic_text(result.centers[0U][maximum]) + ":"
        + dyadic_text(result.centers[1U][maximum]) + ":"
        + dyadic_text(result.delta_residual[maximum]) + ":"
        + dyadic_text(result.transport[maximum]) + ":"
        + dyadic_text(result.common_maximum_lower) + ":"
        + dyadic_text(result.common_maximum_upper) + ":" + contribution_root);
    result.complete = true;
    result.exact = true;
    result.root = center_audit_root(result);
    return result;
}

bool all_contained(const CenterAudit& value) {
    if (!value.exact || !value.complete) return false;
    for (const auto& endpoint : value.containments)
        if (!std::all_of(endpoint.begin(), endpoint.end(),
                [](bool contained) { return contained; }))
            return false;
    return true;
}

bool transport_closed(const CenterAudit& value) {
    return value.exact && value.transport.size() == DIMENSION
        && value.delta_residual.size() == DIMENSION
        && value.closed_transport_rows == DIMENSION;
}

bool common_maximum_excludes_zero(const CenterAudit& value) {
    Dyadic zero;
    return value.exact
        && (compare_dyadic(value.common_maximum_lower, zero) > 0
            || compare_dyadic(value.common_maximum_upper, zero) < 0);
}

std::string classify_r63zh(bool apparatus, bool identity, bool work,
    bool contained, bool maximum_row, bool transport, bool common_nonzero,
    bool cells) {
    if (!apparatus) return "CENTER_PRODUCER_APPARATUS_REJECTED";
    if (!identity) return "CENTER_PRODUCER_IDENTITY_REJECTED";
    if (!work) return "CENTER_PRODUCER_WORK_REJECTED";
    if (!contained) return "FINITE_CENTER_ARITHMETIC_DEFECT";
    if (!maximum_row) return "CENTER_PRODUCER_MAXIMUM_ROW_REJECTED";
    if (!transport) return "CENTER_PRODUCER_TRANSPORT_REJECTED";
    if (common_nonzero && cells)
        return "COMMON_SOLUTION_OUTSIDE_VERIFIER_AFFINE_MODEL";
    return "CENTER_PRODUCER_FURTHER_FACTORIAL_REQUIRED";
}

bool classifier_r63zh_control() {
    bool exact = true;
    for (unsigned mask = 0U; mask < 256U; ++mask) {
        const bool apparatus = (mask & 1U) != 0U;
        const bool identity = (mask & 2U) != 0U;
        const bool work = (mask & 4U) != 0U;
        const bool contained = (mask & 8U) != 0U;
        const bool maximum = (mask & 16U) != 0U;
        const bool transport = (mask & 32U) != 0U;
        const bool nonzero = (mask & 64U) != 0U;
        const bool cells = (mask & 128U) != 0U;
        const char* expected = !apparatus
            ? "CENTER_PRODUCER_APPARATUS_REJECTED"
            : !identity ? "CENTER_PRODUCER_IDENTITY_REJECTED"
            : !work ? "CENTER_PRODUCER_WORK_REJECTED"
            : !contained ? "FINITE_CENTER_ARITHMETIC_DEFECT"
            : !maximum ? "CENTER_PRODUCER_MAXIMUM_ROW_REJECTED"
            : !transport ? "CENTER_PRODUCER_TRANSPORT_REJECTED"
            : nonzero && cells
                ? "COMMON_SOLUTION_OUTSIDE_VERIFIER_AFFINE_MODEL"
                : "CENTER_PRODUCER_FURTHER_FACTORIAL_REQUIRED";
        exact = exact && classify_r63zh(apparatus, identity, work, contained,
            maximum, transport, nonzero, cells) == expected;
    }
    return exact;
}

struct ParentPrefix {
    bool exact = false;
    Primary primary;
    DetailAudit detail;
    std::string observations_root;
    std::string transaction_root;
    std::string result_root;
    std::string root;
};

ParentPrefix run_parent_prefix(
    const nextengine::nonlocal::fcr::FormulaProbeParentFixture& fixture) {
    ParentPrefix result;
    result.primary = run_primary(fixture, fixture.original_rhs);
    const PrimaryAudit parent_audit = audit_primary(result.primary);
    result.detail = run_detail_audit(fixture, result.primary);
    const Work parent_expected{2U, 4U, 3U, 30906U, 612U, 5U, 4U,
        306U, 306U, 102U, 945U, 306U, 96390U, 96390U, 306U,
        102U, 10404U, 0U};
    const AuditWork parent_audit_expected{1U, 102U, 102U, 102U, 2U, 204U};
    const DetailWork detail_expected{2U, 204U, 83640U, 62424U, 204U,
        204U, 204U, 408U, 612U, 8U, 18U, 1836U, 0U, 0U, 0U};
    const std::string reconstructed_r63zf = result_identity(
        "CERTIFICATE_ENCLOSURE_AMPLIFICATION_CANDIDATE", fixture.root,
        result.primary.root, work_root(result.primary.work),
        audit_work_root(result.primary.audit_work), R63ZF_CONTROLS_ROOT);
    const bool parent = parent_audit.apparatus && parent_audit.endpoint
        && parent_audit.quadratic && parent_audit.displacement
        && parent_audit.no_raw_zero && result.primary.root == R63ZF_PRIMARY_ROOT
        && work_equal(result.primary.work, parent_expected)
        && audit_work_equal(result.primary.audit_work, parent_audit_expected)
        && detail_work_equal(result.detail.work, detail_expected)
        && detail_work_root(result.detail.work) == R63ZG_DETAIL_WORK_SHA256
        && classifier_r63zf_control()
        && reconstructed_r63zf == R63ZF_RESULT_SHA256
        && classifier_r63zg_control()
        && std::string(mode_name(BudgetMode::CenterGlobal)) == "GC";
    bool budget_independent = false;
    bool combined_sufficient = false;
    bool center_sufficient = false;
    bool radius_sufficient = false;
    bool contraction_boundary_changed = false;
    bool cells_exact = result.detail.exact && result.detail.complete
        && result.detail.cells.size() == 18U;
    if (cells_exact) {
        const SignCell& tt = result.detail.cells[cell_index(
            0U, 0U, BudgetMode::NativeGlobal)];
        const SignCell& ct = result.detail.cells[cell_index(
            1U, 0U, BudgetMode::NativeGlobal)];
        const SignCell& tc = result.detail.cells[cell_index(
            0U, 1U, BudgetMode::NativeGlobal)];
        const SignCell& cc = result.detail.cells[cell_index(
            1U, 1U, BudgetMode::NativeGlobal)];
        budget_independent = tt.passed && ct.passed
            && !tc.passed && !cc.passed;
        combined_sufficient = !tc.passed && !cc.passed;
        const SignCell& tangent_center = result.detail.cells[cell_index(
            0U, 1U, BudgetMode::CenterGlobal)];
        const SignCell& common_center = result.detail.cells[cell_index(
            1U, 1U, BudgetMode::CenterGlobal)];
        const SignCell& tangent_radius = result.detail.cells[cell_index(
            0U, 1U, BudgetMode::RadiusGlobal)];
        const SignCell& common_radius = result.detail.cells[cell_index(
            1U, 1U, BudgetMode::RadiusGlobal)];
        center_sufficient = !tangent_center.passed && !common_center.passed;
        radius_sufficient = !tangent_radius.passed && !common_radius.passed;
        for (unsigned solution = 0U; solution < 2U; ++solution)
            for (unsigned origin = 0U; origin < 2U; ++origin) {
                const SignCell& unamplified = result.detail.cells[cell_index(
                    solution, origin, BudgetMode::Unamplified)];
                const SignCell& amplified = result.detail.cells[cell_index(
                    solution, origin, BudgetMode::NativeGlobal)];
                contraction_boundary_changed = contraction_boundary_changed
                    || unamplified.passed != amplified.passed
                    || unamplified.positive != amplified.positive
                    || unamplified.negative != amplified.negative
                    || unamplified.unresolved != amplified.unresolved;
            }
    }
    std::ostringstream observation_material;
    observation_material << budget_independent << ':' << combined_sufficient
        << ':' << center_sufficient << ':' << radius_sufficient << ':'
        << contraction_boundary_changed;
    result.observations_root = nextengine::nonlocal::sha256_hex(
        observation_material.str());
    result.transaction_root = nextengine::nonlocal::sha256_hex(
        result.detail.root + ":" + result.observations_root);
    result.result_root = r63zg_result_identity(
        "AFFINE_IMAGE_CENTER_RESIDUAL_SUFFICIENT", fixture.root,
        result.transaction_root, work_root(result.primary.work),
        detail_work_root(result.detail.work), R63ZG_CONTROLS_SHA256);
    result.exact = parent && cells_exact && budget_independent
        && combined_sufficient && center_sufficient && !radius_sufficient
        && !contraction_boundary_changed
        && result.observations_root == R63ZG_OBSERVATIONS_SHA256
        && result.transaction_root == R63ZG_TRANSACTION_SHA256
        && result.result_root == R63ZG_RESULT_SHA256;
    result.root = nextengine::nonlocal::sha256_hex(
        std::to_string(result.exact) + ":" + result.primary.root + ":"
        + result.detail.root + ":" + result.observations_root + ":"
        + result.transaction_root + ":" + result.result_root + ":"
        + R63ZG_STDOUT_SHA256);
    return result;
}

bool common_cells_reproduced(const ParentPrefix& parent) {
    if (!parent.exact || parent.detail.cells.size() != 18U) return false;
    const SignCell& tangent_center = parent.detail.cells[cell_index(
        0U, 1U, BudgetMode::CenterGlobal)];
    const SignCell& common_center = parent.detail.cells[cell_index(
        1U, 1U, BudgetMode::CenterGlobal)];
    const SignCell& tangent_radius = parent.detail.cells[cell_index(
        0U, 1U, BudgetMode::RadiusGlobal)];
    const SignCell& common_radius = parent.detail.cells[cell_index(
        1U, 1U, BudgetMode::RadiusGlobal)];
    return !tangent_center.passed && !common_center.passed
        && tangent_radius.passed && common_radius.passed;
}

std::string r63zh_result_identity(const std::string& route,
    const std::string& fixture_root, const std::string& parent_root,
    const std::string& audit_root, const std::string& work_root_value,
    const std::string& controls_root) {
    return nextengine::nonlocal::sha256_hex(route + ":" + fixture_root + ":"
        + R63ZG_RESULT_SHA256 + ":" + R63ZG_STDOUT_SHA256 + ":"
        + parent_root + ":" + audit_root + ":" + work_root_value + ":"
        + controls_root);
}

std::string route_for_audit(const CenterAudit& audit, bool identity,
    bool controls, bool cells) {
    return classify_r63zh(audit.exact && audit.complete && controls, identity,
        exact_work_valid(audit.work), all_contained(audit),
        audit.exact_maximum_row == audit.finite_maximum_row,
        transport_closed(audit), common_maximum_excludes_zero(audit), cells);
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-center-producer "
                         "<parent-fixture-cache>\n";
            return 2;
        }
        using namespace nextengine::nonlocal::fcr;
        const FormulaProbeParentFixture fixture =
            read_formula_probe_parent_fixture(argv[1]);
        const bool fixture_identity = formula_probe_parent_fixture_valid(fixture);
        const ParentPrefix parent = run_parent_prefix(fixture);
        if (!fixture_identity || !parent.exact
            || parent.detail.details.size() != 2U) {
            std::cerr << "nonlocal-formula-center-producer: parent rejected\n";
            return 1;
        }

        const FormulaProbeSolutionExpansion tangent =
            formula_probe_solution_expansion(fixture, parent.primary.x_t);
        const FormulaProbeSolutionExpansion common =
            formula_probe_solution_expansion(fixture, parent.primary.x_c);
        const std::array<FormulaProbeCertificateDetail, 2U> details{
            parent.detail.details[0U], parent.detail.details[1U]};
        FormulaProbeCertificateDetail refreshed_tangent = details[0U];
        const bool detail_refresh_identity =
            refresh_detail_native_scalars(refreshed_tangent)
            && detail_valid(refreshed_tangent)
            && refreshed_tangent.root == details[0U].root;
        const bool expansion_identity = solution_expansion_valid(tangent)
            && solution_expansion_valid(common)
            && detail_refresh_identity
            && tangent.source_solution_root == TANGENT_STATE2_SOLUTION_ROOT
            && common.source_solution_root == COMMON_STATE2_SOLUTION_ROOT
            && internal_solution_root(tangent)
                == details[0U].certificate.solution_root
            && internal_solution_root(common)
                == details[1U].certificate.solution_root;
        const CenterAudit audit = run_center_audit(
            fixture.verifier_profile, tangent, common, details);

        const bool classifier = classifier_r63zh_control();
        FormulaProbeProfile vector_mutation = fixture.verifier_profile;
        vector_mutation.vector_components[0U] = std::nextafter(
            vector_mutation.vector_components[0U],
            std::numeric_limits<double>::infinity());
        reseal_profile(vector_mutation);
        const CenterAudit vector_changed = run_center_audit(
            vector_mutation, tangent, common, details);
        FormulaProbeProfile matrix_mutation = fixture.verifier_profile;
        matrix_mutation.matrix_components[0U] = std::nextafter(
            matrix_mutation.matrix_components[0U],
            std::numeric_limits<double>::infinity());
        reseal_profile(matrix_mutation);
        const CenterAudit matrix_changed = run_center_audit(
            matrix_mutation, tangent, common, details);
        FormulaProbeSolutionExpansion tangent_mutation = tangent;
        tangent_mutation.components[0U] = std::nextafter(
            tangent_mutation.components[0U],
            std::numeric_limits<double>::infinity());
        reseal_solution_expansion(tangent_mutation);
        const CenterAudit tangent_changed = run_center_audit(
            fixture.verifier_profile, tangent_mutation, common, details);
        FormulaProbeSolutionExpansion common_mutation = common;
        common_mutation.components[0U] = std::nextafter(
            common_mutation.components[0U],
            std::numeric_limits<double>::infinity());
        reseal_solution_expansion(common_mutation);
        const CenterAudit common_changed = run_center_audit(
            fixture.verifier_profile, tangent, common_mutation, details);
        const bool mutation_controls = vector_changed.exact
            && matrix_changed.exact && tangent_changed.exact
            && common_changed.exact && vector_changed.root != audit.root
            && matrix_changed.root != audit.root
            && tangent_changed.root != audit.root
            && common_changed.root != audit.root;

        FormulaProbeSolutionExpansion stale = common;
        std::swap(stale.components[0U], stale.components[1U]);
        const CenterAudit stale_audit = run_center_audit(
            fixture.verifier_profile, tangent, stale, details);
        const CenterAudit reverse = run_center_audit(fixture.verifier_profile,
            tangent, common, details, TransportFault::ReverseSign);
        const CenterAudit transpose = run_center_audit(fixture.verifier_profile,
            tangent, common, details, TransportFault::TransposeMatrix);
        const bool transport_controls = transport_closed(audit)
            && !transport_closed(reverse) && !transport_closed(transpose)
            && route_for_audit(reverse, true, true, true)
                == "CENTER_PRODUCER_TRANSPORT_REJECTED"
            && route_for_audit(transpose, true, true, true)
                == "CENTER_PRODUCER_TRANSPORT_REJECTED";

        std::array<FormulaProbeCertificateDetail, 2U> shrunk = details;
        shrunk[1U].image_radius[audit.exact_maximum_row] = std::nextafter(
            shrunk[1U].image_radius[audit.exact_maximum_row], 0.0);
        const CenterAudit shrunk_audit = run_center_audit(
            fixture.verifier_profile, tangent, common, shrunk);
        Dyadic synthetic_point;
        Dyadic one;
        Dyadic synthetic_lower;
        Dyadic synthetic_upper;
        const bool synthetic_exact = decode_binary64(1.0, one, nullptr)
            && add_dyadic(audit.centers[1U][audit.exact_maximum_row], one,
                synthetic_point, nullptr)
            && (synthetic_lower = synthetic_point, true)
            && (synthetic_upper = synthetic_point, true)
            && !(compare_dyadic(synthetic_lower,
                    audit.centers[1U][audit.exact_maximum_row]) <= 0
                && compare_dyadic(audit.centers[1U][audit.exact_maximum_row],
                    synthetic_upper) <= 0)
            && classify_r63zh(true, true, true, false, true, true, true, true)
                == "FINITE_CENTER_ARITHMETIC_DEFECT";
        const bool interval_controls = !shrunk_audit.exact
            && shrunk_audit.failure == "identity"
            && shrunk_audit.work.center_rows == 0U && synthetic_exact;

        FormulaProbeSolutionExpansion invalid_dimension = tangent;
        --invalid_dimension.dimension;
        FormulaProbeSolutionExpansion nonfinite = tangent;
        nonfinite.components[0U] = std::numeric_limits<double>::infinity();
        reseal_solution_expansion(nonfinite);
        FormulaProbeSolutionExpansion incomplete = tangent;
        incomplete.components.pop_back();
        reseal_solution_expansion(incomplete);
        const CenterAudit invalid_dimension_audit = run_center_audit(
            fixture.verifier_profile, invalid_dimension, common, details);
        const CenterAudit nonfinite_audit = run_center_audit(
            fixture.verifier_profile, nonfinite, common, details);
        const CenterAudit incomplete_audit = run_center_audit(
            fixture.verifier_profile, incomplete, common, details);
        Dyadic invalid_decode;
        int overflow_exponent = 0;
        const bool failure_controls = !invalid_dimension_audit.exact
            && !nonfinite_audit.exact && !incomplete_audit.exact
            && invalid_dimension_audit.work.center_rows == 0U
            && nonfinite_audit.work.center_rows == 0U
            && incomplete_audit.work.center_rows == 0U
            && !decode_binary64(std::numeric_limits<double>::infinity(),
                invalid_decode, nullptr)
            && !checked_exponent_add(INT_MAX, 1, overflow_exponent);

        FormulaProbeParentFixture corrupt_fixture = fixture;
        corrupt_fixture.original_rhs_root = nextengine::nonlocal::sha256_hex(
            corrupt_fixture.original_rhs_root + ":corrupt");
        const FormulaProbeSolutionExpansion corrupt_expansion =
            formula_probe_solution_expansion(
                corrupt_fixture, parent.primary.x_t);
        const bool corrupt_fixture_control =
            !formula_probe_parent_fixture_valid(corrupt_fixture)
            && !corrupt_expansion.exact && corrupt_expansion.root.empty()
            && corrupt_expansion.components.empty()
            && corrupt_expansion.radius.empty();

        const bool parent_cells = common_cells_reproduced(parent);
        const bool local_controls = classifier && mutation_controls
            && !stale_audit.exact && stale_audit.failure == "identity"
            && transport_controls && interval_controls && failure_controls
            && corrupt_fixture_control;
        const std::string provisional_route = route_for_audit(
            audit, expansion_identity, local_controls, parent_cells);
        std::ostringstream preliminary_material;
        preliminary_material << classifier << ':' << mutation_controls << ':'
            << vector_changed.root << ':' << matrix_changed.root << ':'
            << tangent_changed.root << ':' << common_changed.root << ':'
            << stale_audit.root << ':' << transport_controls << ':'
            << reverse.root << ':' << transpose.root << ':'
            << interval_controls << ':' << shrunk_audit.root << ':'
            << synthetic_exact << ':' << failure_controls << ':'
            << invalid_dimension_audit.root << ':' << nonfinite_audit.root
            << ':' << incomplete_audit.root << ':' << corrupt_fixture_control;
        const std::string preliminary_controls_root =
            nextengine::nonlocal::sha256_hex(preliminary_material.str());
        const std::string base_result = r63zh_result_identity(
            provisional_route, fixture.root, parent.root, audit.root,
            exact_work_root(audit.work), preliminary_controls_root);
        const std::string vector_mutation_result = r63zh_result_identity(
            provisional_route, fixture.root, parent.root, vector_changed.root,
            exact_work_root(vector_changed.work), preliminary_controls_root);
        const std::string matrix_mutation_result = r63zh_result_identity(
            provisional_route, fixture.root, parent.root, matrix_changed.root,
            exact_work_root(matrix_changed.work), preliminary_controls_root);
        const std::string tangent_mutation_result = r63zh_result_identity(
            provisional_route, fixture.root, parent.root, tangent_changed.root,
            exact_work_root(tangent_changed.work), preliminary_controls_root);
        const std::string common_mutation_result = r63zh_result_identity(
            provisional_route, fixture.root, parent.root, common_changed.root,
            exact_work_root(common_changed.work), preliminary_controls_root);
        const bool result_sealing = base_result != vector_mutation_result
            && base_result != matrix_mutation_result
            && base_result != tangent_mutation_result
            && base_result != common_mutation_result;
        preliminary_material << ':' << result_sealing << ':' << base_result
            << ':' << vector_mutation_result << ':' << matrix_mutation_result
            << ':' << tangent_mutation_result << ':'
            << common_mutation_result;
        const std::string controls_root = nextengine::nonlocal::sha256_hex(
            preliminary_material.str());
        const bool controls = local_controls && result_sealing;
        const std::string route = route_for_audit(
            audit, expansion_identity, controls, parent_cells);
        const std::string result_root = r63zh_result_identity(route,
            fixture.root, parent.root, audit.root, exact_work_root(audit.work),
            controls_root);
        const bool exact = parent.exact && expansion_identity && controls
            && audit.exact && audit.complete && exact_work_valid(audit.work)
            && (route == "FINITE_CENTER_ARITHMETIC_DEFECT"
                || route == "COMMON_SOLUTION_OUTSIDE_VERIFIER_AFFINE_MODEL"
                || route == "CENTER_PRODUCER_FURTHER_FACTORIAL_REQUIRED");

        std::size_t contained_count = 0U;
        for (const auto& endpoint : audit.containments)
            contained_count += static_cast<std::size_t>(std::count(
                endpoint.begin(), endpoint.end(), true));
        std::cout
            << "{\"schema\":\"nextengine.nonlocal.r63zh_center_producer.v1\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"route\":\"" << route << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"parent\":{\"result_sha256\":\"" << R63ZG_RESULT_SHA256
            << "\",\"stdout_sha256\":\"" << R63ZG_STDOUT_SHA256
            << "\",\"prefix_root\":\"" << parent.root << "\"}"
            << ",\"expansions\":[{\"endpoint\":\"tangent\",\"source_root\":\""
            << tangent.source_solution_root << "\",\"component_root\":\""
            << tangent.component_root << "\",\"radius_root\":\""
            << tangent.radius_root << "\",\"root\":\"" << tangent.root
            << "\"},{\"endpoint\":\"common\",\"source_root\":\""
            << common.source_solution_root << "\",\"component_root\":\""
            << common.component_root << "\",\"radius_root\":\""
            << common.radius_root << "\",\"root\":\"" << common.root
            << "\"}]"
            << ",\"centers\":{\"contained\":" << contained_count
            << ",\"total\":204,\"exact_maximum_row\":"
            << audit.exact_maximum_row << ",\"finite_maximum_row\":"
            << audit.finite_maximum_row << ",\"common_maximum_center\":\""
            << dyadic_text(audit.centers[1U][audit.exact_maximum_row])
            << "\",\"common_maximum_lower\":\""
            << dyadic_text(audit.common_maximum_lower)
            << "\",\"common_maximum_upper\":\""
            << dyadic_text(audit.common_maximum_upper)
            << "\",\"root\":\"" << audit.center_root
            << "\",\"containment_root\":\"" << audit.containment_root
            << "\",\"maximum_row_root\":\"" << audit.maximum_row_root
            << "\"}"
            << ",\"transport\":{\"equalities\":"
            << audit.closed_transport_rows
            << ",\"total\":102,\"root\":\"" << audit.transport_root << "\"}"
            << ",\"controls\":{\"classifier\":"
            << (classifier ? "true" : "false")
            << ",\"mutations\":" << (mutation_controls ? "true" : "false")
            << ",\"stale_swap\":" << (!stale_audit.exact ? "true" : "false")
            << ",\"transport_faults\":"
            << (transport_controls ? "true" : "false")
            << ",\"intervals\":" << (interval_controls ? "true" : "false")
            << ",\"failures\":" << (failure_controls ? "true" : "false")
            << ",\"corrupt_fixture\":"
            << (corrupt_fixture_control ? "true" : "false")
            << ",\"result_sealing\":" << (result_sealing ? "true" : "false")
            << ",\"root\":\"" << controls_root << "\"}"
            << ",\"work\":{\"root\":\"" << exact_work_root(audit.work)
            << "\",\"solution_expansions\":" << audit.work.solution_expansions
            << ",\"copied_solution_components\":"
            << audit.work.copied_solution_components
            << ",\"copied_solution_radii\":"
            << audit.work.copied_solution_radii
            << ",\"vector_decodes\":" << audit.work.vector_decodes
            << ",\"matrix_decodes\":" << audit.work.matrix_decodes
            << ",\"tangent_solution_decodes\":"
            << audit.work.tangent_solution_decodes
            << ",\"common_solution_decodes\":"
            << audit.work.common_solution_decodes
            << ",\"center_rows\":" << audit.work.center_rows
            << ",\"center_products\":" << audit.work.center_products
            << ",\"center_containments\":" << audit.work.center_containments
            << ",\"solution_component_differences\":"
            << audit.work.solution_component_differences
            << ",\"transport_rows\":" << audit.work.transport_rows
            << ",\"transport_products\":" << audit.work.transport_products
            << ",\"transport_equalities\":"
            << audit.work.transport_equalities
            << ",\"maximum_row_scans\":" << audit.work.maximum_row_scans
            << ",\"maximum_row_contributions\":"
            << audit.work.maximum_row_contributions
            << ",\"exact_additions\":" << audit.work.exact_additions
            << ",\"exact_multiplications\":"
            << audit.work.exact_multiplications
            << ",\"normalizations\":" << audit.work.normalizations
            << ",\"alignment_bits\":" << audit.work.alignment_bits
            << ",\"candidate_updates\":0,\"operator_updates\":0"
            << ",\"solver_updates\":0,\"certificate_updates\":0}"
            << ",\"audit_root\":\"" << audit.root << "\""
            << ",\"timing_admitted\":false"
            << ",\"runtime_authority\":false"
            << ",\"gpu_authority\":false"
            << ",\"production_authority\":false"
            << ",\"result_sha256\":\"" << result_root << "\"}\n";
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-center-producer: "
                  << error.what() << '\n';
        return 2;
    }
}
