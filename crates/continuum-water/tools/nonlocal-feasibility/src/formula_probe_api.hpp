#pragma once

#include <cstddef>
#include <string>
#include <vector>

namespace nextengine::nonlocal::fcr {

#if defined(__SIZEOF_FLOAT128__)
using FormulaProbeBinary128 = __float128;
#else
#error "The formula probe API requires the frozen __float128 profile"
#endif

struct FormulaProbeProfile {
    bool exact = false;
    bool contained = false;
    bool contractive = false;
    std::size_t width = 0U;
    std::size_t dimension = 0U;
    std::size_t vector_containments = 0U;
    std::size_t matrix_containments = 0U;
    std::size_t nonzero_vector_lows = 0U;
    std::size_t nonzero_matrix_lows = 0U;
    std::vector<double> vector_components;
    std::vector<double> vector_radius;
    std::vector<double> matrix_components;
    std::vector<double> matrix_radius;
    double rho_upper = 0.0;
    double maximum_vector_radius = 0.0;
    double maximum_matrix_radius = 0.0;
    std::string exact_root;
    std::string component_root;
    std::string radius_root;
    std::string root;
};

struct FormulaProbeCertificate {
    bool exact = false;
    bool passed = false;
    std::size_t positive = 0U;
    std::size_t negative = 0U;
    std::size_t unresolved = 0U;
    double error_upper = 0.0;
    std::string solution_root;
    std::string sign_root;
    std::string root;
};

struct FormulaProbeCertificateDetail {
    bool exact = false;
    std::size_t width = 0U;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    std::size_t dot_products = 0U;
    std::size_t radius_terms = 0U;
    std::size_t solution_dots = 0U;
    std::size_t sign_comparisons = 0U;
    std::size_t detail_solution_dots = 0U;
    std::size_t detail_solution_products = 0U;
    double rho_upper = 0.0;
    double image_infinity_upper = 0.0;
    double denominator_lower = 0.0;
    double error_upper = 0.0;
    double minimum_separation = 0.0;
    std::vector<double> image_center;
    std::vector<double> image_radius;
    std::vector<double> solution_center;
    std::vector<double> solution_local_radius;
    FormulaProbeCertificate certificate;
    std::string profile_root;
    std::string component_root;
    std::string radius_root;
    std::string root;
};

struct FormulaProbeSolutionExpansion {
    bool exact = false;
    std::size_t width = 0U;
    std::size_t dimension = 0U;
    std::size_t containments = 0U;
    std::size_t nonzero_lows = 0U;
    std::vector<double> components;
    std::vector<double> radius;
    std::string source_solution_root;
    std::string component_root;
    std::string radius_root;
    std::string root;
};

struct FormulaProbeProduct {
    bool exact = false;
    std::vector<FormulaProbeBinary128> value;
    std::size_t inner_dots = 0U;
    std::size_t outer_dots = 0U;
    std::size_t inner_terms = 0U;
    std::size_t outer_terms = 0U;
    std::size_t scale_products = 0U;
    std::string root;
};

struct FormulaProbeSolve {
    bool exact = false;
    std::vector<FormulaProbeBinary128> intermediate;
    std::vector<FormulaProbeBinary128> solution;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::string root;
};

struct FormulaProbeScalar {
    bool exact = false;
    bool positive = false;
    FormulaProbeBinary128 value =
        static_cast<FormulaProbeBinary128>(0.0);
    FormulaProbeBinary128 bound =
        static_cast<FormulaProbeBinary128>(0.0);
    std::string root;
};

struct FormulaProbeParentFixture {
    bool exact = false;
    std::string schema;
    std::size_t dimension = 0U;
    std::size_t tangent_columns = 0U;

    std::string r63y_stdout_sha256;
    std::string r63z_stdout_sha256;
    std::string r63za_stdout_sha256;
    std::string r63zb_stdout_sha256;
    std::string r63zb_result_sha256;

    std::string r63x_semantic;
    std::string r63x_profile_root;
    std::string r63x_endpoint_root;
    std::string r63x_solution_set_root;
    std::vector<std::vector<FormulaProbeBinary128>> baseline_solutions;
    std::vector<FormulaProbeCertificate> baseline_certificates;

    std::vector<FormulaProbeBinary128> tangent;
    FormulaProbeBinary128 sigma = static_cast<FormulaProbeBinary128>(0.0);
    std::string tangent_root;

    std::vector<double> factor_upper;
    std::vector<std::size_t> permutation;
    FormulaProbeBinary128 inverse_scale =
        static_cast<FormulaProbeBinary128>(0.0);
    std::string factor_fixture_root;
    std::string factor_root;
    std::string permutation_root;

    std::vector<FormulaProbeBinary128> original_rhs;
    std::vector<FormulaProbeBinary128> projected_rhs;
    FormulaProbeBinary128 projected_scale =
        static_cast<FormulaProbeBinary128>(0.0);
    std::string original_rhs_root;
    std::string projected_rhs_root;

    std::vector<double> common_components;
    std::string common_semantic;
    std::string common_component_root;
    std::string common_root;

    FormulaProbeProfile verifier_profile;
    std::string verifier_semantic;
    std::string verifier_profile_root;

    std::vector<std::vector<FormulaProbeBinary128>>
        common_projected_solutions;
    std::vector<FormulaProbeCertificate> common_projected_certificates;
    std::string finite_transaction_root;
    std::string common_projected_comparator_root;

    std::string root;
};

struct FormulaProbeTwofoldOperator {
    bool exact = false;
    bool contained = false;
    std::string role;
    std::size_t dimension = 0U;
    std::size_t width = 0U;
    std::size_t entries = 0U;
    std::size_t containments = 0U;
    std::size_t nonzero_lows = 0U;
    std::vector<double> components;
    std::vector<double> radius;
    double maximum_radius = 0.0;
    std::string source_root;
    std::string materialization_root;
    std::string tangent_root;
    std::string component_root;
    std::string radius_root;
    std::string carrier_root;
    std::string root;
};

struct FormulaProbeTwofoldProduct {
    bool exact = false;
    std::string site;
    std::size_t dimension = 0U;
    std::size_t entries = 0U;
    std::vector<double> input_components;
    std::vector<double> value_components;
    std::string artifact_root;
    std::string input_root;
    std::string value_root;
    std::string root;
};

struct FormulaProbeTwofoldWork {
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

struct FormulaProbeTwofoldState {
    bool exact = false;
    std::size_t index = 0U;
    std::vector<double> solution_components;
    std::vector<double> residual_components;
    std::vector<double> direction_components;
    std::vector<double> preconditioned_components;
    FormulaProbeCertificate certificate;
    std::string operator_root;
    std::string solve_root;
    std::string rho_root;
    std::string denominator_root;
    std::string alpha_root;
    std::string beta_root;
    std::string root;
};

struct FormulaProbeTwofoldRecurrence {
    bool exact = false;
    bool complete = false;
    bool positivity_exact = false;
    std::size_t sealed_states = 0U;
    std::string failure_stage;
    FormulaProbeTwofoldWork work;
    std::vector<FormulaProbeTwofoldState> states;
    std::vector<FormulaProbeTwofoldProduct> products;
    std::string fixture_root;
    std::string artifact_root;
    std::string certificate_root;
    std::string transaction_root;
    std::string root;
};

struct FormulaProbeTwofoldExactAudit {
    bool exact = false;
    bool complete = false;
    std::size_t operator_containments = 0U;
    std::size_t operator_entries = 0U;
    std::size_t product_containments = 0U;
    std::size_t product_rows = 0U;
    std::size_t exact_upper_dots = 0U;
    std::size_t exact_gram_products = 0U;
    std::size_t exact_mirrors = 0U;
    std::size_t exact_product_rows = 0U;
    std::size_t exact_product_products = 0U;
    std::vector<double> product_radius;
    std::string first_failure;
    std::string exact_operator_root;
    std::string operator_containment_root;
    std::string product_containment_root;
    std::string work_root;
    std::string root;
};

FormulaProbeParentFixture capture_formula_probe_parent_fixture();
std::string formula_probe_parent_fixture_root(
    const FormulaProbeParentFixture& fixture);
bool formula_probe_parent_fixture_valid(
    const FormulaProbeParentFixture& fixture);
std::string formula_probe_binary128_vector_root(
    const std::vector<FormulaProbeBinary128>& values);
std::string formula_probe_binary64_vector_root(
    const std::vector<double>& values);
std::string formula_probe_solution_set_root(
    const std::vector<std::vector<FormulaProbeBinary128>>& solutions);
std::string formula_probe_certificate_set_root(
    const std::vector<FormulaProbeCertificate>& certificates);
FormulaProbeProduct formula_probe_tangent_product(
    const FormulaProbeParentFixture& fixture,
    const std::vector<FormulaProbeBinary128>& input);
FormulaProbeProduct formula_probe_common_product(
    const FormulaProbeParentFixture& fixture,
    const std::vector<FormulaProbeBinary128>& input);
FormulaProbeProduct formula_probe_control_dense_product(
    const std::vector<FormulaProbeBinary128>& matrix,
    const std::vector<FormulaProbeBinary128>& input);
FormulaProbeSolve formula_probe_factor_solve(
    const FormulaProbeParentFixture& fixture,
    const std::vector<FormulaProbeBinary128>& source,
    FormulaProbeBinary128 inverse_scale);
FormulaProbeScalar formula_probe_scalar_dot(
    const std::vector<FormulaProbeBinary128>& left,
    const std::vector<FormulaProbeBinary128>& right);
FormulaProbeCertificate formula_probe_certificate(
    const FormulaProbeParentFixture& fixture,
    const std::vector<FormulaProbeBinary128>& solution);
FormulaProbeCertificateDetail formula_probe_certificate_detail(
    const FormulaProbeParentFixture& fixture,
    const std::vector<FormulaProbeBinary128>& solution);
FormulaProbeSolutionExpansion formula_probe_solution_expansion(
    const FormulaProbeParentFixture& fixture,
    const std::vector<FormulaProbeBinary128>& solution);
FormulaProbeTwofoldOperator formula_probe_tangent_twofold_operator(
    const FormulaProbeParentFixture& fixture,
    const std::vector<FormulaProbeBinary128>& materialized_matrix);
FormulaProbeTwofoldOperator formula_probe_common_twofold_operator(
    const FormulaProbeParentFixture& fixture);
FormulaProbeTwofoldOperator formula_probe_widen_twofold_operator_radius(
    const FormulaProbeTwofoldOperator& source, double minimum_radius);
bool formula_probe_twofold_operator_valid(
    const FormulaProbeTwofoldOperator& value);
FormulaProbeTwofoldRecurrence formula_probe_twofold_recurrence(
    const FormulaProbeParentFixture& fixture,
    const FormulaProbeTwofoldOperator& artifact);
bool formula_probe_twofold_recurrence_valid(
    const FormulaProbeTwofoldRecurrence& value);
FormulaProbeTwofoldExactAudit formula_probe_twofold_exact_audit(
    const FormulaProbeParentFixture& fixture,
    const FormulaProbeTwofoldOperator& artifact,
    const FormulaProbeTwofoldRecurrence& recurrence);
bool formula_probe_twofold_exact_audit_valid(
    const FormulaProbeTwofoldExactAudit& value);

} // namespace nextengine::nonlocal::fcr
