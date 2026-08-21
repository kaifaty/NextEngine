#include "reader.hpp"

#include <iostream>
#include <string_view>

int main(int argc, char **argv) {
    using namespace nextengine::nonlocal_reference_reader;
    ReaderRun run;
    if (argc == 2 && std::string_view(argv[1]) == "--profile-self-test") {
        run = run_profile_self_test();
    } else if (argc == 3 && std::string_view(argv[1]) == "--attest") {
        run = run_attestation(argv[2]);
    } else {
        run = reject_unknown_argument();
    }
    std::cout << run.report;
    return run.passed ? 0 : 1;
}
