#include <PxPhysicsAPI.h>
// Plan `continuum-water/23` (ADR-106): the PBD probe of the GPU lane.
#include <PxPBDParticleSystem.h>
#include <PxParticleBuffer.h>
#include <cudamanager/PxCudaContext.h>
#include <cudamanager/PxCudaContextManager.h>
#include <extensions/PxParticleExt.h>
#include <gpu/PxGpu.h>

#include <chrono>

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <limits>
#include <mutex>
#include <new>
#include <type_traits>

namespace {

constexpr std::uint32_t kAbiVersion = 4;
constexpr std::int32_t kOk = 0;
constexpr std::int32_t kInvalidArgument = 1;
constexpr std::int32_t kOutOfMemory = 2;
constexpr std::int32_t kCapacityExceeded = 3;
constexpr std::int32_t kInternalFailure = 4;
constexpr std::uint32_t kNoParent = 0xffffffffU;
constexpr std::uint32_t kMaterialEncodingF32Bits = 1U;
constexpr std::uint32_t kMaterialEncodingQ16 = 2U;
constexpr std::uint32_t kArithmeticMeanTiesToEven = 3U;
constexpr std::uint32_t kCanonicalParticipantOrder = 1U;

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

bool finite_non_negative(float value) {
    return physx::PxIsFinite(value) && value >= 0.0F;
}

bool finite_vec(const physx::PxVec3& value) {
    return value.isFinite();
}

std::uint64_t token_of(const physx::PxActor* actor) {
    return actor == nullptr
        ? 0
        : static_cast<std::uint64_t>(reinterpret_cast<std::uintptr_t>(actor->userData));
}

std::uint64_t token_of(const physx::PxShape* shape) {
    return shape == nullptr
        ? 0
        : static_cast<std::uint64_t>(reinterpret_cast<std::uintptr_t>(shape->userData));
}

void set_token(physx::PxActor& actor, std::uint64_t token) {
    actor.userData = reinterpret_cast<void*>(static_cast<std::uintptr_t>(token));
}

struct StaticBox {
    physx::PxVec3 centre;
    physx::PxVec3 half_extents;
    std::uint64_t token;
};

}  // namespace

extern "C" {

struct NePhysXVersion {
    std::uint32_t abi;
    std::uint32_t major;
    std::uint32_t minor;
    std::uint32_t patch;
};

// Plan 23: the PBD probe descriptor and report. Floats travel as bits.
struct NePhysXPbdProbeDesc {
    std::uint32_t particle_count;
    std::uint32_t frames;
    std::uint32_t spacing_bits;
    std::uint32_t box_half_bits;
    std::uint32_t timestep_bits;
    std::uint32_t margin_bits;
    /// Absolute path of the GPU library, or null for the SDK default name.
    const char* gpu_library_path;
    /// Host storage for the final positions, `4 * particle_count` floats.
    float* final_positions;
};

struct NePhysXPbdProbeReport {
    std::uint32_t gpu_available;
    std::uint32_t reason;
    std::uint32_t frames_completed;
    std::uint32_t particles_out_of_bounds_max;
    std::uint64_t step_max_microseconds;
    std::uint64_t step_mean_microseconds;
    std::uint64_t readback_max_microseconds;
    std::uint64_t readback_mean_microseconds;
    char device_name[64];
};

// Plan 24: a persistent GPU fluid. The box is closed by a floor at
// `box_min.y` and four walls at the `x`/`z` faces, open at the top; the
// seed block fills `[seed_min, seed_max]` on the spacing grid up to
// `max_particles`.
struct NePhysXFluidDesc {
    std::uint32_t spacing_bits;
    std::uint32_t box_min_bits[3];
    std::uint32_t box_max_bits[3];
    std::uint32_t seed_min_bits[3];
    std::uint32_t seed_max_bits[3];
    std::uint32_t timestep_bits;
    std::uint32_t max_particles;
    const char* gpu_library_path;
};

struct NePhysXSweepOutput {
    std::uint32_t hit;
    std::uint32_t distance_bits;
    std::uint32_t normal_x_bits;
    std::uint32_t normal_y_bits;
    std::uint32_t normal_z_bits;
    std::uint64_t user_token;
};

struct NePhysXSceneProfile {
    std::uint32_t gravity_bits[3];
    std::uint32_t timestep_bits;
    std::uint32_t position_iterations;
    std::uint32_t velocity_iterations;
    std::uint32_t max_contacts;
    std::uint32_t max_actors;
    std::uint32_t max_joints;
};

struct NePhysXMaterialProfile {
    std::uint32_t coefficient_encoding;
    std::uint32_t static_friction;
    std::uint32_t dynamic_friction;
    std::uint32_t restitution;
    std::uint32_t rolling_friction;
    std::uint32_t spinning_friction;
    std::int64_t surface_velocity_micrometres_per_second[3];
    std::uint32_t coefficient_combine_rules[5];
    std::uint32_t surface_velocity_combine_rule;
};

struct NePhysXRigidBodyInput {
    std::uint64_t user_token;
    std::uint32_t position_bits[3];
    std::uint32_t rotation_bits[4];
    std::uint32_t half_extents_bits[3];
    std::uint32_t mass_bits;
    std::uint32_t inertia_bits[3];
};

struct NePhysXArticulationLinkInput {
    std::uint64_t user_token;
    std::uint32_t parent_link_index;
    std::uint32_t shape_kind;
    std::uint32_t position_bits[3];
    std::uint32_t rotation_bits[4];
    std::uint32_t shape_dimensions_bits[3];
    std::uint32_t mass_bits;
    std::uint32_t inertia_bits[3];
    std::uint32_t linear_damping_bits;
    std::uint32_t angular_damping_bits;
};

struct NePhysXArticulationJointInput {
    std::uint32_t child_link_index;
    std::uint32_t reserved;
    std::uint32_t parent_position_bits[3];
    std::uint32_t parent_rotation_bits[4];
    std::uint32_t child_position_bits[3];
    std::uint32_t child_rotation_bits[4];
    std::uint32_t lower_limit_bits;
    std::uint32_t upper_limit_bits;
    std::uint32_t max_velocity_bits;
};

struct NePhysXArticulationLinkInputV2 {
    std::uint64_t user_token;
    std::uint32_t parent_link_index;
    std::uint32_t first_shape_index;
    std::uint32_t shape_count;
    std::uint32_t reserved;
    std::uint32_t position_bits[3];
    std::uint32_t rotation_bits[4];
    std::uint32_t centre_of_mass_position_bits[3];
    std::uint32_t centre_of_mass_rotation_bits[4];
    std::uint32_t mass_bits;
    std::uint32_t inertia_bits[3];
    std::uint32_t linear_damping_bits;
    std::uint32_t angular_damping_bits;
};

struct NePhysXArticulationShapeInputV2 {
    std::uint64_t user_token;
    std::uint32_t link_index;
    std::uint32_t shape_kind;
    std::uint32_t position_bits[3];
    std::uint32_t rotation_bits[4];
    std::uint32_t shape_dimensions_bits[3];
    std::uint32_t collision_layer;
    std::uint32_t collision_mask_low;
    std::uint32_t collision_mask_high;
};

struct NePhysXArticulationCollisionExclusionV2 {
    std::uint32_t first_link_index;
    std::uint32_t second_link_index;
};

struct NePhysXLinkState {
    std::uint64_t user_token;
    std::uint32_t position_bits[3];
    std::uint32_t rotation_bits[4];
    std::uint32_t linear_velocity_bits[3];
    std::uint32_t angular_velocity_bits[3];
};

struct NePhysXJointState {
    std::uint32_t position_bits;
    std::uint32_t velocity_bits;
};

struct NePhysXContactOutput {
    std::uint64_t actor_a_token;
    std::uint64_t actor_b_token;
    std::uint32_t position_bits[3];
    std::uint32_t normal_bits[3];
    std::uint32_t impulse_bits[3];
    std::uint32_t separation_bits;
};

struct NePhysXContactOutputV2 {
    std::uint64_t actor_a_token;
    std::uint64_t actor_b_token;
    std::uint64_t shape_a_token;
    std::uint64_t shape_b_token;
    std::uint32_t position_bits[3];
    std::uint32_t normal_bits[3];
    std::uint32_t impulse_bits[3];
    std::uint32_t separation_bits;
};

static_assert(std::is_standard_layout_v<NePhysXVersion>);
static_assert(sizeof(NePhysXVersion) == 16);
static_assert(std::is_standard_layout_v<NePhysXSweepOutput>);
static_assert(offsetof(NePhysXSweepOutput, user_token) == 24);
static_assert(sizeof(NePhysXSweepOutput) == 32);
static_assert(sizeof(NePhysXSceneProfile) == 36);
static_assert(sizeof(NePhysXMaterialProfile) == 72);
static_assert(sizeof(NePhysXRigidBodyInput) == 64);
static_assert(sizeof(NePhysXArticulationLinkInput) == 80);
static_assert(sizeof(NePhysXArticulationJointInput) == 76);
static_assert(sizeof(NePhysXArticulationLinkInputV2) == 104);
static_assert(sizeof(NePhysXArticulationShapeInputV2) == 72);
static_assert(sizeof(NePhysXArticulationCollisionExclusionV2) == 8);
static_assert(sizeof(NePhysXLinkState) == 64);
static_assert(sizeof(NePhysXJointState) == 8);
static_assert(sizeof(NePhysXContactOutput) == 56);
static_assert(sizeof(NePhysXContactOutputV2) == 72);

namespace {

struct ContactSink final : physx::PxSimulationEventCallback {
    NePhysXContactOutputV2* contacts = nullptr;
    std::uint32_t length = 0;
    std::uint32_t capacity = 0;
    bool overflow = false;

    void clear() {
        length = 0;
        overflow = false;
    }

    void onConstraintBreak(physx::PxConstraintInfo*, physx::PxU32) override {}
    void onWake(physx::PxActor**, physx::PxU32) override {}
    void onSleep(physx::PxActor**, physx::PxU32) override {}
    void onTrigger(physx::PxTriggerPair*, physx::PxU32) override {}
    void onAdvance(const physx::PxRigidBody* const*, const physx::PxTransform*, physx::PxU32) override {}

    void onContact(
        const physx::PxContactPairHeader& header,
        const physx::PxContactPair* pairs,
        physx::PxU32 pair_count) override {
        const std::uint64_t token_a = token_of(header.actors[0]);
        const std::uint64_t token_b = token_of(header.actors[1]);
        physx::PxContactPairPoint points[64];
        for (physx::PxU32 pair_index = 0; pair_index < pair_count; ++pair_index) {
            const std::uint64_t shape_token_a = token_of(pairs[pair_index].shapes[0]);
            const std::uint64_t shape_token_b = token_of(pairs[pair_index].shapes[1]);
            const physx::PxU32 count = pairs[pair_index].extractContacts(points, 64);
            for (physx::PxU32 point_index = 0; point_index < count; ++point_index) {
                if (length >= capacity) {
                    overflow = true;
                    return;
                }
                const physx::PxContactPairPoint& point = points[point_index];
                NePhysXContactOutputV2& output = contacts[length++];
                output.actor_a_token = token_a;
                output.actor_b_token = token_b;
                output.shape_a_token = shape_token_a;
                output.shape_b_token = shape_token_b;
                for (std::uint32_t axis = 0; axis < 3; ++axis) {
                    output.position_bits[axis] = to_bits(point.position[axis]);
                    output.normal_bits[axis] = to_bits(point.normal[axis]);
                    output.impulse_bits[axis] = to_bits(point.impulse[axis]);
                }
                output.separation_bits = to_bits(point.separation);
            }
        }
    }
};

physx::PxFilterFlags contact_filter_shader(
    physx::PxFilterObjectAttributes,
    physx::PxFilterData filter_data_a,
    physx::PxFilterObjectAttributes,
    physx::PxFilterData filter_data_b,
    physx::PxPairFlags& pair_flags,
    const void* constant_block,
    physx::PxU32 constant_block_size) {
    std::uint64_t mask_a = static_cast<std::uint64_t>(filter_data_a.word1)
        | (static_cast<std::uint64_t>(filter_data_a.word2) << 32U);
    std::uint64_t mask_b = static_cast<std::uint64_t>(filter_data_b.word1)
        | (static_cast<std::uint64_t>(filter_data_b.word2) << 32U);
    // Legacy bridge shapes carry zero filter data. Treat them as the engine
    // ground/default layer with a full mask so V1 remains compatible and V2
    // humanoid masks can explicitly admit layer 0.
    if (filter_data_a.word0 == 0U && mask_a == 0U) mask_a = ~std::uint64_t{0};
    if (filter_data_b.word0 == 0U && mask_b == 0U) mask_b = ~std::uint64_t{0};
    if (filter_data_a.word0 >= 64U || filter_data_b.word0 >= 64U
        || (mask_a & (std::uint64_t{1} << filter_data_b.word0)) == 0U
        || (mask_b & (std::uint64_t{1} << filter_data_a.word0)) == 0U) {
        return physx::PxFilterFlag::eSUPPRESS;
    }
    if (filter_data_a.word3 != filter_data_b.word3) {
        if (constant_block != nullptr
            && constant_block_size % sizeof(NePhysXArticulationCollisionExclusionV2) == 0U) {
            const auto* exclusions = static_cast<
                const NePhysXArticulationCollisionExclusionV2*>(constant_block);
            const std::uint32_t count = constant_block_size
                / sizeof(NePhysXArticulationCollisionExclusionV2);
            const std::uint32_t first = physx::PxMin(filter_data_a.word3, filter_data_b.word3);
            const std::uint32_t second = physx::PxMax(filter_data_a.word3, filter_data_b.word3);
            for (std::uint32_t index = 0; index < count; ++index) {
                if (exclusions[index].first_link_index == first
                    && exclusions[index].second_link_index == second) {
                    return physx::PxFilterFlag::eSUPPRESS;
                }
            }
        }
    }
    pair_flags = physx::PxPairFlag::eCONTACT_DEFAULT
        | physx::PxPairFlag::eNOTIFY_TOUCH_FOUND
        | physx::PxPairFlag::eNOTIFY_TOUCH_PERSISTS
        | physx::PxPairFlag::eNOTIFY_CONTACT_POINTS;
    return physx::PxFilterFlag::eDEFAULT;
}

struct World {
    physx::PxFoundation* foundation = nullptr;
    physx::PxPhysics* physics = nullptr;
    physx::PxDefaultCpuDispatcher* dispatcher = nullptr;
    physx::PxScene* scene = nullptr;
    physx::PxMaterial* material = nullptr;
    physx::PxArticulationReducedCoordinate* articulation = nullptr;
    physx::PxArticulationCache* articulation_cache = nullptr;
    physx::PxArticulationLink** links = nullptr;
    std::uint64_t* link_tokens = nullptr;
    std::uint32_t* joint_dof_indices = nullptr;
    std::uint32_t link_count = 0;
    std::uint32_t joint_count = 0;
    physx::PxRigidActor** actors = nullptr;
    std::uint32_t actor_count = 0;
    std::uint32_t actor_capacity = 0;
    StaticBox* boxes = nullptr;
    std::uint32_t box_count = 0;
    std::uint32_t box_capacity = 0;
    float timestep = 0.0F;
    ContactSink contact_sink;
};

struct SdkRuntime {
    physx::PxDefaultAllocator allocator;
    physx::PxDefaultErrorCallback error_callback;
    physx::PxFoundation* foundation = nullptr;
    physx::PxPhysics* physics = nullptr;
    std::uint32_t reference_count = 0;
    std::mutex mutex;
};

SdkRuntime& sdk_runtime() {
    static SdkRuntime runtime;
    return runtime;
}

bool acquire_sdk(World& world) {
    SdkRuntime& runtime = sdk_runtime();
    std::lock_guard<std::mutex> lock(runtime.mutex);
    if (runtime.reference_count == 0) {
        runtime.foundation =
            PxCreateFoundation(PX_PHYSICS_VERSION, runtime.allocator, runtime.error_callback);
        if (runtime.foundation == nullptr) {
            return false;
        }
        const physx::PxTolerancesScale scale;
        runtime.physics =
            PxCreatePhysics(PX_PHYSICS_VERSION, *runtime.foundation, scale, true, nullptr);
        if (runtime.physics == nullptr) {
            runtime.foundation->release();
            runtime.foundation = nullptr;
            return false;
        }
    }
    ++runtime.reference_count;
    world.foundation = runtime.foundation;
    world.physics = runtime.physics;
    return true;
}

void release_sdk() {
    SdkRuntime& runtime = sdk_runtime();
    std::lock_guard<std::mutex> lock(runtime.mutex);
    if (runtime.reference_count == 0) {
        return;
    }
    --runtime.reference_count;
    if (runtime.reference_count == 0) {
        runtime.physics->release();
        runtime.foundation->release();
        runtime.physics = nullptr;
        runtime.foundation = nullptr;
    }
}

physx::PxTransform read_transform(const std::uint32_t* position, const std::uint32_t* rotation) {
    return physx::PxTransform(
        physx::PxVec3(from_bits(position[0]), from_bits(position[1]), from_bits(position[2])),
        physx::PxQuat(
            from_bits(rotation[0]),
            from_bits(rotation[1]),
            from_bits(rotation[2]),
            from_bits(rotation[3])));
}

bool valid_transform(const physx::PxTransform& transform) {
    return transform.isFinite() && transform.q.isUnit();
}

bool attach_shape(World& world, physx::PxRigidActor& actor, std::uint32_t kind, const std::uint32_t* dimensions) {
    if (world.material == nullptr) {
        return false;
    }
    physx::PxShape* shape = nullptr;
    if (kind == 1) {
        const physx::PxVec3 half_extents(
            from_bits(dimensions[0]), from_bits(dimensions[1]), from_bits(dimensions[2]));
        if (!finite_positive(half_extents.x) || !finite_positive(half_extents.y)
            || !finite_positive(half_extents.z)) {
            return false;
        }
        shape = world.physics->createShape(physx::PxBoxGeometry(half_extents), *world.material, true);
    } else if (kind == 2) {
        const float radius = from_bits(dimensions[0]);
        if (!finite_positive(radius)) {
            return false;
        }
        shape = world.physics->createShape(physx::PxSphereGeometry(radius), *world.material, true);
    } else if (kind == 3) {
        const float radius = from_bits(dimensions[0]);
        const float half_height = from_bits(dimensions[1]);
        if (!finite_positive(radius) || !finite_non_negative(half_height)) {
            return false;
        }
        shape = world.physics->createShape(physx::PxCapsuleGeometry(radius, half_height), *world.material, true);
    } else {
        return false;
    }
    if (shape == nullptr) {
        return false;
    }
    shape->setTorsionalPatchRadius(0.0F);
    shape->setMinTorsionalPatchRadius(0.0F);
    const bool attached = actor.attachShape(*shape);
    shape->release();
    return attached;
}

bool attach_shape_v2(
    World& world,
    physx::PxRigidActor& actor,
    const NePhysXArticulationShapeInputV2& input) {
    if (world.material == nullptr) {
        return false;
    }
    physx::PxShape* shape = nullptr;
    if (input.shape_kind == 1) {
        const physx::PxVec3 half_extents(
            from_bits(input.shape_dimensions_bits[0]),
            from_bits(input.shape_dimensions_bits[1]),
            from_bits(input.shape_dimensions_bits[2]));
        if (!finite_positive(half_extents.x) || !finite_positive(half_extents.y)
            || !finite_positive(half_extents.z)) {
            return false;
        }
        shape = world.physics->createShape(
            physx::PxBoxGeometry(half_extents), *world.material, true);
    } else if (input.shape_kind == 2) {
        const float radius = from_bits(input.shape_dimensions_bits[0]);
        if (!finite_positive(radius)) {
            return false;
        }
        shape = world.physics->createShape(
            physx::PxSphereGeometry(radius), *world.material, true);
    } else if (input.shape_kind == 3) {
        const float radius = from_bits(input.shape_dimensions_bits[0]);
        const float half_height = from_bits(input.shape_dimensions_bits[1]);
        if (!finite_positive(radius) || !finite_non_negative(half_height)) {
            return false;
        }
        shape = world.physics->createShape(
            physx::PxCapsuleGeometry(radius, half_height), *world.material, true);
    } else {
        return false;
    }
    if (shape == nullptr) {
        return false;
    }
    shape->setTorsionalPatchRadius(0.0F);
    shape->setMinTorsionalPatchRadius(0.0F);
    const physx::PxTransform local_pose = read_transform(input.position_bits, input.rotation_bits);
    if (!valid_transform(local_pose) || input.collision_layer >= 64U) {
        shape->release();
        return false;
    }
    shape->setLocalPose(local_pose);
    shape->setSimulationFilterData(physx::PxFilterData(
        input.collision_layer,
        input.collision_mask_low,
        input.collision_mask_high,
        input.link_index));
    shape->setQueryFilterData(physx::PxFilterData(
        input.collision_layer,
        input.collision_mask_low,
        input.collision_mask_high,
        input.link_index));
    shape->userData = reinterpret_cast<void*>(static_cast<std::uintptr_t>(input.user_token));
    const bool attached = actor.attachShape(*shape);
    shape->release();
    return attached;
}

bool record_joint_dof_indices(World& world, std::uint32_t link_count, std::uint32_t joint_count) {
    for (std::uint32_t semantic_index = 0; semantic_index < joint_count; ++semantic_index) {
        const std::uint32_t child_link_index = semantic_index + 1;
        if (child_link_index >= link_count || world.links[child_link_index] == nullptr) {
            return false;
        }
        const physx::PxArticulationLink* target = world.links[child_link_index];
        if (target->getInboundJointDof() != 1U) {
            return false;
        }
        const std::uint32_t low_level_link_index = target->getLinkIndex();
        std::uint32_t dof = 0;
        for (std::uint32_t other_index = 1; other_index < link_count; ++other_index) {
            const physx::PxArticulationLink* other = world.links[other_index];
            if (other == nullptr) {
                return false;
            }
            if (other->getLinkIndex() < low_level_link_index) {
                dof += other->getInboundJointDof();
            }
        }
        if (dof >= joint_count) {
            return false;
        }
        for (std::uint32_t previous = 0; previous < semantic_index; ++previous) {
            if (world.joint_dof_indices[previous] == dof) {
                return false;
            }
        }
        world.joint_dof_indices[semantic_index] = dof;
    }
    return true;
}

void destroy_world(World* world) {
    if (world == nullptr) {
        return;
    }
    if (world->articulation_cache != nullptr) {
        world->articulation_cache->release();
    }
    if (world->articulation != nullptr) {
        world->articulation->release();
    }
    for (std::uint32_t index = 0; index < world->actor_count; ++index) {
        world->actors[index]->release();
    }
    if (world->scene != nullptr) {
        world->scene->release();
    }
    if (world->dispatcher != nullptr) {
        world->dispatcher->release();
    }
    if (world->material != nullptr) {
        world->material->release();
    }
    const bool has_sdk = world->physics != nullptr;
    delete[] world->links;
    delete[] world->link_tokens;
    delete[] world->joint_dof_indices;
    delete[] world->actors;
    delete[] world->boxes;
    delete[] world->contact_sink.contacts;
    delete world;
    if (has_sdk) {
        release_sdk();
    }
}

}  // namespace

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
    *output = nullptr;
    auto* world = new (std::nothrow) World{};
    if (world == nullptr) {
        return kOutOfMemory;
    }
    if (!acquire_sdk(*world)) {
        delete world;
        return kInternalFailure;
    }
    *output = world;
    return kOk;
}

void ne_physx_world_destroy(void* opaque_world) noexcept {
    destroy_world(static_cast<World*>(opaque_world));
}

std::int32_t ne_physx_world_configure_material(
    void* opaque_world,
    const NePhysXMaterialProfile* input) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || input == nullptr || world->material != nullptr
        || world->scene != nullptr) {
        return kInvalidArgument;
    }
    for (std::uint32_t index = 0; index < 5U; ++index) {
        if (input->coefficient_combine_rules[index] != kArithmeticMeanTiesToEven) {
            return kInvalidArgument;
        }
    }
    if (input->surface_velocity_combine_rule != kCanonicalParticipantOrder
        || input->rolling_friction != 0U || input->spinning_friction != 0U
        || input->surface_velocity_micrometres_per_second[0] != 0
        || input->surface_velocity_micrometres_per_second[1] != 0
        || input->surface_velocity_micrometres_per_second[2] != 0) {
        return kInvalidArgument;
    }
    float static_friction = 0.0F;
    float dynamic_friction = 0.0F;
    float restitution = 0.0F;
    if (input->coefficient_encoding == kMaterialEncodingF32Bits) {
        static_friction = from_bits(input->static_friction);
        dynamic_friction = from_bits(input->dynamic_friction);
        restitution = from_bits(input->restitution);
    } else if (input->coefficient_encoding == kMaterialEncodingQ16) {
        constexpr float kQ16Scale = 65536.0F;
        static_friction = static_cast<float>(input->static_friction) / kQ16Scale;
        dynamic_friction = static_cast<float>(input->dynamic_friction) / kQ16Scale;
        restitution = static_cast<float>(input->restitution) / kQ16Scale;
    } else {
        return kInvalidArgument;
    }
    if (!finite_non_negative(static_friction) || !finite_non_negative(dynamic_friction)
        || dynamic_friction > static_friction || !finite_non_negative(restitution)
        || restitution > 1.0F) {
        return kInvalidArgument;
    }
    physx::PxMaterial* material =
        world->physics->createMaterial(static_friction, dynamic_friction, restitution);
    if (material == nullptr) {
        return kInternalFailure;
    }
    material->setFrictionCombineMode(physx::PxCombineMode::eAVERAGE);
    material->setRestitutionCombineMode(physx::PxCombineMode::eAVERAGE);
    world->material = material;
    return kOk;
}

std::int32_t ne_physx_world_configure_scene(
    void* opaque_world,
    const NePhysXSceneProfile* input) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || input == nullptr || world->material == nullptr
        || world->scene != nullptr
        || input->position_iterations == 0 || input->velocity_iterations == 0
        || input->max_actors == 0 || input->max_joints == 0) {
        return kInvalidArgument;
    }
    const physx::PxVec3 gravity(
        from_bits(input->gravity_bits[0]),
        from_bits(input->gravity_bits[1]),
        from_bits(input->gravity_bits[2]));
    const float timestep = from_bits(input->timestep_bits);
    if (!finite_vec(gravity) || !finite_positive(timestep)) {
        return kInvalidArgument;
    }
    world->dispatcher = physx::PxDefaultCpuDispatcherCreate(1);
    world->actors = new (std::nothrow) physx::PxRigidActor*[input->max_actors]{};
    world->links = new (std::nothrow) physx::PxArticulationLink*[input->max_actors]{};
    world->link_tokens = new (std::nothrow) std::uint64_t[input->max_actors]{};
    world->joint_dof_indices = new (std::nothrow) std::uint32_t[input->max_joints]{};
    world->contact_sink.contacts = input->max_contacts == 0
        ? nullptr
        : new (std::nothrow) NePhysXContactOutputV2[input->max_contacts]{};
    if (world->dispatcher == nullptr || world->actors == nullptr || world->links == nullptr
        || world->link_tokens == nullptr
        || world->joint_dof_indices == nullptr
        || (input->max_contacts != 0 && world->contact_sink.contacts == nullptr)) {
        return kOutOfMemory;
    }
    world->actor_capacity = input->max_actors;
    world->contact_sink.capacity = input->max_contacts;
    physx::PxSceneDesc description(world->physics->getTolerancesScale());
    description.gravity = gravity;
    description.cpuDispatcher = world->dispatcher;
    description.filterShader = contact_filter_shader;
    description.simulationEventCallback = &world->contact_sink;
    description.solverType = physx::PxSolverType::eTGS;
    description.broadPhaseType = physx::PxBroadPhaseType::ePABP;
    description.frictionType = physx::PxFrictionType::ePATCH;
    description.flags.clear(physx::PxSceneFlag::eENABLE_PCM);
    description.flags |= physx::PxSceneFlag::eDISABLE_CONTACT_CACHE;
    description.flags |= physx::PxSceneFlag::eENABLE_ENHANCED_DETERMINISM;
    description.flags |= physx::PxSceneFlag::eDISABLE_SLEEPING;
    description.limits.maxNbActors = input->max_actors;
    description.limits.maxNbBodies = input->max_actors;
    description.limits.maxNbConstraints = input->max_joints;
    world->scene = world->physics->createScene(description);
    if (world->scene == nullptr) {
        return kInternalFailure;
    }
    world->timestep = timestep;
    return kOk;
}

std::int32_t ne_physx_world_reserve(void* opaque_world, std::uint32_t capacity) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || capacity < world->box_count) {
        return kInvalidArgument;
    }
    if (capacity == world->box_capacity) {
        return kOk;
    }
    if (static_cast<std::size_t>(capacity) > std::numeric_limits<std::size_t>::max() / sizeof(StaticBox)) {
        return kCapacityExceeded;
    }
    auto* boxes = capacity == 0 ? nullptr : new (std::nothrow) StaticBox[capacity];
    if (boxes == nullptr && capacity != 0) {
        return kOutOfMemory;
    }
    for (std::uint32_t index = 0; index < world->box_count; ++index) {
        boxes[index] = world->boxes[index];
    }
    delete[] world->boxes;
    world->boxes = boxes;
    world->box_capacity = capacity;
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
    if (world->box_count >= world->box_capacity) {
        return kCapacityExceeded;
    }
    const physx::PxVec3 centre(
        from_bits(centre_bits[0]), from_bits(centre_bits[1]), from_bits(centre_bits[2]));
    const physx::PxVec3 half_extents(
        from_bits(half_extents_bits[0]),
        from_bits(half_extents_bits[1]),
        from_bits(half_extents_bits[2]));
    if (!finite_vec(centre) || !finite_positive(half_extents.x)
        || !finite_positive(half_extents.y) || !finite_positive(half_extents.z)) {
        return kInvalidArgument;
    }
    if (world->scene != nullptr) {
        if (world->actor_count >= world->actor_capacity) {
            return kCapacityExceeded;
        }
        physx::PxRigidStatic* actor = world->physics->createRigidStatic(physx::PxTransform(centre));
        if (actor == nullptr || !attach_shape(*world, *actor, 1, half_extents_bits)) {
            if (actor != nullptr) actor->release();
            return kInternalFailure;
        }
        set_token(*actor, user_token);
        world->scene->addActor(*actor);
        world->actors[world->actor_count++] = actor;
    }
    world->boxes[world->box_count++] = StaticBox{centre, half_extents, user_token};
    return kOk;
}

std::int32_t ne_physx_world_add_dynamic_box(
    void* opaque_world,
    const NePhysXRigidBodyInput* input) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || input == nullptr || world->scene == nullptr
        || world->actor_count >= world->actor_capacity) {
        return kInvalidArgument;
    }
    const physx::PxTransform transform = read_transform(input->position_bits, input->rotation_bits);
    const float mass = from_bits(input->mass_bits);
    const physx::PxVec3 inertia(
        from_bits(input->inertia_bits[0]),
        from_bits(input->inertia_bits[1]),
        from_bits(input->inertia_bits[2]));
    if (!valid_transform(transform) || !finite_positive(mass) || !finite_positive(inertia.x)
        || !finite_positive(inertia.y) || !finite_positive(inertia.z)) {
        return kInvalidArgument;
    }
    physx::PxRigidDynamic* actor = world->physics->createRigidDynamic(transform);
    if (actor == nullptr || !attach_shape(*world, *actor, 1, input->half_extents_bits)) {
        if (actor != nullptr) actor->release();
        return kInternalFailure;
    }
    actor->setMass(mass);
    actor->setMassSpaceInertiaTensor(inertia);
    actor->setSolverIterationCounts(8, 2);
    set_token(*actor, input->user_token);
    world->scene->addActor(*actor);
    world->actors[world->actor_count++] = actor;
    return kOk;
}

std::int32_t ne_physx_world_add_articulation(
    void* opaque_world,
    const NePhysXArticulationLinkInput* link_inputs,
    std::uint32_t link_count,
    const NePhysXArticulationJointInput* joint_inputs,
    std::uint32_t joint_count,
    std::uint32_t position_iterations,
    std::uint32_t velocity_iterations) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || world->scene == nullptr || world->articulation != nullptr
        || link_inputs == nullptr || link_count == 0 || joint_count + 1 != link_count
        || (joint_count != 0 && joint_inputs == nullptr) || link_count > world->actor_capacity
        || position_iterations == 0 || velocity_iterations == 0) {
        return kInvalidArgument;
    }
    physx::PxArticulationReducedCoordinate* articulation =
        world->physics->createArticulationReducedCoordinate();
    if (articulation == nullptr) {
        return kInternalFailure;
    }
    articulation->setSolverIterationCounts(position_iterations, velocity_iterations);
    for (std::uint32_t index = 0; index < link_count; ++index) {
        const NePhysXArticulationLinkInput& input = link_inputs[index];
        if ((index == 0 && input.parent_link_index != kNoParent)
            || (index != 0 && input.parent_link_index >= index)) {
            articulation->release();
            return kInvalidArgument;
        }
        const physx::PxTransform transform = read_transform(input.position_bits, input.rotation_bits);
        const float mass = from_bits(input.mass_bits);
        const physx::PxVec3 inertia(
            from_bits(input.inertia_bits[0]),
            from_bits(input.inertia_bits[1]),
            from_bits(input.inertia_bits[2]));
        const float linear_damping = from_bits(input.linear_damping_bits);
        const float angular_damping = from_bits(input.angular_damping_bits);
        if (!valid_transform(transform) || !finite_positive(mass) || !finite_positive(inertia.x)
            || !finite_positive(inertia.y) || !finite_positive(inertia.z)
            || !finite_non_negative(linear_damping) || !finite_non_negative(angular_damping)) {
            articulation->release();
            return kInvalidArgument;
        }
        physx::PxArticulationLink* parent = index == 0 ? nullptr : world->links[input.parent_link_index];
        physx::PxArticulationLink* link = articulation->createLink(parent, transform);
        if (link == nullptr || !attach_shape(*world, *link, input.shape_kind, input.shape_dimensions_bits)) {
            articulation->release();
            return kInternalFailure;
        }
        link->setMass(mass);
        link->setMassSpaceInertiaTensor(inertia);
        link->setLinearDamping(linear_damping);
        link->setAngularDamping(angular_damping);
        set_token(*link, input.user_token);
        world->links[index] = link;
        world->link_tokens[index] = input.user_token;
        if (index != 0) {
            const NePhysXArticulationJointInput& joint_input = joint_inputs[index - 1];
            if (joint_input.child_link_index != index) {
                articulation->release();
                return kInvalidArgument;
            }
            physx::PxArticulationJointReducedCoordinate* joint = link->getInboundJoint();
            const physx::PxTransform parent_pose =
                read_transform(joint_input.parent_position_bits, joint_input.parent_rotation_bits);
            const physx::PxTransform child_pose =
                read_transform(joint_input.child_position_bits, joint_input.child_rotation_bits);
            const float lower = from_bits(joint_input.lower_limit_bits);
            const float upper = from_bits(joint_input.upper_limit_bits);
            const float max_velocity = from_bits(joint_input.max_velocity_bits);
            if (joint == nullptr || !valid_transform(parent_pose) || !valid_transform(child_pose)
                || !physx::PxIsFinite(lower) || !physx::PxIsFinite(upper) || lower >= upper
                || !finite_positive(max_velocity)) {
                articulation->release();
                return kInvalidArgument;
            }
            joint->setParentPose(parent_pose);
            joint->setChildPose(child_pose);
            joint->setJointType(physx::PxArticulationJointType::eREVOLUTE);
            joint->setMotion(physx::PxArticulationAxis::eTWIST, physx::PxArticulationMotion::eLIMITED);
            joint->setLimitParams(
                physx::PxArticulationAxis::eTWIST,
                physx::PxArticulationLimit(lower, upper));
            joint->setMaxJointVelocity(physx::PxArticulationAxis::eTWIST, max_velocity);
        }
    }
    world->scene->addArticulation(*articulation);
    if (!record_joint_dof_indices(*world, link_count, joint_count)) {
        articulation->release();
        return kInternalFailure;
    }
    physx::PxArticulationCache* cache = articulation->createCache();
    if (cache == nullptr || articulation->getDofs() != joint_count) {
        if (cache != nullptr) cache->release();
        articulation->release();
        return kInternalFailure;
    }
    world->articulation = articulation;
    world->articulation_cache = cache;
    world->link_count = link_count;
    world->joint_count = joint_count;
    return kOk;
}

std::int32_t ne_physx_world_add_articulation_v2(
    void* opaque_world,
    const NePhysXArticulationLinkInputV2* link_inputs,
    std::uint32_t link_count,
    const NePhysXArticulationShapeInputV2* shape_inputs,
    std::uint32_t shape_count,
    const NePhysXArticulationJointInput* joint_inputs,
    std::uint32_t joint_count,
    const NePhysXArticulationCollisionExclusionV2* exclusions,
    std::uint32_t exclusion_count,
    std::uint32_t position_iterations,
    std::uint32_t velocity_iterations) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || world->scene == nullptr || world->articulation != nullptr
        || link_inputs == nullptr || link_count == 0 || joint_count + 1 != link_count
        || (shape_count != 0 && shape_inputs == nullptr)
        || (joint_count != 0 && joint_inputs == nullptr)
        || (exclusion_count != 0 && exclusions == nullptr)
        || link_count > world->actor_capacity || position_iterations == 0
        || velocity_iterations == 0
        || exclusion_count > std::numeric_limits<physx::PxU32>::max()
            / sizeof(NePhysXArticulationCollisionExclusionV2)) {
        return kInvalidArgument;
    }
    if (exclusion_count != 0) {
        world->scene->setFilterShaderData(
            exclusions,
            exclusion_count * sizeof(NePhysXArticulationCollisionExclusionV2));
    }
    physx::PxArticulationReducedCoordinate* articulation =
        world->physics->createArticulationReducedCoordinate();
    if (articulation == nullptr) {
        return kInternalFailure;
    }
    articulation->setSolverIterationCounts(position_iterations, velocity_iterations);
    std::uint32_t next_shape = 0;
    for (std::uint32_t index = 0; index < link_count; ++index) {
        const NePhysXArticulationLinkInputV2& input = link_inputs[index];
        if ((index == 0 && input.parent_link_index != kNoParent)
            || (index != 0 && input.parent_link_index >= index)
            || input.reserved != 0 || input.first_shape_index != next_shape
            || input.shape_count > shape_count - next_shape) {
            articulation->release();
            return kInvalidArgument;
        }
        const physx::PxTransform transform = read_transform(input.position_bits, input.rotation_bits);
        const physx::PxTransform centre_of_mass = read_transform(
            input.centre_of_mass_position_bits, input.centre_of_mass_rotation_bits);
        const float mass = from_bits(input.mass_bits);
        const physx::PxVec3 inertia(
            from_bits(input.inertia_bits[0]),
            from_bits(input.inertia_bits[1]),
            from_bits(input.inertia_bits[2]));
        const float linear_damping = from_bits(input.linear_damping_bits);
        const float angular_damping = from_bits(input.angular_damping_bits);
        if (!valid_transform(transform) || !valid_transform(centre_of_mass)
            || !finite_positive(mass) || !finite_positive(inertia.x)
            || !finite_positive(inertia.y) || !finite_positive(inertia.z)
            || !finite_non_negative(linear_damping) || !finite_non_negative(angular_damping)) {
            articulation->release();
            return kInvalidArgument;
        }
        physx::PxArticulationLink* parent = index == 0
            ? nullptr
            : world->links[input.parent_link_index];
        physx::PxArticulationLink* link = articulation->createLink(parent, transform);
        if (link == nullptr) {
            articulation->release();
            return kInternalFailure;
        }
        for (std::uint32_t shape_offset = 0; shape_offset < input.shape_count; ++shape_offset) {
            const NePhysXArticulationShapeInputV2& shape = shape_inputs[next_shape + shape_offset];
            if (shape.link_index != index || !attach_shape_v2(*world, *link, shape)) {
                articulation->release();
                return kInvalidArgument;
            }
        }
        next_shape += input.shape_count;
        link->setCMassLocalPose(centre_of_mass);
        link->setMass(mass);
        link->setMassSpaceInertiaTensor(inertia);
        link->setLinearDamping(linear_damping);
        link->setAngularDamping(angular_damping);
        set_token(*link, input.user_token);
        world->links[index] = link;
        world->link_tokens[index] = input.user_token;
        if (index != 0) {
            const NePhysXArticulationJointInput& joint_input = joint_inputs[index - 1];
            physx::PxArticulationJointReducedCoordinate* joint = link->getInboundJoint();
            const physx::PxTransform parent_pose =
                read_transform(joint_input.parent_position_bits, joint_input.parent_rotation_bits);
            const physx::PxTransform child_pose =
                read_transform(joint_input.child_position_bits, joint_input.child_rotation_bits);
            const float lower = from_bits(joint_input.lower_limit_bits);
            const float upper = from_bits(joint_input.upper_limit_bits);
            const float max_velocity = from_bits(joint_input.max_velocity_bits);
            if (joint_input.child_link_index != index || joint_input.reserved != 0
                || joint == nullptr || !valid_transform(parent_pose) || !valid_transform(child_pose)
                || !physx::PxIsFinite(lower) || !physx::PxIsFinite(upper) || lower >= upper
                || !finite_positive(max_velocity)) {
                articulation->release();
                return kInvalidArgument;
            }
            joint->setParentPose(parent_pose);
            joint->setChildPose(child_pose);
            joint->setJointType(physx::PxArticulationJointType::eREVOLUTE);
            joint->setMotion(
                physx::PxArticulationAxis::eTWIST,
                physx::PxArticulationMotion::eLIMITED);
            joint->setLimitParams(
                physx::PxArticulationAxis::eTWIST,
                physx::PxArticulationLimit(lower, upper));
            joint->setMaxJointVelocity(physx::PxArticulationAxis::eTWIST, max_velocity);
        }
    }
    if (next_shape != shape_count) {
        articulation->release();
        return kInvalidArgument;
    }
    world->scene->addArticulation(*articulation);
    if (!record_joint_dof_indices(*world, link_count, joint_count)) {
        articulation->release();
        return kInternalFailure;
    }
    physx::PxArticulationCache* cache = articulation->createCache();
    if (cache == nullptr || articulation->getDofs() != joint_count) {
        if (cache != nullptr) cache->release();
        articulation->release();
        return kInternalFailure;
    }
    world->articulation = articulation;
    world->articulation_cache = cache;
    world->link_count = link_count;
    world->joint_count = joint_count;
    return kOk;
}

std::int32_t ne_physx_world_apply_articulation_efforts(
    void* opaque_world,
    const std::uint32_t* effort_bits,
    std::uint32_t effort_count) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || world->articulation == nullptr || world->articulation_cache == nullptr
        || effort_bits == nullptr || effort_count != world->joint_count) {
        return kInvalidArgument;
    }
    world->articulation->zeroCache(*world->articulation_cache);
    for (std::uint32_t index = 0; index < effort_count; ++index) {
        const float effort = from_bits(effort_bits[index]);
        if (!physx::PxIsFinite(effort)) {
            return kInvalidArgument;
        }
        world->articulation_cache->jointForce[world->joint_dof_indices[index]] = effort;
    }
    world->articulation->applyCache(
        *world->articulation_cache, physx::PxArticulationCacheFlag::eFORCE, true);
    return kOk;
}

std::int32_t ne_physx_world_import_articulation_state(
    void* opaque_world,
    const NePhysXLinkState* root_state,
    const NePhysXJointState* joint_states,
    std::uint32_t joint_count) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || world->articulation == nullptr || world->articulation_cache == nullptr
        || root_state == nullptr || (joint_count != 0 && joint_states == nullptr)
        || joint_count != world->joint_count || root_state->user_token != world->link_tokens[0]) {
        return kInvalidArgument;
    }
    const physx::PxTransform root = read_transform(root_state->position_bits, root_state->rotation_bits);
    const physx::PxVec3 linear(
        from_bits(root_state->linear_velocity_bits[0]),
        from_bits(root_state->linear_velocity_bits[1]),
        from_bits(root_state->linear_velocity_bits[2]));
    const physx::PxVec3 angular(
        from_bits(root_state->angular_velocity_bits[0]),
        from_bits(root_state->angular_velocity_bits[1]),
        from_bits(root_state->angular_velocity_bits[2]));
    if (!valid_transform(root) || !finite_vec(linear) || !finite_vec(angular)) {
        return kInvalidArgument;
    }
    world->articulation->zeroCache(*world->articulation_cache);
    world->articulation_cache->rootLinkData->transform = root;
    world->articulation_cache->rootLinkData->worldLinVel = linear;
    world->articulation_cache->rootLinkData->worldAngVel = angular;
    for (std::uint32_t index = 0; index < joint_count; ++index) {
        const float position = from_bits(joint_states[index].position_bits);
        const float velocity = from_bits(joint_states[index].velocity_bits);
        if (!physx::PxIsFinite(position) || !physx::PxIsFinite(velocity)) {
            return kInvalidArgument;
        }
        const std::uint32_t dof = world->joint_dof_indices[index];
        world->articulation_cache->jointPosition[dof] = position;
        world->articulation_cache->jointVelocity[dof] = velocity;
    }
    world->articulation->applyCache(
        *world->articulation_cache,
        physx::PxArticulationCacheFlag::eROOT_TRANSFORM
            | physx::PxArticulationCacheFlag::eROOT_VELOCITIES
            | physx::PxArticulationCacheFlag::ePOSITION
            | physx::PxArticulationCacheFlag::eVELOCITY,
        true);
    return kOk;
}

std::int32_t ne_physx_world_step(void* opaque_world) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || world->scene == nullptr || !finite_positive(world->timestep)) {
        return kInvalidArgument;
    }
    world->contact_sink.clear();
    world->scene->simulate(world->timestep);
    if (!world->scene->fetchResults(true)) {
        return kInternalFailure;
    }
    return world->contact_sink.overflow ? kCapacityExceeded : kOk;
}

std::int32_t ne_physx_world_export_articulation_state(
    void* opaque_world,
    NePhysXLinkState* link_states,
    std::uint32_t link_capacity,
    std::uint32_t* link_count,
    NePhysXJointState* joint_states,
    std::uint32_t joint_capacity,
    std::uint32_t* joint_count) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || world->articulation == nullptr || world->articulation_cache == nullptr
        || link_count == nullptr || joint_count == nullptr
        || (world->link_count != 0 && link_states == nullptr)
        || (world->joint_count != 0 && joint_states == nullptr)) {
        return kInvalidArgument;
    }
    *link_count = world->link_count;
    *joint_count = world->joint_count;
    if (link_capacity < world->link_count || joint_capacity < world->joint_count) {
        return kCapacityExceeded;
    }
    world->articulation->copyInternalStateToCache(
        *world->articulation_cache,
        physx::PxArticulationCacheFlag::ePOSITION
            | physx::PxArticulationCacheFlag::eVELOCITY
            | physx::PxArticulationCacheFlag::eLINK_VELOCITY);
    for (std::uint32_t index = 0; index < world->link_count; ++index) {
        const physx::PxTransform transform = world->links[index]->getGlobalPose();
        const physx::PxVec3 linear = world->links[index]->getLinearVelocity();
        const physx::PxVec3 angular = world->links[index]->getAngularVelocity();
        NePhysXLinkState& output = link_states[index];
        output.user_token = world->link_tokens[index];
        for (std::uint32_t axis = 0; axis < 3; ++axis) {
            output.position_bits[axis] = to_bits(transform.p[axis]);
            output.linear_velocity_bits[axis] = to_bits(linear[axis]);
            output.angular_velocity_bits[axis] = to_bits(angular[axis]);
        }
        output.rotation_bits[0] = to_bits(transform.q.x);
        output.rotation_bits[1] = to_bits(transform.q.y);
        output.rotation_bits[2] = to_bits(transform.q.z);
        output.rotation_bits[3] = to_bits(transform.q.w);
    }
    for (std::uint32_t index = 0; index < world->joint_count; ++index) {
        const std::uint32_t dof = world->joint_dof_indices[index];
        joint_states[index].position_bits = to_bits(world->articulation_cache->jointPosition[dof]);
        joint_states[index].velocity_bits = to_bits(world->articulation_cache->jointVelocity[dof]);
    }
    return kOk;
}

std::int32_t ne_physx_world_export_contacts(
    void* opaque_world,
    NePhysXContactOutput* output,
    std::uint32_t capacity,
    std::uint32_t* count) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || count == nullptr
        || (world->contact_sink.length != 0 && output == nullptr)) {
        return kInvalidArgument;
    }
    *count = world->contact_sink.length;
    if (capacity < world->contact_sink.length) {
        return kCapacityExceeded;
    }
    for (std::uint32_t index = 0; index < world->contact_sink.length; ++index) {
        const NePhysXContactOutputV2& source = world->contact_sink.contacts[index];
        NePhysXContactOutput& target = output[index];
        target.actor_a_token = source.actor_a_token;
        target.actor_b_token = source.actor_b_token;
        for (std::uint32_t axis = 0; axis < 3; ++axis) {
            target.position_bits[axis] = source.position_bits[axis];
            target.normal_bits[axis] = source.normal_bits[axis];
            target.impulse_bits[axis] = source.impulse_bits[axis];
        }
        target.separation_bits = source.separation_bits;
    }
    return kOk;
}

std::int32_t ne_physx_world_export_contacts_v2(
    void* opaque_world,
    NePhysXContactOutputV2* output,
    std::uint32_t capacity,
    std::uint32_t* count) noexcept {
    auto* world = static_cast<World*>(opaque_world);
    if (world == nullptr || count == nullptr
        || (world->contact_sink.length != 0 && output == nullptr)) {
        return kInvalidArgument;
    }
    *count = world->contact_sink.length;
    if (capacity < world->contact_sink.length) {
        return kCapacityExceeded;
    }
    for (std::uint32_t index = 0; index < world->contact_sink.length; ++index) {
        output[index] = world->contact_sink.contacts[index];
    }
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
    const physx::PxVec3 centre(
        from_bits(centre_bits[0]), from_bits(centre_bits[1]), from_bits(centre_bits[2]));
    const float radius = from_bits(radius_bits);
    const float half_segment = from_bits(half_segment_bits);
    const float distance = from_bits(distance_bits);
    if (!finite_vec(centre) || !finite_positive(radius) || !finite_non_negative(half_segment)
        || !finite_non_negative(distance)) {
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
    for (std::uint32_t index = 0; index < world->box_count; ++index) {
        const StaticBox& box = world->boxes[index];
        const physx::PxBoxGeometry geometry(box.half_extents);
        const physx::PxTransform pose(box.centre);
        physx::PxSweepHit hit{};
        const bool collided = physx::PxGeometryQuery::sweep(
            direction, distance, capsule, capsule_pose, geometry, pose, hit,
            physx::PxHitFlag::ePOSITION | physx::PxHitFlag::eNORMAL, 0.0F);
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

// Plan `continuum-water/23` (ADR-106): the GPU fluid behind the probe and
// the plan 24 lane. Reasons: 1 the GPU library or the CUDA device is
// unavailable, 2 the CUDA context is invalid, 3 the GPU scene failed, 4 the
// particle system or material failed, 5 the particle buffer failed, 6 a
// simulation step or readback failed.
namespace {

struct GpuLibraryHook final : ::PxGpuLoadHook {
    const char* path = nullptr;
    const char* getPhysXGpuDllName() const override { return path; }
};

std::uint64_t microseconds_since(std::chrono::steady_clock::time_point started) {
    return static_cast<std::uint64_t>(std::chrono::duration_cast<std::chrono::microseconds>(
                                          std::chrono::steady_clock::now() - started)
                                          .count());
}

struct GpuFluid {
    World holder;
    physx::PxCudaContextManager* cuda = nullptr;
    physx::PxDefaultCpuDispatcher* dispatcher = nullptr;
    physx::PxScene* scene = nullptr;
    physx::PxMaterial* material = nullptr;
    physx::PxRigidStatic* planes[5] = {};
    physx::PxPBDParticleSystem* particles = nullptr;
    physx::PxPBDMaterial* fluid_material = nullptr;
    physx::PxParticleBuffer* buffer = nullptr;
    physx::PxVec4* host_positions = nullptr;
    physx::PxVec4* host_velocities = nullptr;
    std::uint32_t count = 0;
    std::uint32_t capacity = 0;
    float inverse_mass = 1.0F;
    float timestep = 0.0F;
    char device_name[64] = {};
};

void destroy_gpu_fluid(GpuFluid* fluid) {
    if (fluid == nullptr) {
        return;
    }
    if (fluid->buffer != nullptr) {
        if (fluid->particles != nullptr) {
            fluid->particles->removeParticleBuffer(fluid->buffer);
        }
        fluid->buffer->release();
    }
    if (fluid->particles != nullptr) {
        if (fluid->scene != nullptr) {
            fluid->scene->removeActor(*fluid->particles);
        }
        fluid->particles->release();
    }
    if (fluid->fluid_material != nullptr) {
        fluid->fluid_material->release();
    }
    for (physx::PxRigidStatic* plane : fluid->planes) {
        if (plane != nullptr) {
            plane->release();
        }
    }
    if (fluid->material != nullptr) {
        fluid->material->release();
    }
    if (fluid->scene != nullptr) {
        fluid->scene->release();
    }
    if (fluid->dispatcher != nullptr) {
        fluid->dispatcher->release();
    }
    if (fluid->cuda != nullptr) {
        fluid->cuda->release();
    }
    delete[] fluid->host_positions;
    delete[] fluid->host_velocities;
    const bool has_sdk = fluid->holder.physics != nullptr;
    delete fluid;
    if (has_sdk) {
        release_sdk();
    }
}

/// Creates the GPU scene, the box and the fluid seeded from `seed`
/// (`count` particles, positions with inverse mass). Returns null with the
/// reason set when the GPU lane is unavailable or a step of the creation
/// fails; the CPU SDK stays usable.
GpuFluid* create_gpu_fluid(
    const char* gpu_library_path,
    float spacing,
    const float box_min[3],
    const float box_max[3],
    float timestep,
    const physx::PxVec4* seed,
    std::uint32_t count,
    std::uint32_t capacity,
    std::uint32_t* reason) {
    *reason = 0;
    auto* fluid = new (std::nothrow) GpuFluid{};
    if (fluid == nullptr) {
        *reason = 5;
        return nullptr;
    }
    if (!acquire_sdk(fluid->holder)) {
        delete fluid;
        *reason = 3;
        return nullptr;
    }
    static GpuLibraryHook hook;
    if (gpu_library_path != nullptr) {
        hook.path = gpu_library_path;
        PxSetPhysXGpuLoadHook(&hook);
    }
    physx::PxCudaContextManagerDesc cuda_desc;
    fluid->cuda =
        PxCreateCudaContextManager(*fluid->holder.foundation, cuda_desc, PxGetProfilerCallback());
    if (fluid->cuda == nullptr) {
        *reason = 1;
        destroy_gpu_fluid(fluid);
        return nullptr;
    }
    if (!fluid->cuda->contextIsValid()) {
        *reason = 2;
        destroy_gpu_fluid(fluid);
        return nullptr;
    }
    const char* device = fluid->cuda->getDeviceName();
    if (device != nullptr) {
        std::strncpy(fluid->device_name, device, sizeof(fluid->device_name) - 1);
    }
    fluid->dispatcher = physx::PxDefaultCpuDispatcherCreate(1);
    physx::PxSceneDesc scene_desc(fluid->holder.physics->getTolerancesScale());
    scene_desc.gravity = physx::PxVec3(0.0F, -9.81F, 0.0F);
    scene_desc.cpuDispatcher = fluid->dispatcher;
    scene_desc.filterShader = physx::PxDefaultSimulationFilterShader;
    scene_desc.cudaContextManager = fluid->cuda;
    scene_desc.flags |= physx::PxSceneFlag::eENABLE_GPU_DYNAMICS;
    scene_desc.flags |= physx::PxSceneFlag::eENABLE_PCM;
    scene_desc.broadPhaseType = physx::PxBroadPhaseType::eGPU;
    scene_desc.solverType = physx::PxSolverType::eTGS;
    fluid->scene =
        fluid->dispatcher == nullptr ? nullptr : fluid->holder.physics->createScene(scene_desc);
    fluid->material = fluid->holder.physics->createMaterial(0.5F, 0.5F, 0.1F);
    if (fluid->scene == nullptr || fluid->material == nullptr) {
        *reason = 3;
        destroy_gpu_fluid(fluid);
        return nullptr;
    }
    // A floor at the box bottom and four walls at its x/z faces, open at
    // the top. PxPlane(n, d) is the set n . p + d = 0.
    const physx::PxPlane plane_shapes[5] = {
        physx::PxPlane(physx::PxVec3(0.0F, 1.0F, 0.0F), -box_min[1]),
        physx::PxPlane(physx::PxVec3(1.0F, 0.0F, 0.0F), -box_min[0]),
        physx::PxPlane(physx::PxVec3(-1.0F, 0.0F, 0.0F), box_max[0]),
        physx::PxPlane(physx::PxVec3(0.0F, 0.0F, 1.0F), -box_min[2]),
        physx::PxPlane(physx::PxVec3(0.0F, 0.0F, -1.0F), box_max[2]),
    };
    for (int index = 0; index < 5; ++index) {
        fluid->planes[index] =
            physx::PxCreatePlane(*fluid->holder.physics, plane_shapes[index], *fluid->material);
        if (fluid->planes[index] != nullptr) {
            fluid->scene->addActor(*fluid->planes[index]);
        }
    }
    // The fluid: the SDK snippet's offsets from the particle spacing.
    const float rest_offset = 0.5F * spacing / 0.6F;
    const float fluid_rest_offset = rest_offset * 0.6F;
    const float contact_offset = rest_offset + 0.01F;
    fluid->particles = fluid->holder.physics->createPBDParticleSystem(*fluid->cuda, 96);
    fluid->fluid_material = fluid->holder.physics->createPBDMaterial(
        0.05F, 0.05F, 0.0F, 0.001F, 0.005F, 0.005F, 0.5F, 0.0F, 0.0F);
    if (fluid->particles == nullptr || fluid->fluid_material == nullptr) {
        *reason = 4;
        destroy_gpu_fluid(fluid);
        return nullptr;
    }
    fluid->particles->setRestOffset(rest_offset);
    fluid->particles->setContactOffset(contact_offset);
    fluid->particles->setParticleContactOffset(contact_offset);
    fluid->particles->setSolidRestOffset(rest_offset);
    fluid->particles->setFluidRestOffset(fluid_rest_offset);
    fluid->scene->addActor(*fluid->particles);
    const physx::PxU32 phase = fluid->particles->createPhase(
        fluid->fluid_material,
        physx::PxParticlePhaseFlags(
            physx::PxParticlePhaseFlag::eParticlePhaseFluid
            | physx::PxParticlePhaseFlag::eParticlePhaseSelfCollide));
    if (capacity < count || capacity == 0) {
        *reason = 5;
        destroy_gpu_fluid(fluid);
        return nullptr;
    }
    fluid->host_positions = new (std::nothrow) physx::PxVec4[capacity];
    fluid->host_velocities = new (std::nothrow) physx::PxVec4[capacity];
    auto* phases = new (std::nothrow) physx::PxU32[capacity];
    if (fluid->host_positions == nullptr || fluid->host_velocities == nullptr
        || phases == nullptr) {
        delete[] phases;
        *reason = 5;
        destroy_gpu_fluid(fluid);
        return nullptr;
    }
    const float rest_inverse_mass =
        count > 0 ? seed[0].w : 1.0F / (1000.0F * 1.333F * 3.14159F * fluid_rest_offset
                                          * fluid_rest_offset * fluid_rest_offset);
    for (std::uint32_t index = 0; index < capacity; ++index) {
        fluid->host_positions[index] = index < count
            ? seed[index]
            : physx::PxVec4(box_min[0], box_min[1], box_min[2], rest_inverse_mass);
        fluid->host_velocities[index] = physx::PxVec4(0.0F);
        phases[index] = phase;
    }
    fluid->inverse_mass = rest_inverse_mass;
    physx::ExtGpu::PxParticleBufferDesc buffer_desc;
    buffer_desc.positions = fluid->host_positions;
    buffer_desc.velocities = fluid->host_velocities;
    buffer_desc.phases = phases;
    buffer_desc.numActiveParticles = count;
    buffer_desc.maxParticles = capacity;
    fluid->buffer = physx::ExtGpu::PxCreateAndPopulateParticleBuffer(buffer_desc, fluid->cuda);
    delete[] phases;
    if (fluid->buffer == nullptr) {
        *reason = 5;
        destroy_gpu_fluid(fluid);
        return nullptr;
    }
    fluid->particles->addParticleBuffer(fluid->buffer);
    fluid->count = count;
    fluid->capacity = capacity;
    fluid->timestep = timestep;
    return fluid;
}

/// Replaces the active particle set (`count <= capacity`) from host arrays
/// of positions and velocities (metres, metres per second).
bool gpu_fluid_set(GpuFluid& fluid, const float* positions, const float* velocities,
                   std::uint32_t count) {
    if (count > fluid.capacity) {
        return false;
    }
    for (std::uint32_t index = 0; index < count; ++index) {
        fluid.host_positions[index] = physx::PxVec4(
            positions[index * 3], positions[index * 3 + 1], positions[index * 3 + 2],
            fluid.inverse_mass);
        fluid.host_velocities[index] = physx::PxVec4(
            velocities[index * 3], velocities[index * 3 + 1], velocities[index * 3 + 2], 0.0F);
    }
    fluid.cuda->acquireContext();
    physx::PxCudaContext* context = fluid.cuda->getCudaContext();
    physx::PxCUresult copied = 0;
    if (count > 0) {
        copied = context->memcpyHtoD(
            reinterpret_cast<CUdeviceptr>(fluid.buffer->getPositionInvMasses()),
            fluid.host_positions,
            static_cast<std::size_t>(count) * sizeof(physx::PxVec4));
        if (copied == 0) {
            copied = context->memcpyHtoD(
                reinterpret_cast<CUdeviceptr>(fluid.buffer->getVelocities()),
                fluid.host_velocities,
                static_cast<std::size_t>(count) * sizeof(physx::PxVec4));
        }
    }
    fluid.cuda->releaseContext();
    if (copied != 0) {
        return false;
    }
    fluid.buffer->setNbActiveParticles(count);
    fluid.buffer->raiseFlags(physx::PxParticleBufferFlag::eUPDATE_POSITION);
    fluid.buffer->raiseFlags(physx::PxParticleBufferFlag::eUPDATE_VELOCITY);
    fluid.count = count;
    return true;
}

bool gpu_fluid_step(GpuFluid& fluid) {
    fluid.scene->simulate(fluid.timestep);
    if (!fluid.scene->fetchResults(true)) {
        return false;
    }
    fluid.scene->fetchResultsParticleSystem();
    return true;
}

bool gpu_fluid_readback(GpuFluid& fluid, bool velocities) {
    fluid.cuda->acquireContext();
    physx::PxCudaContext* context = fluid.cuda->getCudaContext();
    physx::PxCUresult copied = context->memcpyDtoH(
        fluid.host_positions,
        reinterpret_cast<CUdeviceptr>(fluid.buffer->getPositionInvMasses()),
        static_cast<std::size_t>(fluid.count) * sizeof(physx::PxVec4));
    if (copied == 0 && velocities) {
        copied = context->memcpyDtoH(
            fluid.host_velocities,
            reinterpret_cast<CUdeviceptr>(fluid.buffer->getVelocities()),
            static_cast<std::size_t>(fluid.count) * sizeof(physx::PxVec4));
    }
    fluid.cuda->releaseContext();
    return copied == 0;
}

}  // namespace

std::int32_t ne_physx_pbd_probe(
    const NePhysXPbdProbeDesc* desc,
    NePhysXPbdProbeReport* report) noexcept {
    if (desc == nullptr || report == nullptr || desc->particle_count == 0 || desc->frames == 0
        || desc->final_positions == nullptr) {
        return kInvalidArgument;
    }
    *report = NePhysXPbdProbeReport{};
    const float spacing = from_bits(desc->spacing_bits);
    const float box_half = from_bits(desc->box_half_bits);
    const float timestep = from_bits(desc->timestep_bits);
    const float margin = from_bits(desc->margin_bits);
    if (!finite_positive(spacing) || !finite_positive(box_half) || !finite_positive(timestep)
        || !finite_non_negative(margin)) {
        return kInvalidArgument;
    }
    // Host seed: a column of particles on the spacing grid over the floor.
    const std::uint32_t count = desc->particle_count;
    const std::uint32_t per_axis =
        static_cast<std::uint32_t>((2.0F * box_half - spacing) / spacing);
    if (per_axis == 0) {
        return kInvalidArgument;
    }
    const float rest_offset = 0.5F * spacing / 0.6F;
    const float fluid_rest_offset = rest_offset * 0.6F;
    const float particle_mass = 1000.0F * 1.333F * 3.14159F * fluid_rest_offset
        * fluid_rest_offset * fluid_rest_offset;
    auto* seed = new (std::nothrow) physx::PxVec4[count];
    if (seed == nullptr) {
        return kOutOfMemory;
    }
    for (std::uint32_t index = 0; index < count; ++index) {
        const std::uint32_t column = index % per_axis;
        const std::uint32_t row = (index / per_axis) % per_axis;
        const std::uint32_t layer = index / (per_axis * per_axis);
        seed[index] = physx::PxVec4(
            -box_half + spacing * (0.5F + static_cast<float>(column)),
            -box_half + spacing * (0.5F + static_cast<float>(layer)),
            -box_half + spacing * (0.5F + static_cast<float>(row)),
            1.0F / particle_mass);
    }
    const float box_min[3] = {-box_half, -box_half, -box_half};
    const float box_max[3] = {box_half, box_half, box_half};
    std::uint32_t reason = 0;
    GpuFluid* fluid = create_gpu_fluid(
        desc->gpu_library_path, spacing, box_min, box_max, timestep, seed, count, count, &reason);
    delete[] seed;
    if (fluid == nullptr) {
        report->reason = reason;
        return kOk;
    }
    std::strncpy(report->device_name, fluid->device_name, sizeof(report->device_name) - 1);
    std::uint64_t step_total = 0;
    std::uint64_t readback_total = 0;
    const float low = -box_half - margin;
    const float high = box_half + margin;
    for (std::uint32_t frame = 0; frame < desc->frames; ++frame) {
        const auto step_started = std::chrono::steady_clock::now();
        if (!gpu_fluid_step(*fluid)) {
            report->reason = 6;
            destroy_gpu_fluid(fluid);
            return kOk;
        }
        const std::uint64_t step_elapsed = microseconds_since(step_started);
        const auto readback_started = std::chrono::steady_clock::now();
        if (!gpu_fluid_readback(*fluid, false)) {
            report->reason = 6;
            destroy_gpu_fluid(fluid);
            return kOk;
        }
        const std::uint64_t readback_elapsed = microseconds_since(readback_started);
        std::uint32_t outside = 0;
        for (std::uint32_t index = 0; index < count; ++index) {
            const physx::PxVec4& position = fluid->host_positions[index];
            if (!physx::PxIsFinite(position.x) || !physx::PxIsFinite(position.y)
                || !physx::PxIsFinite(position.z) || position.x < low || position.x > high
                || position.z < low || position.z > high || position.y < low) {
                ++outside;
            }
        }
        if (outside > report->particles_out_of_bounds_max) {
            report->particles_out_of_bounds_max = outside;
        }
        step_total += step_elapsed;
        readback_total += readback_elapsed;
        if (step_elapsed > report->step_max_microseconds) {
            report->step_max_microseconds = step_elapsed;
        }
        if (readback_elapsed > report->readback_max_microseconds) {
            report->readback_max_microseconds = readback_elapsed;
        }
        report->frames_completed = frame + 1;
    }
    for (std::uint32_t index = 0; index < count; ++index) {
        desc->final_positions[index * 4] = fluid->host_positions[index].x;
        desc->final_positions[index * 4 + 1] = fluid->host_positions[index].y;
        desc->final_positions[index * 4 + 2] = fluid->host_positions[index].z;
        desc->final_positions[index * 4 + 3] = fluid->host_positions[index].w;
    }
    report->step_mean_microseconds = step_total / desc->frames;
    report->readback_mean_microseconds = readback_total / desc->frames;
    report->gpu_available = 1;
    destroy_gpu_fluid(fluid);
    return kOk;
}

// Plan 24: the persistent fluid of the presentation lane.
std::int32_t ne_physx_fluid_create(
    const NePhysXFluidDesc* desc,
    void** output,
    std::uint32_t* reason) noexcept {
    if (desc == nullptr || output == nullptr || reason == nullptr || desc->max_particles == 0) {
        return kInvalidArgument;
    }
    *output = nullptr;
    *reason = 0;
    const float spacing = from_bits(desc->spacing_bits);
    const float timestep = from_bits(desc->timestep_bits);
    float box_min[3];
    float box_max[3];
    float seed_min[3];
    float seed_max[3];
    for (int axis = 0; axis < 3; ++axis) {
        box_min[axis] = from_bits(desc->box_min_bits[axis]);
        box_max[axis] = from_bits(desc->box_max_bits[axis]);
        seed_min[axis] = from_bits(desc->seed_min_bits[axis]);
        seed_max[axis] = from_bits(desc->seed_max_bits[axis]);
        if (!physx::PxIsFinite(box_min[axis]) || !physx::PxIsFinite(box_max[axis])
            || !physx::PxIsFinite(seed_min[axis]) || !physx::PxIsFinite(seed_max[axis])
            || box_min[axis] >= box_max[axis]) {
            return kInvalidArgument;
        }
    }
    if (!finite_positive(spacing) || !finite_positive(timestep)) {
        return kInvalidArgument;
    }
    const float rest_offset = 0.5F * spacing / 0.6F;
    const float fluid_rest_offset = rest_offset * 0.6F;
    const float particle_mass = 1000.0F * 1.333F * 3.14159F * fluid_rest_offset
        * fluid_rest_offset * fluid_rest_offset;
    auto* seed = new (std::nothrow) physx::PxVec4[desc->max_particles];
    if (seed == nullptr) {
        return kOutOfMemory;
    }
    std::uint32_t count = 0;
    for (float y = seed_min[1] + 0.5F * spacing; y < seed_max[1] && count < desc->max_particles;
         y += spacing) {
        for (float z = seed_min[2] + 0.5F * spacing; z < seed_max[2] && count < desc->max_particles;
             z += spacing) {
            for (float x = seed_min[0] + 0.5F * spacing;
                 x < seed_max[0] && count < desc->max_particles;
                 x += spacing) {
                seed[count++] = physx::PxVec4(x, y, z, 1.0F / particle_mass);
            }
        }
    }
    GpuFluid* fluid = create_gpu_fluid(
        desc->gpu_library_path,
        spacing,
        box_min,
        box_max,
        timestep,
        seed,
        count,
        desc->max_particles,
        reason);
    delete[] seed;
    if (fluid == nullptr) {
        return kInternalFailure;
    }
    *output = fluid;
    return kOk;
}

std::int32_t ne_physx_fluid_step(void* opaque_fluid) noexcept {
    auto* fluid = static_cast<GpuFluid*>(opaque_fluid);
    if (fluid == nullptr) {
        return kInvalidArgument;
    }
    return gpu_fluid_step(*fluid) ? kOk : kInternalFailure;
}

std::int32_t ne_physx_fluid_read(
    void* opaque_fluid,
    float* positions,
    float* velocities,
    std::uint32_t capacity,
    std::uint32_t* count) noexcept {
    auto* fluid = static_cast<GpuFluid*>(opaque_fluid);
    if (fluid == nullptr || positions == nullptr || velocities == nullptr || count == nullptr) {
        return kInvalidArgument;
    }
    if (capacity < fluid->count) {
        return kCapacityExceeded;
    }
    if (!gpu_fluid_readback(*fluid, true)) {
        return kInternalFailure;
    }
    for (std::uint32_t index = 0; index < fluid->count; ++index) {
        positions[index * 3] = fluid->host_positions[index].x;
        positions[index * 3 + 1] = fluid->host_positions[index].y;
        positions[index * 3 + 2] = fluid->host_positions[index].z;
        velocities[index * 3] = fluid->host_velocities[index].x;
        velocities[index * 3 + 1] = fluid->host_velocities[index].y;
        velocities[index * 3 + 2] = fluid->host_velocities[index].z;
    }
    *count = fluid->count;
    return kOk;
}

std::int32_t ne_physx_fluid_set(
    void* opaque_fluid,
    const float* positions,
    const float* velocities,
    std::uint32_t count) noexcept {
    auto* fluid = static_cast<GpuFluid*>(opaque_fluid);
    if (fluid == nullptr || (count > 0 && (positions == nullptr || velocities == nullptr))) {
        return kInvalidArgument;
    }
    if (count > fluid->capacity) {
        return kCapacityExceeded;
    }
    return gpu_fluid_set(*fluid, positions, velocities, count) ? kOk : kInternalFailure;
}

void ne_physx_fluid_destroy(void* opaque_fluid) noexcept {
    destroy_gpu_fluid(static_cast<GpuFluid*>(opaque_fluid));
}

}  // extern "C"
