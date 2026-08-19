cmake_minimum_required(VERSION 3.28)

project(nextengine_nonlocal_feasibility LANGUAGES CUDA CXX)

add_executable(
    nonlocal-feasibility
    src/main.cpp
    src/cuda_baseline.cu
    src/oracle.cpp
    src/profiles.cpp
    src/sha256.cpp
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
