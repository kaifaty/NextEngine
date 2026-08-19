cmake_minimum_required(VERSION 3.28)

project(nextengine_continuum_water_gpu_feasibility LANGUAGES CUDA CXX)

add_executable(continuum-water-gpu-feasibility main.cu)
target_compile_features(continuum-water-gpu-feasibility PRIVATE cxx_std_17 cuda_std_17)
set_target_properties(
    continuum-water-gpu-feasibility
    PROPERTIES
        CUDA_ARCHITECTURES 86
        CUDA_SEPARABLE_COMPILATION OFF
)
target_compile_options(
    continuum-water-gpu-feasibility
    PRIVATE
        $<$<COMPILE_LANGUAGE:CUDA>:-O3;--fmad=false;--prec-div=true;--prec-sqrt=true;--ftz=false>
)
