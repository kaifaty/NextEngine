#include "oracle.hpp"
#include "profiles.hpp"

#include <exception>
#include <iostream>
#include <string>

namespace {

void print_usage() {
    std::cerr << "usage: nonlocal-feasibility --describe-profile <profile-id>\n"
              << "       nonlocal-feasibility --self-test\n";
}
} // namespace

int main(int argc, char** argv) {
    try {
        if (argc == 3 && std::string(argv[1]) == "--describe-profile") {
            std::cout << nextengine::nonlocal::described_profile_json(
                             nextengine::nonlocal::find_profile(argv[2]))
                      << '\n';
            return 0;
        }
        if (argc == 2 && std::string(argv[1]) == "--self-test") {
            const auto reports = nextengine::nonlocal::run_cpu_self_test();
            bool passed = true;
            for (const auto& report : reports) {
                passed = passed && report.passed;
            }
            std::cout << nextengine::nonlocal::cpu_self_test_json(reports) << '\n';
            return passed ? 0 : 1;
        }
        print_usage();
        return 2;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-feasibility: " << error.what() << '\n';
        return 1;
    }
}
