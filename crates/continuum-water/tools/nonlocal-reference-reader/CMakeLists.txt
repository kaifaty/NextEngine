cmake_minimum_required(VERSION 3.20)

project(nextengine_nonlocal_reference_reader LANGUAGES CXX)

include(CTest)

set(NEXTENGINE_SHA256_DIR
    "${CMAKE_CURRENT_LIST_DIR}/../nonlocal-feasibility/src")

add_executable(nonlocal-reference-reader
    src/main.cpp
    src/reader.cpp
    src/reader.hpp
    "${NEXTENGINE_SHA256_DIR}/sha256.cpp"
    "${NEXTENGINE_SHA256_DIR}/sha256.hpp")

target_compile_features(nonlocal-reference-reader PRIVATE cxx_std_17)
target_compile_options(nonlocal-reference-reader PRIVATE
    -O3
    -Wall
    -Wextra
    -Wpedantic
    -Werror
    -ffp-contract=off
    -fno-fast-math)
target_include_directories(nonlocal-reference-reader PRIVATE
    "${CMAKE_CURRENT_LIST_DIR}/src"
    "${NEXTENGINE_SHA256_DIR}")

if(BUILD_TESTING)
    add_test(NAME nonlocal_reference_reader.profile
        COMMAND nonlocal-reference-reader --profile-self-test)
    add_test(NAME nonlocal_reference_reader.reject_relative
        COMMAND nonlocal-reference-reader --attest relative)
    add_test(NAME nonlocal_reference_reader.reject_tmp
        COMMAND nonlocal-reference-reader --attest /tmp/)
    add_test(NAME nonlocal_reference_reader.reject_unknown
        COMMAND nonlocal-reference-reader --unknown)
    set_tests_properties(
        nonlocal_reference_reader.profile
        nonlocal_reference_reader.reject_relative
        nonlocal_reference_reader.reject_tmp
        nonlocal_reference_reader.reject_unknown
        PROPERTIES ENVIRONMENT "LC_ALL=C")
    set_tests_properties(
        nonlocal_reference_reader.reject_relative
        nonlocal_reference_reader.reject_tmp
        nonlocal_reference_reader.reject_unknown
        PROPERTIES WILL_FAIL TRUE)
endif()
