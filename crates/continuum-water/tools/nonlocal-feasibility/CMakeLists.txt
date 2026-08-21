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

add_executable(
    nonlocal-formula-reclosure
    src/balanced_canonical.cpp
    src/boundary_reference.cpp
    src/canonical.cpp
    src/dimensional_profile.cpp
    src/formula_discriminators.cpp
    src/formula_reclosure.cpp
    src/formula_reclosure_main.cpp
    src/reference_attestation.cpp
    src/sha256.cpp
    src/variational_reference.cpp
)
find_package(OpenMP REQUIRED COMPONENTS CXX)
target_compile_features(nonlocal-formula-reclosure PRIVATE cxx_std_17)
target_link_libraries(nonlocal-formula-reclosure PRIVATE OpenMP::OpenMP_CXX)
target_compile_definitions(
    nonlocal-formula-reclosure PRIVATE NEXTENGINE_NONLOCAL_OPENMP=1)
target_compile_options(
    nonlocal-formula-reclosure
    PRIVATE
        $<$<CXX_COMPILER_ID:GNU,Clang>:-O3;-Wall;-Wextra;-Wpedantic;-Werror;-ffp-contract=off;-fno-fast-math>
)
