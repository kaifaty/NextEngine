cmake_minimum_required(VERSION 3.28)

project(nextengine_nonlocal_feasibility LANGUAGES CUDA CXX)

add_executable(
    nonlocal-feasibility
    src/main.cpp
    src/boundary_discriminator.cpp
    src/cuda_baseline.cu
    src/oracle.cpp
    src/profiles.cpp
    src/sha256.cpp
    src/tiny_corpus.cpp
)
target_compile_features(nonlocal-feasibility PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-feasibility
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-feasibility
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)

add_executable(
    nonlocal-corrected-cuda-terms
    src/corrected_cuda_terms.cu
    src/corrected_cuda_terms_energy.cpp
    src/corrected_cuda_terms_main.cpp
    src/corrected_cuda_terms_reference.cpp
    src/sha256.cpp
)

add_executable(
    nonlocal-corrected-cuda-neighbors
    src/corrected_cuda_neighbors.cu
    src/corrected_cuda_neighbors_main.cpp
    src/corrected_cuda_neighbors_reference.cpp
    src/sha256.cpp
)

add_executable(
    nonlocal-corrected-cuda-assembly
    src/corrected_cuda_assembly.cu
    src/corrected_cuda_assembly_energy.cpp
    src/corrected_cuda_assembly_main.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)

add_executable(
    nonlocal-corrected-cuda-consequence
    src/corrected_cuda_assembly.cu
    src/corrected_cuda_assembly_energy.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/corrected_cuda_consequence.cu
    src/corrected_cuda_consequence_main.cpp
    src/sha256.cpp
)

add_executable(
    nonlocal-corrected-cuda-trust
    src/corrected_cuda_assembly.cu
    src/corrected_cuda_assembly_reference.cpp
    src/corrected_cuda_trust_main.cpp
    src/sha256.cpp
)

add_executable(
    nonlocal-corrected-cuda-static-solve
    src/corrected_cuda_assembly.cu
    src/corrected_cuda_assembly_reference.cpp
    src/corrected_cuda_static_solve_main.cpp
    src/sha256.cpp
)

add_executable(
    nonlocal-corrected-cuda-full-step
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_full_step_main.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)
target_compile_features(
    nonlocal-corrected-cuda-full-step PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-full-step
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-full-step
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-full-step
    PRIVATE
        $<$<CONFIG:Release>:-s>
)
target_compile_features(
    nonlocal-corrected-cuda-static-solve PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-static-solve
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-static-solve
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-static-solve
    PRIVATE
        $<$<CONFIG:Release>:-s>
)
target_compile_features(
    nonlocal-corrected-cuda-trust PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-trust
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-trust
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-trust
    PRIVATE
        $<$<CONFIG:Release>:-s>
)
target_compile_features(
    nonlocal-corrected-cuda-consequence PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-consequence
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-consequence
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-consequence
    PRIVATE
        $<$<CONFIG:Release>:-s>
)
target_compile_features(
    nonlocal-corrected-cuda-assembly PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-assembly
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-assembly
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-assembly
    PRIVATE
        $<$<CONFIG:Release>:-s>
)
target_compile_features(
    nonlocal-corrected-cuda-neighbors PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-neighbors
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-neighbors
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-neighbors
    PRIVATE
        $<$<CONFIG:Release>:-s>
)
target_compile_features(
    nonlocal-corrected-cuda-terms PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-terms
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-terms
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)

add_executable(
    nonlocal-npr1-canonical
    src/canonical.cpp
    src/canonical_main.cpp
    src/sha256.cpp
    src/term_controls.cpp
)
target_compile_features(nonlocal-npr1-canonical PRIVATE cxx_std_17)
target_compile_options(
    nonlocal-npr1-canonical
    PRIVATE
        $<$<CXX_COMPILER_ID:GNU,Clang>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
)

set(
    NEXTENGINE_NONLOCAL_FORMULA_RECLOSURE_CORE_SOURCES
    src/balanced_canonical.cpp
    src/boundary_reference.cpp
    src/canonical.cpp
    src/dimensional_profile.cpp
    src/formula_discriminators.cpp
    src/formula_reclosure.cpp
    src/reference_attestation.cpp
    src/sha256.cpp
    src/variational_reference.cpp
)

find_package(OpenMP REQUIRED COMPONENTS CXX)

function(nextengine_configure_formula_reclosure_target target optimization)
    target_compile_features(${target} PRIVATE cxx_std_17)
    target_compile_definitions(
        ${target} PRIVATE NEXTENGINE_NONLOCAL_OPENMP=1)
    target_compile_options(
        ${target}
        PRIVATE
            $<$<CXX_COMPILER_ID:GNU,Clang>:${optimization};-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
    )
endfunction()

add_library(
    nonlocal-formula-reclosure-core STATIC
    ${NEXTENGINE_NONLOCAL_FORMULA_RECLOSURE_CORE_SOURCES}
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-reclosure-core -O3)
target_link_libraries(
    nonlocal-formula-reclosure-core PUBLIC OpenMP::OpenMP_CXX quadmath)

add_executable(
    nonlocal-formula-reclosure
    src/formula_reclosure_main.cpp
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-reclosure -O3)
target_link_libraries(
    nonlocal-formula-reclosure PRIVATE nonlocal-formula-reclosure-core)

add_executable(
    nonlocal-formula-parent-fixture-cache-release
    EXCLUDE_FROM_ALL
    src/formula_parent_fixture_cache_main.cpp
    src/formula_probe_cache.cpp
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-parent-fixture-cache-release -O3)
target_link_libraries(
    nonlocal-formula-parent-fixture-cache-release
    PRIVATE nonlocal-formula-reclosure-core)

add_executable(
    nonlocal-formula-operator-schedule-release
    EXCLUDE_FROM_ALL
    src/formula_operator_schedule_main.cpp
    src/formula_probe_cache.cpp
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-operator-schedule-release -O3)
target_link_libraries(
    nonlocal-formula-operator-schedule-release
    PRIVATE nonlocal-formula-reclosure-core)

add_executable(
    nonlocal-formula-denominator-residual-release
    EXCLUDE_FROM_ALL
    src/formula_denominator_residual_main.cpp
    src/formula_probe_cache.cpp
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-denominator-residual-release -O3)
target_link_libraries(
    nonlocal-formula-denominator-residual-release
    PRIVATE nonlocal-formula-reclosure-core)

add_executable(
    nonlocal-formula-quadratic-step-sign-release
    EXCLUDE_FROM_ALL
    src/formula_quadratic_step_sign_main.cpp
    src/formula_probe_cache.cpp
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-quadratic-step-sign-release -O3)
target_link_libraries(
    nonlocal-formula-quadratic-step-sign-release
    PRIVATE nonlocal-formula-reclosure-core)

add_executable(
    nonlocal-formula-certificate-detail-release
    EXCLUDE_FROM_ALL
    src/formula_certificate_detail_main.cpp
    src/formula_probe_cache.cpp
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-certificate-detail-release -O3)
target_link_libraries(
    nonlocal-formula-certificate-detail-release
    PRIVATE nonlocal-formula-reclosure-core)

add_executable(
    nonlocal-formula-center-producer-release
    EXCLUDE_FROM_ALL
    src/formula_center_producer_main.cpp
    src/formula_probe_cache.cpp
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-center-producer-release -O3)
target_link_libraries(
    nonlocal-formula-center-producer-release
    PRIVATE nonlocal-formula-reclosure-core)

add_executable(
    nonlocal-formula-tangent-twofold-recurrence-release
    EXCLUDE_FROM_ALL
    src/formula_tangent_twofold_recurrence_main.cpp
    src/formula_probe_cache.cpp
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-tangent-twofold-recurrence-release -O3)
target_link_libraries(
    nonlocal-formula-tangent-twofold-recurrence-release
    PRIVATE nonlocal-formula-reclosure-core)

add_executable(
    nonlocal-formula-product-precision-localization-release
    EXCLUDE_FROM_ALL
    src/formula_product_precision_localization_main.cpp
    src/formula_probe_cache.cpp
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-product-precision-localization-release -O3)
target_link_libraries(
    nonlocal-formula-product-precision-localization-release
    PRIVATE nonlocal-formula-reclosure-core)

add_executable(
    nonlocal-formula-independent-recurrence-checker-release
    EXCLUDE_FROM_ALL
    src/formula_independent_recurrence_checker_main.cpp
    src/formula_probe_cache.cpp
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-independent-recurrence-checker-release -O3)
target_link_libraries(
    nonlocal-formula-independent-recurrence-checker-release
    PRIVATE nonlocal-formula-reclosure-core)

add_executable(
    nonlocal-formula-admitted-tangent-boundary-release
    EXCLUDE_FROM_ALL
    src/formula_admitted_tangent_boundary_main.cpp
    src/formula_probe_cache.cpp
)
nextengine_configure_formula_reclosure_target(
    nonlocal-formula-admitted-tangent-boundary-release -O3)
target_link_libraries(
    nonlocal-formula-admitted-tangent-boundary-release
    PRIVATE nonlocal-formula-reclosure-core)

function(nextengine_configure_r63zm_target target optimization)
    target_compile_features(${target} PRIVATE cxx_std_20)
    target_compile_options(
        ${target}
        PRIVATE
            $<$<CXX_COMPILER_ID:GNU,Clang>:${optimization};-Wall;-Wextra;-Wpedantic;-Werror;-frounding-math;-fno-fast-math;-ffp-contract=off>
    )
    target_link_libraries(${target} PRIVATE quadmath)
endfunction()

add_executable(
    nonlocal-formula-r63zm-raw-semantic-release
    EXCLUDE_FROM_ALL
    r63zm/raw_semantic_probe.cpp
    r63zm/allocation_probe.cpp
)
nextengine_configure_r63zm_target(
    nonlocal-formula-r63zm-raw-semantic-release -O3)

add_executable(
    nonlocal-formula-r63zm-product-release
    EXCLUDE_FROM_ALL
    r63zm/fixed_product_probe.cpp
    r63zm/allocation_probe.cpp
)
nextengine_configure_r63zm_target(
    nonlocal-formula-r63zm-product-release -O3)

add_executable(
    nonlocal-formula-r63zm-producer-release
    EXCLUDE_FROM_ALL
    r63zm/fixed_binary_producer.cpp
    r63zm/allocation_probe.cpp
)
nextengine_configure_r63zm_target(
    nonlocal-formula-r63zm-producer-release -O3)

add_executable(
    nonlocal-formula-r63zm-checker-release
    EXCLUDE_FROM_ALL
    r63zm/fixed_binary_checker.cpp
    r63zm/allocation_probe.cpp
)
nextengine_configure_r63zm_target(
    nonlocal-formula-r63zm-checker-release -O3)

add_executable(
    nonlocal-formula-r63zn-prefix-candidate-release
    EXCLUDE_FROM_ALL
    r63zn/prefix_candidate.cpp
    r63zm/allocation_probe.cpp
)
nextengine_configure_r63zm_target(
    nonlocal-formula-r63zn-prefix-candidate-release -O3)

add_executable(
    nonlocal-formula-r63zn-prefix-checker-release
    EXCLUDE_FROM_ALL
    r63zn/prefix_checker.cpp
    r63zm/allocation_probe.cpp
)
nextengine_configure_r63zm_target(
    nonlocal-formula-r63zn-prefix-checker-release -O3)

add_executable(
    nonlocal-formula-r63zn-receipt-mutator-release
    EXCLUDE_FROM_ALL
    r63zn/receipt_mutator.cpp
)
nextengine_configure_r63zm_target(
    nonlocal-formula-r63zn-receipt-mutator-release -O3)
set_target_properties(
    nonlocal-formula-r63zn-prefix-candidate-release
    nonlocal-formula-r63zn-prefix-checker-release
    nonlocal-formula-r63zn-receipt-mutator-release
    PROPERTIES CXX_EXTENSIONS OFF
)

add_executable(
    nonlocal-formula-r63zo-rounded-update-preflight-release
    EXCLUDE_FROM_ALL
    r63zo/rounded_update_preflight.cpp
)
nextengine_configure_r63zm_target(
    nonlocal-formula-r63zo-rounded-update-preflight-release -O3)
set_target_properties(
    nonlocal-formula-r63zo-rounded-update-preflight-release
    PROPERTIES CXX_EXTENSIONS OFF
)

add_executable(
    nonlocal-formula-r63zp-direct-product-dyadic-preflight-release
    EXCLUDE_FROM_ALL
    r63zp/direct_product_dyadic_preflight.cpp
)
nextengine_configure_r63zm_target(
    nonlocal-formula-r63zp-direct-product-dyadic-preflight-release -O3)
set_target_properties(
    nonlocal-formula-r63zp-direct-product-dyadic-preflight-release
    PROPERTIES CXX_EXTENSIONS OFF
)

add_executable(
    nonlocal-formula-r63zp-direct-product-producer-release
    EXCLUDE_FROM_ALL
    r63zp/direct_product_producer.cpp
    r63zm/allocation_probe.cpp
)
nextengine_configure_r63zm_target(
    nonlocal-formula-r63zp-direct-product-producer-release -O3)
set_target_properties(
    nonlocal-formula-r63zp-direct-product-producer-release
    PROPERTIES CXX_EXTENSIONS OFF
)

add_executable(
    nonlocal-formula-r63zp-direct-product-checker-release
    EXCLUDE_FROM_ALL
    r63zp/direct_product_checker.cpp
    r63zm/allocation_probe.cpp
)
nextengine_configure_r63zm_target(
    nonlocal-formula-r63zp-direct-product-checker-release -O3)
set_target_properties(
    nonlocal-formula-r63zp-direct-product-checker-release
    PROPERTIES CXX_EXTENSIONS OFF
)

option(
    NEXTENGINE_NONLOCAL_BUILD_FAST_RESEARCH_TARGET
    "Build the strict-FP, compile-latency-oriented formula reclosure author target"
    ON
)
if(NEXTENGINE_NONLOCAL_BUILD_FAST_RESEARCH_TARGET)
    add_library(
        nonlocal-formula-reclosure-dev-core STATIC
        EXCLUDE_FROM_ALL
        ${NEXTENGINE_NONLOCAL_FORMULA_RECLOSURE_CORE_SOURCES}
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-reclosure-dev-core -O0)
    target_compile_definitions(
        nonlocal-formula-reclosure-dev-core PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)
    target_link_libraries(
        nonlocal-formula-reclosure-dev-core
        PUBLIC OpenMP::OpenMP_CXX quadmath)

    add_executable(
        nonlocal-formula-reclosure-dev
        EXCLUDE_FROM_ALL
        src/formula_reclosure_main.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-reclosure-dev -O0)
    target_link_libraries(
        nonlocal-formula-reclosure-dev
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-reclosure-dev PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-probe-api-smoke
        EXCLUDE_FROM_ALL
        src/formula_probe_api_smoke_main.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-probe-api-smoke -O0)
    target_link_libraries(
        nonlocal-formula-probe-api-smoke
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-probe-api-smoke PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-parent-fixture-smoke
        EXCLUDE_FROM_ALL
        src/formula_parent_fixture_smoke_main.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-parent-fixture-smoke -O0)
    target_link_libraries(
        nonlocal-formula-parent-fixture-smoke
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-parent-fixture-smoke PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-parent-fixture-cache
        EXCLUDE_FROM_ALL
        src/formula_parent_fixture_cache_main.cpp
        src/formula_probe_cache.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-parent-fixture-cache -O0)
    target_link_libraries(
        nonlocal-formula-parent-fixture-cache
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-parent-fixture-cache PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-probe-kernel-smoke
        EXCLUDE_FROM_ALL
        src/formula_probe_kernel_smoke_main.cpp
        src/formula_probe_cache.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-probe-kernel-smoke -O0)
    target_link_libraries(
        nonlocal-formula-probe-kernel-smoke
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-probe-kernel-smoke PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-operator-schedule-dev
        EXCLUDE_FROM_ALL
        src/formula_operator_schedule_main.cpp
        src/formula_probe_cache.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-operator-schedule-dev -O0)
    target_link_libraries(
        nonlocal-formula-operator-schedule-dev
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-operator-schedule-dev PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-denominator-residual-dev
        EXCLUDE_FROM_ALL
        src/formula_denominator_residual_main.cpp
        src/formula_probe_cache.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-denominator-residual-dev -O0)
    target_link_libraries(
        nonlocal-formula-denominator-residual-dev
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-denominator-residual-dev PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-quadratic-step-sign-dev
        EXCLUDE_FROM_ALL
        src/formula_quadratic_step_sign_main.cpp
        src/formula_probe_cache.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-quadratic-step-sign-dev -O0)
    target_link_libraries(
        nonlocal-formula-quadratic-step-sign-dev
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-quadratic-step-sign-dev PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-certificate-detail-dev
        EXCLUDE_FROM_ALL
        src/formula_certificate_detail_main.cpp
        src/formula_probe_cache.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-certificate-detail-dev -O0)
    target_link_libraries(
        nonlocal-formula-certificate-detail-dev
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-certificate-detail-dev PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-center-producer-dev
        EXCLUDE_FROM_ALL
        src/formula_center_producer_main.cpp
        src/formula_probe_cache.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-center-producer-dev -O0)
    target_link_libraries(
        nonlocal-formula-center-producer-dev
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-center-producer-dev PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-tangent-twofold-recurrence-dev
        EXCLUDE_FROM_ALL
        src/formula_tangent_twofold_recurrence_main.cpp
        src/formula_probe_cache.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-tangent-twofold-recurrence-dev -O0)
    target_link_libraries(
        nonlocal-formula-tangent-twofold-recurrence-dev
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-tangent-twofold-recurrence-dev PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-product-precision-localization-dev
        EXCLUDE_FROM_ALL
        src/formula_product_precision_localization_main.cpp
        src/formula_probe_cache.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-product-precision-localization-dev -O0)
    target_link_libraries(
        nonlocal-formula-product-precision-localization-dev
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-product-precision-localization-dev PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-independent-recurrence-checker-dev
        EXCLUDE_FROM_ALL
        src/formula_independent_recurrence_checker_main.cpp
        src/formula_probe_cache.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-independent-recurrence-checker-dev -O0)
    target_link_libraries(
        nonlocal-formula-independent-recurrence-checker-dev
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-independent-recurrence-checker-dev PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-admitted-tangent-boundary-dev
        EXCLUDE_FROM_ALL
        src/formula_admitted_tangent_boundary_main.cpp
        src/formula_probe_cache.cpp
    )
    nextengine_configure_formula_reclosure_target(
        nonlocal-formula-admitted-tangent-boundary-dev -O0)
    target_link_libraries(
        nonlocal-formula-admitted-tangent-boundary-dev
        PRIVATE nonlocal-formula-reclosure-dev-core)
    target_compile_definitions(
        nonlocal-formula-admitted-tangent-boundary-dev PRIVATE
            NEXTENGINE_NONLOCAL_AUTHOR_BUILD=1)

    add_executable(
        nonlocal-formula-r63zm-raw-semantic-dev
        EXCLUDE_FROM_ALL
        r63zm/raw_semantic_probe.cpp
        r63zm/allocation_probe.cpp
    )
    nextengine_configure_r63zm_target(
        nonlocal-formula-r63zm-raw-semantic-dev -O0)

    add_executable(
        nonlocal-formula-r63zm-product-dev
        EXCLUDE_FROM_ALL
        r63zm/fixed_product_probe.cpp
        r63zm/allocation_probe.cpp
    )
    nextengine_configure_r63zm_target(
        nonlocal-formula-r63zm-product-dev -O0)

    add_executable(
        nonlocal-formula-r63zm-producer-dev
        EXCLUDE_FROM_ALL
        r63zm/fixed_binary_producer.cpp
        r63zm/allocation_probe.cpp
    )
    nextengine_configure_r63zm_target(
        nonlocal-formula-r63zm-producer-dev -O0)

    add_executable(
        nonlocal-formula-r63zm-checker-dev
        EXCLUDE_FROM_ALL
        r63zm/fixed_binary_checker.cpp
        r63zm/allocation_probe.cpp
    )
    nextengine_configure_r63zm_target(
        nonlocal-formula-r63zm-checker-dev -O0)

    add_executable(
        nonlocal-formula-r63zn-prefix-candidate-dev
        EXCLUDE_FROM_ALL
        r63zn/prefix_candidate.cpp
        r63zm/allocation_probe.cpp
    )
    nextengine_configure_r63zm_target(
        nonlocal-formula-r63zn-prefix-candidate-dev -O0)

    add_executable(
        nonlocal-formula-r63zn-prefix-checker-dev
        EXCLUDE_FROM_ALL
        r63zn/prefix_checker.cpp
        r63zm/allocation_probe.cpp
    )
    nextengine_configure_r63zm_target(
        nonlocal-formula-r63zn-prefix-checker-dev -O0)

    add_executable(
        nonlocal-formula-r63zn-receipt-mutator-dev
        EXCLUDE_FROM_ALL
        r63zn/receipt_mutator.cpp
    )
    nextengine_configure_r63zm_target(
        nonlocal-formula-r63zn-receipt-mutator-dev -O0)
    set_target_properties(
        nonlocal-formula-r63zn-prefix-candidate-dev
        nonlocal-formula-r63zn-prefix-checker-dev
        nonlocal-formula-r63zn-receipt-mutator-dev
        PROPERTIES CXX_EXTENSIONS OFF
    )

    add_executable(
        nonlocal-formula-r63zp-direct-product-producer-dev
        EXCLUDE_FROM_ALL
        r63zp/direct_product_producer.cpp
        r63zm/allocation_probe.cpp
    )
    nextengine_configure_r63zm_target(
        nonlocal-formula-r63zp-direct-product-producer-dev -O0)
    set_target_properties(
        nonlocal-formula-r63zp-direct-product-producer-dev
        PROPERTIES CXX_EXTENSIONS OFF
    )

    add_executable(
        nonlocal-formula-r63zp-direct-product-checker-dev
        EXCLUDE_FROM_ALL
        r63zp/direct_product_checker.cpp
        r63zm/allocation_probe.cpp
    )
    nextengine_configure_r63zm_target(
        nonlocal-formula-r63zp-direct-product-checker-dev -O0)
    set_target_properties(
        nonlocal-formula-r63zp-direct-product-checker-dev
        PROPERTIES CXX_EXTENSIONS OFF
    )
endif()
