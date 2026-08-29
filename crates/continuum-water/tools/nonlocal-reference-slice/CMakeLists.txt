cmake_minimum_required(VERSION 3.20)

project(nextengine_nonlocal_reference_slice LANGUAGES CXX)

include(CTest)

set(NEXTENGINE_SHA256_DIR
    "${CMAKE_CURRENT_LIST_DIR}/../nonlocal-feasibility/src")

add_executable(nonlocal-reference-slice
    src/main.cpp
    src/slice.cpp
    src/slice.hpp
    "${NEXTENGINE_SHA256_DIR}/sha256.cpp"
    "${NEXTENGINE_SHA256_DIR}/sha256.hpp")

target_compile_features(nonlocal-reference-slice PRIVATE cxx_std_17)
target_compile_options(nonlocal-reference-slice PRIVATE
    -O3
    -Wall
    -Wextra
    -Wpedantic
    -Werror
    -ffp-contract=off
    -fno-fast-math
    -frounding-math)
target_include_directories(nonlocal-reference-slice PRIVATE
    "${CMAKE_CURRENT_LIST_DIR}/src"
    "${NEXTENGINE_SHA256_DIR}")

if(BUILD_TESTING)
    add_test(NAME nonlocal_reference_slice.reject_relative
        COMMAND nonlocal-reference-slice --first-output relative)
    add_test(NAME nonlocal_reference_slice.reject_tmp
        COMMAND nonlocal-reference-slice --first-output /tmp/)
    add_test(NAME nonlocal_reference_slice.reject_unknown
        COMMAND nonlocal-reference-slice --unknown)
    set_tests_properties(
        nonlocal_reference_slice.reject_relative
        nonlocal_reference_slice.reject_tmp
        nonlocal_reference_slice.reject_unknown
        PROPERTIES
            ENVIRONMENT "LC_ALL=C"
            WILL_FAIL TRUE)
endif()

