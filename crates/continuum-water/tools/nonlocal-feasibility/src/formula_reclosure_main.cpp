#include "formula_discriminators.hpp"
#include "formula_reclosure.hpp"
#include "variational_reference.hpp"

#include <exception>
#include <iostream>
#include <string>

int main(int argc, char** argv) {
    try {
        if (argc != 2) {
            std::cerr << "usage: nonlocal-formula-reclosure "
                         "--self-test|--pair-pressure-self-test|"
                         "--reference-solver-self-test|--conditioning-self-test|"
                         "--sissm-self-test|--sissm-term-local-self-test\n";
            return 2;
        }
        const std::string command = argv[1];
        if (command == "--self-test") {
            const nextengine::nonlocal::fcr::FormulaReport report =
                nextengine::nonlocal::fcr::run_formula_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--pair-pressure-self-test") {
            const nextengine::nonlocal::fcr::DiscriminatorReport report =
                nextengine::nonlocal::fcr::run_pair_pressure_discriminator();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--reference-solver-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_reference_solver_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--conditioning-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_conditioning_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--sissm-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_sissm_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        if (command == "--sissm-term-local-self-test") {
            const nextengine::nonlocal::fcr::ReferenceSolverReport report =
                nextengine::nonlocal::fcr::run_sissm_term_local_controls();
            std::cout << report.json << '\n';
            return report.passed ? 0 : 1;
        }
        std::cerr << "usage: nonlocal-formula-reclosure "
                     "--self-test|--pair-pressure-self-test|"
                     "--reference-solver-self-test|--conditioning-self-test|"
                     "--sissm-self-test|--sissm-term-local-self-test\n";
        return 2;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-reclosure: " << error.what() << '\n';
        return 1;
    }
}
