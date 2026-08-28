#pragma once

#include <cstddef>
#include <memory>
#include <optional>
#include <string>
#include <utility>
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
    std::size_t kernel_buffer_allocations = 0U;
    std::size_t temporary_operand_allocations = 0U;
    std::size_t operand_components_copied = 0U;
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

struct FormulaProbeK2Scalar {
    bool exact = false;
    double high = 0.0;
    double low = 0.0;
    std::string root;
};

struct FormulaProbeHybridIdentity {
    bool exact = false;
    std::string carrier_root;
    std::string callback_identity_root;
    std::string root;
};

struct FormulaProbeK2Solve {
    bool exact = false;
    std::string role;
    std::size_t dimension = 0U;
    std::size_t forward_terms = 0U;
    std::size_t backward_terms = 0U;
    std::size_t divisions = 0U;
    std::vector<double> intermediate_components;
    std::vector<double> solution_components;
    std::string root;
};

struct FormulaProbeK2Dot {
    bool exact = false;
    std::size_t terms = 0U;
    FormulaProbeK2Scalar value;
    std::string left_root;
    std::string right_root;
    std::string root;
};

struct FormulaProbeK2CertificateCheck {
    bool exact = false;
    std::size_t dimension = 0U;
    std::size_t dots = 0U;
    std::size_t dot_products = 0U;
    std::size_t radius_terms = 0U;
    std::size_t solution_dots = 0U;
    std::size_t sign_comparisons = 0U;
    FormulaProbeCertificate certificate;
    std::string profile_root;
    std::string component_root;
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

struct FormulaProbeTangentBoundaryFixture {
    std::size_t dimension = 0U;
    std::size_t tangent_columns = 0U;
    std::vector<FormulaProbeBinary128> tangent;
    FormulaProbeBinary128 sigma = static_cast<FormulaProbeBinary128>(0.0);
    std::string tangent_root;
    FormulaProbeBinary128 inverse_scale =
        static_cast<FormulaProbeBinary128>(0.0);
    FormulaProbeBinary128 projected_scale =
        static_cast<FormulaProbeBinary128>(0.0);
    std::vector<FormulaProbeBinary128> projected_rhs;
    std::string projected_rhs_root;
    std::vector<FormulaProbeBinary128> original_rhs;
    std::string original_rhs_root;
    std::vector<FormulaProbeBinary128> baseline_solution_0;
    std::vector<FormulaProbeBinary128> baseline_solution_1;
    std::vector<FormulaProbeBinary128> baseline_solution_2;
    std::vector<FormulaProbeBinary128> common_projected_solution_2;
};

enum class FormulaProbeTangentInputRole {
    ProjectedRhs,
    OriginalRhs,
    Baseline0,
    Baseline1,
    Baseline2,
    CommonProjected2,
    Unknown,
};

struct FormulaProbeTangentAdmissionWork {
    std::size_t predicate_checks = 0U;
    std::size_t tangent_payload_hashes = 0U;
    std::size_t frozen_scalar_parses = 0U;
    std::size_t tangent_components_copied = 0U;
    std::size_t context_root_derivations = 0U;
    std::size_t receipt_root_derivations = 0U;
    std::size_t result_root_derivations = 0U;
    std::string root;
};

struct FormulaProbeAdmittedProductWork {
    std::size_t input_guard_checks = 0U;
    std::size_t input_payload_hashes = 0U;
    std::size_t input_identity_checks = 0U;
    std::size_t kernel_calls = 0U;
    std::size_t inner_dots = 0U;
    std::size_t outer_dots = 0U;
    std::size_t inner_terms = 0U;
    std::size_t outer_terms = 0U;
    std::size_t scale_products = 0U;
    std::size_t kernel_buffer_allocations = 0U;
    std::size_t temporary_operand_allocations = 0U;
    std::size_t operand_components_copied = 0U;
    std::size_t kernel_root_derivations = 0U;
    std::size_t receipt_root_derivations = 0U;
    std::size_t result_root_derivations = 0U;
    std::string root;
};

class FormulaProbeTangentAdmission;
class FormulaProbeAdmittedProduct;

class FormulaProbeAdmittedTangent final {
public:
    FormulaProbeAdmittedTangent(const FormulaProbeAdmittedTangent&) = default;
    FormulaProbeAdmittedTangent(FormulaProbeAdmittedTangent&&) = delete;
    FormulaProbeAdmittedTangent& operator=(
        const FormulaProbeAdmittedTangent&) = delete;
    FormulaProbeAdmittedTangent& operator=(
        FormulaProbeAdmittedTangent&&) = delete;

    std::size_t dimension() const noexcept;
    std::size_t tangent_columns() const noexcept;
    const std::string& tangent_root() const noexcept;
    const std::string& context_root() const noexcept;

private:
    FormulaProbeAdmittedTangent(
        std::size_t dimension, std::size_t tangent_columns,
        std::shared_ptr<const std::vector<FormulaProbeBinary128>> tangent,
        FormulaProbeBinary128 sigma, std::string tangent_root,
        std::string context_root);

    std::size_t dimension_ = 0U;
    std::size_t tangent_columns_ = 0U;
    std::shared_ptr<const std::vector<FormulaProbeBinary128>> tangent_;
    FormulaProbeBinary128 sigma_ =
        static_cast<FormulaProbeBinary128>(0.0);
    std::string tangent_root_;
    std::string context_root_;

    friend FormulaProbeTangentAdmission formula_probe_admit_tangent(
        const FormulaProbeTangentBoundaryFixture& fixture);
    friend FormulaProbeAdmittedProduct formula_probe_admitted_tangent_product(
        const FormulaProbeAdmittedTangent& context,
        FormulaProbeTangentInputRole role,
        const std::vector<FormulaProbeBinary128>& input);
};

class FormulaProbeTangentAdmission final {
public:
    FormulaProbeTangentAdmission(const FormulaProbeTangentAdmission&) = default;
    FormulaProbeTangentAdmission(FormulaProbeTangentAdmission&&) = delete;
    FormulaProbeTangentAdmission& operator=(
        const FormulaProbeTangentAdmission&) = delete;
    FormulaProbeTangentAdmission& operator=(
        FormulaProbeTangentAdmission&&) = delete;

    bool exact() const noexcept;
    bool has_context() const noexcept;
    const std::string& failure_stage() const noexcept;
    const FormulaProbeTangentAdmissionWork& work() const noexcept;
    const FormulaProbeAdmittedTangent& context() const;
    const std::string& root() const noexcept;

private:
    FormulaProbeTangentAdmission(bool exact, std::string failure_stage,
        FormulaProbeTangentAdmissionWork work,
        const std::optional<FormulaProbeAdmittedTangent>& context,
        std::string root);

    bool exact_ = false;
    std::string failure_stage_;
    FormulaProbeTangentAdmissionWork work_;
    std::optional<FormulaProbeAdmittedTangent> context_;
    std::string root_;

    friend FormulaProbeTangentAdmission formula_probe_admit_tangent(
        const FormulaProbeTangentBoundaryFixture& fixture);
};

class FormulaProbeAdmittedProduct final {
public:
    FormulaProbeAdmittedProduct(const FormulaProbeAdmittedProduct&) = default;
    FormulaProbeAdmittedProduct(FormulaProbeAdmittedProduct&&) = default;
    FormulaProbeAdmittedProduct& operator=(
        const FormulaProbeAdmittedProduct&) = delete;
    FormulaProbeAdmittedProduct& operator=(
        FormulaProbeAdmittedProduct&&) = delete;

    bool exact() const noexcept;
    const std::string& failure_stage() const noexcept;
    const FormulaProbeProduct& product() const noexcept;
    const FormulaProbeAdmittedProductWork& work() const noexcept;
    const std::string& context_root() const noexcept;
    FormulaProbeTangentInputRole input_role() const noexcept;
    const std::string& input_root() const noexcept;
    const std::string& root() const noexcept;

private:
    FormulaProbeAdmittedProduct(bool exact, std::string failure_stage,
        FormulaProbeProduct product, FormulaProbeAdmittedProductWork work,
        std::string context_root, FormulaProbeTangentInputRole input_role,
        std::string input_root, std::string root);

    bool exact_ = false;
    std::string failure_stage_;
    FormulaProbeProduct product_;
    FormulaProbeAdmittedProductWork work_;
    std::string context_root_;
    FormulaProbeTangentInputRole input_role_ =
        FormulaProbeTangentInputRole::Unknown;
    std::string input_root_;
    std::string root_;

    friend FormulaProbeAdmittedProduct formula_probe_admitted_tangent_product(
        const FormulaProbeAdmittedTangent& context,
        FormulaProbeTangentInputRole role,
        const std::vector<FormulaProbeBinary128>& input);
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
    // Optional execution trace.  Legacy dense products leave these empty so
    // their frozen roots remain unchanged; callback-backed products bind the
    // actual wide-kernel applications and the recurrence state that consumed
    // their projected value.
    std::string execution_root;
    std::string high_product_root;
    std::string low_product_root;
    std::string callback_root;
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
    // Explicit untrusted trace payloads for verifier-first correspondence.
    // Legacy state roots intentionally remain unchanged; R63ZK compares these
    // K2 values directly and seals them in its independent trace root.
    FormulaProbeK2Scalar rho;
    FormulaProbeK2Scalar denominator;
    FormulaProbeK2Scalar alpha;
    FormulaProbeK2Scalar beta;
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

struct FormulaProbeHybridWork {
    // R63ZK compares and independently seals these fields.  The legacy R63ZJ
    // hybrid root deliberately omits them so its frozen regression is stable.
    std::size_t fixture_validations = 0U;
    std::size_t identity_derivations = 0U;
    std::size_t operator_products = 0U;
    std::size_t tangent_kernel_calls = 0U;
    std::size_t tangent_inner_dots = 0U;
    std::size_t tangent_outer_dots = 0U;
    std::size_t tangent_inner_terms = 0U;
    std::size_t tangent_outer_terms = 0U;
    std::size_t tangent_scale_products = 0U;
    std::size_t input_components = 0U;
    std::size_t output_projections = 0U;
    std::size_t output_additions = 0U;
    std::size_t certificate_dots = 0U;
    std::size_t certificate_dot_products = 0U;
    std::size_t certificate_radius_terms = 0U;
    std::size_t certificate_solution_dots = 0U;
    std::size_t certificate_sign_comparisons = 0U;
};

struct FormulaProbeHybridRecurrence {
    bool exact = false;
    FormulaProbeTwofoldRecurrence recurrence;
    FormulaProbeHybridWork hybrid_work;
    std::string callback_identity_root;
    std::string callback_root;
    std::string root;
};

struct FormulaProbeTwofoldProductAudit {
    bool exact = false;
    bool complete = false;
    std::size_t containments = 0U;
    std::size_t rows = 0U;
    std::size_t exact_upper_dots = 0U;
    std::size_t exact_gram_products = 0U;
    std::size_t exact_mirrors = 0U;
    std::size_t exact_product_rows = 0U;
    std::size_t exact_product_products = 0U;
    std::vector<double> radius;
    std::string first_failure;
    std::string exact_operator_root;
    std::string containment_root;
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
std::string formula_probe_twofold_product_root(
    const FormulaProbeTwofoldProduct& value);
std::string formula_probe_twofold_recurrence_root(
    const FormulaProbeTwofoldRecurrence& value);
std::string formula_probe_hybrid_product_callback_root(
    const std::string& callback_identity_root,
    const FormulaProbeTwofoldProduct& value);
std::string formula_probe_hybrid_callback_root(
    const FormulaProbeHybridRecurrence& value);
std::string formula_probe_hybrid_twofold_recurrence_root(
    const FormulaProbeHybridRecurrence& value);
std::string formula_probe_hybrid_carrier_root(
    const FormulaProbeParentFixture& fixture);
std::string formula_probe_hybrid_callback_identity_root(
    const FormulaProbeParentFixture& fixture);
FormulaProbeHybridIdentity formula_probe_hybrid_identity(
    const FormulaProbeParentFixture& fixture);
std::string formula_probe_solution_set_root(
    const std::vector<std::vector<FormulaProbeBinary128>>& solutions);
std::string formula_probe_certificate_set_root(
    const std::vector<FormulaProbeCertificate>& certificates);
FormulaProbeProduct formula_probe_tangent_product(
    const FormulaProbeParentFixture& fixture,
    const std::vector<FormulaProbeBinary128>& input);
FormulaProbeProduct formula_probe_tangent_product(
    const FormulaProbeTangentBoundaryFixture& fixture,
    const std::vector<FormulaProbeBinary128>& input);
FormulaProbeTangentAdmission formula_probe_admit_tangent(
    const FormulaProbeTangentBoundaryFixture& fixture);
FormulaProbeAdmittedProduct formula_probe_admitted_tangent_product(
    const FormulaProbeAdmittedTangent& context,
    FormulaProbeTangentInputRole role,
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
FormulaProbeK2Scalar formula_probe_k2_exact_double(double value);
FormulaProbeK2Scalar formula_probe_k2_project(FormulaProbeBinary128 value);
FormulaProbeK2Scalar formula_probe_k2_add(
    const FormulaProbeK2Scalar& left,
    const FormulaProbeK2Scalar& right,
    double right_sign);
FormulaProbeK2Scalar formula_probe_k2_divide(
    const FormulaProbeK2Scalar& numerator,
    const FormulaProbeK2Scalar& denominator);
bool formula_probe_k2_positive(const FormulaProbeK2Scalar& value);
std::string formula_probe_k2_vector_root(
    const std::vector<double>& components);
FormulaProbeK2Solve formula_probe_k2_factor_solve(
    const FormulaProbeParentFixture& fixture,
    const std::string& role,
    const FormulaProbeK2Scalar& inverse_scale,
    const std::vector<double>& source_components);
FormulaProbeK2Dot formula_probe_k2_dot(
    const std::vector<double>& left_components,
    const std::vector<double>& right_components);
std::vector<double> formula_probe_k2_update(
    const std::vector<double>& base_components,
    const FormulaProbeK2Scalar& scale,
    const std::vector<double>& direction_components,
    double direction_sign);
FormulaProbeK2CertificateCheck formula_probe_k2_certificate_check(
    const FormulaProbeParentFixture& fixture,
    const std::vector<double>& solution_components);
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
FormulaProbeHybridRecurrence formula_probe_hybrid_twofold_recurrence(
    const FormulaProbeParentFixture& fixture);
bool formula_probe_hybrid_twofold_recurrence_valid(
    const FormulaProbeParentFixture& fixture,
    const FormulaProbeHybridRecurrence& value);
FormulaProbeTwofoldProductAudit formula_probe_twofold_product_exact_audit(
    const FormulaProbeParentFixture& fixture,
    const FormulaProbeTwofoldRecurrence& recurrence);
bool formula_probe_twofold_product_exact_audit_valid(
    const FormulaProbeTwofoldProductAudit& value);

} // namespace nextengine::nonlocal::fcr
