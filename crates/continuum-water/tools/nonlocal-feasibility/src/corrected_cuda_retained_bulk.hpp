#pragma once

#include "corrected_cuda_full_step.hpp"

#include <cstdint>
#include <string>
#include <vector>

namespace nextengine::nonlocal::ncgp8 {

struct RetainedBulkWork {
    std::uint64_t samples_validated = 0U;
    std::uint64_t cic_records_emitted = 0U;
    std::uint64_t cic_records_sorted = 0U;
    std::uint64_t cic_records_reduced = 0U;
    std::uint64_t column_records_emitted = 0U;
    std::uint64_t column_records_sorted = 0U;
    std::uint64_t column_records_reduced = 0U;
    std::uint64_t quantile_reads = 0U;
    std::uint64_t node_comparisons = 0U;
    std::uint64_t column_comparisons = 0U;
    std::uint64_t root_derivations = 0U;
};

struct RetainedBulkFieldReceipt {
    bool valid = false;
    RetainedBulkWork work;
    std::string node_root;
    std::string surface_root;
    std::string work_root;
    std::string field_root;
};

struct RetainedBulkComparisonReceipt {
    bool valid = false;
    double mass_tv = 0.0;
    double density_rmse = 0.0;
    double velocity_rmse_normalized = 0.0;
    double center_of_mass_error = 0.0;
    double maximum_node_mass_difference_particles = 0.0;
    double wet_column_symmetric_difference = 0.0;
    double surface_rmse = 0.0;
    double surface_p95 = 0.0;
    double surface_maximum = 0.0;
    std::uint64_t common_nodes = 0U;
    std::uint64_t common_wet_columns = 0U;
    std::uint64_t union_wet_columns = 0U;
    RetainedBulkWork work;
    std::string work_root;
    std::string metrics_root;
};

struct RetainedBulkClosure {
    bool valid = false;
    bool bulk_bands_passed = false;
    bool permutation_exact = false;
    bool frozen_roots_exact = false;
    RetainedBulkFieldReceipt gpu_coarse;
    RetainedBulkFieldReceipt cpu_coarse;
    RetainedBulkFieldReceipt permuted_coarse;
    RetainedBulkFieldReceipt gpu_fine;
    RetainedBulkFieldReceipt cpu_fine;
    RetainedBulkFieldReceipt permuted_fine;
    RetainedBulkComparisonReceipt coarse;
    RetainedBulkComparisonReceipt fine;
    std::string closure_root;
};

RetainedBulkClosure evaluate_retained_bulk(
    const gpu_full_step::NonlocalGpuProfile& profile,
    const std::vector<gpu_full_step::NonlocalGpuSample>& gpu_state,
    const std::vector<double>& gpu_density,
    const std::vector<gpu_full_step::NonlocalGpuSample>& cpu_state,
    const std::vector<double>& cpu_density,
    const std::vector<gpu_full_step::NonlocalGpuSample>& permuted_state,
    const std::vector<double>& permuted_density);

}  // namespace nextengine::nonlocal::ncgp8
