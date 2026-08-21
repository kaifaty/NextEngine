#include "contact_adapter.hpp"
#include "r1c_manifest.hpp"
#include "r1c_trajectory.hpp"

#include "SPlisHSPlasH/Common.h"
#include "Utilities/Counting.h"
#include "Utilities/Timing.h"

#include <iostream>
#include <string_view>

// SPlisHSPlasH intentionally leaves these process-wide utility stores to each
// embedding executable.  Keep their single definitions at the executable
// boundary, matching the upstream standalone tools and SimulatorBase.
INIT_LOGGING
INIT_TIMING
INIT_COUNTING

int main(int argc, char **argv) {
    using nextengine::nonlocal_reference::PreflightMutation;

    PreflightMutation mutation = PreflightMutation::None;
    if (argc == 4 && std::string_view(argv[1]) == "--r1c4-trajectory") {
        const auto run = nextengine::nonlocal_reference::run_r1c4_trajectory(
            argv[2],
            argv[3]);
        std::cout << run.report;
        return run.passed ? 0 : 1;
    }
    if (argc == 4 && std::string_view(argv[1]) == "--r1c3-pressure-cap") {
        const auto run = nextengine::nonlocal_reference::run_r1c_pressure_cap_sweep_point(
            argv[2],
            argv[3]);
        std::cout << run.report;
        return run.passed ? 0 : 1;
    }
    if (argc == 4 && std::string_view(argv[1]) == "--r1c-diagnose-trajectory") {
        const auto run = nextengine::nonlocal_reference::run_r1c_trajectory_diagnostic(
            argv[2],
            argv[3]);
        std::cout << run.report;
        return run.passed ? 0 : 1;
    }
    if (argc == 4 && std::string_view(argv[1]) == "--r1c-trajectory") {
        const auto run = nextengine::nonlocal_reference::run_r1c_trajectory(
            argv[2],
            argv[3]);
        std::cout << run.report;
        return run.passed ? 0 : 1;
    }
    if (argc == 2) {
        const std::string_view argument(argv[1]);
        if (argument == "--negative-rounding") {
            mutation = PreflightMutation::RoundDown;
        } else if (argument == "--negative-ftz") {
            mutation = PreflightMutation::FtzOn;
        } else if (argument == "--r1c-manifest-preflight") {
            const auto run =
                nextengine::nonlocal_reference::run_r1c_manifest_preflight(false);
            std::cout << run.report;
            return run.passed ? 0 : 1;
        } else if (argument == "--r1c-negative-manifest-mismatch") {
            const auto run =
                nextengine::nonlocal_reference::run_r1c_manifest_preflight(true);
            std::cout << run.report;
            return run.passed ? 0 : 1;
        } else {
            const auto run = nextengine::nonlocal_reference::reject_unknown_argument();
            std::cout << run.report;
            return run.passed ? 0 : 1;
        }
    } else if (argc != 1) {
        const auto run = nextengine::nonlocal_reference::reject_unknown_argument();
        std::cout << run.report;
        return run.passed ? 0 : 1;
    }

    const auto run = nextengine::nonlocal_reference::run_contact_adapter(mutation);
    std::cout << run.report;
    return run.passed ? 0 : 1;
}
