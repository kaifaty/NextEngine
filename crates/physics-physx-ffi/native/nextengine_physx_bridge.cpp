#include <PxPhysicsAPI.h>

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <limits>
#include <new>
#include <type_traits>

namespace {

constexpr std::uint32_t kAbiVersion = 1;
constexpr std::int32_t kOk = 0;
constexpr std::int32_t kInvalidArgument = 1;
constexpr std::int32_t kOutOfMemory = 2;
constexpr std::int32_t kCapacityExceeded = 3;
constexpr std::int32_t kInternalFailure = 4;

struct StaticBox {
    physx::PxVec3 centre;
    physx::PxVec3 half_extents;
    std::uint64_t token;
};

struct World {
    StaticBox* boxes;
    std::uint32_t length;
    std::uint32_t capacity;
};

float from_bits(std::uint32_t bits) {
    float value = 0.0F;
    static_assert(sizeof(value) == sizeof(bits));
    std::memcpy(&value, &bits, sizeof(value));
    return value;
}

std::uint32_t to_bits(float value) {
    std::uint32_t bits = 0;
    std::memcpy(&bits, &value, sizeof(bits));
    return bits;
}

bool finite_positive(float value) {
    return physx::PxIsFinite(value) && value > 0.0F;
}

bool finite_vec(const physx::PxVec3& value) {
    return value.isFinite();
}

}  // namespace

extern "C" {

struct NePhysXVersion {
    std::uint32_t abi;
    std::uint32_t major;
    std::uint32_t minor;
    std::uint32_t patch;
};

struct NePhysXSweepOutput {
    std::uint32_t hit;
    std::uint32_t distance_bits;
    std::uint32_t normal_x_bits;
    std::uint32_t normal_y_bits;
    std::uint32_t normal_z_bits;
    std::uint64_t user_token;
};

static_assert(std::is_standard_layout_v<NePhysXVersion>);
static_assert(sizeof(NePhysXVersion) == 16);
static_assert(std::is_standard_layout_v<NePhysXSweepOutput>);
static_assert(offsetof(NePhysXSweepOutput, user_token) == 24);
static_assert(sizeof(NePhysXSweepOutput) == 32);

NePhysXVersion ne_physx_version() noexcept {
    return NePhysXVersion{
        kAbiVersion,
        PX_PHYSICS_VERSION_MAJOR,
        PX_PHYSICS_VERSION_MINOR,
        PX_PHYSICS_VERSION_BUGFIX,
    };
}

std::int32_t ne_physx_world_create(void** output) noexcept {
    if (output == nullptr) {
        return kInvalidArgument;
    }
    auto* world = new (std::nothrow) World{nullptr, 0, 0};
    if (world == nullptr) {
        return kOutOfMemory;
    }
    *output = world;
    return kOk;
}

void ne_physx_world_destroy(void* opaque_world) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr) {
        return;
    }
    delete[] world->boxes;
    delete world;
}

std::int32_t ne_physx_world_reserve(void* opaque_world, std::uint32_t capacity) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || capacity < world->length) {
        return kInvalidArgument;
    }
    if (capacity == world->capacity) {
        return kOk;
    }
    if (static_cast<std::size_t>(capacity)
        > std::numeric_limits<std::size_t>::max() / sizeof(StaticBox)) {
        return kCapacityExceeded;
    }
    auto* boxes = capacity == 0 ? nullptr : new (std::nothrow) StaticBox[capacity];
    if (boxes == nullptr && capacity != 0) {
        return kOutOfMemory;
    }
    for (std::uint32_t index = 0; index < world->length; ++index) {
        boxes[index] = world->boxes[index];
    }
    delete[] world->boxes;
    world->boxes = boxes;
    world->capacity = capacity;
    return kOk;
}

std::int32_t ne_physx_world_add_static_box(
    void* opaque_world,
    const std::uint32_t* centre_bits,
    const std::uint32_t* half_extents_bits,
    std::uint64_t user_token) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || centre_bits == nullptr || half_extents_bits == nullptr) {
        return kInvalidArgument;
    }
    if (world->length >= world->capacity) {
        return kCapacityExceeded;
    }
    physx::PxVec3 centre(
        from_bits(centre_bits[0]),
        from_bits(centre_bits[1]),
        from_bits(centre_bits[2]));
    physx::PxVec3 half_extents(
        from_bits(half_extents_bits[0]),
        from_bits(half_extents_bits[1]),
        from_bits(half_extents_bits[2]));
    if (!finite_vec(centre) || !finite_positive(half_extents.x)
        || !finite_positive(half_extents.y) || !finite_positive(half_extents.z)) {
        return kInvalidArgument;
    }
    world->boxes[world->length++] = StaticBox{centre, half_extents, user_token};
    return kOk;
}

std::int32_t ne_physx_world_sweep_capsule_axis(
    void* opaque_world,
    const std::uint32_t* centre_bits,
    std::uint32_t radius_bits,
    std::uint32_t half_segment_bits,
    std::uint32_t axis,
    std::int32_t direction_sign,
    std::uint32_t distance_bits,
    NePhysXSweepOutput* output) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || centre_bits == nullptr || output == nullptr || axis > 2
        || (direction_sign != -1 && direction_sign != 1)) {
        return kInvalidArgument;
    }
    physx::PxVec3 centre(
        from_bits(centre_bits[0]),
        from_bits(centre_bits[1]),
        from_bits(centre_bits[2]));
    const float radius = from_bits(radius_bits);
    const float half_segment = from_bits(half_segment_bits);
    const float distance = from_bits(distance_bits);
    if (!finite_vec(centre) || !finite_positive(radius) || !physx::PxIsFinite(half_segment)
        || half_segment < 0.0F || !physx::PxIsFinite(distance) || distance < 0.0F) {
        return kInvalidArgument;
    }
    physx::PxVec3 direction(0.0F);
    direction[axis] = static_cast<float>(direction_sign);
    const physx::PxCapsuleGeometry capsule(radius, half_segment);
    const physx::PxQuat capsule_to_y(physx::PxHalfPi, physx::PxVec3(0.0F, 0.0F, 1.0F));
    const physx::PxTransform capsule_pose(centre, capsule_to_y);

    bool selected = false;
    physx::PxSweepHit selected_hit{};
    std::uint64_t selected_token = 0;
    for (std::uint32_t index = 0; index < world->length; ++index) {
        const StaticBox& box = world->boxes[index];
        const physx::PxBoxGeometry geometry(box.half_extents);
        const physx::PxTransform pose(box.centre);
        physx::PxSweepHit hit{};
        const bool collided = physx::PxGeometryQuery::sweep(
            direction,
            distance,
            capsule,
            capsule_pose,
            geometry,
            pose,
            hit,
            physx::PxHitFlag::ePOSITION | physx::PxHitFlag::eNORMAL
                | physx::PxHitFlag::eDISTANCE,
            0.0F);
        // Canonical zero-distance blocking contacts are resolved by the safe
        // engine layer before this call. Ignoring native initial-overlap hits
        // prevents a tangential sweep from selecting an already-touching box.
        if (collided && hit.distance > 0.0F
            && (!selected || hit.distance < selected_hit.distance
                || (hit.distance == selected_hit.distance && box.token < selected_token))) {
            selected = true;
            selected_hit = hit;
            selected_token = box.token;
        }
    }
    output->hit = selected ? 1U : 0U;
    output->distance_bits = to_bits(selected ? selected_hit.distance : distance);
    output->normal_x_bits = to_bits(selected ? selected_hit.normal.x : 0.0F);
    output->normal_y_bits = to_bits(selected ? selected_hit.normal.y : 0.0F);
    output->normal_z_bits = to_bits(selected ? selected_hit.normal.z : 0.0F);
    output->user_token = selected ? selected_token : 0;
    return kOk;
}

}  // extern "C"
