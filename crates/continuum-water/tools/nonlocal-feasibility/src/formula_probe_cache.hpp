#pragma once

#include "formula_probe_api.hpp"

#include <cstddef>
#include <optional>
#include <string>

namespace nextengine::nonlocal::fcr {

void write_formula_probe_parent_fixture(
    const std::string& path, const FormulaProbeParentFixture& fixture);
FormulaProbeParentFixture read_formula_probe_parent_fixture(
    const std::string& path);

struct FormulaProbeTangentBoundaryReadWork {
    std::size_t header_predicate_checks = 0U;
    std::size_t projection_predicate_checks = 0U;
    std::size_t bounded_length_reads = 0U;
    std::size_t selected_size_fields = 0U;
    std::size_t selected_scalar_fields = 0U;
    std::size_t selected_string_fields = 0U;
    std::size_t selected_string_bytes = 0U;
    std::size_t selected_vector_fields = 0U;
    std::size_t selected_vector_allocations = 0U;
    std::size_t selected_vector_components = 0U;
    std::size_t skipped_boolean_fields = 0U;
    std::size_t skipped_scalar_fields = 0U;
    std::size_t skipped_string_fields = 0U;
    std::size_t skipped_vector_fields = 0U;
    std::size_t skipped_collection_elements = 0U;
    std::size_t skipped_payload_bytes = 0U;
    std::size_t eof_checks = 0U;
    std::size_t receipt_root_derivations = 0U;
    std::size_t result_root_derivations = 0U;
    std::string root;
};

struct FormulaProbeTangentBoundaryRead {
    bool exact = false;
    std::string failure_stage;
    std::optional<FormulaProbeTangentBoundaryFixture> fixture;
    FormulaProbeTangentBoundaryReadWork work;
    std::string root;
};

FormulaProbeTangentBoundaryRead read_formula_probe_tangent_boundary_fixture(
    const std::string& path);

} // namespace nextengine::nonlocal::fcr
