#include "boundary_discriminator.hpp"
#include "cuda_baseline.hpp"
#include "oracle.hpp"
#include "profiles.hpp"
#include "tiny_corpus.hpp"

#include <exception>
#include <csignal>
#include <iostream>
#include <stdexcept>
#include <string>

namespace {

void print_usage() {
    std::cerr << "usage: nonlocal-feasibility --describe-profile <profile-id>\n"
              << "       nonlocal-feasibility --production-profile-audit\n"
              << "       nonlocal-feasibility --cpu-self-test\n"
              << "       nonlocal-feasibility --cpu-gather-self-test\n"
              << "       nonlocal-feasibility --cpu-scale-law-self-test\n"
              << "       nonlocal-feasibility --cpu-boundary-discriminator\n"
              << "       nonlocal-feasibility --cpu-tiny-physical-corpus\n"
              << "       nonlocal-feasibility --cpu-hydro-remediation\n"
              << "       nonlocal-feasibility --cpu-h3-profile-corpus\n"
              << "       nonlocal-feasibility --self-test\n"
              << "       nonlocal-feasibility --self-test --accumulation <identity>\n"
              << "       nonlocal-feasibility --self-test --accumulation <identity> "
                 "--handoff <identity>\n"
              << "       nonlocal-feasibility --self-test --accumulation <identity> "
                 "--handoff <identity> --term-kernels <identity>\n"
              << "       nonlocal-feasibility --self-test --accumulation <identity> "
                 "--handoff <identity> --term-kernels <identity> --storage <identity>\n"
              << "       nonlocal-feasibility --check <profile-id> --iterations <count>\n"
              << "       nonlocal-feasibility --check <profile-id> --iterations <count> "
                 "--accumulation <identity>\n"
              << "       nonlocal-feasibility --check <profile-id> --iterations <count> "
                 "--accumulation <identity> --handoff <identity>\n"
              << "       nonlocal-feasibility --check <profile-id> --iterations <count> "
                 "--accumulation <identity> --handoff <identity> --term-kernels <identity>\n"
              << "       nonlocal-feasibility --check <profile-id> --iterations <count> "
                 "--accumulation <identity> --handoff <identity> --term-kernels <identity> "
                 "--storage <identity>\n"
              << "       nonlocal-feasibility --repeatability <profile-id> --iterations <count> "
                 "--runs <count> --accumulation <identity>\n"
              << "       nonlocal-feasibility --repeatability <profile-id> --iterations <count> "
                 "--runs <count> --accumulation <identity> --handoff <identity>\n"
              << "       nonlocal-feasibility --repeatability <profile-id> --iterations <count> "
                 "--runs <count> --accumulation <identity> --handoff <identity> "
                 "--term-kernels <identity>\n"
              << "       nonlocal-feasibility --repeatability <profile-id> --iterations <count> "
                 "--runs <count> --accumulation <identity> --handoff <identity> "
                 "--term-kernels <identity> --storage <identity>\n"
              << "       nonlocal-feasibility --benchmark <profile-id> --warmup <count> "
                 "--runs <count>\n"
              << "       nonlocal-feasibility --benchmark <profile-id> --warmup <count> "
                 "--runs <count> --accumulation <identity>\n"
              << "       nonlocal-feasibility --benchmark <profile-id> --warmup <count> "
                 "--runs <count> --accumulation <identity> --handoff <identity>\n"
              << "       nonlocal-feasibility --benchmark <profile-id> --warmup <count> "
                 "--runs <count> --accumulation <identity> --handoff <identity> "
                 "--term-kernels <identity>\n"
              << "       nonlocal-feasibility --benchmark <profile-id> --warmup <count> "
                 "--runs <count> --accumulation <identity> --handoff <identity> "
                 "--term-kernels <identity> --storage <identity>\n"
              << "       nonlocal-feasibility --np0-baseline <profile-id> --warmup <count> "
                 "--runs <count>\n"
              << "       nonlocal-feasibility --p1-check <profile-id> --iterations <count>\n"
              << "       nonlocal-feasibility --p1-tournament <profile-id> --warmup 32 "
                 "--runs 96\n"
              << "       nonlocal-feasibility --p2-check <profile-id> --iterations <count>\n"
              << "       nonlocal-feasibility --p2-tournament <profile-id> --warmup 32 "
                 "--runs 96\n"
              << "       nonlocal-feasibility --p3-check <profile-id> --iterations <count>\n"
              << "       nonlocal-feasibility --p3-tournament <profile-id> --warmup 32 "
                 "--runs 96\n"
              << "       nonlocal-feasibility --p4-check <profile-id> --iterations <count>\n"
              << "       nonlocal-feasibility --p4-tournament <profile-id> --warmup 32 "
                 "--runs 96\n"
              << "       nonlocal-feasibility --p2-decision <profile-id> --warmup 64 "
                 "--runs 512\n"
              << "       nonlocal-feasibility --game-quality-smoke\n"
              << "       nonlocal-feasibility --game-visual-corpus\n"
              << "       nonlocal-feasibility --game-visual-corpus --frames <prefix>\n"
              << "       nonlocal-feasibility --game-surface-prototype\n"
              << "       nonlocal-feasibility --game-surface-prototype --frames <prefix>\n"
              << "       nonlocal-feasibility --game-surface-stream --lane <4k|16k|48k|48k-dam> "
                 "[--steps 960] [--every 4] [--cycles 1] [--workers 3] "
                 "[--extractor cpu|gpu|verify] [--surface-model sphere|closing] [--dump-particles <prefix>] [--boundary-layers 0|1|2] [--boundary-support full|density] [--boundary-lid 1|0]\n"
              << "       nonlocal-feasibility --layout-tournament <profile-id> --warmup 32 "
                 "--runs 96\n"
              << "       nonlocal-feasibility --locality-tournament <profile-id> --warmup 32 "
                 "--runs 96\n"
              << "       nonlocal-feasibility --retained-tournament <profile-id> --warmup 32 "
                 "--runs 96\n";
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
        if (argc == 2 && std::string(argv[1]) == "--production-profile-audit") {
            std::cout << nextengine::nonlocal::production_profile_audit_json() << '\n';
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
        if (argc == 2 && std::string(argv[1]) == "--cpu-scale-law-self-test") {
            const auto report = nextengine::nonlocal::run_cpu_scale_law_self_test();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 2 && std::string(argv[1]) == "--cpu-boundary-discriminator") {
            const auto report = nextengine::nonlocal::run_cpu_boundary_discriminator();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 2 && std::string(argv[1]) == "--cpu-tiny-physical-corpus") {
            const auto report = nextengine::nonlocal::run_cpu_tiny_physical_corpus();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 2 && std::string(argv[1]) == "--cpu-hydro-remediation") {
            const auto report = nextengine::nonlocal::run_cpu_hydro_remediation();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 2 && std::string(argv[1]) == "--cpu-h3-profile-corpus") {
            const auto report = nextengine::nonlocal::run_cpu_h3_profile_corpus();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 2 && std::string(argv[1]) == "--self-test") {
            const auto report = nextengine::nonlocal::run_cuda_self_test();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 2 && std::string(argv[1]) == "--game-quality-smoke") {
            const auto report = nextengine::nonlocal::run_cuda_game_quality_smoke();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 2 && std::string(argv[1]) == "--game-visual-corpus") {
            const auto report = nextengine::nonlocal::run_cuda_game_visual_corpus();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 4 && std::string(argv[1]) == "--game-visual-corpus"
            && std::string(argv[2]) == "--frames") {
            const auto report =
                nextengine::nonlocal::run_cuda_game_visual_corpus(argv[3]);
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc >= 4 && std::string(argv[1]) == "--game-surface-stream") {
            std::string lane;
            int steps = 960;
            int every = 4;
            int cycles = 1;
            int workers = 3;
            std::string extractor = "cpu";
            std::string surface_model = "sphere";
            std::string particle_dump;
            int boundary_layers = 0;
            std::string boundary_support = "full";
            bool boundary_lid = true;
            for (int index = 2; index + 1 < argc; index += 2) {
                const std::string key = argv[index];
                const std::string value = argv[index + 1];
                if (key == "--lane") {
                    lane = value;
                } else if (key == "--steps") {
                    steps = std::stoi(value);
                } else if (key == "--every") {
                    every = std::stoi(value);
                } else if (key == "--cycles") {
                    cycles = std::stoi(value);
                } else if (key == "--workers") {
                    workers = std::stoi(value);
                } else if (key == "--extractor") {
                    extractor = value;
                } else if (key == "--surface-model") {
                    surface_model = value;
                } else if (key == "--dump-particles") {
                    particle_dump = value;
                } else if (key == "--boundary-layers") {
                    boundary_layers = std::stoi(value);
                } else if (key == "--boundary-support") {
                    boundary_support = value;
                } else if (key == "--boundary-lid") {
                    boundary_lid = value == "1" || value == "true";
                } else {
                    print_usage();
                    return 2;
                }
            }
            if (argc % 2 != 0 || lane.empty()) {
                print_usage();
                return 2;
            }
            // A consumer that stops reading closes the pipe; report that as
            // a bounded `stream_closed` frame failure instead of dying.
            std::signal(SIGPIPE, SIG_IGN);
            const auto report = nextengine::nonlocal::run_cuda_game_surface_stream(
                lane, steps, every, cycles, workers, extractor, surface_model, std::cout,
                particle_dump, boundary_layers, boundary_support, boundary_lid);
            std::cerr << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 2 && std::string(argv[1]) == "--game-surface-prototype") {
            const auto report =
                nextengine::nonlocal::run_cuda_game_surface_prototype();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 4 && std::string(argv[1]) == "--game-surface-prototype"
            && std::string(argv[2]) == "--frames") {
            const auto report =
                nextengine::nonlocal::run_cuda_game_surface_prototype(argv[3]);
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
        if (argc == 6 && std::string(argv[1]) == "--self-test"
            && std::string(argv[2]) == "--accumulation"
            && std::string(argv[4]) == "--handoff") {
            const auto report = nextengine::nonlocal::run_cuda_self_test(
                nextengine::nonlocal::parse_accumulation_identity(argv[3]),
                nextengine::nonlocal::parse_handoff_identity(argv[5]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 8 && std::string(argv[1]) == "--self-test"
            && std::string(argv[2]) == "--accumulation"
            && std::string(argv[4]) == "--handoff"
            && std::string(argv[6]) == "--term-kernels") {
            const auto report = nextengine::nonlocal::run_cuda_self_test(
                nextengine::nonlocal::parse_accumulation_identity(argv[3]),
                nextengine::nonlocal::parse_handoff_identity(argv[5]),
                nextengine::nonlocal::parse_term_kernel_identity(argv[7]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 10 && std::string(argv[1]) == "--self-test"
            && std::string(argv[2]) == "--accumulation"
            && std::string(argv[4]) == "--handoff"
            && std::string(argv[6]) == "--term-kernels"
            && std::string(argv[8]) == "--storage") {
            const auto report = nextengine::nonlocal::run_cuda_self_test(
                nextengine::nonlocal::parse_accumulation_identity(argv[3]),
                nextengine::nonlocal::parse_handoff_identity(argv[5]),
                nextengine::nonlocal::parse_term_kernel_identity(argv[7]),
                nextengine::nonlocal::parse_storage_identity(argv[9]));
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
        if (argc == 9 && std::string(argv[1]) == "--check"
            && std::string(argv[3]) == "--iterations"
            && std::string(argv[5]) == "--accumulation"
            && std::string(argv[7]) == "--handoff") {
            const auto report = nextengine::nonlocal::run_cuda_check(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[6]),
                nextengine::nonlocal::parse_handoff_identity(argv[8]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 11 && std::string(argv[1]) == "--check"
            && std::string(argv[3]) == "--iterations"
            && std::string(argv[5]) == "--accumulation"
            && std::string(argv[7]) == "--handoff"
            && std::string(argv[9]) == "--term-kernels") {
            const auto report = nextengine::nonlocal::run_cuda_check(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[6]),
                nextengine::nonlocal::parse_handoff_identity(argv[8]),
                nextengine::nonlocal::parse_term_kernel_identity(argv[10]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 13 && std::string(argv[1]) == "--check"
            && std::string(argv[3]) == "--iterations"
            && std::string(argv[5]) == "--accumulation"
            && std::string(argv[7]) == "--handoff"
            && std::string(argv[9]) == "--term-kernels"
            && std::string(argv[11]) == "--storage") {
            const auto report = nextengine::nonlocal::run_cuda_check(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[6]),
                nextengine::nonlocal::parse_handoff_identity(argv[8]),
                nextengine::nonlocal::parse_term_kernel_identity(argv[10]),
                nextengine::nonlocal::parse_storage_identity(argv[12]));
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
        if (argc == 11 && std::string(argv[1]) == "--repeatability"
            && std::string(argv[3]) == "--iterations"
            && std::string(argv[5]) == "--runs"
            && std::string(argv[7]) == "--accumulation"
            && std::string(argv[9]) == "--handoff") {
            const auto report = nextengine::nonlocal::run_cuda_repeatability(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"),
                bounded_integer(argv[6], "run count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[8]),
                nextengine::nonlocal::parse_handoff_identity(argv[10]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 13 && std::string(argv[1]) == "--repeatability"
            && std::string(argv[3]) == "--iterations"
            && std::string(argv[5]) == "--runs"
            && std::string(argv[7]) == "--accumulation"
            && std::string(argv[9]) == "--handoff"
            && std::string(argv[11]) == "--term-kernels") {
            const auto report = nextengine::nonlocal::run_cuda_repeatability(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"),
                bounded_integer(argv[6], "run count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[8]),
                nextengine::nonlocal::parse_handoff_identity(argv[10]),
                nextengine::nonlocal::parse_term_kernel_identity(argv[12]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 15 && std::string(argv[1]) == "--repeatability"
            && std::string(argv[3]) == "--iterations"
            && std::string(argv[5]) == "--runs"
            && std::string(argv[7]) == "--accumulation"
            && std::string(argv[9]) == "--handoff"
            && std::string(argv[11]) == "--term-kernels"
            && std::string(argv[13]) == "--storage") {
            const auto report = nextengine::nonlocal::run_cuda_repeatability(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"),
                bounded_integer(argv[6], "run count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[8]),
                nextengine::nonlocal::parse_handoff_identity(argv[10]),
                nextengine::nonlocal::parse_term_kernel_identity(argv[12]),
                nextengine::nonlocal::parse_storage_identity(argv[14]));
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
        if (argc == 11 && std::string(argv[1]) == "--benchmark"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs"
            && std::string(argv[7]) == "--accumulation"
            && std::string(argv[9]) == "--handoff") {
            const auto report = nextengine::nonlocal::run_cuda_benchmark(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[8]),
                nextengine::nonlocal::parse_handoff_identity(argv[10]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 13 && std::string(argv[1]) == "--benchmark"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs"
            && std::string(argv[7]) == "--accumulation"
            && std::string(argv[9]) == "--handoff"
            && std::string(argv[11]) == "--term-kernels") {
            const auto report = nextengine::nonlocal::run_cuda_benchmark(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[8]),
                nextengine::nonlocal::parse_handoff_identity(argv[10]),
                nextengine::nonlocal::parse_term_kernel_identity(argv[12]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 15 && std::string(argv[1]) == "--benchmark"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs"
            && std::string(argv[7]) == "--accumulation"
            && std::string(argv[9]) == "--handoff"
            && std::string(argv[11]) == "--term-kernels"
            && std::string(argv[13]) == "--storage") {
            const auto report = nextengine::nonlocal::run_cuda_benchmark(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"),
                nextengine::nonlocal::parse_accumulation_identity(argv[8]),
                nextengine::nonlocal::parse_handoff_identity(argv[10]),
                nextengine::nonlocal::parse_term_kernel_identity(argv[12]),
                nextengine::nonlocal::parse_storage_identity(argv[14]));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 7 && std::string(argv[1]) == "--np0-baseline"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs") {
            const auto report = nextengine::nonlocal::run_cuda_np0_baseline(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 5 && std::string(argv[1]) == "--p1-check"
            && std::string(argv[3]) == "--iterations") {
            const auto report = nextengine::nonlocal::run_cuda_p1_check(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 7 && std::string(argv[1]) == "--p1-tournament"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs") {
            const auto report = nextengine::nonlocal::run_cuda_p1_tournament(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 5 && std::string(argv[1]) == "--p2-check"
            && std::string(argv[3]) == "--iterations") {
            const auto report = nextengine::nonlocal::run_cuda_p2_check(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 7 && std::string(argv[1]) == "--p2-tournament"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs") {
            const auto report = nextengine::nonlocal::run_cuda_p2_tournament(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 5 && std::string(argv[1]) == "--p3-check"
            && std::string(argv[3]) == "--iterations") {
            const auto report = nextengine::nonlocal::run_cuda_p3_check(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 7 && std::string(argv[1]) == "--p3-tournament"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs") {
            const auto report = nextengine::nonlocal::run_cuda_p3_tournament(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 5 && std::string(argv[1]) == "--p4-check"
            && std::string(argv[3]) == "--iterations") {
            const auto report = nextengine::nonlocal::run_cuda_p4_check(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "iteration count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 7 && std::string(argv[1]) == "--p4-tournament"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs") {
            const auto report = nextengine::nonlocal::run_cuda_p4_tournament(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 7 && std::string(argv[1]) == "--p2-decision"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs") {
            const auto report = nextengine::nonlocal::run_cuda_p2_decision(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 7 && std::string(argv[1]) == "--layout-tournament"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs") {
            const auto report = nextengine::nonlocal::run_cuda_layout_tournament(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 7 && std::string(argv[1]) == "--locality-tournament"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs") {
            const auto report = nextengine::nonlocal::run_cuda_locality_tournament(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"));
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (argc == 7 && std::string(argv[1]) == "--retained-tournament"
            && std::string(argv[3]) == "--warmup" && std::string(argv[5]) == "--runs") {
            const auto report = nextengine::nonlocal::run_cuda_retained_tournament(
                nextengine::nonlocal::find_profile(argv[2]),
                bounded_integer(argv[4], "warmup count"),
                bounded_integer(argv[6], "run count"));
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
