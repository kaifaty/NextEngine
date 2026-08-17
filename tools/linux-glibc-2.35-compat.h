#ifndef NEXTENGINE_LINUX_GLIBC_2_35_COMPAT_H
#define NEXTENGINE_LINUX_GLIBC_2_35_COMPAT_H

/*
 * SDL requires GNU extensions on Linux.  Starting with glibc 2.38,
 * _GNU_SOURCE also redirects the existing strto* and scanf APIs to new
 * __isoc23_* symbol versions.  The redirects are not available in the
 * declared glibc 2.35 package baseline.
 *
 * Load feature selection once with GNU APIs enabled, then retain the pre-C23
 * semantics for this SDL build.  Public headers included later observe the
 * complete GNU feature set but bind these APIs to their baseline symbols.
 */
#ifndef _GNU_SOURCE
#define _GNU_SOURCE 1
#endif
#include <features.h>

#ifdef __GLIBC__
#undef __GLIBC_USE_C23_STRTOL
#define __GLIBC_USE_C23_STRTOL 0
#endif

#endif
