#pragma once

#include <array>
#include <cstddef>
#include <cstdint>

namespace r63zp_format {

constexpr std::size_t dimension = 102U;
constexpr std::size_t columns = 315U;
constexpr std::size_t parent_cache_bytes = 1033625U;
constexpr std::size_t parent_artifact_bytes = 12916U;
constexpr std::size_t parent_audit_bytes = 420U;
constexpr std::size_t candidate_work_fields = 64U;
constexpr std::size_t checker_work_fields = 71U;
constexpr std::size_t event_slots = 8U;
constexpr std::size_t quad_bytes = 16U;

constexpr std::array<std::uint8_t, 8U> candidate_magic{{
    'N', 'E', 'R', '6', '3', 'Z', 'P', '6'}};
constexpr std::array<std::uint8_t, 8U> checker_magic{{
    'N', 'E', 'R', '6', '3', 'Z', 'Q', '6'}};
constexpr std::uint32_t version = 6U;

enum class CandidateRoute : std::uint32_t {
    ApparatusRejected = 1U,
    DirectionInputRejected = 2U,
    ProductBoundRejected = 3U,
    CurvatureRejected = 4U,
    StepRejected = 5U,
    UpdateRejected = 6U,
    Admitted = 7U,
};

enum class CheckerRoute : std::uint32_t {
    ApparatusRejectedVerified = 1U,
    DirectionInputRejectedVerified = 2U,
    ProductBoundRejectedVerified = 3U,
    CurvatureRejectedVerified = 4U,
    StepRejectedVerified = 5U,
    UpdateRejectedVerified = 6U,
    Admitted = 7U,
    CandidateMalformed = 8U,
    SemanticMismatch = 9U,
    WorkMismatch = 10U,
    EventMismatch = 11U,
    SealMismatch = 12U,
    CheckerApparatusRejected = 13U,
};

enum CandidateWork : std::size_t {
    CwRoundingModeSetCalls,
    CwRoundingModeChecks,
    CwInputOpenAttempts,
    CwInputReadCalls,
    CwInputReadBytes,
    CwInputTrailingChecks,
    CwInputCloseCalls,
    CwInputHashCalls,
    CwInputHashBytes,
    CwParentHeaderPredicates,
    CwCacheQuadDecodesPreseal,
    CwCacheDoubleDecodes,
    CwCacheIndexDecodes,
    CwParentArtifactQuadDecodes,
    CwFinitePredicates,
    CwRangePredicates,
    CwX0ComponentComparisons,
    CwParentRootComparisons,
    CwFactorSolves,
    CwFactorTerms,
    CwFactorDivisions,
    CwResidualDots,
    CwResidualTerms,
    CwRhoDots,
    CwRhoTerms,
    CwProductKernels,
    CwInnerDots,
    CwInnerTerms,
    CwOuterDots,
    CwOuterTerms,
    CwPropagationTerms,
    CwScaleProducts,
    CwDenominatorDots,
    CwDenominatorTerms,
    CwIntervalEndpointOperations,
    CwPositivityPredicates,
    CwDivisionEndpointOperations,
    CwDivisionPredicates,
    CwControlSelectorPredicates,
    CwPresealControlInjections,
    CwPresealRootCalls,
    CwPresealRootBytes,
    CwPostsealX1Decodes,
    CwPostsealX1FinitePredicates,
    CwPostsealControlInjections,
    CwUpdateDots,
    CwUpdateTerms,
    CwUpdateComparisons,
    CwEventRootCalls,
    CwEventRootBytes,
    CwTraceRootCalls,
    CwTraceRootBytes,
    CwFinalRootCalls,
    CwFinalRootBytes,
    CwRoutePredicates,
    CwReceiptZeroFillBytes,
    CwReceiptFieldsSerialized,
    CwReceiptBytesSerialized,
    CwOutputOpenAttempts,
    CwOutputWriteCalls,
    CwOutputWriteBytes,
    CwOutputCloseCalls,
    CwPackageAllocations,
    CwFixedLoopIterations,
    CwCount,
};

enum CheckerWork : std::size_t {
    KwRoundingModeSetCalls,
    KwRoundingModeChecks,
    KwInputOpenAttempts,
    KwInputReadCalls,
    KwInputReadBytes,
    KwInputTrailingChecks,
    KwInputCloseCalls,
    KwInputHashCalls,
    KwInputHashBytes,
    KwHeaderPredicatesExecuted,
    KwCandidateBytesDecoded,
    KwCandidateQuadDecodes,
    KwCandidateU64Decodes,
    KwCandidateDigestDecodes,
    KwCandidatePaddingBytesChecked,
    KwParentHeaderPredicates,
    KwCacheQuadDecodesPreseal,
    KwCacheDoubleDecodes,
    KwCacheIndexDecodes,
    KwParentArtifactQuadDecodes,
    KwFinitePredicates,
    KwRangePredicates,
    KwControlInjections,
    KwX0ComponentComparisons,
    KwParentRootComparisons,
    KwFactorSolves,
    KwFactorTerms,
    KwFactorDivisions,
    KwResidualDots,
    KwResidualTerms,
    KwRhoDots,
    KwRhoTerms,
    KwRoundedProductCalls,
    KwRoundedInnerDots,
    KwRoundedInnerTerms,
    KwRoundedOuterDots,
    KwRoundedOuterTerms,
    KwRoundedPropagationTerms,
    KwRoundedScaleProducts,
    KwDyadicDecodes,
    KwExactMultiplies,
    KwExactAdditions,
    KwAlignmentShifts,
    KwAlignmentBits,
    KwBigintCapacityPredicates,
    KwIntervalComparisons,
    KwExactDenominatorTerms,
    KwPositivityPredicates,
    KwDivisionEndpointOperations,
    KwDivisionPredicates,
    KwPostsealX1Decodes,
    KwUpdateDots,
    KwUpdateTerms,
    KwUpdateComparisons,
    KwSemanticFieldComparisons,
    KwCandidateWorkComparisons,
    KwEventComparisons,
    KwSealComparisons,
    KwRoutePredicates,
    KwRootCalls,
    KwRootBytes,
    KwAuditZeroFillBytes,
    KwAuditFieldsSerialized,
    KwAuditBytesSerialized,
    KwOutputOpenAttempts,
    KwOutputWriteCalls,
    KwOutputWriteBytes,
    KwOutputCloseCalls,
    KwPackageAllocations,
    KwExactVectorHashFields,
    KwCandidateFixedLoopIterations,
    KwCount,
};

constexpr std::size_t candidate_bytes = 6176U;
constexpr std::size_t c_version = 8U;
constexpr std::size_t c_total = 12U;
constexpr std::size_t c_route = 16U;
constexpr std::size_t c_flags = 20U;
constexpr std::size_t c_parent_roots = 24U;
constexpr std::size_t c_role_roots = 120U;
constexpr std::size_t c_semantic_roots = 184U;
constexpr std::size_t c_scalars = 280U;
constexpr std::size_t c_dimension = 360U;
constexpr std::size_t c_work_count = 368U;
constexpr std::size_t c_work = 376U;
constexpr std::size_t c_work_root = 888U;
constexpr std::size_t c_event_count = 920U;
constexpr std::size_t c_events = 928U;
constexpr std::size_t c_trace_root = 1184U;
constexpr std::size_t c_p0 = 1216U;
constexpr std::size_t c_q0 = 2848U;
constexpr std::size_t c_bounds = 4480U;
constexpr std::size_t c_product_body_root = 6112U;
constexpr std::size_t c_result_root = 6144U;

constexpr std::size_t checker_bytes = 1176U;
constexpr std::size_t k_version = 8U;
constexpr std::size_t k_total = 12U;
constexpr std::size_t k_route = 16U;
constexpr std::size_t k_flags = 20U;
constexpr std::size_t k_parent_roots = 24U;
constexpr std::size_t k_candidate_roots = 120U;
constexpr std::size_t k_work = 248U;
constexpr std::size_t k_work_root = 816U;
constexpr std::size_t k_event_count = 848U;
constexpr std::size_t k_events = 856U;
constexpr std::size_t k_trace_root = 1112U;
constexpr std::size_t k_result_root = 1144U;

static_assert(CwCount == candidate_work_fields);
static_assert(KwCount == checker_work_fields);
static_assert(candidate_magic.size() == c_version);
static_assert(c_version + 4U == c_total);
static_assert(c_total + 4U == c_route);
static_assert(c_route + 4U == c_flags);
static_assert(c_flags + 4U == c_parent_roots);
static_assert(c_parent_roots + 3U * 32U == c_role_roots);
static_assert(c_role_roots + 2U * 32U == c_semantic_roots);
static_assert(c_semantic_roots + 3U * 32U == c_scalars);
static_assert(c_scalars + 5U * quad_bytes == c_dimension);
static_assert(c_dimension + 8U == c_work_count);
static_assert(c_work_count + 8U == c_work);
static_assert(c_work + candidate_work_fields * 8U == c_work_root);
static_assert(c_work_root + 32U == c_event_count);
static_assert(c_event_count + 8U == c_events);
static_assert(c_events + event_slots * 32U == c_trace_root);
static_assert(c_trace_root + 32U == c_p0);
static_assert(c_p0 + dimension * quad_bytes == c_q0);
static_assert(c_q0 + dimension * quad_bytes == c_bounds);
static_assert(c_bounds + dimension * quad_bytes == c_product_body_root);
static_assert(c_product_body_root + 32U == c_result_root);
static_assert(c_result_root + 32U == candidate_bytes);

static_assert(checker_magic.size() == k_version);
static_assert(k_version + 4U == k_total);
static_assert(k_total + 4U == k_route);
static_assert(k_route + 4U == k_flags);
static_assert(k_flags + 4U == k_parent_roots);
static_assert(k_parent_roots + 3U * 32U == k_candidate_roots);
static_assert(k_candidate_roots + 4U * 32U == k_work);
static_assert(k_work + checker_work_fields * 8U == k_work_root);
static_assert(k_work_root + 32U == k_event_count);
static_assert(k_event_count + 8U == k_events);
static_assert(k_events + event_slots * 32U == k_trace_root);
static_assert(k_trace_root + 32U == k_result_root);
static_assert(k_result_root + 32U == checker_bytes);

} // namespace r63zp_format
