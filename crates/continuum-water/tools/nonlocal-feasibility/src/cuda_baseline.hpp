#pragma once

#include "oracle.hpp"
#include "profiles.hpp"

#include <string>

namespace nextengine::nonlocal {

struct CommandReport {
    bool passed = false;
    std::string json;
};

enum class AccumulationMode {
    SourceAtomicV0,
    GatherDirectedR0,
    UniquePairSegmentedO3,
};

enum class HandoffMode {
    CopyV0,
    PointerSwapO1,
};

enum class TermKernelMode {
    RuntimeV0,
    SpecializedO2,
};

enum class StorageMode {
    StableSampleV0,
    CellSortedO4,
};

const char* accumulation_identity(AccumulationMode mode);
AccumulationMode parse_accumulation_identity(const std::string& identity);
const char* handoff_identity(HandoffMode mode);
HandoffMode parse_handoff_identity(const std::string& identity);
const char* term_kernel_identity(TermKernelMode mode);
TermKernelMode parse_term_kernel_identity(const std::string& identity);
const char* storage_identity(StorageMode mode);
StorageMode parse_storage_identity(const std::string& identity);

CommandReport run_cuda_self_test(
    AccumulationMode mode = AccumulationMode::SourceAtomicV0,
    HandoffMode handoff = HandoffMode::CopyV0,
    TermKernelMode term_kernels = TermKernelMode::RuntimeV0,
    StorageMode storage = StorageMode::StableSampleV0);
CommandReport run_cuda_check(
    const Profile& profile,
    int iterations,
    AccumulationMode mode = AccumulationMode::SourceAtomicV0,
    HandoffMode handoff = HandoffMode::CopyV0,
    TermKernelMode term_kernels = TermKernelMode::RuntimeV0,
    StorageMode storage = StorageMode::StableSampleV0);
CommandReport run_cuda_repeatability(
    const Profile& profile,
    int iterations,
    int runs,
    AccumulationMode mode,
    HandoffMode handoff = HandoffMode::CopyV0,
    TermKernelMode term_kernels = TermKernelMode::RuntimeV0,
    StorageMode storage = StorageMode::StableSampleV0);
CommandReport run_cuda_benchmark(
    const Profile& profile,
    int warmup,
    int runs,
    AccumulationMode mode = AccumulationMode::SourceAtomicV0,
    HandoffMode handoff = HandoffMode::CopyV0,
    TermKernelMode term_kernels = TermKernelMode::RuntimeV0,
    StorageMode storage = StorageMode::StableSampleV0);
CommandReport run_cuda_np0_baseline(
    const Profile& profile,
    int warmup,
    int runs);
CommandReport run_cuda_p1_check(
    const Profile& profile,
    int iterations);
CommandReport run_cuda_p1_tournament(
    const Profile& profile,
    int warmup,
    int runs);
CommandReport run_cuda_layout_tournament(
    const Profile& profile,
    int warmup,
    int runs);
CommandReport run_cuda_locality_tournament(
    const Profile& profile,
    int warmup,
    int runs);
CommandReport run_cuda_retained_tournament(
    const Profile& profile,
    int warmup,
    int runs);

} // namespace nextengine::nonlocal
