#include "formula_reclosure.hpp"

#include <exception>
#include <iostream>
#include <string>

int main(int argc, char** argv) {
    try {
        if (argc != 2 || std::string(argv[1]) != "--self-test") {
            std::cerr << "usage: nonlocal-formula-reclosure --self-test\n";
            return 2;
        }
        const nextengine::nonlocal::fcr::FormulaReport report =
            nextengine::nonlocal::fcr::run_formula_controls();
        std::cout << report.json << '\n';
        return report.passed ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-reclosure: " << error.what() << '\n';
        return 1;
    }
}
