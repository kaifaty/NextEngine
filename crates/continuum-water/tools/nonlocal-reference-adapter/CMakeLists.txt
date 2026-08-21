cmake_minimum_required(VERSION 3.20)

project(nextengine_nonlocal_reference_adapter LANGUAGES CXX)

include(CTest)

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

foreach(required_path
        "${SPLISHSPLASH_SOURCE_DIR}/SPlisHSPlasH/DFSPH/TimeStepDFSPH.h"
        "${SPLISHSPLASH_SOURCE_DIR}/extern/eigen/Eigen/Core"
        ${SPLISHSPLASH_ARCHIVES})
    if(NOT EXISTS "${required_path}")
        message(FATAL_ERROR "required pinned external input is missing: ${required_path}")
    endif()
endforeach()

find_package(OpenMP REQUIRED COMPONENTS CXX)

add_executable(nonlocal_reference_adapter
    src/contact_adapter.cpp
    src/contact_adapter.hpp
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

    set_tests_properties(
        nonlocal_reference_adapter.pass
        nonlocal_reference_adapter.reject_rounding
        nonlocal_reference_adapter.reject_ftz
        nonlocal_reference_adapter.reject_unknown
        PROPERTIES ENVIRONMENT "LC_ALL=C;OMP_NUM_THREADS=1;OMP_DYNAMIC=FALSE")
    set_tests_properties(
        nonlocal_reference_adapter.reject_rounding
        nonlocal_reference_adapter.reject_ftz
        nonlocal_reference_adapter.reject_unknown
        PROPERTIES WILL_FAIL TRUE)
endif()
