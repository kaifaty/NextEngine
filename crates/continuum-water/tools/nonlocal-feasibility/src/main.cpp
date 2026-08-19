#include "cuda_baseline.hpp"
#include "oracle.hpp"
#include "profiles.hpp"

#include <exception>
#include <iostream>
#include <stdexcept>
#include <string>

namespace {

void print_usage() {
    std::cerr << "usage: nonlocal-feasibility --describe-profile <profile-id>\n"
              << "       nonlocal-feasibility --cpu-self-test\n"
              << "       nonlocal-feasibility --cpu-gather-self-test\n"
              << "       nonlocal-feasibility --self-test\n"
              << "       nonlocal-feasibility --self-test --accumulation <identity>\n"
              << "       nonlocal-feasibility --check <profile-id> --iterations <count>\n"
              << "       nonlocal-feasibility --check <profile-id> --iterations <count> "
                 "--accumulation <identity>\n"
              << "       nonlocal-feasibility --repeatability <profile-id> --iterations <count> "
                 "--runs <count> --accumulation <identity>\n"
              << "       nonlocal-feasibility --benchmark <profile-id> --warmup <count> "
                 "--runs <count>\n"
              << "       nonlocal-feasibility --benchmark <profile-id> --warmup <count> "
                 "--runs <count> --accumulation <identity>\n";
}

int bounded_integer(const char* text, const char* name) {
    std::size_t consumed = 0;
    const std::string value(text);
    const int parsed = std::stoi(value, &consumed);
    if (consumed != value.size()) {
        throw std::invalid_argument(std::string("invalid ") + name + ": " + value);
    }
    return parsed;
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
        if (argc == 2 && std::string(argv[1]) == "--cpu-self-test") {
            const auto reports = nextengine::nonlocal::run_cpu_self_test();
            bool passed = true;
            for (const auto& report : reports) {
                passed = passed && report.passed;
            }
            std::cout << nextengine::nonlocal::cpu_self_test_json(reports) << '\n';
            return passed ? 0 : 1;
        }
        if (argc == 2 && std::string(argv[1]) == "--cpu-gather-self-test") {
            const auto report = nextengine::nonlocal::run_cpu_gather_self_test();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 2 && std::string(argv[1]) == "--self-test") {
            const auto report = nextengine::nonlocal::run_cuda_self_test();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 4 && std::string(argv[1]) == "--self-test"
            && std::string(argv[2]) == "--accumulation") {
            const auto report = nextengine::nonlocal::run_cuda_self_test(
                nextengine::nonlocal::parse_accumulation_identity(argv[3]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 5 && std::string(argv[1]) == "--check"
            && std::string(argv[3]) == "--iterations") {
            const auto report = nextengine::nonlocal::run_cuda_check(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 7 && std::string(argv[1]) == "--check"
            && std::string(argv[3]) == "--iterations"
            && std::string(argv[5]) == "--accumulation") {
            const auto report = nextengine::nonlocal::run_cuda_check(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[6]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 9 && std::string(argv[1]) == "--repeatability"
            && std::string(argv[3]) == "--iterations"
            && std::string(argv[5]) == "--runs"
            && std::string(argv[7]) == "--accumulation") {
            const auto report = nextengine::nonlocal::run_cuda_repeatability(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"),
                bounded_integer(argv[6], "run count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[8]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 7 && std::string(argv[1]) == "--benchmark"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs") {
            const auto report = nextengine::nonlocal::run_cuda_benchmark(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 9 && std::string(argv[1]) == "--benchmark"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs"
            && std::string(argv[7]) == "--accumulation") {
            const auto report = nextengine::nonlocal::run_cuda_benchmark(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[8]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        print_usage();
        return 2;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-feasibility: " << error.what() << '\n';
        return 1;
    }
}
