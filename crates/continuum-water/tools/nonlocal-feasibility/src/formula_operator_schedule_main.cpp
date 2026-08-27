#include "formula_probe_cache.hpp"
#include "sha256.hpp"

#include <algorithm>
#include <array>
#include <cstddef>
#include <exception>
#include <iostream>
#include <limits>
#include <quadmath.h>
#include <sstream>
#include <string>
#include <vector>

namespace {

using nextengine::nonlocal::fcr::FormulaProbeBinary128;
using nextengine::nonlocal::fcr::FormulaProbeCertificate;
using nextengine::nonlocal::fcr::FormulaProbeParentFixture;
using nextengine::nonlocal::fcr::FormulaProbeProduct;
using nextengine::nonlocal::fcr::FormulaProbeScalar;
using nextengine::nonlocal::fcr::FormulaProbeSolve;

constexpr std::size_t DIMENSION = 102U;

struct Work {
    std::size_t certificates = 0U;
    std::size_t products = 0U;
    std::size_t solves = 0U;
    std::size_t factor_terms = 0U;
    std::size_t factor_divisions = 0U;
    std::size_t scalar_dots = 0U;
    std::size_t scalar_divisions = 0U;
    std::size_t solution_updates = 0U;
    std::size_t residual_updates = 0U;
    std::size_t direction_updates = 0U;
    std::size_t tangent_inner_dots = 0U;
    std::size_t tangent_outer_dots = 0U;
    std::size_t tangent_inner_terms = 0U;
    std::size_t tangent_outer_terms = 0U;
    std::size_t tangent_scale_products = 0U;
    std::size_t common_dots = 0U;
    std::size_t common_terms = 0U;
    std::size_t adaptive_stops = 0U;
};

void add_work(Work& target, const Work& source) {
    target.certificates += source.certificates;
    target.products += source.products;
    target.solves += source.solves;
    target.factor_terms += source.factor_terms;
    target.factor_divisions += source.factor_divisions;
    target.scalar_dots += source.scalar_dots;
    target.scalar_divisions += source.scalar_divisions;
    target.solution_updates += source.solution_updates;
    target.residual_updates += source.residual_updates;
    target.direction_updates += source.direction_updates;
    target.tangent_inner_dots += source.tangent_inner_dots;
    target.tangent_outer_dots += source.tangent_outer_dots;
    target.tangent_inner_terms += source.tangent_inner_terms;
    target.tangent_outer_terms += source.tangent_outer_terms;
    target.tangent_scale_products += source.tangent_scale_products;
    target.common_dots += source.common_dots;
    target.common_terms += source.common_terms;
    target.adaptive_stops += source.adaptive_stops;
}

std::string work_root(const Work& work) {
    std::ostringstream material;
    material << work.certificates << ':' << work.products << ':'
        << work.solves << ':' << work.factor_terms << ':'
        << work.factor_divisions << ':' << work.scalar_dots << ':'
        << work.scalar_divisions << ':' << work.solution_updates << ':'
        << work.residual_updates << ':' << work.direction_updates << ':'
        << work.tangent_inner_dots << ':' << work.tangent_outer_dots << ':'
        << work.tangent_inner_terms << ':' << work.tangent_outer_terms << ':'
        << work.tangent_scale_products << ':' << work.common_dots << ':'
        << work.common_terms << ':' << work.adaptive_stops;
    return nextengine::nonlocal::sha256_hex(material.str());
}

bool work_equal(const Work& left, const Work& right) {
    return work_root(left) == work_root(right);
}

bool finite_vector(const std::vector<FormulaProbeBinary128>& values) {
    return !values.empty()
        && std::all_of(values.begin(), values.end(), [](auto value) {
            return finiteq(value) != 0;
        });
}

bool normal_or_zero(FormulaProbeBinary128 value) {
    return finiteq(value) != 0
        && (value == static_cast<FormulaProbeBinary128>(0.0)
            || fabsq(value) >= ldexpq(
                static_cast<FormulaProbeBinary128>(1.0), -16382));
}

bool normal_vector(const std::vector<FormulaProbeBinary128>& values) {
    return !values.empty()
        && std::all_of(values.begin(), values.end(), normal_or_zero);
}

bool two_term_update(FormulaProbeBinary128 left0,
    FormulaProbeBinary128 left1, FormulaProbeBinary128 right0,
    FormulaProbeBinary128 right1, FormulaProbeBinary128& value) {
    const FormulaProbeScalar update =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(
            {left0, left1}, {right0, right1});
    value = update.value;
    return update.exact && normal_or_zero(value);
}

struct ProductResult {
    bool exact = false;
    bool common = false;
    std::vector<FormulaProbeBinary128> value;
    Work work;
    std::string root;
};

ProductResult apply_product(const FormulaProbeParentFixture& fixture,
    bool common, const std::vector<FormulaProbeBinary128>& input) {
    ProductResult result;
    result.common = common;
    const FormulaProbeProduct product = common
        ? nextengine::nonlocal::fcr::formula_probe_common_product(
            fixture, input)
        : nextengine::nonlocal::fcr::formula_probe_tangent_product(
            fixture, input);
    result.exact = product.exact && finite_vector(product.value);
    result.value = product.value;
    result.work.products = 1U;
    if (common) {
        result.work.common_dots = product.inner_dots;
        result.work.common_terms = product.inner_terms;
    } else {
        result.work.tangent_inner_dots = product.inner_dots;
        result.work.tangent_outer_dots = product.outer_dots;
        result.work.tangent_inner_terms = product.inner_terms;
        result.work.tangent_outer_terms = product.outer_terms;
        result.work.tangent_scale_products = product.scale_products;
    }
    result.root = nextengine::nonlocal::sha256_hex(
        std::string(common ? "common:" : "tangent:") + product.root);
    return result;
}

void count_solve(Work& work, const FormulaProbeSolve& solve) {
    ++work.solves;
    work.factor_terms += solve.forward_terms + solve.backward_terms;
    work.factor_divisions += solve.divisions;
}

struct State {
    bool exact = false;
    std::size_t index = 0U;
    std::vector<FormulaProbeBinary128> solution;
    std::vector<FormulaProbeBinary128> residual;
    std::vector<FormulaProbeBinary128> direction;
    std::vector<FormulaProbeBinary128> preconditioned;
    std::string product_root;
    std::string solve_root;
    std::string scalar_root;
    std::string root;
};

State seal_state(std::size_t index,
    const std::vector<FormulaProbeBinary128>& solution,
    const std::vector<FormulaProbeBinary128>& residual,
    const std::vector<FormulaProbeBinary128>& direction,
    const std::vector<FormulaProbeBinary128>& preconditioned,
    const std::string& product_root, const std::string& solve_root,
    const std::string& scalar_root) {
    State result;
    result.index = index;
    result.solution = solution;
    result.residual = residual;
    result.direction = direction;
    result.preconditioned = preconditioned;
    result.product_root = product_root;
    result.solve_root = solve_root;
    result.scalar_root = scalar_root;
    result.exact = solution.size() == DIMENSION
        && residual.size() == DIMENSION && direction.size() == DIMENSION
        && preconditioned.size() == DIMENSION && normal_vector(solution)
        && normal_vector(residual) && normal_vector(direction)
        && normal_vector(preconditioned) && !product_root.empty()
        && !solve_root.empty() && !scalar_root.empty();
    std::ostringstream material;
    material << result.exact << ':' << result.index << ':'
        << nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(
            result.solution)
        << ':'
        << nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(
            result.residual)
        << ':'
        << nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(
            result.direction)
        << ':'
        << nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(
            result.preconditioned)
        << ':' << result.product_root << ':' << result.solve_root << ':'
        << result.scalar_root;
    result.root = nextengine::nonlocal::sha256_hex(material.str());
    return result;
}

struct Transaction {
    bool exact = false;
    bool complete = false;
    std::size_t mask = 0U;
    std::string failure;
    std::string input_root;
    Work work;
    std::vector<State> states;
    std::vector<std::string> product_roots;
    std::vector<std::string> solve_roots;
    std::vector<std::string> scalar_roots;
    std::string root;
};

std::string transaction_root(const Transaction& transaction) {
    std::ostringstream material;
    material << transaction.exact << ':' << transaction.complete << ':'
        << transaction.mask << ':' << transaction.failure << ':'
        << transaction.input_root << ':'
        << work_root(transaction.work) << '|';
    for (const auto& root : transaction.product_roots)
        material << "product:" << root << ';';
    for (const auto& root : transaction.solve_roots)
        material << "solve:" << root << ';';
    for (const auto& root : transaction.scalar_roots)
        material << "scalar:" << root << ';';
    for (const auto& state : transaction.states)
        material << "state:" << state.root << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

Transaction run_transaction(
    const FormulaProbeParentFixture& fixture, std::size_t mask,
    const std::vector<FormulaProbeBinary128>& rhs) {
    Transaction result;
    result.mask = mask;
    result.input_root =
        nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(rhs);
    const auto fail = [&](const std::string& stage) {
        result.failure = stage;
        result.root = transaction_root(result);
        return result;
    };
    if (mask > 7U || fixture.dimension != DIMENSION
        || rhs.size() != DIMENSION || !normal_vector(rhs))
        return fail("identity");

    const FormulaProbeSolve start =
        nextengine::nonlocal::fcr::formula_probe_factor_solve(
            fixture, rhs, fixture.inverse_scale);
    count_solve(result.work, start);
    result.solve_roots.push_back(start.root);
    if (!start.exact || !finite_vector(start.solution))
        return fail("initial_solve");

    const ProductResult hx0 = apply_product(
        fixture, (mask & 1U) != 0U, start.solution);
    add_work(result.work, hx0.work);
    result.product_roots.push_back(hx0.root);
    if (!hx0.exact) return fail("initial_product");

    std::vector<FormulaProbeBinary128> r0(DIMENSION);
    bool update_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        update_exact = two_term_update(rhs[index],
            -hx0.value[index], static_cast<FormulaProbeBinary128>(1.0),
            static_cast<FormulaProbeBinary128>(1.0), r0[index])
            && update_exact;
    }
    if (!update_exact || !normal_vector(r0))
        return fail("initial_residual");
    const FormulaProbeSolve z0 =
        nextengine::nonlocal::fcr::formula_probe_factor_solve(
            fixture, r0, fixture.inverse_scale);
    count_solve(result.work, z0);
    result.solve_roots.push_back(z0.root);
    if (!z0.exact || !finite_vector(z0.solution))
        return fail("initial_preconditioner");
    const FormulaProbeScalar rho0 =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(r0, z0.solution);
    ++result.work.scalar_dots;
    result.scalar_roots.push_back(rho0.root);
    if (!rho0.exact || !rho0.positive) return fail("initial_rho");
    const std::vector<FormulaProbeBinary128> p0 = z0.solution;
    const State state0 = seal_state(0U, start.solution, r0, p0,
        z0.solution, hx0.root, z0.root, rho0.root);
    if (!state0.exact) return fail("state0");

    const ProductResult hp0 = apply_product(
        fixture, (mask & 2U) != 0U, p0);
    add_work(result.work, hp0.work);
    result.product_roots.push_back(hp0.root);
    if (!hp0.exact) return fail("iteration0_product");
    const FormulaProbeScalar denominator0 =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(p0, hp0.value);
    ++result.work.scalar_dots;
    result.scalar_roots.push_back(denominator0.root);
    if (!denominator0.exact || !denominator0.positive)
        return fail("iteration0_denominator");
    const FormulaProbeBinary128 alpha0 = rho0.value / denominator0.value;
    ++result.work.scalar_divisions;
    if (!normal_or_zero(alpha0)) return fail("iteration0_alpha");
    std::vector<FormulaProbeBinary128> x1(DIMENSION);
    std::vector<FormulaProbeBinary128> r1(DIMENSION);
    update_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        update_exact = two_term_update(start.solution[index], alpha0,
            static_cast<FormulaProbeBinary128>(1.0), p0[index], x1[index])
            && update_exact;
        update_exact = two_term_update(r0[index], -alpha0,
            static_cast<FormulaProbeBinary128>(1.0), hp0.value[index],
            r1[index]) && update_exact;
        ++result.work.solution_updates;
        ++result.work.residual_updates;
    }
    if (!update_exact || !normal_vector(x1) || !normal_vector(r1))
        return fail("iteration0_update");
    const FormulaProbeSolve z1 =
        nextengine::nonlocal::fcr::formula_probe_factor_solve(
            fixture, r1, fixture.inverse_scale);
    count_solve(result.work, z1);
    result.solve_roots.push_back(z1.root);
    if (!z1.exact || !finite_vector(z1.solution))
        return fail("iteration1_preconditioner");
    const FormulaProbeScalar rho1 =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(r1, z1.solution);
    ++result.work.scalar_dots;
    result.scalar_roots.push_back(rho1.root);
    if (!rho1.exact || !rho1.positive) return fail("iteration1_rho");
    const FormulaProbeBinary128 beta1 = rho1.value / rho0.value;
    ++result.work.scalar_divisions;
    if (!normal_or_zero(beta1)) return fail("iteration1_beta");
    std::vector<FormulaProbeBinary128> p1(DIMENSION);
    update_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        update_exact = two_term_update(z1.solution[index], beta1,
            static_cast<FormulaProbeBinary128>(1.0), p0[index], p1[index])
            && update_exact;
        ++result.work.direction_updates;
    }
    if (!update_exact || !normal_vector(p1))
        return fail("iteration1_direction");
    const State state1 = seal_state(1U, x1, r1, p1, z1.solution,
        hp0.root, z1.root, rho1.root);
    if (!state1.exact) return fail("state1");

    const ProductResult hp1 = apply_product(
        fixture, (mask & 4U) != 0U, p1);
    add_work(result.work, hp1.work);
    result.product_roots.push_back(hp1.root);
    if (!hp1.exact) return fail("iteration1_product");
    const FormulaProbeScalar denominator1 =
        nextengine::nonlocal::fcr::formula_probe_scalar_dot(p1, hp1.value);
    ++result.work.scalar_dots;
    result.scalar_roots.push_back(denominator1.root);
    if (!denominator1.exact || !denominator1.positive)
        return fail("iteration1_denominator");
    const FormulaProbeBinary128 alpha1 = rho1.value / denominator1.value;
    ++result.work.scalar_divisions;
    if (!normal_or_zero(alpha1)) return fail("iteration1_alpha");
    std::vector<FormulaProbeBinary128> x2(DIMENSION);
    std::vector<FormulaProbeBinary128> r2(DIMENSION);
    update_exact = true;
    for (std::size_t index = 0U; index < DIMENSION; ++index) {
        update_exact = two_term_update(x1[index], alpha1,
            static_cast<FormulaProbeBinary128>(1.0), p1[index], x2[index])
            && update_exact;
        update_exact = two_term_update(r1[index], -alpha1,
            static_cast<FormulaProbeBinary128>(1.0), hp1.value[index],
            r2[index]) && update_exact;
        ++result.work.solution_updates;
        ++result.work.residual_updates;
    }
    if (!update_exact || !normal_vector(x2) || !normal_vector(r2))
        return fail("iteration1_update");
    const State state2 = seal_state(2U, x2, r2, p1, z1.solution,
        hp1.root, z1.root, denominator1.root);
    if (!state2.exact) return fail("state2");

    result.states = {state0, state1, state2};
    result.complete = true;
    result.exact = result.product_roots.size() == 3U
        && result.solve_roots.size() == 3U
        && result.scalar_roots.size() == 4U
        && result.states.size() == 3U;
    result.root = transaction_root(result);
    return result;
}

Transaction run_finite_overflow_transaction() {
    Transaction result;
    result.mask = 8U;
    const FormulaProbeBinary128 finite_max = strtoflt128(
        "1.18973149535723176508575932662800702e4932", nullptr);
    const std::vector<FormulaProbeBinary128> input{finite_max};
    const std::vector<FormulaProbeBinary128> matrix{finite_max};
    result.input_root =
        nextengine::nonlocal::fcr::formula_probe_binary128_vector_root(input);
    const FormulaProbeProduct product =
        nextengine::nonlocal::fcr::formula_probe_control_dense_product(
            matrix, input);
    result.work.products = 1U;
    result.work.common_dots = product.inner_dots;
    result.work.common_terms = product.inner_terms;
    result.product_roots.push_back(nextengine::nonlocal::sha256_hex(
        std::string("control_dense:") + product.root));
    result.failure = product.exact
        ? "finite_overflow_not_rejected" : "initial_product";
    result.exact = product.exact;
    result.root = transaction_root(result);
    return result;
}

struct Lane {
    bool exact = false;
    bool passed = false;
    std::size_t mask = 0U;
    Transaction transaction;
    std::vector<FormulaProbeCertificate> certificates;
    std::string root;
};

std::string lane_root(const Lane& lane) {
    std::ostringstream material;
    material << lane.exact << ':' << lane.passed << ':' << lane.mask << ':'
        << lane.transaction.root << ':';
    for (const auto& certificate : lane.certificates)
        material << certificate.root << ';';
    return nextengine::nonlocal::sha256_hex(material.str());
}

Lane run_lane(const FormulaProbeParentFixture& fixture, std::size_t mask) {
    Lane result;
    result.mask = mask;
    result.transaction = run_transaction(
        fixture, mask, fixture.original_rhs);
    if (result.transaction.complete) {
        for (const auto& state : result.transaction.states) {
            result.certificates.push_back(
                nextengine::nonlocal::fcr::formula_probe_certificate(
                    fixture, state.solution));
            ++result.transaction.work.certificates;
        }
        result.transaction.root = transaction_root(result.transaction);
    }
    result.passed = result.certificates.size() == 3U
        && !result.certificates[0U].passed
        && !result.certificates[1U].passed
        && result.certificates[2U].passed;
    result.exact = result.transaction.exact && result.transaction.complete
        && result.certificates.size() == 3U
        && std::all_of(result.certificates.begin(),
            result.certificates.end(), [](const auto& certificate) {
                return certificate.exact;
            });
    result.root = lane_root(result);
    return result;
}

std::vector<std::size_t> minimal_rejects(
    const std::array<bool, 8U>& passes) {
    std::vector<std::size_t> result;
    for (std::size_t mask = 1U; mask < 8U; ++mask) {
        if (passes[mask]) continue;
        bool has_rejecting_subset = false;
        for (std::size_t subset = (mask - 1U) & mask; subset != 0U;
             subset = (subset - 1U) & mask) {
            has_rejecting_subset = has_rejecting_subset || !passes[subset];
        }
        if (!has_rejecting_subset) result.push_back(mask);
    }
    return result;
}

bool classifier_control() {
    for (std::size_t pattern = 0U; pattern < 256U; ++pattern) {
        std::array<bool, 8U> passes{};
        for (std::size_t mask = 0U; mask < 8U; ++mask)
            passes[mask] = (pattern & (1U << mask)) != 0U;
        if (!passes[0U] || passes[7U]) continue;
        const std::vector<std::size_t> minimal = minimal_rejects(passes);
        if (minimal.empty()) return false;
        for (std::size_t mask : minimal) {
            if (mask == 0U || mask > 7U || passes[mask]) return false;
            for (std::size_t subset = (mask - 1U) & mask; subset != 0U;
                 subset = (subset - 1U) & mask) {
                if (!passes[subset]) return false;
            }
        }
        for (std::size_t mask = 1U; mask < 8U; ++mask) {
            if (passes[mask]) continue;
            bool covered = false;
            for (std::size_t minimal_mask : minimal)
                covered = covered
                    || (minimal_mask & mask) == minimal_mask;
            if (!covered) return false;
        }
    }
    return true;
}

std::string classify(bool apparatus, bool identity, bool work,
    bool tangent_endpoint, bool common_endpoint,
    const std::vector<std::size_t>& minimal) {
    if (!apparatus) return "OPERATOR_USE_SCHEDULE_APPARATUS_REJECTED";
    if (!identity) return "OPERATOR_USE_SCHEDULE_IDENTITY_REJECTED";
    if (!work) return "OPERATOR_USE_SCHEDULE_WORK_REJECTED";
    if (!tangent_endpoint) return "TANGENT_ENDPOINT_REJECTED";
    if (!common_endpoint) return "COMMON_ENDPOINT_REJECTED";
    if (std::find(minimal.begin(), minimal.end(), 1U) != minimal.end())
        return "INITIAL_PRODUCT_MINIMAL_SUFFICIENT";
    if (std::find(minimal.begin(), minimal.end(), 2U) != minimal.end())
        return "FIRST_KRYLOV_PRODUCT_MINIMAL_SUFFICIENT";
    if (std::find(minimal.begin(), minimal.end(), 4U) != minimal.end())
        return "SECOND_KRYLOV_PRODUCT_MINIMAL_SUFFICIENT";
    if (minimal.size() > 1U) return "MULTIPLE_MINIMAL_SCHEDULES";
    if (minimal.size() == 1U
        && (minimal[0U] == 3U || minimal[0U] == 5U
            || minimal[0U] == 6U))
        return "PAIRWISE_OPERATOR_INTERACTION_REQUIRED";
    if (minimal.size() == 1U && minimal[0U] == 7U)
        return "THREE_PRODUCT_INTERACTION_REQUIRED";
    return "OPERATOR_USE_SCHEDULE_APPARATUS_REJECTED";
}

std::string schedule_name(std::size_t mask) {
    std::string result;
    for (std::size_t slot = 0U; slot < 3U; ++slot)
        result.push_back((mask & (1U << slot)) != 0U ? 'C' : 'T');
    return result;
}

} // namespace

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-operator-schedule "
                         "<parent-fixture-cache>\n";
            return 2;
        }
        using namespace nextengine::nonlocal::fcr;
        const FormulaProbeParentFixture fixture =
            read_formula_probe_parent_fixture(argv[1]);
        const bool identity = formula_probe_parent_fixture_valid(fixture);

        std::array<Lane, 8U> lanes;
        std::array<bool, 8U> passes{};
        Work aggregate;
        bool apparatus = identity;
        for (std::size_t mask = 0U; mask < lanes.size(); ++mask) {
            lanes[mask] = run_lane(fixture, mask);
            passes[mask] = lanes[mask].passed;
            add_work(aggregate, lanes[mask].transaction.work);
            apparatus = apparatus && lanes[mask].exact
                && lanes[mask].mask == mask;
        }

        const Work expected{24U, 24U, 24U, 247248U, 4896U, 32U, 24U,
            1632U, 1632U, 816U, 3780U, 1224U, 385560U, 385560U,
            1224U, 1224U, 124848U, 0U};
        const bool work = work_equal(aggregate, expected);
        const bool tangent_endpoint = lanes[0U].passed
            && lanes[0U].transaction.states.size() == 3U
            && formula_probe_solution_set_root({
                lanes[0U].transaction.states[0U].solution,
                lanes[0U].transaction.states[1U].solution,
                lanes[0U].transaction.states[2U].solution})
                == fixture.r63x_solution_set_root;
        const std::array<std::string, 3U> common_roots{
            "11170afba73f379a19c02f4a9a1514944cc8f9f18bf7c79872d77abc3f478292",
            "7d5ee1b8c6c9f2a9496e52ddaa5d08dcc2ccf7f419ec8d30c690ee1ccf103bb5",
            "1341b62cb0d8ab3e561682b1560e951d3e7c4d5faa4684ffd7c48db3838cdb12"};
        bool common_endpoint = !lanes[7U].passed
            && lanes[7U].transaction.states.size() == 3U
            && lanes[7U].certificates.size() == 3U;
        if (common_endpoint) {
            for (std::size_t state = 0U; state < 3U; ++state) {
                const auto& certificate = lanes[7U].certificates[state];
                common_endpoint = common_endpoint
                    && formula_probe_binary128_vector_root(
                        lanes[7U].transaction.states[state].solution)
                        == common_roots[state]
                    && !certificate.passed && certificate.positive == 12U
                    && certificate.negative == 24U
                    && certificate.unresolved == 66U;
            }
        }

        const std::vector<std::size_t> minimal = minimal_rejects(passes);
        const bool classifier = classifier_control();
        const Lane original_lane = lanes[0U];
        bool certificate_isolation = false;
        if (original_lane.certificates.size() == 3U) {
            Lane certificate_mutation = original_lane;
            certificate_mutation.certificates[2U].root =
                nextengine::nonlocal::sha256_hex(
                    certificate_mutation.certificates[2U].root
                        + ":mutation");
            certificate_mutation.root = lane_root(certificate_mutation);
            certificate_isolation =
                certificate_mutation.transaction.root
                    == original_lane.transaction.root
                && certificate_mutation.root != original_lane.root;
        }

        std::vector<FormulaProbeBinary128> invalid(DIMENSION - 1U,
            static_cast<FormulaProbeBinary128>(1.0));
        const Transaction invalid_transaction =
            run_transaction(fixture, 0U, invalid);
        const Work no_work;
        const bool invalid_control = !invalid_transaction.exact
            && !invalid_transaction.complete
            && invalid_transaction.failure == "identity"
            && invalid_transaction.states.empty()
            && invalid_transaction.product_roots.empty()
            && invalid_transaction.solve_roots.empty()
            && invalid_transaction.scalar_roots.empty()
            && work_equal(invalid_transaction.work, no_work);

        const Transaction overflow_transaction =
            run_finite_overflow_transaction();
        Work overflow_work;
        overflow_work.products = 1U;
        overflow_work.common_dots = 1U;
        overflow_work.common_terms = 1U;
        const bool overflow_control = !overflow_transaction.exact
            && !overflow_transaction.complete
            && overflow_transaction.failure == "initial_product"
            && overflow_transaction.states.empty()
            && work_equal(overflow_transaction.work, overflow_work);

        std::vector<FormulaProbeBinary128> zero(DIMENSION,
            static_cast<FormulaProbeBinary128>(0.0));
        const Transaction nonpositive_transaction =
            run_transaction(fixture, 1U, zero);
        Work nonpositive_work;
        nonpositive_work.products = 1U;
        nonpositive_work.solves = 2U;
        nonpositive_work.factor_terms = 20604U;
        nonpositive_work.factor_divisions = 408U;
        nonpositive_work.scalar_dots = 1U;
        nonpositive_work.common_dots = 102U;
        nonpositive_work.common_terms = 10404U;
        const bool nonpositive_control = !nonpositive_transaction.exact
            && !nonpositive_transaction.complete
            && nonpositive_transaction.failure == "initial_rho"
            && nonpositive_transaction.states.empty()
            && work_equal(nonpositive_transaction.work, nonpositive_work);
        const bool controls = classifier && certificate_isolation
            && invalid_control && overflow_control
            && nonpositive_control;

        std::ostringstream control_material;
        control_material << classifier << ':' << certificate_isolation << ':'
            << invalid_control << ':' << invalid_transaction.root << ':'
            << overflow_control << ':' << overflow_transaction.root << ':'
            << nonpositive_control << ':' << nonpositive_transaction.root;
        const std::string controls_root =
            nextengine::nonlocal::sha256_hex(control_material.str());

        const std::string route = classify(apparatus && controls, identity,
            work, tangent_endpoint, common_endpoint, minimal);
        const bool exact = apparatus && identity && work && tangent_endpoint
            && common_endpoint && controls && !minimal.empty()
            && route != "OPERATOR_USE_SCHEDULE_APPARATUS_REJECTED"
            && route != "OPERATOR_USE_SCHEDULE_IDENTITY_REJECTED"
            && route != "OPERATOR_USE_SCHEDULE_WORK_REJECTED"
            && route != "TANGENT_ENDPOINT_REJECTED"
            && route != "COMMON_ENDPOINT_REJECTED";

        std::ostringstream semantic;
        semantic << route << ':' << fixture.root << ':' << work_root(aggregate)
            << ':' << controls_root << ':';
        for (std::size_t mask = 0U; mask < 8U; ++mask)
            semantic << mask << ':' << passes[mask] << ':'
                << lanes[mask].root << ';';
        semantic << '|';
        for (std::size_t mask : minimal) semantic << mask << ';';
        const std::string result_root =
            nextengine::nonlocal::sha256_hex(semantic.str());

        std::cout
            << "{\"schema\":\"nextengine.nonlocal.r63zd_operator_use_schedule.v2\""
            << ",\"status\":\"" << (exact ? "PASS" : "FAIL") << "\""
            << ",\"route\":\"" << route << "\""
            << ",\"fixture_root\":\"" << fixture.root << "\""
            << ",\"work_root\":\"" << work_root(aggregate) << "\""
            << ",\"minimal_masks\":[";
        for (std::size_t index = 0U; index < minimal.size(); ++index) {
            if (index != 0U) std::cout << ',';
            std::cout << minimal[index];
        }
        std::cout << "],\"lanes\":[";
        for (std::size_t mask = 0U; mask < lanes.size(); ++mask) {
            if (mask != 0U) std::cout << ',';
            std::cout << "{\"mask\":" << mask << ",\"schedule\":\""
                << schedule_name(mask) << "\",\"exact\":"
                << (lanes[mask].exact ? "true" : "false")
                << ",\"passed\":"
                << (lanes[mask].passed ? "true" : "false")
                << ",\"transaction_root\":\""
                << lanes[mask].transaction.root << "\",\"states\":[";
            const std::size_t state_count = std::min(
                lanes[mask].transaction.states.size(),
                lanes[mask].certificates.size());
            for (std::size_t state = 0U; state < state_count; ++state) {
                if (state != 0U) std::cout << ',';
                const auto& certificate = lanes[mask].certificates[state];
                std::cout << "{\"state\":" << state
                    << ",\"solution_root\":\""
                    << formula_probe_binary128_vector_root(
                        lanes[mask].transaction.states[state].solution)
                    << "\",\"passed\":"
                    << (certificate.passed ? "true" : "false")
                    << ",\"positive\":" << certificate.positive
                    << ",\"negative\":" << certificate.negative
                    << ",\"unresolved\":" << certificate.unresolved
                    << ",\"certificate_root\":\""
                    << certificate.root << "\"}";
            }
            std::cout << "],\"root\":\"" << lanes[mask].root << "\"}";
        }
        std::cout << "],\"controls\":{\"classifier\":"
            << (classifier ? "true" : "false")
            << ",\"certificate_isolation\":"
            << (certificate_isolation ? "true" : "false")
            << ",\"invalid_transaction\":{\"passed\":"
            << (invalid_control ? "true" : "false")
            << ",\"failure\":\"" << invalid_transaction.failure
            << "\",\"work_root\":\""
            << work_root(invalid_transaction.work) << "\",\"root\":\""
            << invalid_transaction.root << "\"}"
            << ",\"overflow_transaction\":{\"passed\":"
            << (overflow_control ? "true" : "false")
            << ",\"failure\":\"" << overflow_transaction.failure
            << "\",\"work_root\":\""
            << work_root(overflow_transaction.work) << "\",\"root\":\""
            << overflow_transaction.root << "\"}"
            << ",\"nonpositive_transaction\":{\"passed\":"
            << (nonpositive_control ? "true" : "false")
            << ",\"failure\":\"" << nonpositive_transaction.failure
            << "\",\"work_root\":\""
            << work_root(nonpositive_transaction.work) << "\",\"root\":\""
            << nonpositive_transaction.root << "\"}"
            << ",\"root\":\"" << controls_root << "\"}"
            << ",\"work\":{\"certificates\":" << aggregate.certificates
            << ",\"products\":" << aggregate.products
            << ",\"solves\":" << aggregate.solves
            << ",\"factor_terms\":" << aggregate.factor_terms
            << ",\"factor_divisions\":" << aggregate.factor_divisions
            << ",\"scalar_dots\":" << aggregate.scalar_dots
            << ",\"scalar_divisions\":" << aggregate.scalar_divisions
            << ",\"solution_updates\":" << aggregate.solution_updates
            << ",\"residual_updates\":" << aggregate.residual_updates
            << ",\"direction_updates\":" << aggregate.direction_updates
            << ",\"tangent_inner_dots\":"
            << aggregate.tangent_inner_dots
            << ",\"tangent_outer_dots\":"
            << aggregate.tangent_outer_dots
            << ",\"tangent_inner_terms\":"
            << aggregate.tangent_inner_terms
            << ",\"tangent_outer_terms\":"
            << aggregate.tangent_outer_terms
            << ",\"tangent_scale_products\":"
            << aggregate.tangent_scale_products
            << ",\"common_dots\":" << aggregate.common_dots
            << ",\"common_terms\":" << aggregate.common_terms
            << ",\"adaptive_stops\":" << aggregate.adaptive_stops << "}"
            << ",\"timing_admitted\":false"
            << ",\"runtime_authority\":false"
            << ",\"gpu_authority\":false"
            << ",\"production_authority\":false"
            << ",\"result_sha256\":\"" << result_root << "\"}"
            << '\n';
        return exact ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-operator-schedule: "
                  << error.what() << '\n';
        return 2;
    }
}
