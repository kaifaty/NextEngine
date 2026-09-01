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
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)

add_executable(
    nonlocal-corrected-cuda-surface-translation
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_surface_translation_main.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cuda-compensated-scale
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_compensated_scale_main.cpp
    src/corrected_cuda_compensated_scale_physics.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cuda-full-step-diagnosis
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_compensated_scale_main.cpp
    src/corrected_cuda_compensated_scale_physics.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cuda-step92-diagnosis
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_compensated_scale_main.cpp
    src/corrected_cuda_compensated_scale_physics.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cuda-product-gate
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_compensated_scale_main.cpp
    src/corrected_cuda_compensated_scale_physics.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cuda-eulerian-diagnostic
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_compensated_scale_main.cpp
    src/corrected_cuda_compensated_scale_physics.cpp
    src/corrected_cuda_eulerian_diagnostic.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cuda-visible-surface
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_visible_surface_main.cpp
    src/corrected_cuda_retained_bulk.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cuda-complete-4k
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_complete_4k_main.cpp
    src/corrected_cuda_retained_bulk.cpp
    src/corrected_cuda_compensated_scale_physics.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cuda-cost-benchmark
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_cost_benchmark_main.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cpu-pressure-state
    src/corrected_cpu_pressure_state_main.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cpu-pressure-contact
    src/corrected_cpu_pressure_contact_main.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cpu-confined-pressure-contact
    src/corrected_cpu_confined_pressure_contact_main.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/sha256.cpp
)
add_executable(
    nonlocal-corrected-cpu-unified-constrained
    src/corrected_cpu_unified_constrained.hpp
    src/corrected_cpu_unified_constrained_candidate.cpp
    src/corrected_cpu_unified_constrained_oracle.cpp
    src/corrected_cpu_unified_constrained_main.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/sha256.cpp
)
set(NCGP1_SOURCE_FILES
    CMakeLists.txt
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_full_step_main.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/corrected_cuda_assembly.hpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP1_SOURCE_MATERIAL "")
foreach(NCGP1_SOURCE_FILE IN LISTS NCGP1_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP1_SOURCE_FILE}"
        NCGP1_SOURCE_FILE_HASH)
    string(APPEND NCGP1_SOURCE_MATERIAL
        "${NCGP1_SOURCE_FILE}:${NCGP1_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP1_SOURCE_ROOT "${NCGP1_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-corrected-gpu-full-step/00-physical-performance-contract.md"
    NCGP1_CONTRACT_0_ROOT)
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-corrected-gpu-full-step/01-scale-aware-solver-terminal.md"
    NCGP1_CONTRACT_1_ROOT)
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-corrected-gpu-full-step/02-trajectory-corpus.md"
    NCGP1_CONTRACT_2_ROOT)
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-corrected-gpu-full-step/03-review-repair-closure.md"
    NCGP1_CONTRACT_3_ROOT)
string(SHA256 NCGP1_CONTRACT_ROOT
    "${NCGP1_CONTRACT_0_ROOT}:${NCGP1_CONTRACT_1_ROOT}:${NCGP1_CONTRACT_2_ROOT}:${NCGP1_CONTRACT_3_ROOT}")
execute_process(
    COMMAND git rev-parse HEAD
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    OUTPUT_VARIABLE NCGP1_SOURCE_COMMIT
    OUTPUT_STRIP_TRAILING_WHITESPACE
    COMMAND_ERROR_IS_FATAL ANY
)
execute_process(
    COMMAND git rev-parse "HEAD^{tree}"
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    OUTPUT_VARIABLE NCGP1_SOURCE_TREE
    OUTPUT_STRIP_TRAILING_WHITESPACE
    COMMAND_ERROR_IS_FATAL ANY
)
set(NCGP1_COMPILER_FLAGS
    "CXX:-O3,-Wall,-Wextra,-Wpedantic,-Werror,-ffp-contract=off,-fno-fast-math|CUDA:-O3,--fmad=false,--prec-div=true,--prec-sqrt=true,--ftz=false|SM=86")
target_compile_definitions(
    nonlocal-corrected-cuda-full-step
    PRIVATE
        NCGP1_CONTRACT_ROOT="${NCGP1_CONTRACT_ROOT}"
        NCGP1_SOURCE_ROOT="${NCGP1_SOURCE_ROOT}"
        NCGP1_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP1_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP1_COMPILER_FLAGS="${NCGP1_COMPILER_FLAGS}"
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

set(NCGP2_SOURCE_FILES
    CMakeLists.txt
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_surface_translation_main.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/corrected_cuda_assembly.hpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP2_SOURCE_MATERIAL "")
foreach(NCGP2_SOURCE_FILE IN LISTS NCGP2_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP2_SOURCE_FILE}"
        NCGP2_SOURCE_FILE_HASH)
    string(APPEND NCGP2_SOURCE_MATERIAL
        "${NCGP2_SOURCE_FILE}:${NCGP2_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP2_SOURCE_ROOT "${NCGP2_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-surface-translation/00-surface-translation-contract.md"
    NCGP2_CONTRACT_ROOT)
set(NCGP2_COMPILER_FLAGS
    "CXX:-O3,-Wall,-Wextra,-Wpedantic,-Werror,-ffp-contract=off,-fno-fast-math|CUDA:-O3,--fmad=false,--prec-div=true,--prec-sqrt=true,--ftz=false|SM=86")
target_compile_definitions(
    nonlocal-corrected-cuda-surface-translation
    PRIVATE
        NCGP2_CONTRACT_ROOT="${NCGP2_CONTRACT_ROOT}"
        NCGP2_SOURCE_ROOT="${NCGP2_SOURCE_ROOT}"
        NCGP2_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP2_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP2_COMPILER_FLAGS="${NCGP2_COMPILER_FLAGS}"
        NCGP2_EXPERIMENTAL=1
)
target_compile_features(
    nonlocal-corrected-cuda-surface-translation PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-surface-translation
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-surface-translation
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-surface-translation
    PRIVATE
        $<$<CONFIG:Release>:-s>
)

set(NCGP3_SOURCE_FILES
    CMakeLists.txt
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_compensated_scale_main.cpp
    src/corrected_cuda_compensated_scale_physics.cpp
    src/corrected_cuda_compensated_scale_physics.hpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/corrected_cuda_assembly.hpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP3_SOURCE_MATERIAL "")
foreach(NCGP3_SOURCE_FILE IN LISTS NCGP3_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP3_SOURCE_FILE}"
        NCGP3_SOURCE_FILE_HASH)
    string(APPEND NCGP3_SOURCE_MATERIAL
        "${NCGP3_SOURCE_FILE}:${NCGP3_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP3_SOURCE_ROOT "${NCGP3_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-compensated-scale/00-compensated-scale-contract.md"
    NCGP3_CONTRACT_0_ROOT)
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-compensated-scale/01-graph-control-corrigendum.md"
    NCGP3_CONTRACT_1_ROOT)
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-compensated-scale/02-boundary-transaction-fixtures.md"
    NCGP3_CONTRACT_2_ROOT)
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-compensated-scale/03-corrected-fcr-profile-corrigendum.md"
    NCGP3_CONTRACT_3_ROOT)
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-compensated-scale/04-review-repair-contract.md"
    NCGP3_CONTRACT_4_ROOT)
string(SHA256 NCGP3_CONTRACT_ROOT
    "${NCGP3_CONTRACT_0_ROOT}:${NCGP3_CONTRACT_1_ROOT}:${NCGP3_CONTRACT_2_ROOT}:${NCGP3_CONTRACT_3_ROOT}:${NCGP3_CONTRACT_4_ROOT}")
set(NCGP3_COMPILER_FLAGS
    "CXX:-O3,-Wall,-Wextra,-Wpedantic,-Werror,-ffp-contract=off,-fno-fast-math|CUDA:-O3,--fmad=false,--prec-div=true,--prec-sqrt=true,--ftz=false|SM=86")
target_compile_definitions(
    nonlocal-corrected-cuda-compensated-scale
    PRIVATE
        NCGP3_CONTRACT_ROOT="${NCGP3_CONTRACT_ROOT}"
        NCGP3_SOURCE_ROOT="${NCGP3_SOURCE_ROOT}"
        NCGP3_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP3_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP3_COMPILER_FLAGS="${NCGP3_COMPILER_FLAGS}"
        NCGP2_EXPERIMENTAL=1
        NCGP3_EXPERIMENTAL=1
)
target_compile_features(
    nonlocal-corrected-cuda-compensated-scale PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-compensated-scale
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-compensated-scale
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-compensated-scale
    PRIVATE
        $<$<CONFIG:Release>:-s>
)

set(NCGP4_SOURCE_FILES
    CMakeLists.txt
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_compensated_scale_main.cpp
    src/corrected_cuda_compensated_scale_physics.cpp
    src/corrected_cuda_compensated_scale_physics.hpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/corrected_cuda_assembly.hpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP4_SOURCE_MATERIAL "")
foreach(NCGP4_SOURCE_FILE IN LISTS NCGP4_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP4_SOURCE_FILE}"
        NCGP4_SOURCE_FILE_HASH)
    string(APPEND NCGP4_SOURCE_MATERIAL
        "${NCGP4_SOURCE_FILE}:${NCGP4_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP4_SOURCE_ROOT "${NCGP4_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/00-ncgp4-solver-diagnosis-contract.md"
    NCGP4_CONTRACT_0_ROOT)
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/01-unpreconditioned-selection.md"
    NCGP4_CONTRACT_1_ROOT)
string(SHA256 NCGP4_CONTRACT_ROOT
    "${NCGP4_CONTRACT_0_ROOT}:${NCGP4_CONTRACT_1_ROOT}")
set(NCGP4_COMPILER_FLAGS
    "CXX:-O3,-Wall,-Wextra,-Wpedantic,-Werror,-ffp-contract=off,-fno-fast-math|CUDA:-O3,--fmad=false,--prec-div=true,--prec-sqrt=true,--ftz=false|SM=86")
target_compile_definitions(
    nonlocal-corrected-cuda-full-step-diagnosis
    PRIVATE
        NCGP3_CONTRACT_ROOT="${NCGP3_CONTRACT_ROOT}"
        NCGP3_SOURCE_ROOT="${NCGP3_SOURCE_ROOT}"
        NCGP3_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP3_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP3_COMPILER_FLAGS="${NCGP3_COMPILER_FLAGS}"
        NCGP4_CONTRACT_ROOT="${NCGP4_CONTRACT_ROOT}"
        NCGP4_SOURCE_ROOT="${NCGP4_SOURCE_ROOT}"
        NCGP4_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP4_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP4_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP2_EXPERIMENTAL=1
        NCGP3_EXPERIMENTAL=1
        NCGP4_EXPERIMENTAL=1
)
target_compile_features(
    nonlocal-corrected-cuda-full-step-diagnosis PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-full-step-diagnosis
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-full-step-diagnosis
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-full-step-diagnosis
    PRIVATE
        $<$<CONFIG:Release>:-s>
)

set(NCGP5_SOURCE_FILES ${NCGP4_SOURCE_FILES})
set(NCGP5_SOURCE_MATERIAL "")
foreach(NCGP5_SOURCE_FILE IN LISTS NCGP5_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP5_SOURCE_FILE}"
        NCGP5_SOURCE_FILE_HASH)
    string(APPEND NCGP5_SOURCE_MATERIAL
        "${NCGP5_SOURCE_FILE}:${NCGP5_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP5_SOURCE_ROOT "${NCGP5_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/02-step92-outlier-diagnosis.md"
    NCGP5_CONTRACT_2_ROOT)
string(SHA256 NCGP5_CONTRACT_ROOT
    "${NCGP4_CONTRACT_ROOT}:${NCGP5_CONTRACT_2_ROOT}")
target_compile_definitions(
    nonlocal-corrected-cuda-step92-diagnosis
    PRIVATE
        NCGP3_CONTRACT_ROOT="${NCGP3_CONTRACT_ROOT}"
        NCGP3_SOURCE_ROOT="${NCGP3_SOURCE_ROOT}"
        NCGP3_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP3_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP3_COMPILER_FLAGS="${NCGP3_COMPILER_FLAGS}"
        NCGP4_CONTRACT_ROOT="${NCGP4_CONTRACT_ROOT}"
        NCGP4_SOURCE_ROOT="${NCGP4_SOURCE_ROOT}"
        NCGP4_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP4_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP4_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP5_CONTRACT_ROOT="${NCGP5_CONTRACT_ROOT}"
        NCGP5_SOURCE_ROOT="${NCGP5_SOURCE_ROOT}"
        NCGP5_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP5_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP5_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP2_EXPERIMENTAL=1
        NCGP3_EXPERIMENTAL=1
        NCGP4_EXPERIMENTAL=1
        NCGP5_EXPERIMENTAL=1
)
target_compile_features(
    nonlocal-corrected-cuda-step92-diagnosis PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-step92-diagnosis
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-step92-diagnosis
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-step92-diagnosis
    PRIVATE
        $<$<CONFIG:Release>:-s>
)

set(NCGP6_SOURCE_FILES ${NCGP5_SOURCE_FILES})
set(NCGP6_SOURCE_MATERIAL "")
foreach(NCGP6_SOURCE_FILE IN LISTS NCGP6_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP6_SOURCE_FILE}"
        NCGP6_SOURCE_FILE_HASH)
    string(APPEND NCGP6_SOURCE_MATERIAL
        "${NCGP6_SOURCE_FILE}:${NCGP6_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP6_SOURCE_ROOT "${NCGP6_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/03-product-trajectory-gate.md"
    NCGP6_CONTRACT_3_ROOT)
string(SHA256 NCGP6_CONTRACT_ROOT
    "${NCGP5_CONTRACT_ROOT}:${NCGP6_CONTRACT_3_ROOT}")
target_compile_definitions(
    nonlocal-corrected-cuda-product-gate
    PRIVATE
        NCGP3_CONTRACT_ROOT="${NCGP3_CONTRACT_ROOT}"
        NCGP3_SOURCE_ROOT="${NCGP3_SOURCE_ROOT}"
        NCGP3_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP3_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP3_COMPILER_FLAGS="${NCGP3_COMPILER_FLAGS}"
        NCGP4_CONTRACT_ROOT="${NCGP4_CONTRACT_ROOT}"
        NCGP4_SOURCE_ROOT="${NCGP4_SOURCE_ROOT}"
        NCGP4_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP4_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP4_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP5_CONTRACT_ROOT="${NCGP5_CONTRACT_ROOT}"
        NCGP5_SOURCE_ROOT="${NCGP5_SOURCE_ROOT}"
        NCGP5_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP5_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP5_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP6_CONTRACT_ROOT="${NCGP6_CONTRACT_ROOT}"
        NCGP6_SOURCE_ROOT="${NCGP6_SOURCE_ROOT}"
        NCGP6_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP6_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP6_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP2_EXPERIMENTAL=1
        NCGP3_EXPERIMENTAL=1
        NCGP4_EXPERIMENTAL=1
        NCGP5_EXPERIMENTAL=1
        NCGP6_EXPERIMENTAL=1
)
target_compile_features(
    nonlocal-corrected-cuda-product-gate PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-product-gate
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-product-gate
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-product-gate
    PRIVATE
        $<$<CONFIG:Release>:-s>
)

set(NCGP7_SOURCE_FILES
    ${NCGP6_SOURCE_FILES}
    src/corrected_cuda_eulerian_diagnostic.cpp
    src/corrected_cuda_eulerian_diagnostic.hpp
)
set(NCGP7_SOURCE_MATERIAL "")
foreach(NCGP7_SOURCE_FILE IN LISTS NCGP7_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP7_SOURCE_FILE}"
        NCGP7_SOURCE_FILE_HASH)
    string(APPEND NCGP7_SOURCE_MATERIAL
        "${NCGP7_SOURCE_FILE}:${NCGP7_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP7_SOURCE_ROOT "${NCGP7_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/04-eulerian-step112-diagnostic.md"
    NCGP7_CONTRACT_4_ROOT)
string(SHA256 NCGP7_CONTRACT_ROOT
    "${NCGP6_CONTRACT_ROOT}:${NCGP7_CONTRACT_4_ROOT}")
target_compile_definitions(
    nonlocal-corrected-cuda-eulerian-diagnostic
    PRIVATE
        NCGP3_CONTRACT_ROOT="${NCGP3_CONTRACT_ROOT}"
        NCGP3_SOURCE_ROOT="${NCGP3_SOURCE_ROOT}"
        NCGP3_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP3_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP3_COMPILER_FLAGS="${NCGP3_COMPILER_FLAGS}"
        NCGP4_CONTRACT_ROOT="${NCGP4_CONTRACT_ROOT}"
        NCGP4_SOURCE_ROOT="${NCGP4_SOURCE_ROOT}"
        NCGP4_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP4_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP4_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP5_CONTRACT_ROOT="${NCGP5_CONTRACT_ROOT}"
        NCGP5_SOURCE_ROOT="${NCGP5_SOURCE_ROOT}"
        NCGP5_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP5_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP5_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP6_CONTRACT_ROOT="${NCGP6_CONTRACT_ROOT}"
        NCGP6_SOURCE_ROOT="${NCGP6_SOURCE_ROOT}"
        NCGP6_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP6_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP6_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP7_CONTRACT_ROOT="${NCGP7_CONTRACT_ROOT}"
        NCGP7_SOURCE_ROOT="${NCGP7_SOURCE_ROOT}"
        NCGP7_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP7_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP7_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP2_EXPERIMENTAL=1
        NCGP3_EXPERIMENTAL=1
        NCGP4_EXPERIMENTAL=1
        NCGP5_EXPERIMENTAL=1
        NCGP6_EXPERIMENTAL=1
        NCGP7_EXPERIMENTAL=1
)
target_compile_features(
    nonlocal-corrected-cuda-eulerian-diagnostic PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-eulerian-diagnostic
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-eulerian-diagnostic
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-eulerian-diagnostic
    PRIVATE
        $<$<CONFIG:Release>:-s>
)

set(NCGP8_SOURCE_FILES
    CMakeLists.txt
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_visible_surface_main.cpp
    src/corrected_cuda_retained_bulk.cpp
    src/corrected_cuda_retained_bulk.hpp
    src/corrected_cuda_eulerian_diagnostic.cpp
    src/corrected_cuda_eulerian_diagnostic.hpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/corrected_cuda_assembly.hpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP8_SOURCE_MATERIAL "")
foreach(NCGP8_SOURCE_FILE IN LISTS NCGP8_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP8_SOURCE_FILE}"
        NCGP8_SOURCE_FILE_HASH)
    string(APPEND NCGP8_SOURCE_MATERIAL
        "${NCGP8_SOURCE_FILE}:${NCGP8_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP8_SOURCE_ROOT "${NCGP8_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/05-visible-surface-observer.md"
    NCGP8_CONTRACT_5_ROOT)
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/06-visible-surface-control-corrigendum.md"
    NCGP8_CONTRACT_6_ROOT)
string(SHA256 NCGP8_CONTRACT_ROOT
    "${NCGP7_CONTRACT_ROOT}:${NCGP8_CONTRACT_5_ROOT}:${NCGP8_CONTRACT_6_ROOT}")
target_compile_definitions(
    nonlocal-corrected-cuda-visible-surface
    PRIVATE
        NCGP8_CONTRACT_ROOT="${NCGP8_CONTRACT_ROOT}"
        NCGP8_SOURCE_ROOT="${NCGP8_SOURCE_ROOT}"
        NCGP8_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP8_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP8_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP2_EXPERIMENTAL=1
        NCGP3_EXPERIMENTAL=1
        NCGP8_EXPERIMENTAL=1
)
target_compile_features(
    nonlocal-corrected-cuda-visible-surface PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-visible-surface
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-visible-surface
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-visible-surface
    PRIVATE
        $<$<CONFIG:Release>:-s>
)

set(NCGP9_SOURCE_FILES
    CMakeLists.txt
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_complete_4k_main.cpp
    src/corrected_cuda_visible_surface_main.cpp
    src/corrected_cuda_retained_bulk.cpp
    src/corrected_cuda_retained_bulk.hpp
    src/corrected_cuda_eulerian_diagnostic.cpp
    src/corrected_cuda_eulerian_diagnostic.hpp
    src/corrected_cuda_compensated_scale_physics.cpp
    src/corrected_cuda_compensated_scale_physics.hpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/corrected_cuda_assembly.hpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP9_SOURCE_MATERIAL "")
foreach(NCGP9_SOURCE_FILE IN LISTS NCGP9_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP9_SOURCE_FILE}"
        NCGP9_SOURCE_FILE_HASH)
    string(APPEND NCGP9_SOURCE_MATERIAL
        "${NCGP9_SOURCE_FILE}:${NCGP9_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP9_SOURCE_ROOT "${NCGP9_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/07-complete-4k-corpus.md"
    NCGP9_CONTRACT_7_ROOT)
string(SHA256 NCGP9_CONTRACT_ROOT
    "${NCGP8_CONTRACT_ROOT}:${NCGP9_CONTRACT_7_ROOT}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/08-pressure-f64-complete-4k.md"
    NCGP10_CONTRACT_8_ROOT)
string(SHA256 NCGP10_CONTRACT_ROOT
    "${NCGP9_CONTRACT_ROOT}:${NCGP10_CONTRACT_8_ROOT}")
target_compile_definitions(
    nonlocal-corrected-cuda-complete-4k
    PRIVATE
        NCGP8_CONTRACT_ROOT="${NCGP8_CONTRACT_ROOT}"
        NCGP8_SOURCE_ROOT="${NCGP8_SOURCE_ROOT}"
        NCGP8_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP8_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP8_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP9_CONTRACT_ROOT="${NCGP9_CONTRACT_ROOT}"
        NCGP9_SOURCE_ROOT="${NCGP9_SOURCE_ROOT}"
        NCGP9_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP9_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP9_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP10_CONTRACT_ROOT="${NCGP10_CONTRACT_ROOT}"
        NCGP10_SOURCE_ROOT="${NCGP9_SOURCE_ROOT}"
        NCGP10_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP10_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP10_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP2_EXPERIMENTAL=1
        NCGP3_EXPERIMENTAL=1
        NCGP8_EXPERIMENTAL=1
        NCGP9_EXPERIMENTAL=1
)
target_compile_features(
    nonlocal-corrected-cuda-complete-4k PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-complete-4k
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-complete-4k
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-complete-4k
    PRIVATE
        $<$<CONFIG:Release>:-s>
)

set(NCGP11_SOURCE_FILES
    CMakeLists.txt
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step.cu
    src/corrected_cuda_cost_benchmark_main.cpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_full_step_cpu_solver.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/corrected_cuda_assembly.hpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP11_SOURCE_MATERIAL "")
foreach(NCGP11_SOURCE_FILE IN LISTS NCGP11_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP11_SOURCE_FILE}"
        NCGP11_SOURCE_FILE_HASH)
    string(APPEND NCGP11_SOURCE_MATERIAL
        "${NCGP11_SOURCE_FILE}:${NCGP11_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP11_SOURCE_ROOT "${NCGP11_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/09-invalid-physics-cost-only.md"
    NCGP11_CONTRACT_9_ROOT)
string(SHA256 NCGP11_CONTRACT_ROOT
    "${NCGP10_CONTRACT_ROOT}:${NCGP11_CONTRACT_9_ROOT}")
target_compile_definitions(
    nonlocal-corrected-cuda-cost-benchmark
    PRIVATE
        NCGP11_CONTRACT_ROOT="${NCGP11_CONTRACT_ROOT}"
        NCGP11_SOURCE_ROOT="${NCGP11_SOURCE_ROOT}"
        NCGP11_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP11_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP11_COMPILER_FLAGS="${NCGP4_COMPILER_FLAGS}"
        NCGP2_EXPERIMENTAL=1
        NCGP3_EXPERIMENTAL=1
)
target_compile_features(
    nonlocal-corrected-cuda-cost-benchmark PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    nonlocal-corrected-cuda-cost-benchmark
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    nonlocal-corrected-cuda-cost-benchmark
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
target_link_options(
    nonlocal-corrected-cuda-cost-benchmark
    PRIVATE
        $<$<CONFIG:Release>:-s>
)

set(NCGP12_SOURCE_FILES
    CMakeLists.txt
    src/corrected_cpu_pressure_state_main.cpp
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step_reference.cpp
    src/corrected_cuda_assembly_reference.cpp
    src/corrected_cuda_assembly.hpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP12_SOURCE_MATERIAL "")
foreach(NCGP12_SOURCE_FILE IN LISTS NCGP12_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP12_SOURCE_FILE}"
        NCGP12_SOURCE_FILE_HASH)
    string(APPEND NCGP12_SOURCE_MATERIAL
        "${NCGP12_SOURCE_FILE}:${NCGP12_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP12_SOURCE_ROOT "${NCGP12_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/10-pressure-state-equilibrium-discriminator.md"
    NCGP12_CONTRACT_10_ROOT)
string(SHA256 NCGP12_CONTRACT_ROOT
    "${NCGP11_CONTRACT_ROOT}:${NCGP12_CONTRACT_10_ROOT}")
target_compile_definitions(
    nonlocal-corrected-cpu-pressure-state
    PRIVATE
        NCGP12_CONTRACT_ROOT="${NCGP12_CONTRACT_ROOT}"
        NCGP12_SOURCE_ROOT="${NCGP12_SOURCE_ROOT}"
        NCGP12_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP12_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP12_COMPILER_FLAGS="CXX:-O3,-Wall,-Wextra,-Wpedantic,-Werror,-ffp-contract=off,-fno-fast-math"
)
target_compile_features(
    nonlocal-corrected-cpu-pressure-state PRIVATE cxx_std_17)
set_target_properties(
    nonlocal-corrected-cpu-pressure-state
    PROPERTIES CXX_EXTENSIONS OFF)
target_compile_options(
    nonlocal-corrected-cpu-pressure-state
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
)
target_link_options(
    nonlocal-corrected-cpu-pressure-state
    PRIVATE
        $<$<CONFIG:Release>:-s>
)

set(NCGP13_SOURCE_FILES
    CMakeLists.txt
    src/corrected_cpu_pressure_contact_main.cpp
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step_reference.cpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP13_SOURCE_MATERIAL "")
foreach(NCGP13_SOURCE_FILE IN LISTS NCGP13_SOURCE_FILES)
    file(SHA256 "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP13_SOURCE_FILE}"
        NCGP13_SOURCE_FILE_HASH)
    string(APPEND NCGP13_SOURCE_MATERIAL
        "${NCGP13_SOURCE_FILE}:${NCGP13_SOURCE_FILE_HASH}\n")
endforeach()
string(SHA256 NCGP13_SOURCE_ROOT "${NCGP13_SOURCE_MATERIAL}")
file(SHA256
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/11-pressure-contact-tiny-trajectory.md"
    NCGP13_CONTRACT_11_ROOT)
string(SHA256 NCGP13_CONTRACT_ROOT
    "${NCGP12_CONTRACT_ROOT}:${NCGP13_CONTRACT_11_ROOT}")
target_compile_definitions(
    nonlocal-corrected-cpu-pressure-contact
    PRIVATE
        NCGP13_CONTRACT_ROOT="${NCGP13_CONTRACT_ROOT}"
        NCGP13_SOURCE_ROOT="${NCGP13_SOURCE_ROOT}"
        NCGP13_SOURCE_COMMIT="${NCGP1_SOURCE_COMMIT}"
        NCGP13_SOURCE_TREE="${NCGP1_SOURCE_TREE}"
        NCGP13_COMPILER_FLAGS="CXX:-O3,-Wall,-Wextra,-Wpedantic,-Werror,-ffp-contract=off,-fno-fast-math"
)
target_compile_features(
    nonlocal-corrected-cpu-pressure-contact PRIVATE cxx_std_17)
set_target_properties(
    nonlocal-corrected-cpu-pressure-contact
    PROPERTIES CXX_EXTENSIONS OFF)
target_compile_options(
    nonlocal-corrected-cpu-pressure-contact
    PRIVATE
        $<$<AND:$<COMPILE_LANGUAGE:CXX>,$<CXX_COMPILER_ID:GNU,Clang>>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
)
target_link_options(
    nonlocal-corrected-cpu-pressure-contact
    PRIVATE
        $<$<CONFIG:Release>:-s>
)

execute_process(
    COMMAND git rev-parse --show-toplevel
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    OUTPUT_VARIABLE NCGP14_REPO_ROOT
    OUTPUT_STRIP_TRAILING_WHITESPACE
    COMMAND_ERROR_IS_FATAL ANY
)
execute_process(
    COMMAND git rev-parse --verify HEAD
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    OUTPUT_VARIABLE NCGP14_SOURCE_COMMIT
    OUTPUT_STRIP_TRAILING_WHITESPACE
    COMMAND_ERROR_IS_FATAL ANY
)
execute_process(
    COMMAND git rev-parse --verify "HEAD^{tree}"
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    OUTPUT_VARIABLE NCGP14_SOURCE_TREE
    OUTPUT_STRIP_TRAILING_WHITESPACE
    COMMAND_ERROR_IS_FATAL ANY
)
execute_process(
    COMMAND git rev-parse --path-format=absolute --git-path HEAD
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    OUTPUT_VARIABLE NCGP14_GIT_HEAD_PATH
    OUTPUT_STRIP_TRAILING_WHITESPACE
    COMMAND_ERROR_IS_FATAL ANY
)
execute_process(
    COMMAND git rev-parse --path-format=absolute --git-path index
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    OUTPUT_VARIABLE NCGP14_GIT_INDEX_PATH
    OUTPUT_STRIP_TRAILING_WHITESPACE
    COMMAND_ERROR_IS_FATAL ANY
)
foreach(NCGP14_REQUIRED_GIT_FILE IN ITEMS
        "${NCGP14_GIT_HEAD_PATH}" "${NCGP14_GIT_INDEX_PATH}")
    if(NOT EXISTS "${NCGP14_REQUIRED_GIT_FILE}"
            OR IS_DIRECTORY "${NCGP14_REQUIRED_GIT_FILE}")
        message(FATAL_ERROR
            "NCGP14 requires a regular Git identity file: ${NCGP14_REQUIRED_GIT_FILE}")
    endif()
endforeach()
set(NCGP14_GIT_DEPENDENCIES
    "${NCGP14_GIT_HEAD_PATH}" "${NCGP14_GIT_INDEX_PATH}")
execute_process(
    COMMAND git symbolic-ref -q HEAD
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    OUTPUT_VARIABLE NCGP14_GIT_SYMBOLIC_REF
    OUTPUT_STRIP_TRAILING_WHITESPACE
    RESULT_VARIABLE NCGP14_GIT_SYMBOLIC_REF_RESULT
)
if(NCGP14_GIT_SYMBOLIC_REF_RESULT EQUAL 0)
    execute_process(
        COMMAND git rev-parse --path-format=absolute --git-path
            "${NCGP14_GIT_SYMBOLIC_REF}"
        WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
        OUTPUT_VARIABLE NCGP14_GIT_REF_PATH
        OUTPUT_STRIP_TRAILING_WHITESPACE
        COMMAND_ERROR_IS_FATAL ANY
    )
    execute_process(
        COMMAND git rev-parse --path-format=absolute --git-path packed-refs
        WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
        OUTPUT_VARIABLE NCGP14_GIT_PACKED_REFS_PATH
        OUTPUT_STRIP_TRAILING_WHITESPACE
        COMMAND_ERROR_IS_FATAL ANY
    )
    execute_process(
        COMMAND git rev-parse --path-format=absolute --git-path
            "logs/${NCGP14_GIT_SYMBOLIC_REF}"
        WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
        OUTPUT_VARIABLE NCGP14_GIT_REF_LOG_PATH
        OUTPUT_STRIP_TRAILING_WHITESPACE
        COMMAND_ERROR_IS_FATAL ANY
    )
    set(NCGP14_GIT_REF_STORAGE_FOUND FALSE)
    foreach(NCGP14_OPTIONAL_GIT_FILE IN ITEMS
            "${NCGP14_GIT_REF_PATH}"
            "${NCGP14_GIT_PACKED_REFS_PATH}"
            "${NCGP14_GIT_REF_LOG_PATH}")
        if(EXISTS "${NCGP14_OPTIONAL_GIT_FILE}"
                AND NOT IS_DIRECTORY "${NCGP14_OPTIONAL_GIT_FILE}")
            list(APPEND NCGP14_GIT_DEPENDENCIES
                "${NCGP14_OPTIONAL_GIT_FILE}")
            set(NCGP14_GIT_REF_STORAGE_FOUND TRUE)
        endif()
    endforeach()
    if(NOT NCGP14_GIT_REF_STORAGE_FOUND)
        message(FATAL_ERROR
            "NCGP14 cannot observe storage for the current symbolic Git ref")
    endif()
elseif(NOT NCGP14_GIT_SYMBOLIC_REF_RESULT EQUAL 1)
    message(FATAL_ERROR "NCGP14 failed to classify Git HEAD")
endif()

function(ncgp14_require_head_blob NCGP14_ABSOLUTE_PATH)
    if(NOT EXISTS "${NCGP14_ABSOLUTE_PATH}"
            OR IS_DIRECTORY "${NCGP14_ABSOLUTE_PATH}"
            OR IS_SYMLINK "${NCGP14_ABSOLUTE_PATH}")
        message(FATAL_ERROR
            "NCGP14 identity leaf is not a regular file: ${NCGP14_ABSOLUTE_PATH}")
    endif()
    file(RELATIVE_PATH NCGP14_REPO_RELATIVE_PATH
        "${NCGP14_REPO_ROOT}" "${NCGP14_ABSOLUTE_PATH}")
    if(NCGP14_REPO_RELATIVE_PATH MATCHES "^\\.\\.(\\/|$)")
        message(FATAL_ERROR
            "NCGP14 identity leaf escapes the repository: ${NCGP14_ABSOLUTE_PATH}")
    endif()
    execute_process(
        COMMAND git rev-parse --verify
            "${NCGP14_SOURCE_COMMIT}:${NCGP14_REPO_RELATIVE_PATH}"
        WORKING_DIRECTORY "${NCGP14_REPO_ROOT}"
        OUTPUT_VARIABLE NCGP14_COMMITTED_BLOB
        OUTPUT_STRIP_TRAILING_WHITESPACE
        RESULT_VARIABLE NCGP14_COMMITTED_BLOB_RESULT
    )
    if(NOT NCGP14_COMMITTED_BLOB_RESULT EQUAL 0)
        message(FATAL_ERROR
            "NCGP14 identity leaf is not owned by the reported commit: ${NCGP14_REPO_RELATIVE_PATH}")
    endif()
    execute_process(
        COMMAND git rev-parse --verify ":${NCGP14_REPO_RELATIVE_PATH}"
        WORKING_DIRECTORY "${NCGP14_REPO_ROOT}"
        OUTPUT_VARIABLE NCGP14_INDEX_BLOB
        OUTPUT_STRIP_TRAILING_WHITESPACE
        RESULT_VARIABLE NCGP14_INDEX_BLOB_RESULT
    )
    if(NOT NCGP14_INDEX_BLOB_RESULT EQUAL 0)
        message(FATAL_ERROR
            "NCGP14 identity leaf is not present at index stage zero: ${NCGP14_REPO_RELATIVE_PATH}")
    endif()
    if(NOT NCGP14_INDEX_BLOB STREQUAL NCGP14_COMMITTED_BLOB)
        message(FATAL_ERROR
            "NCGP14 identity leaf differs between index and reported commit: ${NCGP14_REPO_RELATIVE_PATH}")
    endif()
    execute_process(
        COMMAND git diff --cached --quiet --no-ext-diff
            "${NCGP14_SOURCE_COMMIT}" -- "${NCGP14_REPO_RELATIVE_PATH}"
        WORKING_DIRECTORY "${NCGP14_REPO_ROOT}"
        RESULT_VARIABLE NCGP14_CLEAN_INDEX_RESULT
    )
    if(NOT NCGP14_CLEAN_INDEX_RESULT EQUAL 0)
        message(FATAL_ERROR
            "NCGP14 identity leaf index metadata differs from the reported commit: ${NCGP14_REPO_RELATIVE_PATH}")
    endif()
    execute_process(
        COMMAND git hash-object --no-filters -- "${NCGP14_ABSOLUTE_PATH}"
        WORKING_DIRECTORY "${NCGP14_REPO_ROOT}"
        OUTPUT_VARIABLE NCGP14_WORKTREE_BLOB
        OUTPUT_STRIP_TRAILING_WHITESPACE
        COMMAND_ERROR_IS_FATAL ANY
    )
    if(NOT NCGP14_WORKTREE_BLOB STREQUAL NCGP14_COMMITTED_BLOB)
        message(FATAL_ERROR
            "NCGP14 identity leaf differs from the reported commit: ${NCGP14_REPO_RELATIVE_PATH}")
    endif()
    execute_process(
        COMMAND git diff-files --quiet --no-ext-diff --
            "${NCGP14_REPO_RELATIVE_PATH}"
        WORKING_DIRECTORY "${NCGP14_REPO_ROOT}"
        RESULT_VARIABLE NCGP14_CLEAN_WORKTREE_RESULT
    )
    if(NOT NCGP14_CLEAN_WORKTREE_RESULT EQUAL 0)
        message(FATAL_ERROR
            "NCGP14 identity leaf worktree metadata differs from the index: ${NCGP14_REPO_RELATIVE_PATH}")
    endif()
endfunction()

set(NCGP14_PARENT_SOURCE
    "${CMAKE_CURRENT_SOURCE_DIR}/src/corrected_cpu_pressure_contact_main.cpp")
file(SHA256 "${NCGP14_PARENT_SOURCE}" NCGP14_PARENT_SOURCE_ROOT)
set(NCGP14_FROZEN_PARENT_SOURCE_ROOT
    "ca62174997aaeee928efef9622c9869f7e4f3cf0909fd6f562005684d17e3873")
if(NOT NCGP14_PARENT_SOURCE_ROOT STREQUAL NCGP14_FROZEN_PARENT_SOURCE_ROOT)
    message(FATAL_ERROR
        "NCGP14 requires the byte-exact reviewed NCGP13 parent source")
endif()
set(NCGP14_SOURCE_FILES
    CMakeLists.txt
    src/corrected_cpu_confined_pressure_contact_main.cpp
    src/corrected_cpu_pressure_contact_main.cpp
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step_reference.cpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP14_SOURCE_MATERIAL "")
set(NCGP14_SOURCE_DEPENDENCIES "")
foreach(NCGP14_SOURCE_FILE IN LISTS NCGP14_SOURCE_FILES)
    set(NCGP14_SOURCE_PATH
        "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP14_SOURCE_FILE}")
    list(APPEND NCGP14_SOURCE_DEPENDENCIES "${NCGP14_SOURCE_PATH}")
    ncgp14_require_head_blob("${NCGP14_SOURCE_PATH}")
    file(SHA256 "${NCGP14_SOURCE_PATH}"
        NCGP14_SOURCE_FILE_ROOT)
    string(APPEND NCGP14_SOURCE_MATERIAL
        "${NCGP14_SOURCE_FILE}:${NCGP14_SOURCE_FILE_ROOT}\n")
endforeach()
string(SHA256 NCGP14_SOURCE_ROOT "${NCGP14_SOURCE_MATERIAL}")
set(NCGP14_CONTRACT_12_PATH
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/12-confined-pressure-contact-discriminator.md")
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
    ${NCGP14_SOURCE_DEPENDENCIES}
    ${NCGP14_GIT_DEPENDENCIES}
    "${NCGP14_CONTRACT_12_PATH}")
ncgp14_require_head_blob("${NCGP14_CONTRACT_12_PATH}")
file(SHA256 "${NCGP14_CONTRACT_12_PATH}" NCGP14_CONTRACT_12_ROOT)
set(NCGP14_FROZEN_CONTRACT_12_ROOT
    "4573690e22e79c999b2cdcd609747d0cb061ee59df475dd9040450c6678fd880")
if(NOT NCGP14_CONTRACT_12_ROOT STREQUAL NCGP14_FROZEN_CONTRACT_12_ROOT)
    message(FATAL_ERROR "NCGP14 frozen contract file identity mismatch")
endif()
set(NCGP14_FROZEN_PARENT_CONTRACT_ROOT
    "3c26ed0b806308113384e54198935828bf61fb4a5447cc7311b78285e6b3c904")
string(SHA256 NCGP14_CONTRACT_ROOT
    "${NCGP14_FROZEN_PARENT_CONTRACT_ROOT}:${NCGP14_CONTRACT_12_ROOT}")
set(NCGP14_FROZEN_CONTRACT_ROOT
    "e6d9cdce3a67818024c68ad2da7f4d2405613b7b27953678530b4a415609779f")
if(NOT NCGP14_CONTRACT_ROOT STREQUAL NCGP14_FROZEN_CONTRACT_ROOT)
    message(FATAL_ERROR "NCGP14 nested contract identity mismatch")
endif()
if(CMAKE_CONFIGURATION_TYPES)
    message(FATAL_ERROR "NCGP14 requires a single-config Release generator")
endif()
if(NOT CMAKE_BUILD_TYPE STREQUAL "Release")
    message(FATAL_ERROR "NCGP14 evidence target requires CMAKE_BUILD_TYPE=Release")
endif()
if(NOT CMAKE_CXX_COMPILER_ID STREQUAL "GNU"
        AND NOT CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    message(FATAL_ERROR "NCGP14 requires the frozen GNU/Clang compiler family")
endif()
string(STRIP "${CMAKE_CXX_FLAGS}" NCGP14_GLOBAL_CXX_FLAGS)
string(STRIP "${CMAKE_CXX_FLAGS_RELEASE}" NCGP14_RELEASE_CXX_FLAGS)
string(STRIP "${CMAKE_EXE_LINKER_FLAGS}" NCGP14_GLOBAL_LINK_FLAGS)
string(STRIP "${CMAKE_EXE_LINKER_FLAGS_RELEASE}"
    NCGP14_RELEASE_LINK_FLAGS)
if(NOT NCGP14_GLOBAL_CXX_FLAGS STREQUAL "")
    message(FATAL_ERROR "NCGP14 rejects ambient CMAKE_CXX_FLAGS")
endif()
if(NOT NCGP14_RELEASE_CXX_FLAGS STREQUAL "-O3 -DNDEBUG")
    message(FATAL_ERROR "NCGP14 requires exact Release CXX defaults")
endif()
if(NOT NCGP14_GLOBAL_LINK_FLAGS STREQUAL ""
        OR NOT NCGP14_RELEASE_LINK_FLAGS STREQUAL "")
    message(FATAL_ERROR "NCGP14 rejects ambient executable linker flags")
endif()
if(CMAKE_CXX_COMPILER_ARG1)
    message(FATAL_ERROR "NCGP14 rejects CMAKE_CXX_COMPILER_ARG1")
endif()
if(CMAKE_CXX_COMPILER_LAUNCHER)
    message(FATAL_ERROR "NCGP14 rejects compiler launchers")
endif()
if(CMAKE_INTERPROCEDURAL_OPTIMIZATION
        OR CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELEASE)
    message(FATAL_ERROR "NCGP14 rejects implicit interprocedural optimization")
endif()
if(DEFINED CMAKE_CXX_STANDARD
        AND NOT CMAKE_CXX_STANDARD STREQUAL ""
        AND NOT CMAKE_CXX_STANDARD STREQUAL "17")
    message(FATAL_ERROR "NCGP14 rejects every ambient C++ standard except exact decimal 17")
endif()
set(NCGP14_TARGET_CXX_OPTIONS
    -O3
    -Wall
    -Wextra
    -Wpedantic
    -Werror
    -ffp-contract=off
    -fno-fast-math
)
string(JOIN "," NCGP14_TARGET_CXX_FLAGS_CSV
    ${NCGP14_TARGET_CXX_OPTIONS})
set(NCGP14_EFFECTIVE_COMPILER_FLAGS
    "CXX_STANDARD:-std=c++17|CXX_RELEASE:-O3,-DNDEBUG|CXX_TARGET:${NCGP14_TARGET_CXX_FLAGS_CSV}|LINK_RELEASE:-s")
target_compile_definitions(
    nonlocal-corrected-cpu-confined-pressure-contact
    PRIVATE
        NCGP13_CONTRACT_ROOT="${NCGP14_FROZEN_PARENT_CONTRACT_ROOT}"
        NCGP13_SOURCE_ROOT="fe71ebecca1bbac5232927931a65186d145d6f2b093ee48d28f24c505ae28af7"
        NCGP13_SOURCE_COMMIT="b1fdcd59fa105e48039bd4c1b5dda72bf7401a64"
        NCGP13_SOURCE_TREE="c541413c73fee10186c1f9652264b7b64dd473cf"
        NCGP13_COMPILER_FLAGS="CXX:-O3,-Wall,-Wextra,-Wpedantic,-Werror,-ffp-contract=off,-fno-fast-math"
        NCGP14_CONTRACT_ROOT="${NCGP14_CONTRACT_ROOT}"
        NCGP14_SOURCE_ROOT="${NCGP14_SOURCE_ROOT}"
        NCGP14_SOURCE_COMMIT="${NCGP14_SOURCE_COMMIT}"
        NCGP14_SOURCE_TREE="${NCGP14_SOURCE_TREE}"
        NCGP14_COMPILER_FAMILY="${CMAKE_CXX_COMPILER_ID}"
        NCGP14_COMPILER_VERSION="${CMAKE_CXX_COMPILER_VERSION}"
        NCGP14_COMPILER_FLAGS="${NCGP14_EFFECTIVE_COMPILER_FLAGS}"
)
set_target_properties(
    nonlocal-corrected-cpu-confined-pressure-contact
    PROPERTIES
        CXX_STANDARD 17
        CXX_STANDARD_REQUIRED YES
        CXX_EXTENSIONS NO
        EXPORT_COMPILE_COMMANDS ON
        UNITY_BUILD OFF)
target_include_directories(
    nonlocal-corrected-cpu-unified-constrained
    PRIVATE
        "${CMAKE_CURRENT_BINARY_DIR}/CMakeFiles")
target_compile_options(
    nonlocal-corrected-cpu-confined-pressure-contact
    PRIVATE
        ${NCGP14_TARGET_CXX_OPTIONS}
)
target_link_options(
    nonlocal-corrected-cpu-confined-pressure-contact
    PRIVATE
        -s
)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(NCGP14_COMPILE_PROFILE_CHECK_SCRIPT
    "${CMAKE_CURRENT_BINARY_DIR}/CMakeFiles/ncgp14_verify_compile_profile.cmake")
file(WRITE "${NCGP14_COMPILE_PROFILE_CHECK_SCRIPT}" [=[
if(NOT DEFINED NCGP14_COMPILE_COMMANDS
        OR NCGP14_COMPILE_COMMANDS STREQUAL "")
    message(FATAL_ERROR "NCGP14 compile-command path is unavailable")
endif()
if(NOT EXISTS "${NCGP14_COMPILE_COMMANDS}"
        OR IS_DIRECTORY "${NCGP14_COMPILE_COMMANDS}")
    message(FATAL_ERROR
        "NCGP14 generated compile commands are unavailable: ${NCGP14_COMPILE_COMMANDS}")
endif()
if(NOT DEFINED NCGP14_TARGET_MARKER OR NCGP14_TARGET_MARKER STREQUAL "")
    message(FATAL_ERROR "NCGP14 target marker is unavailable")
endif()
if(NOT DEFINED NCGP14_EXPECTED_STANDARD
        OR NCGP14_EXPECTED_STANDARD STREQUAL "")
    message(FATAL_ERROR "NCGP14 strict C++17 selector is unavailable")
endif()

file(READ "${NCGP14_COMPILE_COMMANDS}" NCGP14_COMPILE_DATABASE)
string(JSON NCGP14_COMPILE_COMMAND_COUNT ERROR_VARIABLE NCGP14_JSON_ERROR
    LENGTH "${NCGP14_COMPILE_DATABASE}")
if(NCGP14_JSON_ERROR)
    message(FATAL_ERROR
        "NCGP14 generated compile commands are not valid JSON: ${NCGP14_JSON_ERROR}")
endif()
if(NCGP14_COMPILE_COMMAND_COUNT LESS 1)
    message(FATAL_ERROR "NCGP14 generated compile-command database is empty")
endif()

set(NCGP14_EXPECTED_SOURCE_NAMES
    corrected_cpu_confined_pressure_contact_main.cpp
    corrected_cuda_full_step_reference.cpp
    sha256.cpp)
set(NCGP14_TARGET_COMMAND_COUNT 0)
math(EXPR NCGP14_LAST_COMPILE_COMMAND "${NCGP14_COMPILE_COMMAND_COUNT} - 1")
foreach(NCGP14_COMPILE_COMMAND_INDEX RANGE 0 ${NCGP14_LAST_COMPILE_COMMAND})
    string(JSON NCGP14_COMPILE_COMMAND ERROR_VARIABLE NCGP14_COMMAND_ERROR
        GET "${NCGP14_COMPILE_DATABASE}"
        ${NCGP14_COMPILE_COMMAND_INDEX} command)
    if(NCGP14_COMMAND_ERROR)
        message(FATAL_ERROR
            "NCGP14 compile-command entry has no command string: ${NCGP14_COMMAND_ERROR}")
    endif()
    string(FIND "${NCGP14_COMPILE_COMMAND}" "${NCGP14_TARGET_MARKER}"
        NCGP14_TARGET_MARKER_OFFSET)
    if(NCGP14_TARGET_MARKER_OFFSET EQUAL -1)
        continue()
    endif()

    string(JSON NCGP14_COMPILE_SOURCE ERROR_VARIABLE NCGP14_SOURCE_ERROR
        GET "${NCGP14_COMPILE_DATABASE}"
        ${NCGP14_COMPILE_COMMAND_INDEX} file)
    if(NCGP14_SOURCE_ERROR)
        message(FATAL_ERROR
            "NCGP14 compile-command entry has no source path: ${NCGP14_SOURCE_ERROR}")
    endif()
    get_filename_component(NCGP14_COMPILE_SOURCE_NAME
        "${NCGP14_COMPILE_SOURCE}" NAME)
    list(FIND NCGP14_EXPECTED_SOURCE_NAMES "${NCGP14_COMPILE_SOURCE_NAME}"
        NCGP14_EXPECTED_SOURCE_INDEX)
    if(NCGP14_EXPECTED_SOURCE_INDEX EQUAL -1)
        message(FATAL_ERROR
            "NCGP14 generated target has an unexpected source command: ${NCGP14_COMPILE_SOURCE}")
    endif()
    string(MAKE_C_IDENTIFIER "${NCGP14_COMPILE_SOURCE_NAME}"
        NCGP14_COMPILE_SOURCE_KEY)
    if(DEFINED NCGP14_SEEN_${NCGP14_COMPILE_SOURCE_KEY})
        message(FATAL_ERROR
            "NCGP14 generated target has a duplicate source command: ${NCGP14_COMPILE_SOURCE_NAME}")
    endif()
    set(NCGP14_SEEN_${NCGP14_COMPILE_SOURCE_KEY} TRUE)

    separate_arguments(NCGP14_COMPILE_ARGUMENTS NATIVE_COMMAND
        "${NCGP14_COMPILE_COMMAND}")
    set(NCGP14_STANDARD_SELECTOR_COUNT 0)
    foreach(NCGP14_COMPILE_ARGUMENT IN LISTS NCGP14_COMPILE_ARGUMENTS)
        if(NCGP14_COMPILE_ARGUMENT MATCHES "^(-std=|/std:)")
            math(EXPR NCGP14_STANDARD_SELECTOR_COUNT
                "${NCGP14_STANDARD_SELECTOR_COUNT} + 1")
            if(NOT NCGP14_COMPILE_ARGUMENT STREQUAL NCGP14_EXPECTED_STANDARD)
                message(FATAL_ERROR
                    "NCGP14 generated command contains a conflicting language selector: ${NCGP14_COMPILE_ARGUMENT}")
            endif()
        endif()
    endforeach()
    if(NOT NCGP14_STANDARD_SELECTOR_COUNT EQUAL 1)
        message(FATAL_ERROR
            "NCGP14 generated command must contain exactly one strict C++17 selector: ${NCGP14_COMPILE_SOURCE_NAME}")
    endif()
    math(EXPR NCGP14_TARGET_COMMAND_COUNT
        "${NCGP14_TARGET_COMMAND_COUNT} + 1")
endforeach()

list(LENGTH NCGP14_EXPECTED_SOURCE_NAMES NCGP14_EXPECTED_COMMAND_COUNT)
if(NOT NCGP14_TARGET_COMMAND_COUNT EQUAL NCGP14_EXPECTED_COMMAND_COUNT)
    message(FATAL_ERROR
        "NCGP14 generated target does not contain the exact frozen source command set")
endif()
foreach(NCGP14_EXPECTED_SOURCE_NAME IN LISTS NCGP14_EXPECTED_SOURCE_NAMES)
    string(MAKE_C_IDENTIFIER "${NCGP14_EXPECTED_SOURCE_NAME}"
        NCGP14_EXPECTED_SOURCE_KEY)
    if(NOT DEFINED NCGP14_SEEN_${NCGP14_EXPECTED_SOURCE_KEY})
        message(FATAL_ERROR
            "NCGP14 generated target is missing a frozen source command: ${NCGP14_EXPECTED_SOURCE_NAME}")
    endif()
endforeach()
]=])
add_custom_target(
    ncgp14-verify-compile-profile
    COMMAND "${CMAKE_COMMAND}"
        "-DNCGP14_COMPILE_COMMANDS=${CMAKE_BINARY_DIR}/compile_commands.json"
        "-DNCGP14_TARGET_MARKER=CMakeFiles/nonlocal-corrected-cpu-confined-pressure-contact.dir/"
        "-DNCGP14_EXPECTED_STANDARD=-std=c++17"
        -P "${NCGP14_COMPILE_PROFILE_CHECK_SCRIPT}"
    COMMENT "Verifying the generated NCGP14 strict C++17 compile profile"
    VERBATIM
)
add_dependencies(
    nonlocal-corrected-cpu-confined-pressure-contact
    ncgp14-verify-compile-profile)

set(NCGP15_SOURCE_COMMIT "${NCGP14_SOURCE_COMMIT}")
set(NCGP15_SOURCE_TREE "${NCGP14_SOURCE_TREE}")
set(NCGP15_REPO_ROOT "${NCGP14_REPO_ROOT}")

function(ncgp15_require_head_blob NCGP15_ABSOLUTE_PATH)
    ncgp14_require_head_blob("${NCGP15_ABSOLUTE_PATH}")
endfunction()

set(NCGP15_FROZEN_PARENT_SOURCE_FILES
    src/corrected_cpu_confined_pressure_contact_main.cpp
    src/corrected_cpu_pressure_contact_main.cpp
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step_reference.cpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP15_FROZEN_PARENT_SOURCE_ROOTS
    c1c1b454a383b0d5c53eda5b4567da809597dbefc9664e2dd260ce7b56ad786e
    ca62174997aaeee928efef9622c9869f7e4f3cf0909fd6f562005684d17e3873
    edef5f3f6983f79d4fda9658ae78f0badb761b8a48b43666fb1ba850292e836a
    1de41781e2fe140a3ae76e37341bf904add26efc615a27b5e9684b5255d30a0b
    a2c7ef6d047e4c00aaf0a83ea364bdf293a521245b382b86782d205ece3fe13b
    2551f6fb6bfcd6ce30f801f46979f72a5e1bda92a69e1530ee03eb509ba0c76e
)
list(LENGTH NCGP15_FROZEN_PARENT_SOURCE_FILES
    NCGP15_FROZEN_PARENT_SOURCE_FILE_COUNT)
list(LENGTH NCGP15_FROZEN_PARENT_SOURCE_ROOTS
    NCGP15_FROZEN_PARENT_SOURCE_ROOT_COUNT)
if(NOT NCGP15_FROZEN_PARENT_SOURCE_FILE_COUNT EQUAL
        NCGP15_FROZEN_PARENT_SOURCE_ROOT_COUNT)
    message(FATAL_ERROR "NCGP15 frozen parent leaf table is malformed")
endif()
math(EXPR NCGP15_FROZEN_PARENT_LAST_INDEX
    "${NCGP15_FROZEN_PARENT_SOURCE_FILE_COUNT} - 1")
foreach(NCGP15_FROZEN_PARENT_INDEX RANGE
        0 ${NCGP15_FROZEN_PARENT_LAST_INDEX})
    list(GET NCGP15_FROZEN_PARENT_SOURCE_FILES
        ${NCGP15_FROZEN_PARENT_INDEX} NCGP15_FROZEN_PARENT_SOURCE_FILE)
    list(GET NCGP15_FROZEN_PARENT_SOURCE_ROOTS
        ${NCGP15_FROZEN_PARENT_INDEX} NCGP15_FROZEN_PARENT_EXPECTED_ROOT)
    set(NCGP15_FROZEN_PARENT_SOURCE_PATH
        "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP15_FROZEN_PARENT_SOURCE_FILE}")
    ncgp15_require_head_blob("${NCGP15_FROZEN_PARENT_SOURCE_PATH}")
    file(SHA256 "${NCGP15_FROZEN_PARENT_SOURCE_PATH}"
        NCGP15_FROZEN_PARENT_OBSERVED_ROOT)
    if(NOT NCGP15_FROZEN_PARENT_OBSERVED_ROOT STREQUAL
            NCGP15_FROZEN_PARENT_EXPECTED_ROOT)
        message(FATAL_ERROR
            "NCGP15 frozen parent leaf identity mismatch: ${NCGP15_FROZEN_PARENT_SOURCE_FILE}")
    endif()
endforeach()

set(NCGP15_SOURCE_FILES
    CMakeLists.txt
    src/corrected_cpu_unified_constrained.hpp
    src/corrected_cpu_unified_constrained_candidate.cpp
    src/corrected_cpu_unified_constrained_oracle.cpp
    src/corrected_cpu_unified_constrained_main.cpp
    src/corrected_cpu_confined_pressure_contact_main.cpp
    src/corrected_cpu_pressure_contact_main.cpp
    src/corrected_cuda_full_step.hpp
    src/corrected_cuda_full_step_reference.cpp
    src/sha256.cpp
    src/sha256.hpp
)
set(NCGP15_SOURCE_MATERIAL "")
set(NCGP15_SOURCE_DEPENDENCIES "")
foreach(NCGP15_SOURCE_FILE IN LISTS NCGP15_SOURCE_FILES)
    set(NCGP15_SOURCE_PATH
        "${CMAKE_CURRENT_SOURCE_DIR}/${NCGP15_SOURCE_FILE}")
    list(APPEND NCGP15_SOURCE_DEPENDENCIES "${NCGP15_SOURCE_PATH}")
    ncgp15_require_head_blob("${NCGP15_SOURCE_PATH}")
    file(SHA256 "${NCGP15_SOURCE_PATH}" NCGP15_SOURCE_FILE_ROOT)
    string(APPEND NCGP15_SOURCE_MATERIAL
        "${NCGP15_SOURCE_FILE}:${NCGP15_SOURCE_FILE_ROOT}\n")
endforeach()
string(SHA256 NCGP15_SOURCE_ROOT "${NCGP15_SOURCE_MATERIAL}")

set(NCGP15_CONTRACT_13_PATH
    "${CMAKE_CURRENT_SOURCE_DIR}/../../../../docs/plans/nonlocal-gpu-full-step-performance/13-unified-constrained-surface-viscosity-step.md")
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
    ${NCGP15_SOURCE_DEPENDENCIES}
    ${NCGP14_GIT_DEPENDENCIES}
    "${NCGP15_CONTRACT_13_PATH}")
ncgp15_require_head_blob("${NCGP15_CONTRACT_13_PATH}")
file(SHA256 "${NCGP15_CONTRACT_13_PATH}" NCGP15_CONTRACT_13_ROOT)
set(NCGP15_FROZEN_CONTRACT_13_ROOT
    "3c1b486094828efb6d6887a7d75d50131de6e4e844426725916bf414f4454f5b")
if(NOT NCGP15_CONTRACT_13_ROOT STREQUAL
        NCGP15_FROZEN_CONTRACT_13_ROOT)
    message(FATAL_ERROR "NCGP15 frozen Revision-5 contract identity mismatch")
endif()
set(NCGP15_CONTRACT_ROOT "${NCGP15_CONTRACT_13_ROOT}")

if(CMAKE_CONFIGURATION_TYPES)
    message(FATAL_ERROR "NCGP15 requires a single-config Release generator")
endif()
if(NOT CMAKE_BUILD_TYPE STREQUAL "Release")
    message(FATAL_ERROR "NCGP15 evidence target requires CMAKE_BUILD_TYPE=Release")
endif()
if(NOT CMAKE_CXX_COMPILER_ID STREQUAL "GNU"
        AND NOT CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    message(FATAL_ERROR "NCGP15 requires the frozen GNU/Clang compiler family")
endif()
string(STRIP "${CMAKE_CXX_FLAGS}" NCGP15_GLOBAL_CXX_FLAGS)
string(STRIP "${CMAKE_CXX_FLAGS_RELEASE}" NCGP15_RELEASE_CXX_FLAGS)
string(STRIP "${CMAKE_EXE_LINKER_FLAGS}" NCGP15_GLOBAL_LINK_FLAGS)
string(STRIP "${CMAKE_EXE_LINKER_FLAGS_RELEASE}"
    NCGP15_RELEASE_LINK_FLAGS)
if(NOT NCGP15_GLOBAL_CXX_FLAGS STREQUAL "")
    message(FATAL_ERROR "NCGP15 rejects ambient CMAKE_CXX_FLAGS")
endif()
if(NOT NCGP15_RELEASE_CXX_FLAGS STREQUAL "-O3 -DNDEBUG")
    message(FATAL_ERROR "NCGP15 requires exact Release CXX defaults")
endif()
if(NOT NCGP15_GLOBAL_LINK_FLAGS STREQUAL ""
        OR NOT NCGP15_RELEASE_LINK_FLAGS STREQUAL "")
    message(FATAL_ERROR "NCGP15 rejects ambient executable linker flags")
endif()
if(CMAKE_CXX_COMPILER_ARG1)
    message(FATAL_ERROR "NCGP15 rejects CMAKE_CXX_COMPILER_ARG1")
endif()
if(CMAKE_CXX_COMPILER_LAUNCHER)
    message(FATAL_ERROR "NCGP15 rejects compiler launchers")
endif()
if(CMAKE_INTERPROCEDURAL_OPTIMIZATION
        OR CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELEASE)
    message(FATAL_ERROR "NCGP15 rejects implicit interprocedural optimization")
endif()
if(DEFINED CMAKE_CXX_STANDARD
        AND NOT CMAKE_CXX_STANDARD STREQUAL ""
        AND NOT CMAKE_CXX_STANDARD STREQUAL "17")
    message(FATAL_ERROR
        "NCGP15 rejects every ambient C++ standard except exact decimal 17")
endif()

set(NCGP15_TARGET_CXX_OPTIONS
    -O3
    -Wall
    -Wextra
    -Wpedantic
    -Werror
    -ffp-contract=off
    -fno-fast-math
)
string(JOIN "," NCGP15_TARGET_CXX_FLAGS_CSV
    ${NCGP15_TARGET_CXX_OPTIONS})
set(NCGP15_EFFECTIVE_COMPILER_FLAGS
    "CXX_STANDARD:-std=c++17|CXX_RELEASE:-O3,-DNDEBUG|CXX_TARGET:${NCGP15_TARGET_CXX_FLAGS_CSV}|LINK_RELEASE:-s")
target_compile_definitions(
    nonlocal-corrected-cpu-unified-constrained
    PRIVATE
        NCGP13_CONTRACT_ROOT="3c26ed0b806308113384e54198935828bf61fb4a5447cc7311b78285e6b3c904"
        NCGP13_SOURCE_ROOT="fe71ebecca1bbac5232927931a65186d145d6f2b093ee48d28f24c505ae28af7"
        NCGP13_SOURCE_COMMIT="b1fdcd59fa105e48039bd4c1b5dda72bf7401a64"
        NCGP13_SOURCE_TREE="c541413c73fee10186c1f9652264b7b64dd473cf"
        NCGP13_COMPILER_FLAGS="CXX:-O3,-Wall,-Wextra,-Wpedantic,-Werror,-ffp-contract=off,-fno-fast-math"
        NCGP14_CONTRACT_ROOT="e6d9cdce3a67818024c68ad2da7f4d2405613b7b27953678530b4a415609779f"
        NCGP14_SOURCE_ROOT="a3f980e56dff5ce0e599f74b2048092ebac4d4b2d875f8d1c2a4d918a40ccd16"
        NCGP14_SOURCE_COMMIT="d1cfe76c4d2a342e99dc0954ba7e322709af1784"
        NCGP14_SOURCE_TREE="bbdf9f83918bd21698396e069907b6c3aa198255"
        NCGP14_COMPILER_FAMILY="${CMAKE_CXX_COMPILER_ID}"
        NCGP14_COMPILER_VERSION="${CMAKE_CXX_COMPILER_VERSION}"
        NCGP14_COMPILER_FLAGS="${NCGP15_EFFECTIVE_COMPILER_FLAGS}"
        NCGP15_CONTRACT_ROOT="${NCGP15_CONTRACT_ROOT}"
        NCGP15_CONTRACT_SHA="${NCGP15_CONTRACT_13_ROOT}"
        NCGP15_SOURCE_ROOT="${NCGP15_SOURCE_ROOT}"
        NCGP15_SOURCE_COMMIT="${NCGP15_SOURCE_COMMIT}"
        NCGP15_SOURCE_TREE="${NCGP15_SOURCE_TREE}"
        NCGP15_COMPILER_FAMILY="${CMAKE_CXX_COMPILER_ID}"
        NCGP15_COMPILER_VERSION="${CMAKE_CXX_COMPILER_VERSION}"
        NCGP15_COMPILER_FLAGS="${NCGP15_EFFECTIVE_COMPILER_FLAGS}"
        NCGP15_PARENT_CONTRACT_FILE_SHA="4573690e22e79c999b2cdcd609747d0cb061ee59df475dd9040450c6678fd880"
        NCGP15_PARENT_CONTRACT_ROOT="e6d9cdce3a67818024c68ad2da7f4d2405613b7b27953678530b4a415609779f"
        NCGP15_PARENT_SOURCE_ROOT="a3f980e56dff5ce0e599f74b2048092ebac4d4b2d875f8d1c2a4d918a40ccd16"
        NCGP15_PARENT_SOURCE_COMMIT="d1cfe76c4d2a342e99dc0954ba7e322709af1784"
        NCGP15_PARENT_SOURCE_TREE="bbdf9f83918bd21698396e069907b6c3aa198255"
        NCGP15_PARENT_BINARY_ROOT="f924209d34ce75ed679bda043156d0d4f79ec13f266ea7ce4bdd6cb5141cab14"
        NCGP15_PARENT_STDOUT_ROOT="15a92dff1e927f4137c43a607a9b31e30200a76e1ebd49a99127543d7a98bd37"
        NCGP15_PARENT_RESULT_ROOT="54c89f7a0bd4fd13920db325a2b401591cd8cfca3690fc694440d28b42f54221"
        NCGP15_PARENT_TIGHT_FIXTURE_ROOT="6dbaddf563b825e28e37aee58606f25cf50b17479cc3260e919103be79f7c2b2"
        NCGP15_PARENT_TIGHT_LANE_ROOT="8d13d0eefb7e27acafb0588c052ed252a1246ec8e3c5cd5ebe4c748b59796ba6"
)
set_target_properties(
    nonlocal-corrected-cpu-unified-constrained
    PROPERTIES
        CXX_STANDARD 17
        CXX_STANDARD_REQUIRED YES
        CXX_EXTENSIONS NO
        EXPORT_COMPILE_COMMANDS ON
        UNITY_BUILD OFF)
target_include_directories(
    nonlocal-corrected-cpu-unified-constrained
    PRIVATE
        "${CMAKE_CURRENT_BINARY_DIR}/CMakeFiles")
target_compile_options(
    nonlocal-corrected-cpu-unified-constrained
    PRIVATE
        ${NCGP15_TARGET_CXX_OPTIONS}
)
target_link_options(
    nonlocal-corrected-cpu-unified-constrained
    PRIVATE
        -s
)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(NCGP15_COMPILE_PROFILE_CHECK_SCRIPT
    "${CMAKE_CURRENT_BINARY_DIR}/CMakeFiles/ncgp15_verify_compile_profile.cmake")
file(WRITE "${NCGP15_COMPILE_PROFILE_CHECK_SCRIPT}" [=[
if(NOT DEFINED NCGP15_COMPILE_COMMANDS
        OR NCGP15_COMPILE_COMMANDS STREQUAL "")
    message(FATAL_ERROR "NCGP15 compile-command path is unavailable")
endif()
if(NOT EXISTS "${NCGP15_COMPILE_COMMANDS}"
        OR IS_DIRECTORY "${NCGP15_COMPILE_COMMANDS}")
    message(FATAL_ERROR
        "NCGP15 generated compile commands are unavailable: ${NCGP15_COMPILE_COMMANDS}")
endif()
get_filename_component(NCGP15_COMPILE_DATABASE_ROOT
    "${NCGP15_COMPILE_COMMANDS}" DIRECTORY)
if(NCGP15_COMPILE_DATABASE_ROOT STREQUAL ""
        OR NCGP15_COMPILE_DATABASE_ROOT STREQUAL "/")
    message(FATAL_ERROR
        "NCGP15 compile-command build root is not a bounded directory")
endif()
if(NOT IS_ABSOLUTE "${NCGP15_COMPILE_DATABASE_ROOT}")
    get_filename_component(NCGP15_COMPILE_DATABASE_ROOT
        "${NCGP15_COMPILE_DATABASE_ROOT}" ABSOLUTE)
endif()
file(TO_CMAKE_PATH "${NCGP15_COMPILE_DATABASE_ROOT}"
    NCGP15_COMPILE_DATABASE_ROOT)
if(NOT DEFINED NCGP15_TARGET_MARKER OR NCGP15_TARGET_MARKER STREQUAL "")
    message(FATAL_ERROR "NCGP15 target marker is unavailable")
endif()
if(NOT DEFINED NCGP15_EXPECTED_STANDARD
        OR NCGP15_EXPECTED_STANDARD STREQUAL "")
    message(FATAL_ERROR "NCGP15 strict C++17 selector is unavailable")
endif()
if(NOT DEFINED NCGP15_PROFILE_HEADER OR NCGP15_PROFILE_HEADER STREQUAL "")
    message(FATAL_ERROR "NCGP15 generated compile-profile header is unavailable")
endif()

file(READ "${NCGP15_COMPILE_COMMANDS}" NCGP15_COMPILE_DATABASE)
string(JSON NCGP15_COMPILE_COMMAND_COUNT ERROR_VARIABLE NCGP15_JSON_ERROR
    LENGTH "${NCGP15_COMPILE_DATABASE}")
if(NCGP15_JSON_ERROR)
    message(FATAL_ERROR
        "NCGP15 generated compile commands are not valid JSON: ${NCGP15_JSON_ERROR}")
endif()
if(NCGP15_COMPILE_COMMAND_COUNT LESS 1)
    message(FATAL_ERROR "NCGP15 generated compile-command database is empty")
endif()

set(NCGP15_EXPECTED_SOURCE_NAMES
    corrected_cpu_unified_constrained_candidate.cpp
    corrected_cpu_unified_constrained_oracle.cpp
    corrected_cpu_unified_constrained_main.cpp
    corrected_cuda_full_step_reference.cpp
    sha256.cpp)
set(NCGP15_TARGET_COMMAND_COUNT 0)
set(NCGP15_COMMAND_MATERIAL "")
math(EXPR NCGP15_LAST_COMPILE_COMMAND
    "${NCGP15_COMPILE_COMMAND_COUNT} - 1")
foreach(NCGP15_COMPILE_COMMAND_INDEX RANGE 0 ${NCGP15_LAST_COMPILE_COMMAND})
    string(JSON NCGP15_COMPILE_COMMAND ERROR_VARIABLE NCGP15_COMMAND_ERROR
        GET "${NCGP15_COMPILE_DATABASE}"
        ${NCGP15_COMPILE_COMMAND_INDEX} command)
    if(NCGP15_COMMAND_ERROR)
        message(FATAL_ERROR
            "NCGP15 compile-command entry has no command string: ${NCGP15_COMMAND_ERROR}")
    endif()
    string(FIND "${NCGP15_COMPILE_COMMAND}" "${NCGP15_TARGET_MARKER}"
        NCGP15_TARGET_MARKER_OFFSET)
    if(NCGP15_TARGET_MARKER_OFFSET EQUAL -1)
        continue()
    endif()

    string(JSON NCGP15_COMPILE_SOURCE ERROR_VARIABLE NCGP15_SOURCE_ERROR
        GET "${NCGP15_COMPILE_DATABASE}"
        ${NCGP15_COMPILE_COMMAND_INDEX} file)
    if(NCGP15_SOURCE_ERROR)
        message(FATAL_ERROR
            "NCGP15 compile-command entry has no source path: ${NCGP15_SOURCE_ERROR}")
    endif()
    get_filename_component(NCGP15_COMPILE_SOURCE_NAME
        "${NCGP15_COMPILE_SOURCE}" NAME)
    list(FIND NCGP15_EXPECTED_SOURCE_NAMES "${NCGP15_COMPILE_SOURCE_NAME}"
        NCGP15_EXPECTED_SOURCE_INDEX)
    if(NCGP15_EXPECTED_SOURCE_INDEX EQUAL -1)
        message(FATAL_ERROR
            "NCGP15 generated target has an unexpected source command: ${NCGP15_COMPILE_SOURCE}")
    endif()
    string(MAKE_C_IDENTIFIER "${NCGP15_COMPILE_SOURCE_NAME}"
        NCGP15_COMPILE_SOURCE_KEY)
    if(DEFINED NCGP15_SEEN_${NCGP15_COMPILE_SOURCE_KEY})
        message(FATAL_ERROR
            "NCGP15 generated target has a duplicate source command: ${NCGP15_COMPILE_SOURCE_NAME}")
    endif()
    set(NCGP15_SEEN_${NCGP15_COMPILE_SOURCE_KEY} TRUE)

    separate_arguments(NCGP15_COMPILE_ARGUMENTS NATIVE_COMMAND
        "${NCGP15_COMPILE_COMMAND}")
    set(NCGP15_STANDARD_SELECTOR_COUNT 0)
    set(NCGP15_O3_COUNT 0)
    set(NCGP15_DNDEBUG_COUNT 0)
    set(NCGP15_WALL_COUNT 0)
    set(NCGP15_WEXTRA_COUNT 0)
    set(NCGP15_WPEDANTIC_COUNT 0)
    set(NCGP15_WERROR_COUNT 0)
    set(NCGP15_FFP_CONTRACT_COUNT 0)
    set(NCGP15_NO_FAST_MATH_COUNT 0)
    foreach(NCGP15_COMPILE_ARGUMENT IN LISTS NCGP15_COMPILE_ARGUMENTS)
        if(NCGP15_COMPILE_ARGUMENT MATCHES "^(-std=|/std:)")
            math(EXPR NCGP15_STANDARD_SELECTOR_COUNT
                "${NCGP15_STANDARD_SELECTOR_COUNT} + 1")
            if(NOT NCGP15_COMPILE_ARGUMENT STREQUAL
                    NCGP15_EXPECTED_STANDARD)
                message(FATAL_ERROR
                    "NCGP15 generated command contains a conflicting language selector: ${NCGP15_COMPILE_ARGUMENT}")
            endif()
        elseif(NCGP15_COMPILE_ARGUMENT STREQUAL "-O3")
            math(EXPR NCGP15_O3_COUNT "${NCGP15_O3_COUNT} + 1")
        elseif(NCGP15_COMPILE_ARGUMENT MATCHES "^-O")
            message(FATAL_ERROR
                "NCGP15 generated command contains a conflicting optimization selector: ${NCGP15_COMPILE_ARGUMENT}")
        elseif(NCGP15_COMPILE_ARGUMENT STREQUAL "-DNDEBUG")
            math(EXPR NCGP15_DNDEBUG_COUNT "${NCGP15_DNDEBUG_COUNT} + 1")
        elseif(NCGP15_COMPILE_ARGUMENT STREQUAL "-Wall")
            math(EXPR NCGP15_WALL_COUNT "${NCGP15_WALL_COUNT} + 1")
        elseif(NCGP15_COMPILE_ARGUMENT STREQUAL "-Wextra")
            math(EXPR NCGP15_WEXTRA_COUNT "${NCGP15_WEXTRA_COUNT} + 1")
        elseif(NCGP15_COMPILE_ARGUMENT STREQUAL "-Wpedantic")
            math(EXPR NCGP15_WPEDANTIC_COUNT "${NCGP15_WPEDANTIC_COUNT} + 1")
        elseif(NCGP15_COMPILE_ARGUMENT STREQUAL "-Werror")
            math(EXPR NCGP15_WERROR_COUNT "${NCGP15_WERROR_COUNT} + 1")
        elseif(NCGP15_COMPILE_ARGUMENT STREQUAL "-ffp-contract=off")
            math(EXPR NCGP15_FFP_CONTRACT_COUNT
                "${NCGP15_FFP_CONTRACT_COUNT} + 1")
        elseif(NCGP15_COMPILE_ARGUMENT STREQUAL "-fno-fast-math")
            math(EXPR NCGP15_NO_FAST_MATH_COUNT
                "${NCGP15_NO_FAST_MATH_COUNT} + 1")
        elseif(NCGP15_COMPILE_ARGUMENT STREQUAL "-ffast-math"
                OR NCGP15_COMPILE_ARGUMENT MATCHES "^-ffp-contract="
                OR NCGP15_COMPILE_ARGUMENT MATCHES "^-m"
                OR NCGP15_COMPILE_ARGUMENT MATCHES "^--?target(=|$)"
                OR NCGP15_COMPILE_ARGUMENT MATCHES "^-Xarch_"
                OR NCGP15_COMPILE_ARGUMENT MATCHES "^-f(no-)?(approx-func|associative-math|cx-fortran-rules|cx-limited-range|finite-math-only|float-store|honor-infinities|honor-nans|math-errno|protect-parens|reciprocal-math|rounding-math|signaling-nans|signed-zeros|trapping-math|unsafe-math-optimizations)$"
                OR NCGP15_COMPILE_ARGUMENT MATCHES "^-f(excess-precision|fp-exception-behavior|fp-eval-method|fp-model|denormal-fp-math|denormal-fp-math-f32|veclib)="
                OR NCGP15_COMPILE_ARGUMENT MATCHES "^-g($|[0-9])")
            message(FATAL_ERROR
                "NCGP15 generated command contains a forbidden profile flag: ${NCGP15_COMPILE_ARGUMENT}")
        endif()
    endforeach()
    if(NOT NCGP15_STANDARD_SELECTOR_COUNT EQUAL 1)
        message(FATAL_ERROR
            "NCGP15 generated command must contain exactly one strict C++17 selector: ${NCGP15_COMPILE_SOURCE_NAME}")
    endif()
    if(NOT NCGP15_O3_COUNT EQUAL 2
            OR NOT NCGP15_DNDEBUG_COUNT EQUAL 1
            OR NOT NCGP15_WALL_COUNT EQUAL 1
            OR NOT NCGP15_WEXTRA_COUNT EQUAL 1
            OR NOT NCGP15_WPEDANTIC_COUNT EQUAL 1
            OR NOT NCGP15_WERROR_COUNT EQUAL 1
            OR NOT NCGP15_FFP_CONTRACT_COUNT EQUAL 1
            OR NOT NCGP15_NO_FAST_MATH_COUNT EQUAL 1)
        message(FATAL_ERROR
            "NCGP15 generated command does not contain the exact frozen Release/profile flag multiset: ${NCGP15_COMPILE_SOURCE_NAME}")
    endif()
    set(NCGP15_NORMALIZED_COMPILE_COMMAND "${NCGP15_COMPILE_COMMAND}")
    string(REPLACE "${NCGP15_COMPILE_DATABASE_ROOT}" "<BUILD_ROOT>"
        NCGP15_NORMALIZED_COMPILE_COMMAND
        "${NCGP15_NORMALIZED_COMPILE_COMMAND}")
    string(APPEND NCGP15_COMMAND_MATERIAL
        "${NCGP15_COMPILE_SOURCE_NAME}:${NCGP15_NORMALIZED_COMPILE_COMMAND}\n")
    math(EXPR NCGP15_TARGET_COMMAND_COUNT
        "${NCGP15_TARGET_COMMAND_COUNT} + 1")
endforeach()

list(LENGTH NCGP15_EXPECTED_SOURCE_NAMES NCGP15_EXPECTED_COMMAND_COUNT)
if(NOT NCGP15_TARGET_COMMAND_COUNT EQUAL NCGP15_EXPECTED_COMMAND_COUNT)
    message(FATAL_ERROR
        "NCGP15 generated target does not contain the exact frozen source command set")
endif()
foreach(NCGP15_EXPECTED_SOURCE_NAME IN LISTS NCGP15_EXPECTED_SOURCE_NAMES)
    string(MAKE_C_IDENTIFIER "${NCGP15_EXPECTED_SOURCE_NAME}"
        NCGP15_EXPECTED_SOURCE_KEY)
    if(NOT DEFINED NCGP15_SEEN_${NCGP15_EXPECTED_SOURCE_KEY})
        message(FATAL_ERROR
            "NCGP15 generated target is missing a frozen source command: ${NCGP15_EXPECTED_SOURCE_NAME}")
    endif()
endforeach()
string(SHA256 NCGP15_GENERATED_COMPILE_COMMAND_ROOT
    "${NCGP15_COMMAND_MATERIAL}")
string(HEX "${NCGP15_COMMAND_MATERIAL}"
    NCGP15_GENERATED_COMPILE_COMMAND_MATERIAL_HEX)
file(WRITE "${NCGP15_PROFILE_HEADER}"
    "#pragma once\n#define NCGP15_GENERATED_COMPILE_COMMAND_ROOT \"${NCGP15_GENERATED_COMPILE_COMMAND_ROOT}\"\n#define NCGP15_GENERATED_COMPILE_COMMAND_MATERIAL_HEX \"${NCGP15_GENERATED_COMPILE_COMMAND_MATERIAL_HEX}\"\n")
]=])
add_custom_target(
    ncgp15-verify-compile-profile
    COMMAND "${CMAKE_COMMAND}"
        "-DNCGP15_COMPILE_COMMANDS=${CMAKE_BINARY_DIR}/compile_commands.json"
        "-DNCGP15_TARGET_MARKER=CMakeFiles/nonlocal-corrected-cpu-unified-constrained.dir/"
        "-DNCGP15_EXPECTED_STANDARD=-std=c++17"
        "-DNCGP15_PROFILE_HEADER=${CMAKE_CURRENT_BINARY_DIR}/CMakeFiles/ncgp15_generated_compile_profile.hpp"
        -P "${NCGP15_COMPILE_PROFILE_CHECK_SCRIPT}"
    COMMENT "Verifying the generated NCGP15 strict C++17 compile profile"
    VERBATIM
)
add_dependencies(
    nonlocal-corrected-cpu-unified-constrained
    ncgp15-verify-compile-profile)
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
