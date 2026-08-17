# NextEngine Linux packages target the Ubuntu 22.04 / glibc 2.35 ABI.
#
# GCC 15 defaults to a C23 dialect on newer distributions.  In addition, new
# glibc releases make _GNU_SOURCE opt into C23 strto*/scanf semantics.  Pin
# SDL's C sources to C17 and force-include the narrow compatibility header so
# package compatibility does not depend on either build-host default.
set(CMAKE_C_STANDARD 17 CACHE STRING "NextEngine Linux package C standard" FORCE)
set(CMAKE_C_STANDARD_REQUIRED ON CACHE BOOL "Require the package C standard" FORCE)
set(CMAKE_C_EXTENSIONS ON CACHE BOOL "Retain GNU C17 extensions" FORCE)
set(NEXTENGINE_GLIBC_COMPAT_HEADER
    "${CMAKE_CURRENT_LIST_DIR}/linux-glibc-2.35-compat.h"
)
string(FIND "${CMAKE_C_FLAGS}" "linux-glibc-2.35-compat.h" NEXTENGINE_COMPAT_FLAG_POSITION)
if(NEXTENGINE_COMPAT_FLAG_POSITION EQUAL -1)
    set(CMAKE_C_FLAGS
        "${CMAKE_C_FLAGS} -D_GNU_SOURCE=1 -include \"${NEXTENGINE_GLIBC_COMPAT_HEADER}\""
        CACHE STRING "NextEngine Linux package C flags" FORCE
    )
endif()

# These libc entry points were added or re-versioned after glibc 2.35.  SDL has
# compatible internal fallbacks, so preseed its feature checks as unavailable.
foreach(symbol
    ACOSF
    ASINF
    ATAN2F
    LOG10F
    STRLCAT
    STRLCPY
    WCSLCAT
    WCSLCPY
)
    set(LIBC_HAS_${symbol} "" CACHE INTERNAL "Unavailable at glibc 2.35 baseline" FORCE)
endforeach()
