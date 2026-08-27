#define NEXTENGINE_R63ZF_EMBEDDED
#include "formula_quadratic_step_sign_main.cpp"
#undef NEXTENGINE_R63ZF_EMBEDDED

#include <cmath>
#include <cstdint>
#include <cstring>
#include <limits>

namespace {

using nextengine::nonlocal::fcr::FormulaProbeCertificateDetail;

constexpr const char* R63ZF_RESULT_SHA256 =
    "0770e6cf6339163088e4980482d74675360cd1267d4db7780a5ee02638ba38dd";
constexpr const char* R63ZF_STDOUT_SHA256 =
    "be74e412ec6d708128506eeef5cd7284a56a2f150de8e1decb5be12ca394de0e";
constexpr const char* R63ZF_PRIMARY_ROOT =
    "b7151b3f6f94aa25dbc93a5e9be33d9ae8b49a34ca78bf15314ba373b5b3a01e";
constexpr const char* R63ZF_CONTROLS_ROOT =
    "e211b307fd35b6156e2a2f1655710a8bd25e022847e306dd772646dace4cd22c";
constexpr const char* R63Y_PROFILE_ROOT =
    "b1c430444e04e23d282874ae08130ea27d5eedb951d551151cc09423ca90a291";
constexpr std::uint64_t R63Y_RHO_BITS = 0x3f828ed8e0fe9914ULL;
constexpr std::uint64_t R63Y_DENOMINATOR_BITS = 0x3fefb5c49c7c059bULL;

std::uint64_t double_bits(double value) {
    std::uint64_t bits = 0U;
    static_assert(sizeof(bits) == sizeof(value));
    std::memcpy(&bits, &value, sizeof(bits));
    return bits;
}

bool normal64_or_zero(double value) {
    return std::isfinite(value) && std::fpclassify(value) != FP_SUBNORMAL;
}

double up_add64(double left, double right) {
    if (left == 0.0 && right == 0.0) return 0.0;
    const double value = left + right;
    return std::isfinite(value)
        ? std::nextafter(value, std::numeric_limits<double>::infinity())
        : value;
}

double up_divide64(double numerator, double denominator) {
    if (numerator == 0.0 && denominator > 0.0) return 0.0;
    const double value = numerator / denominator;
    return std::isfinite(value)
        ? std::nextafter(value, std::numeric_limits<double>::infinity())
        : value;
}

std::string detail_component_root(const FormulaProbeCertificateDetail& value) {
    using nextengine::nonlocal::fcr::formula_probe_binary64_vector_root;
    return nextengine::nonlocal::sha256_hex(
        formula_probe_binary64_vector_root(value.image_center) + ":"
        + formula_probe_binary64_vector_root(value.solution_center));
}

std::string detail_radius_root(const FormulaProbeCertificateDetail& value) {
    using nextengine::nonlocal::fcr::formula_probe_binary64_vector_root;
    return nextengine::nonlocal::sha256_hex(
        formula_probe_binary64_vector_root(value.image_radius) + ":"
        + formula_probe_binary64_vector_root(value.solution_local_radius));
}

std::string detail_root(const FormulaProbeCertificateDetail& value) {
    std::ostringstream material;
    material << value.exact << ':' << value.width << ':' << value.dimension
        << ':' << value.dots << ':' << value.dot_products << ':'
        << value.radius_terms << ':' << value.solution_dots << ':'
        << value.sign_comparisons << ':' << value.detail_solution_dots << ':'
        << value.detail_solution_products << ':' << double_bits(value.rho_upper)
        << ':' << double_bits(value.image_infinity_upper) << ':'
        << double_bits(value.denominator_lower) << ':'
        << double_bits(value.error_upper) << ':'
        << double_bits(value.minimum_separation) << ':'
        << value.certificate.root << ':' << value.profile_root << ':'
        << value.component_root << ':' << value.radius_root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

void reseal_detail(FormulaProbeCertificateDetail& value) {
    value.component_root = detail_component_root(value);
    value.radius_root = detail_radius_root(value);
    value.root = detail_root(value);
}

bool refresh_detail_native_scalars(FormulaProbeCertificateDetail& value) {
    if (value.image_center.size() != DIMENSION
        || value.image_radius.size() != DIMENSION)
        return false;
    double image_infinity = 0.0;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        const double combined = up_add64(
            std::abs(value.image_center[index]), value.image_radius[index]);
        if (!normal64_or_zero(combined)) return false;
        image_infinity = std::max(image_infinity, combined);
    }
    const double denominator = std::nextafter(1.0 - value.rho_upper,
        -std::numeric_limits<double>::infinity());
    const double error = up_divide64(image_infinity, denominator);
    if (!normal64_or_zero(denominator) || denominator <= 0.0
        || !normal64_or_zero(error))
        return false;
    value.image_infinity_upper = image_infinity;
    value.denominator_lower = denominator;
    value.error_upper = error;
    reseal_detail(value);
    return true;
}

bool detail_valid(const FormulaProbeCertificateDetail& value) {
    return value.exact && value.width == 2U && value.dimension == DIMENSION
        && value.dots == DIMENSION && value.dot_products == 41820U
        && value.radius_terms == 31212U
        && value.solution_dots == DIMENSION
        && value.sign_comparisons == DIMENSION
        && value.detail_solution_dots == DIMENSION
        && value.detail_solution_products == 204U
        && value.image_center.size() == DIMENSION
        && value.image_radius.size() == DIMENSION
        && value.solution_center.size() == DIMENSION
        && value.solution_local_radius.size() == DIMENSION
        && value.certificate.exact && value.profile_root == R63Y_PROFILE_ROOT
        && detail_component_root(value) == value.component_root
        && detail_radius_root(value) == value.radius_root
        && detail_root(value) == value.root;
}

struct DetailWork {
    std::size_t detail_certificates = 0U;
    std::size_t image_dots = 0U;
    std::size_t image_dot_products = 0U;
    std::size_t image_radius_terms = 0U;
    std::size_t native_solution_dots = 0U;
    std::size_t native_sign_comparisons = 0U;
    std::size_t detail_solution_dots = 0U;
    std::size_t detail_solution_products = 0U;
    std::size_t max_component_visits = 0U;
    std::size_t budget_divisions = 0U;
    std::size_t synthetic_cells = 0U;
    std::size_t synthetic_sign_comparisons = 0U;
    std::size_t candidate_updates = 0U;
    std::size_t operator_updates = 0U;
    std::size_t solver_updates = 0U;
};

void add_detail_work(DetailWork& work,
    const FormulaProbeCertificateDetail& detail) {
    ++work.detail_certificates;
    work.image_dots += detail.dots;
    work.image_dot_products += detail.dot_products;
    work.image_radius_terms += detail.radius_terms;
    work.native_solution_dots += detail.solution_dots;
    work.native_sign_comparisons += detail.sign_comparisons;
    work.detail_solution_dots += detail.detail_solution_dots;
    work.detail_solution_products += detail.detail_solution_products;
}

std::string detail_work_root(const DetailWork& work) {
    std::ostringstream material;
    material << work.detail_certificates << ':' << work.image_dots << ':'
        << work.image_dot_products << ':' << work.image_radius_terms << ':'
        << work.native_solution_dots << ':'
        << work.native_sign_comparisons << ':'
        << work.detail_solution_dots << ':'
        << work.detail_solution_products << ':'
        << work.max_component_visits << ':' << work.budget_divisions << ':'
        << work.synthetic_cells << ':' << work.synthetic_sign_comparisons
        << ':' << work.candidate_updates << ':' << work.operator_updates
        << ':' << work.solver_updates;
    return nextengine::nonlocal::sha256_hex(material.str());
}

bool detail_work_equal(const DetailWork& left, const DetailWork& right) {
    return detail_work_root(left) == detail_work_root(right);
}

struct DerivedDetail {
    bool exact = false;
    double center_infinity = 0.0;
    double radius_infinity = 0.0;
    double image_infinity = 0.0;
    double denominator = 0.0;
    double amplification = 0.0;
    double center_global = 0.0;
    double radius_global = 0.0;
    double native_global = 0.0;
    std::size_t maximum_center_index = 0U;
    std::size_t maximum_radius_index = 0U;
    std::string detail_root_value;
    std::string root;
};

DerivedDetail derive_detail(const FormulaProbeCertificateDetail& detail,
    DetailWork& work) {
    DerivedDetail result;
    result.detail_root_value = detail.root;
    if (!detail_valid(detail)) return result;
    result.denominator = std::nextafter(1.0 - detail.rho_upper,
        -std::numeric_limits<double>::infinity());
    if (!normal64_or_zero(result.denominator) || result.denominator <= 0.0)
        return result;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        const double center = std::abs(detail.image_center[index]);
        const double radius = detail.image_radius[index];
        const double combined = up_add64(center, radius);
        work.max_component_visits += 3U;
        if (!normal64_or_zero(center) || !normal64_or_zero(radius)
            || radius < 0.0 || !normal64_or_zero(combined))
            return result;
        if (center > result.center_infinity) {
            result.center_infinity = center;
            result.maximum_center_index = index;
        }
        if (radius > result.radius_infinity) {
            result.radius_infinity = radius;
            result.maximum_radius_index = index;
        }
        result.image_infinity = std::max(result.image_infinity, combined);
    }
    result.amplification = up_divide64(1.0, result.denominator);
    result.center_global =
        up_divide64(result.center_infinity, result.denominator);
    result.radius_global =
        up_divide64(result.radius_infinity, result.denominator);
    result.native_global =
        up_divide64(result.image_infinity, result.denominator);
    work.budget_divisions += 4U;
    result.exact = normal64_or_zero(result.amplification)
        && normal64_or_zero(result.center_global)
        && normal64_or_zero(result.radius_global)
        && normal64_or_zero(result.native_global)
        && double_bits(result.denominator)
            == double_bits(detail.denominator_lower)
        && double_bits(result.image_infinity)
            == double_bits(detail.image_infinity_upper)
        && double_bits(result.native_global) == double_bits(detail.error_upper);
    std::ostringstream material;
    material << result.exact << ':' << double_bits(result.center_infinity)
        << ':' << double_bits(result.radius_infinity) << ':'
        << double_bits(result.image_infinity) << ':'
        << double_bits(result.denominator) << ':'
        << double_bits(result.amplification) << ':'
        << double_bits(result.center_global) << ':'
        << double_bits(result.radius_global) << ':'
        << double_bits(result.native_global) << ':'
        << result.maximum_center_index << ':' << result.maximum_radius_index
        << ':' << result.detail_root_value;
    result.root = nextengine::nonlocal::sha256_hex(material.str());
    return result;
}

enum class BudgetMode : unsigned {
    Unamplified = 0U,
    CenterGlobal = 1U,
    RadiusGlobal = 2U,
    NativeGlobal = 3U,
    LocalOnly = 4U
};

const char* mode_name(BudgetMode mode) {
    switch (mode) {
    case BudgetMode::Unamplified: return "U";
    case BudgetMode::CenterGlobal: return "GC";
    case BudgetMode::RadiusGlobal: return "GR";
    case BudgetMode::NativeGlobal: return "G";
    case BudgetMode::LocalOnly: return "L";
    }
    return "INVALID";
}

double selected_budget(const DerivedDetail& value, BudgetMode mode) {
    switch (mode) {
    case BudgetMode::Unamplified: return value.image_infinity;
    case BudgetMode::CenterGlobal: return value.center_global;
    case BudgetMode::RadiusGlobal: return value.radius_global;
    case BudgetMode::NativeGlobal: return value.native_global;
    case BudgetMode::LocalOnly: return 0.0;
    }
    return std::numeric_limits<double>::infinity();
}

struct SignCell {
    bool exact = false;
    bool passed = false;
    unsigned solution = 0U;
    unsigned origin = 0U;
    BudgetMode mode = BudgetMode::LocalOnly;
    double budget = 0.0;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    std::vector<int> signs;
    std::string sign_root_value;
    std::string root;
};

SignCell make_sign_cell(const FormulaProbeCertificateDetail& solution,
    const DerivedDetail& budget, unsigned solution_selector,
    unsigned origin_selector, BudgetMode mode, DetailWork& work) {
    SignCell result;
    result.solution = solution_selector;
    result.origin = origin_selector;
    result.mode = mode;
    result.budget = selected_budget(budget, mode);
    if (!detail_valid(solution) || !budget.exact
        || !normal64_or_zero(result.budget) || result.budget < 0.0)
        return result;
    bool exact = true;
    result.signs.resize(DIMENSION);
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        const double total = up_add64(
            solution.solution_local_radius[index], result.budget);
        const double lower = std::nextafter(
            solution.solution_center[index] - total,
            -std::numeric_limits<double>::infinity());
        const double upper = std::nextafter(
            solution.solution_center[index] + total,
            std::numeric_limits<double>::infinity());
        exact = exact && normal64_or_zero(total) && std::isfinite(lower)
            && std::isfinite(upper) && lower <= upper;
        if (lower > 0.0) {
            result.signs[index] = 1;
            ++result.positive;
        } else if (upper < 0.0) {
            result.signs[index] = -1;
            ++result.negative;
        } else {
            ++result.unresolved;
        }
        ++work.synthetic_sign_comparisons;
    }
    ++work.synthetic_cells;
    result.sign_root_value = sign_root(result.signs);
    result.exact = exact
        && result.positive + result.negative + result.unresolved == DIMENSION;
    result.passed = result.exact && result.unresolved == 0U;
    std::ostringstream material;
    material << result.exact << ':' << result.passed << ':' << result.solution
        << ':' << result.origin << ':' << static_cast<unsigned>(result.mode)
        << ':' << double_bits(result.budget) << ':' << result.positive << ':'
        << result.negative << ':' << result.unresolved << ':'
        << result.sign_root_value << ':' << solution.root << ':'
        << budget.root;
    result.root = nextengine::nonlocal::sha256_hex(material.str());
    return result;
}

std::size_t cell_index(
    unsigned solution, unsigned origin, BudgetMode mode) {
    return 2U + ((solution * 2U + origin) * 4U
        + static_cast<unsigned>(mode));
}

enum class DetailFault {
    None,
    InvalidSolutionShape,
    FiniteOverflow,
    NoncontractiveDenominator,
    IncompleteDetail
};

struct DetailAudit {
    bool exact = false;
    bool complete = false;
    std::string failure;
    DetailWork work;
    std::vector<FormulaProbeCertificateDetail> details;
    std::vector<DerivedDetail> derived;
    std::vector<SignCell> cells;
    std::string root;
};

std::string detail_audit_root(const DetailAudit& value) {
    std::ostringstream material;
    material << value.exact << ':' << value.complete << ':' << value.failure
        << ':' << detail_work_root(value.work) << '|';
    for (const auto& detail : value.details)
        material << "d:" << detail.root << ';';
    for (const auto& derived : value.derived)
        material << "r:" << derived.root << ';';
    for (const auto& cell : value.cells)
        material << "c:" << cell.root << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

DetailAudit run_detail_audit(const FormulaProbeParentFixture& fixture,
    const Primary& primary, DetailFault fault = DetailFault::None) {
    using nextengine::nonlocal::fcr::formula_probe_certificate_detail;
    DetailAudit result;
    const auto fail = [&](const char* stage) {
        result.failure = stage;
        result.root = detail_audit_root(result);
        return result;
    };
    if (!primary.exact || !primary.complete || primary.x_t.size() != DIMENSION
        || primary.x_c.size() != DIMENSION)
        return fail("parent");
    std::vector<FormulaProbeBinary128> tangent = primary.x_t;
    if (fault == DetailFault::InvalidSolutionShape) tangent.pop_back();
    if (tangent.size() != DIMENSION) return fail("detail_identity");

    FormulaProbeCertificateDetail tangent_detail =
        formula_probe_certificate_detail(fixture, tangent);
    FormulaProbeCertificateDetail common_detail =
        formula_probe_certificate_detail(fixture, primary.x_c);
    if (tangent_detail.exact) add_detail_work(result.work, tangent_detail);
    if (common_detail.exact) add_detail_work(result.work, common_detail);
    if (!detail_valid(tangent_detail) || !detail_valid(common_detail))
        return fail("detail");

    if (fault == DetailFault::IncompleteDetail) {
        common_detail.exact = false;
        common_detail.image_radius.clear();
        return fail("incomplete_detail");
    }
    if (fault == DetailFault::FiniteOverflow) {
        tangent_detail.image_center[0U] =
            std::numeric_limits<double>::max();
        tangent_detail.image_radius[0U] =
            std::numeric_limits<double>::max();
        reseal_detail(tangent_detail);
    }
    if (fault == DetailFault::NoncontractiveDenominator) {
        tangent_detail.rho_upper = 1.0;
        reseal_detail(tangent_detail);
    }
    result.details = {tangent_detail, common_detail};

    DerivedDetail tangent_derived =
        derive_detail(result.details[0U], result.work);
    if (!tangent_derived.exact)
        return fail(fault == DetailFault::FiniteOverflow
                ? "finite_overflow"
                : fault == DetailFault::NoncontractiveDenominator
                    ? "noncontractive_denominator"
                    : "tangent_scalar");
    DerivedDetail common_derived =
        derive_detail(result.details[1U], result.work);
    if (!common_derived.exact) return fail("common_scalar");
    result.derived = {tangent_derived, common_derived};

    std::vector<SignCell> cells;
    cells.reserve(18U);
    cells.push_back(make_sign_cell(result.details[0U], result.derived[0U],
        0U, 0U, BudgetMode::LocalOnly, result.work));
    cells.push_back(make_sign_cell(result.details[1U], result.derived[1U],
        1U, 1U, BudgetMode::LocalOnly, result.work));
    for (unsigned solution = 0U; solution < 2U; ++solution) {
        for (unsigned origin = 0U; origin < 2U; ++origin) {
            for (unsigned mode = 0U; mode < 4U; ++mode) {
                cells.push_back(make_sign_cell(result.details[solution],
                    result.derived[origin], solution, origin,
                    static_cast<BudgetMode>(mode), result.work));
            }
        }
    }
    const bool cells_exact = cells.size() == 18U
        && std::all_of(cells.begin(), cells.end(),
            [](const SignCell& cell) { return cell.exact; });
    if (!cells_exact) return fail("cell");
    result.cells = std::move(cells);
    result.complete = true;
    result.exact = true;
    result.root = detail_audit_root(result);
    return result;
}

std::string classify_r63zg(bool apparatus, bool identity, bool work,
    bool correspondence, bool native, bool budget_independent,
    bool combined_sufficient, bool center_sufficient,
    bool radius_sufficient) {
    if (!apparatus) return "CERTIFICATE_DETAIL_APPARATUS_REJECTED";
    if (!identity) return "CERTIFICATE_DETAIL_IDENTITY_REJECTED";
    if (!work) return "CERTIFICATE_DETAIL_WORK_REJECTED";
    if (!correspondence) return "CERTIFICATE_DETAIL_CORRESPONDENCE_REJECTED";
    if (!native) return "CERTIFICATE_DETAIL_NATIVE_CELL_REJECTED";
    if (!budget_independent) return "SOLUTION_MARGIN_INTERACTION_REQUIRED";
    if (center_sufficient && radius_sufficient)
        return "AFFINE_IMAGE_CENTER_AND_RADIUS_INDEPENDENTLY_SUFFICIENT";
    if (center_sufficient) return "AFFINE_IMAGE_CENTER_RESIDUAL_SUFFICIENT";
    if (radius_sufficient) return "AFFINE_IMAGE_RADIUS_SUFFICIENT";
    if (combined_sufficient) return "AFFINE_IMAGE_MIXED_BOUND_REQUIRED";
    return "CERTIFICATE_DETAIL_FURTHER_FACTORIAL_REQUIRED";
}

bool classifier_r63zg_control() {
    bool exact = true;
    for (unsigned mask = 0U; mask < 32U; ++mask) {
        const bool apparatus = (mask & 1U) != 0U;
        const bool identity = (mask & 2U) != 0U;
        const bool work = (mask & 4U) != 0U;
        const bool correspondence = (mask & 8U) != 0U;
        const bool native = (mask & 16U) != 0U;
        const char* expected = !apparatus
            ? "CERTIFICATE_DETAIL_APPARATUS_REJECTED"
            : !identity ? "CERTIFICATE_DETAIL_IDENTITY_REJECTED"
            : !work ? "CERTIFICATE_DETAIL_WORK_REJECTED"
            : !correspondence ? "CERTIFICATE_DETAIL_CORRESPONDENCE_REJECTED"
            : !native ? "CERTIFICATE_DETAIL_NATIVE_CELL_REJECTED"
            : "SOLUTION_MARGIN_INTERACTION_REQUIRED";
        exact = exact && classify_r63zg(apparatus, identity, work,
            correspondence, native, false, false, false, false) == expected;
    }
    for (unsigned mask = 0U; mask < 16U; ++mask) {
        const bool independent = (mask & 1U) != 0U;
        const bool combined = (mask & 2U) != 0U;
        const bool center = (mask & 4U) != 0U;
        const bool radius = (mask & 8U) != 0U;
        const char* expected = !independent
            ? "SOLUTION_MARGIN_INTERACTION_REQUIRED"
            : center && radius
                ? "AFFINE_IMAGE_CENTER_AND_RADIUS_INDEPENDENTLY_SUFFICIENT"
                : center ? "AFFINE_IMAGE_CENTER_RESIDUAL_SUFFICIENT"
                : radius ? "AFFINE_IMAGE_RADIUS_SUFFICIENT"
                : combined ? "AFFINE_IMAGE_MIXED_BOUND_REQUIRED"
                : "CERTIFICATE_DETAIL_FURTHER_FACTORIAL_REQUIRED";
        exact = exact && classify_r63zg(true, true, true, true, true,
            independent, combined, center, radius) == expected;
    }
    return exact;
}

std::string r63zg_result_identity(const std::string& route,
    const std::string& fixture_root, const std::string& transaction_root,
    const std::string& parent_work_root, const std::string& audit_work,
    const std::string& controls_root) {
    std::ostringstream material;
    material << route << ':' << fixture_root << ':' << R63ZF_RESULT_SHA256
        << ':' << R63ZF_STDOUT_SHA256 << ':' << transaction_root << ':'
        << parent_work_root << ':' << audit_work << ':' << controls_root;
    return nextengine::nonlocal::sha256_hex(material.str());
}

} // namespace

#ifndef NEXTENGINE_R63ZG_EMBEDDED
int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-certificate-detail "
                         "<parent-fixture-cache>\n";
            return 2;
        }
        using namespace nextengine::nonlocal::fcr;
        const FormulaProbeParentFixture fixture =
            read_formula_probe_parent_fixture(argv[1]);
        const bool identity = formula_probe_parent_fixture_valid(fixture);
        const Primary primary = run_primary(fixture, fixture.original_rhs);
        const PrimaryAudit parent_audit = audit_primary(primary);
        const Work parent_expected{2U, 4U, 3U, 30906U, 612U, 5U, 4U,
            306U, 306U, 102U, 945U, 306U, 96390U, 96390U, 306U,
            102U, 10404U, 0U};
        const AuditWork parent_audit_expected{
            1U, 102U, 102U, 102U, 2U, 204U};
        const std::string reconstructed_parent_result = result_identity(
            "CERTIFICATE_ENCLOSURE_AMPLIFICATION_CANDIDATE", fixture.root,
            primary.root, work_root(primary.work),
            audit_work_root(primary.audit_work), R63ZF_CONTROLS_ROOT);
        const bool parent = parent_audit.apparatus
            && parent_audit.endpoint && parent_audit.quadratic
            && parent_audit.displacement && parent_audit.no_raw_zero
            && primary.root == R63ZF_PRIMARY_ROOT
            && work_equal(primary.work, parent_expected)
            && audit_work_equal(primary.audit_work, parent_audit_expected)
            && classifier_r63zf_control()
            && reconstructed_parent_result == R63ZF_RESULT_SHA256;

        const DetailAudit audit = run_detail_audit(fixture, primary);
        const DetailWork expected{2U, 204U, 83640U, 62424U, 204U, 204U,
            204U, 408U, 612U, 8U, 18U, 1836U, 0U, 0U, 0U};
        const bool work = detail_work_equal(audit.work, expected);
        const bool apparatus = audit.exact && audit.complete
            && audit.details.size() == 2U && audit.derived.size() == 2U
            && audit.cells.size() == 18U;

        bool correspondence = apparatus;
        if (apparatus) {
            correspondence = audit.details[0U].certificate.root
                    == R63ZE_TANGENT_CERTIFICATE_ROOT
                && audit.details[1U].certificate.root
                    == R63ZE_COMMON_CERTIFICATE_ROOT
                && audit.details[0U].profile_root == R63Y_PROFILE_ROOT
                && audit.details[1U].profile_root == R63Y_PROFILE_ROOT
                && double_bits(audit.details[0U].rho_upper) == R63Y_RHO_BITS
                && double_bits(audit.details[1U].rho_upper) == R63Y_RHO_BITS
                && double_bits(audit.derived[0U].denominator)
                    == R63Y_DENOMINATOR_BITS
                && double_bits(audit.derived[1U].denominator)
                    == R63Y_DENOMINATOR_BITS
                && double_bits(audit.derived[0U].native_global)
                    == 0x3f508e20150de982ULL
                && double_bits(audit.derived[1U].native_global)
                    == 0x42bc23fa1e984623ULL;
        }

        bool native = correspondence;
        bool local = correspondence;
        bool budget_independent = false;
        bool combined_sufficient = false;
        bool center_sufficient = false;
        bool radius_sufficient = false;
        bool contraction_boundary_changed = false;
        if (correspondence) {
            const SignCell& local_t = audit.cells[0U];
            const SignCell& local_c = audit.cells[1U];
            const SignCell& tt = audit.cells[cell_index(
                0U, 0U, BudgetMode::NativeGlobal)];
            const SignCell& ct = audit.cells[cell_index(
                1U, 0U, BudgetMode::NativeGlobal)];
            const SignCell& tc = audit.cells[cell_index(
                0U, 1U, BudgetMode::NativeGlobal)];
            const SignCell& cc = audit.cells[cell_index(
                1U, 1U, BudgetMode::NativeGlobal)];
            native = tt.passed == audit.details[0U].certificate.passed
                && tt.positive == audit.details[0U].certificate.positive
                && tt.negative == audit.details[0U].certificate.negative
                && tt.unresolved == audit.details[0U].certificate.unresolved
                && tt.sign_root_value
                    == audit.details[0U].certificate.sign_root
                && cc.passed == audit.details[1U].certificate.passed
                && cc.positive == audit.details[1U].certificate.positive
                && cc.negative == audit.details[1U].certificate.negative
                && cc.unresolved == audit.details[1U].certificate.unresolved
                && cc.sign_root_value
                    == audit.details[1U].certificate.sign_root;
            local = local_t.passed && local_c.passed
                && local_t.sign_root_value == primary.tangent_sign_root
                && local_c.sign_root_value == primary.common_sign_root;
            budget_independent = tt.passed && ct.passed
                && !tc.passed && !cc.passed;
            combined_sufficient = !tc.passed && !cc.passed;
            const SignCell& t_center = audit.cells[cell_index(
                0U, 1U, BudgetMode::CenterGlobal)];
            const SignCell& c_center = audit.cells[cell_index(
                1U, 1U, BudgetMode::CenterGlobal)];
            const SignCell& t_radius = audit.cells[cell_index(
                0U, 1U, BudgetMode::RadiusGlobal)];
            const SignCell& c_radius = audit.cells[cell_index(
                1U, 1U, BudgetMode::RadiusGlobal)];
            center_sufficient = !t_center.passed && !c_center.passed;
            radius_sufficient = !t_radius.passed && !c_radius.passed;
            for (unsigned solution = 0U; solution < 2U; ++solution) {
                for (unsigned origin = 0U; origin < 2U; ++origin) {
                    const SignCell& unamplified = audit.cells[cell_index(
                        solution, origin, BudgetMode::Unamplified)];
                    const SignCell& amplified = audit.cells[cell_index(
                        solution, origin, BudgetMode::NativeGlobal)];
                    contraction_boundary_changed =
                        contraction_boundary_changed
                        || unamplified.passed != amplified.passed
                        || unamplified.positive != amplified.positive
                        || unamplified.negative != amplified.negative
                        || unamplified.unresolved != amplified.unresolved;
                }
            }
        }
        correspondence = correspondence && local;
        std::ostringstream observation_material;
        observation_material << budget_independent << ':'
            << combined_sufficient << ':' << center_sufficient << ':'
            << radius_sufficient << ':' << contraction_boundary_changed;
        const std::string observations_root =
            nextengine::nonlocal::sha256_hex(observation_material.str());
        const std::string transaction_root =
            nextengine::nonlocal::sha256_hex(
                audit.root + ":" + observations_root);

        const bool classifier = classifier_r63zg_control();
        const DetailAudit invalid = run_detail_audit(
            fixture, primary, DetailFault::InvalidSolutionShape);
        const DetailAudit overflow = run_detail_audit(
            fixture, primary, DetailFault::FiniteOverflow);
        const DetailAudit noncontractive = run_detail_audit(
            fixture, primary, DetailFault::NoncontractiveDenominator);
        const DetailAudit incomplete = run_detail_audit(
            fixture, primary, DetailFault::IncompleteDetail);
        DetailWork base_prefix{2U, 204U, 83640U, 62424U, 204U, 204U,
            204U, 408U, 0U, 0U, 0U, 0U, 0U, 0U, 0U};
        DetailWork overflow_prefix = base_prefix;
        overflow_prefix.max_component_visits = 3U;
        const DetailWork no_detail_work;
        const bool failure_controls = !invalid.exact && !invalid.complete
            && invalid.failure == "detail_identity" && invalid.cells.empty()
            && detail_work_equal(invalid.work, no_detail_work)
            && !overflow.exact && !overflow.complete
            && overflow.failure == "finite_overflow" && overflow.cells.empty()
            && detail_work_equal(overflow.work, overflow_prefix)
            && !noncontractive.exact && !noncontractive.complete
            && noncontractive.failure == "noncontractive_denominator"
            && noncontractive.cells.empty()
            && detail_work_equal(noncontractive.work, base_prefix)
            && !incomplete.exact && !incomplete.complete
            && incomplete.failure == "incomplete_detail"
            && incomplete.cells.empty()
            && detail_work_equal(incomplete.work, base_prefix);

        FormulaProbeParentFixture corrupt_fixture = fixture;
        corrupt_fixture.original_rhs_root = nextengine::nonlocal::sha256_hex(
            corrupt_fixture.original_rhs_root + ":corrupt");
        const FormulaProbeCertificateDetail corrupt_fixture_detail =
            formula_probe_certificate_detail(corrupt_fixture, primary.x_t);
        const bool corrupt_fixture_control =
            corrupt_fixture.verifier_profile.root
                == fixture.verifier_profile.root
            && !formula_probe_parent_fixture_valid(corrupt_fixture)
            && !corrupt_fixture_detail.exact
            && corrupt_fixture_detail.root.empty()
            && corrupt_fixture_detail.image_center.empty()
            && corrupt_fixture_detail.image_radius.empty()
            && corrupt_fixture_detail.solution_center.empty()
            && corrupt_fixture_detail.solution_local_radius.empty();

        FormulaProbeCertificateDetail center_mutation = audit.details[1U];
        const std::size_t center_index =
            audit.derived[1U].maximum_center_index;
        center_mutation.image_center[center_index] = std::nextafter(
            center_mutation.image_center[center_index],
            center_mutation.image_center[center_index] >= 0.0
                ? std::numeric_limits<double>::infinity()
                : -std::numeric_limits<double>::infinity());
        const bool center_refresh =
            refresh_detail_native_scalars(center_mutation);
        DetailWork mutation_work;
        const DerivedDetail center_derived =
            derive_detail(center_mutation, mutation_work);
        FormulaProbeCertificateDetail radius_mutation = audit.details[1U];
        const std::size_t radius_index =
            audit.derived[1U].maximum_radius_index;
        radius_mutation.image_radius[radius_index] = std::nextafter(
            radius_mutation.image_radius[radius_index],
            std::numeric_limits<double>::infinity());
        const bool radius_refresh =
            refresh_detail_native_scalars(radius_mutation);
        const DerivedDetail radius_derived =
            derive_detail(radius_mutation, mutation_work);
        const SignCell center_cell = make_sign_cell(audit.details[1U],
            center_derived, 1U, 1U, BudgetMode::CenterGlobal, mutation_work);
        const SignCell radius_cell = make_sign_cell(audit.details[1U],
            radius_derived, 1U, 1U, BudgetMode::RadiusGlobal, mutation_work);
        SignCell budget_mutation = audit.cells[cell_index(
            1U, 1U, BudgetMode::NativeGlobal)];
        DerivedDetail budget_derived = audit.derived[1U];
        budget_derived.native_global = std::nextafter(
            budget_derived.native_global,
            std::numeric_limits<double>::infinity());
        std::ostringstream budget_material;
        budget_material << budget_derived.root << ":budget_mutation:"
            << double_bits(budget_derived.native_global);
        budget_derived.root =
            nextengine::nonlocal::sha256_hex(budget_material.str());
        DetailWork budget_work;
        const SignCell mutated_budget_cell = make_sign_cell(audit.details[1U],
            budget_derived, 1U, 1U, BudgetMode::NativeGlobal, budget_work);
        FormulaProbeCertificateDetail stale_swap = audit.details[1U];
        std::swap(stale_swap.image_center[0U], stale_swap.image_center[1U]);
        FormulaProbeCertificateDetail native_error = audit.details[1U];
        native_error.error_upper = std::nextafter(native_error.error_upper,
            std::numeric_limits<double>::infinity());
        reseal_detail(native_error);
        DetailWork native_error_work;
        const DerivedDetail native_error_derived =
            derive_detail(native_error, native_error_work);
        DerivedDetail native_error_budget = audit.derived[1U];
        native_error_budget.native_global = native_error.error_upper;
        native_error_budget.root = nextengine::nonlocal::sha256_hex(
            native_error_budget.root + ":native_error_mutation:"
            + std::to_string(double_bits(native_error_budget.native_global)));
        const SignCell native_error_cell = make_sign_cell(audit.details[1U],
            native_error_budget, 1U, 1U, BudgetMode::NativeGlobal,
            native_error_work);
        const SignCell& original_center_cell = audit.cells[cell_index(
            1U, 1U, BudgetMode::CenterGlobal)];
        const SignCell& original_radius_cell = audit.cells[cell_index(
            1U, 1U, BudgetMode::RadiusGlobal)];
        const SignCell& original_native_cell = audit.cells[cell_index(
            1U, 1U, BudgetMode::NativeGlobal)];
        const bool mutation_controls = center_refresh && radius_refresh
            && center_derived.exact && radius_derived.exact
            && center_mutation.root != audit.details[1U].root
            && radius_mutation.root != audit.details[1U].root
            && center_derived.root != audit.derived[1U].root
            && radius_derived.root != audit.derived[1U].root
            && center_cell.exact && radius_cell.exact
            && center_cell.root != original_center_cell.root
            && radius_cell.root != original_radius_cell.root
            && mutated_budget_cell.exact
            && mutated_budget_cell.root != budget_mutation.root
            && !detail_valid(stale_swap) && !native_error_derived.exact
            && native_error_cell.exact
            && native_error_cell.passed == original_native_cell.passed
            && native_error_cell.positive == original_native_cell.positive
            && native_error_cell.negative == original_native_cell.negative
            && native_error_cell.unresolved == original_native_cell.unresolved;

        const std::string mutation_transaction =
            nextengine::nonlocal::sha256_hex(transaction_root + ":"
                + center_derived.root + ":" + radius_derived.root + ":"
                + center_cell.root + ":" + radius_cell.root + ":"
                + mutated_budget_cell.root + ":" + native_error_cell.root);
        const bool local_controls = classifier && failure_controls
            && corrupt_fixture_control && mutation_controls;
        std::ostringstream control_material;
        control_material << classifier << ':' << failure_controls << ':'
            << invalid.root << ':' << overflow.root << ':'
            << noncontractive.root << ':' << incomplete.root << ':'
            << corrupt_fixture_control << ':'
            << corrupt_fixture.original_rhs_root << ':'
            << corrupt_fixture_detail.root << ':'
            << mutation_controls << ':' << center_mutation.root << ':'
            << radius_mutation.root << ':' << center_cell.root << ':'
            << radius_cell.root << ':' << mutated_budget_cell.root << ':'
            << stale_swap.root << ':' << native_error.root << ':'
            << native_error_cell.root << ':'
            << mutation_transaction;
        const std::string preliminary_controls_root =
            nextengine::nonlocal::sha256_hex(control_material.str());
        const std::string provisional_route = classify_r63zg(
            apparatus && local_controls, identity && parent, work,
            correspondence, native, budget_independent, combined_sufficient,
            center_sufficient, radius_sufficient);
        const std::string base_result = r63zg_result_identity(
            provisional_route, fixture.root, transaction_root,
            work_root(primary.work), detail_work_root(audit.work),
            preliminary_controls_root);
        const std::string mutation_result = r63zg_result_identity(
            provisional_route, fixture.root, mutation_transaction,
            work_root(primary.work), detail_work_root(audit.work),
            preliminary_controls_root);
        const bool result_sealing = base_result != mutation_result;
        control_material << ':' << result_sealing << ':' << base_result << ':'
            << mutation_result;
        const std::string controls_root =
            nextengine::nonlocal::sha256_hex(control_material.str());
        const bool controls = local_controls && result_sealing;
        const std::string route = classify_r63zg(apparatus && controls,
            identity && parent, work, correspondence, native,
            budget_independent, combined_sufficient, center_sufficient,
            radius_sufficient);
        const std::string result_root = r63zg_result_identity(route,
            fixture.root, transaction_root, work_root(primary.work),
            detail_work_root(audit.work), controls_root);
        const bool exact = apparatus && identity && parent && work
            && correspondence && native && controls
            && (route == "SOLUTION_MARGIN_INTERACTION_REQUIRED"
                || route
                    == "AFFINE_IMAGE_CENTER_AND_RADIUS_INDEPENDENTLY_SUFFICIENT"
                || route == "AFFINE_IMAGE_CENTER_RESIDUAL_SUFFICIENT"
                || route == "AFFINE_IMAGE_RADIUS_SUFFICIENT"
                || route == "AFFINE_IMAGE_MIXED_BOUND_REQUIRED"
                || route == "CERTIFICATE_DETAIL_FURTHER_FACTORIAL_REQUIRED");

        std::cout
            << "{\"schema\":\"nextengine.nonlocal.r63zg_certificate_detail.v1\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"route\":\"" << route << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"parent_result_sha256\":\"" << R63ZF_RESULT_SHA256
            << "\",\"parent_stdout_sha256\":\"" << R63ZF_STDOUT_SHA256
            << "\",\"details\":[";
        for (std::size_t index = 0U; index < audit.details.size(); ++index) {
            if (index != 0U) std::cout << ',';
            const auto& detail = audit.details[index];
            const auto& derived = audit.derived[index];
            std::cout << "{\"endpoint\":\""
                << (index == 0U ? "tangent" : "common")
                << "\",\"rho_bits\":\"" << binary64_bits_hex(detail.rho_upper)
                << "\",\"denominator_bits\":\""
                << binary64_bits_hex(derived.denominator)
                << "\",\"amplification_bits\":\""
                << binary64_bits_hex(derived.amplification)
                << "\",\"center_infinity_bits\":\""
                << binary64_bits_hex(derived.center_infinity)
                << "\",\"radius_infinity_bits\":\""
                << binary64_bits_hex(derived.radius_infinity)
                << "\",\"image_infinity_bits\":\""
                << binary64_bits_hex(derived.image_infinity)
                << "\",\"center_global_bits\":\""
                << binary64_bits_hex(derived.center_global)
                << "\",\"radius_global_bits\":\""
                << binary64_bits_hex(derived.radius_global)
                << "\",\"native_global_bits\":\""
                << binary64_bits_hex(derived.native_global)
                << "\",\"certificate_root\":\"" << detail.certificate.root
                << "\",\"detail_root\":\"" << detail.root
                << "\",\"derived_root\":\"" << derived.root << "\"}";
        }
        std::cout << "],\"cells\":[";
        for (std::size_t index = 0U; index < audit.cells.size(); ++index) {
            if (index != 0U) std::cout << ',';
            const SignCell& cell = audit.cells[index];
            std::cout << "{\"solution\":\""
                << (cell.solution == 0U ? "tangent" : "common")
                << "\",\"origin\":\""
                << (cell.origin == 0U ? "tangent" : "common")
                << "\",\"mode\":\"" << mode_name(cell.mode)
                << "\",\"passed\":" << (cell.passed ? "true" : "false")
                << ",\"positive\":" << cell.positive
                << ",\"negative\":" << cell.negative
                << ",\"unresolved\":" << cell.unresolved
                << ",\"sign_root\":\"" << cell.sign_root_value
                << "\",\"root\":\"" << cell.root << "\"}";
        }
        std::cout << "],\"observations\":{\"native_budget_independent\":"
            << (budget_independent ? "true" : "false")
            << ",\"combined_sufficient\":"
            << (combined_sufficient ? "true" : "false")
            << ",\"center_sufficient\":"
            << (center_sufficient ? "true" : "false")
            << ",\"radius_sufficient\":"
            << (radius_sufficient ? "true" : "false")
            << ",\"contraction_boundary_changed\":"
            << (contraction_boundary_changed ? "true" : "false")
            << ",\"root\":\"" << observations_root << "\"}"
            << ",\"controls\":{\"classifier\":"
            << (classifier ? "true" : "false")
            << ",\"failures\":" << (failure_controls ? "true" : "false")
            << ",\"corrupt_fixture\":"
            << (corrupt_fixture_control ? "true" : "false")
            << ",\"mutations\":" << (mutation_controls ? "true" : "false")
            << ",\"result_sealing\":" << (result_sealing ? "true" : "false")
            << ",\"root\":\"" << controls_root << "\"}"
            << ",\"work\":{\"parent_root\":\"" << work_root(primary.work)
            << "\",\"detail_root\":\"" << detail_work_root(audit.work)
            << "\",\"detail_certificates\":"
            << audit.work.detail_certificates
            << ",\"image_dots\":" << audit.work.image_dots
            << ",\"image_dot_products\":" << audit.work.image_dot_products
            << ",\"image_radius_terms\":" << audit.work.image_radius_terms
            << ",\"native_solution_dots\":"
            << audit.work.native_solution_dots
            << ",\"native_sign_comparisons\":"
            << audit.work.native_sign_comparisons
            << ",\"detail_solution_dots\":"
            << audit.work.detail_solution_dots
            << ",\"detail_solution_products\":"
            << audit.work.detail_solution_products
            << ",\"max_component_visits\":"
            << audit.work.max_component_visits
            << ",\"budget_divisions\":" << audit.work.budget_divisions
            << ",\"synthetic_cells\":" << audit.work.synthetic_cells
            << ",\"synthetic_sign_comparisons\":"
            << audit.work.synthetic_sign_comparisons
            << ",\"candidate_updates\":" << audit.work.candidate_updates
            << ",\"operator_updates\":" << audit.work.operator_updates
            << ",\"solver_updates\":" << audit.work.solver_updates << "}"
            << ",\"transaction_root\":\"" << transaction_root << "\""
            << ",\"timing_admitted\":false"
            << ",\"runtime_authority\":false"
            << ",\"gpu_authority\":false"
            << ",\"production_authority\":false"
            << ",\"result_sha256\":\"" << result_root << "\"}\n";
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-certificate-detail: "
                  << error.what() << '\n';
        return 2;
    }
}
#endif
