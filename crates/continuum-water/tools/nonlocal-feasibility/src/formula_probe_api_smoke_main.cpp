#include "boundary_reference.hpp"

#include <exception>
#include <iostream>

int main() {
    try {
        const nextengine::nonlocal::fcr::SplitBoundaryReport report =
            nextengine::nonlocal::fcr::run_split_static_boundary_controls();
        std::cout << report.json << '\n';
        return report.passed ? 0 : 1;
    } catch (const std::exception& error) {
        std::cerr << "nonlocal-formula-probe-api-smoke: "
                  << error.what() << '\n';
        return 2;
    }
}
