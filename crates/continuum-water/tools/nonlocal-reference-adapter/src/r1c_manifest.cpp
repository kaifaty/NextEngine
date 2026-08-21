#include "r1c_manifest.hpp"

#include "sha256.hpp"

#include <array>
#include <cstddef>
#include <cstdint>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>

namespace nextengine::nonlocal_reference {
namespace {

constexpr std::string_view SCHEMA =
    "nextengine.nonlocal.nsr3b4dr1c-manifest-preflight.v1";
constexpr std::string_view CONTRACT_IDENTITY =
    "865570e18864ec55cdbbbbad8b9cfa3f200a085087ecf272366c342144488927";
constexpr std::string_view PARENT_IDENTITY =
    "ade621f889a08fd26ca713592625c50316b893af8c4f1797d25f4e4d4c96b86a";
constexpr std::string_view R1B_IDENTITY =
    "c65346ec7b215a7a173eabfdd6c91e20869d4a0679b9d014a1e897369cb71f84";
constexpr std::string_view UPSTREAM_COMMIT =
    "eccce86155776f6ac52d5080b1f720a52bf29450";
constexpr std::string_view PATCH_SHA256 =
    "e89cf9befc2a08a9c15bd6b290a97b3f6815da1ea699508c41c15645f86c33bc";

constexpr std::string_view HYDRO_MANIFEST = R"(B4DR1C_SCENARIO_V1_BEGIN
scenario_id=CW-HYDRO-001
kind=hydrostatic-cube
box_um=0,0,0;1000000,1000000,1000000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=7d4e661d08de08b18d43a76342329b51f6ae98bca9baee3d850e0f403eae5606
boundary=two-layer-outer-complement
boundary_count=5824
boundary_root=25de85b5eeec041c12bbb5de10e00b8374457b4cf09cb61d99dfc4d5511d8d62
steps=24
outputs=0..24/every=1
B4DR1C_SCENARIO_V1_END
)";

constexpr std::string_view DAM_MANIFEST = R"(B4DR1C_SCENARIO_V1_BEGIN
scenario_id=CW-DAMBREAK-001
kind=dam-break
box_um=0,0,0;4000000,1000000,1000000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76
boundary=two-layer-outer-complement
boundary_count=16384
boundary_root=1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d
steps=24
outputs=0..24/every=1
B4DR1C_SCENARIO_V1_END
)";

constexpr std::string_view ORIFICE_MANIFEST = R"(B4DR1C_SCENARIO_V1_BEGIN
scenario_id=CW-ORIFICE-001
kind=orifice-release
box_um=0,0,0;2000000,1000000,1000000
wall_um=x=1000000;opening_y=200000..400000;opening_z=400000..600000;radius=25000
fluid=20,15,20;first=(25000,25000,25000);velocity=(0,0,0);id=iy-iz-ix
fluid_root=21307ab2d1655ab5da33f3b5d4e887152601ed531679723b8452023df1f48425
boundary=source-chamber-two-layer-minus-safe-opening
boundary_count=5792
boundary_root=5d23bd8c407c1b1d6bb86fc6dda2250c36db4cde1227d9f48fc6239c728a2cb3
steps=24
outputs=0..24/every=1
B4DR1C_SCENARIO_V1_END
)";

struct Scenario {
    std::string_view id;
    std::string_view manifest;
    std::string_view manifest_root;
    std::string_view fluid_root;
    std::string_view boundary_root;
    int boundary_nx;
    std::size_t boundary_count;
    bool orifice;
};

constexpr std::array<Scenario, 3> SCENARIOS = {{
    {
        "CW-HYDRO-001",
        HYDRO_MANIFEST,
        "88d7b5ee86876d05b8cdef4f5c308fa61ffeb9d559374674db7dda2757e611b7",
        "7d4e661d08de08b18d43a76342329b51f6ae98bca9baee3d850e0f403eae5606",
        "25de85b5eeec041c12bbb5de10e00b8374457b4cf09cb61d99dfc4d5511d8d62",
        20,
        5'824,
        false,
    },
    {
        "CW-DAMBREAK-001",
        DAM_MANIFEST,
        "4c126bd7af7cac871c72d2ab4bfd47c902c105650005f35cdaefae4a3e10cc05",
        "9c12e445666c7b0eada3e6e2c258c733323e4eb8ca6474a6f3d5b863f1566e76",
        "1cf0fd172dcb321e995f372119ea956d1e376b8a409b804bc07e31a729aa830d",
        80,
        16'384,
        false,
    },
    {
        "CW-ORIFICE-001",
        ORIFICE_MANIFEST,
        "5bb0a197f40ec5e5d3f8536c5db7840da4228978fcb1c788b9de7d233482d42e",
        "21307ab2d1655ab5da33f3b5d4e887152601ed531679723b8452023df1f48425",
        "5d23bd8c407c1b1d6bb86fc6dda2250c36db4cde1227d9f48fc6239c728a2cb3",
        20,
        5'792,
        true,
    },
}};

std::string scenario_root(std::string_view manifest) {
    constexpr char DOMAIN[] = "nextengine.nonlocal.nsr3b4dr1c-scenario.v1";
    std::string projection(DOMAIN, sizeof(DOMAIN));
    projection.append(manifest);
    return nextengine::nonlocal::sha256_hex(projection);
}

std::string fluid_projection(std::string_view scenario_id, bool swap_first_ids) {
    std::ostringstream output;
    output << "B4DR1C_FLUID_V1_BEGIN\n"
           << "scenario=" << scenario_id << '\n'
           << "count=6000\n";
    for (std::uint32_t iy = 0; iy < 15; ++iy) {
        for (std::uint32_t iz = 0; iz < 20; ++iz) {
            for (std::uint32_t ix = 0; ix < 20; ++ix) {
                const std::uint32_t id = ((iy * 20U) + iz) * 20U + ix;
                std::uint32_t projected_id = id;
                if (swap_first_ids && id < 2U) {
                    projected_id = 1U - id;
                }
                output << projected_id << '=' << (25'000U + (50'000U * ix)) << ','
                       << (25'000U + (50'000U * iy)) << ','
                       << (25'000U + (50'000U * iz)) << ";0,0,0\n";
            }
        }
    }
    output << "B4DR1C_FLUID_V1_END\n";
    return output.str();
}

bool is_orifice_omission(int ix, int iy, int iz) {
    return (ix == 20 || ix == 21) && iy >= 4 && iy <= 7 && iz >= 8 && iz <= 11;
}

std::string boundary_projection(
    const Scenario &scenario,
    bool mutate_first_coordinate,
    std::size_t &count) {
    std::ostringstream output;
    output << "B4DR1C_BOUNDARY_V1_BEGIN\n"
           << "scenario=" << scenario.id << '\n';
    count = 0;
    for (int ix = -2; ix < scenario.boundary_nx + 2; ++ix) {
        for (int iy = -2; iy < 22; ++iy) {
            for (int iz = -2; iz < 22; ++iz) {
                const bool interior = ix >= 0 && ix < scenario.boundary_nx && iy >= 0
                    && iy < 20 && iz >= 0 && iz < 20;
                if (interior || (scenario.orifice && is_orifice_omission(ix, iy, iz))) {
                    continue;
                }
                std::int64_t x = 25'000 + (50'000 * static_cast<std::int64_t>(ix));
                if (mutate_first_coordinate && count == 0U) {
                    ++x;
                }
                const std::int64_t y =
                    25'000 + (50'000 * static_cast<std::int64_t>(iy));
                const std::int64_t z =
                    25'000 + (50'000 * static_cast<std::int64_t>(iz));
                output << count << '=' << x << ',' << y << ',' << z << '\n';
                ++count;
            }
        }
    }
    output << "count=" << count << "\nB4DR1C_BOUNDARY_V1_END\n";
    return output.str();
}

std::string rejected_report(std::string_view reason) {
    std::ostringstream output;
    output << "schema=" << SCHEMA << '\n'
           << "contract_identity=" << CONTRACT_IDENTITY << '\n'
           << "status=REJECTED\n"
           << "reason=" << reason << '\n'
           << "simulation_created=false\n"
           << "trajectory_started=false\n";
    return output.str();
}

} // namespace

AdapterRun run_r1c_manifest_preflight(bool force_manifest_mismatch) {
    const std::string process_failure = process_preflight_failure();
    if (!process_failure.empty()) {
        return {false, rejected_report(process_failure)};
    }

    try {
        std::ostringstream scenario_report;
        for (const Scenario &scenario : SCENARIOS) {
            const std::string actual_manifest_root = scenario_root(scenario.manifest);
            if (actual_manifest_root != scenario.manifest_root) {
                throw std::runtime_error(std::string(scenario.id) + ":MANIFEST_ROOT");
            }
            const std::string fluid = fluid_projection(scenario.id, false);
            const std::string fluid_root = nextengine::nonlocal::sha256_hex(fluid);
            if (fluid_root != scenario.fluid_root) {
                throw std::runtime_error(std::string(scenario.id) + ":FLUID_ROOT");
            }
            std::size_t boundary_count = 0;
            const std::string boundary = boundary_projection(scenario, false, boundary_count);
            const std::string boundary_root = nextengine::nonlocal::sha256_hex(boundary);
            if (boundary_count != scenario.boundary_count) {
                throw std::runtime_error(std::string(scenario.id) + ":BOUNDARY_COUNT");
            }
            if (boundary_root != scenario.boundary_root) {
                throw std::runtime_error(std::string(scenario.id) + ":BOUNDARY_ROOT");
            }
            scenario_report << "scenario." << scenario.id << "=PASS;manifest_bytes="
                            << scenario.manifest.size() << ";manifest_root="
                            << actual_manifest_root << ";fluid_count=6000;fluid_root="
                            << fluid_root << ";boundary_count=" << boundary_count
                            << ";boundary_root=" << boundary_root << '\n';
        }

        const std::string fluid_base = fluid_projection(SCENARIOS[0].id, false);
        const std::string fluid_mutated = fluid_projection(SCENARIOS[0].id, true);
        const std::string fluid_mutation_root =
            nextengine::nonlocal::sha256_hex(fluid_mutated);
        if (nextengine::nonlocal::sha256_hex(fluid_base) == fluid_mutation_root) {
            throw std::runtime_error("FLUID_ID_MUTATION_COLLISION");
        }
        std::size_t boundary_base_count = 0;
        std::size_t boundary_mutated_count = 0;
        const std::string boundary_base =
            boundary_projection(SCENARIOS[0], false, boundary_base_count);
        const std::string boundary_mutated =
            boundary_projection(SCENARIOS[0], true, boundary_mutated_count);
        const std::string boundary_mutation_root =
            nextengine::nonlocal::sha256_hex(boundary_mutated);
        if (boundary_base_count != boundary_mutated_count
            || nextengine::nonlocal::sha256_hex(boundary_base) == boundary_mutation_root) {
            throw std::runtime_error("BOUNDARY_COORDINATE_MUTATION_COLLISION");
        }
        if (force_manifest_mismatch) {
            return {false, rejected_report("FORCED_MANIFEST_ROOT_MISMATCH")};
        }

        std::ostringstream output;
        output << "schema=" << SCHEMA << '\n'
               << "contract_identity=" << CONTRACT_IDENTITY << '\n'
               << "parent_identity=" << PARENT_IDENTITY << '\n'
               << "r1b_identity=" << R1B_IDENTITY << '\n'
               << "upstream_commit=" << UPSTREAM_COMMIT << '\n'
               << "patch_sha256=" << PATCH_SHA256 << '\n'
               << "status=PASS\n"
               << "simulation_created=false\n"
               << "trajectory_started=false\n"
               << scenario_report.str()
               << "fluid_id_mutation_root=" << fluid_mutation_root << '\n'
               << "boundary_coordinate_mutation_root=" << boundary_mutation_root << '\n'
               << "manifest_mismatch_rejected=true\n"
               << "trajectory_authorized=true\n"
               << "r1d_authorized=false\n"
               << "b4e_authorized=false\n";
        return {true, output.str()};
    } catch (const std::exception &error) {
        return {false, rejected_report(error.what())};
    }
}

} // namespace nextengine::nonlocal_reference
