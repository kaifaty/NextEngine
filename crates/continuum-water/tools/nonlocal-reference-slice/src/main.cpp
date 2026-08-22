#include "slice.hpp"

#include <iostream>
#include <string_view>

int main(int argc, char** argv) {
    using namespace nextengine::nonlocal_reference_slice;
    SliceRun run;
    if (argc == 3
        && std::string_view(argv[1]) == "--first-output") {
        run = run_first_output(argv[2]);
    } else {
        run = reject_unknown_argument();
    }
    std::cout << run.report;
    return run.passed ? 0 : 1;
}

