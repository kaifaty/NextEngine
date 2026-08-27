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
endif()
