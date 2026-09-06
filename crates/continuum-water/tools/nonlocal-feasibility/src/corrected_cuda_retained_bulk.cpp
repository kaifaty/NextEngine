#include "corrected_cuda_retained_bulk.hpp"

// NCGP7 is frozen. Include its exact evaluator in this new translation unit so
// NCGP8 can execute the retained bulk observer without modifying the NCGP7
// target or its source identity.
#include "corrected_cuda_eulerian_diagnostic.cpp"

namespace nextengine::nonlocal::ncgp8 {
namespace {

RetainedBulkWork copy_work(const EulerianWork& source) {
    return {source.samples_validated, source.cic_records_emitted,
        source.cic_records_sorted, source.cic_records_reduced,
        source.column_records_emitted, source.column_records_sorted,
        source.column_records_reduced, source.quantile_reads,
        source.node_comparisons, source.column_comparisons,
        source.root_derivations};
}

RetainedBulkFieldReceipt copy_field(const EulerianField& source) {
    RetainedBulkFieldReceipt result;
    result.valid = source.valid;
    result.work = copy_work(source.work);
    result.node_root = source.node_root;
    result.surface_root = source.surface_root;
    result.work_root = source.work_root;
    result.field_root = source.field_root;
    return result;
}

RetainedBulkComparisonReceipt copy_comparison(
    const EulerianComparison& source) {
    RetainedBulkComparisonReceipt result;
    result.valid = source.valid;
    result.mass_tv = source.mass_tv;
    result.density_rmse = source.density_rmse;
    result.velocity_rmse_normalized = source.velocity_rmse_normalized;
    result.center_of_mass_error = source.center_of_mass_error;
    result.maximum_node_mass_difference_particles =
        source.maximum_node_mass_difference_particles;
    result.wet_column_symmetric_difference =
        source.wet_column_symmetric_difference;
    result.surface_rmse = source.surface_rmse;
    result.surface_p95 = source.surface_p95;
    result.surface_maximum = source.surface_maximum;
    result.common_nodes = source.common_nodes;
    result.common_wet_columns = source.common_wet_columns;
    result.union_wet_columns = source.union_wet_columns;
    result.work = copy_work(source.work);
    result.work_root = source.work_root;
    result.metrics_root = source.metrics_root;
    return result;
}

bool bulk_band(const EulerianComparison& value) {
    return value.valid && value.mass_tv <= 0.01
        && value.density_rmse <= 0.01
        && value.velocity_rmse_normalized <= 0.01
        && value.center_of_mass_error <= 0.0025;
}

void append_work(std::ostringstream& material, const RetainedBulkWork& work) {
    material << work.samples_validated << ':' << work.cic_records_emitted << ':'
             << work.cic_records_sorted << ':' << work.cic_records_reduced << ':'
             << work.column_records_emitted << ':'
             << work.column_records_sorted << ':'
             << work.column_records_reduced << ':' << work.quantile_reads << ':'
             << work.node_comparisons << ':' << work.column_comparisons << ':'
             << work.root_derivations << '\n';
}

void append_field(
    std::ostringstream& material, const RetainedBulkFieldReceipt& field) {
    material << field.valid << ':' << field.node_root << ':'
             << field.surface_root << ':' << field.work_root << ':'
             << field.field_root << '\n';
    append_work(material, field.work);
}

void append_comparison(std::ostringstream& material,
    const RetainedBulkComparisonReceipt& comparison) {
    material << comparison.valid << ':' << std::hex
             << bits(comparison.mass_tv) << ':'
             << bits(comparison.density_rmse) << ':'
             << bits(comparison.velocity_rmse_normalized) << ':'
             << bits(comparison.center_of_mass_error) << ':'
             << bits(comparison.maximum_node_mass_difference_particles) << ':'
             << bits(comparison.wet_column_symmetric_difference) << ':'
             << bits(comparison.surface_rmse) << ':'
             << bits(comparison.surface_p95) << ':'
             << bits(comparison.surface_maximum) << std::dec << ':'
             << comparison.common_nodes << ':'
             << comparison.common_wet_columns << ':'
             << comparison.union_wet_columns << ':' << comparison.work_root
             << ':' << comparison.metrics_root << '\n';
    append_work(material, comparison.work);
}

}  // namespace

RetainedBulkClosure evaluate_retained_bulk(
    const gpu_full_step::NonlocalGpuProfile& profile,
    const std::vector<gpu_full_step::NonlocalGpuSample>& gpu_state,
    const std::vector<double>& gpu_density,
    const std::vector<gpu_full_step::NonlocalGpuSample>& cpu_state,
    const std::vector<double>& cpu_density,
    const std::vector<gpu_full_step::NonlocalGpuSample>& permuted_state,
    const std::vector<double>& permuted_density) {
    constexpr double surface_quantile = 0.99;
    const EulerianField gpu_coarse_internal = build_field(
        profile, gpu_state, gpu_density, 0.05, surface_quantile);
    const EulerianField cpu_coarse_internal = build_field(
        profile, cpu_state, cpu_density, 0.05, surface_quantile);
    const EulerianField permuted_coarse_internal = build_field(
        profile, permuted_state, permuted_density, 0.05, surface_quantile);
    const EulerianField gpu_fine_internal = build_field(
        profile, gpu_state, gpu_density, 0.025, surface_quantile);
    const EulerianField cpu_fine_internal = build_field(
        profile, cpu_state, cpu_density, 0.025, surface_quantile);
    const EulerianField permuted_fine_internal = build_field(
        profile, permuted_state, permuted_density, 0.025, surface_quantile);
    const EulerianComparison coarse_internal = compare_fields(
        profile, gpu_coarse_internal, cpu_coarse_internal);
    const EulerianComparison fine_internal = compare_fields(
        profile, gpu_fine_internal, cpu_fine_internal);

    RetainedBulkClosure result;
    result.gpu_coarse = copy_field(gpu_coarse_internal);
    result.cpu_coarse = copy_field(cpu_coarse_internal);
    result.permuted_coarse = copy_field(permuted_coarse_internal);
    result.gpu_fine = copy_field(gpu_fine_internal);
    result.cpu_fine = copy_field(cpu_fine_internal);
    result.permuted_fine = copy_field(permuted_fine_internal);
    result.coarse = copy_comparison(coarse_internal);
    result.fine = copy_comparison(fine_internal);
    result.permutation_exact = gpu_coarse_internal.field_root
            == permuted_coarse_internal.field_root
        && gpu_fine_internal.field_root == permuted_fine_internal.field_root;
    result.bulk_bands_passed = bulk_band(coarse_internal)
        && bulk_band(fine_internal);
    result.frozen_roots_exact =
        gpu_coarse_internal.field_root
            == "1c25e1f7535423ebd2a98a59d10778ef2027a78eac26d064bb4a9a3cc5e4d6d3"
        && cpu_coarse_internal.field_root
            == "b980ba01d68cce5200a617fcff3a6d346cf4ea3f628c2f03e54e2f91e993e33f"
        && permuted_coarse_internal.field_root
            == "1c25e1f7535423ebd2a98a59d10778ef2027a78eac26d064bb4a9a3cc5e4d6d3"
        && gpu_fine_internal.field_root
            == "4391d023fe83ab9faefee5ad0aa5d6b86e8a49f1fddaa2ae644303977a554972"
        && cpu_fine_internal.field_root
            == "6a40ed1da14b0805999c50a283af946e17d6a59a59a93cdad88ea6ba0c6c1538"
        && permuted_fine_internal.field_root
            == "4391d023fe83ab9faefee5ad0aa5d6b86e8a49f1fddaa2ae644303977a554972"
        && gpu_coarse_internal.work_root
            == "7418e5df7ffab41a1ce5c8b9631fb55ffa734e3dfe556e9eda26fa250e5390ab"
        && cpu_coarse_internal.work_root
            == "cf7641f088724a3064818da3050e655f1b27e45ccd1387a5fc6041ff361b390e"
        && permuted_coarse_internal.work_root
            == "7418e5df7ffab41a1ce5c8b9631fb55ffa734e3dfe556e9eda26fa250e5390ab"
        && gpu_fine_internal.work_root
            == "9ee3243e1cc3d36a6d1c0af2192211fad143e43817bff6e8685c49b3cc29e70d"
        && cpu_fine_internal.work_root
            == "1929691b9b3a59d9551a278463ed511254274ad3f6d449eb341d8743135f56f5"
        && permuted_fine_internal.work_root
            == "9ee3243e1cc3d36a6d1c0af2192211fad143e43817bff6e8685c49b3cc29e70d"
        && coarse_internal.work_root
            == "2b9bcb7b7a4c524e7b576606dd2e6d1f8e49d2599d743ceaaad8332450fa1c20"
        && coarse_internal.metrics_root
            == "62ca6d50f1b0c9451a57090ddba20a4bf28f577529c309b5e3b604f20e8d822d"
        && fine_internal.work_root
            == "8455ea3554bbe6c776b2fc890e2bcabbbae1c9b3a36faa24fae76e8e5be7fc5c"
        && fine_internal.metrics_root
            == "3b7c0beb2fc1c0367fc686ca478509234563096ef02a38750f6400c9e5602d25";
    result.valid = gpu_coarse_internal.valid && cpu_coarse_internal.valid
        && permuted_coarse_internal.valid && gpu_fine_internal.valid
        && cpu_fine_internal.valid && permuted_fine_internal.valid
        && coarse_internal.valid && fine_internal.valid
        && result.permutation_exact && result.bulk_bands_passed
        && result.frozen_roots_exact;

    std::ostringstream material;
    material << "nextengine.nonlocal.ncgp8.retained-bulk.v1\n"
             << result.valid << ':' << result.bulk_bands_passed << ':'
             << result.permutation_exact << ':' << result.frozen_roots_exact
             << '\n';
    append_field(material, result.gpu_coarse);
    append_field(material, result.cpu_coarse);
    append_field(material, result.permuted_coarse);
    append_field(material, result.gpu_fine);
    append_field(material, result.cpu_fine);
    append_field(material, result.permuted_fine);
    append_comparison(material, result.coarse);
    append_comparison(material, result.fine);
    result.closure_root = sha256_hex(material.str());
    return result;
}

}  // namespace nextengine::nonlocal::ncgp8
