cmake_minimum_required(VERSION 3.20)

project(nextengine_nonlocal_reference_adapter LANGUAGES CXX)

include(CTest)
find_package(Git REQUIRED)

if(NOT DEFINED SPLISHSPLASH_SOURCE_DIR)
    message(FATAL_ERROR "SPLISHSPLASH_SOURCE_DIR is required")
endif()
if(NOT DEFINED SPLISHSPLASH_BUILD_DIR)
    message(FATAL_ERROR "SPLISHSPLASH_BUILD_DIR is required")
endif()

set(NEXTENGINE_SHA256_DIR
    "${CMAKE_CURRENT_LIST_DIR}/../nonlocal-feasibility/src")

set(SPLISHSPLASH_ARCHIVES
    "${SPLISHSPLASH_BUILD_DIR}/lib/libSPlisHSPlasH.a"
    "${SPLISHSPLASH_BUILD_DIR}/lib/libUtilities.a"
    "${SPLISHSPLASH_BUILD_DIR}/lib/libtinyexpr.a"
    "${SPLISHSPLASH_BUILD_DIR}/lib/libMD5.a"
    "${SPLISHSPLASH_BUILD_DIR}/lib/libpartio.a"
    "${SPLISHSPLASH_BUILD_DIR}/lib/libzlib.a"
    "${SPLISHSPLASH_BUILD_DIR}/extern/install/NeighborhoodSearch/lib/libCompactNSearch.a"
    "${SPLISHSPLASH_BUILD_DIR}/extern/install/Discregrid/lib/libDiscregrid.a")

if(NOT IS_DIRECTORY "${SPLISHSPLASH_SOURCE_DIR}/.git")
    message(FATAL_ERROR
        "pinned external source must be an ordinary Git clone, not a linked worktree")
endif()

execute_process(
    COMMAND "${GIT_EXECUTABLE}" -C "${SPLISHSPLASH_SOURCE_DIR}" rev-parse HEAD
    RESULT_VARIABLE SPLISHSPLASH_HEAD_RESULT
    OUTPUT_VARIABLE SPLISHSPLASH_HEAD
    OUTPUT_STRIP_TRAILING_WHITESPACE
    ERROR_QUIET)
if(NOT SPLISHSPLASH_HEAD_RESULT EQUAL 0
        OR NOT SPLISHSPLASH_HEAD STREQUAL
            "eccce86155776f6ac52d5080b1f720a52bf29450")
    message(FATAL_ERROR "pinned external HEAD mismatch: ${SPLISHSPLASH_HEAD}")
endif()

execute_process(
    COMMAND "${GIT_EXECUTABLE}" -C "${SPLISHSPLASH_SOURCE_DIR}"
        config --get-regexp "^remote\\..*\\.promisor$"
    RESULT_VARIABLE SPLISHSPLASH_PROMISOR_RESULT
    OUTPUT_QUIET
    ERROR_QUIET)
if(SPLISHSPLASH_PROMISOR_RESULT EQUAL 0)
    message(FATAL_ERROR "pinned external source is a promisor/partial clone")
endif()

execute_process(
    COMMAND "${GIT_EXECUTABLE}" -C "${SPLISHSPLASH_SOURCE_DIR}"
        fsck --full --no-dangling
    RESULT_VARIABLE SPLISHSPLASH_FSCK_RESULT
    OUTPUT_QUIET
    ERROR_QUIET)
if(NOT SPLISHSPLASH_FSCK_RESULT EQUAL 0)
    message(FATAL_ERROR "pinned external source does not have a complete Git object database")
endif()

execute_process(
    COMMAND "${GIT_EXECUTABLE}" -C "${SPLISHSPLASH_SOURCE_DIR}"
        status --porcelain=v1 --untracked-files=all
    RESULT_VARIABLE SPLISHSPLASH_STATUS_RESULT
    OUTPUT_VARIABLE SPLISHSPLASH_STATUS
    OUTPUT_STRIP_TRAILING_WHITESPACE
    ERROR_QUIET)
set(SPLISHSPLASH_EXPECTED_STATUS
    " M SPlisHSPlasH/DFSPH/TimeStepDFSPH.cpp\n M SPlisHSPlasH/DFSPH/TimeStepDFSPH.h\n?? Utilities/Version.h")
if(NOT SPLISHSPLASH_STATUS_RESULT EQUAL 0
        OR NOT SPLISHSPLASH_STATUS STREQUAL SPLISHSPLASH_EXPECTED_STATUS)
    message(FATAL_ERROR
        "pinned external dirty-path mismatch; actual status:\n${SPLISHSPLASH_STATUS}")
endif()

execute_process(
    COMMAND "${GIT_EXECUTABLE}" -C "${SPLISHSPLASH_SOURCE_DIR}"
        diff --binary --no-ext-diff
    RESULT_VARIABLE SPLISHSPLASH_DIFF_RESULT
    OUTPUT_VARIABLE SPLISHSPLASH_DIFF
    ERROR_QUIET)
string(SHA256 SPLISHSPLASH_DIFF_SHA256 "${SPLISHSPLASH_DIFF}")
if(NOT SPLISHSPLASH_DIFF_RESULT EQUAL 0
        OR NOT SPLISHSPLASH_DIFF_SHA256 STREQUAL
            "6e54baf340651a7c3eefd3fd39c4ff209bcf9948e07cf0829a96345271aab824")
    message(FATAL_ERROR
        "pinned external patch diff mismatch: ${SPLISHSPLASH_DIFF_SHA256}")
endif()

foreach(required_path
        "${SPLISHSPLASH_SOURCE_DIR}/SPlisHSPlasH/DFSPH/TimeStepDFSPH.cpp"
        "${SPLISHSPLASH_SOURCE_DIR}/SPlisHSPlasH/DFSPH/TimeStepDFSPH.h"
        "${SPLISHSPLASH_SOURCE_DIR}/Utilities/Version.h"
        "${SPLISHSPLASH_SOURCE_DIR}/extern/eigen/Eigen/Core"
        ${SPLISHSPLASH_ARCHIVES})
    if(NOT EXISTS "${required_path}")
        message(FATAL_ERROR "required pinned external input is missing: ${required_path}")
    endif()
endforeach()

function(nextengine_require_sha256 path expected)
    file(SHA256 "${path}" actual)
    if(NOT actual STREQUAL expected)
        message(FATAL_ERROR
            "pinned external SHA-256 mismatch: ${path}; expected ${expected}; actual ${actual}")
    endif()
endfunction()

nextengine_require_sha256(
    "${SPLISHSPLASH_SOURCE_DIR}/SPlisHSPlasH/DFSPH/TimeStepDFSPH.cpp"
    cb22e2c3006060c07526c813c10e9483a2dacf5073bc7cb57697fff84bfe8c7a)
nextengine_require_sha256(
    "${SPLISHSPLASH_SOURCE_DIR}/SPlisHSPlasH/DFSPH/TimeStepDFSPH.h"
    06ddb8131108e0f9fd54ada44c3327d1ef548a7d47a94889e26cc592f9aa18af)
nextengine_require_sha256(
    "${SPLISHSPLASH_SOURCE_DIR}/Utilities/Version.h"
    975287f2e39757092e5d8d91551b6beaf344008eead09fd00de0b5a348d620ce)
nextengine_require_sha256(
    "${SPLISHSPLASH_BUILD_DIR}/lib/libSPlisHSPlasH.a"
    48511ee107e85abeec29f474e5c6ae0761ec11055c1a761ab83f667fd264c393)
nextengine_require_sha256(
    "${SPLISHSPLASH_BUILD_DIR}/lib/libUtilities.a"
    93c6dd16ae500aae89b370b8a3bca902da3ba5a32e91a99630f66dfa89a1d0c4)
nextengine_require_sha256(
    "${SPLISHSPLASH_BUILD_DIR}/lib/libtinyexpr.a"
    63e8e8bda564e8738dfbb085a4032ba893207b4dffb2ba0b3acb25f723126cbc)
nextengine_require_sha256(
    "${SPLISHSPLASH_BUILD_DIR}/lib/libMD5.a"
    e983a90676a7200508467fc2a8e1ae3c0453515cdc77d9ca3c0e620ab0d11239)
nextengine_require_sha256(
    "${SPLISHSPLASH_BUILD_DIR}/lib/libpartio.a"
    a0f9ae32d8661ea47e848fbed167ea2ed9d1f652293736b3700eb6d360892494)
nextengine_require_sha256(
    "${SPLISHSPLASH_BUILD_DIR}/lib/libzlib.a"
    70f93ec9b3400961560606548e672b036106d82e6ba8b37aa2d870a00cbf7d97)
nextengine_require_sha256(
    "${SPLISHSPLASH_BUILD_DIR}/extern/install/NeighborhoodSearch/lib/libCompactNSearch.a"
    5d3df5a05d8148b2fae02a7cd9a81a2fe09d06af82c5a98a041255f772c87e61)
nextengine_require_sha256(
    "${SPLISHSPLASH_BUILD_DIR}/extern/install/Discregrid/lib/libDiscregrid.a"
    8f6fa4d0f0992e0154e02554a836673ad676e3b52bc68ef773bc9c2c3885c19a)

find_package(OpenMP REQUIRED COMPONENTS CXX)

add_executable(nonlocal_reference_adapter
    src/contact_adapter.cpp
    src/contact_adapter.hpp
    src/r1c_manifest.cpp
    src/r1c_manifest.hpp
    src/r1c_trajectory.cpp
    src/r1c_trajectory.hpp
    src/main.cpp
    "${NEXTENGINE_SHA256_DIR}/sha256.cpp"
    "${NEXTENGINE_SHA256_DIR}/sha256.hpp")

target_compile_features(nonlocal_reference_adapter PRIVATE cxx_std_17)
target_compile_definitions(nonlocal_reference_adapter PRIVATE
    EIGEN_DISABLE_UNALIGNED_ARRAY_ASSERT
    USE_CompactNSearch
    USE_DOUBLE)
target_compile_options(nonlocal_reference_adapter PRIVATE
    -Wall
    -Wextra
    -Wpedantic
    -Werror
    -ffp-contract=off
    -fno-fast-math
    -mno-avx
    -mno-avx2
    -mno-fma
    -march=x86-64)

target_include_directories(nonlocal_reference_adapter PRIVATE
    "${CMAKE_CURRENT_LIST_DIR}/src"
    "${NEXTENGINE_SHA256_DIR}")

target_include_directories(nonlocal_reference_adapter SYSTEM PRIVATE
    "${SPLISHSPLASH_SOURCE_DIR}"
    "${SPLISHSPLASH_SOURCE_DIR}/extern/eigen"
    "${SPLISHSPLASH_SOURCE_DIR}/extern/zlib/src"
    "${SPLISHSPLASH_BUILD_DIR}/extern/install/NeighborhoodSearch/include"
    "${SPLISHSPLASH_BUILD_DIR}/extern/install/GenericParameters/include"
    "${SPLISHSPLASH_BUILD_DIR}/extern/install/Discregrid/include")

target_link_libraries(nonlocal_reference_adapter PRIVATE
    -Wl,--start-group
    ${SPLISHSPLASH_ARCHIVES}
    -Wl,--end-group
    OpenMP::OpenMP_CXX)

if(BUILD_TESTING)
    add_test(NAME nonlocal_reference_adapter.pass
        COMMAND nonlocal_reference_adapter)
    add_test(NAME nonlocal_reference_adapter.reject_rounding
        COMMAND nonlocal_reference_adapter --negative-rounding)
    add_test(NAME nonlocal_reference_adapter.reject_ftz
        COMMAND nonlocal_reference_adapter --negative-ftz)
    add_test(NAME nonlocal_reference_adapter.reject_unknown
        COMMAND nonlocal_reference_adapter --unknown)
    add_test(NAME nonlocal_reference_adapter.r1c_manifest
        COMMAND nonlocal_reference_adapter --r1c-manifest-preflight)
    add_test(NAME nonlocal_reference_adapter.r1c_reject_manifest_mismatch
        COMMAND nonlocal_reference_adapter --r1c-negative-manifest-mismatch)
    add_test(NAME nonlocal_reference_adapter.r1c_reject_relative_output
        COMMAND nonlocal_reference_adapter --r1c-trajectory CW-HYDRO-001 relative)

    set_tests_properties(
        nonlocal_reference_adapter.pass
        nonlocal_reference_adapter.reject_rounding
        nonlocal_reference_adapter.reject_ftz
        nonlocal_reference_adapter.reject_unknown
        nonlocal_reference_adapter.r1c_manifest
        nonlocal_reference_adapter.r1c_reject_manifest_mismatch
        nonlocal_reference_adapter.r1c_reject_relative_output
        PROPERTIES ENVIRONMENT "LC_ALL=C;OMP_NUM_THREADS=1;OMP_DYNAMIC=FALSE")
    set_tests_properties(
        nonlocal_reference_adapter.reject_rounding
        nonlocal_reference_adapter.reject_ftz
        nonlocal_reference_adapter.reject_unknown
        nonlocal_reference_adapter.r1c_reject_manifest_mismatch
        nonlocal_reference_adapter.r1c_reject_relative_output
        PROPERTIES WILL_FAIL TRUE)
endif()
