cmake_minimum_required(VERSION 3.28)

project(nextengine_nonlocal_feasibility LANGUAGES CXX)

add_executable(
    nonlocal-feasibility
    src/main.cpp
    src/oracle.cpp
    src/profiles.cpp
    src/sha256.cpp
)
target_compile_features(nonlocal-feasibility PRIVATE cxx_std_17)
target_compile_options(
    nonlocal-feasibility
    PRIVATE
        $<$<CXX_COMPILER_ID:GNU,Clang>:-O3;-Wall;-Wextra;-Wpedantic;-Werror>
)
