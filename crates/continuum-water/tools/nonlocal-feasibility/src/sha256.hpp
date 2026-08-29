#pragma once

#include <string>
#include <string_view>

namespace nextengine::nonlocal {

std::string sha256_hex(std::string_view input);

} // namespace nextengine::nonlocal
