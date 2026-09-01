#define main ncgp13_embedded_entry
#include "corrected_cpu_pressure_contact_main.cpp"
#undef main

#include <cctype>
#include <cstring>
#include <functional>
#include <optional>

#if __cplusplus != 201703L
#error "NCGP14 requires exact ISO C++17 mode"
#endif

namespace ncgp14 {

#ifndef NCGP14_CONTRACT_ROOT
#define NCGP14_CONTRACT_ROOT "unconfigured"
#endif
#ifndef NCGP14_SOURCE_ROOT
#define NCGP14_SOURCE_ROOT "unconfigured"
#endif
#ifndef NCGP14_SOURCE_COMMIT
#define NCGP14_SOURCE_COMMIT "unconfigured"
#endif
#ifndef NCGP14_SOURCE_TREE
#define NCGP14_SOURCE_TREE "unconfigured"
#endif
#ifndef NCGP14_COMPILER_FAMILY
#define NCGP14_COMPILER_FAMILY "unconfigured"
#endif
#ifndef NCGP14_COMPILER_VERSION
#define NCGP14_COMPILER_VERSION "unconfigured"
#endif
#ifndef NCGP14_COMPILER_FLAGS
#define NCGP14_COMPILER_FLAGS "unconfigured"
#endif

constexpr std::string_view kSchema =
    "nextengine.nonlocal.ncgp14.result.v3";
constexpr std::string_view kInvocation =
    "nonlocal-corrected-cpu-confined-pressure-contact --confined-pressure-contact-discriminator";
constexpr std::string_view kParentCommit =
    "b1fdcd59fa105e48039bd4c1b5dda72bf7401a64";
constexpr std::string_view kParentTree =
    "c541413c73fee10186c1f9652264b7b64dd473cf";
constexpr std::string_view kParentSourceFile =
    "ca62174997aaeee928efef9622c9869f7e4f3cf0909fd6f562005684d17e3873";
constexpr std::string_view kParentSourceAggregate =
    "fe71ebecca1bbac5232927931a65186d145d6f2b093ee48d28f24c505ae28af7";
constexpr std::string_view kParentContract =
    "3c26ed0b806308113384e54198935828bf61fb4a5447cc7311b78285e6b3c904";
constexpr std::string_view kParentBinary =
    "877c0e99f6984772cb8c74d2105a90e4f15162fd00df8d774c10c69b516662a4";
constexpr std::string_view kParentStdout =
    "496c450fe80b5bfd3f20b042a4874e69afef8d2604286447ad35f50f6e72246e";
constexpr std::string_view kParentResult =
    "053a6a924a331153a72673d9a5d3809b7a0ea014799c36a3c6cba77ef088c6aa";
constexpr std::string_view kOpen128Input =
    "f9dbf235d5e176efe29468eba26551a1958106a5eb9d6cffbcb74ee57b4b4d1e";
constexpr std::string_view kOpen512Input =
    "82cf83cbfc5839026c8dff77c55b81be2e5597982c31493a7016a9f5fca2a8fa";

constexpr std::array<std::string_view, 4> kProfileRoots{
    "3d9b1349c5dd80e94b6ac147d25e86982ba237eed995174ab2420dfa1abad882",
    "d44f72c90122aa7a626e855dfee7424280e9524fa3eff151ec69e3a62cfdeff8",
    "bb8c13cde7f47f687c6c7fc074a19ba0fdb3e3976194551c244267c2eac514c6",
    "afaac597db6668e448af2b5302084c32572687c28608736cedcb7eb4bbfb5857"};
constexpr std::array<std::string_view, 3> kFixtureRoots{
    "e97e2d738f096abeb33918f7ac3d3b6a648cc94f4b13d68c49dd8df5fa0f4bba",
    "465986cb5bd54e921337a12f5221b86fd935fa0a63d91ba9653a2dcc977b5ffa",
    "6dbaddf563b825e28e37aee58606f25cf50b17479cc3260e919103be79f7c2b2"};
constexpr std::array<std::string_view, 5> kLaneRoots{
    "9e46870992b526c2e4764dae8d2d0a37d11cc6832fd9bba74ce4507b790f7c58",
    "a6ff5502e46f3f8bc41aa55c11169e956f875399e250b8cdf8cce1095f593393",
    "7bac382c7edf33ca4db698d29f95b5ce62e485f1a0378e8e983f0ef274cc875c",
    "8d13d0eefb7e27acafb0588c052ed252a1246ec8e3c5cd5ebe4c748b59796ba6",
    "0646ecf9468815f35f6968c408e1e7489d59e28ff61dfc9fe3baff9e924bfcb2"};

void put_u8(std::string& bytes, std::uint8_t value) {
    bytes.push_back(static_cast<char>(value));
}
void put_u32(std::string& bytes, std::uint32_t value) {
    append_u32(bytes, value);
}
void put_u64(std::string& bytes, std::uint64_t value) {
    append_u64(bytes, value);
}
void put_f32(std::string& bytes, float value) {
    append_f32(bytes, value);
}
void put_f64(std::string& bytes, long double value) {
    append_wide(bytes, value);
}
void put_string(std::string& bytes, std::string_view value) {
    append_string(bytes, value);
}
std::string root_of(std::string bytes) {
    return digest(std::move(bytes));
}
bool hex_root(std::string_view value) {
    return value.size() == 64U
        && std::all_of(value.begin(), value.end(), [](char item) {
            return (item >= '0' && item <= '9')
                || (item >= 'a' && item <= 'f');
        });
}

struct Work14 {
    Work parent;
    std::uint64_t fixture_lattice_sites_generated = 0U;
    std::uint64_t ghost_cells_tested = 0U;
    std::uint64_t records_canonicalized = 0U;
    std::uint64_t profile_fields_checked = 0U;
    std::uint64_t dynamic_records_admitted = 0U;
    std::uint64_t ghost_records_admitted = 0U;
    std::uint64_t raw_order_records_hashed = 0U;
    std::uint64_t face_classifications = 0U;
    std::uint64_t ghost_pair_distance_tests = 0U;
    std::uint64_t ghost_pairs_accepted = 0U;
    std::uint64_t row_degree_checks = 0U;
    std::uint64_t surface_pair_distance_tests = 0U;
    std::uint64_t surface_active_pairs = 0U;
    std::uint64_t surface_force_evaluations = 0U;
    std::uint64_t surface_reduction_adds = 0U;
    std::uint64_t lane_scalar_comparisons = 0U;
    std::uint64_t transaction_values_compared = 0U;
    std::uint64_t scratch_values_compared = 0U;
    std::uint64_t receipt_children_aggregated = 0U;
    std::uint64_t portable_content_fields_serialized = 0U;
    std::uint64_t binary_file_reads = 0U;
    std::uint64_t binary_bytes_hashed = 0U;
};

std::array<std::uint64_t, 42> work14_values(const Work14& work) {
    const auto base = work_values(work.parent);
    std::array<std::uint64_t, 42> result{};
    std::copy(base.begin(), base.end(), result.begin());
    const std::array<std::uint64_t, 22> extra{
        work.fixture_lattice_sites_generated, work.ghost_cells_tested,
        work.records_canonicalized, work.profile_fields_checked,
        work.dynamic_records_admitted, work.ghost_records_admitted,
        work.raw_order_records_hashed, work.face_classifications,
        work.ghost_pair_distance_tests, work.ghost_pairs_accepted,
        work.row_degree_checks, work.surface_pair_distance_tests,
        work.surface_active_pairs, work.surface_force_evaluations,
        work.surface_reduction_adds, work.lane_scalar_comparisons,
        work.transaction_values_compared, work.scratch_values_compared,
        work.receipt_children_aggregated,
        work.portable_content_fields_serialized, work.binary_file_reads,
        work.binary_bytes_hashed};
    std::copy(extra.begin(), extra.end(), result.begin() + 20);
    return result;
}

void add_work14(Work14& target, const Work14& value) {
    add_work(target.parent, value.parent);
    target.fixture_lattice_sites_generated +=
        value.fixture_lattice_sites_generated;
    target.ghost_cells_tested += value.ghost_cells_tested;
    target.records_canonicalized += value.records_canonicalized;
    target.profile_fields_checked += value.profile_fields_checked;
    target.dynamic_records_admitted += value.dynamic_records_admitted;
    target.ghost_records_admitted += value.ghost_records_admitted;
    target.raw_order_records_hashed += value.raw_order_records_hashed;
    target.face_classifications += value.face_classifications;
    target.ghost_pair_distance_tests += value.ghost_pair_distance_tests;
    target.ghost_pairs_accepted += value.ghost_pairs_accepted;
    target.row_degree_checks += value.row_degree_checks;
    target.surface_pair_distance_tests += value.surface_pair_distance_tests;
    target.surface_active_pairs += value.surface_active_pairs;
    target.surface_force_evaluations += value.surface_force_evaluations;
    target.surface_reduction_adds += value.surface_reduction_adds;
    target.lane_scalar_comparisons += value.lane_scalar_comparisons;
    target.transaction_values_compared += value.transaction_values_compared;
    target.scratch_values_compared += value.scratch_values_compared;
    target.receipt_children_aggregated += value.receipt_children_aggregated;
    target.portable_content_fields_serialized +=
        value.portable_content_fields_serialized;
    target.binary_file_reads += value.binary_file_reads;
    target.binary_bytes_hashed += value.binary_bytes_hashed;
}
void add_child(Work14& target, const Work14& child) {
    add_work14(target, child);
    ++target.receipt_children_aggregated;
}
void add_parent_work(Work14& target, const Work& child) {
    add_work(target.parent, child);
}
bool same_work14(const Work14& lhs, const Work14& rhs) {
    const auto lhs_values = work14_values(lhs);
    const auto rhs_values = work14_values(rhs);
    bool same = true;
    for (std::size_t index = 0U; index < lhs_values.size(); ++index) {
        const bool field_same = lhs_values[index] == rhs_values[index];
        same = field_same && same;
    }
    return same;
}
std::string work14_root(const Work14& work) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.work.v1");
    for (const std::uint64_t value : work14_values(work)) put_u64(bytes, value);
    return root_of(std::move(bytes));
}

struct WorkSeal14 {
    Work14 expected;
    bool exact = false;
    std::string expected_root;
    std::string actual_root;
};

WorkSeal14 verify_work14(const Work14& actual, const Work14& expected) {
    WorkSeal14 result;
    result.expected = expected;
    result.expected_root = work14_root(expected);
    result.actual_root = work14_root(actual);
    result.exact = same_work14(actual, expected);
    return result;
}

void append_work_verifier(std::string& bytes, const WorkSeal14& verifier) {
    put_u8(bytes, verifier.exact ? 1U : 0U);
    put_string(bytes, verifier.expected_root);
    put_string(bytes, verifier.actual_root);
}

bool counted_predicate(Work14& work, bool predicate) {
    ++work.lane_scalar_comparisons;
    return predicate;
}

void combine_predicate(bool& aggregate, Work14& work, bool predicate) {
    const bool evaluated = counted_predicate(work, predicate);
    aggregate = evaluated && aggregate;
}

void combine_failure_predicate(bool& aggregate, Work14& work,
    bool predicate) {
    const bool evaluated = counted_predicate(work, predicate);
    aggregate = evaluated || aggregate;
}

bool counted_same_parent_work14(const Work& lhs, const Work& rhs,
    Work14& owner) {
    const auto lhs_values = work_values(lhs);
    const auto rhs_values = work_values(rhs);
    bool same = true;
    for (std::size_t index = 0U; index < lhs_values.size(); ++index) {
        combine_predicate(same, owner,
            lhs_values[index] == rhs_values[index]);
    }
    return same;
}

bool counted_same_work14(const Work14& lhs, const Work14& rhs,
    Work14& owner) {
    const auto lhs_values = work14_values(lhs);
    const auto rhs_values = work14_values(rhs);
    bool same = true;
    for (std::size_t index = 0U; index < lhs_values.size(); ++index) {
        combine_predicate(same, owner,
            lhs_values[index] == rhs_values[index]);
    }
    return same;
}

State working_state14(const std::vector<NonlocalGpuSample>& samples,
    Work14& owner) {
    State result = working_state(samples);
    owner.records_canonicalized += result.id.size();
    return result;
}


struct Json {
    enum class Type { Object, Array, String, Number, Boolean, Null };
    Type type = Type::Null;
    std::size_t begin = 0U;
    std::size_t end = 0U;
    std::size_t content_begin = 0U;
    std::size_t content_end = 0U;
    std::string text;
    bool boolean = false;
    std::vector<std::pair<std::string, Json>> object;
    std::vector<Json> array;
};

bool json_whitespace14(unsigned char value) {
    return value == 0x20U || value == 0x09U
        || value == 0x0aU || value == 0x0dU;
}

int json_hex14(unsigned char value) {
    if (value >= '0' && value <= '9') return value - '0';
    if (value >= 'a' && value <= 'f') return value - 'a' + 10;
    if (value >= 'A' && value <= 'F') return value - 'A' + 10;
    return -1;
}

void append_utf8_14(std::string& output, std::uint32_t codepoint) {
    if (codepoint <= 0x7fU) {
        output.push_back(static_cast<char>(codepoint));
    } else if (codepoint <= 0x7ffU) {
        output.push_back(static_cast<char>(0xc0U | (codepoint >> 6U)));
        output.push_back(static_cast<char>(0x80U | (codepoint & 0x3fU)));
    } else if (codepoint <= 0xffffU) {
        output.push_back(static_cast<char>(0xe0U | (codepoint >> 12U)));
        output.push_back(static_cast<char>(0x80U
            | ((codepoint >> 6U) & 0x3fU)));
        output.push_back(static_cast<char>(0x80U | (codepoint & 0x3fU)));
    } else {
        output.push_back(static_cast<char>(0xf0U | (codepoint >> 18U)));
        output.push_back(static_cast<char>(0x80U
            | ((codepoint >> 12U) & 0x3fU)));
        output.push_back(static_cast<char>(0x80U
            | ((codepoint >> 6U) & 0x3fU)));
        output.push_back(static_cast<char>(0x80U | (codepoint & 0x3fU)));
    }
}

class JsonParser {
public:
    JsonParser(std::string_view source, Work14& work,
        std::uint64_t& syntax_predicates_completed)
        : source_(source), work_(work),
          syntax_predicates_completed_(syntax_predicates_completed) {}
    Json parse() {
        skip();
        Json value = parse_value();
        skip();
        if (!syntax(position_ == source_.size())) {
            fail("trailing JSON bytes");
        }
        return value;
    }
    std::size_t position() const { return position_; }
private:
    bool syntax(bool predicate) {
        ++work_.lane_scalar_comparisons;
        ++syntax_predicates_completed_;
        return predicate;
    }
    [[noreturn]] void fail(const char* message) const {
        throw std::runtime_error(std::string("embedded JSON: ") + message);
    }
    void skip() {
        for (;;) {
            if (!syntax(position_ < source_.size())) break;
            if (!syntax(json_whitespace14(
                    static_cast<unsigned char>(source_[position_])))) {
                break;
            }
            ++position_;
        }
    }
    char take() {
        if (!syntax(position_ < source_.size())) fail("unexpected end");
        return source_[position_++];
    }
    Json parse_value() {
        skip();
        if (!syntax(position_ < source_.size())) fail("missing value");
        const char lead = source_[position_];
        if (syntax(lead == '{')) return parse_object();
        if (syntax(lead == '[')) return parse_array();
        if (syntax(lead == '"')) return parse_string();
        if (syntax(lead == 't')) {
            return parse_literal("true", Json::Type::Boolean, true);
        }
        if (syntax(lead == 'f')) {
            return parse_literal("false", Json::Type::Boolean, false);
        }
        if (syntax(lead == 'n')) {
            return parse_literal("null", Json::Type::Null, false);
        }
        return parse_number();
    }
    Json parse_string() {
        Json result;
        result.type = Json::Type::String;
        result.begin = position_;
        if (!syntax(take() == '"')) fail("string opening quote");
        result.content_begin = position_;
        std::string decoded;
        for (;;) {
            if (!syntax(position_ < source_.size())) {
                fail("unterminated string");
            }
            const char item = take();
            if (syntax(item == '"')) {
                result.content_end = position_ - 1U;
                result.end = position_;
                result.text = std::move(decoded);
                return result;
            }
            if (syntax(static_cast<unsigned char>(item) < 0x20U)) {
                fail("control byte in string");
            }
            const unsigned char unsigned_item =
                static_cast<unsigned char>(item);
            if (unsigned_item >= 0x80U) {
                std::uint32_t codepoint = 0U;
                std::size_t continuation_count = 0U;
                std::uint32_t minimum = 0U;
                if (unsigned_item >= 0xc2U && unsigned_item <= 0xdfU) {
                    codepoint = unsigned_item & 0x1fU;
                    continuation_count = 1U;
                    minimum = 0x80U;
                } else if (unsigned_item >= 0xe0U
                    && unsigned_item <= 0xefU) {
                    codepoint = unsigned_item & 0x0fU;
                    continuation_count = 2U;
                    minimum = 0x800U;
                } else if (unsigned_item >= 0xf0U
                    && unsigned_item <= 0xf4U) {
                    codepoint = unsigned_item & 0x07U;
                    continuation_count = 3U;
                    minimum = 0x10000U;
                } else {
                    fail("bad UTF-8 lead byte");
                }
                for (std::size_t continuation = 0U;
                     continuation < continuation_count; ++continuation) {
                    const unsigned char next =
                        static_cast<unsigned char>(take());
                    if (!syntax((next & 0xc0U) == 0x80U)) {
                        fail("bad UTF-8 continuation byte");
                    }
                    codepoint = (codepoint << 6U) | (next & 0x3fU);
                }
                if (!syntax(codepoint >= minimum)
                    || !syntax(codepoint <= 0x10ffffU)
                    || !syntax(!(codepoint >= 0xd800U
                        && codepoint <= 0xdfffU))) {
                    fail("bad UTF-8 codepoint");
                }
                append_utf8_14(decoded, codepoint);
                continue;
            }
            if (!syntax(item == '\\')) {
                decoded.push_back(item);
                continue;
            }
            const char escaped = take();
            if (syntax(escaped == '"')) {
                decoded.push_back('"');
            } else if (syntax(escaped == '\\')) {
                decoded.push_back('\\');
            } else if (syntax(escaped == '/')) {
                decoded.push_back('/');
            } else if (syntax(escaped == 'b')) {
                decoded.push_back('\b');
            } else if (syntax(escaped == 'f')) {
                decoded.push_back('\f');
            } else if (syntax(escaped == 'n')) {
                decoded.push_back('\n');
            } else if (syntax(escaped == 'r')) {
                decoded.push_back('\r');
            } else if (syntax(escaped == 't')) {
                decoded.push_back('\t');
            } else if (syntax(escaped == 'u')) {
                const auto unicode_unit = [&]() {
                    std::uint32_t unit = 0U;
                    for (std::size_t digit = 0U; digit < 4U; ++digit) {
                        const char item_hex = take();
                        const int value = json_hex14(
                            static_cast<unsigned char>(item_hex));
                        if (!syntax(value >= 0)) fail("bad unicode escape");
                        unit = (unit << 4U)
                            | static_cast<std::uint32_t>(value);
                    }
                    return unit;
                };
                std::uint32_t codepoint = unicode_unit();
                if (codepoint >= 0xd800U && codepoint <= 0xdbffU) {
                    if (!syntax(take() == '\\') || !syntax(take() == 'u')) {
                        fail("missing low surrogate");
                    }
                    const std::uint32_t low = unicode_unit();
                    if (!syntax(low >= 0xdc00U && low <= 0xdfffU)) {
                        fail("bad low surrogate");
                    }
                    codepoint = 0x10000U
                        + ((codepoint - 0xd800U) << 10U)
                        + (low - 0xdc00U);
                } else if (codepoint >= 0xdc00U && codepoint <= 0xdfffU) {
                    fail("lone low surrogate");
                }
                append_utf8_14(decoded, codepoint);
            } else {
                fail("unsupported JSON escape");
            }
        }
    }
    Json parse_object() {
        Json result;
        result.type = Json::Type::Object;
        result.begin = position_;
        take();
        skip();
        if (syntax(position_ < source_.size())) {
            if (syntax(source_[position_] == '}')) {
                ++position_;
                result.end = position_;
                return result;
            }
        }
        for (;;) {
            skip();
            Json key = parse_string();
            skip();
            if (!syntax(take() == ':')) fail("missing colon");
            Json value = parse_value();
            result.object.emplace_back(std::move(key.text), std::move(value));
            skip();
            const char separator = take();
            if (syntax(separator == '}')) break;
            if (!syntax(separator == ',')) fail("missing object comma");
        }
        result.end = position_;
        return result;
    }
    Json parse_array() {
        Json result;
        result.type = Json::Type::Array;
        result.begin = position_;
        take();
        skip();
        if (syntax(position_ < source_.size())) {
            if (syntax(source_[position_] == ']')) {
                ++position_;
                result.end = position_;
                return result;
            }
        }
        for (;;) {
            result.array.push_back(parse_value());
            skip();
            const char separator = take();
            if (syntax(separator == ']')) break;
            if (!syntax(separator == ',')) fail("missing array comma");
        }
        result.end = position_;
        return result;
    }
    Json parse_number() {
        Json result;
        result.type = Json::Type::Number;
        result.begin = position_;
        if (syntax(source_[position_] == '-')) ++position_;
        if (!syntax(position_ < source_.size())) fail("bad number");
        if (syntax(source_[position_] == '0')) {
            ++position_;
        } else {
            if (!syntax(std::isdigit(static_cast<unsigned char>(
                    source_[position_])) != 0)) {
                fail("bad number integer");
            }
            for (;;) {
                if (!syntax(position_ < source_.size())) break;
                if (!syntax(std::isdigit(static_cast<unsigned char>(
                        source_[position_])) != 0)) {
                    break;
                }
                ++position_;
            }
        }
        if (syntax(position_ < source_.size())) {
            if (syntax(source_[position_] == '.')) {
                ++position_;
                if (!syntax(position_ < source_.size())) {
                    fail("bad number fraction");
                }
                if (!syntax(std::isdigit(static_cast<unsigned char>(
                        source_[position_])) != 0)) {
                    fail("bad number fraction");
                }
                for (;;) {
                    if (!syntax(position_ < source_.size())) break;
                    if (!syntax(std::isdigit(static_cast<unsigned char>(
                            source_[position_])) != 0)) {
                        break;
                    }
                    ++position_;
                }
            }
        }
        if (syntax(position_ < source_.size())) {
            if (syntax(source_[position_] == 'e')) {
                parse_exponent();
            } else if (syntax(source_[position_] == 'E')) {
                parse_exponent();
            }
        }
        result.end = position_;
        result.text = std::string(source_.substr(result.begin,
            result.end - result.begin));
        return result;
    }
    void parse_exponent() {
        ++position_;
        if (syntax(position_ < source_.size())) {
            if (syntax(source_[position_] == '+')) {
                ++position_;
            } else if (syntax(source_[position_] == '-')) {
                ++position_;
            }
        }
        if (!syntax(position_ < source_.size())) {
            fail("bad number exponent");
        }
        if (!syntax(std::isdigit(static_cast<unsigned char>(
                source_[position_])) != 0)) {
            fail("bad number exponent");
        }
        for (;;) {
            if (!syntax(position_ < source_.size())) break;
            if (!syntax(std::isdigit(static_cast<unsigned char>(
                    source_[position_])) != 0)) {
                break;
            }
            ++position_;
        }
    }
    Json parse_literal(std::string_view literal, Json::Type type, bool value) {
        Json result;
        result.type = type;
        result.begin = position_;
        if (!syntax(source_.substr(position_, literal.size()) == literal)) {
            fail("bad literal");
        }
        position_ += literal.size();
        result.end = position_;
        result.boolean = value;
        return result;
    }
    std::string_view source_;
    Work14& work_;
    std::uint64_t& syntax_predicates_completed_;
    std::size_t position_ = 0U;
};

// Total, byte-driven RFC-8259 recognizer used by the production B+140
// validator.  Its grammar state is deliberately independent of JsonParser's
// decoded-tree construction.  Once a byte makes the document invalid, every
// remaining byte is still consumed in the explicit rejecting sink.
class ProductionJsonRecognizer14 {
public:
    bool consume(unsigned char byte) {
        if (rejecting_) return false;
        process(byte);
        return !rejecting_;
    }

    bool finish() {
        if (rejecting_) return false;
        if (mode_ == Mode::Number) {
            if (!number_complete()) return reject();
            mode_ = Mode::Structural;
            complete_value();
        }
        return !rejecting_ && mode_ == Mode::Structural
            && contexts_.empty() && root_complete_;
    }

private:
    enum class Mode { Structural, String, Literal, Number };
    enum class ContextKind { Object, Array };
    enum class Expect {
        ObjectKeyOrEnd,
        ObjectKey,
        ObjectColon,
        ObjectValue,
        ObjectCommaOrEnd,
        ArrayValueOrEnd,
        ArrayValue,
        ArrayCommaOrEnd,
    };
    enum class StringState {
        Normal,
        Escape,
        Unicode,
        LowBackslash,
        LowU,
        Utf8,
    };
    enum class NumberState {
        Minus,
        Zero,
        Integer,
        FractionMark,
        Fraction,
        ExponentMark,
        ExponentSign,
        Exponent,
    };
    struct Context {
        ContextKind kind;
        Expect expect;
    };

    bool reject() {
        rejecting_ = true;
        return false;
    }

    static bool digit(unsigned char byte) {
        return byte >= static_cast<unsigned char>('0')
            && byte <= static_cast<unsigned char>('9');
    }

    static bool delimiter(unsigned char byte) {
        return json_whitespace14(byte) || byte == ',' || byte == ']'
            || byte == '}';
    }

    void process(unsigned char byte) {
        bool again = true;
        while (again && !rejecting_) {
            again = false;
            if (mode_ == Mode::String) {
                process_string(byte);
            } else if (mode_ == Mode::Literal) {
                process_literal(byte);
            } else if (mode_ == Mode::Number) {
                again = process_number(byte);
            } else {
                process_structural(byte);
            }
        }
    }

    void process_structural(unsigned char byte) {
        if (root_complete_ && contexts_.empty()) {
            if (!json_whitespace14(byte)) reject();
            return;
        }
        if (contexts_.empty()) {
            if (json_whitespace14(byte)) return;
            start_value(byte);
            return;
        }
        Context& context = contexts_.back();
        switch (context.expect) {
            case Expect::ObjectKeyOrEnd:
                if (json_whitespace14(byte)) return;
                if (byte == '}') {
                    contexts_.pop_back();
                    complete_value();
                    return;
                }
                if (byte != '"') {
                    reject();
                    return;
                }
                start_string(true);
                return;
            case Expect::ObjectKey:
                if (json_whitespace14(byte)) return;
                if (byte != '"') {
                    reject();
                    return;
                }
                start_string(true);
                return;
            case Expect::ObjectColon:
                if (json_whitespace14(byte)) return;
                if (byte != ':') {
                    reject();
                    return;
                }
                context.expect = Expect::ObjectValue;
                return;
            case Expect::ObjectValue:
                if (json_whitespace14(byte)) return;
                start_value(byte);
                return;
            case Expect::ObjectCommaOrEnd:
                if (json_whitespace14(byte)) return;
                if (byte == ',') {
                    context.expect = Expect::ObjectKey;
                } else if (byte == '}') {
                    contexts_.pop_back();
                    complete_value();
                } else {
                    reject();
                }
                return;
            case Expect::ArrayValueOrEnd:
                if (json_whitespace14(byte)) return;
                if (byte == ']') {
                    contexts_.pop_back();
                    complete_value();
                    return;
                }
                start_value(byte);
                return;
            case Expect::ArrayValue:
                if (json_whitespace14(byte)) return;
                start_value(byte);
                return;
            case Expect::ArrayCommaOrEnd:
                if (json_whitespace14(byte)) return;
                if (byte == ',') {
                    context.expect = Expect::ArrayValue;
                } else if (byte == ']') {
                    contexts_.pop_back();
                    complete_value();
                } else {
                    reject();
                }
                return;
        }
    }

    void start_value(unsigned char byte) {
        if (byte == '{') {
            contexts_.push_back(
                {ContextKind::Object, Expect::ObjectKeyOrEnd});
        } else if (byte == '[') {
            contexts_.push_back(
                {ContextKind::Array, Expect::ArrayValueOrEnd});
        } else if (byte == '"') {
            start_string(false);
        } else if (byte == 't') {
            start_literal("true");
        } else if (byte == 'f') {
            start_literal("false");
        } else if (byte == 'n') {
            start_literal("null");
        } else if (byte == '-') {
            mode_ = Mode::Number;
            number_state_ = NumberState::Minus;
        } else if (byte == '0') {
            mode_ = Mode::Number;
            number_state_ = NumberState::Zero;
        } else if (digit(byte)) {
            mode_ = Mode::Number;
            number_state_ = NumberState::Integer;
        } else {
            reject();
        }
    }

    void start_string(bool key) {
        mode_ = Mode::String;
        string_state_ = StringState::Normal;
        string_is_key_ = key;
        unicode_digits_ = 0U;
        unicode_value_ = 0U;
        expecting_low_surrogate_ = false;
        utf8_remaining_ = 0U;
    }

    void process_string(unsigned char byte) {
        switch (string_state_) {
            case StringState::Normal:
                if (byte == '"') {
                    mode_ = Mode::Structural;
                    if (string_is_key_) {
                        if (contexts_.empty()
                            || contexts_.back().kind != ContextKind::Object) {
                            reject();
                        } else {
                            contexts_.back().expect = Expect::ObjectColon;
                        }
                    } else {
                        complete_value();
                    }
                } else if (byte == '\\') {
                    string_state_ = StringState::Escape;
                } else if (byte < 0x20U) {
                    reject();
                } else if (byte >= 0x80U) {
                    start_utf8(byte);
                }
                return;
            case StringState::Escape:
                if (byte == '"' || byte == '\\' || byte == '/'
                    || byte == 'b' || byte == 'f' || byte == 'n'
                    || byte == 'r' || byte == 't') {
                    string_state_ = StringState::Normal;
                } else if (byte == 'u') {
                    string_state_ = StringState::Unicode;
                    unicode_digits_ = 0U;
                    unicode_value_ = 0U;
                    expecting_low_surrogate_ = false;
                } else {
                    reject();
                }
                return;
            case StringState::Unicode: {
                const int value = json_hex14(byte);
                if (value < 0) {
                    reject();
                    return;
                }
                unicode_value_ = (unicode_value_ << 4U)
                    | static_cast<std::uint32_t>(value);
                ++unicode_digits_;
                if (unicode_digits_ != 4U) return;
                if (expecting_low_surrogate_) {
                    if (unicode_value_ < 0xdc00U
                        || unicode_value_ > 0xdfffU) {
                        reject();
                    } else {
                        expecting_low_surrogate_ = false;
                        string_state_ = StringState::Normal;
                    }
                } else if (unicode_value_ >= 0xd800U
                    && unicode_value_ <= 0xdbffU) {
                    expecting_low_surrogate_ = true;
                    string_state_ = StringState::LowBackslash;
                } else if (unicode_value_ >= 0xdc00U
                    && unicode_value_ <= 0xdfffU) {
                    reject();
                } else {
                    string_state_ = StringState::Normal;
                }
                return;
            }
            case StringState::LowBackslash:
                if (byte != '\\') {
                    reject();
                } else {
                    string_state_ = StringState::LowU;
                }
                return;
            case StringState::LowU:
                if (byte != 'u') {
                    reject();
                } else {
                    string_state_ = StringState::Unicode;
                    unicode_digits_ = 0U;
                    unicode_value_ = 0U;
                }
                return;
            case StringState::Utf8:
                if ((byte & 0xc0U) != 0x80U) {
                    reject();
                    return;
                }
                utf8_codepoint_ = (utf8_codepoint_ << 6U)
                    | (byte & 0x3fU);
                --utf8_remaining_;
                if (utf8_remaining_ == 0U) {
                    if (utf8_codepoint_ < utf8_minimum_
                        || utf8_codepoint_ > 0x10ffffU
                        || (utf8_codepoint_ >= 0xd800U
                            && utf8_codepoint_ <= 0xdfffU)) {
                        reject();
                    } else {
                        string_state_ = StringState::Normal;
                    }
                }
                return;
        }
    }

    void start_utf8(unsigned char lead) {
        string_state_ = StringState::Utf8;
        if (lead >= 0xc2U && lead <= 0xdfU) {
            utf8_codepoint_ = lead & 0x1fU;
            utf8_remaining_ = 1U;
            utf8_minimum_ = 0x80U;
        } else if (lead >= 0xe0U && lead <= 0xefU) {
            utf8_codepoint_ = lead & 0x0fU;
            utf8_remaining_ = 2U;
            utf8_minimum_ = 0x800U;
        } else if (lead >= 0xf0U && lead <= 0xf4U) {
            utf8_codepoint_ = lead & 0x07U;
            utf8_remaining_ = 3U;
            utf8_minimum_ = 0x10000U;
        } else {
            reject();
        }
    }

    void start_literal(std::string_view literal) {
        mode_ = Mode::Literal;
        literal_ = literal;
        literal_index_ = 1U;
        if (literal_index_ == literal_.size()) {
            mode_ = Mode::Structural;
            complete_value();
        }
    }

    void process_literal(unsigned char byte) {
        if (literal_index_ >= literal_.size()
            || byte != static_cast<unsigned char>(literal_[literal_index_])) {
            reject();
            return;
        }
        ++literal_index_;
        if (literal_index_ == literal_.size()) {
            mode_ = Mode::Structural;
            complete_value();
        }
    }

    bool process_number(unsigned char byte) {
        switch (number_state_) {
            case NumberState::Minus:
                if (byte == '0') number_state_ = NumberState::Zero;
                else if (digit(byte)) number_state_ = NumberState::Integer;
                else reject();
                return false;
            case NumberState::Zero:
                if (byte == '.') number_state_ = NumberState::FractionMark;
                else if (byte == 'e' || byte == 'E') {
                    number_state_ = NumberState::ExponentMark;
                } else if (delimiter(byte)) {
                    mode_ = Mode::Structural;
                    complete_value();
                    return true;
                } else reject();
                return false;
            case NumberState::Integer:
                if (digit(byte)) return false;
                if (byte == '.') number_state_ = NumberState::FractionMark;
                else if (byte == 'e' || byte == 'E') {
                    number_state_ = NumberState::ExponentMark;
                } else if (delimiter(byte)) {
                    mode_ = Mode::Structural;
                    complete_value();
                    return true;
                } else reject();
                return false;
            case NumberState::FractionMark:
                if (digit(byte)) number_state_ = NumberState::Fraction;
                else reject();
                return false;
            case NumberState::Fraction:
                if (digit(byte)) return false;
                if (byte == 'e' || byte == 'E') {
                    number_state_ = NumberState::ExponentMark;
                } else if (delimiter(byte)) {
                    mode_ = Mode::Structural;
                    complete_value();
                    return true;
                } else reject();
                return false;
            case NumberState::ExponentMark:
                if (byte == '+' || byte == '-') {
                    number_state_ = NumberState::ExponentSign;
                } else if (digit(byte)) {
                    number_state_ = NumberState::Exponent;
                } else reject();
                return false;
            case NumberState::ExponentSign:
                if (digit(byte)) number_state_ = NumberState::Exponent;
                else reject();
                return false;
            case NumberState::Exponent:
                if (digit(byte)) return false;
                if (delimiter(byte)) {
                    mode_ = Mode::Structural;
                    complete_value();
                    return true;
                }
                reject();
                return false;
        }
        return false;
    }

    bool number_complete() const {
        return number_state_ == NumberState::Zero
            || number_state_ == NumberState::Integer
            || number_state_ == NumberState::Fraction
            || number_state_ == NumberState::Exponent;
    }

    void complete_value() {
        if (contexts_.empty()) {
            if (root_complete_) {
                reject();
            } else {
                root_complete_ = true;
            }
            return;
        }
        Context& context = contexts_.back();
        if (context.kind == ContextKind::Object
            && context.expect == Expect::ObjectValue) {
            context.expect = Expect::ObjectCommaOrEnd;
        } else if (context.kind == ContextKind::Array
            && (context.expect == Expect::ArrayValueOrEnd
                || context.expect == Expect::ArrayValue)) {
            context.expect = Expect::ArrayCommaOrEnd;
        } else {
            reject();
        }
    }

    Mode mode_ = Mode::Structural;
    std::vector<Context> contexts_;
    bool root_complete_ = false;
    bool rejecting_ = false;
    StringState string_state_ = StringState::Normal;
    bool string_is_key_ = false;
    std::uint32_t unicode_value_ = 0U;
    std::size_t unicode_digits_ = 0U;
    bool expecting_low_surrogate_ = false;
    std::uint32_t utf8_codepoint_ = 0U;
    std::uint32_t utf8_minimum_ = 0U;
    std::size_t utf8_remaining_ = 0U;
    std::string_view literal_;
    std::size_t literal_index_ = 0U;
    NumberState number_state_ = NumberState::Zero;
};

struct ExpectedJsonSyntax14 {
    std::uint64_t predicates_completed = 0U;
    bool valid = false;
    std::size_t rejection_offset = 0U;
};

class ExpectedJsonSyntaxReplay14 {
public:
    explicit ExpectedJsonSyntaxReplay14(std::string_view source)
        : source_(source) {}

    ExpectedJsonSyntax14 replay() {
        try {
            whitespace();
            value();
            whitespace();
            require(position_ == source_.size());
            valid_ = true;
        } catch (const Stop&) {
            valid_ = false;
            // The independent expected recognizer also consumes the complete
            // captured byte extent after entering its rejecting sink.
            while (position_ < source_.size()) advance();
        }
        ++predicates_completed_; // EOF / complete-single-object predicate.
        return {predicates_completed_, valid_, source_.size()};
    }

private:
    struct Stop {};

    bool predicate(bool value) {
        return value;
    }

    void advance() {
        ++position_;
        ++predicates_completed_;
    }

    void require(bool value) {
        if (!predicate(value)) throw Stop{};
    }

    void whitespace() {
        for (;;) {
            if (!predicate(position_ < source_.size())) break;
            if (!predicate(json_whitespace14(
                    static_cast<unsigned char>(source_[position_])))) {
                break;
            }
            advance();
        }
    }

    char consume() {
        require(position_ < source_.size());
        const char result = source_[position_];
        advance();
        return result;
    }

    void value() {
        whitespace();
        require(position_ < source_.size());
        const char lead = source_[position_];
        if (predicate(lead == '{')) {
            object();
        } else if (predicate(lead == '[')) {
            array();
        } else if (predicate(lead == '"')) {
            string();
        } else if (predicate(lead == 't')) {
            literal("true");
        } else if (predicate(lead == 'f')) {
            literal("false");
        } else if (predicate(lead == 'n')) {
            literal("null");
        } else {
            number();
        }
    }

    void string() {
        require(consume() == '"');
        for (;;) {
            require(position_ < source_.size());
            const char item = consume();
            if (predicate(item == '"')) return;
            require(!(static_cast<unsigned char>(item) < 0x20U));
            if (static_cast<unsigned char>(item) >= 0x80U) {
                utf8_sequence(static_cast<unsigned char>(item));
                continue;
            }
            if (!predicate(item == '\\')) continue;
            const char escaped = consume();
            if (predicate(escaped == '"')) {
                continue;
            }
            if (predicate(escaped == '\\')) {
                continue;
            }
            if (predicate(escaped == '/')) {
                continue;
            }
            if (predicate(escaped == 'b')) {
                continue;
            }
            if (predicate(escaped == 'f')) {
                continue;
            }
            if (predicate(escaped == 'n')) {
                continue;
            }
            if (predicate(escaped == 'r')) {
                continue;
            }
            if (predicate(escaped == 't')) {
                continue;
            }
            require(escaped == 'u');
            std::uint32_t codepoint = unicode_unit();
            if (codepoint >= 0xd800U && codepoint <= 0xdbffU) {
                require(consume() == '\\');
                require(consume() == 'u');
                const std::uint32_t low = unicode_unit();
                require(low >= 0xdc00U && low <= 0xdfffU);
            } else {
                require(!(codepoint >= 0xdc00U && codepoint <= 0xdfffU));
            }
        }
    }

    std::uint32_t unicode_unit() {
        std::uint32_t result = 0U;
        for (std::size_t digit = 0U; digit < 4U; ++digit) {
            const int value = json_hex14(
                static_cast<unsigned char>(consume()));
            require(value >= 0);
            result = (result << 4U) | static_cast<std::uint32_t>(value);
        }
        return result;
    }

    void utf8_sequence(unsigned char lead) {
        std::uint32_t codepoint = 0U;
        std::size_t continuation_count = 0U;
        std::uint32_t minimum = 0U;
        if (lead >= 0xc2U && lead <= 0xdfU) {
            codepoint = lead & 0x1fU;
            continuation_count = 1U;
            minimum = 0x80U;
        } else if (lead >= 0xe0U && lead <= 0xefU) {
            codepoint = lead & 0x0fU;
            continuation_count = 2U;
            minimum = 0x800U;
        } else if (lead >= 0xf0U && lead <= 0xf4U) {
            codepoint = lead & 0x07U;
            continuation_count = 3U;
            minimum = 0x10000U;
        } else {
            throw Stop{};
        }
        for (std::size_t index = 0U; index < continuation_count; ++index) {
            const unsigned char item = static_cast<unsigned char>(consume());
            require((item & 0xc0U) == 0x80U);
            codepoint = (codepoint << 6U) | (item & 0x3fU);
        }
        require(codepoint >= minimum);
        require(codepoint <= 0x10ffffU);
        require(!(codepoint >= 0xd800U && codepoint <= 0xdfffU));
    }

    void object() {
        static_cast<void>(consume());
        whitespace();
        if (predicate(position_ < source_.size())) {
            if (predicate(source_[position_] == '}')) {
                advance();
                return;
            }
        }
        for (;;) {
            whitespace();
            string();
            whitespace();
            require(consume() == ':');
            value();
            whitespace();
            const char separator = consume();
            if (predicate(separator == '}')) return;
            require(separator == ',');
        }
    }

    void array() {
        static_cast<void>(consume());
        whitespace();
        if (predicate(position_ < source_.size())) {
            if (predicate(source_[position_] == ']')) {
                advance();
                return;
            }
        }
        for (;;) {
            value();
            whitespace();
            const char separator = consume();
            if (predicate(separator == ']')) return;
            require(separator == ',');
        }
    }

    void number() {
        if (predicate(source_[position_] == '-')) advance();
        require(position_ < source_.size());
        if (predicate(source_[position_] == '0')) {
            advance();
        } else {
            require(std::isdigit(
                static_cast<unsigned char>(source_[position_])) != 0);
            for (;;) {
                if (!predicate(position_ < source_.size())) break;
                if (!predicate(std::isdigit(static_cast<unsigned char>(
                        source_[position_])) != 0)) {
                    break;
                }
                advance();
            }
        }
        if (predicate(position_ < source_.size())) {
            if (predicate(source_[position_] == '.')) {
                advance();
                require(position_ < source_.size());
                require(std::isdigit(
                    static_cast<unsigned char>(source_[position_])) != 0);
                for (;;) {
                    if (!predicate(position_ < source_.size())) break;
                    if (!predicate(std::isdigit(static_cast<unsigned char>(
                        source_[position_])) != 0)) {
                        break;
                    }
                    advance();
                }
            }
        }
        if (predicate(position_ < source_.size())) {
            if (predicate(source_[position_] == 'e')) {
                exponent();
            } else if (predicate(source_[position_] == 'E')) {
                exponent();
            }
        }
    }

    void exponent() {
        advance();
        if (predicate(position_ < source_.size())) {
            if (predicate(source_[position_] == '+')) {
                advance();
            } else if (predicate(source_[position_] == '-')) {
                advance();
            }
        }
        require(position_ < source_.size());
        require(std::isdigit(
            static_cast<unsigned char>(source_[position_])) != 0);
        for (;;) {
            if (!predicate(position_ < source_.size())) break;
                if (!predicate(std::isdigit(static_cast<unsigned char>(
                        source_[position_])) != 0)) {
                    break;
                }
            advance();
        }
    }

    void literal(std::string_view expected) {
        require(source_.substr(position_, expected.size()) == expected);
        for (std::size_t index = 0U; index < expected.size(); ++index) {
            advance();
        }
    }

    std::string_view source_;
    std::size_t position_ = 0U;
    std::uint64_t predicates_completed_ = 0U;
    bool valid_ = false;
};

ExpectedJsonSyntax14 expected_json_syntax14(std::string_view source) {
    return ExpectedJsonSyntaxReplay14(source).replay();
}

struct RootFieldShape14 {
    bool occurrence_exact = false;
    bool string_type = false;
    bool length_exact = false;
    std::array<bool, 64> lowercase_hex{};
    const Json* node = nullptr;
    std::string value;
};

struct RootShape14 {
    bool top_level_object = false;
    std::array<RootFieldShape14, 2> fields;
    bool exact = false;
};

RootShape14 inspect_root_shape14(const Json& root) {
    RootShape14 result;
    result.top_level_object = root.type == Json::Type::Object;
    constexpr std::array<std::string_view, 2> names{
        "binary_root", "result_root"};
    for (std::size_t field_index = 0U; field_index < names.size();
         ++field_index) {
        RootFieldShape14& field = result.fields[field_index];
        std::size_t occurrences = 0U;
        if (result.top_level_object) {
            for (const auto& item : root.object) {
                if (item.first == names[field_index]) {
                    ++occurrences;
                    if (field.node == nullptr) field.node = &item.second;
                }
            }
        }
        field.occurrence_exact = occurrences == 1U;
        field.string_type = field.node != nullptr
            && field.node->type == Json::Type::String;
        if (field.string_type) field.value = field.node->text;
        field.length_exact = field.string_type && field.value.size() == 64U;
        for (std::size_t index = 0U; index < field.lowercase_hex.size();
             ++index) {
            const unsigned char item = field.length_exact
                ? static_cast<unsigned char>(field.value[index]) : 0xffU;
            field.lowercase_hex[index] = (item >= '0' && item <= '9')
                || (item >= 'a' && item <= 'f');
        }
    }
    result.exact = result.top_level_object;
    for (const RootFieldShape14& field : result.fields) {
        result.exact = field.occurrence_exact && field.string_type
            && field.length_exact && result.exact;
        for (const bool character : field.lowercase_hex) {
            result.exact = character && result.exact;
        }
    }
    return result;
}

RootShape14 inspect_expected_root_shape14(const Json& root) {
    RootShape14 result;
    result.top_level_object = root.type == Json::Type::Object;
    constexpr std::array<std::string_view, 2> required{
        "binary_root", "result_root"};
    std::array<std::size_t, 2> occurrences{};
    if (result.top_level_object) {
        for (const auto& member : root.object) {
            for (std::size_t index = 0U; index < required.size(); ++index) {
                if (member.first != required[index]) continue;
                ++occurrences[index];
                if (result.fields[index].node == nullptr) {
                    result.fields[index].node = &member.second;
                }
            }
        }
    }
    result.exact = result.top_level_object;
    for (std::size_t field_index = 0U; field_index < required.size();
         ++field_index) {
        RootFieldShape14& field = result.fields[field_index];
        field.occurrence_exact = occurrences[field_index] == 1U;
        field.string_type = field.node != nullptr
            && field.node->type == Json::Type::String;
        field.value = field.string_type ? field.node->text : std::string{};
        field.length_exact = field.value.size() == 64U;
        for (std::size_t character = 0U;
             character < field.lowercase_hex.size(); ++character) {
            const unsigned char byte = field.length_exact
                ? static_cast<unsigned char>(field.value[character]) : 0xffU;
            field.lowercase_hex[character] =
                (byte >= '0' && byte <= '9')
                || (byte >= 'a' && byte <= 'f');
            result.exact = field.lowercase_hex[character] && result.exact;
        }
        result.exact = field.occurrence_exact && field.string_type
            && field.length_exact && result.exact;
    }
    return result;
}

bool count_root_shape14(const RootShape14& shape, Work14& work) {
    bool exact = true;
    combine_predicate(exact, work, shape.top_level_object);
    for (const RootFieldShape14& field : shape.fields) {
        combine_predicate(exact, work, field.occurrence_exact);
        combine_predicate(exact, work, field.string_type);
        combine_predicate(exact, work, field.length_exact);
        for (const bool character : field.lowercase_hex) {
            combine_predicate(exact, work, character);
        }
    }
    return exact;
}

struct ParentResultPayload14 {
    std::string contract_root;
    std::string source_root;
    std::string source_commit;
    std::string source_tree;
    std::string compiler_flags;
    std::string profile_root;
    std::string phase_a_input_root;
    std::string phase_b_input_root;
    std::string status;
    std::string legacy_witness_result_root;
    std::string controls_result_root;
    std::string phase_a_result_root;
    std::string phase_a_permuted_result_root;
    std::string phase_b_trajectory_root;
    std::string phase_b_permuted_trajectory_root;
};

struct ParentControlReceipt14 {
    std::string number;
    std::string status;
    bool hash_accounting_exact = false;
    std::uint64_t expected_hash_derivations = 0U;
    std::uint64_t actual_hash_derivations = 0U;
};

struct ParentRetainedStep14 {
    std::string step_root;
    bool committed = false;
    std::string velocity_root;
    std::vector<std::string> round_roots;
};

struct ParentRetainedTrajectory14 {
    std::string raw_json;
    std::string failure;
    std::uint64_t accepted_steps = 0U;
    std::vector<ParentRetainedStep14> steps;
    std::string state_root;
    std::string failing_trial_state_root;
    bool apparatus_valid = false;
    bool physical_pass = false;
    std::uint64_t failure_step = 0U;
    long double maximum_position_rms = 0.0L;
    long double maximum_position = 0.0L;
    long double maximum_velocity_rms = 0.0L;
    long double maximum_speed = 0.0L;
    long double energy_excess = 0.0L;
    long double momentum_residual = 0.0L;
    long double failing_trial_position_rms = 0.0L;
    long double failing_trial_position_maximum = 0.0L;
    long double failing_trial_velocity_rms = 0.0L;
    long double failing_trial_maximum_speed = 0.0L;
    long double failing_trial_energy_excess = 0.0L;
    long double failing_trial_momentum_residual = 0.0L;
    std::string work_root;
    std::string trajectory_root;
};

struct ValidatedParent14 {
    std::string schema;
    std::string raw_binary_root;
    std::string raw_result_root;
    ParentResultPayload14 payload;
    std::array<Work, 5> child_work{};
    std::vector<ParentControlReceipt14> controls;
    std::array<ParentRetainedTrajectory14, 2> retained;
};

struct EmbeddedValidationTrace14 {
    bool child_binary_derived = false;
    bool raw_stdout_derived = false;
    bool raw_result_derived = false;
    bool normalized_stdout_derived = false;
    bool normalized_result_derived = false;
    bool json_syntax_valid = false;
    bool expected_json_syntax_valid = false;
    std::uint64_t json_syntax_predicates_completed = 0U;
    std::uint64_t expected_json_syntax_predicates = 0U;
    std::uint64_t validation_predicates_completed = 0U;
    std::array<std::uint8_t, 5> child_work_fields_decoded{};
    std::array<std::uint8_t, 5> child_work_fields_compared{};
    std::array<bool, 5> child_aggregated{};
};

class ParentValidator14 {
public:
    ParentValidator14(Work14& work, EmbeddedValidationTrace14& trace)
        : work_(work), trace_(trace) {}

    bool valid() const { return valid_; }

    bool require(bool predicate) {
        const bool evaluated = counted_predicate(work_, predicate);
        ++trace_.validation_predicates_completed;
        valid_ = evaluated && valid_;
        return evaluated;
    }

    bool observe(bool predicate) {
        static_cast<void>(counted_predicate(work_, predicate));
        ++trace_.validation_predicates_completed;
        return predicate;
    }

    const Json* node(const Json& root,
        std::initializer_list<std::string_view> keys) {
        const Json* current = &root;
        for (const std::string_view key : keys) {
            const bool object = require(
                current->type == Json::Type::Object);
            if (!object) return nullptr;
            const Json* found = nullptr;
            std::uint64_t occurrences = 0U;
            for (const auto& item : current->object) {
                const bool matches = observe(item.first == key);
                if (matches) {
                    if (found == nullptr) found = &item.second;
                    ++occurrences;
                }
            }
            const bool present = require(occurrences > 0U);
            const bool unique = require(occurrences == 1U);
            if (!present || !unique) return nullptr;
            current = found;
        }
        return current;
    }

    bool string_value(const Json& root,
        std::initializer_list<std::string_view> keys, std::string& output,
        const Json** raw_node = nullptr) {
        const Json* value = node(root, keys);
        if (value == nullptr) return false;
        const bool type = require(value->type == Json::Type::String);
        if (!type) return false;
        output = value->text;
        if (raw_node != nullptr) *raw_node = value;
        return true;
    }

    bool root_value(const Json& root,
        std::initializer_list<std::string_view> keys, std::string& output,
        const Json** raw_node = nullptr) {
        const Json* value = nullptr;
        if (!string_value(root, keys, output, &value)) return false;
        const bool length = require(output.size() == 64U);
        bool characters = true;
        for (const char item : output) {
            const bool character = observe((item >= '0' && item <= '9')
                || (item >= 'a' && item <= 'f'));
            characters = character && characters;
        }
        const bool character_set = require(characters);
        const bool span = require(value->content_end >= value->content_begin
            && value->content_end - value->content_begin == 64U);
        if (raw_node != nullptr) *raw_node = value;
        return length && character_set && span;
    }

    bool bool_value(const Json& root,
        std::initializer_list<std::string_view> keys, bool& output) {
        const Json* value = node(root, keys);
        if (value == nullptr) return false;
        const bool type = require(value->type == Json::Type::Boolean);
        if (!type) return false;
        output = value->boolean;
        return true;
    }

    bool u64_value(const Json& root,
        std::initializer_list<std::string_view> keys,
        std::uint64_t& output) {
        const Json* value = node(root, keys);
        if (value == nullptr) return false;
        const bool type = require(value->type == Json::Type::Number);
        if (!type) return false;
        const bool nonempty = require(!value->text.empty());
        bool digits = true;
        for (const char item : value->text) {
            const bool digit = observe(item >= '0' && item <= '9');
            digits = digit && digits;
        }
        const bool digit_set = require(digits);
        bool parsed = false;
        if (nonempty && digit_set) {
            try {
                output = std::stoull(value->text);
                parsed = true;
            } catch (const std::exception&) {
                parsed = false;
            }
        }
        const bool conversion = require(parsed);
        return nonempty && digit_set && conversion;
    }

    bool wide_value(const Json& root,
        std::initializer_list<std::string_view> keys, long double& output) {
        const Json* value = node(root, keys);
        if (value == nullptr) return false;
        const bool type = require(value->type == Json::Type::Number);
        if (!type) return false;
        bool parsed = false;
        try {
            std::size_t consumed = 0U;
            output = std::stold(value->text, &consumed);
            parsed = consumed == value->text.size();
        } catch (const std::exception&) {
            parsed = false;
        }
        const bool conversion = require(parsed);
        const bool finite = require(parsed && std::isfinite(output));
        return conversion && finite;
    }

    const Json* object_value(const Json& root,
        std::initializer_list<std::string_view> keys) {
        const Json* value = node(root, keys);
        if (value == nullptr) return nullptr;
        return require(value->type == Json::Type::Object) ? value : nullptr;
    }

    const Json* array_value(const Json& root,
        std::initializer_list<std::string_view> keys) {
        const Json* value = node(root, keys);
        if (value == nullptr) return nullptr;
        return require(value->type == Json::Type::Array) ? value : nullptr;
    }

    bool object_record(const Json& value) {
        return require(value.type == Json::Type::Object);
    }

    bool string_record(const Json& value, std::string& output,
        bool root = false) {
        const bool type = require(value.type == Json::Type::String);
        if (!type) return false;
        output = value.text;
        if (!root) return true;
        const bool length = require(output.size() == 64U);
        bool characters = true;
        for (const char item : output) {
            const bool character = observe((item >= '0' && item <= '9')
                || (item >= 'a' && item <= 'f'));
            characters = character && characters;
        }
        const bool character_set = require(characters);
        return length && character_set;
    }

    bool boolean_record(const Json& value, bool& output) {
        const bool type = require(value.type == Json::Type::Boolean);
        if (!type) return false;
        output = value.boolean;
        return true;
    }

private:
    Work14& work_;
    EmbeddedValidationTrace14& trace_;
    bool valid_ = true;
};

Work make_parent_work(const std::array<std::uint64_t, 20>& values) {
    Work result;
    result.graph_builds = values[0];
    result.graph_candidates = values[1];
    result.accepted_pairs = values[2];
    result.density_pairs = values[3];
    result.derivative_pairs = values[4];
    result.matrix_products = values[5];
    result.qp_sweeps = values[6];
    result.qp_updates = values[7];
    result.gradient_recomputations = values[8];
    result.finite_difference_candidates = values[9];
    result.independent_density_candidates = values[10];
    result.plane_tests = values[11];
    result.plane_hits = values[12];
    result.contact_projections = values[13];
    result.projection_rounds = values[14];
    result.analytic_jv_multiply_adds = values[15];
    result.topology_distance_tests = values[16];
    result.topology_discoveries = values[17];
    result.hash_derivations = values[18];
    result.penalty_work = values[19];
    return result;
}

std::array<Work, 5> frozen_parent_child_work14() {
    const Work controls = make_parent_work({7U, 2937343U, 198959U,
        198953U, 192980U, 24058492U, 2300U, 157294U, 2303U,
        133840896U, 66920448U, 15522U, 277U, 273U, 0U, 3145728U,
        0U, 0U, 71U, 0U});
    const Work phase_a = make_parent_work({6U, 2986201U, 197028U,
        196517U, 78321U, 12672656U, 1462U, 86585U, 1464U,
        89227264U, 66920448U, 9216U, 192U, 192U, 2U, 1572864U,
        48866U, 512U, 31U, 0U});
    const Work phase_b = make_parent_work({8U, 689590U, 43064U, 42810U,
        14014U, 1010676U, 2U, 0U, 4U, 0U, 22110208U, 3072U, 32U,
        32U, 2U, 0U, 6628U, 256U, 39U, 0U});
    return {controls, phase_a, phase_a, phase_b, phase_b};
}

bool decode_parent_work14(ParentValidator14& validator,
    EmbeddedValidationTrace14& trace, const Json& json,
    std::size_t child_index, Work& output) {
    constexpr std::array<std::string_view, 20> keys{
        "graph_builds", "graph_candidates", "accepted_pairs",
        "density_pairs", "derivative_pairs", "matrix_products",
        "qp_sweeps", "qp_updates", "gradient_recomputations",
        "finite_difference_candidates", "independent_density_candidates",
        "plane_tests", "plane_hits", "contact_projections",
        "projection_rounds", "analytic_jv_multiply_adds",
        "topology_distance_tests", "topology_discoveries",
        "hash_derivations", "penalty_work"};
    constexpr std::array<std::uint64_t Work::*, 20> members{
        &Work::graph_builds, &Work::graph_candidates, &Work::accepted_pairs,
        &Work::density_pairs, &Work::derivative_pairs,
        &Work::matrix_products, &Work::qp_sweeps, &Work::qp_updates,
        &Work::gradient_recomputations, &Work::finite_difference_candidates,
        &Work::independent_density_candidates, &Work::plane_tests,
        &Work::plane_hits, &Work::contact_projections,
        &Work::projection_rounds, &Work::analytic_jv_multiply_adds,
        &Work::topology_distance_tests, &Work::topology_discoveries,
        &Work::hash_derivations, &Work::penalty_work};
    for (std::size_t index = 0U; index < keys.size(); ++index) {
        std::uint64_t value = 0U;
        if (!validator.u64_value(json, {keys[index]}, value)) return false;
        output.*members[index] = value;
        ++trace.child_work_fields_decoded[child_index];
    }
    return true;
}

bool validate_parent_payload14(ParentValidator14& validator,
    const Json& json, ParentResultPayload14& payload) {
    return validator.root_value(json, {"contract_root"},
               payload.contract_root)
        && validator.root_value(json, {"source_root"}, payload.source_root)
        && validator.string_value(json, {"source_commit"},
            payload.source_commit)
        && validator.string_value(json, {"source_tree"}, payload.source_tree)
        && validator.string_value(json, {"compiler_flags"},
            payload.compiler_flags)
        && validator.root_value(json, {"profile_root"}, payload.profile_root)
        && validator.root_value(json, {"phase_a_input_root"},
            payload.phase_a_input_root)
        && validator.root_value(json, {"phase_b_input_root"},
            payload.phase_b_input_root)
        && validator.string_value(json, {"status"}, payload.status)
        && validator.root_value(json, {"legacy_witness", "result_root"},
            payload.legacy_witness_result_root)
        && validator.root_value(json, {"controls", "result_root"},
            payload.controls_result_root)
        && validator.root_value(json, {"phase_a", "result_root"},
            payload.phase_a_result_root)
        && validator.root_value(json,
            {"phase_a_permuted", "result_root"},
            payload.phase_a_permuted_result_root)
        && validator.root_value(json, {"phase_b", "trajectory_root"},
            payload.phase_b_trajectory_root)
        && validator.root_value(json,
            {"phase_b_permuted", "trajectory_root"},
            payload.phase_b_permuted_trajectory_root);
}

bool validate_parent_controls14(ParentValidator14& validator,
    const Json& json, std::vector<ParentControlReceipt14>& output) {
    const Json* receipts = validator.array_value(
        json, {"controls", "receipts"});
    if (receipts == nullptr) return false;
    if (!validator.require(receipts->array.size() == 11U)) return false;
    output.clear();
    output.reserve(receipts->array.size());
    for (const Json& value : receipts->array) {
        if (!validator.object_record(value)) return false;
        ParentControlReceipt14 receipt;
        if (!validator.string_value(value, {"number"}, receipt.number)
            || !validator.string_value(value, {"status"}, receipt.status)
            || !validator.bool_value(value, {"hash_accounting_exact"},
                receipt.hash_accounting_exact)
            || !validator.u64_value(value,
                {"expected_hash_derivations"},
                receipt.expected_hash_derivations)
            || !validator.u64_value(value,
                {"actual_hash_derivations"},
                receipt.actual_hash_derivations)) {
            return false;
        }
        output.push_back(std::move(receipt));
    }
    return true;
}

bool validate_parent_trajectory14(ParentValidator14& validator,
    const Json& json, std::string_view raw_stdout,
    std::string_view member_name, ParentRetainedTrajectory14& output) {
    const Json* child = validator.object_value(json, {member_name});
    if (child == nullptr) return false;
    const bool span_order = validator.require(child->begin <= child->end);
    const bool span_bounds = validator.require(child->end <= raw_stdout.size());
    if (!span_order || !span_bounds) return false;
    output.raw_json = std::string(raw_stdout.substr(
        child->begin, child->end - child->begin));
    if (!validator.string_value(*child, {"failure"}, output.failure)
        || !validator.u64_value(*child, {"accepted_steps"},
            output.accepted_steps)) {
        return false;
    }
    const Json* step_roots = validator.array_value(*child, {"step_roots"});
    const Json* committed = validator.array_value(
        *child, {"trial_committed"});
    const Json* steps = validator.array_value(*child, {"steps"});
    if (step_roots == nullptr || committed == nullptr || steps == nullptr) {
        return false;
    }
    const bool roots_committed = validator.require(
        step_roots->array.size() == committed->array.size());
    const bool roots_steps = validator.require(
        step_roots->array.size() == steps->array.size());
    const bool bounded = validator.require(step_roots->array.size() <= 2U);
    const bool accepted_bounded = validator.require(
        output.accepted_steps <= step_roots->array.size());
    if (!roots_committed || !roots_steps || !bounded || !accepted_bounded) {
        return false;
    }
    output.steps.clear();
    output.steps.reserve(steps->array.size());
    for (std::size_t index = 0U; index < steps->array.size(); ++index) {
        ParentRetainedStep14 step;
        if (!validator.string_record(step_roots->array[index],
                step.step_root, true)
            || !validator.boolean_record(committed->array[index],
                step.committed)
            || !validator.object_record(steps->array[index])
            || !validator.root_value(steps->array[index],
                {"velocity_root"}, step.velocity_root)) {
            return false;
        }
        const Json* rounds = validator.array_value(
            steps->array[index], {"rounds"});
        if (rounds == nullptr) return false;
        step.round_roots.reserve(rounds->array.size());
        for (const Json& round : rounds->array) {
            if (!validator.object_record(round)) return false;
            std::string root;
            if (!validator.root_value(round, {"result_root"}, root)) {
                return false;
            }
            step.round_roots.push_back(std::move(root));
        }
        output.steps.push_back(std::move(step));
    }
    return validator.root_value(*child, {"state_root"}, output.state_root)
        && validator.root_value(*child, {"failing_trial_state_root"},
            output.failing_trial_state_root)
        && validator.bool_value(*child, {"apparatus_valid"},
            output.apparatus_valid)
        && validator.bool_value(*child, {"physical_pass"},
            output.physical_pass)
        && validator.u64_value(*child, {"failure_step"}, output.failure_step)
        && validator.wide_value(*child, {"maximum_position_rmse_m"},
            output.maximum_position_rms)
        && validator.wide_value(*child, {"maximum_position_m"},
            output.maximum_position)
        && validator.wide_value(*child, {"maximum_velocity_rms_mps"},
            output.maximum_velocity_rms)
        && validator.wide_value(*child, {"maximum_speed_mps"},
            output.maximum_speed)
        && validator.wide_value(*child, {"energy_positive_excess"},
            output.energy_excess)
        && validator.wide_value(*child, {"momentum_residual"},
            output.momentum_residual)
        && validator.wide_value(*child,
            {"failing_trial_position_rmse_m"},
            output.failing_trial_position_rms)
        && validator.wide_value(*child,
            {"failing_trial_position_maximum_m"},
            output.failing_trial_position_maximum)
        && validator.wide_value(*child,
            {"failing_trial_velocity_rms_mps"},
            output.failing_trial_velocity_rms)
        && validator.wide_value(*child,
            {"failing_trial_maximum_speed_mps"},
            output.failing_trial_maximum_speed)
        && validator.wide_value(*child,
            {"failing_trial_energy_positive_excess"},
            output.failing_trial_energy_excess)
        && validator.wide_value(*child,
            {"failing_trial_momentum_residual"},
            output.failing_trial_momentum_residual)
        && validator.root_value(*child, {"work_root"}, output.work_root)
        && validator.root_value(*child, {"trajectory_root"},
            output.trajectory_root);
}

std::string recompute_parent_result(const ParentResultPayload14& payload,
    std::string_view binary, Work14& work) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp13.result.v1");
    for (const std::string_view field : std::array<std::string_view, 16>{
             payload.contract_root, payload.source_root,
             payload.source_commit, payload.source_tree,
             payload.compiler_flags, binary, payload.profile_root,
             payload.phase_a_input_root, payload.phase_b_input_root,
             payload.status, payload.legacy_witness_result_root,
             payload.controls_result_root, payload.phase_a_result_root,
             payload.phase_a_permuted_result_root,
             payload.phase_b_trajectory_root,
             payload.phase_b_permuted_trajectory_root}) {
        put_string(bytes, field);
    }
    work.portable_content_fields_serialized += 17U;
    return root_of(std::move(bytes));
}

std::string expected_parent_result14(const ParentResultPayload14& payload,
    std::string_view binary, Work14& expected) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp13.result.v1");
    put_string(bytes, payload.contract_root);
    put_string(bytes, payload.source_root);
    put_string(bytes, payload.source_commit);
    put_string(bytes, payload.source_tree);
    put_string(bytes, payload.compiler_flags);
    put_string(bytes, binary);
    put_string(bytes, payload.profile_root);
    put_string(bytes, payload.phase_a_input_root);
    put_string(bytes, payload.phase_b_input_root);
    put_string(bytes, payload.status);
    put_string(bytes, payload.legacy_witness_result_root);
    put_string(bytes, payload.controls_result_root);
    put_string(bytes, payload.phase_a_result_root);
    put_string(bytes, payload.phase_a_permuted_result_root);
    put_string(bytes, payload.phase_b_trajectory_root);
    put_string(bytes, payload.phase_b_permuted_trajectory_root);
    expected.portable_content_fields_serialized += 17U;
    ++expected.parent.hash_derivations;
    return root_of(std::move(bytes));
}

struct BinaryRead {
    std::string root;
    std::uint64_t bytes = 0U;
};
BinaryRead read_binary() {
    std::ifstream stream("/proc/self/exe", std::ios::binary);
    if (!stream) throw std::runtime_error("outer binary open failed");
    std::ostringstream payload;
    payload << stream.rdbuf();
    if (!stream.good() && !stream.eof()) {
        throw std::runtime_error("outer binary read failed");
    }
    const std::string bytes = payload.str();
    return {digest(bytes), static_cast<std::uint64_t>(bytes.size())};
}

struct EmbeddedParent {
    bool pass = false;
    int exit_code = -1;
    std::string outcome = "NOT_RUN_BY_PRECEDENCE";
    std::string raw_stdout;
    std::string raw_stdout_root;
    bool raw_stdout_root_present = false;
    std::string raw_result_root;
    bool observed_raw_result_root_present = false;
    std::string recomputed_raw_result_root;
    bool recomputed_raw_result_root_present = false;
    std::string normalized_stdout_root;
    bool normalized_stdout_root_present = false;
    std::string recomputed_normalized_result_root;
    bool normalized_result_root_present = false;
    std::string inherited_parent_result_root;
    bool inherited_parent_result_root_present = false;
    Json json;
    ValidatedParent14 validated;
    EmbeddedValidationTrace14 validation_trace;
    Work14 work;
    Work14 expected_work;
    WorkSeal14 verifier;
    std::string result_root;
    std::uint8_t validation_stage = 0U;
};

class EmbeddedValidationFailure14 final : public std::runtime_error {
public:
    explicit EmbeddedValidationFailure14(const char* message)
        : std::runtime_error(message) {}
};

Work14 expected_embedded_work(std::string_view raw_stdout,
    std::uint64_t outer_binary_bytes, std::string_view outer_binary,
    bool capture_completed) {
    Work14 result;
    if (!capture_completed) return result;
    result.parent.hash_derivations = 2U;
    result.binary_file_reads = 1U;
    result.binary_bytes_hashed = outer_binary_bytes;
    result.portable_content_fields_serialized = 1U;
    result.lane_scalar_comparisons = raw_stdout.size() + 1U;
    const ExpectedJsonSyntax14 syntax = expected_json_syntax14(raw_stdout);
    if (!syntax.valid) return result;
    result.lane_scalar_comparisons += 135U;
    Work14 ignored_work;
    std::uint64_t ignored_count = 0U;
    Json expected_json = JsonParser(raw_stdout, ignored_work,
        ignored_count).parse();
    const RootShape14 shape = inspect_expected_root_shape14(expected_json);
    if (!shape.exact) return result;
    ++result.lane_scalar_comparisons;
    if (shape.fields[0].value != outer_binary) return result;
    ++result.parent.hash_derivations;
    ++result.portable_content_fields_serialized;
    ++result.lane_scalar_comparisons;
    std::string normalized(raw_stdout);
    const Json* const binary_node = shape.fields[0].node;
    const Json* const result_node = shape.fields[1].node;
    if (binary_node == nullptr || result_node == nullptr) return result;
    normalized.replace(binary_node->content_begin, 64U, kParentBinary);
    normalized.replace(result_node->content_begin, 64U, kParentResult);
    if (digest(normalized) != kParentStdout) return result;
    Work14 ignored_validation_work;
    EmbeddedValidationTrace14 ignored_trace;
    ParentValidator14 validator(ignored_validation_work, ignored_trace);
    ParentResultPayload14 payload;
    if (!validate_parent_payload14(validator, expected_json, payload)) {
        return result;
    }
    const std::string expected_raw = expected_parent_result14(
        payload, outer_binary, result);
    const std::string expected_normalized = expected_parent_result14(
        payload, kParentBinary, result);
    result.lane_scalar_comparisons += 2U;
    const bool expected_raw_exact =
        expected_raw == shape.fields[1].value;
    const bool expected_normalized_exact =
        expected_normalized == kParentResult;
    if (!expected_raw_exact || !expected_normalized_exact) return result;
    for (const Work& child : frozen_parent_child_work14()) {
        add_parent_work(result, child);
        ++result.receipt_children_aggregated;
    }
    return result;
}

void put_optional_root(std::string& bytes, bool present,
    std::string_view root) {
    put_u8(bytes, present ? 1U : 0U);
    put_string(bytes, present ? root : std::string_view{});
}

std::string seal_embedded_parent(EmbeddedParent& result) {
    if (result.verifier.expected_root.empty()) {
        result.verifier = verify_work14(result.work, result.expected_work);
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.embedded-envelope.v1");
    put_string(bytes, result.outcome);
    put_optional_root(bytes, result.raw_stdout_root_present,
        result.raw_stdout_root);
    put_optional_root(bytes, result.recomputed_raw_result_root_present,
        result.recomputed_raw_result_root);
    put_optional_root(bytes, result.normalized_stdout_root_present,
        result.normalized_stdout_root);
    put_optional_root(bytes, result.normalized_result_root_present,
        result.recomputed_normalized_result_root);
    put_optional_root(bytes, result.inherited_parent_result_root_present,
        result.inherited_parent_result_root);
    append_work_verifier(bytes, result.verifier);
    result.result_root = root_of(std::move(bytes));
    return result.result_root;
}

EmbeddedParent skipped_embedded_parent() {
    EmbeddedParent result;
    result.outcome = "NOT_RUN_BY_PRECEDENCE";
    return result;
}

EmbeddedParent run_embedded_parent(std::string_view outer_binary,
    std::uint64_t outer_binary_bytes) {
    EmbeddedParent result;
    result.outcome = "EMBEDDED_PARENT_INVALID";
    std::ostringstream capture;
    std::streambuf* const prior = std::cout.rdbuf(capture.rdbuf());
    try {
        result.exit_code = ::run();
        std::cout.rdbuf(prior);
    } catch (const std::exception&) {
        std::cout.rdbuf(prior);
        throw;
    } catch (...) {
        std::cout.rdbuf(prior);
        throw;
    }
    result.raw_stdout = capture.str();
    result.validation_stage = 1U;
    result.validation_trace.child_binary_derived = true;
    ++result.work.parent.hash_derivations;
    result.work.binary_file_reads = 1U;
    result.work.binary_bytes_hashed = outer_binary_bytes;
    result.raw_stdout_root = digest(result.raw_stdout);
    result.raw_stdout_root_present = true;
    result.validation_trace.raw_stdout_derived = true;
    ++result.work.parent.hash_derivations;
    ++result.work.portable_content_fields_serialized;
    const ExpectedJsonSyntax14 syntax =
        expected_json_syntax14(result.raw_stdout);
    result.validation_trace.expected_json_syntax_valid = syntax.valid;
    result.validation_trace.expected_json_syntax_predicates =
        result.raw_stdout.size() + 1U;
    ProductionJsonRecognizer14 production_recognizer;
    bool grammar_prefix = true;
    for (const unsigned char byte : result.raw_stdout) {
        combine_predicate(grammar_prefix, result.work,
            production_recognizer.consume(byte));
    }
    const bool grammar_complete = production_recognizer.finish();
    combine_predicate(grammar_prefix, result.work, grammar_complete);
    result.validation_trace.json_syntax_predicates_completed =
        result.raw_stdout.size() + 1U;
    try {
        if (!grammar_prefix) {
            throw EmbeddedValidationFailure14(
                "embedded strict JSON invalid");
        }
        Work14 ignored_parser_work;
        std::uint64_t ignored_parser_predicates = 0U;
        result.json = JsonParser(result.raw_stdout, ignored_parser_work,
            ignored_parser_predicates).parse();
        result.validation_trace.json_syntax_valid = true;
        result.validation_stage = 2U;
        const RootShape14 shape = inspect_root_shape14(result.json);
        const bool shape_exact = count_root_shape14(shape, result.work);
        result.validation_trace.validation_predicates_completed = 135U;
        if (!shape_exact) {
            throw EmbeddedValidationFailure14(
                "embedded top roots invalid");
        }
        const Json* const raw_binary_node = shape.fields[0].node;
        const Json* const raw_result_node = shape.fields[1].node;
        result.validated.raw_binary_root = shape.fields[0].value;
        result.validated.raw_result_root = shape.fields[1].value;
        result.raw_result_root = result.validated.raw_result_root;
        result.observed_raw_result_root_present = true;
        const bool raw_binary_exact = counted_predicate(result.work,
            result.validated.raw_binary_root == outer_binary);
        if (!raw_binary_exact) {
            throw EmbeddedValidationFailure14(
                "embedded binary root mismatch");
        }
        if (raw_binary_node == nullptr || raw_result_node == nullptr) {
            throw EmbeddedValidationFailure14(
                "embedded root spans missing");
        }
        std::string normalized = result.raw_stdout;
        normalized.replace(raw_binary_node->content_begin, 64U, kParentBinary);
        normalized.replace(raw_result_node->content_begin, 64U, kParentResult);
        result.normalized_stdout_root = digest(normalized);
        result.normalized_stdout_root_present = true;
        result.validation_trace.normalized_stdout_derived = true;
        ++result.work.parent.hash_derivations;
        ++result.work.portable_content_fields_serialized;
        const bool normalized_stdout_exact = counted_predicate(result.work,
            result.normalized_stdout_root == kParentStdout);
        if (!normalized_stdout_exact) {
            throw EmbeddedValidationFailure14(
                "embedded normalized stdout mismatch");
        }
        Work14 ignored_validation_work;
        EmbeddedValidationTrace14 ignored_trace;
        ParentValidator14 validator(ignored_validation_work, ignored_trace);
        if (!validator.string_value(result.json, {"schema"},
                result.validated.schema)
            || !validate_parent_payload14(validator, result.json,
                result.validated.payload)) {
            throw EmbeddedValidationFailure14(
                "embedded result payload invalid");
        }
        const std::array<std::string_view, 5> child_names{
            "controls", "phase_a", "phase_a_permuted", "phase_b",
            "phase_b_permuted"};
        for (std::size_t index = 0U; index < child_names.size(); ++index) {
            const Json* work = validator.object_value(
                result.json, {child_names[index], "work"});
            if (work == nullptr || !decode_parent_work14(validator,
                    result.validation_trace, *work, index,
                    result.validated.child_work[index])) {
                throw EmbeddedValidationFailure14(
                    "embedded child work invalid");
            }
            add_parent_work(result.work, result.validated.child_work[index]);
            ++result.work.receipt_children_aggregated;
            result.validation_trace.child_aggregated[index] = true;
        }
        if (!validate_parent_controls14(validator, result.json,
                result.validated.controls)
            || !validate_parent_trajectory14(validator, result.json,
                result.raw_stdout, "phase_b",
                result.validated.retained[0])
            || !validate_parent_trajectory14(validator, result.json,
                result.raw_stdout, "phase_b_permuted",
                result.validated.retained[1])) {
            throw EmbeddedValidationFailure14(
                "embedded retained evidence invalid");
        }
        result.recomputed_raw_result_root = recompute_parent_result(
            result.validated.payload, outer_binary, result.work);
        ++result.work.parent.hash_derivations;
        result.validation_trace.raw_result_derived = true;
        result.recomputed_raw_result_root_present = true;
        result.recomputed_normalized_result_root = recompute_parent_result(
            result.validated.payload, kParentBinary, result.work);
        ++result.work.parent.hash_derivations;
        result.validation_trace.normalized_result_derived = true;
        result.normalized_result_root_present = true;
        const bool raw_result_exact = counted_predicate(result.work,
            result.recomputed_raw_result_root == result.raw_result_root);
        const bool normalized_result_exact = counted_predicate(result.work,
            result.recomputed_normalized_result_root == kParentResult);
        result.validation_stage = 3U;
        const bool valid = raw_result_exact && normalized_result_exact
            && result.exit_code == 0
            && result.validated.schema
                == "nextengine.nonlocal.ncgp13_pressure_contact.v1"
            && result.validated.payload.status
                == "PRESSURE_CONTACT_TRAJECTORY_REFUTED"
            && result.validated.payload.contract_root == kParentContract
            && result.validated.payload.source_root == kParentSourceAggregate
            && result.validated.payload.source_commit == kParentCommit
            && result.validated.payload.source_tree == kParentTree;
        result.inherited_parent_result_root =
            result.recomputed_normalized_result_root;
        result.inherited_parent_result_root_present = valid;
        result.pass = valid && validator.valid();
    } catch (const EmbeddedValidationFailure14&) {
        result.pass = false;
    }
    result.expected_work = expected_embedded_work(result.raw_stdout,
        outer_binary_bytes, outer_binary, true);
    result.verifier = verify_work14(result.work, result.expected_work);
    result.pass = result.pass && result.verifier.exact;
    result.outcome = result.pass ? "PASS" : "EMBEDDED_PARENT_INVALID";
    if (!result.pass) result.inherited_parent_result_root_present = false;
    seal_embedded_parent(result);
    return result;
}

struct Fixture14 {
    std::string name;
    NonlocalGpuProfile profile;
    std::vector<NonlocalGpuSample> canonical_samples;
    std::vector<NonlocalGpuSample> permuted_samples;
    std::vector<NonlocalGpuGhost> ghosts;
    std::string profile_root;
    std::string fixture_root;
    std::string legacy_input_root;
};

struct LanePolicy {
    std::string name;
    std::string root;
    const Fixture14* fixture = nullptr;
    std::uint32_t maximum_steps = 2U;
    std::uint32_t projection_cap = 8U;
    std::uint64_t qp_cap = 4096U;
    bool tight = false;
    bool retained = false;
    bool disable_xy_contact = false;
};

NonlocalGpuProfile integrated_profile(bool tight) {
    NonlocalGpuProfile profile = nonlocal_water_corrected_profile();
    profile.lambda = 0.0;
    profile.mu = 0.0;
    profile.gamma = 0.0;
    if (tight) profile.basin_extent = {0.2, 0.2, 0.6};
    return profile;
}

NonlocalGpuProfile census_profile(bool tight) {
    NonlocalGpuProfile profile = integrated_profile(tight);
    profile.gamma = 0.010664424039285813;
    return profile;
}

std::string profile14_root(const NonlocalGpuProfile& profile, Work14& work) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.profile.v1");
    put_string(bytes, profile.id);
    for (const double value : std::array<double, 19>{profile.dt,
             profile.spacing, profile.horizon, profile.mass,
             profile.rest_density, profile.kernel_scale, profile.kappa,
             profile.lambda, profile.mu, profile.gamma, profile.gravity.x,
             profile.gravity.y, profile.gravity.z, profile.basin_extent.x,
             profile.basin_extent.y, profile.basin_extent.z,
             static_cast<double>(profile.ghost_layers),
             static_cast<double>(profile.maximum_dynamic_samples),
             static_cast<double>(profile.maximum_neighbors)}) {
        put_f64(bytes, value);
    }
    work.portable_content_fields_serialized += 21U;
    ++work.parent.hash_derivations;
    return root_of(std::move(bytes));
}

void append_sample_record(std::string& bytes,
    const NonlocalGpuSample& sample) {
    put_u32(bytes, sample.sample_id);
    for (const Vec3d value : std::array<Vec3d, 3>{sample.reference,
             sample.current, sample.velocity}) {
        put_f32(bytes, static_cast<float>(value.x));
        put_f32(bytes, static_cast<float>(value.y));
        put_f32(bytes, static_cast<float>(value.z));
    }
}
void append_ghost_record(std::string& bytes, const NonlocalGpuGhost& ghost) {
    put_u32(bytes, ghost.sample_id);
    put_f32(bytes, static_cast<float>(ghost.position.x));
    put_f32(bytes, static_cast<float>(ghost.position.y));
    put_f32(bytes, static_cast<float>(ghost.position.z));
}

std::string fixture14_root(std::string_view name, std::string_view profile_root,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts, Work14& work) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.fixture.v1");
    put_string(bytes, name);
    put_string(bytes, profile_root);
    put_u64(bytes, samples.size());
    for (const NonlocalGpuSample& sample : samples) {
        append_sample_record(bytes, sample);
    }
    put_u64(bytes, ghosts.size());
    for (const NonlocalGpuGhost& ghost : ghosts) append_ghost_record(bytes, ghost);
    work.portable_content_fields_serialized += 5U + 10U * samples.size()
        + 4U * ghosts.size();
    ++work.parent.hash_derivations;
    return root_of(std::move(bytes));
}

std::string raw_order_root(std::string_view name,
    std::string_view profile_root,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts, Work14& work) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.raw-order.v1");
    put_string(bytes, name);
    put_string(bytes, profile_root);
    put_u64(bytes, samples.size());
    for (const NonlocalGpuSample& sample : samples) {
        append_sample_record(bytes, sample);
        ++work.raw_order_records_hashed;
    }
    put_u64(bytes, ghosts.size());
    for (const NonlocalGpuGhost& ghost : ghosts) {
        append_ghost_record(bytes, ghost);
        ++work.raw_order_records_hashed;
    }
    work.portable_content_fields_serialized += 5U + 10U * samples.size()
        + 4U * ghosts.size();
    ++work.parent.hash_derivations;
    return root_of(std::move(bytes));
}

constexpr std::array<long double, 16> kTolerances{
    2.0e-7L, 2.0e-12L, 1.0e-8L, 1.0e-8L, 1.0e-10L, 1.0e-8L,
    1.0e-3L, 2.5e-4L, 0.0L, 1.0e-8L, 0.0025L, 0.005L,
    0.05L, 0.10L, 0.01L, 0.01L};

std::string lane14_root(std::string_view name, std::string_view fixture_root,
    std::uint32_t projection_cap, std::uint64_t qp_cap, Work14& work) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.lane.v1");
    put_string(bytes, name);
    put_string(bytes, fixture_root);
    put_u32(bytes, 2U);
    put_u32(bytes, projection_cap);
    put_u64(bytes, qp_cap);
    for (const long double value : kTolerances) put_f64(bytes, value);
    put_u32(bytes, 1U);
    work.portable_content_fields_serialized += 23U;
    ++work.parent.hash_derivations;
    return root_of(std::move(bytes));
}

std::vector<NonlocalGpuSample> permute_samples(
    const std::vector<NonlocalGpuSample>& canonical, Work14& owner) {
    std::vector<NonlocalGpuSample> result;
    result.reserve(canonical.size());
    std::vector<bool> seen(canonical.size(), false);
    bool bijection = true;
    for (std::size_t input = 0U; input < canonical.size(); ++input) {
        const std::size_t logical =
            (23449ULL * input + 7919ULL) % canonical.size();
        const bool unused = counted_predicate(owner, !seen[logical]);
        bijection = unused && bijection;
        if (unused) seen[logical] = true;
        result.push_back(canonical[logical]);
    }
    if (!bijection) throw std::logic_error("NCGP14 permutation invalid");
    return result;
}

std::vector<NonlocalGpuSample> tight_samples(Work14& work) {
    std::vector<NonlocalGpuSample> result;
    result.reserve(128U);
    for (std::size_t iz = 0U; iz < 8U; ++iz) {
        for (std::size_t iy = 0U; iy < 4U; ++iy) {
            for (std::size_t ix = 0U; ix < 4U; ++ix) {
                const std::size_t logical = ix + 4U * (iy + 4U * iz);
                const Vec3d raw{0.025 + 0.05 * static_cast<double>(ix),
                    0.025 + 0.05 * static_cast<double>(iy),
                    0.025 + 0.05 * static_cast<double>(iz)};
                const Vec3d position{
                    static_cast<double>(static_cast<float>(raw.x)),
                    static_cast<double>(static_cast<float>(raw.y)),
                    static_cast<double>(static_cast<float>(raw.z))};
                result.push_back({static_cast<std::uint32_t>(
                        1000U + 17U * logical), position, position, {}});
                ++work.fixture_lattice_sites_generated;
                ++work.records_canonicalized;
            }
        }
    }
    return result;
}

std::uint64_t extended_ghost_cells(const NonlocalGpuProfile& profile) {
    const auto cells = [&](double extent) {
        return static_cast<std::uint64_t>(std::llround(extent / profile.spacing)
            + 2 * static_cast<int>(profile.ghost_layers));
    };
    return cells(profile.basin_extent.x) * cells(profile.basin_extent.y)
        * cells(profile.basin_extent.z);
}

Fixture14 make_fixture(std::string name, NonlocalGpuProfile profile,
    std::uint32_t nx, std::uint32_t ny, std::uint32_t nz, bool tight,
    std::string profile_root, Work14& work) {
    Fixture14 fixture;
    fixture.name = std::move(name);
    fixture.profile = profile;
    if (tight) {
        fixture.canonical_samples = tight_samples(work);
    } else {
        fixture.canonical_samples = make_lattice_state(
            profile, nx, ny, nz, false, false);
        work.fixture_lattice_sites_generated += fixture.canonical_samples.size();
        work.records_canonicalized += fixture.canonical_samples.size();
    }
    fixture.permuted_samples = permute_samples(
        fixture.canonical_samples, work);
    fixture.ghosts = make_basin_ghosts(profile);
    work.ghost_cells_tested += extended_ghost_cells(profile);
    work.records_canonicalized += fixture.ghosts.size();
    fixture.profile_root = std::move(profile_root);
    fixture.fixture_root = fixture14_root(fixture.name, fixture.profile_root,
        fixture.canonical_samples, fixture.ghosts, work);
    if (fixture.name == "OPEN-128") fixture.legacy_input_root = kOpen128Input;
    if (fixture.name == "OPEN-512") fixture.legacy_input_root = kOpen512Input;
    return fixture;
}

enum class Admission14 {
    Ok, InvalidProfile, CapacityExceeded, DuplicateId, Nonfinite,
    NonBinary32, RowNeighborCapacityExceeded
};

struct AdmissionTrace14 {
    std::size_t dynamic_records = 0U;
    std::size_t ghost_records = 0U;
    std::vector<std::uint64_t> row_candidate_extents;
    std::vector<std::uint64_t> row_degrees;
    bool expected_profile = false;
    bool expected_ghost_count = false;
    bool record_scan_reached = false;
    bool row_scan_reached = false;
};

Work14 expected_admission_work(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts, bool check_rows,
    const NonlocalGpuProfile* expected_profile,
    std::size_t expected_ghost_count,
    std::size_t maximum_total_records = 100000U) {
    Work14 result;
    result.profile_fields_checked = 19U;
    result.lane_scalar_comparisons = 1U + 2U * 19U + 15U + 1U + 3U;
    const bool compare_profile = expected_profile != nullptr;
    const bool compare_ghost_count = expected_ghost_count
        != std::numeric_limits<std::size_t>::max();
    if (compare_profile) result.lane_scalar_comparisons += 19U;
    if (compare_ghost_count) ++result.lane_scalar_comparisons;

    const std::array<double, 19> fields{profile.dt, profile.spacing,
        profile.horizon, profile.mass, profile.rest_density,
        profile.kernel_scale, profile.kappa, profile.lambda, profile.mu,
        profile.gamma, profile.gravity.x, profile.gravity.y, profile.gravity.z,
        profile.basin_extent.x, profile.basin_extent.y, profile.basin_extent.z,
        static_cast<double>(profile.ghost_layers),
        static_cast<double>(profile.maximum_dynamic_samples),
        static_cast<double>(profile.maximum_neighbors)};
    bool invalid_profile = profile.id != "nonlocal-water-50k-v1";
    for (const double value : fields) {
        invalid_profile = !std::isfinite(value) || invalid_profile;
        invalid_profile = !std::isfinite(static_cast<float>(value))
            || invalid_profile;
    }
    if (compare_profile) {
        const std::array<double, 19> expected{expected_profile->dt,
            expected_profile->spacing, expected_profile->horizon,
            expected_profile->mass, expected_profile->rest_density,
            expected_profile->kernel_scale, expected_profile->kappa,
            expected_profile->lambda, expected_profile->mu,
            expected_profile->gamma, expected_profile->gravity.x,
            expected_profile->gravity.y, expected_profile->gravity.z,
            expected_profile->basin_extent.x,
            expected_profile->basin_extent.y,
            expected_profile->basin_extent.z,
            static_cast<double>(expected_profile->ghost_layers),
            static_cast<double>(expected_profile->maximum_dynamic_samples),
            static_cast<double>(expected_profile->maximum_neighbors)};
        for (std::size_t index = 0U; index < fields.size(); ++index) {
            invalid_profile = fields[index] != expected[index]
                || invalid_profile;
        }
    }
    if (compare_ghost_count) {
        invalid_profile = ghosts.size() != expected_ghost_count
            || invalid_profile;
    }
    for (const bool invalid : std::array<bool, 15>{profile.dt <= 0.0,
             profile.spacing <= 0.0, profile.horizon <= 0.0,
             profile.mass <= 0.0, profile.rest_density <= 0.0,
             profile.kernel_scale <= 0.0, profile.kappa < 0.0,
             profile.lambda != 0.0, profile.mu != 0.0,
             profile.gamma != 0.0, profile.ghost_layers != 3U,
             profile.maximum_dynamic_samples != kMaximumDynamicSamples,
             profile.maximum_neighbors != kMaximumNeighbors,
             profile.basin_extent.x <= 0.0, profile.basin_extent.y <= 0.0}) {
        invalid_profile = invalid || invalid_profile;
    }
    const bool capacity = samples.empty()
        || samples.size() > profile.maximum_dynamic_samples
        || samples.size() + ghosts.size() > maximum_total_records;
    if (invalid_profile || capacity) return result;

    result.dynamic_records_admitted = samples.size();
    result.ghost_records_admitted = ghosts.size();
    result.lane_scalar_comparisons += 7U * samples.size();
    if (!ghosts.empty()) {
        result.lane_scalar_comparisons +=
            3U + 4U * (ghosts.size() - 1U);
    }
    result.lane_scalar_comparisons += 3U;
    bool duplicate = false;
    bool nonfinite = false;
    bool nonbinary = false;
    std::set<std::uint32_t> ids;
    for (const NonlocalGpuSample& sample : samples) {
        duplicate = !ids.insert(sample.sample_id).second || duplicate;
        for (const Vec3d value : std::array<Vec3d, 3>{sample.reference,
                 sample.current, sample.velocity}) {
            nonfinite = !finite(widen(value)) || nonfinite;
            nonbinary = !binary32_exact(value) || nonbinary;
        }
    }
    std::uint32_t prior = 0U;
    for (std::size_t index = 0U; index < ghosts.size(); ++index) {
        const NonlocalGpuGhost& ghost = ghosts[index];
        duplicate = !ids.insert(ghost.sample_id).second || duplicate;
        if (index != 0U) duplicate = ghost.sample_id <= prior || duplicate;
        prior = ghost.sample_id;
        nonfinite = !finite(widen(ghost.position)) || nonfinite;
        nonbinary = !binary32_exact(ghost.position) || nonbinary;
    }
    if (duplicate || nonfinite || nonbinary || !check_rows) return result;

    ++result.parent.graph_builds;
    for (const NonlocalGpuSample& owner : samples) {
        std::size_t degree = 0U;
        auto inspect = [&](const Vec3l& position) {
            ++result.parent.graph_candidates;
            if (norm(widen(owner.current) - position) > profile.horizon) {
                return false;
            }
            ++degree;
            ++result.parent.accepted_pairs;
            return degree == profile.maximum_neighbors + 1U;
        };
        bool overflow = false;
        for (const NonlocalGpuSample& neighbor : samples) {
            if (inspect(widen(neighbor.current))) {
                overflow = true;
                break;
            }
        }
        if (!overflow) {
            for (const NonlocalGpuGhost& ghost : ghosts) {
                if (inspect(widen(ghost.position))) {
                    overflow = true;
                    break;
                }
            }
        }
        ++result.lane_scalar_comparisons;
        if (overflow) return result;
        ++result.row_degree_checks;
    }
    return result;
}
std::string_view admission_name(Admission14 value) {
    switch (value) {
    case Admission14::Ok: return "OK";
    case Admission14::InvalidProfile: return "INVALID_PROFILE";
    case Admission14::CapacityExceeded: return "CAPACITY_EXCEEDED";
    case Admission14::DuplicateId: return "DUPLICATE_ID";
    case Admission14::Nonfinite: return "NONFINITE";
    case Admission14::NonBinary32: return "NON_BINARY32";
    case Admission14::RowNeighborCapacityExceeded:
        return "ROW_NEIGHBOR_CAPACITY_EXCEEDED";
    }
    throw std::logic_error("unknown admission");
}

Admission14 admit14(const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts, Work14& work,
    bool check_rows = false,
    const NonlocalGpuProfile* expected_profile = nullptr,
    std::size_t expected_ghost_count = std::numeric_limits<std::size_t>::max(),
    std::size_t maximum_total_records = 100000U,
    AdmissionTrace14* trace = nullptr) {
    AdmissionTrace14 local_trace;
    AdmissionTrace14& observed = trace == nullptr ? local_trace : *trace;
    observed.expected_profile = expected_profile != nullptr;
    observed.expected_ghost_count =
        expected_ghost_count != std::numeric_limits<std::size_t>::max();
    bool invalid_profile = false;
    combine_failure_predicate(invalid_profile, work,
        profile.id != "nonlocal-water-50k-v1");
    const std::array<double, 19> fields{profile.dt, profile.spacing,
        profile.horizon, profile.mass, profile.rest_density,
        profile.kernel_scale, profile.kappa, profile.lambda, profile.mu,
        profile.gamma, profile.gravity.x, profile.gravity.y, profile.gravity.z,
        profile.basin_extent.x, profile.basin_extent.y, profile.basin_extent.z,
        static_cast<double>(profile.ghost_layers),
        static_cast<double>(profile.maximum_dynamic_samples),
        static_cast<double>(profile.maximum_neighbors)};
    for (std::size_t index = 0U; index < fields.size(); ++index) {
        ++work.profile_fields_checked;
        const double value = fields[index];
        combine_failure_predicate(invalid_profile, work,
            !std::isfinite(value));
        combine_failure_predicate(invalid_profile, work,
            !std::isfinite(static_cast<float>(value)));
    }
    if (expected_profile != nullptr) {
        const std::array<double, 19> expected{expected_profile->dt,
            expected_profile->spacing, expected_profile->horizon,
            expected_profile->mass, expected_profile->rest_density,
            expected_profile->kernel_scale, expected_profile->kappa,
            expected_profile->lambda, expected_profile->mu,
            expected_profile->gamma, expected_profile->gravity.x,
            expected_profile->gravity.y, expected_profile->gravity.z,
            expected_profile->basin_extent.x,
            expected_profile->basin_extent.y,
            expected_profile->basin_extent.z,
            static_cast<double>(expected_profile->ghost_layers),
            static_cast<double>(expected_profile->maximum_dynamic_samples),
            static_cast<double>(expected_profile->maximum_neighbors)};
        for (std::size_t index = 0U; index < fields.size(); ++index) {
            combine_failure_predicate(invalid_profile, work,
                fields[index] != expected[index]);
        }
    }
    if (expected_ghost_count != std::numeric_limits<std::size_t>::max()) {
        combine_failure_predicate(invalid_profile, work,
            ghosts.size() != expected_ghost_count);
    }
    for (const bool invalid : std::array<bool, 15>{profile.dt <= 0.0,
             profile.spacing <= 0.0, profile.horizon <= 0.0,
             profile.mass <= 0.0, profile.rest_density <= 0.0,
             profile.kernel_scale <= 0.0, profile.kappa < 0.0,
             profile.lambda != 0.0, profile.mu != 0.0,
             profile.gamma != 0.0, profile.ghost_layers != 3U,
             profile.maximum_dynamic_samples != kMaximumDynamicSamples,
             profile.maximum_neighbors != kMaximumNeighbors,
             profile.basin_extent.x <= 0.0, profile.basin_extent.y <= 0.0}) {
        combine_failure_predicate(invalid_profile, work, invalid);
    }
    const bool invalid_profile_result = counted_predicate(work,
        invalid_profile);
    const bool empty = counted_predicate(work, samples.empty());
    const bool dynamic_capacity = counted_predicate(work,
        samples.size() > profile.maximum_dynamic_samples);
    const bool total_capacity = counted_predicate(work,
        samples.size() + ghosts.size() > maximum_total_records);
    if (invalid_profile_result) return Admission14::InvalidProfile;
    if (empty || dynamic_capacity || total_capacity) {
        return Admission14::CapacityExceeded;
    }
    observed.record_scan_reached = true;
    observed.dynamic_records = samples.size();
    observed.ghost_records = ghosts.size();
    bool duplicate = false;
    bool nonfinite = false;
    bool nonbinary = false;
    std::set<std::uint32_t> ids;
    for (const NonlocalGpuSample& sample : samples) {
        ++work.dynamic_records_admitted;
        const bool duplicate_id = !ids.insert(sample.sample_id).second;
        combine_failure_predicate(duplicate, work, duplicate_id);
        for (const Vec3d value : std::array<Vec3d, 3>{sample.reference,
                 sample.current, sample.velocity}) {
            combine_failure_predicate(nonfinite, work,
                !finite(widen(value)));
            combine_failure_predicate(nonbinary, work,
                !binary32_exact(value));
        }
    }
    std::uint32_t prior = 0U;
    for (std::size_t index = 0U; index < ghosts.size(); ++index) {
        const NonlocalGpuGhost& ghost = ghosts[index];
        ++work.ghost_records_admitted;
        combine_failure_predicate(duplicate, work,
            !ids.insert(ghost.sample_id).second);
        if (index != 0U) {
            combine_failure_predicate(duplicate, work,
                ghost.sample_id <= prior);
        }
        prior = ghost.sample_id;
        combine_failure_predicate(nonfinite, work,
            !finite(widen(ghost.position)));
        combine_failure_predicate(nonbinary, work,
            !binary32_exact(ghost.position));
    }
    const bool duplicate_result = counted_predicate(work, duplicate);
    const bool nonfinite_result = counted_predicate(work, nonfinite);
    const bool nonbinary_result = counted_predicate(work, nonbinary);
    if (duplicate_result) return Admission14::DuplicateId;
    if (nonfinite_result) return Admission14::Nonfinite;
    if (nonbinary_result) return Admission14::NonBinary32;
    if (check_rows) {
        observed.row_scan_reached = true;
        ++work.parent.graph_builds;
        const long double horizon = profile.horizon;
        for (const NonlocalGpuSample& owner : samples) {
            std::size_t degree = 0U;
            for (const NonlocalGpuSample& neighbor : samples) {
                ++work.parent.graph_candidates;
                if (norm(widen(owner.current) - widen(neighbor.current))
                    <= horizon) {
                    ++degree;
                    ++work.parent.accepted_pairs;
                    if (degree == profile.maximum_neighbors + 1U) {
                        observed.row_candidate_extents.push_back(
                            work.parent.graph_candidates);
                        observed.row_degrees.push_back(degree);
                        static_cast<void>(counted_predicate(work, false));
                        return Admission14::RowNeighborCapacityExceeded;
                    }
                }
            }
            for (const NonlocalGpuGhost& ghost : ghosts) {
                ++work.parent.graph_candidates;
                if (norm(widen(owner.current) - widen(ghost.position))
                    <= horizon) {
                    ++degree;
                    ++work.parent.accepted_pairs;
                    if (degree == profile.maximum_neighbors + 1U) {
                        observed.row_candidate_extents.push_back(
                            work.parent.graph_candidates);
                        observed.row_degrees.push_back(degree);
                        static_cast<void>(counted_predicate(work, false));
                        return Admission14::RowNeighborCapacityExceeded;
                    }
                }
            }
            ++work.row_degree_checks;
            observed.row_candidate_extents.push_back(
                samples.size() + ghosts.size());
            observed.row_degrees.push_back(degree);
            static_cast<void>(counted_predicate(work, true));
        }
    }
    return Admission14::Ok;
}

struct AdmissionDecision14 {
    Admission14 outcome = Admission14::InvalidProfile;
    State accepted_state;
};

AdmissionDecision14 production_admission14(const State& accepted_state,
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuSample>& samples,
    const std::vector<NonlocalGpuGhost>& ghosts, Work14& work,
    bool check_rows, const NonlocalGpuProfile& expected_profile,
    std::size_t expected_ghost_count, std::size_t maximum_total_records,
    AdmissionTrace14& trace) {
    AdmissionDecision14 result;
    result.accepted_state = accepted_state;
    result.outcome = admit14(profile, samples, ghosts, work, check_rows,
        &expected_profile, expected_ghost_count, maximum_total_records,
        &trace);
    return result;
}

void require_identity(std::string_view actual, std::string_view expected,
    const char* label) {
    if (actual != expected) {
        throw std::runtime_error(std::string(label) + " root mismatch");
    }
}

struct FaceCounts {
    std::uint64_t dynamic_dynamic = 0U;
    std::uint64_t dynamic_ghost = 0U;
    std::uint64_t lateral_unique = 0U;
    std::array<std::uint64_t, 6> membership{};
    std::uint64_t maximum_row = 0U;
};

std::array<int, 3> ghost_cell(const NonlocalGpuProfile& profile,
    const NonlocalGpuGhost& ghost) {
    const auto index = [&](double value) {
        return static_cast<int>(std::llround(value / profile.spacing - 0.5));
    };
    return {index(ghost.position.x), index(ghost.position.y),
        index(ghost.position.z)};
}

std::array<bool, 6> ghost_faces(const NonlocalGpuProfile& profile,
    const NonlocalGpuGhost& ghost) {
    const auto cell = ghost_cell(profile, ghost);
    const int nx = static_cast<int>(std::llround(
        profile.basin_extent.x / profile.spacing));
    const int ny = static_cast<int>(std::llround(
        profile.basin_extent.y / profile.spacing));
    const int nz = static_cast<int>(std::llround(
        profile.basin_extent.z / profile.spacing));
    return {cell[0] < 0, cell[0] >= nx, cell[1] < 0, cell[1] >= ny,
        cell[2] < 0, cell[2] >= nz};
}

struct Graph14 {
    Graph graph;
    FaceCounts faces;
    bool capacity = false;
    Work14 extra;
    Work14 expected_extra;
};

struct GraphSchedule14 {
    Work14 work;
    bool capacity = false;
    std::uint64_t accepted = 0U;
};

GraphSchedule14 expected_graph_schedule14(
    const NonlocalGpuProfile& profile,
    const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts, const GhostGrid& grid,
    bool strict, bool seal_graph) {
    GraphSchedule14 result;
    ++result.work.parent.graph_builds;
    result.work.face_classifications = 6U * ghosts.size();
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        std::uint64_t degree = 0U;
        for (std::size_t neighbor = 0U; neighbor < positions.size();
             ++neighbor) {
            ++result.work.parent.graph_candidates;
            const long double radius = norm(
                positions[owner] - positions[neighbor]);
            const bool accepted = strict ? radius < profile.horizon
                                         : radius <= profile.horizon;
            if (!accepted) continue;
            ++degree;
            ++result.accepted;
            ++result.work.parent.accepted_pairs;
            if (degree == profile.maximum_neighbors + 1U) {
                ++result.work.lane_scalar_comparisons;
                result.capacity = true;
                return result;
            }
        }
        const std::vector<std::size_t> nearby = nearby_ghosts(
            grid, positions[owner]);
        for (const std::size_t ghost : nearby) {
            ++result.work.parent.graph_candidates;
            ++result.work.ghost_pair_distance_tests;
            const long double radius = norm(
                positions[owner] - widen(ghosts[ghost].position));
            const bool accepted = strict ? radius < profile.horizon
                                         : radius <= profile.horizon;
            if (!accepted) continue;
            ++degree;
            ++result.accepted;
            ++result.work.parent.accepted_pairs;
            ++result.work.ghost_pairs_accepted;
            if (degree == profile.maximum_neighbors + 1U) {
                ++result.work.lane_scalar_comparisons;
                result.capacity = true;
                return result;
            }
        }
        ++result.work.row_degree_checks;
        ++result.work.lane_scalar_comparisons;
    }
    if (seal_graph) {
        ++result.work.parent.hash_derivations;
        result.work.portable_content_fields_serialized = 2U
            + 2U * positions.size() + 2U * result.accepted;
    }
    return result;
}

Graph14 build_graph14(const NonlocalGpuProfile& profile,
    const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts, const GhostGrid& grid,
    bool strict, bool seal_graph = true,
    const std::vector<NonlocalGpuGhost>* face_identity = nullptr) {
    Graph14 result;
    result.graph.row.resize(positions.size());
    result.graph.valid = true;
    Work& work = result.extra.parent;
    ++work.graph_builds;
    std::vector<std::array<bool, 6>> classified_faces;
    classified_faces.reserve(ghosts.size());
    for (std::size_t ghost_index = 0U; ghost_index < ghosts.size();
         ++ghost_index) {
        const NonlocalGpuGhost& classified = face_identity == nullptr
            ? ghosts[ghost_index] : (*face_identity)[ghost_index];
        classified_faces.push_back(ghost_faces(profile, classified));
        result.extra.face_classifications += 6U;
    }
    bool stop = false;
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        for (std::size_t neighbor = 0U; neighbor < positions.size(); ++neighbor) {
            ++work.graph_candidates;
            const long double radius = norm(positions[owner] - positions[neighbor]);
            if (strict ? radius < profile.horizon : radius <= profile.horizon) {
                result.graph.row[owner].push_back({neighbor, false});
                ++work.accepted_pairs;
                ++result.faces.dynamic_dynamic;
                if (result.graph.row[owner].size()
                    == profile.maximum_neighbors + 1U) {
                    static_cast<void>(counted_predicate(
                        result.extra, false));
                    result.capacity = true;
                    result.graph.valid = false;
                    stop = true;
                    break;
                }
            }
        }
        if (stop) break;
        const std::vector<std::size_t> nearby = nearby_ghosts(
            grid, positions[owner]);
        for (const std::size_t ghost : nearby) {
            ++work.graph_candidates;
            ++result.extra.ghost_pair_distance_tests;
            const long double radius = norm(
                positions[owner] - widen(ghosts[ghost].position));
            if (strict ? radius < profile.horizon : radius <= profile.horizon) {
                result.graph.row[owner].push_back({ghost, true});
                ++work.accepted_pairs;
                ++result.extra.ghost_pairs_accepted;
                ++result.faces.dynamic_ghost;
                const auto& faces = classified_faces[ghost];
                bool lateral = false;
                for (std::size_t face = 0U; face < faces.size(); ++face) {
                    if (!faces[face]) continue;
                    ++result.faces.membership[face];
                    lateral = lateral || face < 4U;
                }
                if (lateral) ++result.faces.lateral_unique;
                if (result.graph.row[owner].size()
                    == profile.maximum_neighbors + 1U) {
                    static_cast<void>(counted_predicate(
                        result.extra, false));
                    result.capacity = true;
                    result.graph.valid = false;
                    stop = true;
                    break;
                }
            }
        }
        if (stop) break;
        ++result.extra.row_degree_checks;
        static_cast<void>(counted_predicate(result.extra, true));
        result.faces.maximum_row = std::max(result.faces.maximum_row,
            static_cast<std::uint64_t>(result.graph.row[owner].size()));
    }
    std::uint64_t accepted_count = 0U;
    for (std::size_t owner = 0U; owner < result.graph.row.size(); ++owner) {
        accepted_count += result.graph.row[owner].size();
    }
    if (seal_graph && !result.capacity) {
        std::string bytes;
        put_string(bytes, strict ? "nextengine.nonlocal.ncgp13.graph.strict.v1"
                                 : "nextengine.nonlocal.ncgp13.graph.v1");
        put_u64(bytes, positions.size());
        for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
            put_u32(bytes, ids[owner]);
            put_u64(bytes, result.graph.row[owner].size());
            for (const Edge edge : result.graph.row[owner]) {
                put_u32(bytes, edge.ghost ? ghosts[edge.index].sample_id
                                          : ids[edge.index]);
                put_u32(bytes, edge.ghost ? 1U : 0U);
            }
        }
        result.graph.root = root_of(std::move(bytes));
        ++work.hash_derivations;
        result.extra.portable_content_fields_serialized += 2U
            + 2U * positions.size() + 2U * accepted_count;
    }
    const GraphSchedule14 expected = expected_graph_schedule14(
        profile, positions, ghosts, grid, strict, seal_graph);
    result.expected_extra = expected.work;
    if (expected.capacity != result.capacity) result.graph.valid = false;
    return result;
}

std::string seal_graph14_root(const std::vector<std::uint32_t>& ids,
    const std::vector<NonlocalGpuGhost>& ghosts, bool strict,
    Graph& graph, Work& parent_owner, Work14& portable_owner) {
    if (!graph.valid) {
        throw std::logic_error("cannot seal invalid NCGP14 graph");
    }
    std::string bytes;
    put_string(bytes, strict ? "nextengine.nonlocal.ncgp13.graph.strict.v1"
                             : "nextengine.nonlocal.ncgp13.graph.v1");
    put_u64(bytes, graph.row.size());
    std::uint64_t pair_count = 0U;
    for (std::size_t owner = 0U; owner < graph.row.size(); ++owner) {
        put_u32(bytes, ids[owner]);
        put_u64(bytes, graph.row[owner].size());
        pair_count += graph.row[owner].size();
        for (const Edge edge : graph.row[owner]) {
            put_u32(bytes, edge.ghost ? ghosts[edge.index].sample_id
                                      : ids[edge.index]);
            put_u32(bytes, edge.ghost ? 1U : 0U);
        }
    }
    ++parent_owner.hash_derivations;
    portable_owner.portable_content_fields_serialized +=
        2U + 2U * graph.row.size() + 2U * pair_count;
    graph.root = root_of(std::move(bytes));
    return graph.root;
}

struct Assembly14 {
    Assembly value;
    Graph graph;
    FaceCounts faces;
    bool capacity = false;
    bool nonfinite = false;
    Work expected_parent_work;
    Work14 extra;
    Work14 expected_extra;
};

Assembly14 assemble14(const NonlocalGpuProfile& profile,
    const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts, const GhostGrid& grid,
    bool include_ghost_derivative,
    const std::vector<long double>* external_displacement) {
    Assembly14 wrapped;
    Assembly& result = wrapped.value;
    result.count = positions.size();
    result.dofs = 3U * result.count;
    Graph14 built = build_graph14(
        profile, ids, positions, ghosts, grid, false, false);
    result.work = built.extra.parent;
    built.extra.parent = {};
    wrapped.extra = built.extra;
    wrapped.expected_extra = built.expected_extra;
    wrapped.expected_extra.parent = {};
    wrapped.graph = built.graph;
    wrapped.faces = built.faces;
    wrapped.capacity = built.capacity;
    result.valid = built.graph.valid;
    wrapped.expected_parent_work = built.expected_extra.parent;
    wrapped.expected_extra.parent = {};
    if (built.capacity || !built.graph.valid) return wrapped;

    result.input_root = position_state_root(ids, positions);
    ++result.work.hash_derivations;
    ++wrapped.expected_parent_work.hash_derivations;
    result.density.assign(result.count, 0.0L);
    result.constraint.assign(result.count, 0.0L);
    result.jacobian.assign(result.count * result.dofs, 0.0L);
    result.matrix.assign(result.count * result.count, 0.0L);
    result.rhs.assign(result.count, 0.0L);
    const long double mass = profile.mass;
    const long double rho0 = profile.rest_density;
    for (std::size_t owner = 0U; owner < result.count; ++owner) {
        for (const Edge edge : built.graph.row[owner]) {
            ++result.work.density_pairs;
            const Vec3l neighbor = edge.ghost
                ? widen(ghosts[edge.index].position) : positions[edge.index];
            const Vec3l difference = positions[owner] - neighbor;
            const long double radius = norm(difference);
            const Kernel values = kernel(radius, profile);
            result.density[owner] += mass * values.value;
            if (!(radius > 0.0L) || (edge.ghost && !include_ghost_derivative)
                || (!edge.ghost && edge.index == owner)) continue;
            ++result.work.derivative_pairs;
            const Vec3l derivative = difference
                * (mass / rho0 * values.first / radius);
            add_jacobian(result.jacobian, result.dofs, owner, owner,
                derivative, 1.0L);
            if (!edge.ghost) {
                add_jacobian(result.jacobian, result.dofs, owner, edge.index,
                    derivative, -1.0L);
            }
        }
        result.constraint[owner] = result.density[owner] / rho0 - 1.0L;
    }
    const long double scale = static_cast<long double>(profile.dt) * profile.dt
        / profile.mass;
    for (std::size_t column = 0U; column < result.dofs; ++column) {
        std::vector<std::pair<std::size_t, long double>> entries;
        for (std::size_t row = 0U; row < result.count; ++row) {
            const long double value = result.jacobian[row * result.dofs + column];
            if (value != 0.0L) entries.emplace_back(row, value);
        }
        for (const auto& lhs : entries) {
            for (const auto& rhs : entries) {
                result.matrix[lhs.first * result.count + rhs.first] +=
                    scale * lhs.second * rhs.second;
                ++result.work.matrix_products;
            }
        }
    }
    for (std::size_t row = 0U; row < result.count; ++row) {
        result.rhs[row] = result.constraint[row];
        if (external_displacement != nullptr) {
            for (std::size_t column = 0U; column < result.dofs; ++column) {
                result.rhs[row] += result.jacobian[row * result.dofs + column]
                    * (*external_displacement)[column];
            }
        }
    }
    long double difference2 = 0.0L;
    long double reference2 = 0.0L;
    bool finite_payload = true;
    for (std::size_t row = 0U; row < result.count; ++row) {
        const long double diagonal = result.matrix[row * result.count + row];
        finite_payload = finite_payload && std::isfinite(result.density[row])
            && std::isfinite(result.constraint[row])
            && std::isfinite(result.rhs[row]) && std::isfinite(diagonal)
            && diagonal > 0.0L;
        for (std::size_t column = 0U; column < result.dofs; ++column) {
            finite_payload = std::isfinite(
                result.jacobian[row * result.dofs + column])
                && finite_payload;
        }
        for (std::size_t column = 0U; column < result.count; ++column) {
            const long double lhs = result.matrix[row * result.count + column];
            const long double rhs = result.matrix[column * result.count + row];
            difference2 += (lhs - rhs) * (lhs - rhs);
            reference2 += lhs * lhs + rhs * rhs;
            finite_payload = std::isfinite(lhs) && finite_payload;
        }
    }
    result.symmetry = std::sqrt(difference2)
        / std::max(std::sqrt(0.5L * reference2), 1.0e-30L);
    finite_payload = std::isfinite(result.symmetry) && finite_payload;
    result.valid = finite_payload;
    wrapped.nonfinite = !finite_payload;

    wrapped.expected_parent_work.density_pairs = 0U;
    wrapped.expected_parent_work.derivative_pairs = 0U;
    std::vector<long double> expected_jacobian(
        result.count * result.dofs, 0.0L);
    const long double expected_mass = profile.mass;
    const long double expected_rho0 = profile.rest_density;
    for (std::size_t owner = 0U; owner < result.count; ++owner) {
        wrapped.expected_parent_work.density_pairs +=
            built.graph.row[owner].size();
        for (const Edge edge : built.graph.row[owner]) {
            const Vec3l neighbor = edge.ghost
                ? widen(ghosts[edge.index].position) : positions[edge.index];
            const Vec3l difference = positions[owner] - neighbor;
            const long double radius = norm(difference);
            if (!(radius > 0.0L) || (edge.ghost && !include_ghost_derivative)
                || (!edge.ghost && edge.index == owner)) {
                continue;
            }
            ++wrapped.expected_parent_work.derivative_pairs;
            const Kernel values = kernel(radius, profile);
            const Vec3l derivative = difference
                * (expected_mass / expected_rho0
                    * values.first / radius);
            add_jacobian(expected_jacobian, result.dofs, owner, owner,
                derivative, 1.0L);
            if (!edge.ghost) {
                add_jacobian(expected_jacobian, result.dofs, owner,
                    edge.index, derivative, -1.0L);
            }
        }
    }
    wrapped.expected_parent_work.matrix_products = 0U;
    for (std::size_t column = 0U; column < result.dofs; ++column) {
        std::uint64_t nonzero_rows = 0U;
        for (std::size_t row = 0U; row < result.count; ++row) {
            if (expected_jacobian[row * result.dofs + column] != 0.0L) {
                ++nonzero_rows;
            }
        }
        wrapped.expected_parent_work.matrix_products +=
            nonzero_rows * nonzero_rows;
    }
    if (!finite_payload) return wrapped;

    result.graph_root = seal_graph14_root(ids, ghosts, false,
        wrapped.graph, result.work, wrapped.extra);
    ++wrapped.expected_parent_work.hash_derivations;
    std::uint64_t graph_pairs = 0U;
    for (const auto& row : built.graph.row) graph_pairs += row.size();
    wrapped.expected_extra.portable_content_fields_serialized +=
        2U + 2U * positions.size() + 2U * graph_pairs;
    result.density_root = id_scalar_root(
        "nextengine.nonlocal.ncgp13.density.v1", ids, result.density);
    {
        std::string bytes;
        put_string(bytes, "nextengine.nonlocal.ncgp13.jacobian.v1");
        put_u64(bytes, result.count);
        for (std::size_t row = 0U; row < result.count; ++row) {
            put_u32(bytes, ids[row]);
            for (std::size_t column = 0U; column < result.dofs; ++column) {
                put_u32(bytes, ids[column / 3U]);
                put_u32(bytes, static_cast<std::uint32_t>(column % 3U));
                put_f64(bytes, result.jacobian[row * result.dofs + column]);
            }
        }
        result.jacobian_root = root_of(std::move(bytes));
    }
    {
        std::string bytes;
        put_string(bytes, "nextengine.nonlocal.ncgp13.matrix.v1");
        put_u64(bytes, result.count);
        for (std::size_t row = 0U; row < result.count; ++row) {
            put_u32(bytes, ids[row]);
            for (std::size_t column = 0U; column < result.count; ++column) {
                put_u32(bytes, ids[column]);
                put_f64(bytes, result.matrix[row * result.count + column]);
            }
        }
        result.matrix_root = root_of(std::move(bytes));
    }
    result.work.hash_derivations += 3U;
    wrapped.expected_parent_work.hash_derivations += 3U;
    if (result.work.hash_derivations != 5U) {
        result.valid = false;
    }
    return wrapped;
}

struct Density14 {
    DensityResult value;
    Graph graph;
    Work parent_work;
    Work expected_parent_work;
    Work14 extra;
    Work14 expected_extra;
    bool capacity = false;
    bool nonfinite = false;
};
Density14 candidate_density14(const NonlocalGpuProfile& profile,
    const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts, const GhostGrid& grid) {
    Density14 result;
    result.value.value.assign(positions.size(), 0.0L);
    Graph14 built = build_graph14(
        profile, ids, positions, ghosts, grid, false, false);
    result.parent_work = built.extra.parent;
    result.expected_parent_work = built.expected_extra.parent;
    built.extra.parent = {};
    built.expected_extra.parent = {};
    result.extra = built.extra;
    result.expected_extra = built.expected_extra;
    result.capacity = built.capacity;
    result.value.valid = built.graph.valid;
    result.graph = built.graph;
    if (built.capacity || !built.graph.valid) return result;
    result.value.graph_root = seal_graph14_root(ids, ghosts, false,
        result.graph, result.parent_work, result.extra);
    ++result.expected_parent_work.hash_derivations;
    std::uint64_t graph_pairs = 0U;
    for (const auto& row : result.graph.row) graph_pairs += row.size();
    result.expected_extra.portable_content_fields_serialized +=
        2U + 2U * positions.size() + 2U * graph_pairs;
    for (std::size_t owner = 0U; owner < positions.size(); ++owner) {
        result.expected_parent_work.density_pairs +=
            built.graph.row[owner].size();
        for (const Edge edge : built.graph.row[owner]) {
            ++result.parent_work.density_pairs;
            const Vec3l neighbor = edge.ghost
                ? widen(ghosts[edge.index].position) : positions[edge.index];
            result.value.value[owner] += profile.mass
                * kernel(norm(positions[owner] - neighbor), profile).value;
        }
        result.value.valid = result.value.valid
            && std::isfinite(result.value.value[owner]);
    }
    result.nonfinite = !result.value.valid;
    return result;
}

struct Solve14 {
    Solve value;
};
Solve14 solve_qp14(const Assembly& assembly, std::uint64_t maximum_sweeps,
    Work& work) {
    Solve14 result;
    Solve& solve = result.value;
    solve.lambda.assign(assembly.count, 0.0L);
    solve.gradient.resize(assembly.count);
    recompute_gradient(assembly, solve);
    ++work.gradient_recomputations;
    for (std::uint64_t sweep = 1U; sweep <= maximum_sweeps; ++sweep) {
        for (std::size_t row = 0U; row < assembly.count; ++row) {
            const long double diagonal = assembly.matrix[row * assembly.count + row];
            if (!(diagonal > 0.0L) || !std::isfinite(diagonal)) {
                solve.valid = false;
                work.qp_sweeps += solve.sweeps;
                work.qp_updates += solve.updates;
                return result;
            }
            const long double prior = solve.lambda[row];
            const long double next = std::max(0.0L,
                prior - solve.gradient[row] / diagonal);
            const long double delta = next - prior;
            if (delta == 0.0L) continue;
            solve.lambda[row] = next;
            ++solve.updates;
            for (std::size_t affected = 0U; affected < assembly.count;
                 ++affected) {
                solve.gradient[affected] +=
                    assembly.matrix[affected * assembly.count + row] * delta;
            }
        }
        solve.sweeps = static_cast<std::size_t>(sweep);
        recompute_gradient(assembly, solve);
        ++work.gradient_recomputations;
        residuals(assembly, solve);
        solve.valid = solve.valid && std::isfinite(solve.primal)
            && std::isfinite(solve.projected_kkt)
            && std::isfinite(solve.complementarity)
            && std::all_of(solve.lambda.begin(), solve.lambda.end(),
                [](long double value) {
                    return std::isfinite(value) && value >= 0.0L;
                });
        if (!solve.valid) break;
        if (solve.primal <= kPrimalLimit
            && solve.projected_kkt <= kKktLimit
            && solve.complementarity <= kComplementarityLimit) {
            solve.converged = true;
            break;
        }
    }
    work.qp_sweeps += solve.sweeps;
    work.qp_updates += solve.updates;
    return result;
}

struct RoundExtra14 {
    std::string lane_root;
    std::uint32_t trial_index = 0U;
    std::uint32_t round_index = 0U;
    std::string inherited_round_root;
    FaceCounts pre_faces;
    std::array<std::uint64_t, 6> first_hit_faces{};
    std::array<std::uint64_t, 6> clamp_faces{};
    bool fully_computed = false;
    bool finite_capacity_valid = false;
    bool assembly_valid = false;
    bool row_capacity_ok = false;
    bool jv_ok = false;
    bool symmetry_valid = false;
    bool qp_valid = false;
    bool qp_converged = false;
    bool contact_valid = false;
    bool candidate_density_valid = false;
    bool independent_density_valid = false;
    bool density_correspondence_valid = false;
    bool inset_ok = false;
    bool balance_ok = false;
    bool round_apparatus_ok = false;
    bool round_closed = false;
    bool qp_cap_exhausted = false;
    long double jv_relative_l2 = 0.0L;
    long double symmetry_relative_l2 = 0.0L;
    long double pressure_balance = 0.0L;
    long double density_correspondence = 0.0L;
    long double maximum_positive_strain = 0.0L;
    long double rms_positive_strain = 0.0L;
    long double penetration_m = 0.0L;
    std::uint64_t positive_multiplier_count = 0U;
    std::string root;
};

std::string seal_round_extra14(std::string_view lane_root,
    RoundExtra14& extra, Work14& owner) {
    if (!extra.fully_computed || extra.inherited_round_root.empty()) {
        throw std::logic_error("NCGP14 partial round-extra seal");
    }
    extra.lane_root = lane_root;
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.round-extra.v1");
    put_string(bytes, lane_root);
    put_u32(bytes, extra.trial_index);
    put_u32(bytes, extra.round_index);
    put_string(bytes, extra.inherited_round_root);
    for (const bool flag : std::array<bool, 14>{
             extra.finite_capacity_valid, extra.assembly_valid,
             extra.jv_ok, extra.symmetry_valid, extra.qp_valid,
             extra.qp_converged, extra.contact_valid,
             extra.candidate_density_valid, extra.independent_density_valid,
             extra.density_correspondence_valid, extra.balance_ok,
             extra.inset_ok, extra.round_apparatus_ok,
             extra.round_closed}) {
        put_u8(bytes, flag ? 1U : 0U);
    }
    for (const long double value : std::array<long double, 7>{
             extra.jv_relative_l2, extra.symmetry_relative_l2,
             extra.pressure_balance, extra.density_correspondence,
             extra.maximum_positive_strain, extra.rms_positive_strain,
             extra.penetration_m}) {
        put_f64(bytes, value);
    }
    put_u64(bytes, extra.positive_multiplier_count);
    for (const std::uint64_t value : extra.pre_faces.membership) {
        put_u64(bytes, value);
    }
    for (const std::uint64_t value : extra.first_hit_faces) {
        put_u64(bytes, value);
    }
    for (const std::uint64_t value : extra.clamp_faces) {
        put_u64(bytes, value);
    }
    put_u64(bytes, extra.pre_faces.maximum_row);
    owner.portable_content_fields_serialized += 46U;
    ++owner.parent.hash_derivations;
    extra.root = root_of(std::move(bytes));
    return extra.root;
}

struct ContactBatch14 {
    ContactBatch value;
    std::vector<long double> first_time;
};

ContactBatch14 contact_batch14(const NonlocalGpuProfile& profile,
    const std::vector<std::uint32_t>& ids,
    const std::vector<Vec3l>& start,
    const std::vector<Vec3l>& proposed, bool disable_xy) {
    if (ids.size() != start.size() || ids.size() != proposed.size()) {
        throw std::logic_error("NCGP14 contact size mismatch");
    }
    ContactBatch14 wrapped;
    ContactBatch& result = wrapped.value;
    result.endpoint.reserve(ids.size());
    result.correction.reserve(ids.size());
    result.impulse.reserve(ids.size());
    result.mask.reserve(ids.size());
    result.first_mask.reserve(ids.size());
    wrapped.first_time.reserve(ids.size());
    for (std::size_t index = 0U; index < ids.size(); ++index) {
        Contact contact = swept_contact(profile, start[index], proposed[index]);
        if (disable_xy) {
            contact.endpoint.x = proposed[index].x;
            contact.endpoint.y = proposed[index].y;
            contact.mask &= (1U << 4U) | (1U << 5U);
            contact.first_mask &= (1U << 4U) | (1U << 5U);
            contact.hits = ((contact.mask & (1U << 4U)) != 0U ? 1U : 0U)
                + ((contact.mask & (1U << 5U)) != 0U ? 1U : 0U);
            if (contact.first_mask == 0U) {
                contact.t_first =
                    std::numeric_limits<long double>::infinity();
            }
            contact.correction = contact.endpoint - proposed[index];
            contact.impulse = contact.correction
                * (static_cast<long double>(profile.mass) / profile.dt);
        }
        const bool finite_payload = finite(start[index])
            && finite(proposed[index]) && finite(contact.endpoint)
            && finite(contact.correction) && finite(contact.impulse)
            && (std::isfinite(contact.t_first)
                || contact.first_mask == 0U);
        result.valid = contact.valid && finite_payload && result.valid;
        result.endpoint.push_back(contact.endpoint);
        result.correction.push_back(contact.correction);
        result.impulse.push_back(contact.impulse);
        result.mask.push_back(contact.mask);
        result.first_mask.push_back(contact.first_mask);
        wrapped.first_time.push_back(contact.t_first);
        result.work.plane_tests += contact.tests;
        result.work.plane_hits += contact.hits;
        if (contact.mask != 0U) ++result.work.contact_projections;
        if ((contact.mask & (1U << 4U)) != 0U) ++result.lower_hits;
    }
    return wrapped;
}

void seal_contact_batch14(const std::vector<std::uint32_t>& ids,
    ContactBatch14& wrapped) {
    ContactBatch& contact = wrapped.value;
    if (!contact.valid || wrapped.first_time.size() != ids.size()) {
        throw std::logic_error("cannot seal invalid NCGP14 contact");
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp13.contact.v1");
    put_u64(bytes, ids.size());
    for (std::size_t index = 0U; index < ids.size(); ++index) {
        put_u32(bytes, ids[index]);
        put_u32(bytes, contact.mask[index]);
        put_u32(bytes, contact.first_mask[index]);
        for (const Vec3l value : std::array<Vec3l, 3>{
                 contact.endpoint[index], contact.correction[index],
                 contact.impulse[index]}) {
            put_f64(bytes, value.x);
            put_f64(bytes, value.y);
            put_f64(bytes, value.z);
        }
        put_f64(bytes, std::isfinite(wrapped.first_time[index])
                ? wrapped.first_time[index] : -1.0L);
    }
    contact.root = root_of(std::move(bytes));
    ++contact.work.hash_derivations;
}

Work expected_contact_parent_work(const NonlocalGpuProfile& profile,
    const std::vector<Vec3l>& start, const std::vector<Vec3l>& proposed,
    bool disable_xy, bool seal) {
    Work result;
    bool valid = start.size() == proposed.size();
    for (std::size_t index = 0U; index < start.size(); ++index) {
        Contact contact = swept_contact(profile, start[index], proposed[index]);
        if (disable_xy) {
            contact.endpoint.x = proposed[index].x;
            contact.endpoint.y = proposed[index].y;
            contact.mask &= (1U << 4U) | (1U << 5U);
            contact.first_mask &= (1U << 4U) | (1U << 5U);
            contact.hits = ((contact.mask & (1U << 4U)) != 0U ? 1U : 0U)
                + ((contact.mask & (1U << 5U)) != 0U ? 1U : 0U);
            if (contact.first_mask == 0U) {
                contact.t_first =
                    std::numeric_limits<long double>::infinity();
            }
            contact.correction = contact.endpoint - proposed[index];
            contact.impulse = contact.correction
                * (static_cast<long double>(profile.mass) / profile.dt);
        }
        result.plane_tests += contact.tests;
        result.plane_hits += contact.hits;
        if (contact.mask != 0U) ++result.contact_projections;
        valid = contact.valid && finite(start[index])
            && finite(proposed[index]) && finite(contact.endpoint)
            && finite(contact.correction) && finite(contact.impulse)
            && (std::isfinite(contact.t_first)
                || contact.first_mask == 0U) && valid;
    }
    result.hash_derivations = seal && valid ? 1U : 0U;
    return result;
}

Work expected_jacobian_parent_work(std::size_t dynamic_count,
    std::size_t ghost_count) {
    Work result;
    result.analytic_jv_multiply_adds = dynamic_count * 3U * dynamic_count;
    result.finite_difference_candidates = 2U * dynamic_count
        * (dynamic_count + ghost_count);
    return result;
}

Work expected_qp_parent_work(const Solve& solve) {
    Work result;
    result.qp_sweeps = solve.sweeps;
    result.qp_updates = solve.updates;
    result.gradient_recomputations = 1U + solve.sweeps;
    return result;
}

Work expected_independent_density_parent_work(std::size_t dynamic_count,
    std::size_t ghost_count) {
    Work result;
    result.independent_density_candidates = dynamic_count
        * (dynamic_count + ghost_count);
    return result;
}

struct Scratch14 {
    std::vector<std::uint32_t> owner_ids;
    std::vector<std::uint64_t> row_begin;
    std::vector<std::uint64_t> row_end;
    std::vector<std::pair<std::uint8_t, std::uint32_t>> entries;
    std::vector<long double> lambda;
    std::vector<long double> gradient;
    std::vector<std::uint8_t> first_mask;
    std::vector<std::uint8_t> clamp_mask;
    std::vector<Vec3l> correction;
    std::vector<Vec3l> impulse;
};

std::string scratch_root(std::string_view lane_root,
    const Scratch14& scratch, Work14* owner = nullptr) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.scratch.v1");
    put_string(bytes, lane_root);
    put_u64(bytes, scratch.owner_ids.size());
    for (std::size_t index = 0U; index < scratch.owner_ids.size(); ++index) {
        put_u32(bytes, scratch.owner_ids[index]);
        put_u64(bytes, scratch.row_begin[index]);
        put_u64(bytes, scratch.row_end[index]);
    }
    put_u64(bytes, scratch.entries.size());
    for (const auto& entry : scratch.entries) {
        put_u8(bytes, entry.first);
        put_u32(bytes, entry.second);
    }
    put_u64(bytes, scratch.lambda.size());
    for (std::size_t index = 0U; index < scratch.lambda.size(); ++index) {
        put_u32(bytes, scratch.owner_ids[index]);
        put_f64(bytes, scratch.lambda[index]);
        put_f64(bytes, scratch.gradient[index]);
    }
    put_u64(bytes, scratch.clamp_mask.size());
    for (std::size_t index = 0U; index < scratch.clamp_mask.size(); ++index) {
        put_u32(bytes, scratch.owner_ids[index]);
        put_u8(bytes, scratch.first_mask[index]);
        put_u8(bytes, scratch.clamp_mask[index]);
        put_f64(bytes, scratch.correction[index].x);
        put_f64(bytes, scratch.correction[index].y);
        put_f64(bytes, scratch.correction[index].z);
        put_f64(bytes, scratch.impulse[index].x);
        put_f64(bytes, scratch.impulse[index].y);
        put_f64(bytes, scratch.impulse[index].z);
    }
    if (owner != nullptr) {
        owner->portable_content_fields_serialized += 6U
            + 3U * scratch.owner_ids.size() + 2U * scratch.entries.size()
            + 3U * scratch.lambda.size() + 9U * scratch.clamp_mask.size();
        ++owner->parent.hash_derivations;
    }
    return root_of(std::move(bytes));
}

std::string state14_root(std::string_view base_lane_root,
    const State& state) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.state.v1");
    put_string(bytes, base_lane_root);
    put_u64(bytes, state.id.size());
    for (std::size_t index = 0U; index < state.id.size(); ++index) {
        put_u32(bytes, state.id[index]);
        for (const Vec3l value : std::array<Vec3l, 3>{
                 state.reference[index], state.position[index],
                 state.velocity[index]}) {
            put_f64(bytes, value.x);
            put_f64(bytes, value.y);
            put_f64(bytes, value.z);
        }
    }
    return root_of(std::move(bytes));
}

constexpr std::uint64_t state14_portable_fields(std::size_t count) {
    return 3U + 10U * count;
}

void clear_scratch14(Scratch14& scratch) {
    scratch.owner_ids.clear();
    scratch.row_begin.clear();
    scratch.row_end.clear();
    scratch.entries.clear();
    scratch.lambda.clear();
    scratch.gradient.clear();
    scratch.first_mask.clear();
    scratch.clamp_mask.clear();
    scratch.correction.clear();
    scratch.impulse.clear();
}

std::uint64_t compare_scratch_values(const Scratch14& lhs,
    const Scratch14& rhs, bool& equal) {
    std::uint64_t compared = 0U;
    auto compare = [&](bool predicate) {
        ++compared;
        equal = predicate && equal;
    };
    equal = true;
    compare(lhs.owner_ids.size() == rhs.owner_ids.size());
    compare(lhs.row_begin.size() == rhs.row_begin.size());
    compare(lhs.row_end.size() == rhs.row_end.size());
    compare(lhs.entries.size() == rhs.entries.size());
    compare(lhs.lambda.size() == rhs.lambda.size());
    compare(lhs.gradient.size() == rhs.gradient.size());
    compare(lhs.first_mask.size() == rhs.first_mask.size());
    compare(lhs.clamp_mask.size() == rhs.clamp_mask.size());
    compare(lhs.correction.size() == rhs.correction.size());
    compare(lhs.impulse.size() == rhs.impulse.size());
    const std::size_t owner_count = std::min({lhs.owner_ids.size(),
        rhs.owner_ids.size(), lhs.row_begin.size(), rhs.row_begin.size(),
        lhs.row_end.size(), rhs.row_end.size()});
    for (std::size_t index = 0U; index < owner_count; ++index) {
        compare(lhs.owner_ids[index] == rhs.owner_ids[index]);
        compare(lhs.row_begin[index] == rhs.row_begin[index]);
        compare(lhs.row_end[index] == rhs.row_end[index]);
    }
    const std::size_t entry_count = std::min(lhs.entries.size(),
        rhs.entries.size());
    for (std::size_t index = 0U; index < entry_count; ++index) {
        compare(lhs.entries[index].first == rhs.entries[index].first);
        compare(lhs.entries[index].second == rhs.entries[index].second);
    }
    const std::size_t qp_count = std::min({lhs.lambda.size(),
        rhs.lambda.size(), lhs.gradient.size(), rhs.gradient.size()});
    for (std::size_t index = 0U; index < qp_count; ++index) {
        compare(lhs.lambda[index] == rhs.lambda[index]);
        compare(lhs.gradient[index] == rhs.gradient[index]);
    }
    const std::size_t contact_count = std::min({lhs.first_mask.size(),
        rhs.first_mask.size(), lhs.clamp_mask.size(), rhs.clamp_mask.size(),
        lhs.correction.size(), rhs.correction.size(), lhs.impulse.size(),
        rhs.impulse.size()});
    for (std::size_t index = 0U; index < contact_count; ++index) {
        compare(lhs.first_mask[index] == rhs.first_mask[index]);
        compare(lhs.clamp_mask[index] == rhs.clamp_mask[index]);
        compare(lhs.correction[index].x == rhs.correction[index].x);
        compare(lhs.correction[index].y == rhs.correction[index].y);
        compare(lhs.correction[index].z == rhs.correction[index].z);
        compare(lhs.impulse[index].x == rhs.impulse[index].x);
        compare(lhs.impulse[index].y == rhs.impulse[index].y);
        compare(lhs.impulse[index].z == rhs.impulse[index].z);
    }
    return compared;
}

struct Step14 {
    StepResult step;
    Work expected_parent_work;
    std::vector<RoundExtra14> extra_rounds;
    Work14 extra_work;
    Work14 expected_extra_work;
    std::string typed_route;
    std::string failing_velocity_root;
    bool qp_cap_exhausted = false;
    bool projection_cap_exhausted = false;
    bool trial_invariants_ok = false;
    bool tight_support_ok = true;
    bool top_contact_zero = true;
    bool bottom_contact = false;
    bool positive_pressure = false;
    bool predictor_witnesses_evaluated = false;
    std::uint64_t maximum_qp_sweeps = 0U;
    std::uint64_t satellite_count = 0U;
    std::array<std::uint64_t, 6> predictor_first_hit_faces{};
    std::array<std::uint64_t, 6> predictor_clamp_faces{};
    Scratch14 scratch;
    std::vector<Vec3l> scratch_input_positions;
    std::string scratch_input_root;
    State private_trial_state;
    Graph accepted_post_graph;
    std::vector<std::string> early_child_stages;
    std::vector<std::string> early_child_roots;
    std::vector<std::uint32_t> early_child_rounds;
    std::uint64_t expected_step_hashes = 0U;
    bool semantic_payload_preflighted = false;
    bool semantic_roots_derived = false;
    bool attempted_step_sealed = false;
    bool fully_computed_trial = false;
    std::uint64_t expected_direct_comparisons = 0U;
};

void close_step14_expected_extra(Step14& step, bool tight) {
    static_cast<void>(tight);
    step.expected_extra_work.parent.hash_derivations +=
        step.extra_rounds.size();
    step.expected_extra_work.portable_content_fields_serialized +=
        46U * step.extra_rounds.size();
    step.expected_extra_work.lane_scalar_comparisons +=
        step.expected_direct_comparisons;
}

void close_step14_expected_parent(Step14& step) {
    if (step.semantic_roots_derived) {
        step.expected_parent_work.hash_derivations += 5U;
    }
}

std::array<std::uint64_t, 6> contact_face_counts(
    const std::vector<std::uint32_t>& masks) {
    std::array<std::uint64_t, 6> result{};
    for (const std::uint32_t mask : masks) {
        for (std::size_t face = 0U; face < result.size(); ++face) {
            if ((mask & (1U << face)) != 0U) ++result[face];
        }
    }
    return result;
}

bool counted_tight_faces_ok(const FaceCounts& faces, Work14& work) {
    bool pass = true;
    for (const bool predicate : std::array<bool, 6>{
             faces.membership[0] > 0U, faces.membership[1] > 0U,
             faces.membership[2] > 0U, faces.membership[3] > 0U,
             faces.membership[4] > 0U, faces.membership[5] == 0U}) {
        combine_predicate(pass, work, predicate);
    }
    return pass;
}

struct TopologyMetrics14 {
    std::size_t components = 0U;
    std::size_t satellites = 0U;
};

TopologyMetrics14 topology_metrics14(const Graph& graph, Work& work) {
    TopologyMetrics14 result;
    const std::size_t count = graph.row.size();
    if (count == 0U) return result;
    std::vector<std::size_t> parent(count);
    std::iota(parent.begin(), parent.end(), 0U);
    auto find = [&](std::size_t value) {
        std::size_t root = value;
        while (parent[root] != root) root = parent[root];
        while (parent[value] != value) {
            const std::size_t next = parent[value];
            parent[value] = root;
            value = next;
        }
        return root;
    };
    for (std::size_t owner = 0U; owner < count; ++owner) {
        for (const Edge edge : graph.row[owner]) {
            if (edge.ghost) continue;
            ++work.topology_distance_tests;
            if (edge.index == owner) continue;
            const std::size_t lhs = find(owner);
            const std::size_t rhs = find(edge.index);
            if (lhs != rhs) parent[rhs] = lhs;
        }
    }
    std::map<std::size_t, std::size_t> sizes;
    for (std::size_t index = 0U; index < count; ++index) {
        ++work.topology_discoveries;
        ++sizes[find(index)];
    }
    result.components = sizes.size();
    std::size_t largest = 0U;
    for (const auto& entry : sizes) largest = std::max(largest, entry.second);
    result.satellites = count - largest;
    return result;
}

Work expected_topology_parent_work(const Graph& graph) {
    Work result;
    result.topology_discoveries = graph.row.size();
    for (const auto& row : graph.row) {
        for (const Edge edge : row) {
            if (!edge.ghost) ++result.topology_distance_tests;
        }
    }
    return result;
}

bool finite_scalar_vector14(const std::vector<long double>& values) {
    bool result = true;
    for (const long double value : values) {
        result = std::isfinite(value) && result;
    }
    return result;
}

bool counted_finite_scalar_vector14(const std::vector<long double>& values,
    Work14& owner) {
    bool result = true;
    for (const long double value : values) {
        combine_predicate(result, owner, std::isfinite(value));
    }
    return result;
}

bool counted_nonnegative_scalar_vector14(
    const std::vector<long double>& values, Work14& owner) {
    bool result = true;
    for (const long double value : values) {
        combine_predicate(result, owner, value >= 0.0L);
    }
    return result;
}

bool counted_finite_vec_vector14(const std::vector<Vec3l>& values,
    Work14& owner) {
    bool result = true;
    for (const Vec3l value : values) {
        combine_predicate(result, owner, std::isfinite(value.x));
        combine_predicate(result, owner, std::isfinite(value.y));
        combine_predicate(result, owner, std::isfinite(value.z));
    }
    return result;
}

void derive_step_semantic_roots14(Step14& wrapped,
    const std::vector<std::uint32_t>& ids,
    std::uint64_t expected_before_standard_roots) {
    StepResult& result = wrapped.step;
    if (wrapped.semantic_roots_derived) {
        throw std::logic_error("NCGP14 step semantic roots derived twice");
    }
    if (!wrapped.semantic_payload_preflighted) {
        result.apparatus_valid = false;
        result.failure = "NONFINITE_OBSERVABLE";
        wrapped.typed_route = result.failure;
        return;
    }
    result.multiplier_root = id_scalar_root(
        "nextengine.nonlocal.ncgp13.step-multiplier.v1", ids,
        result.lambda_sum);
    result.contact_mask_root = id_mask_root(
        "nextengine.nonlocal.ncgp13.step-contact-mask.v1", ids,
        result.contact_mask);
    result.contact_impulse_root = id_vec_root(
        "nextengine.nonlocal.ncgp13.step-contact-impulse.v1", ids,
        result.contact_impulse_sum);
    result.state_root = state_root(
        "nextengine.nonlocal.ncgp13.accepted-state.v1", result.state);
    result.velocity_root = id_vec_root(
        "nextengine.nonlocal.ncgp13.velocity.v1", ids,
        result.state.velocity);
    result.work.hash_derivations += 5U;
    result.expected_hash_derivations = expected_before_standard_roots + 5U;
    result.hash_accounting_exact = result.work.hash_derivations
        == result.expected_hash_derivations;
    wrapped.semantic_roots_derived = true;
    wrapped.expected_step_hashes = result.expected_hash_derivations;
    for (const auto& child :
         std::array<std::pair<const char*, const std::string*>, 5>{
             std::pair{"TRIAL_STATE", &result.state_root},
             std::pair{"TRIAL_VELOCITY", &result.velocity_root},
             std::pair{"TRIAL_MULTIPLIER", &result.multiplier_root},
             std::pair{"TRIAL_CONTACT_MASK", &result.contact_mask_root},
             std::pair{"TRIAL_CONTACT_IMPULSE",
                 &result.contact_impulse_root}}) {
        wrapped.early_child_stages.emplace_back(child.first);
        wrapped.early_child_roots.push_back(*child.second);
        wrapped.early_child_rounds.push_back(0U);
    }
    if (!result.hash_accounting_exact) {
        result.apparatus_valid = false;
        result.failure = "TRAJECTORY_HASH_ACCOUNTING_INVALID";
        wrapped.typed_route = result.failure;
    }
}

void seal_attempted_step14(Step14& wrapped) {
    if (!wrapped.semantic_roots_derived || wrapped.attempted_step_sealed) {
        throw std::logic_error("invalid NCGP14 attempted-step seal order");
    }
    wrapped.step.result_root = step_root(wrapped.step);
    wrapped.attempted_step_sealed = true;
    wrapped.failing_velocity_root = wrapped.step.velocity_root;
}

#if 0
Step14 run_step14_rev4_legacy(const LanePolicy& policy, const State& input,
    const std::vector<NonlocalGpuGhost>& ghosts, const GhostGrid& grid,
    std::uint32_t trial_index = 1U) {
    const NonlocalGpuProfile& profile = policy.fixture->profile;
    Step14 wrapped;
    StepResult& result = wrapped.step;
    result.state = input;
    const std::size_t count = input.id.size();
    result.lambda_sum.assign(count, 0.0L);
    result.pressure_jt_sum.assign(count, {});
    result.contact_delta_sum.assign(count, {});
    result.contact_impulse_sum.assign(count, {});
    result.contact_mask.assign(count, 0U);
    const long double dt = profile.dt;
    const long double dt2 = dt * dt;
    const Vec3l gravity = widen(profile.gravity);
    std::vector<Vec3l> displacement(count);
    std::vector<Vec3l> proposed(count);
    for (std::size_t index = 0U; index < count; ++index) {
        displacement[index] = input.velocity[index] * dt + gravity * dt2;
        proposed[index] = input.position[index] + displacement[index];
    }
    ContactBatch predictor = contact_batch14(profile, input.id,
        input.position, proposed, policy.disable_xy_contact);
    add_work(result.work, predictor.work);
    add_work(wrapped.expected_parent_work,
        expected_contact_parent_work(predictor));
    std::uint64_t expected_step_hashes = 1U;
    result.predictor_contact_root = predictor.root;
    const bool predictor_valid = counted_predicate(
        wrapped.extra_work, predictor.valid);
    if (!predictor_valid) {
        result.apparatus_valid = false;
        result.failure = "PREDICTOR_CONTACT_INVALID";
        wrapped.typed_route = result.failure;
        seal_step(result, input.id, expected_step_hashes);
        close_step14_expected_parent(wrapped);
        wrapped.failing_velocity_root = result.velocity_root;
        close_step14_expected_extra(wrapped, policy.tight);
        return wrapped;
    }
    std::vector<Vec3l> y = predictor.endpoint;
    for (std::size_t index = 0U; index < count; ++index) {
        result.contact_delta_sum[index] += predictor.correction[index];
        result.contact_impulse_sum[index] += predictor.impulse[index];
        result.contact_mask[index] |= predictor.mask[index];
    }
    result.lower_contacts += predictor.lower_hits;
    const auto predictor_faces = contact_face_counts(predictor.mask);
    wrapped.predictor_first_hit_faces = contact_face_counts(
        predictor.first_mask);
    wrapped.predictor_clamp_faces = predictor_faces;
    wrapped.bottom_contact = counted_predicate(wrapped.extra_work,
        predictor.lower_hits > 0U);
    wrapped.top_contact_zero = counted_predicate(wrapped.extra_work,
        predictor_faces[5] == 0U);
    wrapped.predictor_witnesses_evaluated = true;
    wrapped.scratch.owner_ids = input.id;
    wrapped.scratch.first_mask.assign(predictor.first_mask.begin(),
        predictor.first_mask.end());
    wrapped.scratch.clamp_mask.assign(predictor.mask.begin(),
        predictor.mask.end());
    wrapped.scratch.correction = predictor.correction;
    wrapped.scratch.impulse = predictor.impulse;

    std::vector<long double> last_density;
    auto record_round = [&](RoundSummary& round, RoundExtra14 extra,
                            std::uint64_t expected_hash_derivations) {
        seal_round(round, expected_hash_derivations);
        expected_step_hashes += expected_hash_derivations;
        result.rounds.push_back(round);
        if (extra.fully_computed) {
            extra.inherited_round_root = round.result_root;
            seal_round_extra14(policy.root, extra, wrapped.extra_work);
            wrapped.extra_rounds.push_back(extra);
        }
        add_work(result.work, round.work);
    };
    for (std::uint32_t round_index = 0U;
         round_index < policy.projection_cap; ++round_index) {
        RoundSummary round;
        RoundExtra14 extra;
        extra.trial_index = trial_index;
        extra.round_index = round_index + 1U;
        ++round.work.projection_rounds;
        ++round.work.hash_derivations;
        ++wrapped.expected_parent_work.projection_rounds;
        ++wrapped.expected_parent_work.hash_derivations;
        const std::string expected_input = position_state_root(input.id, y);
        Assembly14 built = assemble14(profile, input.id, y, ghosts, grid,
            true, nullptr);
        Assembly& assembly = built.value;
        add_work(round.work, assembly.work);
        add_work(wrapped.expected_parent_work, built.expected_parent_work);
        add_work14(wrapped.extra_work, built.extra);
        add_work14(wrapped.expected_extra_work, built.expected_extra);
        extra.pre_faces = built.faces;
        extra.row_capacity_ok = !built.capacity;
        extra.finite_capacity_valid = assembly.valid && !built.capacity;
        extra.assembly_valid = assembly.valid;
        round.assembly_valid = assembly.valid;
        round.assembly_input_root = assembly.input_root;
        round.graph_root = assembly.graph_root;
        round.density_root = assembly.density_root;
        round.jacobian_root = assembly.jacobian_root;
        round.matrix_root = assembly.matrix_root;
        round.stale = assembly.input_root != expected_input;
        round.symmetry = assembly.symmetry;
        round.jacobian = jacobian_check(profile, input.id, y, ghosts, grid,
            assembly, true, round.work);
        add_work(wrapped.expected_parent_work,
            expected_jacobian_parent_work(count, ghosts.size()));
        const bool jv_finite = counted_predicate(wrapped.extra_work,
            std::isfinite(round.jacobian));
        const bool jv_within_limit = counted_predicate(wrapped.extra_work,
            round.jacobian <= kJacobianLimit);
        extra.jv_ok = jv_finite && jv_within_limit;
        extra.jv_relative_l2 = round.jacobian;
        extra.symmetry_relative_l2 = round.symmetry;
        const bool symmetry_finite = counted_predicate(wrapped.extra_work,
            std::isfinite(round.symmetry));
        const bool symmetry_within_limit = counted_predicate(
            wrapped.extra_work, round.symmetry <= kSymmetryLimit);
        extra.symmetry_valid = symmetry_finite && symmetry_within_limit;
        bool pre_qp_valid = true;
        combine_predicate(pre_qp_valid, wrapped.extra_work, assembly.valid);
        combine_predicate(pre_qp_valid, wrapped.extra_work, !built.capacity);
        combine_predicate(pre_qp_valid, wrapped.extra_work, !round.stale);
        pre_qp_valid = symmetry_finite && pre_qp_valid;
        pre_qp_valid = symmetry_within_limit && pre_qp_valid;
        pre_qp_valid = jv_finite && pre_qp_valid;
        pre_qp_valid = jv_within_limit && pre_qp_valid;
        if (!pre_qp_valid) {
            result.apparatus_valid = false;
            result.failure = built.capacity ? "ROW_NEIGHBOR_CAPACITY_EXCEEDED"
                : round.stale ? "STALE_ASSEMBLY_REJECTED"
                : "ASSEMBLY_OR_DERIVATIVE_INVALID";
            wrapped.typed_route = result.failure;
            record_round(round, extra, 6U);
            break;
        }
        Solve14 solved = solve_qp14(assembly, policy.qp_cap, round.work);
        Solve& solve = solved.value;
        add_work(wrapped.expected_parent_work,
            expected_qp_parent_work(solve));
        round.solve_valid = solve.valid;
        round.converged = solve.converged;
        round.sweeps = solve.sweeps;
        round.updates = solve.updates;
        round.primal = solve.primal;
        round.kkt = solve.projected_kkt;
        round.complementarity = solve.complementarity;
        wrapped.maximum_qp_sweeps = std::max(wrapped.maximum_qp_sweeps,
            static_cast<std::uint64_t>(solve.sweeps));
        round.positive = static_cast<std::size_t>(std::count_if(
            solve.lambda.begin(), solve.lambda.end(),
            [](long double value) { return value > 0.0L; }));
        round.multiplier_root = id_scalar_root(
            "nextengine.nonlocal.ncgp13.round-multiplier.v1", input.id,
            solve.lambda);
        ++round.work.hash_derivations;
        ++wrapped.expected_parent_work.hash_derivations;
        wrapped.scratch.lambda = solve.lambda;
        wrapped.scratch.gradient = solve.gradient;
        wrapped.scratch_input_positions = y;
        wrapped.scratch_input_root = round.assembly_input_root;
        wrapped.scratch.row_begin.clear();
        wrapped.scratch.row_end.clear();
        wrapped.scratch.entries.clear();
        std::uint64_t row_offset = 0U;
        for (const auto& row : built.graph.row) {
            wrapped.scratch.row_begin.push_back(row_offset);
            for (const Edge edge : row) {
                wrapped.scratch.entries.emplace_back(
                    edge.ghost ? 1U : 0U,
                    edge.ghost ? ghosts[edge.index].sample_id
                               : input.id[edge.index]);
                ++row_offset;
            }
            wrapped.scratch.row_end.push_back(row_offset);
        }
        extra.qp_valid = solve.valid;
        extra.qp_converged = solve.converged;
        const bool qp_valid_predicate = counted_predicate(
            wrapped.extra_work, solve.valid);
        const bool qp_converged_predicate = counted_predicate(
            wrapped.extra_work, solve.converged);
        if (!qp_valid_predicate) {
            result.apparatus_valid = false;
            result.failure = "QP_INVALID";
            wrapped.typed_route = result.failure;
            record_round(round, extra, 7U);
            break;
        }
        if (!qp_converged_predicate) {
            const bool completed_cap = counted_predicate(
                wrapped.extra_work, solve.sweeps == policy.qp_cap);
            extra.qp_cap_exhausted = solve.valid
                && !solve.converged && completed_cap;
            wrapped.qp_cap_exhausted = extra.qp_cap_exhausted;
            result.failure = extra.qp_cap_exhausted
                ? "QP_SWEEP_CAP_EXHAUSTED" : "QP_INVALID";
            wrapped.typed_route = result.failure;
            record_round(round, extra, 7U);
            break;
        }
        const auto jt = jt_times(assembly, solve.lambda);
        const long double correction_scale = dt2 / profile.mass;
        std::vector<Vec3l> pressure_delta(count);
        std::vector<Vec3l> pressure_force(count);
        std::vector<Vec3l> pressure_residual(count);
        std::vector<Vec3l> pressure_proposed(count);
        for (std::size_t index = 0U; index < count; ++index) {
            pressure_force[index] = {jt[3U * index], jt[3U * index + 1U],
                jt[3U * index + 2U]};
            pressure_delta[index] = pressure_force[index] * (-correction_scale);
            pressure_residual[index] = pressure_delta[index]
                    * (profile.mass / dt2) + pressure_force[index];
            pressure_proposed[index] = y[index] + pressure_delta[index];
            result.lambda_sum[index] += solve.lambda[index];
            result.pressure_jt_sum[index] += pressure_force[index];
        }
        round.pressure_balance = dense_norm(pressure_residual)
            / std::max({dense_norm(pressure_force),
                profile.mass * norm(gravity)
                    * std::sqrt(static_cast<long double>(count)),
                1.0e-30L * static_cast<long double>(count)});
        const ContactBatch contact = contact_batch14(profile, input.id, y,
            pressure_proposed, policy.disable_xy_contact);
        add_work(round.work, contact.work);
        add_work(wrapped.expected_parent_work,
            expected_contact_parent_work(contact));
        round.contact_root = contact.root;
        extra.first_hit_faces = contact_face_counts(contact.first_mask);
        extra.clamp_faces = contact_face_counts(contact.mask);
        extra.contact_valid = contact.valid;
        const bool contact_valid_predicate = counted_predicate(
            wrapped.extra_work, contact.valid);
        if (!contact_valid_predicate) {
            result.apparatus_valid = false;
            result.failure = "ROUND_CONTACT_INVALID";
            wrapped.typed_route = result.failure;
            record_round(round, extra, 8U);
            break;
        }
        const bool round_top_clear = counted_predicate(wrapped.extra_work,
            extra.clamp_faces[5] == 0U);
        const bool round_lower_contact = counted_predicate(
            wrapped.extra_work, contact.lower_hits > 0U);
        wrapped.top_contact_zero = round_top_clear
            && wrapped.top_contact_zero;
        const bool pressure_balance_finite = counted_predicate(
            wrapped.extra_work, std::isfinite(round.pressure_balance));
        const bool pressure_balance_within = counted_predicate(
            wrapped.extra_work, round.pressure_balance <= kBalanceLimit);
        extra.balance_ok = pressure_balance_finite
            && pressure_balance_within;
        y = contact.endpoint;
        for (std::size_t index = 0U; index < count; ++index) {
            result.contact_delta_sum[index] += contact.correction[index];
            result.contact_impulse_sum[index] += contact.impulse[index];
            result.contact_mask[index] |= contact.mask[index];
        }
        result.lower_contacts += contact.lower_hits;
        wrapped.bottom_contact = round_lower_contact
            || wrapped.bottom_contact;
        wrapped.scratch.first_mask.assign(contact.first_mask.begin(),
            contact.first_mask.end());
        wrapped.scratch.clamp_mask.assign(contact.mask.begin(),
            contact.mask.end());
        wrapped.scratch.correction = contact.correction;
        wrapped.scratch.impulse = contact.impulse;
        Density14 candidate = candidate_density14(
            profile, input.id, y, ghosts, grid);
        add_work(round.work, candidate.parent_work);
        add_work(wrapped.expected_parent_work,
            candidate.expected_parent_work);
        add_work14(wrapped.extra_work, candidate.extra);
        add_work14(wrapped.expected_extra_work, candidate.expected_extra);
        const DensityResult independent = independent_density(
            profile, y, ghosts, grid, true);
        round.work.independent_density_candidates += independent.candidates;
        add_work(wrapped.expected_parent_work,
            expected_independent_density_parent_work(count, ghosts.size()));
        round.post_graph_root = candidate.value.graph_root;
        round.post_density_root = id_scalar_root(
            "nextengine.nonlocal.ncgp13.post-density.v1", input.id,
            candidate.value.value);
        round.independent_density_root = id_scalar_root(
            "nextengine.nonlocal.ncgp13.independent-density.v1", input.id,
            independent.value);
        round.work.hash_derivations += 2U;
        wrapped.expected_parent_work.hash_derivations += 2U;
        last_density = independent.value;
        std::vector<long double> density_difference(count, 0.0L);
        for (std::size_t index = 0U; index < count; ++index) {
            density_difference[index] = candidate.value.value[index]
                - independent.value[index];
        }
        round.density_correspondence = vector_norm(density_difference)
            / std::max(vector_norm(independent.value), 1.0e-30L);
        const auto strains = strain_metrics(last_density, profile.rest_density);
        round.maximum_strain = strains.first;
        round.rms_strain = strains.second;
        const bool inset_valid = counted_predicate(wrapped.extra_work,
            inset_oracle(profile, y, round.penetration));
        const bool zero_penetration = counted_predicate(wrapped.extra_work,
            round.penetration == 0.0L);
        extra.inset_ok = inset_valid && zero_penetration;
        round.state_root = id_vec_root(
            "nextengine.nonlocal.ncgp13.round-state.v1", input.id, y);
        ++round.work.hash_derivations;
        ++wrapped.expected_parent_work.hash_derivations;
        const bool candidate_valid = counted_predicate(wrapped.extra_work,
            candidate.value.valid);
        const bool candidate_capacity_valid = counted_predicate(
            wrapped.extra_work, !candidate.capacity);
        extra.candidate_density_valid = candidate_valid
            && candidate_capacity_valid;
        extra.independent_density_valid = counted_predicate(
            wrapped.extra_work, independent.valid);
        extra.density_correspondence_valid = counted_predicate(
            wrapped.extra_work,
            round.density_correspondence <= 2.0e-12L);
        extra.pressure_balance = round.pressure_balance;
        extra.density_correspondence = round.density_correspondence;
        extra.maximum_positive_strain = round.maximum_strain;
        extra.rms_positive_strain = round.rms_strain;
        extra.penetration_m = round.penetration;
        extra.positive_multiplier_count = round.positive;
        bool post_qp_valid = true;
        post_qp_valid = extra.candidate_density_valid && post_qp_valid;
        post_qp_valid = extra.independent_density_valid && post_qp_valid;
        post_qp_valid = extra.density_correspondence_valid && post_qp_valid;
        post_qp_valid = extra.inset_ok && post_qp_valid;
        post_qp_valid = extra.balance_ok && post_qp_valid;
        bool tight_faces_valid = true;
        if (policy.tight) {
            tight_faces_valid = counted_tight_faces_ok(
                extra.pre_faces, wrapped.extra_work);
            post_qp_valid = tight_faces_valid && round_top_clear
                && post_qp_valid;
        }
        extra.round_apparatus_ok = pre_qp_valid && qp_valid_predicate
            && qp_converged_predicate && contact_valid_predicate
            && post_qp_valid;
        bool physical_closed = true;
        combine_predicate(physical_closed, wrapped.extra_work,
            round.maximum_strain <= kDensityMaximumLimit);
        combine_predicate(physical_closed, wrapped.extra_work,
            round.rms_strain <= kDensityRmsLimit);
        const bool round_positive_pressure = counted_predicate(
            wrapped.extra_work, round.positive > 0U);
        extra.round_closed = extra.round_apparatus_ok && physical_closed;
        extra.fully_computed = true;
        record_round(round, extra, 12U);
        result.positive_multipliers += round.positive;
        wrapped.positive_pressure = round_positive_pressure
            || wrapped.positive_pressure;
        wrapped.tight_support_ok = wrapped.tight_support_ok
            && tight_faces_valid;
        if (!extra.round_apparatus_ok) {
            result.apparatus_valid = false;
            if (!extra.balance_ok) {
                result.failure = "PRESSURE_BALANCE_CORRESPONDENCE_INVALID";
            } else if (!extra.inset_ok) {
                result.failure = "CONTACT_INSET_CORRESPONDENCE_INVALID";
            } else {
                result.failure = "ROUND_APPARATUS_INVALID";
            }
            wrapped.typed_route = result.failure;
            break;
        }
        if (extra.round_closed) {
            result.accepted = true;
            break;
        }
    }

    if (!result.accepted && result.failure.empty()) {
        result.failure = "PROJECTION_ROUND_CAP_EXHAUSTED";
        wrapped.typed_route = result.failure;
        bool exhausted = true;
        combine_predicate(exhausted, wrapped.extra_work,
            result.rounds.size() == policy.projection_cap);
        for (const RoundExtra14& extra : wrapped.extra_rounds) {
            combine_predicate(exhausted, wrapped.extra_work,
                extra.round_apparatus_ok);
            combine_predicate(exhausted, wrapped.extra_work,
                !extra.round_closed);
        }
        wrapped.projection_cap_exhausted = exhausted;
    }
    if (result.accepted) {
        result.state.position = y;
        for (std::size_t index = 0U; index < count; ++index) {
            result.state.velocity[index] =
                (y[index] - input.position[index]) * (1.0L / dt);
        }
        Density14 candidate = candidate_density14(
            profile, input.id, y, ghosts, grid);
        add_work(result.work, candidate.parent_work);
        add_work(wrapped.expected_parent_work,
            candidate.expected_parent_work);
        add_work14(wrapped.extra_work, candidate.extra);
        add_work14(wrapped.expected_extra_work, candidate.expected_extra);
        ++expected_step_hashes;
        const DensityResult independent = independent_density(
            profile, y, ghosts, grid, true);
        result.work.independent_density_candidates += independent.candidates;
        add_work(wrapped.expected_parent_work,
            expected_independent_density_parent_work(count, ghosts.size()));
        const auto strains = strain_metrics(independent.value,
            profile.rest_density);
        result.maximum_strain = strains.first;
        result.rms_strain = strains.second;
        static_cast<void>(inset_oracle(profile, y, result.penetration));
        std::vector<long double> density_error(count, 0.0L);
        for (std::size_t index = 0U; index < count; ++index) {
            density_error[index] = candidate.value.value[index]
                - independent.value[index];
        }
        result.independent_correspondence = vector_norm(density_error)
            / std::max(vector_norm(independent.value), 1.0e-30L);
        std::vector<Vec3l> inertial(count);
        std::vector<Vec3l> contact_term(count);
        std::vector<Vec3l> residual(count);
        for (std::size_t index = 0U; index < count; ++index) {
            inertial[index] = (y[index] - input.position[index]
                    - displacement[index]) * (profile.mass / dt2);
            contact_term[index] = result.contact_delta_sum[index]
                * (-profile.mass / dt2);
            residual[index] = inertial[index] + result.pressure_jt_sum[index]
                + contact_term[index];
        }
        result.balance = dense_norm(residual) / std::max({dense_norm(inertial),
            dense_norm(result.pressure_jt_sum), dense_norm(contact_term),
            1.0e-30L * static_cast<long double>(count)});
        long double bottom_sum = 0.0L;
        long double top_sum = 0.0L;
        std::size_t bottom_count = 0U;
        std::size_t top_count = 0U;
        const std::size_t nx = input.id.size() == 512U ? 8U : 4U;
        const std::size_t ny = nx;
        const std::size_t nz = 8U;
        for (std::size_t index = 0U; index < count; ++index) {
            const std::size_t logical = (input.id[index] - 1000U) / 17U;
            const std::size_t layer = logical / (nx * ny);
            if (layer == 0U) {
                bottom_sum += result.lambda_sum[index];
                ++bottom_count;
            }
            if (layer + 1U == nz) {
                top_sum += result.lambda_sum[index];
                ++top_count;
            }
        }
        result.bottom_pressure_proxy = static_cast<long double>(
                profile.rest_density) / profile.mass
            * bottom_sum / bottom_count;
        result.top_pressure_proxy = static_cast<long double>(
                profile.rest_density) / profile.mass
            * top_sum / top_count;
        const TopologyMetrics14 topology = topology_metrics14(
            profile, y, result.work);
        add_work(wrapped.expected_parent_work,
            expected_topology_parent_work(profile, y));
        result.component_count = topology.components;
        wrapped.satellite_count = topology.satellites;
        bool identity_mass = true;
        combine_predicate(identity_mass, wrapped.extra_work,
            result.state.id.size() == input.id.size());
        combine_predicate(identity_mass, wrapped.extra_work,
            result.state.reference.size() == input.reference.size());
        const std::size_t identity_count = std::min({result.state.id.size(),
            input.id.size(), result.state.reference.size(),
            input.reference.size()});
        for (std::size_t index = 0U; index < identity_count; ++index) {
            combine_predicate(identity_mass, wrapped.extra_work,
                result.state.id[index] == input.id[index]);
            combine_predicate(identity_mass, wrapped.extra_work,
                result.state.reference[index].x == input.reference[index].x);
            combine_predicate(identity_mass, wrapped.extra_work,
                result.state.reference[index].y == input.reference[index].y);
            combine_predicate(identity_mass, wrapped.extra_work,
                result.state.reference[index].z == input.reference[index].z);
        }
        combine_predicate(identity_mass, wrapped.extra_work,
            static_cast<long double>(result.state.id.size()) * profile.mass
                == static_cast<long double>(input.id.size()) * profile.mass);
        result.identity_mass_exact = identity_mass;
        const bool balance_valid = counted_predicate(wrapped.extra_work,
            result.balance <= kBalanceLimit);
        wrapped.trial_invariants_ok = identity_mass && balance_valid;
        bool physical = true;
        combine_predicate(physical, wrapped.extra_work,
            result.maximum_strain <= kDensityMaximumLimit);
        combine_predicate(physical, wrapped.extra_work,
            result.rms_strain <= kDensityRmsLimit);
        combine_predicate(physical, wrapped.extra_work,
            result.penetration == 0.0L);
        combine_predicate(physical, wrapped.extra_work,
            result.independent_correspondence <= 2.0e-12L);
        combine_predicate(physical, wrapped.extra_work,
            wrapped.trial_invariants_ok);
        combine_predicate(physical, wrapped.extra_work,
            result.component_count == 1U);
        if (policy.tight) {
            combine_predicate(physical, wrapped.extra_work,
                wrapped.tight_support_ok);
            combine_predicate(physical, wrapped.extra_work,
                wrapped.top_contact_zero);
            combine_predicate(physical, wrapped.extra_work,
                wrapped.positive_pressure);
            combine_predicate(physical, wrapped.extra_work,
                wrapped.bottom_contact);
        }
        result.physical_pass = physical;
        if (!wrapped.trial_invariants_ok) {
            result.apparatus_valid = false;
            result.failure = result.balance > kBalanceLimit
                ? "STEP_BALANCE_CORRESPONDENCE_INVALID"
                : "STATE_ID_MASS_INVARIANT_INVALID";
            wrapped.typed_route = result.failure;
        } else if (!result.physical_pass) {
            result.failure = "PHYSICAL_TRIAL_GATE_REJECTED";
            wrapped.typed_route = result.failure;
        } else {
            wrapped.typed_route = "TRIAL_SUPPORTED";
        }
        wrapped.private_trial_state = result.state;
    }
    seal_step(result, input.id, expected_step_hashes);
    close_step14_expected_parent(wrapped);
    wrapped.failing_velocity_root = result.velocity_root;
    close_step14_expected_extra(wrapped, policy.tight);
    return wrapped;
}
#endif

Step14 run_step14(const LanePolicy& policy, const State& input,
    const std::vector<NonlocalGpuGhost>& ghosts, const GhostGrid& grid,
    std::uint32_t trial_index = 1U) {
    const NonlocalGpuProfile& profile = policy.fixture->profile;
    Step14 wrapped;
    StepResult& result = wrapped.step;
    result.state = input;
    const std::size_t count = input.id.size();
    result.lambda_sum.assign(count, 0.0L);
    result.pressure_jt_sum.assign(count, {});
    result.contact_delta_sum.assign(count, {});
    result.contact_impulse_sum.assign(count, {});
    result.contact_mask.assign(count, 0U);
    const long double dt = profile.dt;
    const long double dt2 = dt * dt;
    const Vec3l gravity = widen(profile.gravity);
    std::vector<Vec3l> displacement(count);
    std::vector<Vec3l> proposed(count);
    for (std::size_t index = 0U; index < count; ++index) {
        displacement[index] = input.velocity[index] * dt + gravity * dt2;
        proposed[index] = input.position[index] + displacement[index];
    }

    ContactBatch14 predictor_wrapped = contact_batch14(profile, input.id,
        input.position, proposed, policy.disable_xy_contact);
    ContactBatch& predictor = predictor_wrapped.value;
    ++wrapped.expected_direct_comparisons;
    const bool predictor_valid = counted_predicate(
        wrapped.extra_work, predictor.valid);
    if (!predictor_valid) {
        add_work(result.work, predictor.work);
        add_work(wrapped.expected_parent_work,
            expected_contact_parent_work(profile, input.position, proposed,
                policy.disable_xy_contact, false));
        result.apparatus_valid = false;
        result.failure = "PREDICTOR_CONTACT_INVALID";
        wrapped.typed_route = result.failure;
        close_step14_expected_extra(wrapped, policy.tight);
        return wrapped;
    }
    seal_contact_batch14(input.id, predictor_wrapped);
    add_work(result.work, predictor.work);
    add_work(wrapped.expected_parent_work,
        expected_contact_parent_work(profile, input.position, proposed,
            policy.disable_xy_contact, true));
    result.predictor_contact_root = predictor.root;
    wrapped.early_child_stages.push_back("TRIAL_PREDICTOR_CONTACT");
    wrapped.early_child_roots.push_back(predictor.root);
    wrapped.early_child_rounds.push_back(0U);
    std::vector<Vec3l> y = predictor.endpoint;
    for (std::size_t index = 0U; index < count; ++index) {
        result.contact_delta_sum[index] += predictor.correction[index];
        result.contact_impulse_sum[index] += predictor.impulse[index];
        result.contact_mask[index] |= predictor.mask[index];
    }
    result.lower_contacts += predictor.lower_hits;
    const auto predictor_faces = contact_face_counts(predictor.mask);
    wrapped.predictor_first_hit_faces = contact_face_counts(
        predictor.first_mask);
    wrapped.predictor_clamp_faces = predictor_faces;
    wrapped.expected_direct_comparisons += 2U;
    wrapped.bottom_contact = counted_predicate(wrapped.extra_work,
        predictor.lower_hits > 0U);
    wrapped.top_contact_zero = counted_predicate(wrapped.extra_work,
        predictor_faces[5] == 0U);
    wrapped.predictor_witnesses_evaluated = true;
    wrapped.scratch.owner_ids = input.id;
    wrapped.scratch.first_mask.assign(predictor.first_mask.begin(),
        predictor.first_mask.end());
    wrapped.scratch.clamp_mask.assign(predictor.mask.begin(),
        predictor.mask.end());
    wrapped.scratch.correction = predictor.correction;
    wrapped.scratch.impulse = predictor.impulse;

    std::vector<long double> last_independent_density;
    std::vector<long double> last_candidate_density;
    std::uint64_t expected_before_standard_roots = 1U;
    auto add_assembly_prefix = [&](const RoundSummary& round,
                                   std::uint32_t round_number,
                                   bool include_graph,
                                   bool include_multiplier,
                                   bool include_contact,
                                   bool include_post_graph) {
        wrapped.early_child_stages.push_back("ROUND_INPUT_STATE");
        wrapped.early_child_roots.push_back(round.assembly_input_root);
        wrapped.early_child_rounds.push_back(round_number);
        if (include_graph) {
            for (const auto& entry : std::array<std::pair<const char*,
                     const std::string*>, 4>{
                     std::pair{"ROUND_GRAPH", &round.graph_root},
                     std::pair{"ROUND_DENSITY", &round.density_root},
                     std::pair{"ROUND_JACOBIAN", &round.jacobian_root},
                     std::pair{"ROUND_MATRIX", &round.matrix_root}}) {
                wrapped.early_child_stages.emplace_back(entry.first);
                wrapped.early_child_roots.push_back(*entry.second);
                wrapped.early_child_rounds.push_back(round_number);
            }
        }
        if (include_multiplier) {
            wrapped.early_child_stages.push_back("ROUND_MULTIPLIER");
            wrapped.early_child_roots.push_back(round.multiplier_root);
            wrapped.early_child_rounds.push_back(round_number);
        }
        if (include_contact) {
            wrapped.early_child_stages.push_back("ROUND_CONTACT");
            wrapped.early_child_roots.push_back(round.contact_root);
            wrapped.early_child_rounds.push_back(round_number);
        }
        if (include_post_graph) {
            wrapped.early_child_stages.push_back("ROUND_POST_GRAPH");
            wrapped.early_child_roots.push_back(round.post_graph_root);
            wrapped.early_child_rounds.push_back(round_number);
        }
    };
    auto record_round = [&](RoundSummary& round, RoundExtra14& extra,
                            std::uint64_t expected_hashes) {
        seal_round(round, expected_hashes);
        expected_before_standard_roots += expected_hashes;
        result.rounds.push_back(round);
        if (extra.fully_computed) {
            extra.inherited_round_root = round.result_root;
            seal_round_extra14(policy.root, extra, wrapped.extra_work);
            wrapped.extra_rounds.push_back(extra);
        }
        add_work(result.work, round.work);
    };
    const auto record_unsealed_round_work = [&](const RoundSummary& round) {
        add_work(result.work, round.work);
    };

    for (std::uint32_t round_index = 0U;
         round_index < policy.projection_cap; ++round_index) {
        RoundSummary round;
        RoundExtra14 extra;
        extra.trial_index = trial_index;
        extra.round_index = round_index + 1U;
        ++round.work.projection_rounds;
        ++wrapped.expected_parent_work.projection_rounds;
        Assembly14 built = assemble14(profile, input.id, y, ghosts, grid,
            true, nullptr);
        Assembly& assembly = built.value;
        add_work(round.work, assembly.work);
        add_work(wrapped.expected_parent_work, built.expected_parent_work);
        add_work14(wrapped.extra_work, built.extra);
        add_work14(wrapped.expected_extra_work, built.expected_extra);
        extra.pre_faces = built.faces;
        extra.row_capacity_ok = !built.capacity;
        extra.finite_capacity_valid = !built.capacity;
        extra.assembly_valid = assembly.valid;
        round.assembly_valid = assembly.valid;
        round.assembly_input_root = assembly.input_root;
        round.graph_root = assembly.graph_root;
        round.density_root = assembly.density_root;
        round.jacobian_root = assembly.jacobian_root;
        round.matrix_root = assembly.matrix_root;
        round.symmetry = assembly.symmetry;

        if (built.capacity) {
            result.apparatus_valid = false;
            result.failure = "ROW_NEIGHBOR_CAPACITY_EXCEEDED";
            wrapped.typed_route = result.failure;
            record_unsealed_round_work(round);
            break;
        }
        ++wrapped.expected_direct_comparisons;
        const bool assembly_valid = counted_predicate(
            wrapped.extra_work, assembly.valid);
        if (!assembly_valid) {
            result.apparatus_valid = false;
            result.failure = built.nonfinite
                ? "NONFINITE_ASSEMBLY" : "GRAPH_ASSEMBLY_INVALID";
            wrapped.typed_route = result.failure;
            if (!round.assembly_input_root.empty()) {
                add_assembly_prefix(round, round_index + 1U,
                    false, false, false, false);
            }
            record_unsealed_round_work(round);
            break;
        }

        round.jacobian = jacobian_check(profile, input.id, y, ghosts, grid,
            assembly, true, round.work);
        add_work(wrapped.expected_parent_work,
            expected_jacobian_parent_work(count, ghosts.size()));
        const bool jv_finite = counted_predicate(wrapped.extra_work,
            std::isfinite(round.jacobian));
        const bool jv_within = counted_predicate(wrapped.extra_work,
            round.jacobian <= kJacobianLimit);
        const bool symmetry_finite = counted_predicate(wrapped.extra_work,
            std::isfinite(round.symmetry));
        const bool symmetry_within = counted_predicate(wrapped.extra_work,
            round.symmetry <= kSymmetryLimit);
        const bool capacity_valid = counted_predicate(wrapped.extra_work,
            !built.capacity);
        const bool graph_valid = counted_predicate(wrapped.extra_work,
            built.graph.valid);
        const bool input_current = counted_predicate(wrapped.extra_work,
            !round.assembly_input_root.empty());
        wrapped.expected_direct_comparisons += 7U;
        extra.jv_ok = jv_finite && jv_within;
        extra.symmetry_valid = symmetry_finite && symmetry_within;
        extra.jv_relative_l2 = round.jacobian;
        extra.symmetry_relative_l2 = round.symmetry;
        const bool pre_qp_valid = assembly_valid && jv_finite && jv_within
            && symmetry_finite && symmetry_within && capacity_valid
            && graph_valid && input_current;
        if (!pre_qp_valid) {
            result.apparatus_valid = false;
            result.failure = !extra.jv_ok
                ? "JACOBIAN_CORRESPONDENCE_INVALID"
                : "MATRIX_SYMMETRY_CORRESPONDENCE_INVALID";
            wrapped.typed_route = result.failure;
            add_assembly_prefix(round, round_index + 1U,
                true, false, false, false);
            record_unsealed_round_work(round);
            break;
        }

        Solve14 solved = solve_qp14(assembly, policy.qp_cap, round.work);
        Solve& solve = solved.value;
        add_work(wrapped.expected_parent_work, expected_qp_parent_work(solve));
        round.solve_valid = solve.valid;
        round.converged = solve.converged;
        round.sweeps = solve.sweeps;
        round.updates = solve.updates;
        round.primal = solve.primal;
        round.kkt = solve.projected_kkt;
        round.complementarity = solve.complementarity;
        wrapped.maximum_qp_sweeps = std::max(wrapped.maximum_qp_sweeps,
            static_cast<std::uint64_t>(solve.sweeps));
        bool solve_finite = true;
        combine_predicate(solve_finite, wrapped.extra_work,
            std::isfinite(solve.primal));
        combine_predicate(solve_finite, wrapped.extra_work,
            std::isfinite(solve.projected_kkt));
        combine_predicate(solve_finite, wrapped.extra_work,
            std::isfinite(solve.complementarity));
        const bool lambda_finite = counted_finite_scalar_vector14(
            solve.lambda, wrapped.extra_work);
        const bool gradient_finite = counted_finite_scalar_vector14(
            solve.gradient, wrapped.extra_work);
        const bool lambda_nonnegative = counted_nonnegative_scalar_vector14(
            solve.lambda, wrapped.extra_work);
        solve_finite = lambda_finite && gradient_finite
            && lambda_nonnegative && solve_finite;
        const bool solve_declared_valid = counted_predicate(
            wrapped.extra_work, solve.valid);
        wrapped.expected_direct_comparisons +=
            4U + solve.lambda.size() + solve.gradient.size()
            + solve.lambda.size();
        const bool qp_valid = solve_declared_valid && solve_finite;
        if (!qp_valid) {
            result.apparatus_valid = false;
            result.failure = solve_finite
                ? "QP_CORRESPONDENCE_INVALID" : "NONFINITE_QP";
            wrapped.typed_route = result.failure;
            add_assembly_prefix(round, round_index + 1U,
                true, false, false, false);
            record_unsealed_round_work(round);
            break;
        }
        round.positive = static_cast<std::size_t>(std::count_if(
            solve.lambda.begin(), solve.lambda.end(),
            [](long double value) { return value > 0.0L; }));
        round.multiplier_root = id_scalar_root(
            "nextengine.nonlocal.ncgp13.round-multiplier.v1", input.id,
            solve.lambda);
        ++round.work.hash_derivations;
        ++wrapped.expected_parent_work.hash_derivations;
        wrapped.scratch.lambda = solve.lambda;
        wrapped.scratch.gradient = solve.gradient;
        wrapped.scratch_input_positions = y;
        wrapped.scratch_input_root = round.assembly_input_root;
        wrapped.scratch.row_begin.clear();
        wrapped.scratch.row_end.clear();
        wrapped.scratch.entries.clear();
        std::uint64_t row_offset = 0U;
        for (const auto& row : built.graph.row) {
            wrapped.scratch.row_begin.push_back(row_offset);
            for (const Edge edge : row) {
                wrapped.scratch.entries.emplace_back(
                    edge.ghost ? 1U : 0U,
                    edge.ghost ? ghosts[edge.index].sample_id
                               : input.id[edge.index]);
                ++row_offset;
            }
            wrapped.scratch.row_end.push_back(row_offset);
        }
        ++wrapped.expected_direct_comparisons;
        const bool qp_converged = counted_predicate(
            wrapped.extra_work, solve.converged);
        extra.qp_valid = qp_valid;
        extra.qp_converged = qp_converged;
        if (!qp_converged) {
            ++wrapped.expected_direct_comparisons;
            const bool completed_cap = counted_predicate(
                wrapped.extra_work, solve.sweeps == policy.qp_cap);
            wrapped.qp_cap_exhausted = completed_cap;
            result.failure = completed_cap ? "QP_SWEEP_CAP_EXHAUSTED"
                                           : "QP_CORRESPONDENCE_INVALID";
            result.apparatus_valid = completed_cap;
            wrapped.typed_route = result.failure;
            record_round(round, extra, 6U);
            break;
        }

        const auto jt = jt_times(assembly, solve.lambda);
        const long double correction_scale = dt2 / profile.mass;
        std::vector<Vec3l> pressure_delta(count);
        std::vector<Vec3l> pressure_force(count);
        std::vector<Vec3l> pressure_residual(count);
        std::vector<Vec3l> pressure_proposed(count);
        bool pressure_finite = true;
        for (std::size_t index = 0U; index < count; ++index) {
            pressure_force[index] = {jt[3U * index], jt[3U * index + 1U],
                jt[3U * index + 2U]};
            pressure_delta[index] = pressure_force[index] * (-correction_scale);
            pressure_residual[index] = pressure_delta[index]
                    * (profile.mass / dt2) + pressure_force[index];
            pressure_proposed[index] = y[index] + pressure_delta[index];
        }
        const bool pressure_force_finite = counted_finite_vec_vector14(
            pressure_force, wrapped.extra_work);
        const bool pressure_delta_finite = counted_finite_vec_vector14(
            pressure_delta, wrapped.extra_work);
        const bool pressure_residual_finite = counted_finite_vec_vector14(
            pressure_residual, wrapped.extra_work);
        const bool pressure_proposed_finite = counted_finite_vec_vector14(
            pressure_proposed, wrapped.extra_work);
        pressure_finite = pressure_force_finite && pressure_delta_finite
            && pressure_residual_finite && pressure_proposed_finite;
        wrapped.expected_direct_comparisons += 12U * count;
        round.pressure_balance = pressure_finite
            ? dense_norm(pressure_residual)
                / std::max({dense_norm(pressure_force),
                    profile.mass * norm(gravity)
                        * std::sqrt(static_cast<long double>(count)),
                    1.0e-30L * static_cast<long double>(count)})
            : std::numeric_limits<long double>::infinity();
        if (!pressure_finite) {
            result.apparatus_valid = false;
            result.failure = "NONFINITE_CONTACT";
            wrapped.typed_route = result.failure;
            add_assembly_prefix(round, round_index + 1U,
                true, true, false, false);
            record_unsealed_round_work(round);
            break;
        }
        ContactBatch14 contact_wrapped = contact_batch14(profile, input.id,
            y, pressure_proposed, policy.disable_xy_contact);
        ContactBatch& contact = contact_wrapped.value;
        bool contact_payload_finite = true;
        const bool endpoint_finite = counted_finite_vec_vector14(
            contact.endpoint, wrapped.extra_work);
        const bool correction_finite = counted_finite_vec_vector14(
            contact.correction, wrapped.extra_work);
        const bool impulse_finite = counted_finite_vec_vector14(
            contact.impulse, wrapped.extra_work);
        contact_payload_finite = endpoint_finite && correction_finite
            && impulse_finite;
        for (std::size_t index = 0U;
             index < contact_wrapped.first_time.size(); ++index) {
            const bool finite_first_time = counted_predicate(
                wrapped.extra_work,
                std::isfinite(contact_wrapped.first_time[index]));
            const bool no_first_hit = counted_predicate(
                wrapped.extra_work, contact.first_mask[index] == 0U);
            contact_payload_finite = (finite_first_time || no_first_hit)
                && contact_payload_finite;
        }
        const bool contact_declared_valid = counted_predicate(
            wrapped.extra_work, contact.valid);
        wrapped.expected_direct_comparisons +=
            11U * count + 1U;
        const bool contact_valid = contact_declared_valid
            && contact_payload_finite;
        if (!contact_valid) {
            add_work(round.work, contact.work);
            add_work(wrapped.expected_parent_work,
                expected_contact_parent_work(profile, y, pressure_proposed,
                    policy.disable_xy_contact, false));
            result.apparatus_valid = false;
            result.failure = contact_payload_finite
                ? "CONTACT_CORRESPONDENCE_INVALID" : "NONFINITE_CONTACT";
            wrapped.typed_route = result.failure;
            add_assembly_prefix(round, round_index + 1U,
                true, true, false, false);
            record_unsealed_round_work(round);
            break;
        }
        seal_contact_batch14(input.id, contact_wrapped);
        add_work(round.work, contact.work);
        add_work(wrapped.expected_parent_work,
            expected_contact_parent_work(profile, y, pressure_proposed,
                policy.disable_xy_contact, true));
        round.contact_root = contact.root;
        extra.first_hit_faces = contact_face_counts(contact.first_mask);
        extra.clamp_faces = contact_face_counts(contact.mask);
        extra.contact_valid = true;
        for (std::size_t index = 0U; index < count; ++index) {
            result.lambda_sum[index] += solve.lambda[index];
            result.pressure_jt_sum[index] += pressure_force[index];
            result.contact_delta_sum[index] += contact.correction[index];
            result.contact_impulse_sum[index] += contact.impulse[index];
            result.contact_mask[index] |= contact.mask[index];
        }
        result.lower_contacts += contact.lower_hits;
        y = contact.endpoint;
        wrapped.scratch.first_mask.assign(contact.first_mask.begin(),
            contact.first_mask.end());
        wrapped.scratch.clamp_mask.assign(contact.mask.begin(),
            contact.mask.end());
        wrapped.scratch.correction = contact.correction;
        wrapped.scratch.impulse = contact.impulse;

        Density14 candidate = candidate_density14(
            profile, input.id, y, ghosts, grid);
        add_work(round.work, candidate.parent_work);
        add_work(wrapped.expected_parent_work,
            candidate.expected_parent_work);
        add_work14(wrapped.extra_work, candidate.extra);
        add_work14(wrapped.expected_extra_work, candidate.expected_extra);
        if (candidate.capacity) {
            result.apparatus_valid = false;
            result.failure = "ROW_NEIGHBOR_CAPACITY_EXCEEDED";
            wrapped.typed_route = result.failure;
            add_assembly_prefix(round, round_index + 1U,
                true, true, true, false);
            record_unsealed_round_work(round);
            break;
        }
        round.post_graph_root = candidate.value.graph_root;
        const DensityResult independent = independent_density(
            profile, y, ghosts, grid, true);
        round.work.independent_density_candidates += independent.candidates;
        add_work(wrapped.expected_parent_work,
            expected_independent_density_parent_work(count, ghosts.size()));
        const bool candidate_declared_valid = counted_predicate(
            wrapped.extra_work, candidate.value.valid);
        const bool independent_declared_valid = counted_predicate(
            wrapped.extra_work, independent.valid);
        const bool candidate_values_finite = counted_finite_scalar_vector14(
            candidate.value.value, wrapped.extra_work);
        const bool independent_values_finite = counted_finite_scalar_vector14(
            independent.value, wrapped.extra_work);
        wrapped.expected_direct_comparisons += 2U
            + candidate.value.value.size() + independent.value.size();
        const bool densities_finite = candidate_declared_valid
            && independent_declared_valid && candidate_values_finite
            && independent_values_finite;
        if (!densities_finite) {
            result.apparatus_valid = false;
            result.failure = "NONFINITE_DENSITY";
            wrapped.typed_route = result.failure;
            add_assembly_prefix(round, round_index + 1U,
                true, true, true, true);
            record_unsealed_round_work(round);
            break;
        }
        std::vector<long double> density_difference(count, 0.0L);
        for (std::size_t index = 0U; index < count; ++index) {
            density_difference[index] = candidate.value.value[index]
                - independent.value[index];
        }
        round.density_correspondence = vector_norm(density_difference)
            / std::max(vector_norm(independent.value), 1.0e-30L);
        long double penetration = 0.0L;
        const bool inset_valid_value = inset_oracle(profile, y, penetration);
        round.penetration = penetration;
        const bool density_metric_finite = counted_predicate(
            wrapped.extra_work,
            std::isfinite(round.density_correspondence));
        const bool pressure_balance_payload_finite = counted_predicate(
            wrapped.extra_work, std::isfinite(round.pressure_balance));
        const bool penetration_payload_finite = counted_predicate(
            wrapped.extra_work, std::isfinite(round.penetration));
        wrapped.expected_direct_comparisons += 3U;
        const bool post_metrics_finite = density_metric_finite
            && pressure_balance_payload_finite
            && penetration_payload_finite;
        if (!post_metrics_finite) {
            result.apparatus_valid = false;
            result.failure = "NONFINITE_DENSITY";
            wrapped.typed_route = result.failure;
            add_assembly_prefix(round, round_index + 1U,
                true, true, true, true);
            record_unsealed_round_work(round);
            break;
        }
        const bool round_top_clear = counted_predicate(wrapped.extra_work,
            extra.clamp_faces[5] == 0U);
        const bool round_lower_contact = counted_predicate(
            wrapped.extra_work, contact.lower_hits > 0U);
        const bool pressure_balance_within = counted_predicate(
            wrapped.extra_work, round.pressure_balance <= kBalanceLimit);
        const bool candidate_capacity_valid = counted_predicate(
            wrapped.extra_work, !candidate.capacity);
        const bool density_correspondence_valid = counted_predicate(
            wrapped.extra_work, round.density_correspondence <= 2.0e-12L);
        const bool inset_valid = counted_predicate(wrapped.extra_work,
            inset_valid_value);
        const bool inset_zero = counted_predicate(wrapped.extra_work,
            round.penetration == 0.0L);
        wrapped.expected_direct_comparisons += 7U;
        wrapped.top_contact_zero = round_top_clear
            && wrapped.top_contact_zero;
        wrapped.bottom_contact = round_lower_contact
            || wrapped.bottom_contact;
        extra.balance_ok = pressure_balance_payload_finite
            && pressure_balance_within;
        extra.candidate_density_valid = candidate_declared_valid
            && candidate_capacity_valid;
        extra.independent_density_valid = independent_declared_valid;
        extra.density_correspondence_valid =
            density_correspondence_valid;
        extra.inset_ok = inset_valid && inset_zero;
        bool tight_faces_valid = true;
        if (policy.tight) {
            tight_faces_valid = counted_tight_faces_ok(
                extra.pre_faces, wrapped.extra_work);
            wrapped.expected_direct_comparisons += 6U;
        }
        extra.round_apparatus_ok = pre_qp_valid && qp_valid
            && qp_converged && contact_valid
            && extra.candidate_density_valid
            && extra.independent_density_valid
            && extra.density_correspondence_valid && extra.balance_ok
            && extra.inset_ok && tight_faces_valid
            && (!policy.tight || round_top_clear);
        extra.pressure_balance = round.pressure_balance;
        extra.density_correspondence = round.density_correspondence;
        extra.penetration_m = round.penetration;
        extra.positive_multiplier_count = round.positive;
        extra.fully_computed = true;

        if (extra.round_apparatus_ok) {
            const auto strains = strain_metrics(
                independent.value, profile.rest_density);
            round.maximum_strain = strains.first;
            round.rms_strain = strains.second;
            extra.maximum_positive_strain = round.maximum_strain;
            extra.rms_positive_strain = round.rms_strain;
            bool strain_payload_finite = true;
            combine_predicate(strain_payload_finite, wrapped.extra_work,
                std::isfinite(round.maximum_strain));
            combine_predicate(strain_payload_finite, wrapped.extra_work,
                std::isfinite(round.rms_strain));
            wrapped.expected_direct_comparisons += 2U;
            if (!strain_payload_finite) {
                result.apparatus_valid = false;
                result.failure = "NONFINITE_DENSITY";
                wrapped.typed_route = result.failure;
                add_assembly_prefix(round, round_index + 1U,
                    true, true, true, true);
                record_unsealed_round_work(round);
                break;
            }
            bool round_closed = true;
            combine_predicate(round_closed, wrapped.extra_work,
                round.maximum_strain <= kDensityMaximumLimit);
            combine_predicate(round_closed, wrapped.extra_work,
                round.rms_strain <= kDensityRmsLimit);
            const bool positive = counted_predicate(wrapped.extra_work,
                round.positive > 0U);
            wrapped.expected_direct_comparisons += 3U;
            extra.round_closed = round_closed;
            wrapped.positive_pressure = positive
                || wrapped.positive_pressure;
            result.positive_multipliers += round.positive;
        } else {
            extra.round_closed = false;
        }
        round.post_density_root = id_scalar_root(
            "nextengine.nonlocal.ncgp13.post-density.v1", input.id,
            candidate.value.value);
        round.independent_density_root = id_scalar_root(
            "nextengine.nonlocal.ncgp13.independent-density.v1", input.id,
            independent.value);
        round.state_root = id_vec_root(
            "nextengine.nonlocal.ncgp13.round-state.v1", input.id, y);
        round.work.hash_derivations += 3U;
        wrapped.expected_parent_work.hash_derivations += 3U;
        last_candidate_density = candidate.value.value;
        last_independent_density = independent.value;
        wrapped.tight_support_ok = wrapped.tight_support_ok
            && tight_faces_valid;
        record_round(round, extra, 11U);
        if (!extra.round_apparatus_ok) {
            result.apparatus_valid = false;
            if (!extra.balance_ok) {
                result.failure = "PRESSURE_BALANCE_CORRESPONDENCE_INVALID";
            } else if (!extra.inset_ok) {
                result.failure = "CONTACT_INSET_CORRESPONDENCE_INVALID";
            } else if (policy.tight && !tight_faces_valid) {
                result.failure = "TIGHT_SUPPORT_CORRESPONDENCE_INVALID";
            } else if (policy.tight && !round_top_clear) {
                result.failure = "TOP_CONTACT_CORRESPONDENCE_INVALID";
            } else {
                result.failure = "DENSITY_CORRESPONDENCE_INVALID";
            }
            wrapped.typed_route = result.failure;
            break;
        }
        if (extra.round_closed) {
            result.accepted = true;
            wrapped.accepted_post_graph = candidate.graph;
            break;
        }
    }

    if (!result.accepted && result.failure.empty()) {
        result.failure = "PROJECTION_ROUND_CAP_EXHAUSTED";
        wrapped.typed_route = result.failure;
        bool exhausted = true;
        combine_predicate(exhausted, wrapped.extra_work,
            result.rounds.size() == policy.projection_cap);
        for (const RoundExtra14& extra : wrapped.extra_rounds) {
            combine_predicate(exhausted, wrapped.extra_work,
                extra.round_apparatus_ok);
            combine_predicate(exhausted, wrapped.extra_work,
                !extra.round_closed);
        }
        wrapped.expected_direct_comparisons +=
            1U + 2U * wrapped.extra_rounds.size();
        wrapped.projection_cap_exhausted = exhausted;
    }

    if (result.accepted) {
        result.state.position = y;
        for (std::size_t index = 0U; index < count; ++index) {
            result.state.velocity[index] =
                (y[index] - input.position[index]) * (1.0L / dt);
        }
        Density14 final_candidate = candidate_density14(
            profile, input.id, y, ghosts, grid);
        add_work(result.work, final_candidate.parent_work);
        add_work(wrapped.expected_parent_work,
            final_candidate.expected_parent_work);
        add_work14(wrapped.extra_work, final_candidate.extra);
        add_work14(wrapped.expected_extra_work,
            final_candidate.expected_extra);
        if (final_candidate.capacity) {
            result.apparatus_valid = false;
            result.failure = "ROW_NEIGHBOR_CAPACITY_EXCEEDED";
            wrapped.typed_route = result.failure;
        } else {
            ++expected_before_standard_roots;
            const DensityResult independent = independent_density(
                profile, y, ghosts, grid, true);
            result.work.independent_density_candidates +=
                independent.candidates;
            add_work(wrapped.expected_parent_work,
                expected_independent_density_parent_work(
                    count, ghosts.size()));
            const bool final_candidate_valid = counted_predicate(
                wrapped.extra_work, final_candidate.value.valid);
            const bool final_independent_valid = counted_predicate(
                wrapped.extra_work, independent.valid);
            const bool final_candidate_finite =
                counted_finite_scalar_vector14(
                    final_candidate.value.value, wrapped.extra_work);
            const bool final_independent_finite =
                counted_finite_scalar_vector14(
                    independent.value, wrapped.extra_work);
            const bool final_position_finite = counted_finite_vec_vector14(
                result.state.position, wrapped.extra_work);
            const bool final_velocity_finite = counted_finite_vec_vector14(
                result.state.velocity, wrapped.extra_work);
            wrapped.expected_direct_comparisons += 2U
                + final_candidate.value.value.size()
                + independent.value.size()
                + 3U * result.state.position.size()
                + 3U * result.state.velocity.size();
            const bool finite_observables = final_candidate_valid
                && final_independent_valid && final_candidate_finite
                && final_independent_finite && final_position_finite
                && final_velocity_finite;
            if (!finite_observables) {
                result.apparatus_valid = false;
                result.failure = "NONFINITE_DENSITY";
                wrapped.typed_route = result.failure;
            } else {
                const auto strains = strain_metrics(
                    independent.value, profile.rest_density);
                result.maximum_strain = strains.first;
                result.rms_strain = strains.second;
                static_cast<void>(inset_oracle(
                    profile, y, result.penetration));
                std::vector<long double> density_error(count, 0.0L);
                for (std::size_t index = 0U; index < count; ++index) {
                    density_error[index] = final_candidate.value.value[index]
                        - independent.value[index];
                }
                result.independent_correspondence = vector_norm(density_error)
                    / std::max(vector_norm(independent.value), 1.0e-30L);
                std::vector<Vec3l> inertial(count);
                std::vector<Vec3l> contact_term(count);
                std::vector<Vec3l> residual(count);
                for (std::size_t index = 0U; index < count; ++index) {
                    inertial[index] = (y[index] - input.position[index]
                            - displacement[index]) * (profile.mass / dt2);
                    contact_term[index] = result.contact_delta_sum[index]
                        * (-profile.mass / dt2);
                    residual[index] = inertial[index]
                        + result.pressure_jt_sum[index]
                        + contact_term[index];
                }
                result.balance = dense_norm(residual)
                    / std::max({dense_norm(inertial),
                        dense_norm(result.pressure_jt_sum),
                        dense_norm(contact_term),
                        1.0e-30L * static_cast<long double>(count)});
                long double bottom_sum = 0.0L;
                long double top_sum = 0.0L;
                std::size_t bottom_count = 0U;
                std::size_t top_count = 0U;
                const std::size_t nx = input.id.size() == 512U ? 8U : 4U;
                const std::size_t ny = nx;
                constexpr std::size_t nz = 8U;
                for (std::size_t index = 0U; index < count; ++index) {
                    const std::size_t logical =
                        (input.id[index] - 1000U) / 17U;
                    const std::size_t layer = logical / (nx * ny);
                    if (layer == 0U) {
                        bottom_sum += result.lambda_sum[index];
                        ++bottom_count;
                    }
                    if (layer + 1U == nz) {
                        top_sum += result.lambda_sum[index];
                        ++top_count;
                    }
                }
                result.bottom_pressure_proxy = static_cast<long double>(
                        profile.rest_density) / profile.mass
                    * bottom_sum / bottom_count;
                result.top_pressure_proxy = static_cast<long double>(
                        profile.rest_density) / profile.mass
                    * top_sum / top_count;
                bool step_payload_finite = true;
                for (const long double value : std::array<long double, 7>{
                         result.maximum_strain, result.rms_strain,
                         result.penetration,
                         result.independent_correspondence, result.balance,
                         result.bottom_pressure_proxy,
                         result.top_pressure_proxy}) {
                    combine_predicate(step_payload_finite,
                        wrapped.extra_work, std::isfinite(value));
                }
                wrapped.expected_direct_comparisons += 7U;
                if (!step_payload_finite) {
                    result.apparatus_valid = false;
                    result.failure = "NONFINITE_DENSITY";
                    wrapped.typed_route = result.failure;
                } else {
                    const bool multiplier_payload_finite =
                        counted_finite_scalar_vector14(
                            result.lambda_sum, wrapped.extra_work);
                    const bool contact_payload_finite =
                        counted_finite_vec_vector14(
                            result.contact_impulse_sum,
                            wrapped.extra_work);
                    const bool reference_payload_finite =
                        counted_finite_vec_vector14(
                            result.state.reference, wrapped.extra_work);
                    const bool position_payload_finite =
                        counted_finite_vec_vector14(
                            result.state.position, wrapped.extra_work);
                    const bool velocity_payload_finite =
                        counted_finite_vec_vector14(
                            result.state.velocity, wrapped.extra_work);
                    wrapped.expected_direct_comparisons +=
                        result.lambda_sum.size()
                        + 3U * result.contact_impulse_sum.size()
                        + 3U * result.state.reference.size()
                        + 3U * result.state.position.size()
                        + 3U * result.state.velocity.size();
                    wrapped.semantic_payload_preflighted =
                        multiplier_payload_finite
                        && contact_payload_finite
                        && reference_payload_finite
                        && position_payload_finite
                        && velocity_payload_finite;
                    if (!wrapped.semantic_payload_preflighted) {
                        result.apparatus_valid = false;
                        result.failure = !multiplier_payload_finite
                            ? "NONFINITE_QP" : "NONFINITE_CONTACT";
                        wrapped.typed_route = result.failure;
                    } else {
                    bool identity_mass = true;
                    combine_predicate(identity_mass, wrapped.extra_work,
                        result.state.id.size() == input.id.size());
                    combine_predicate(identity_mass, wrapped.extra_work,
                        result.state.reference.size()
                            == input.reference.size());
                    const std::size_t identity_count = std::min({
                        result.state.id.size(), input.id.size(),
                        result.state.reference.size(),
                        input.reference.size()});
                    for (std::size_t index = 0U; index < identity_count;
                         ++index) {
                        combine_predicate(identity_mass, wrapped.extra_work,
                            result.state.id[index] == input.id[index]);
                        combine_predicate(identity_mass, wrapped.extra_work,
                            result.state.reference[index].x
                                == input.reference[index].x);
                        combine_predicate(identity_mass, wrapped.extra_work,
                            result.state.reference[index].y
                                == input.reference[index].y);
                        combine_predicate(identity_mass, wrapped.extra_work,
                            result.state.reference[index].z
                                == input.reference[index].z);
                    }
                    combine_predicate(identity_mass, wrapped.extra_work,
                        static_cast<long double>(result.state.id.size())
                                * profile.mass
                            == static_cast<long double>(input.id.size())
                                * profile.mass);
                    const bool balance_valid = counted_predicate(
                        wrapped.extra_work,
                        result.balance <= kBalanceLimit);
                    wrapped.expected_direct_comparisons +=
                        4U + 4U * identity_count;
                    result.identity_mass_exact = identity_mass;
                    wrapped.trial_invariants_ok =
                        identity_mass && balance_valid;
                    wrapped.private_trial_state = result.state;
                    wrapped.fully_computed_trial = true;
                    wrapped.typed_route = "TRIAL_INVARIANTS_READY";
                    derive_step_semantic_roots14(wrapped, input.id,
                        expected_before_standard_roots);
                    }
                }
            }
        }
    }

    close_step14_expected_parent(wrapped);
    close_step14_expected_extra(wrapped, policy.tight);
    return wrapped;
}

struct TrialObservables14;

struct TrialSkip14 {
    std::string lane_root;
    std::uint32_t trial_index = 2U;
    std::string cause = "NOT_RUN_BY_PRECEDENCE";
    Work14 work;
    Work14 expected_work;
    WorkSeal14 verifier;
    std::string result_root;
};

std::string seal_trial_skip14(TrialSkip14& skip) {
    skip.verifier = verify_work14(skip.work, skip.expected_work);
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.trial-skip.v1");
    put_string(bytes, skip.lane_root);
    put_u32(bytes, skip.trial_index);
    put_string(bytes, skip.cause);
    append_work_verifier(bytes, skip.verifier);
    skip.result_root = root_of(std::move(bytes));
    return skip.result_root;
}

TrialSkip14 trial_skip14(std::string_view lane_root, std::uint32_t trial_index,
    std::string cause) {
    TrialSkip14 result;
    result.lane_root = lane_root;
    result.trial_index = trial_index;
    result.cause = std::move(cause);
    seal_trial_skip14(result);
    return result;
}

enum class LaneCategory14 {
    Supported,
    PhysicalGateRejected,
    QpSweepCapExhausted,
    ProjectionRoundCapExhausted,
    ApparatusInvalid
};

struct LaneCategoryClosure14 {
    LaneCategory14 value = LaneCategory14::ApparatusInvalid;
    bool apparatus_was_valid = false;
    bool one_hot = false;
};

struct ReachedChild14 {
    std::uint32_t trial_index = 0U;
    std::uint32_t round_index = 0U;
    std::string stage;
    std::string root;
};

struct LaneResult14 {
    LanePolicy policy;
    std::string typed_route;
    std::string route_category = "APPARATUS_INVALID";
    std::string failure_cause;
    std::uint32_t attempted_trials = 0U;
    std::uint32_t committed_trials = 0U;
    std::vector<std::string> round_roots;
    std::vector<std::string> round_extra_roots;
    std::vector<std::string> attempted_step_roots;
    std::vector<std::string> rejected_step_roots;
    std::vector<std::string> trial_observable_roots;
    std::vector<std::string> trial_skip_roots;
    std::vector<ReachedChild14> reached_children;
    std::vector<TrialObservables14> trial_observables;
    std::vector<TrialSkip14> trial_skips;
    std::string inherited_trajectory_json;
    std::string trajectory_root;
    std::string accepted_state_root;
    std::string accepted_velocity_root;
    std::string failing_state_root;
    std::string failing_velocity_root;
    bool apparatus_valid = true;
    bool both_steps_supported = false;
    bool physical_gate_rejected = false;
    bool qp_sweep_cap_exhausted = false;
    bool projection_round_cap_exhausted = false;
    bool trial_1_committed = false;
    bool trial_2_attempted = false;
    bool trial_2_committed = false;
    bool projection_cap_saturated = false;
    std::string trial_2_execution = "NOT_RUN_BY_PRECEDENCE";
    std::vector<Step14> trials;
    TrajectoryResult trajectory;
    Work14 work;
    Work14 expected_work;
    WorkSeal14 verifier;
    std::string result_root;
    LaneCategory14 category = LaneCategory14::ApparatusInvalid;
    LaneCategoryClosure14 category_closure;
};

LaneCategoryClosure14 lane_category14(const LaneResult14& lane,
    Work14* owner = nullptr);

std::string lane_result_root(LaneResult14& lane) {
    lane.verifier = verify_work14(lane.work, lane.expected_work);
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.lane-result.v3");
    put_string(bytes, lane.policy.root);
    put_string(bytes, lane.route_category);
    put_string(bytes, lane.failure_cause);
    put_u32(bytes, lane.attempted_trials);
    put_u32(bytes, lane.committed_trials);
    put_u64(bytes, lane.round_roots.size());
    for (const std::string& root : lane.round_roots) put_string(bytes, root);
    put_u64(bytes, lane.round_extra_roots.size());
    for (const std::string& root : lane.round_extra_roots) {
        put_string(bytes, root);
    }
    put_u64(bytes, lane.attempted_step_roots.size());
    for (const std::string& root : lane.attempted_step_roots) {
        put_string(bytes, root);
    }
    put_u64(bytes, lane.rejected_step_roots.size());
    for (const std::string& root : lane.rejected_step_roots) {
        put_string(bytes, root);
    }
    put_u64(bytes, lane.trial_observable_roots.size());
    for (const std::string& root : lane.trial_observable_roots) {
        put_string(bytes, root);
    }
    put_u64(bytes, lane.trial_skip_roots.size());
    for (const std::string& root : lane.trial_skip_roots) {
        put_string(bytes, root);
    }
    put_u64(bytes, lane.reached_children.size());
    for (const ReachedChild14& child : lane.reached_children) {
        put_u32(bytes, child.trial_index);
        put_u32(bytes, child.round_index);
        put_string(bytes, child.stage);
        put_string(bytes, child.root);
    }
    put_string(bytes, lane.trajectory_root);
    put_string(bytes, lane.accepted_state_root);
    put_string(bytes, lane.accepted_velocity_root);
    put_string(bytes, lane.failing_state_root);
    put_string(bytes, lane.failing_velocity_root);
    put_string(bytes, lane.trial_2_execution);
    for (const bool selector : std::array<bool, 9>{lane.apparatus_valid,
             lane.both_steps_supported, lane.physical_gate_rejected,
             lane.qp_sweep_cap_exhausted,
             lane.projection_round_cap_exhausted,
             lane.trial_1_committed, lane.trial_2_attempted,
             lane.trial_2_committed, lane.projection_cap_saturated}) {
        put_u8(bytes, selector ? 1U : 0U);
    }
    append_work_verifier(bytes, lane.verifier);
    lane.result_root = root_of(std::move(bytes));
    return lane.result_root;
}

struct Sequence14 {
    Vec3l gravity_impulse;
    Vec3l pressure_impulse;
    Vec3l contact_impulse;
    long double maximum_position_rms = 0.0L;
    long double maximum_position = 0.0L;
    long double maximum_velocity_rms = 0.0L;
    long double maximum_speed = 0.0L;
    long double energy_excess = 0.0L;
    long double momentum_residual = 0.0L;
    std::size_t maximum_components = 0U;
};

struct TrialObservables14 {
    std::string lane_root;
    std::uint32_t trial_index = 0U;
    std::string attempted_step_root;
    long double position_rms = 0.0L;
    long double position_maximum = 0.0L;
    long double velocity_rms = 0.0L;
    long double maximum_speed = 0.0L;
    long double energy_excess = 0.0L;
    long double momentum_residual = 0.0L;
    std::size_t components = 0U;
    std::uint64_t dynamic_count = 0U;
    std::uint64_t stable_id_count = 0U;
    std::uint64_t satellite_count = 0U;
    std::uint64_t positive_multiplier_count = 0U;
    long double total_mass = 0.0L;
    bool lower_contact = false;
    bool trial_invariants_ok = false;
    std::array<std::uint64_t, 6> predictor_first_hit_faces{};
    std::array<std::uint64_t, 6> predictor_clamp_faces{};
    std::vector<std::string> round_extra_roots;
    std::string state_root;
    std::string velocity_root;
    std::string root;
    Sequence14 next;
    bool actual_payload_finite = false;
    bool expected_payload_finite = false;
    bool pass = false;
};

std::string seal_trial_observables14(std::string_view lane_root,
    std::uint32_t trial_index, std::string_view attempted_step_root,
    TrialObservables14& observables, Work14& owner) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.trial-observables.v1");
    put_string(bytes, lane_root);
    put_u32(bytes, trial_index);
    put_string(bytes, attempted_step_root);
    put_string(bytes, observables.state_root);
    put_string(bytes, observables.velocity_root);
    for (const long double value : std::array<long double, 6>{
             observables.position_rms, observables.position_maximum,
             observables.velocity_rms, observables.maximum_speed,
             observables.energy_excess, observables.momentum_residual}) {
        put_f64(bytes, value);
    }
    put_u64(bytes, observables.dynamic_count);
    put_u64(bytes, observables.stable_id_count);
    put_u64(bytes, observables.components);
    put_u64(bytes, observables.satellite_count);
    put_u64(bytes, observables.positive_multiplier_count);
    put_f64(bytes, observables.total_mass);
    put_u8(bytes, observables.lower_contact ? 1U : 0U);
    put_u8(bytes, observables.trial_invariants_ok ? 1U : 0U);
    for (const std::uint64_t value : observables.predictor_first_hit_faces) {
        put_u64(bytes, value);
    }
    for (const std::uint64_t value : observables.predictor_clamp_faces) {
        put_u64(bytes, value);
    }
    put_u64(bytes, observables.round_extra_roots.size());
    for (const std::string& root : observables.round_extra_roots) {
        put_string(bytes, root);
    }
    owner.portable_content_fields_serialized +=
        33U + observables.round_extra_roots.size();
    ++owner.parent.hash_derivations;
    observables.root = root_of(std::move(bytes));
    return observables.root;
}

TrialObservables14 trial_observables(std::string_view lane_root,
    std::uint32_t trial_index, const State& initial,
    const Step14& wrapped, const NonlocalGpuProfile& profile,
    const Sequence14& prior, Work14& owner, Work14& expected_owner) {
    const StepResult& trial = wrapped.step;
    TrialObservables14 result;
    result.lane_root = lane_root;
    result.trial_index = trial_index;
    result.attempted_step_root = trial.result_root;
    long double position_sum = 0.0L;
    long double velocity_sum = 0.0L;
    for (std::size_t index = 0U; index < trial.state.id.size(); ++index) {
        const long double displacement = norm(
            trial.state.position[index] - initial.position[index]);
        const long double speed = norm(trial.state.velocity[index]);
        position_sum += displacement * displacement;
        velocity_sum += speed * speed;
        result.position_maximum = std::max(result.position_maximum,
            displacement);
        result.maximum_speed = std::max(result.maximum_speed, speed);
    }
    result.position_rms = std::sqrt(position_sum / trial.state.id.size());
    result.velocity_rms = std::sqrt(velocity_sum / trial.state.id.size());
    result.next = prior;
    result.next.maximum_position_rms = std::max(
        prior.maximum_position_rms, result.position_rms);
    result.next.maximum_position = std::max(
        prior.maximum_position, result.position_maximum);
    result.next.maximum_velocity_rms = std::max(
        prior.maximum_velocity_rms, result.velocity_rms);
    result.next.maximum_speed = std::max(prior.maximum_speed,
        result.maximum_speed);
    const long double initial_energy = energy(initial, profile);
    const long double current_energy = energy(trial.state, profile);
    const long double total_mass = static_cast<long double>(profile.mass)
        * initial.id.size();
    const Vec3l gravity = widen(profile.gravity);
    const long double denominator = std::max({std::abs(initial_energy),
        total_mass * norm(gravity) * profile.spacing, 1.0e-30L});
    result.next.energy_excess = std::max(prior.energy_excess,
        std::max(current_energy - initial_energy, 0.0L) / denominator);
    result.energy_excess = result.next.energy_excess;
    result.next.gravity_impulse = prior.gravity_impulse
        + gravity * (total_mass * profile.dt);
    Vec3l step_pressure;
    Vec3l step_contact;
    for (std::size_t index = 0U; index < trial.state.id.size(); ++index) {
        step_pressure += trial.pressure_jt_sum[index] * (-profile.dt);
        step_contact += trial.contact_impulse_sum[index];
    }
    result.next.pressure_impulse = prior.pressure_impulse + step_pressure;
    result.next.contact_impulse = prior.contact_impulse + step_contact;
    const Vec3l momentum_change = momentum(trial.state, profile.mass)
        - momentum(initial, profile.mass);
    const Vec3l momentum_error = momentum_change
        - result.next.gravity_impulse - result.next.pressure_impulse
        - result.next.contact_impulse;
    result.next.momentum_residual = norm(momentum_error)
        / std::max({norm(momentum_change), norm(result.next.gravity_impulse),
            norm(result.next.pressure_impulse),
            norm(result.next.contact_impulse), 1.0e-30L});
    result.momentum_residual = result.next.momentum_residual;
    result.components = trial.component_count;
    result.dynamic_count = trial.state.id.size();
    std::set<std::uint32_t> stable_ids(trial.state.id.begin(),
        trial.state.id.end());
    result.stable_id_count = stable_ids.size();
    result.satellite_count = wrapped.satellite_count;
    result.positive_multiplier_count = trial.positive_multipliers;
    result.total_mass = static_cast<long double>(profile.mass)
        * trial.state.id.size();
    result.lower_contact = wrapped.bottom_contact;
    result.trial_invariants_ok = wrapped.trial_invariants_ok;
    result.predictor_first_hit_faces = wrapped.predictor_first_hit_faces;
    result.predictor_clamp_faces = wrapped.predictor_clamp_faces;
    result.state_root = trial.state_root;
    result.velocity_root = trial.velocity_root;
    for (const RoundExtra14& extra : wrapped.extra_rounds) {
        result.round_extra_roots.push_back(extra.root);
    }
    result.next.maximum_components = std::max(prior.maximum_components,
        result.components);
    const std::array<long double, 7> finite_payload{
        result.position_rms, result.position_maximum,
        result.velocity_rms, result.maximum_speed,
        result.energy_excess, result.momentum_residual,
        result.total_mass};
    result.actual_payload_finite = true;
    for (const long double value : finite_payload) {
        combine_predicate(result.actual_payload_finite, owner,
            std::isfinite(value));
    }
    result.expected_payload_finite = true;
    for (const long double value : finite_payload) {
        result.expected_payload_finite = std::isfinite(value)
            && result.expected_payload_finite;
    }
    expected_owner.lane_scalar_comparisons += finite_payload.size();
    if (result.actual_payload_finite) {
        seal_trial_observables14(lane_root, trial_index,
            trial.result_root, result, owner);
    }
    if (result.expected_payload_finite) {
        ++expected_owner.parent.hash_derivations;
        expected_owner.portable_content_fields_serialized +=
            33U + result.round_extra_roots.size();
    }
    return result;
}

bool sequence_gates14(TrialObservables14& observables, Work14& owner) {
    bool pass = true;
    combine_predicate(pass, owner,
        observables.next.maximum_position_rms <= kPositionRmsLimit);
    combine_predicate(pass, owner,
        observables.next.maximum_position <= kPositionMaximumLimit);
    combine_predicate(pass, owner,
        observables.next.maximum_velocity_rms <= kVelocityRmsLimit);
    combine_predicate(pass, owner,
        observables.next.maximum_speed <= kVelocityMaximumLimit);
    combine_predicate(pass, owner,
        observables.next.energy_excess <= kEnergyLimit);
    combine_predicate(pass, owner,
        observables.next.momentum_residual <= kMomentumLimit);
    combine_predicate(pass, owner,
        observables.next.maximum_components == 1U);
    observables.pass = pass;
    return pass;
}

std::pair<Work14, Work14> lane_prefix_work14(const LaneResult14& lane) {
    Work14 actual = lane.work;
    Work14 expected = lane.expected_work;
    Work trajectory_work;
    Work expected_trajectory_work;
    for (std::size_t index = 0U; index < lane.trials.size(); ++index) {
        const Step14& trial = lane.trials[index];
        add_work14(actual, trial.extra_work);
        add_work14(expected, trial.expected_extra_work);
        if (trial.attempted_step_sealed) {
            add_work(trajectory_work, trial.step.work);
            add_work(expected_trajectory_work,
                trial.expected_parent_work);
        } else {
            add_parent_work(actual, trial.step.work);
            add_parent_work(expected, trial.expected_parent_work);
            actual.receipt_children_aggregated +=
                trial.step.rounds.size();
            expected.receipt_children_aggregated +=
                trial.step.rounds.size();
        }
    }
    for (const TrialSkip14& skip : lane.trial_skips) {
        add_child(actual, skip.work);
        add_child(expected, skip.expected_work);
    }
    add_parent_work(actual, trajectory_work);
    add_parent_work(expected, expected_trajectory_work);
    ++actual.receipt_children_aggregated;
    ++expected.receipt_children_aggregated;
    return {actual, expected};
}

bool lane_prefix_work_exact14(const LaneResult14& lane) {
    const auto prefix = lane_prefix_work14(lane);
    return same_work14(prefix.first, prefix.second);
}

bool prospective_trial_prefix_exact14(const LaneResult14& lane,
    const Step14& trial) {
    auto prefix = lane_prefix_work14(lane);
    add_parent_work(prefix.first, trial.step.work);
    add_parent_work(prefix.second, trial.expected_parent_work);
    add_work14(prefix.first, trial.extra_work);
    add_work14(prefix.second, trial.expected_extra_work);
    return same_work14(prefix.first, prefix.second)
        && (!trial.attempted_step_sealed
            || trial.step.work.hash_derivations
                == trial.step.expected_hash_derivations);
}

bool lane_trajectory_hash_exact14(const LaneResult14& lane) {
    std::uint64_t actual = 1U;
    std::uint64_t expected = 1U;
    for (const Step14& trial : lane.trials) {
        if (!trial.attempted_step_sealed) continue;
        actual += trial.step.work.hash_derivations;
        expected += trial.step.expected_hash_derivations;
    }
    return actual == expected;
}

void finalize_trajectory_v5(LaneResult14& lane, const State& initial_state,
    const State& proposed_state, const Sequence14& proposed_sequence,
    bool force_pair_rollback) {
    auto prefix = lane_prefix_work14(lane);
    Work14 actual = std::move(prefix.first);
    Work14 expected = std::move(prefix.second);
    TrajectoryResult trajectory = lane.trajectory;
    trajectory.work = {};
    trajectory.steps.clear();
    trajectory.step_roots.clear();
    trajectory.trial_committed.clear();
    trajectory.expected_hash_derivations = 1U;

    const WorkSeal14 preview = verify_work14(actual, expected);
    const bool rollback = force_pair_rollback || !preview.exact;
    const bool zero_step_trajectory = rollback || !lane.apparatus_valid;
    if (zero_step_trajectory) {
        for (const Step14& trial : lane.trials) {
            if (!trial.attempted_step_sealed) continue;
            ++actual.receipt_children_aggregated;
            ++expected.receipt_children_aggregated;
        }
    }
    const State& published_state = zero_step_trajectory
        ? initial_state : proposed_state;
    const Sequence14 sequence = zero_step_trajectory
        ? Sequence14{} : proposed_sequence;
    if (zero_step_trajectory) {
        const bool previously_valid = lane.apparatus_valid;
        if (lane.trial_2_attempted) {
            lane.trial_2_execution = "EXECUTED_REJECTED";
        }
        if (lane.rejected_step_roots.empty()) {
            for (auto trial = lane.trials.rbegin();
                 trial != lane.trials.rend(); ++trial) {
                if (!trial->attempted_step_sealed) continue;
                lane.rejected_step_roots.push_back(
                    trial->step.result_root);
                lane.failing_state_root = trial->step.state_root;
                lane.failing_velocity_root = trial->step.velocity_root;
                break;
            }
        }
        lane.apparatus_valid = false;
        lane.both_steps_supported = false;
        lane.physical_gate_rejected = false;
        lane.qp_sweep_cap_exhausted = false;
        lane.projection_round_cap_exhausted = false;
        lane.trial_1_committed = false;
        lane.trial_2_committed = false;
        lane.committed_trials = 0U;
        lane.route_category = "APPARATUS_INVALID";
        if (previously_valid) lane.failure_cause = "LANE_WORK_MISMATCH";
        lane.typed_route = lane.failure_cause;
    }
    if (!zero_step_trajectory) {
        for (const Step14& trial : lane.trials) {
            if (!trial.attempted_step_sealed) continue;
            add_work(trajectory.work, trial.step.work);
            trajectory.steps.push_back(trial.step);
            trajectory.step_roots.push_back(trial.step.result_root);
            trajectory.trial_committed.push_back(false);
            trajectory.expected_hash_derivations +=
                trial.step.expected_hash_derivations;
        }
    }
    trajectory.final_state = published_state;
    trajectory.accepted_steps = lane.committed_trials;
    for (std::size_t index = 0U;
        index < trajectory.trial_committed.size(); ++index) {
        trajectory.trial_committed[index] = !zero_step_trajectory
            && index < lane.committed_trials;
    }
    trajectory.apparatus_valid = lane.apparatus_valid;
    trajectory.physical_pass = lane.both_steps_supported;
    trajectory.failure = lane.failure_cause;
    if (!trajectory.failure.empty()) {
        trajectory.failure_step = lane.attempted_trials;
    }
    trajectory.maximum_position_rms = sequence.maximum_position_rms;
    trajectory.maximum_position = sequence.maximum_position;
    trajectory.maximum_velocity_rms = sequence.maximum_velocity_rms;
    trajectory.maximum_speed = sequence.maximum_speed;
    trajectory.energy_excess = sequence.energy_excess;
    trajectory.momentum_residual = sequence.momentum_residual;
    trajectory.maximum_components = sequence.maximum_components;
    for (std::size_t index = 0U;
         index < lane.trials.size() && index < lane.committed_trials;
         ++index) {
        if (lane.trials[index].positive_pressure) {
            ++trajectory.pressure_occurrences;
        }
        if (lane.trials[index].bottom_contact) {
            ++trajectory.lower_contact_occurrences;
        }
    }
    if (!lane.rejected_step_roots.empty()) {
        const std::string& rejected_root = lane.rejected_step_roots.front();
        trajectory.failing_trial_published = true;
        trajectory.failing_trial_outcome = lane.failure_cause;
        trajectory.failing_trial_result_root = rejected_root;
        for (std::size_t index = 0U; index < lane.trials.size(); ++index) {
            const Step14& trial = lane.trials[index];
            if (trial.step.result_root != rejected_root) continue;
            trajectory.failure_step = index + 1U;
            trajectory.failing_trial_step = index + 1U;
            trajectory.failing_trial_state_root = trial.step.state_root;
            trajectory.failing_trial_work_root = work_root(trial.step.work);
            break;
        }
        for (const TrialObservables14& observables :
             lane.trial_observables) {
            if (observables.attempted_step_root != rejected_root) continue;
            trajectory.failing_trial_observables_published = true;
            trajectory.failing_trial_gate_evaluated = true;
            trajectory.failing_trial_gate_pass = false;
            trajectory.failing_trial_position_rms = observables.position_rms;
            trajectory.failing_trial_position_maximum =
                observables.position_maximum;
            trajectory.failing_trial_velocity_rms = observables.velocity_rms;
            trajectory.failing_trial_maximum_speed =
                observables.maximum_speed;
            trajectory.failing_trial_energy_excess =
                observables.energy_excess;
            trajectory.failing_trial_momentum_residual =
                observables.momentum_residual;
            trajectory.failing_trial_components = observables.components;
            break;
        }
    }
    trajectory.state_root = state_root(
        "nextengine.nonlocal.ncgp13.trajectory-state.v1", published_state);
    ++trajectory.work.hash_derivations;
    ++actual.parent.hash_derivations;
    ++expected.parent.hash_derivations;
    trajectory.hash_accounting_exact = trajectory.work.hash_derivations
        == trajectory.expected_hash_derivations;
    if (!trajectory.hash_accounting_exact) {
        const bool previously_valid = lane.apparatus_valid;
        lane.apparatus_valid = false;
        lane.both_steps_supported = false;
        lane.route_category = "APPARATUS_INVALID";
        if (previously_valid) {
            lane.failure_cause = "TRAJECTORY_HASH_ACCOUNTING_INVALID";
        }
        lane.typed_route = lane.failure_cause;
    }
    trajectory.trajectory_root = trajectory_root(trajectory);
    lane.trajectory = std::move(trajectory);
    lane.trajectory_root = lane.trajectory.trajectory_root;
    lane.accepted_state_root = lane.trajectory.state_root;
    lane.accepted_velocity_root = id_vec_root(
        "nextengine.nonlocal.ncgp13.velocity.v1", published_state.id,
        published_state.velocity);
    ++actual.parent.hash_derivations;
    ++expected.parent.hash_derivations;
    actual.portable_content_fields_serialized +=
        2U + 4U * published_state.id.size();
    expected.portable_content_fields_serialized +=
        2U + 4U * published_state.id.size();
    lane.work = actual;
    lane.expected_work = expected;
}

struct LanePair14 {
    LaneResult14 canonical;
    LaneResult14 permuted;
    bool correspondence = true;
};

bool counted_trial_correspondence(const Step14& lhs, const Step14& rhs,
    Work14& owner) {
    bool pass = true;
    combine_predicate(pass, owner,
        lhs.step.result_root == rhs.step.result_root);
    combine_predicate(pass, owner,
        lhs.step.state_root == rhs.step.state_root);
    combine_predicate(pass, owner,
        lhs.step.velocity_root == rhs.step.velocity_root);
    combine_predicate(pass, owner,
        lhs.step.multiplier_root == rhs.step.multiplier_root);
    combine_predicate(pass, owner,
        lhs.step.contact_mask_root == rhs.step.contact_mask_root);
    combine_predicate(pass, owner,
        lhs.step.contact_impulse_root == rhs.step.contact_impulse_root);
    const bool parent_work_equal = counted_same_parent_work14(
        lhs.step.work, rhs.step.work, owner);
    const bool extra_work_equal = counted_same_work14(
        lhs.extra_work, rhs.extra_work, owner);
    pass = parent_work_equal && extra_work_equal && pass;
    combine_predicate(pass, owner,
        lhs.extra_rounds.size() == rhs.extra_rounds.size());
    const std::size_t round_count = std::min(
        lhs.extra_rounds.size(), rhs.extra_rounds.size());
    for (std::size_t index = 0U; index < round_count; ++index) {
        combine_predicate(pass, owner,
            lhs.extra_rounds[index].root == rhs.extra_rounds[index].root);
    }
    combine_predicate(pass, owner, lhs.typed_route == rhs.typed_route);
    return pass;
}

bool counted_preseal_correspondence14(const Step14& lhs,
    const Step14& rhs, Work14& owner) {
    bool pass = true;
    combine_predicate(pass, owner, lhs.semantic_roots_derived);
    combine_predicate(pass, owner, rhs.semantic_roots_derived);
    combine_predicate(pass, owner, lhs.trial_invariants_ok);
    combine_predicate(pass, owner, rhs.trial_invariants_ok);
    for (const bool predicate : std::array<bool, 5>{
             lhs.step.state_root == rhs.step.state_root,
             lhs.step.velocity_root == rhs.step.velocity_root,
             lhs.step.multiplier_root == rhs.step.multiplier_root,
             lhs.step.contact_mask_root == rhs.step.contact_mask_root,
             lhs.step.contact_impulse_root == rhs.step.contact_impulse_root}) {
        combine_predicate(pass, owner, predicate);
    }
    const bool parent_work_equal = counted_same_parent_work14(
        lhs.step.work, rhs.step.work, owner);
    const bool extra_work_equal = counted_same_work14(
        lhs.extra_work, rhs.extra_work, owner);
    pass = parent_work_equal && extra_work_equal && pass;
    combine_predicate(pass, owner,
        lhs.extra_rounds.size() == rhs.extra_rounds.size());
    const std::size_t round_count = std::min(
        lhs.extra_rounds.size(), rhs.extra_rounds.size());
    for (std::size_t index = 0U; index < round_count; ++index) {
        combine_predicate(pass, owner,
            lhs.extra_rounds[index].root == rhs.extra_rounds[index].root);
    }
    return pass;
}

void finalize_local_step14(const LanePolicy& policy, Step14& wrapped) {
    StepResult& result = wrapped.step;
    if (!wrapped.semantic_roots_derived
        || wrapped.accepted_post_graph.root.empty()) {
        result.apparatus_valid = false;
        result.failure = "GRAPH_ASSEMBLY_INVALID";
        wrapped.typed_route = result.failure;
        return;
    }
    const TopologyMetrics14 topology = topology_metrics14(
        wrapped.accepted_post_graph, result.work);
    add_work(wrapped.expected_parent_work,
        expected_topology_parent_work(wrapped.accepted_post_graph));
    result.component_count = topology.components;
    wrapped.satellite_count = topology.satellites;
    bool physical = true;
    combine_predicate(physical, wrapped.extra_work,
        result.maximum_strain <= kDensityMaximumLimit);
    combine_predicate(physical, wrapped.extra_work,
        result.rms_strain <= kDensityRmsLimit);
    combine_predicate(physical, wrapped.extra_work,
        result.penetration == 0.0L);
    combine_predicate(physical, wrapped.extra_work,
        result.independent_correspondence <= 2.0e-12L);
    combine_predicate(physical, wrapped.extra_work,
        result.component_count == 1U);
    wrapped.expected_extra_work.lane_scalar_comparisons += 5U;
    if (policy.tight) {
        combine_predicate(physical, wrapped.extra_work,
            wrapped.tight_support_ok);
        combine_predicate(physical, wrapped.extra_work,
            wrapped.top_contact_zero);
        combine_predicate(physical, wrapped.extra_work,
            wrapped.positive_pressure);
        combine_predicate(physical, wrapped.extra_work,
            wrapped.bottom_contact);
        wrapped.expected_extra_work.lane_scalar_comparisons += 4U;
    }
    result.physical_pass = physical;
    if (!physical) {
        result.failure = "PHYSICAL_TRIAL_GATE_REJECTED";
        wrapped.typed_route = result.failure;
    } else {
        result.failure.clear();
        wrapped.typed_route = "TRIAL_SUPPORTED";
    }
    seal_attempted_step14(wrapped);
}

bool lane_acceptance14(bool tight, std::uint32_t trial_index,
    bool numerical_gates, bool cumulative_pressure, bool cumulative_lower,
    bool trial_pressure, bool trial_lower) {
    if (!numerical_gates) return false;
    if (tight) return trial_pressure && trial_lower;
    if (trial_index == 1U) return true;
    return cumulative_pressure && cumulative_lower;
}

bool full_lane_acceptance14(bool tight, bool numerical_gates,
    bool step1_pressure, bool step1_lower, bool step2_pressure,
    bool step2_lower, Work14* owner = nullptr) {
    bool result = false;
    if (!numerical_gates) {
        result = false;
    } else if (tight) {
        result = step1_pressure && step1_lower
            && step2_pressure && step2_lower;
    } else {
        result = (step1_pressure || step2_pressure)
            && (step1_lower || step2_lower);
    }
    return owner == nullptr ? result : counted_predicate(*owner, result);
}

struct ProductionTransaction14 {
    State accepted_state;
    Scratch14 scratch;
    bool force_after_private_observable_seal_armed = false;
    bool hook_fired = false;
};

void stage_private_trial14(ProductionTransaction14& transaction,
    const Step14& private_step) {
    transaction.scratch = private_step.scratch;
}

void rollback14(ProductionTransaction14& transaction,
    const State& prior) {
    transaction.accepted_state = prior;
    clear_scratch14(transaction.scratch);
}

bool settle_private_trial14(ProductionTransaction14& transaction,
    const State& prior, const Step14& private_step, bool commit) {
    if (commit && transaction.force_after_private_observable_seal_armed) {
        transaction.hook_fired = true;
        rollback14(transaction, prior);
        return false;
    }
    if (commit) {
        transaction.accepted_state = private_step.step.state;
        clear_scratch14(transaction.scratch);
        return true;
    }
    rollback14(transaction, prior);
    return false;
}

#if 0
LanePair14 run_lane_pair_rev4_legacy(const LanePolicy& policy) {
    LanePair14 pair;
    pair.canonical.policy = policy;
    pair.permuted.policy = policy;
    ProductionTransaction14 canonical_transaction;
    ProductionTransaction14 permuted_transaction;
    canonical_transaction.accepted_state = working_state14(
        policy.fixture->canonical_samples, pair.canonical.work);
    permuted_transaction.accepted_state = working_state14(
        policy.fixture->permuted_samples, pair.permuted.work);
    pair.canonical.expected_work.records_canonicalized +=
        policy.fixture->canonical_samples.size();
    pair.permuted.expected_work.records_canonicalized +=
        policy.fixture->permuted_samples.size();
    const State canonical_initial = canonical_transaction.accepted_state;
    const State permuted_initial = permuted_transaction.accepted_state;
    const GhostGrid grid = make_ghost_grid(policy.fixture->profile,
        policy.fixture->ghosts);
    Sequence14 canonical_sequence;
    Sequence14 permuted_sequence;
    bool cumulative_pressure = false;
    bool cumulative_lower = false;
    std::array<bool, 2> trial_pressure{};
    std::array<bool, 2> trial_lower{};
    std::uint64_t observable_gate_schedules = 0U;
    std::uint64_t saturation_schedules = 0U;
    for (std::uint32_t trial_index = 0U;
         trial_index < policy.maximum_steps; ++trial_index) {
        const State canonical_prior = canonical_transaction.accepted_state;
        const State permuted_prior = permuted_transaction.accepted_state;
        Step14 canonical = run_step14(policy,
            canonical_transaction.accepted_state,
            policy.fixture->ghosts, grid, trial_index + 1U);
        Step14 permuted = run_step14(policy,
            permuted_transaction.accepted_state,
            policy.fixture->ghosts, grid, trial_index + 1U);
        stage_private_trial14(canonical_transaction, canonical);
        stage_private_trial14(permuted_transaction, permuted);
        ++pair.canonical.attempted_trials;
        ++pair.permuted.attempted_trials;
        if (trial_index == 1U) {
            pair.canonical.trial_2_attempted = true;
            pair.permuted.trial_2_attempted = true;
            pair.canonical.trial_2_execution = "EXECUTED_REJECTED";
            pair.permuted.trial_2_execution = "EXECUTED_REJECTED";
        }
        pair.canonical.attempted_step_roots.push_back(
            canonical.step.result_root);
        pair.permuted.attempted_step_roots.push_back(
            permuted.step.result_root);
        for (const RoundSummary& round : canonical.step.rounds) {
            pair.canonical.round_roots.push_back(round.result_root);
        }
        for (const RoundExtra14& extra : canonical.extra_rounds) {
            pair.canonical.round_extra_roots.push_back(extra.root);
        }
        for (const RoundSummary& round : permuted.step.rounds) {
            pair.permuted.round_roots.push_back(round.result_root);
        }
        for (const RoundExtra14& extra : permuted.extra_rounds) {
            pair.permuted.round_extra_roots.push_back(extra.root);
        }
        add_work(pair.canonical.trajectory.work, canonical.step.work);
        add_work(pair.permuted.trajectory.work, permuted.step.work);
        pair.canonical.trajectory.steps.push_back(canonical.step);
        pair.permuted.trajectory.steps.push_back(permuted.step);
        pair.canonical.trajectory.step_roots.push_back(
            canonical.step.result_root);
        pair.permuted.trajectory.step_roots.push_back(
            permuted.step.result_root);
        pair.canonical.trajectory.trial_committed.push_back(false);
        pair.permuted.trajectory.trial_committed.push_back(false);
        const bool canonical_correspondence = counted_trial_correspondence(
            canonical, permuted, pair.canonical.work);
        const bool permuted_correspondence = counted_trial_correspondence(
            permuted, canonical, pair.permuted.work);
        pair.correspondence = canonical_correspondence
            && permuted_correspondence && pair.correspondence;
        bool canonical_apparatus = true;
        combine_predicate(canonical_apparatus, pair.canonical.work,
            canonical.step.apparatus_valid);
        combine_predicate(canonical_apparatus, pair.canonical.work,
            permuted.step.apparatus_valid);
        combine_predicate(canonical_apparatus, pair.canonical.work,
            pair.correspondence);
        bool permuted_apparatus = true;
        combine_predicate(permuted_apparatus, pair.permuted.work,
            permuted.step.apparatus_valid);
        combine_predicate(permuted_apparatus, pair.permuted.work,
            canonical.step.apparatus_valid);
        combine_predicate(permuted_apparatus, pair.permuted.work,
            pair.correspondence);
        const bool canonical_qp = counted_predicate(pair.canonical.work,
            canonical.qp_cap_exhausted);
        const bool canonical_projection = counted_predicate(
            pair.canonical.work, canonical.projection_cap_exhausted);
        const bool canonical_accepted = counted_predicate(pair.canonical.work,
            canonical.step.accepted);
        const bool canonical_local_physical = counted_predicate(
            pair.canonical.work, canonical.step.physical_pass);
        const bool permuted_qp = counted_predicate(pair.permuted.work,
            permuted.qp_cap_exhausted);
        const bool permuted_projection = counted_predicate(
            pair.permuted.work, permuted.projection_cap_exhausted);
        const bool permuted_accepted = counted_predicate(pair.permuted.work,
            permuted.step.accepted);
        const bool permuted_local_physical = counted_predicate(
            pair.permuted.work, permuted.step.physical_pass);
        const bool apparatus = canonical_apparatus && permuted_apparatus;
        if (!apparatus) {
            pair.canonical.apparatus_valid = false;
            pair.permuted.apparatus_valid = false;
            pair.canonical.typed_route = pair.correspondence
                ? canonical.typed_route : "PERMUTATION_CORRESPONDENCE_INVALID";
            pair.permuted.typed_route = pair.canonical.typed_route;
        } else if (canonical_qp || canonical_projection
            || permuted_qp || permuted_projection) {
            pair.canonical.qp_sweep_cap_exhausted =
                canonical.qp_cap_exhausted;
            pair.permuted.qp_sweep_cap_exhausted =
                permuted.qp_cap_exhausted;
            pair.canonical.projection_round_cap_exhausted =
                canonical.projection_cap_exhausted;
            pair.permuted.projection_round_cap_exhausted =
                permuted.projection_cap_exhausted;
            pair.canonical.typed_route = canonical.typed_route;
            pair.permuted.typed_route = permuted.typed_route;
        } else {
            ++observable_gate_schedules;
            bool canonical_observable_stage = true;
            combine_predicate(canonical_observable_stage,
                pair.canonical.work, canonical_accepted);
            combine_predicate(canonical_observable_stage,
                pair.canonical.work, permuted_accepted);
            combine_predicate(canonical_observable_stage,
                pair.canonical.work, canonical.trial_invariants_ok);
            combine_predicate(canonical_observable_stage,
                pair.canonical.work, permuted.trial_invariants_ok);
            bool permuted_observable_stage = true;
            combine_predicate(permuted_observable_stage,
                pair.permuted.work, permuted_accepted);
            combine_predicate(permuted_observable_stage,
                pair.permuted.work, canonical_accepted);
            combine_predicate(permuted_observable_stage,
                pair.permuted.work, permuted.trial_invariants_ok);
            combine_predicate(permuted_observable_stage,
                pair.permuted.work, canonical.trial_invariants_ok);
            const bool observable_stage = canonical_observable_stage
                && permuted_observable_stage;
            if (!observable_stage) {
                pair.canonical.physical_gate_rejected = true;
                pair.permuted.physical_gate_rejected = true;
                pair.canonical.typed_route = canonical.typed_route;
                pair.permuted.typed_route = permuted.typed_route;
            } else {
            TrialObservables14 canonical_observables = trial_observables(
                policy.root, trial_index + 1U, canonical_initial, canonical,
                policy.fixture->profile, canonical_sequence,
                pair.canonical.work);
            TrialObservables14 permuted_observables = trial_observables(
                policy.root, trial_index + 1U, permuted_initial, permuted,
                policy.fixture->profile, permuted_sequence,
                pair.permuted.work);
            pair.canonical.trial_observable_roots.push_back(
                canonical_observables.root);
            pair.permuted.trial_observable_roots.push_back(
                permuted_observables.root);
            bool canonical_observables_equal = true;
            bool permuted_observables_equal = true;
            const std::array<bool, 8> equalities{
                canonical_observables.position_rms
                    == permuted_observables.position_rms,
                canonical_observables.position_maximum
                    == permuted_observables.position_maximum,
                canonical_observables.velocity_rms
                    == permuted_observables.velocity_rms,
                canonical_observables.maximum_speed
                    == permuted_observables.maximum_speed,
                canonical_observables.energy_excess
                    == permuted_observables.energy_excess,
                canonical_observables.momentum_residual
                    == permuted_observables.momentum_residual,
                canonical_observables.components
                    == permuted_observables.components,
                canonical_observables.root
                    == permuted_observables.root};
            for (const bool equality : equalities) {
                combine_predicate(canonical_observables_equal,
                    pair.canonical.work, equality);
                combine_predicate(permuted_observables_equal,
                    pair.permuted.work, equality);
            }
            const bool observables_equal = canonical_observables_equal
                && permuted_observables_equal;
            pair.correspondence = pair.correspondence && observables_equal;
            if (!observables_equal) {
                pair.canonical.apparatus_valid = false;
                pair.permuted.apparatus_valid = false;
                pair.canonical.typed_route =
                    "PERMUTATION_CORRESPONDENCE_INVALID";
                pair.permuted.typed_route = pair.canonical.typed_route;
            } else {
                const bool canonical_sequence_pass = sequence_gates14(
                    canonical_observables, pair.canonical.work);
                const bool permuted_sequence_pass = sequence_gates14(
                    permuted_observables, pair.permuted.work);
                const bool next_pressure = cumulative_pressure
                    || canonical.positive_pressure;
                const bool next_lower = cumulative_lower
                    || canonical.bottom_contact;
                const bool canonical_commit = counted_predicate(
                    pair.canonical.work, lane_acceptance14(policy.tight,
                        trial_index + 1U,
                        canonical_local_physical && canonical_sequence_pass,
                        next_pressure, next_lower, canonical.positive_pressure,
                        canonical.bottom_contact));
                const bool permuted_commit = counted_predicate(
                    pair.permuted.work, lane_acceptance14(policy.tight,
                        trial_index + 1U,
                        permuted_local_physical && permuted_sequence_pass,
                        next_pressure, next_lower, permuted.positive_pressure,
                        permuted.bottom_contact));
                if (!canonical_commit || !permuted_commit) {
                pair.canonical.physical_gate_rejected = true;
                pair.permuted.physical_gate_rejected = true;
                pair.canonical.typed_route = "PHYSICAL_SEQUENCE_GATE_REJECTED";
                pair.permuted.typed_route = pair.canonical.typed_route;
                auto publish = [&](LaneResult14& lane,
                                   const TrialObservables14& observables) {
                    TrajectoryResult& trajectory = lane.trajectory;
                    trajectory.failure = "TRAJECTORY_GATE_FAILED";
                    trajectory.failure_step = trial_index + 1U;
                    trajectory.failing_trial_published = true;
                    trajectory.failing_trial_observables_published = true;
                    trajectory.failing_trial_gate_evaluated = true;
                    trajectory.failing_trial_step = trial_index + 1U;
                    trajectory.failing_trial_outcome = lane.typed_route;
                    trajectory.failing_trial_position_rms =
                        observables.position_rms;
                    trajectory.failing_trial_position_maximum =
                        observables.position_maximum;
                    trajectory.failing_trial_velocity_rms =
                        observables.velocity_rms;
                    trajectory.failing_trial_maximum_speed =
                        observables.maximum_speed;
                    trajectory.failing_trial_energy_excess =
                        observables.energy_excess;
                    trajectory.failing_trial_momentum_residual =
                        observables.momentum_residual;
                    trajectory.failing_trial_components =
                        observables.components;
                };
                publish(pair.canonical, canonical_observables);
                publish(pair.permuted, permuted_observables);
                } else {
                bool saturation_equal = true;
                if (trial_index == 1U) {
                    ++saturation_schedules;
                    const bool canonical_extra_present = counted_predicate(
                        pair.canonical.work, !canonical.extra_rounds.empty());
                    const bool canonical_round_count = counted_predicate(
                        pair.canonical.work,
                        canonical.step.rounds.size()
                            == policy.projection_cap);
                    const bool canonical_last_closed = counted_predicate(
                        pair.canonical.work, canonical_extra_present
                            ? canonical.extra_rounds.back().round_closed
                            : false);
                    const bool canonical_saturation = canonical_extra_present
                        && canonical_round_count && canonical_last_closed;
                    const bool permuted_extra_present = counted_predicate(
                        pair.permuted.work, !permuted.extra_rounds.empty());
                    const bool permuted_round_count = counted_predicate(
                        pair.permuted.work,
                        permuted.step.rounds.size()
                            == policy.projection_cap);
                    const bool permuted_last_closed = counted_predicate(
                        pair.permuted.work, permuted_extra_present
                            ? permuted.extra_rounds.back().round_closed
                            : false);
                    const bool permuted_saturation = permuted_extra_present
                        && permuted_round_count && permuted_last_closed;
                    const bool canonical_equal = counted_predicate(
                        pair.canonical.work,
                        canonical_saturation == permuted_saturation);
                    const bool permuted_equal = counted_predicate(
                        pair.permuted.work,
                        permuted_saturation == canonical_saturation);
                    saturation_equal = canonical_equal && permuted_equal;
                    pair.canonical.projection_cap_saturated =
                        canonical_saturation;
                    pair.permuted.projection_cap_saturated =
                        permuted_saturation;
                }
                if (!saturation_equal) {
                    pair.correspondence = false;
                    pair.canonical.apparatus_valid = false;
                    pair.permuted.apparatus_valid = false;
                    pair.canonical.typed_route =
                        "PERMUTATION_CORRESPONDENCE_INVALID";
                    pair.permuted.typed_route = pair.canonical.typed_route;
                } else {
                static_cast<void>(settle_private_trial14(
                    canonical_transaction, canonical_prior, canonical,
                    true, false));
                static_cast<void>(settle_private_trial14(
                    permuted_transaction, permuted_prior, permuted,
                    true, false));
                canonical_sequence = canonical_observables.next;
                permuted_sequence = permuted_observables.next;
                ++pair.canonical.committed_trials;
                ++pair.permuted.committed_trials;
                pair.canonical.trajectory.trial_committed.back() = true;
                pair.permuted.trajectory.trial_committed.back() = true;
                if (canonical.positive_pressure) {
                    ++pair.canonical.trajectory.pressure_occurrences;
                }
                if (permuted.positive_pressure) {
                    ++pair.permuted.trajectory.pressure_occurrences;
                }
                if (canonical.bottom_contact) {
                    ++pair.canonical.trajectory.lower_contact_occurrences;
                }
                if (permuted.bottom_contact) {
                    ++pair.permuted.trajectory.lower_contact_occurrences;
                }
                if (trial_index == 0U) {
                    pair.canonical.trial_1_committed = true;
                    pair.permuted.trial_1_committed = true;
                } else {
                    pair.canonical.trial_2_committed = true;
                    pair.permuted.trial_2_committed = true;
                }
                cumulative_pressure = next_pressure;
                cumulative_lower = next_lower;
                trial_pressure[trial_index] = canonical.positive_pressure;
                trial_lower[trial_index] = canonical.bottom_contact;
                }
                }
            }
            pair.canonical.trial_observables.push_back(
                std::move(canonical_observables));
            pair.permuted.trial_observables.push_back(
                std::move(permuted_observables));
            }
        }
        pair.canonical.trials.push_back(std::move(canonical));
        pair.permuted.trials.push_back(std::move(permuted));
        if (pair.canonical.committed_trials != trial_index + 1U) {
            static_cast<void>(settle_private_trial14(canonical_transaction,
                canonical_prior, pair.canonical.trials.back(), false, false));
            static_cast<void>(settle_private_trial14(permuted_transaction,
                permuted_prior, pair.permuted.trials.back(), false, false));
        }
        if (trial_index == 1U) {
            pair.canonical.trial_2_attempted = true;
            pair.permuted.trial_2_attempted = true;
            pair.canonical.trial_2_execution =
                pair.canonical.trial_2_committed
                    ? "EXECUTED_COMMITTED" : "EXECUTED_REJECTED";
            pair.permuted.trial_2_execution =
                pair.permuted.trial_2_committed
                    ? "EXECUTED_COMMITTED" : "EXECUTED_REJECTED";
        }
        if (pair.canonical.committed_trials != trial_index + 1U) {
            pair.canonical.rejected_step_roots.push_back(
                pair.canonical.attempted_step_roots.back());
            pair.permuted.rejected_step_roots.push_back(
                pair.permuted.attempted_step_roots.back());
            pair.canonical.failing_state_root =
                pair.canonical.trials.back().step.state_root;
            pair.permuted.failing_state_root =
                pair.permuted.trials.back().step.state_root;
            pair.canonical.failing_velocity_root =
                pair.canonical.trials.back().step.velocity_root;
            pair.permuted.failing_velocity_root =
                pair.permuted.trials.back().step.velocity_root;
            if (trial_index == 0U) {
                pair.canonical.trial_skips.push_back(trial_skip14(
                    policy.root, 2U, "NOT_RUN_PRIOR_TRIAL_REJECTED"));
                pair.permuted.trial_skips.push_back(trial_skip14(
                    policy.root, 2U, "NOT_RUN_PRIOR_TRIAL_REJECTED"));
                pair.canonical.trial_skip_roots.push_back(
                    pair.canonical.trial_skips.back().result_root);
                pair.permuted.trial_skip_roots.push_back(
                    pair.permuted.trial_skips.back().result_root);
                pair.canonical.trial_2_execution =
                    "NOT_RUN_PRIOR_TRIAL_REJECTED";
                pair.permuted.trial_2_execution =
                    "NOT_RUN_PRIOR_TRIAL_REJECTED";
            }
            break;
        }
    }
    pair.canonical.both_steps_supported = full_lane_acceptance14(policy.tight,
            pair.canonical.committed_trials == 2U, trial_pressure[0],
            trial_lower[0], trial_pressure[1], trial_lower[1],
            &pair.canonical.work);
    pair.permuted.both_steps_supported = full_lane_acceptance14(policy.tight,
            pair.permuted.committed_trials == 2U, trial_pressure[0],
            trial_lower[0], trial_pressure[1], trial_lower[1],
            &pair.permuted.work);
    if (pair.canonical.both_steps_supported) {
        pair.canonical.typed_route = "BOTH_STEPS_SUPPORTED";
        pair.permuted.typed_route = "BOTH_STEPS_SUPPORTED";
    } else {
        auto publish_failure = [](LaneResult14& lane) {
            if (lane.trials.empty()) return;
            TrajectoryResult& trajectory = lane.trajectory;
            const Step14& failed = lane.trials.back();
            trajectory.failure = lane.typed_route;
            trajectory.failure_step = lane.attempted_trials;
            trajectory.failing_trial_published = true;
            trajectory.failing_trial_step = lane.attempted_trials;
            trajectory.failing_trial_outcome = lane.typed_route;
            trajectory.failing_trial_state_root = failed.step.state_root;
            trajectory.failing_trial_work_root = work_root(failed.step.work);
            trajectory.failing_trial_result_root = failed.step.result_root;
        };
        publish_failure(pair.canonical);
        publish_failure(pair.permuted);
    }
    std::uint64_t round_correspondence = 0U;
    const std::size_t paired_trials = std::min(
        pair.canonical.trials.size(), pair.permuted.trials.size());
    for (std::size_t index = 0U; index < paired_trials; ++index) {
        round_correspondence += std::min(
            pair.canonical.trials[index].extra_rounds.size(),
            pair.permuted.trials[index].extra_rounds.size());
    }
    const std::uint64_t common_orchestration_comparisons =
        round_correspondence + 4U * observable_gate_schedules
        + 4U * saturation_schedules + 1U;
    const std::uint64_t canonical_orchestration_comparisons =
        77U * pair.canonical.attempted_trials
        + 16U * pair.canonical.trial_observables.size()
        + common_orchestration_comparisons;
    const std::uint64_t permuted_orchestration_comparisons =
        77U * pair.permuted.attempted_trials
        + 16U * pair.permuted.trial_observables.size()
        + common_orchestration_comparisons;
    pair.canonical.expected_work.lane_scalar_comparisons +=
        canonical_orchestration_comparisons;
    pair.permuted.expected_work.lane_scalar_comparisons +=
        permuted_orchestration_comparisons;
    finalize_trajectory(pair.canonical, canonical_transaction.accepted_state,
        canonical_sequence);
    finalize_trajectory(pair.permuted, permuted_transaction.accepted_state,
        permuted_sequence);
    pair.canonical.category_closure = lane_category14(
        pair.canonical, &pair.canonical.work);
    pair.permuted.category_closure = lane_category14(
        pair.permuted, &pair.permuted.work);
    pair.canonical.category = pair.canonical.category_closure.value;
    pair.permuted.category = pair.permuted.category_closure.value;
    pair.canonical.expected_work.lane_scalar_comparisons += 6U;
    pair.permuted.expected_work.lane_scalar_comparisons += 6U;
    if (pair.canonical.category_closure.apparatus_was_valid
        && !pair.canonical.category_closure.one_hot) {
        pair.canonical.apparatus_valid = false;
        pair.canonical.both_steps_supported = false;
        pair.canonical.typed_route = "LANE_CATEGORY_INVALID";
    }
    if (pair.permuted.category_closure.apparatus_was_valid
        && !pair.permuted.category_closure.one_hot) {
        pair.permuted.apparatus_valid = false;
        pair.permuted.both_steps_supported = false;
        pair.permuted.typed_route = "LANE_CATEGORY_INVALID";
    }
    const bool canonical_category_equal = counted_predicate(
        pair.canonical.work,
        pair.canonical.category == pair.permuted.category);
    const bool permuted_category_equal = counted_predicate(
        pair.permuted.work,
        pair.permuted.category == pair.canonical.category);
    ++pair.canonical.expected_work.lane_scalar_comparisons;
    ++pair.permuted.expected_work.lane_scalar_comparisons;
    pair.correspondence = canonical_category_equal
        && permuted_category_equal && pair.correspondence;
    if (!pair.correspondence) {
        pair.canonical.apparatus_valid = false;
        pair.permuted.apparatus_valid = false;
        pair.canonical.both_steps_supported = false;
        pair.permuted.both_steps_supported = false;
        pair.canonical.typed_route = "PERMUTATION_CORRESPONDENCE_INVALID";
        pair.permuted.typed_route = pair.canonical.typed_route;
    }
    lane_result_root(pair.canonical);
    lane_result_root(pair.permuted);
    return pair;
}
#endif

LanePair14 run_lane_pair(const LanePolicy& policy) {
    LanePair14 pair;
    pair.canonical.policy = policy;
    pair.permuted.policy = policy;
    ProductionTransaction14 canonical_transaction;
    ProductionTransaction14 permuted_transaction;
    canonical_transaction.accepted_state = working_state14(
        policy.fixture->canonical_samples, pair.canonical.work);
    permuted_transaction.accepted_state = working_state14(
        policy.fixture->permuted_samples, pair.permuted.work);
    pair.canonical.expected_work.records_canonicalized +=
        policy.fixture->canonical_samples.size();
    pair.permuted.expected_work.records_canonicalized +=
        policy.fixture->permuted_samples.size();
    const State canonical_initial = canonical_transaction.accepted_state;
    const State permuted_initial = permuted_transaction.accepted_state;
    const GhostGrid grid = make_ghost_grid(
        policy.fixture->profile, policy.fixture->ghosts);
    Sequence14 canonical_sequence;
    Sequence14 permuted_sequence;
    bool cumulative_pressure = false;
    bool cumulative_lower = false;
    std::array<bool, 2> trial_pressure{};
    std::array<bool, 2> trial_lower{};

    auto publish_early = [](LaneResult14& lane, const Step14& trial,
                            std::uint32_t one_based_trial) {
        const std::size_t count = std::min({trial.early_child_stages.size(),
            trial.early_child_roots.size(),
            trial.early_child_rounds.size()});
        for (std::size_t index = 0U; index < count; ++index) {
            if (trial.early_child_roots[index].empty()) continue;
            lane.reached_children.push_back({one_based_trial,
                trial.early_child_rounds[index],
                trial.early_child_stages[index],
                trial.early_child_roots[index]});
        }
    };
    auto append_sealed = [](LaneResult14& lane, const Step14& trial,
                            bool committed) {
        for (const RoundSummary& round : trial.step.rounds) {
            lane.round_roots.push_back(round.result_root);
        }
        for (const RoundExtra14& extra : trial.extra_rounds) {
            lane.round_extra_roots.push_back(extra.root);
        }
        if (!trial.attempted_step_sealed) return;
        lane.attempted_step_roots.push_back(trial.step.result_root);
        if (!committed) {
            lane.rejected_step_roots.push_back(trial.step.result_root);
            lane.failing_state_root = trial.step.state_root;
            lane.failing_velocity_root = trial.step.velocity_root;
        }
    };
    auto add_trial_skip = [&](std::string cause) {
        pair.canonical.trial_skips.push_back(trial_skip14(
            policy.root, 2U, cause));
        pair.permuted.trial_skips.push_back(trial_skip14(
            policy.root, 2U, cause));
        pair.canonical.trial_skip_roots.push_back(
            pair.canonical.trial_skips.back().result_root);
        pair.permuted.trial_skip_roots.push_back(
            pair.permuted.trial_skips.back().result_root);
        pair.canonical.trial_2_execution = cause;
        pair.permuted.trial_2_execution = cause;
    };

    bool apparatus_failure = false;
    for (std::uint32_t trial_index = 0U;
         trial_index < policy.maximum_steps; ++trial_index) {
        const State canonical_prior = canonical_transaction.accepted_state;
        const State permuted_prior = permuted_transaction.accepted_state;
        Step14 canonical = run_step14(policy, canonical_prior,
            policy.fixture->ghosts, grid, trial_index + 1U);
        Step14 permuted = run_step14(policy, permuted_prior,
            policy.fixture->ghosts, grid, trial_index + 1U);
        stage_private_trial14(canonical_transaction, canonical);
        stage_private_trial14(permuted_transaction, permuted);
        ++pair.canonical.attempted_trials;
        ++pair.permuted.attempted_trials;
        if (trial_index == 1U) {
            pair.canonical.trial_2_attempted = true;
            pair.permuted.trial_2_attempted = true;
            pair.canonical.trial_2_execution = "EXECUTED_REJECTED";
            pair.permuted.trial_2_execution = "EXECUTED_REJECTED";
        }

        const bool early_apparatus = !canonical.step.apparatus_valid
            || !permuted.step.apparatus_valid;
        if (early_apparatus) {
            apparatus_failure = true;
            pair.correspondence = canonical.typed_route
                == permuted.typed_route;
            pair.canonical.apparatus_valid = false;
            pair.permuted.apparatus_valid = false;
            pair.canonical.route_category = "APPARATUS_INVALID";
            pair.permuted.route_category = "APPARATUS_INVALID";
            pair.canonical.failure_cause = pair.correspondence
                ? canonical.typed_route
                : "PERMUTATION_CORRESPONDENCE_INVALID";
            pair.permuted.failure_cause = pair.canonical.failure_cause;
            pair.canonical.typed_route = pair.canonical.failure_cause;
            pair.permuted.typed_route = pair.permuted.failure_cause;
            append_sealed(pair.canonical, canonical, false);
            append_sealed(pair.permuted, permuted, false);
            publish_early(pair.canonical, canonical, trial_index + 1U);
            publish_early(pair.permuted, permuted, trial_index + 1U);
            rollback14(canonical_transaction, canonical_initial);
            rollback14(permuted_transaction, permuted_initial);
            pair.canonical.trials.push_back(std::move(canonical));
            pair.permuted.trials.push_back(std::move(permuted));
            if (trial_index == 0U) {
                add_trial_skip("NOT_RUN_BY_PRECEDENCE");
            }
            break;
        }

        const bool budget_route = canonical.qp_cap_exhausted
            || canonical.projection_cap_exhausted
            || permuted.qp_cap_exhausted
            || permuted.projection_cap_exhausted;
        if (budget_route) {
            const bool canonical_equal = counted_trial_correspondence(
                canonical, permuted, pair.canonical.work);
            const bool permuted_equal = counted_trial_correspondence(
                permuted, canonical, pair.permuted.work);
            const std::uint64_t rounds = std::min(
                canonical.extra_rounds.size(),
                permuted.extra_rounds.size());
            pair.canonical.expected_work.lane_scalar_comparisons +=
                70U + rounds;
            pair.permuted.expected_work.lane_scalar_comparisons +=
                70U + rounds;
            pair.correspondence = canonical_equal && permuted_equal;
            if (!pair.correspondence) {
                apparatus_failure = true;
                pair.canonical.apparatus_valid = false;
                pair.permuted.apparatus_valid = false;
                pair.canonical.route_category = "APPARATUS_INVALID";
                pair.permuted.route_category = "APPARATUS_INVALID";
                pair.canonical.failure_cause =
                    "PERMUTATION_CORRESPONDENCE_INVALID";
                pair.permuted.failure_cause = pair.canonical.failure_cause;
                rollback14(canonical_transaction, canonical_initial);
                rollback14(permuted_transaction, permuted_initial);
            } else {
                pair.canonical.qp_sweep_cap_exhausted =
                    canonical.qp_cap_exhausted;
                pair.permuted.qp_sweep_cap_exhausted =
                    permuted.qp_cap_exhausted;
                pair.canonical.projection_round_cap_exhausted =
                    canonical.projection_cap_exhausted;
                pair.permuted.projection_round_cap_exhausted =
                    permuted.projection_cap_exhausted;
                pair.canonical.route_category = canonical.qp_cap_exhausted
                    ? "QP_SWEEP_CAP_EXHAUSTED"
                    : "PROJECTION_ROUND_CAP_EXHAUSTED";
                pair.permuted.route_category =
                    pair.canonical.route_category;
                pair.canonical.category = canonical.qp_cap_exhausted
                    ? LaneCategory14::QpSweepCapExhausted
                    : LaneCategory14::ProjectionRoundCapExhausted;
                pair.permuted.category = pair.canonical.category;
                pair.canonical.failure_cause =
                    pair.canonical.route_category;
                pair.permuted.failure_cause = pair.canonical.failure_cause;
                pair.canonical.typed_route = pair.canonical.failure_cause;
                pair.permuted.typed_route = pair.permuted.failure_cause;
                rollback14(canonical_transaction, canonical_prior);
                rollback14(permuted_transaction, permuted_prior);
            }
            append_sealed(pair.canonical, canonical, false);
            append_sealed(pair.permuted, permuted, false);
            publish_early(pair.canonical, canonical, trial_index + 1U);
            publish_early(pair.permuted, permuted, trial_index + 1U);
            pair.canonical.trials.push_back(std::move(canonical));
            pair.permuted.trials.push_back(std::move(permuted));
            if (trial_index == 0U) {
                add_trial_skip(apparatus_failure
                    ? "NOT_RUN_BY_PRECEDENCE"
                    : "NOT_RUN_PRIOR_TRIAL_REJECTED");
            }
            break;
        }

        const bool canonical_preseal = counted_preseal_correspondence14(
            canonical, permuted, pair.canonical.work);
        const bool permuted_preseal = counted_preseal_correspondence14(
            permuted, canonical, pair.permuted.work);
        const std::uint64_t preseal_comparisons = 72U + std::min(
            canonical.extra_rounds.size(), permuted.extra_rounds.size());
        pair.canonical.expected_work.lane_scalar_comparisons +=
            preseal_comparisons;
        pair.permuted.expected_work.lane_scalar_comparisons +=
            preseal_comparisons;
        if (!canonical_preseal || !permuted_preseal) {
            apparatus_failure = true;
            pair.correspondence = false;
            pair.canonical.apparatus_valid = false;
            pair.permuted.apparatus_valid = false;
            pair.canonical.route_category = "APPARATUS_INVALID";
            pair.permuted.route_category = "APPARATUS_INVALID";
            pair.canonical.failure_cause =
                "PERMUTATION_CORRESPONDENCE_INVALID";
            pair.permuted.failure_cause = pair.canonical.failure_cause;
            rollback14(canonical_transaction, canonical_initial);
            rollback14(permuted_transaction, permuted_initial);
            append_sealed(pair.canonical, canonical, false);
            append_sealed(pair.permuted, permuted, false);
            publish_early(pair.canonical, canonical, trial_index + 1U);
            publish_early(pair.permuted, permuted, trial_index + 1U);
            pair.canonical.trials.push_back(std::move(canonical));
            pair.permuted.trials.push_back(std::move(permuted));
            if (trial_index == 0U) add_trial_skip("NOT_RUN_BY_PRECEDENCE");
            break;
        }

        finalize_local_step14(policy, canonical);
        finalize_local_step14(policy, permuted);
        const bool canonical_step_equal = counted_predicate(
            pair.canonical.work,
            canonical.step.result_root == permuted.step.result_root);
        const bool permuted_step_equal = counted_predicate(
            pair.permuted.work,
            permuted.step.result_root == canonical.step.result_root);
        ++pair.canonical.expected_work.lane_scalar_comparisons;
        ++pair.permuted.expected_work.lane_scalar_comparisons;
        if (!canonical_step_equal || !permuted_step_equal) {
            apparatus_failure = true;
            pair.correspondence = false;
            pair.canonical.apparatus_valid = false;
            pair.permuted.apparatus_valid = false;
            pair.canonical.route_category = "APPARATUS_INVALID";
            pair.permuted.route_category = "APPARATUS_INVALID";
            pair.canonical.failure_cause =
                "PERMUTATION_CORRESPONDENCE_INVALID";
            pair.permuted.failure_cause = pair.canonical.failure_cause;
            rollback14(canonical_transaction, canonical_initial);
            rollback14(permuted_transaction, permuted_initial);
            append_sealed(pair.canonical, canonical, false);
            append_sealed(pair.permuted, permuted, false);
            pair.canonical.trials.push_back(std::move(canonical));
            pair.permuted.trials.push_back(std::move(permuted));
            if (trial_index == 0U) add_trial_skip("NOT_RUN_BY_PRECEDENCE");
            break;
        }

        TrialObservables14 canonical_observables = trial_observables(
            policy.root, trial_index + 1U, canonical_initial, canonical,
            policy.fixture->profile, canonical_sequence,
            pair.canonical.work, pair.canonical.expected_work);
        TrialObservables14 permuted_observables = trial_observables(
            policy.root, trial_index + 1U, permuted_initial, permuted,
            policy.fixture->profile, permuted_sequence,
            pair.permuted.work, pair.permuted.expected_work);
        const bool canonical_observable_valid =
            canonical_observables.actual_payload_finite
            && canonical_observables.expected_payload_finite
            && !canonical_observables.root.empty();
        const bool permuted_observable_valid =
            permuted_observables.actual_payload_finite
            && permuted_observables.expected_payload_finite
            && !permuted_observables.root.empty();
        if (!canonical_observables.root.empty()) {
            pair.canonical.trial_observable_roots.push_back(
                canonical_observables.root);
            pair.canonical.trial_observables.push_back(
                std::move(canonical_observables));
        }
        if (!permuted_observables.root.empty()) {
            pair.permuted.trial_observable_roots.push_back(
                permuted_observables.root);
            pair.permuted.trial_observables.push_back(
                std::move(permuted_observables));
        }
        if (!canonical_observable_valid || !permuted_observable_valid) {
            apparatus_failure = true;
            pair.correspondence = canonical_observable_valid
                == permuted_observable_valid;
            pair.canonical.apparatus_valid = false;
            pair.permuted.apparatus_valid = false;
            pair.canonical.route_category = "APPARATUS_INVALID";
            pair.permuted.route_category = "APPARATUS_INVALID";
            pair.canonical.failure_cause = pair.correspondence
                ? "NONFINITE_OBSERVABLE"
                : "PERMUTATION_CORRESPONDENCE_INVALID";
            pair.permuted.failure_cause = pair.canonical.failure_cause;
            pair.canonical.typed_route = pair.canonical.failure_cause;
            pair.permuted.typed_route = pair.permuted.failure_cause;
            rollback14(canonical_transaction, canonical_initial);
            rollback14(permuted_transaction, permuted_initial);
            append_sealed(pair.canonical, canonical, false);
            append_sealed(pair.permuted, permuted, false);
            pair.canonical.trials.push_back(std::move(canonical));
            pair.permuted.trials.push_back(std::move(permuted));
            if (trial_index == 0U) add_trial_skip("NOT_RUN_BY_PRECEDENCE");
            break;
        }
        TrialObservables14& canonical_observables_ref =
            pair.canonical.trial_observables.back();
        TrialObservables14& permuted_observables_ref =
            pair.permuted.trial_observables.back();
        bool canonical_observables_equal = true;
        bool permuted_observables_equal = true;
        const std::array<bool, 8> equalities{
            canonical_observables_ref.position_rms
                == permuted_observables_ref.position_rms,
            canonical_observables_ref.position_maximum
                == permuted_observables_ref.position_maximum,
            canonical_observables_ref.velocity_rms
                == permuted_observables_ref.velocity_rms,
            canonical_observables_ref.maximum_speed
                == permuted_observables_ref.maximum_speed,
            canonical_observables_ref.energy_excess
                == permuted_observables_ref.energy_excess,
            canonical_observables_ref.momentum_residual
                == permuted_observables_ref.momentum_residual,
            canonical_observables_ref.components
                == permuted_observables_ref.components,
            canonical_observables_ref.root
                == permuted_observables_ref.root};
        for (const bool equality : equalities) {
            combine_predicate(canonical_observables_equal,
                pair.canonical.work, equality);
            combine_predicate(permuted_observables_equal,
                pair.permuted.work, equality);
        }
        pair.canonical.expected_work.lane_scalar_comparisons += 8U;
        pair.permuted.expected_work.lane_scalar_comparisons += 8U;
        if (!canonical_observables_equal || !permuted_observables_equal) {
            apparatus_failure = true;
            pair.correspondence = false;
            pair.canonical.apparatus_valid = false;
            pair.permuted.apparatus_valid = false;
            pair.canonical.route_category = "APPARATUS_INVALID";
            pair.permuted.route_category = "APPARATUS_INVALID";
            pair.canonical.failure_cause =
                "PERMUTATION_CORRESPONDENCE_INVALID";
            pair.permuted.failure_cause = pair.canonical.failure_cause;
            rollback14(canonical_transaction, canonical_initial);
            rollback14(permuted_transaction, permuted_initial);
            append_sealed(pair.canonical, canonical, false);
            append_sealed(pair.permuted, permuted, false);
            pair.canonical.trials.push_back(std::move(canonical));
            pair.permuted.trials.push_back(std::move(permuted));
            if (trial_index == 0U) add_trial_skip("NOT_RUN_BY_PRECEDENCE");
            break;
        }

        const bool canonical_sequence_pass = sequence_gates14(
            canonical_observables_ref, pair.canonical.work);
        const bool permuted_sequence_pass = sequence_gates14(
            permuted_observables_ref, pair.permuted.work);
        pair.canonical.expected_work.lane_scalar_comparisons += 7U;
        pair.permuted.expected_work.lane_scalar_comparisons += 7U;
        const bool next_pressure = cumulative_pressure
            || canonical.positive_pressure;
        const bool next_lower = cumulative_lower || canonical.bottom_contact;
        const bool canonical_commit = counted_predicate(pair.canonical.work,
            lane_acceptance14(policy.tight, trial_index + 1U,
                canonical.step.physical_pass && canonical_sequence_pass,
                next_pressure, next_lower, canonical.positive_pressure,
                canonical.bottom_contact));
        const bool permuted_commit = counted_predicate(pair.permuted.work,
            lane_acceptance14(policy.tight, trial_index + 1U,
                permuted.step.physical_pass && permuted_sequence_pass,
                next_pressure, next_lower, permuted.positive_pressure,
                permuted.bottom_contact));
        ++pair.canonical.expected_work.lane_scalar_comparisons;
        ++pair.permuted.expected_work.lane_scalar_comparisons;
        const bool commit = canonical_commit && permuted_commit;
        if (commit && trial_index == 1U) {
            const bool canonical_extra_present = counted_predicate(
                pair.canonical.work, !canonical.extra_rounds.empty());
            const bool canonical_round_count = counted_predicate(
                pair.canonical.work,
                canonical.step.rounds.size() == policy.projection_cap);
            bool canonical_last_closed_value = false;
            if (canonical_extra_present) {
                canonical_last_closed_value =
                    canonical.extra_rounds.back().round_closed;
            }
            const bool canonical_last_closed = counted_predicate(
                pair.canonical.work, canonical_last_closed_value);
            const bool canonical_saturation = canonical_extra_present
                && canonical_round_count && canonical_last_closed;
            const bool permuted_extra_present = counted_predicate(
                pair.permuted.work, !permuted.extra_rounds.empty());
            const bool permuted_round_count = counted_predicate(
                pair.permuted.work,
                permuted.step.rounds.size() == policy.projection_cap);
            bool permuted_last_closed_value = false;
            if (permuted_extra_present) {
                permuted_last_closed_value =
                    permuted.extra_rounds.back().round_closed;
            }
            const bool permuted_last_closed = counted_predicate(
                pair.permuted.work, permuted_last_closed_value);
            const bool permuted_saturation = permuted_extra_present
                && permuted_round_count && permuted_last_closed;
            const bool canonical_saturation_equal = counted_predicate(
                pair.canonical.work,
                canonical_saturation == permuted_saturation);
            const bool permuted_saturation_equal = counted_predicate(
                pair.permuted.work,
                permuted_saturation == canonical_saturation);
            pair.canonical.expected_work.lane_scalar_comparisons += 4U;
            pair.permuted.expected_work.lane_scalar_comparisons += 4U;
            pair.canonical.projection_cap_saturated =
                canonical_saturation;
            pair.permuted.projection_cap_saturated =
                permuted_saturation;
            if (!canonical_saturation_equal
                || !permuted_saturation_equal) {
                apparatus_failure = true;
                pair.correspondence = false;
                pair.canonical.apparatus_valid = false;
                pair.permuted.apparatus_valid = false;
                pair.canonical.route_category = "APPARATUS_INVALID";
                pair.permuted.route_category = "APPARATUS_INVALID";
                pair.canonical.failure_cause =
                    "PERMUTATION_CORRESPONDENCE_INVALID";
                pair.permuted.failure_cause =
                    pair.canonical.failure_cause;
                rollback14(canonical_transaction, canonical_initial);
                rollback14(permuted_transaction, permuted_initial);
                append_sealed(pair.canonical, canonical, false);
                append_sealed(pair.permuted, permuted, false);
                pair.canonical.trials.push_back(std::move(canonical));
                pair.permuted.trials.push_back(std::move(permuted));
                break;
            }
        }
        const std::string trial_category = commit ? "SUPPORTED"
            : "PHYSICAL_GATE_REJECTED";
        const std::string trial_failure = commit ? std::string{}
            : (!canonical.step.physical_pass
                ? "PHYSICAL_TRIAL_GATE_REJECTED"
                : "PHYSICAL_SEQUENCE_GATE_REJECTED");
        bool canonical_disposition_equal = true;
        bool permuted_disposition_equal = true;
        for (const bool predicate : std::array<bool, 3>{
                 trial_category == (permuted_commit
                     ? "SUPPORTED" : "PHYSICAL_GATE_REJECTED"),
                 trial_failure == (permuted_commit ? std::string{}
                     : (!permuted.step.physical_pass
                         ? "PHYSICAL_TRIAL_GATE_REJECTED"
                         : "PHYSICAL_SEQUENCE_GATE_REJECTED")),
                 canonical_commit == permuted_commit}) {
            combine_predicate(canonical_disposition_equal,
                pair.canonical.work, predicate);
            combine_predicate(permuted_disposition_equal,
                pair.permuted.work, predicate);
        }
        pair.canonical.expected_work.lane_scalar_comparisons += 3U;
        pair.permuted.expected_work.lane_scalar_comparisons += 3U;
        const bool canonical_prefix_exact =
            prospective_trial_prefix_exact14(pair.canonical, canonical);
        const bool permuted_prefix_exact =
            prospective_trial_prefix_exact14(pair.permuted, permuted);
        if (!canonical_disposition_equal || !permuted_disposition_equal) {
            apparatus_failure = true;
            pair.correspondence = false;
            pair.canonical.apparatus_valid = false;
            pair.permuted.apparatus_valid = false;
            pair.canonical.route_category = "APPARATUS_INVALID";
            pair.permuted.route_category = "APPARATUS_INVALID";
            pair.canonical.failure_cause =
                "PERMUTATION_CORRESPONDENCE_INVALID";
            pair.permuted.failure_cause = pair.canonical.failure_cause;
            rollback14(canonical_transaction, canonical_initial);
            rollback14(permuted_transaction, permuted_initial);
        } else if (!canonical_prefix_exact || !permuted_prefix_exact) {
            apparatus_failure = true;
            pair.correspondence = false;
            pair.canonical.apparatus_valid = false;
            pair.permuted.apparatus_valid = false;
            pair.canonical.route_category = "APPARATUS_INVALID";
            pair.permuted.route_category = "APPARATUS_INVALID";
            pair.canonical.failure_cause = "LANE_WORK_MISMATCH";
            pair.permuted.failure_cause = pair.canonical.failure_cause;
            rollback14(canonical_transaction, canonical_initial);
            rollback14(permuted_transaction, permuted_initial);
        } else if (commit) {
            static_cast<void>(settle_private_trial14(canonical_transaction,
                canonical_prior, canonical, true));
            static_cast<void>(settle_private_trial14(permuted_transaction,
                permuted_prior, permuted, true));
            canonical_sequence = canonical_observables_ref.next;
            permuted_sequence = permuted_observables_ref.next;
            ++pair.canonical.committed_trials;
            ++pair.permuted.committed_trials;
            if (trial_index == 0U) {
                pair.canonical.trial_1_committed = true;
                pair.permuted.trial_1_committed = true;
            } else {
                pair.canonical.trial_2_committed = true;
                pair.permuted.trial_2_committed = true;
            }
            cumulative_pressure = next_pressure;
            cumulative_lower = next_lower;
            trial_pressure[trial_index] = canonical.positive_pressure;
            trial_lower[trial_index] = canonical.bottom_contact;
        } else {
            pair.canonical.physical_gate_rejected = true;
            pair.permuted.physical_gate_rejected = true;
            pair.canonical.route_category = "PHYSICAL_GATE_REJECTED";
            pair.permuted.route_category = "PHYSICAL_GATE_REJECTED";
            pair.canonical.category = LaneCategory14::PhysicalGateRejected;
            pair.permuted.category = LaneCategory14::PhysicalGateRejected;
            pair.canonical.failure_cause = trial_failure;
            pair.permuted.failure_cause = trial_failure;
            rollback14(canonical_transaction, canonical_prior);
            rollback14(permuted_transaction, permuted_prior);
        }
        const bool transaction_commit = commit && !apparatus_failure;
        append_sealed(pair.canonical, canonical, transaction_commit);
        append_sealed(pair.permuted, permuted, transaction_commit);
        pair.canonical.trials.push_back(std::move(canonical));
        pair.permuted.trials.push_back(std::move(permuted));
        if (trial_index == 1U) {
            pair.canonical.trial_2_attempted = true;
            pair.permuted.trial_2_attempted = true;
            pair.canonical.trial_2_execution = transaction_commit
                ? "EXECUTED_COMMITTED" : "EXECUTED_REJECTED";
            pair.permuted.trial_2_execution =
                pair.canonical.trial_2_execution;
        }
        if (apparatus_failure) {
            if (trial_index == 0U) {
                add_trial_skip("NOT_RUN_BY_PRECEDENCE");
            }
            break;
        }
        if (!commit) {
            if (trial_index == 0U) {
                add_trial_skip("NOT_RUN_PRIOR_TRIAL_REJECTED");
            }
            break;
        }
    }

    if (apparatus_failure) {
        pair.canonical.committed_trials = 0U;
        pair.permuted.committed_trials = 0U;
        pair.canonical.trial_1_committed = false;
        pair.permuted.trial_1_committed = false;
        pair.canonical.trial_2_committed = false;
        pair.permuted.trial_2_committed = false;
        canonical_transaction.accepted_state = canonical_initial;
        permuted_transaction.accepted_state = permuted_initial;
        canonical_sequence = {};
        permuted_sequence = {};
    } else if (pair.canonical.committed_trials == 2U) {
        pair.canonical.both_steps_supported = full_lane_acceptance14(
            policy.tight, true, trial_pressure[0], trial_lower[0],
            trial_pressure[1], trial_lower[1], &pair.canonical.work);
        pair.permuted.both_steps_supported = full_lane_acceptance14(
            policy.tight, true, trial_pressure[0], trial_lower[0],
            trial_pressure[1], trial_lower[1], &pair.permuted.work);
        ++pair.canonical.expected_work.lane_scalar_comparisons;
        ++pair.permuted.expected_work.lane_scalar_comparisons;
        pair.canonical.route_category = pair.canonical.both_steps_supported
            ? "SUPPORTED" : "PHYSICAL_GATE_REJECTED";
        pair.permuted.route_category = pair.canonical.route_category;
        pair.canonical.category = pair.canonical.both_steps_supported
            ? LaneCategory14::Supported
            : LaneCategory14::PhysicalGateRejected;
        pair.permuted.category = pair.canonical.category;
        pair.canonical.failure_cause = pair.canonical.both_steps_supported
            ? std::string{} : "PHYSICAL_SEQUENCE_GATE_REJECTED";
        pair.permuted.failure_cause = pair.canonical.failure_cause;
    }
    if (!apparatus_failure) {
        pair.canonical.category_closure = lane_category14(
            pair.canonical, &pair.canonical.work);
        pair.permuted.category_closure = lane_category14(
            pair.permuted, &pair.permuted.work);
        pair.canonical.expected_work.lane_scalar_comparisons += 6U;
        pair.permuted.expected_work.lane_scalar_comparisons += 6U;
        const bool canonical_category_valid =
            pair.canonical.category_closure.apparatus_was_valid
            && pair.canonical.category_closure.one_hot;
        const bool permuted_category_valid =
            pair.permuted.category_closure.apparatus_was_valid
            && pair.permuted.category_closure.one_hot;
        bool canonical_category_equal = false;
        bool permuted_category_equal = false;
        if (canonical_category_valid && permuted_category_valid) {
            pair.canonical.category =
                pair.canonical.category_closure.value;
            pair.permuted.category =
                pair.permuted.category_closure.value;
            canonical_category_equal = counted_predicate(
                pair.canonical.work,
                pair.canonical.category == pair.permuted.category);
            permuted_category_equal = counted_predicate(
                pair.permuted.work,
                pair.permuted.category == pair.canonical.category);
            ++pair.canonical.expected_work.lane_scalar_comparisons;
            ++pair.permuted.expected_work.lane_scalar_comparisons;
        }
        if (!canonical_category_equal || !permuted_category_equal) {
            apparatus_failure = true;
            pair.correspondence = false;
            pair.canonical.apparatus_valid = false;
            pair.permuted.apparatus_valid = false;
            pair.canonical.both_steps_supported = false;
            pair.permuted.both_steps_supported = false;
            pair.canonical.route_category = "APPARATUS_INVALID";
            pair.permuted.route_category = "APPARATUS_INVALID";
            pair.canonical.failure_cause =
                canonical_category_valid && permuted_category_valid
                ? "PERMUTATION_CORRESPONDENCE_INVALID"
                : "LANE_CATEGORY_INVALID";
            pair.permuted.failure_cause = pair.canonical.failure_cause;
            pair.canonical.committed_trials = 0U;
            pair.permuted.committed_trials = 0U;
            pair.canonical.trial_1_committed = false;
            pair.permuted.trial_1_committed = false;
            pair.canonical.trial_2_committed = false;
            pair.permuted.trial_2_committed = false;
            canonical_transaction.accepted_state = canonical_initial;
            permuted_transaction.accepted_state = permuted_initial;
            canonical_sequence = {};
            permuted_sequence = {};
        }
    }
    pair.canonical.typed_route = pair.canonical.failure_cause.empty()
        ? pair.canonical.route_category : pair.canonical.failure_cause;
    pair.permuted.typed_route = pair.permuted.failure_cause.empty()
        ? pair.permuted.route_category : pair.permuted.failure_cause;
    const bool canonical_prefix_exact =
        lane_prefix_work_exact14(pair.canonical);
    const bool permuted_prefix_exact =
        lane_prefix_work_exact14(pair.permuted);
    const bool canonical_hash_exact =
        lane_trajectory_hash_exact14(pair.canonical);
    const bool permuted_hash_exact =
        lane_trajectory_hash_exact14(pair.permuted);
    const bool pair_work_or_hash_mismatch = !canonical_prefix_exact
        || !permuted_prefix_exact || !canonical_hash_exact
        || !permuted_hash_exact;
    finalize_trajectory_v5(pair.canonical, canonical_initial,
        canonical_transaction.accepted_state, canonical_sequence,
        pair_work_or_hash_mismatch);
    finalize_trajectory_v5(pair.permuted, permuted_initial,
        permuted_transaction.accepted_state, permuted_sequence,
        pair_work_or_hash_mismatch);
    lane_result_root(pair.canonical);
    lane_result_root(pair.permuted);
    return pair;
}

LaneResult14 skipped_lane(const LanePolicy& policy, std::string cause,
    bool apparatus_valid, bool include_trial_skip = true) {
    LaneResult14 result;
    result.policy = policy;
    result.typed_route = cause;
    result.route_category = cause;
    result.failure_cause = std::move(cause);
    result.apparatus_valid = apparatus_valid;
    result.trial_2_execution = "NOT_RUN_BY_PRECEDENCE";
    if (include_trial_skip) {
        result.trial_skips.push_back(trial_skip14(
            policy.root, 2U, "NOT_RUN_BY_PRECEDENCE"));
        result.trial_skip_roots.push_back(
            result.trial_skips.back().result_root);
    }
    return result;
}

LaneResult14 retained_open128_lane(const LanePolicy& policy,
    const ParentRetainedTrajectory14& parent) {
    LaneResult14 lane;
    lane.policy = policy;
    lane.inherited_trajectory_json = parent.raw_json;
    lane.typed_route = parent.failure;
    lane.attempted_trials = static_cast<std::uint32_t>(parent.steps.size());
    lane.committed_trials = static_cast<std::uint32_t>(
        parent.accepted_steps);
    for (const ParentRetainedStep14& step : parent.steps) {
        lane.attempted_step_roots.push_back(step.step_root);
        if (!step.committed) {
            lane.rejected_step_roots.push_back(step.step_root);
        }
        lane.round_roots.insert(lane.round_roots.end(),
            step.round_roots.begin(), step.round_roots.end());
    }
    lane.accepted_state_root = parent.state_root;
    if (lane.committed_trials > 0U) {
        lane.accepted_velocity_root =
            parent.steps[lane.committed_trials - 1U].velocity_root;
    }
    lane.failing_state_root = parent.failing_trial_state_root;
    if (!parent.steps.empty()) {
        lane.failing_velocity_root = parent.steps.back().velocity_root;
    }
    lane.apparatus_valid = parent.apparatus_valid;
    lane.both_steps_supported = lane.committed_trials == 2U;
    lane.physical_gate_rejected = !parent.physical_pass
        && lane.apparatus_valid;
    lane.trial_1_committed = !parent.steps.empty()
        && parent.steps[0].committed;
    lane.trial_2_attempted = parent.steps.size() >= 2U;
    lane.trial_2_committed = parent.steps.size() >= 2U
        && parent.steps[1].committed;
    lane.trial_2_execution = lane.trial_2_committed
        ? "EXECUTED_COMMITTED" : "EXECUTED_REJECTED";
    if (!lane.apparatus_valid) {
        lane.route_category = "APPARATUS_INVALID";
        lane.failure_cause = "PERMUTATION_CORRESPONDENCE_INVALID";
        lane.category = LaneCategory14::ApparatusInvalid;
    } else if (lane.both_steps_supported) {
        lane.route_category = "SUPPORTED";
        lane.failure_cause.clear();
        lane.category = LaneCategory14::Supported;
    } else {
        lane.route_category = "PHYSICAL_GATE_REJECTED";
        lane.failure_cause = "PHYSICAL_TRIAL_GATE_REJECTED";
        lane.category = LaneCategory14::PhysicalGateRejected;
    }
    lane.typed_route = lane.failure_cause.empty()
        ? lane.route_category : lane.failure_cause;
    lane.trajectory.apparatus_valid = lane.apparatus_valid;
    lane.trajectory.physical_pass = parent.physical_pass;
    lane.trajectory.accepted_steps = lane.committed_trials;
    lane.trajectory.failure = parent.failure;
    lane.trajectory.failure_step = parent.failure_step;
    lane.trajectory.maximum_position_rms = parent.maximum_position_rms;
    lane.trajectory.maximum_position = parent.maximum_position;
    lane.trajectory.maximum_velocity_rms = parent.maximum_velocity_rms;
    lane.trajectory.maximum_speed = parent.maximum_speed;
    lane.trajectory.energy_excess = parent.energy_excess;
    lane.trajectory.momentum_residual = parent.momentum_residual;
    lane.trajectory.failing_trial_position_rms =
        parent.failing_trial_position_rms;
    lane.trajectory.failing_trial_position_maximum =
        parent.failing_trial_position_maximum;
    lane.trajectory.failing_trial_velocity_rms =
        parent.failing_trial_velocity_rms;
    lane.trajectory.failing_trial_maximum_speed =
        parent.failing_trial_maximum_speed;
    lane.trajectory.failing_trial_energy_excess =
        parent.failing_trial_energy_excess;
    lane.trajectory.failing_trial_momentum_residual =
        parent.failing_trial_momentum_residual;
    lane.trajectory.trajectory_root = parent.trajectory_root;
    lane.trajectory_root = lane.trajectory.trajectory_root;
    lane_result_root(lane);
    return lane;
}

bool retained_parent_controls_pass(
    const std::vector<ParentControlReceipt14>& receipts,
    Work14* owner = nullptr) {
    const auto check = [&](bool predicate) {
        return owner == nullptr ? predicate
            : counted_predicate(*owner, predicate);
    };
    const bool array_size = check(receipts.size() == 11U);
    if (!array_size) return false;
    constexpr std::array<std::uint64_t, 11> expected{
        7U, 1U, 4U, 10U, 3U, 16U, 2U, 9U, 1U, 2U, 16U};
    bool pass = true;
    for (std::size_t index = 0U; index < receipts.size(); ++index) {
        const ParentControlReceipt14& receipt = receipts[index];
        const std::array<bool, 5> predicates{
            receipt.number == std::to_string(index + 1U),
            receipt.status == "PASS", receipt.hash_accounting_exact,
            receipt.expected_hash_derivations == expected[index],
            receipt.actual_hash_derivations == expected[index]};
        for (const bool predicate : predicates) {
            const bool evaluated = check(predicate);
            pass = evaluated && pass;
        }
    }
    return pass;
}

struct Geometry14 {
    FaceCounts counts;
    std::string graph_root;
    std::string root;
    Work14 work;
    Work14 expected_work;
};

std::string geometry_root(std::string_view fixture_root,
    const FaceCounts& counts, const Work14& work) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.geometry-census.v1");
    put_string(bytes, fixture_root);
    const std::array<std::uint64_t, 13> fields{
        128U, 1608U, counts.dynamic_dynamic, counts.dynamic_ghost,
        counts.dynamic_dynamic + counts.dynamic_ghost, counts.lateral_unique,
        counts.membership[0], counts.membership[1], counts.membership[2],
        counts.membership[3], counts.membership[4], counts.membership[5],
        counts.maximum_row};
    for (const std::uint64_t value : fields) put_u64(bytes, value);
    put_string(bytes, work14_root(work));
    return root_of(std::move(bytes));
}

Geometry14 candidate_geometry(const Fixture14& fixture,
    const std::vector<NonlocalGpuGhost>* face_identity = nullptr) {
    Geometry14 result;
    const State state = working_state14(
        fixture.canonical_samples, result.work);
    const GhostGrid grid = make_ghost_grid(fixture.profile, fixture.ghosts);
    Graph14 graph = build_graph14(fixture.profile, state.id, state.position,
        fixture.ghosts, grid, false, true, face_identity);
    result.counts = graph.faces;
    result.graph_root = graph.graph.root;
    add_work14(result.work, graph.extra);
    add_work14(result.expected_work, graph.expected_extra);
    result.expected_work.records_canonicalized +=
        fixture.canonical_samples.size();
    result.work.portable_content_fields_serialized += 16U;
    ++result.work.parent.hash_derivations;
    result.expected_work.portable_content_fields_serialized += 16U;
    ++result.expected_work.parent.hash_derivations;
    result.root = geometry_root(fixture.fixture_root, result.counts,
        result.work);
    return result;
}

Geometry14 oracle_geometry(const Fixture14& fixture,
    const std::vector<NonlocalGpuGhost>& ghosts,
    const std::vector<NonlocalGpuGhost>* face_identity = nullptr) {
    Geometry14 result;
    const State state = working_state14(
        fixture.canonical_samples, result.work);
    std::vector<std::array<bool, 6>> classified_faces;
    classified_faces.reserve(ghosts.size());
    for (std::size_t ghost_index = 0U; ghost_index < ghosts.size();
         ++ghost_index) {
        const NonlocalGpuGhost& classified = face_identity == nullptr
            ? ghosts[ghost_index] : (*face_identity)[ghost_index];
        classified_faces.push_back(ghost_faces(fixture.profile, classified));
        result.work.face_classifications += 6U;
    }
    ++result.work.parent.graph_builds;
    for (std::size_t owner = 0U; owner < state.position.size(); ++owner) {
        std::uint64_t degree = 0U;
        for (std::size_t neighbor = 0U; neighbor < state.position.size();
             ++neighbor) {
            ++result.work.parent.graph_candidates;
            if (norm(state.position[owner] - state.position[neighbor])
                <= fixture.profile.horizon) {
                ++degree;
                ++result.counts.dynamic_dynamic;
                ++result.work.parent.accepted_pairs;
            }
        }
        for (std::size_t ghost_index = 0U; ghost_index < ghosts.size();
             ++ghost_index) {
            const NonlocalGpuGhost& ghost = ghosts[ghost_index];
            ++result.work.parent.graph_candidates;
            ++result.work.ghost_pair_distance_tests;
            if (norm(state.position[owner] - widen(ghost.position))
                > fixture.profile.horizon) continue;
            ++degree;
            ++result.counts.dynamic_ghost;
            ++result.work.parent.accepted_pairs;
            ++result.work.ghost_pairs_accepted;
            const auto& faces = classified_faces[ghost_index];
            bool lateral = false;
            for (std::size_t face = 0U; face < faces.size(); ++face) {
                if (!faces[face]) continue;
                ++result.counts.membership[face];
                lateral = lateral || face < 4U;
            }
            if (lateral) ++result.counts.lateral_unique;
        }
        ++result.work.row_degree_checks;
        result.counts.maximum_row = std::max(
            result.counts.maximum_row, degree);
    }
    result.work.portable_content_fields_serialized += 16U;
    ++result.work.parent.hash_derivations;
    result.expected_work.parent.graph_builds = 1U;
    result.expected_work.records_canonicalized +=
        fixture.canonical_samples.size();
    result.expected_work.parent.graph_candidates = state.position.size()
        * (state.position.size() + ghosts.size());
    result.expected_work.parent.accepted_pairs =
        result.counts.dynamic_dynamic + result.counts.dynamic_ghost;
    result.expected_work.face_classifications = 6U * ghosts.size();
    result.expected_work.ghost_pair_distance_tests =
        state.position.size() * ghosts.size();
    result.expected_work.ghost_pairs_accepted =
        result.counts.dynamic_ghost;
    result.expected_work.row_degree_checks = state.position.size();
    result.expected_work.portable_content_fields_serialized = 16U;
    result.expected_work.parent.hash_derivations = 1U;
    result.root = geometry_root(fixture.fixture_root, result.counts,
        result.work);
    return result;
}

bool counted_exact_geometry(const FaceCounts& counts, Work14& work) {
    bool pass = true;
    combine_predicate(pass, work, counts.dynamic_dynamic == 6560U);
    combine_predicate(pass, work, counts.dynamic_ghost == 6988U);
    combine_predicate(pass, work, counts.lateral_unique == 6480U);
    constexpr std::array<std::uint64_t, 6> memberships{
        1809U, 1865U, 1809U, 1865U, 885U, 0U};
    for (std::size_t face = 0U; face < memberships.size(); ++face) {
        combine_predicate(pass, work,
            counts.membership[face] == memberships[face]);
    }
    combine_predicate(pass, work, counts.maximum_row == 117U);
    return pass;
}

std::vector<NonlocalGpuGhost> mutate_top_layer(
    const NonlocalGpuProfile& profile,
    const std::vector<NonlocalGpuGhost>& ghosts, Work14& owner) {
    std::vector<NonlocalGpuGhost> result = ghosts;
    for (NonlocalGpuGhost& ghost : result) {
        const bool top = counted_predicate(
            owner, ghost_cell(profile, ghost)[2] == 12);
        if (top) {
            ghost.position.z = static_cast<double>(static_cast<float>(0.5));
        }
    }
    return result;
}

struct GeometryMutationRecord14 {
    std::uint32_t id = 0U;
    std::array<std::int32_t, 3> cell{};
    Vec3d old_position;
    Vec3d new_position;
};

struct GeometryMutation14 {
    std::vector<GeometryMutationRecord14> records;
    Work14 work;
    Work14 expected_work;
    std::string work_root;
    std::string root;
};

GeometryMutation14 geometry_mutation_witness(const Fixture14& fixture,
    const std::vector<NonlocalGpuGhost>& mutated,
    std::string_view original_census_root,
    std::string_view mutated_census_root) {
    GeometryMutation14 result;
    for (std::size_t index = 0U; index < fixture.ghosts.size(); ++index) {
        const auto cell = ghost_cell(fixture.profile, fixture.ghosts[index]);
        const bool top = counted_predicate(result.work, cell[2] == 12);
        if (!top) continue;
        result.records.push_back({fixture.ghosts[index].sample_id,
            {static_cast<std::int32_t>(cell[0]),
                static_cast<std::int32_t>(cell[1]),
                static_cast<std::int32_t>(cell[2])},
            fixture.ghosts[index].position, mutated[index].position});
    }
    result.work.portable_content_fields_serialized =
        6U + 10U * result.records.size();
    result.work.parent.hash_derivations = 1U;
    result.expected_work.parent.hash_derivations = 1U;
    result.expected_work.lane_scalar_comparisons = fixture.ghosts.size();
    result.expected_work.portable_content_fields_serialized =
        6U + 10U * result.records.size();
    result.work_root = work14_root(result.work);
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.geometry-mutation.v1");
    put_string(bytes, fixture.fixture_root);
    put_u64(bytes, result.records.size());
    for (const GeometryMutationRecord14& record : result.records) {
        put_u32(bytes, record.id);
        put_u32(bytes, static_cast<std::uint32_t>(record.cell[0]));
        put_u32(bytes, static_cast<std::uint32_t>(record.cell[1]));
        put_u32(bytes, static_cast<std::uint32_t>(record.cell[2]));
        put_f32(bytes, static_cast<float>(record.old_position.x));
        put_f32(bytes, static_cast<float>(record.old_position.y));
        put_f32(bytes, static_cast<float>(record.old_position.z));
        put_f32(bytes, static_cast<float>(record.new_position.x));
        put_f32(bytes, static_cast<float>(record.new_position.y));
        put_f32(bytes, static_cast<float>(record.new_position.z));
    }
    put_string(bytes, original_census_root);
    put_string(bytes, mutated_census_root);
    put_string(bytes, result.work_root);
    result.root = root_of(std::move(bytes));
    return result;
}

enum class SurfaceVariant {
    Correct, ZeroGamma, WrongBranch, WrongSign, HalfForce
};

std::string_view surface_variant_name(SurfaceVariant variant) {
    switch (variant) {
    case SurfaceVariant::Correct: return "CORRECT";
    case SurfaceVariant::ZeroGamma: return "ZERO_GAMMA";
    case SurfaceVariant::WrongBranch: return "WRONG_BRANCH";
    case SurfaceVariant::WrongSign: return "WRONG_SIGN";
    case SurfaceVariant::HalfForce: return "HALF_FORCE";
    }
    throw std::logic_error("unknown NCGP14 surface variant");
}
struct Surface14 {
    std::vector<Vec3l> force;
    std::uint64_t active_pairs = 0U;
    long double acceleration_rms = 0.0L;
    long double acceleration_maximum = 0.0L;
    long double net_residual = 0.0L;
    long double two_step_velocity_scale = 0.0L;
    long double delta_mean_bound = 0.0L;
    std::string root;
    SurfaceVariant variant = SurfaceVariant::Correct;
    Work14 work;
    Work14 expected_work;
};

long double surface_coefficient(long double q, SurfaceVariant variant) {
    if (variant == SurfaceVariant::WrongBranch) {
        return q < 3.0L ? q * q - 1.0L : 0.0L;
    }
    if (q <= 1.0L) return q * q - 1.0L;
    if (q < 3.0L) {
        const long double shifted = q - 2.0L;
        return 1.0L - shifted * shifted;
    }
    return 0.0L;
}

std::string surface_root(std::string_view profile_root,
    SurfaceVariant variant,
    std::string_view fixture_root, const std::vector<std::uint32_t>& ids,
    const Surface14& surface) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.surface-force.v2");
    put_string(bytes, profile_root);
    put_string(bytes, surface_variant_name(variant));
    put_string(bytes, fixture_root);
    put_u64(bytes, surface.active_pairs);
    put_u64(bytes, ids.size());
    for (std::size_t index = 0U; index < ids.size(); ++index) {
        put_u32(bytes, ids[index]);
        put_f64(bytes, surface.force[index].x);
        put_f64(bytes, surface.force[index].y);
        put_f64(bytes, surface.force[index].z);
    }
    return root_of(std::move(bytes));
}

Surface14 surface_candidate(const Fixture14& fixture,
    const NonlocalGpuProfile& profile, std::string_view profile_root,
    SurfaceVariant variant) {
    Surface14 result;
    result.variant = variant;
    const State state = working_state14(
        fixture.canonical_samples, result.work);
    result.force.assign(state.id.size(), {});
    std::vector<std::pair<std::size_t, std::size_t>> active;
    for (std::size_t first = 0U; first < state.id.size(); ++first) {
        for (std::size_t second = first + 1U; second < state.id.size();
             ++second) {
            ++result.work.surface_pair_distance_tests;
            const long double radius = norm(
                state.position[first] - state.position[second]);
            if (radius > 0.0L && radius < 3.0L * profile.spacing) {
                active.emplace_back(first, second);
                ++result.work.surface_active_pairs;
            }
        }
    }
    for (const auto [first, second] : active) {
        const Vec3l difference = state.position[first] - state.position[second];
        const long double radius = norm(difference);
        const long double coefficient = surface_coefficient(
            radius / profile.spacing, variant);
        long double scale = -2.0L * profile.gamma * profile.mass * profile.mass
            * coefficient / radius;
        if (variant == SurfaceVariant::WrongSign) scale = -scale;
        if (variant == SurfaceVariant::HalfForce) scale *= 0.5L;
        const Vec3l force = difference * scale;
        result.force[first] += force;
        result.force[second] += force * -1.0L;
        ++result.work.surface_force_evaluations;
        result.work.surface_reduction_adds += 2U;
    }
    result.active_pairs = active.size();
    Vec3l net;
    long double sum_force_norm = 0.0L;
    long double acceleration2 = 0.0L;
    for (const Vec3l force : result.force) {
        net += force;
        sum_force_norm += norm(force);
        const long double acceleration = norm(force) / profile.mass;
        acceleration2 += acceleration * acceleration;
        result.acceleration_maximum = std::max(
            result.acceleration_maximum, acceleration);
        result.work.surface_reduction_adds += 3U;
    }
    result.acceleration_rms = std::sqrt(
        acceleration2 / result.force.size());
    result.net_residual = norm(net) / std::max(sum_force_norm, 1.0e-30L);
    result.two_step_velocity_scale = 2.0L * profile.dt
        * result.acceleration_rms;
    result.delta_mean_bound = 2.0L * profile.dt * norm(net)
        / (result.force.size() * profile.mass);
    result.work.portable_content_fields_serialized +=
        6U + 4U * state.id.size();
    ++result.work.parent.hash_derivations;
    result.root = surface_root(profile_root, variant, fixture.fixture_root,
        state.id, result);
    result.expected_work.surface_pair_distance_tests =
        state.id.size() * (state.id.size() - 1U) / 2U;
    result.expected_work.records_canonicalized +=
        fixture.canonical_samples.size();
    result.expected_work.surface_active_pairs = result.active_pairs;
    result.expected_work.surface_force_evaluations = result.active_pairs;
    result.expected_work.surface_reduction_adds =
        2U * result.active_pairs + 3U * state.id.size();
    result.expected_work.portable_content_fields_serialized =
        6U + 4U * state.id.size();
    result.expected_work.parent.hash_derivations = 1U;
    return result;
}

Surface14 surface_oracle(const Fixture14& fixture,
    const NonlocalGpuProfile& profile, std::string_view profile_root) {
    Surface14 result;
    result.variant = SurfaceVariant::Correct;
    const State state = working_state14(
        fixture.canonical_samples, result.work);
    result.force.assign(state.id.size(), {});
    for (std::size_t first = 0U; first < state.id.size(); ++first) {
        for (std::size_t second = first + 1U; second < state.id.size();
             ++second) {
            ++result.work.surface_pair_distance_tests;
            const Vec3l difference = state.position[first]
                - state.position[second];
            const long double radius = norm(difference);
            if (!(radius > 0.0L && radius < 3.0L * profile.spacing)) continue;
            ++result.active_pairs;
            ++result.work.surface_active_pairs;
            const long double q = radius / profile.spacing;
            long double coefficient = 0.0L;
            if (q <= 1.0L) {
                coefficient = q * q - 1.0L;
            } else if (q < 3.0L) {
                coefficient = 1.0L - (q - 2.0L) * (q - 2.0L);
            }
            const Vec3l force = difference
                * (-2.0L * profile.gamma * profile.mass * profile.mass
                    * coefficient / radius);
            result.force[first] += force;
            result.force[second] += force * -1.0L;
            ++result.work.surface_force_evaluations;
            result.work.surface_reduction_adds += 2U;
        }
    }
    Vec3l net;
    long double sum_force_norm = 0.0L;
    long double acceleration2 = 0.0L;
    for (const Vec3l force : result.force) {
        net += force;
        sum_force_norm += norm(force);
        const long double acceleration = norm(force) / profile.mass;
        acceleration2 += acceleration * acceleration;
        result.acceleration_maximum = std::max(
            result.acceleration_maximum, acceleration);
        result.work.surface_reduction_adds += 3U;
    }
    result.acceleration_rms = std::sqrt(
        acceleration2 / result.force.size());
    result.net_residual = norm(net) / std::max(sum_force_norm, 1.0e-30L);
    result.two_step_velocity_scale = 2.0L * profile.dt
        * result.acceleration_rms;
    result.delta_mean_bound = 2.0L * profile.dt * norm(net)
        / (result.force.size() * profile.mass);
    result.work.portable_content_fields_serialized +=
        6U + 4U * state.id.size();
    ++result.work.parent.hash_derivations;
    result.root = surface_root(profile_root, SurfaceVariant::Correct,
        fixture.fixture_root,
        state.id, result);
    result.expected_work.surface_pair_distance_tests =
        state.id.size() * (state.id.size() - 1U) / 2U;
    result.expected_work.records_canonicalized +=
        fixture.canonical_samples.size();
    result.expected_work.surface_active_pairs = result.active_pairs;
    result.expected_work.surface_force_evaluations = result.active_pairs;
    result.expected_work.surface_reduction_adds =
        2U * result.active_pairs + 3U * state.id.size();
    result.expected_work.portable_content_fields_serialized =
        6U + 4U * state.id.size();
    result.expected_work.parent.hash_derivations = 1U;
    return result;
}

long double force_relative_l2(const Surface14& lhs, const Surface14& rhs,
    Work14& owner) {
    std::vector<Vec3l> difference(lhs.force.size());
    for (std::size_t index = 0U; index < lhs.force.size(); ++index) {
        difference[index] = lhs.force[index] - rhs.force[index];
    }
    owner.surface_reduction_adds += 3U * lhs.force.size();
    return dense_norm(difference)
        / std::max({dense_norm(lhs.force), dense_norm(rhs.force), 1.0e-30L});
}

struct SideRemoval14 {
    bool pass = false;
    Fixture14 fixture;
    std::vector<Step14> steps;
    std::vector<long double> velocity_rms;
    std::vector<long double> expected_velocity_rms;
    std::vector<TrialObservables14> observables;
    std::string analytic_velocity_root;
    Work14 work;
    Work14 expected_work;
};

SideRemoval14 side_removal_control(const LanePolicy& base_policy,
    Work14 fixed_root_work) {
    SideRemoval14 result;
    const Fixture14& tight = *base_policy.fixture;
    result.fixture.name = "TIGHT-128-SIDE-REMOVED";
    result.fixture.profile = tight.profile;
    result.fixture.profile_root = tight.profile_root;
    result.fixture.canonical_samples = tight.canonical_samples;
    result.fixture.permuted_samples = tight.permuted_samples;
    result.work = fixed_root_work;
    for (const NonlocalGpuGhost& ghost : tight.ghosts) {
        const auto cell = ghost_cell(tight.profile, ghost);
        const bool x_low = counted_predicate(result.work, cell[0] < 0);
        const bool x_high = counted_predicate(result.work, cell[0] >= 4);
        const bool y_low = counted_predicate(result.work, cell[1] < 0);
        const bool y_high = counted_predicate(result.work, cell[1] >= 4);
        const bool z_low = counted_predicate(result.work, cell[2] < 0);
        const bool side = x_low || x_high || y_low || y_high;
        const bool retain = counted_predicate(result.work, z_low || !side);
        if (retain) result.fixture.ghosts.push_back(ghost);
    }
    result.expected_work.lane_scalar_comparisons +=
        6U * tight.ghosts.size();
    result.fixture.fixture_root = fixture14_root(result.fixture.name,
        result.fixture.profile_root, result.fixture.canonical_samples,
        result.fixture.ghosts, result.work);
    result.expected_work.parent.hash_derivations = 1U;
    result.expected_work.portable_content_fields_serialized =
        5U + 10U * result.fixture.canonical_samples.size()
        + 4U * result.fixture.ghosts.size();
    LanePolicy policy{"TIGHT-128-SIDE-REMOVED", base_policy.root,
        &result.fixture, 2U, 8U, 16384U, false, false, true};
    const GhostGrid grid = make_ghost_grid(result.fixture.profile,
        result.fixture.ghosts);
    State state = working_state14(
        result.fixture.canonical_samples, result.work);
    result.expected_work.records_canonicalized +=
        result.fixture.canonical_samples.size();
    const State initial = state;
    Sequence14 sequence;
    const long double dt = result.fixture.profile.dt;
    const long double gravity = 9.81L;
    bool pass = true;
    combine_predicate(pass, result.work,
        result.fixture.ghosts.size() == 348U);
    std::vector<std::vector<Vec3l>> actual_velocities;
    result.steps.reserve(2U);
    result.velocity_rms.reserve(2U);
    result.expected_velocity_rms.reserve(2U);
    result.observables.reserve(2U);
    actual_velocities.reserve(2U);
    for (std::size_t step_index = 0U; step_index < 2U; ++step_index) {
        result.steps.push_back(run_step14(policy, state,
            result.fixture.ghosts, grid,
            static_cast<std::uint32_t>(step_index + 1U)));
        Step14& step = result.steps.back();
        add_parent_work(result.work, step.step.work);
        add_work14(result.work, step.extra_work);
        add_parent_work(result.expected_work,
            step.expected_parent_work);
        add_work14(result.expected_work,
            step.expected_extra_work);
        ++result.work.receipt_children_aggregated;
        ++result.expected_work.receipt_children_aggregated;
        bool observable_stage = true;
        combine_predicate(observable_stage, result.work,
            step.step.apparatus_valid);
        combine_predicate(observable_stage, result.work,
            step.step.accepted);
        combine_predicate(observable_stage, result.work,
            step.trial_invariants_ok);
        if (!observable_stage) {
            pass = false;
            break;
        }
        TrialObservables14 observables = trial_observables(policy.root,
            static_cast<std::uint32_t>(step_index + 1U), initial, step,
            result.fixture.profile, sequence, result.work,
            result.expected_work);
        if (!observables.actual_payload_finite
            || !observables.expected_payload_finite
            || observables.root.empty()) {
            pass = false;
            break;
        }
        result.observables.push_back(std::move(observables));
        sequence = result.observables.back().next;
        actual_velocities.push_back(step.step.state.velocity);
        long double sum = 0.0L;
        for (const Vec3l velocity : actual_velocities.back()) {
            sum += dot(velocity, velocity);
        }
        result.velocity_rms.push_back(std::sqrt(sum / 128.0L));
        const long double bottom_velocity = step_index == 0U
            ? (0.5L * result.fixture.profile.spacing
                - widen(result.fixture.canonical_samples.front().current).z)
                / dt
            : 0.0L;
        const long double falling = -static_cast<long double>(step_index + 1U)
            * gravity * dt;
        result.expected_velocity_rms.push_back(std::sqrt(
            (16.0L * bottom_velocity * bottom_velocity
                + 112.0L * falling * falling) / 128.0L));
        bool velocities = true;
        for (std::size_t index = 0U; index < state.id.size(); ++index) {
            const std::size_t logical = (state.id[index] - 1000U) / 17U;
            const bool bottom = counted_predicate(
                result.work, logical / 16U == 0U);
            const long double expected_z = bottom ? bottom_velocity : falling;
            combine_predicate(velocities, result.work,
                step.step.state.velocity[index].x == 0.0L);
            combine_predicate(velocities, result.work,
                step.step.state.velocity[index].y == 0.0L);
            combine_predicate(velocities, result.work, relative_close(
                step.step.state.velocity[index].z,
                expected_z));
        }
        bool step_pass = true;
        for (const bool predicate : std::array<bool, 6>{
                 step.step.maximum_strain == 0.0L,
                 step.step.rms_strain == 0.0L,
                 step.step.positive_multipliers == 0U,
                 step.step.lower_contacts == 16U,
                 velocities, relative_close(result.velocity_rms.back(),
                     result.expected_velocity_rms.back())}) {
            combine_predicate(step_pass, result.work, predicate);
        }
        pass = step_pass && pass;
        if (!step_pass) break;
        state = step.step.state;
    }
    if (result.observables.size() == 2U) {
        combine_predicate(pass, result.work,
            result.velocity_rms[0] <= kVelocityRmsLimit);
        combine_predicate(pass, result.work,
            result.velocity_rms[1] > kVelocityRmsLimit);
        std::string bytes;
        put_string(bytes,
            "nextengine.nonlocal.ncgp14.side-removal-velocity.v1");
        put_string(bytes, base_policy.root);
        put_string(bytes, result.fixture.fixture_root);
        for (std::size_t step_index = 0U; step_index < 2U; ++step_index) {
            put_u64(bytes, actual_velocities[step_index].size());
            for (std::size_t index = 0U; index < initial.id.size(); ++index) {
                put_u32(bytes, initial.id[index]);
                put_f64(bytes, actual_velocities[step_index][index].x);
                put_f64(bytes, actual_velocities[step_index][index].y);
                put_f64(bytes, actual_velocities[step_index][index].z);
            }
            put_f64(bytes, result.velocity_rms[step_index]);
            put_f64(bytes, result.expected_velocity_rms[step_index]);
        }
        result.work.portable_content_fields_serialized += 9U + 8U * 128U;
        ++result.work.parent.hash_derivations;
        result.analytic_velocity_root = root_of(std::move(bytes));
        ++result.expected_work.parent.hash_derivations;
        result.expected_work.portable_content_fields_serialized +=
            9U + 8U * 128U;
    }
    result.expected_work.lane_scalar_comparisons += 1U
        + 3U * result.steps.size()
        + (4U * initial.id.size() + 6U) * result.observables.size()
        + (result.observables.size() == 2U ? 2U : 0U);
    result.pass = pass;
    return result;
}

struct Scalar14 {
    std::string name;
    std::uint8_t type = 0U;
    bool boolean = false;
    std::uint32_t u32 = 0U;
    std::uint64_t u64 = 0U;
    long double f64 = 0.0L;
};
Scalar14 scalar_bool(std::string name, bool value) {
    Scalar14 result;
    result.name = std::move(name);
    result.type = 0U;
    result.boolean = value;
    return result;
}
Scalar14 scalar_u32(std::string name, std::uint32_t value) {
    Scalar14 result;
    result.name = std::move(name);
    result.type = 1U;
    result.u32 = value;
    return result;
}
Scalar14 scalar_u64(std::string name, std::uint64_t value) {
    Scalar14 result;
    result.name = std::move(name);
    result.type = 2U;
    result.u64 = value;
    return result;
}
Scalar14 scalar_f64(std::string name, long double value) {
    Scalar14 result;
    result.name = std::move(name);
    result.type = 3U;
    result.f64 = value;
    return result;
}

struct Control14 {
    std::uint32_t index = 0U;
    std::string name;
    std::string outcome;
    bool pass = false;
    std::vector<std::string> evidence_roots;
    std::vector<Scalar14> scalars;
    Work14 work;
    Work14 expected_work;
    WorkSeal14 verifier;
    std::string result_root;
};

void seal_control14(Control14& control) {
    control.verifier = verify_work14(control.work, control.expected_work);
    if (!control.verifier.exact
        && control.outcome != "NOT_RUN_BY_PRECEDENCE") {
        control.pass = false;
        control.outcome = "CONTROL_INVALID";
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.control.v1");
    put_u32(bytes, control.index);
    put_string(bytes, control.name);
    put_string(bytes, control.outcome);
    put_u64(bytes, control.evidence_roots.size());
    for (const std::string& root : control.evidence_roots) {
        put_string(bytes, root);
    }
    put_u64(bytes, control.scalars.size());
    for (const Scalar14& scalar : control.scalars) {
        put_string(bytes, scalar.name);
        put_u8(bytes, scalar.type);
        if (scalar.type == 0U) put_u8(bytes, scalar.boolean ? 1U : 0U);
        else if (scalar.type == 1U) put_u32(bytes, scalar.u32);
        else if (scalar.type == 2U) put_u64(bytes, scalar.u64);
        else if (scalar.type == 3U) put_f64(bytes, scalar.f64);
        else throw std::logic_error("NCGP14 scalar type invalid");
    }
    append_work_verifier(bytes, control.verifier);
    control.result_root = root_of(std::move(bytes));
}

Control14 skipped_control14(std::uint32_t index, std::string name) {
    Control14 result;
    result.index = index;
    result.name = std::move(name);
    result.outcome = "NOT_RUN_BY_PRECEDENCE";
    return result;
}

struct TransactionRejection14 {
    std::string base_lane_root;
    std::string subroute;
    std::string outcome;
    std::uint32_t attempted_trials = 1U;
    std::uint32_t committed_trials = 0U;
    std::vector<std::string> child_roots;
    std::string rejected_state_root;
    std::string pre_scratch_root;
    std::string post_scratch_root;
    std::string trial_skip_root;
    bool hook_fired = false;
    Work14 work;
    Work14 expected_work;
    WorkSeal14 verifier;
    std::string result_root;
};

std::string seal_transaction_rejection14(TransactionRejection14& receipt) {
    receipt.verifier = verify_work14(receipt.work, receipt.expected_work);
    std::string bytes;
    put_string(bytes,
        "nextengine.nonlocal.ncgp14.transaction-rejection.v1");
    put_string(bytes, receipt.base_lane_root);
    put_string(bytes, receipt.subroute);
    put_string(bytes, receipt.outcome);
    put_u32(bytes, receipt.attempted_trials);
    put_u32(bytes, receipt.committed_trials);
    put_u64(bytes, receipt.child_roots.size());
    for (const std::string& root : receipt.child_roots) {
        put_string(bytes, root);
    }
    put_string(bytes, receipt.rejected_state_root);
    put_string(bytes, receipt.pre_scratch_root);
    put_string(bytes, receipt.post_scratch_root);
    put_string(bytes, receipt.trial_skip_root);
    put_u8(bytes, receipt.hook_fired ? 1U : 0U);
    append_work_verifier(bytes, receipt.verifier);
    receipt.result_root = root_of(std::move(bytes));
    return receipt.result_root;
}

struct TransactionSubroute14 {
    Step14 private_step;
    std::vector<TrialObservables14> observables;
    TrialSkip14 trial_skip;
    TransactionRejection14 rejection;
    ProductionTransaction14 transaction;
    Scratch14 pre_scratch;
    Scratch14 post_scratch;
    std::string pre_scratch_root;
    std::string post_scratch_root;
    std::string rejected_state_root;
    bool pass = false;
};

struct Transaction14 {
    bool pass = false;
    bool qp_cap_1_subroute = false;
    bool forced_hook_observed = false;
    TransactionSubroute14 qp;
    TransactionSubroute14 forced;
    std::string prior_root;
    std::string qp_rejected_root;
    std::string forced_rejected_root;
    Scratch14 canonical_empty;
    std::string empty_scratch_root;
    Work14 work;
    Work14 expected_work;
};

std::uint64_t compare_state_values(const State& lhs, const State& rhs,
    bool& equal) {
    std::uint64_t compared = 0U;
    equal = lhs.id.size() == rhs.id.size();
    ++compared;
    const std::size_t count = std::min(lhs.id.size(), rhs.id.size());
    for (std::size_t index = 0U; index < count; ++index) {
        const bool id_equal = lhs.id[index] == rhs.id[index];
        equal = id_equal && equal;
        ++compared;
        for (const auto& values : std::array<std::pair<Vec3l, Vec3l>, 3>{
                 std::pair{lhs.reference[index], rhs.reference[index]},
                 std::pair{lhs.position[index], rhs.position[index]},
                 std::pair{lhs.velocity[index], rhs.velocity[index]}}) {
            const bool x_equal = values.first.x == values.second.x;
            const bool y_equal = values.first.y == values.second.y;
            const bool z_equal = values.first.z == values.second.z;
            equal = x_equal && equal;
            equal = y_equal && equal;
            equal = z_equal && equal;
            compared += 3U;
        }
    }
    return compared;
}

std::uint64_t expected_csr_entries14(
    const LanePolicy& policy, const std::vector<Vec3l>& positions,
    const std::vector<NonlocalGpuGhost>& ghosts) {
    const NonlocalGpuProfile& profile = policy.fixture->profile;
    std::uint64_t entries = 0U;
    for (const Vec3l owner : positions) {
        for (const Vec3l neighbor : positions) {
            if (norm(owner - neighbor) <= profile.horizon) ++entries;
        }
        for (const NonlocalGpuGhost& ghost : ghosts) {
            if (norm(owner - widen(ghost.position)) <= profile.horizon) {
                ++entries;
            }
        }
    }
    return entries;
}

Work14 expected_pre_scratch_work14(std::size_t dynamic_count,
    std::uint64_t csr_entries) {
    Work14 result;
    result.parent.hash_derivations = 1U;
    result.portable_content_fields_serialized = 6U
        + 15U * dynamic_count + 2U * csr_entries;
    return result;
}

Transaction14 transaction_control(const LanePolicy& tight16384) {
    Transaction14 result;
    const State prior = working_state14(
        tight16384.fixture->canonical_samples, result.work);
    result.expected_work.records_canonicalized +=
        tight16384.fixture->canonical_samples.size();
    const GhostGrid grid = make_ghost_grid(tight16384.fixture->profile,
        tight16384.fixture->ghosts);
    LanePolicy cap1 = tight16384;
    cap1.name = "TIGHT-128-TRANSACTION-QP1";
    cap1.qp_cap = 1U;
    result.prior_root = state14_root(tight16384.root, prior);
    ++result.work.parent.hash_derivations;
    result.work.portable_content_fields_serialized +=
        state14_portable_fields(prior.id.size());
    result.expected_work.parent.hash_derivations = 1U;
    result.expected_work.portable_content_fields_serialized =
        state14_portable_fields(prior.id.size());
    auto complete_subroute = [&](TransactionSubroute14& subroute,
                                 const LanePolicy& execution_policy,
                                 bool forced) -> bool {
        subroute.transaction.accepted_state = prior;
        subroute.transaction.force_after_private_observable_seal_armed =
            forced;
        subroute.private_step = run_step14(execution_policy,
            subroute.transaction.accepted_state,
            tight16384.fixture->ghosts, grid, 1U);
        stage_private_trial14(subroute.transaction, subroute.private_step);
        TransactionRejection14& rejection = subroute.rejection;
        rejection.base_lane_root = tight16384.root;
        rejection.subroute = forced
            ? "FORCED_AFTER_PRIVATE_OBSERVABLE_SEAL"
            : "QP_CAP_1_SUBROUTE";
        rejection.outcome = forced
            ? "FORCED_ROLLBACK_OBSERVED"
            : "QP_SWEEP_CAP_EXHAUSTED";
        const auto close_invalid_prefix = [&]() {
            add_parent_work(result.work, subroute.private_step.step.work);
            add_work14(result.work, subroute.private_step.extra_work);
            add_parent_work(result.expected_work,
                subroute.private_step.expected_parent_work);
            add_work14(result.expected_work,
                subroute.private_step.expected_extra_work);
            subroute.pass = false;
            return false;
        };
        if (!forced) {
            const bool exact_qp_route =
                subroute.private_step.typed_route
                    == "QP_SWEEP_CAP_EXHAUSTED"
                && subroute.private_step.early_child_roots.size() == 1U
                && subroute.private_step.early_child_stages.size() == 1U
                && subroute.private_step.early_child_stages.front()
                    == "TRIAL_PREDICTOR_CONTACT"
                && !subroute.private_step.early_child_roots.front().empty()
                && subroute.private_step.step.rounds.size() == 1U
                && !subroute.private_step.step.rounds.front().result_root.empty();
            if (!exact_qp_route) return close_invalid_prefix();
        } else {
            const bool forced_trial_ready =
                subroute.private_step.step.apparatus_valid
                && subroute.private_step.step.accepted
                && subroute.private_step.trial_invariants_ok
                && !subroute.private_step.qp_cap_exhausted
                && !subroute.private_step.projection_cap_exhausted;
            if (!forced_trial_ready) return close_invalid_prefix();
        }
        bool production_commit = false;
        if (forced) {
            finalize_local_step14(execution_policy, subroute.private_step);
            TrialObservables14 observables = trial_observables(
                tight16384.root, 1U, prior, subroute.private_step,
                tight16384.fixture->profile, {}, rejection.work,
                rejection.expected_work);
            const bool observable_valid =
                observables.actual_payload_finite
                && observables.expected_payload_finite
                && !observables.root.empty();
            if (observable_valid) {
                const bool sequence_pass = sequence_gates14(
                    observables, rejection.work);
                production_commit = counted_predicate(rejection.work,
                    lane_acceptance14(true, 1U,
                        subroute.private_step.step.physical_pass
                            && sequence_pass,
                        subroute.private_step.positive_pressure,
                        subroute.private_step.bottom_contact,
                        subroute.private_step.positive_pressure,
                        subroute.private_step.bottom_contact));
                rejection.expected_work.lane_scalar_comparisons += 8U;
                rejection.child_roots = {
                    subroute.private_step.step.result_root,
                    observables.root,
                    subroute.private_step.step.state_root,
                    subroute.private_step.step.velocity_root};
                subroute.observables.push_back(std::move(observables));
                if (!production_commit) {
                    add_work14(result.work, rejection.work);
                    add_work14(result.expected_work,
                        rejection.expected_work);
                    return close_invalid_prefix();
                }
            } else {
                add_work14(result.work, rejection.work);
                add_work14(result.expected_work, rejection.expected_work);
                return close_invalid_prefix();
            }
        } else {
            if (!subroute.private_step.early_child_roots.empty()
                && !subroute.private_step.step.rounds.empty()) {
                rejection.child_roots = {
                    subroute.private_step.early_child_roots.front(),
                    subroute.private_step.step.rounds.front().result_root};
            }
        }
        subroute.pre_scratch_root = scratch_root(tight16384.root,
            subroute.transaction.scratch, &rejection.work);
        const std::uint64_t expected_entries = expected_csr_entries14(
            execution_policy, subroute.private_step.scratch_input_positions,
            tight16384.fixture->ghosts);
        add_work14(rejection.expected_work,
            expected_pre_scratch_work14(prior.id.size(), expected_entries));
        subroute.pre_scratch = subroute.transaction.scratch;
        static_cast<void>(settle_private_trial14(subroute.transaction,
            prior, subroute.private_step, production_commit));
        subroute.rejected_state_root = state14_root(tight16384.root,
            subroute.transaction.accepted_state);
        ++result.work.parent.hash_derivations;
        result.work.portable_content_fields_serialized +=
            state14_portable_fields(
                subroute.transaction.accepted_state.id.size());
        ++result.expected_work.parent.hash_derivations;
        result.expected_work.portable_content_fields_serialized +=
            state14_portable_fields(
                subroute.transaction.accepted_state.id.size());
        if (forced) {
            result.forced_rejected_root = subroute.rejected_state_root;
        } else {
            result.qp_rejected_root = subroute.rejected_state_root;
        }
        subroute.post_scratch_root = scratch_root(tight16384.root,
            subroute.transaction.scratch, &rejection.work);
        Work14 expected_post;
        expected_post.parent.hash_derivations = 1U;
        expected_post.portable_content_fields_serialized = 6U;
        add_work14(rejection.expected_work, expected_post);
        subroute.post_scratch = subroute.transaction.scratch;
        subroute.trial_skip = trial_skip14(tight16384.root, 2U,
            "NOT_RUN_PRIOR_TRIAL_REJECTED");
        add_parent_work(rejection.work, subroute.private_step.step.work);
        add_work14(rejection.work, subroute.private_step.extra_work);
        add_parent_work(rejection.expected_work,
            subroute.private_step.expected_parent_work);
        add_work14(rejection.expected_work,
            subroute.private_step.expected_extra_work);
        ++rejection.work.receipt_children_aggregated;
        ++rejection.expected_work.receipt_children_aggregated;
        add_child(rejection.work, subroute.trial_skip.work);
        add_child(rejection.expected_work, subroute.trial_skip.expected_work);
        rejection.rejected_state_root = subroute.rejected_state_root;
        rejection.pre_scratch_root = subroute.pre_scratch_root;
        rejection.post_scratch_root = subroute.post_scratch_root;
        rejection.trial_skip_root = subroute.trial_skip.result_root;
        rejection.hook_fired = subroute.transaction.hook_fired;
        seal_transaction_rejection14(rejection);
        return true;
    };
    if (!complete_subroute(result.qp, cap1, false)) return result;
    if (!complete_subroute(result.forced, tight16384, true)) return result;
    result.empty_scratch_root = scratch_root(tight16384.root,
        result.canonical_empty, &result.work);
    ++result.expected_work.parent.hash_derivations;
    result.expected_work.portable_content_fields_serialized += 6U;
    add_child(result.work, result.qp.rejection.work);
    add_child(result.work, result.forced.rejection.work);
    add_child(result.expected_work, result.qp.rejection.expected_work);
    add_child(result.expected_work, result.forced.rejection.expected_work);
    bool qp_equal = false;
    bool forced_equal = false;
    result.work.transaction_values_compared += compare_state_values(
        prior, result.qp.transaction.accepted_state, qp_equal);
    result.work.transaction_values_compared += compare_state_values(
        prior, result.forced.transaction.accepted_state, forced_equal);
    result.expected_work.transaction_values_compared =
        2U * (1U + 10U * prior.id.size());
    bool qp_scratch_equal = false;
    bool forced_scratch_equal = false;
    result.work.scratch_values_compared += compare_scratch_values(
        result.qp.post_scratch, result.canonical_empty, qp_scratch_equal);
    result.work.scratch_values_compared += compare_scratch_values(
        result.forced.post_scratch, result.canonical_empty,
        forced_scratch_equal);
    result.expected_work.scratch_values_compared = 20U;
    bool pass = true;
    result.qp_cap_1_subroute = counted_predicate(result.work,
        result.qp.private_step.typed_route == "QP_SWEEP_CAP_EXHAUSTED");
    pass = result.qp_cap_1_subroute && pass;
    const bool qp_one_round = counted_predicate(result.work,
        result.qp.private_step.step.rounds.size() == 1U);
    pass = qp_one_round && pass;
    const std::uint64_t qp_first_round_sweeps =
        qp_one_round ? result.qp.private_step.step.rounds.front().sweeps
                     : 0U;
    combine_predicate(pass, result.work,
        qp_first_round_sweeps == 1U);
    combine_predicate(pass, result.work,
        result.forced.rejection.outcome == "FORCED_ROLLBACK_OBSERVED");
    combine_predicate(pass, result.work,
        result.qp.pre_scratch.owner_ids.size() == 128U);
    combine_predicate(pass, result.work,
        !result.qp.pre_scratch.entries.empty());
    combine_predicate(pass, result.work,
        result.qp.pre_scratch.lambda.size() == 128U);
    combine_predicate(pass, result.work,
        result.qp.pre_scratch.clamp_mask.size() == 128U);
    combine_predicate(pass, result.work,
        result.forced.pre_scratch.owner_ids.size() == 128U);
    combine_predicate(pass, result.work,
        !result.forced.pre_scratch.entries.empty());
    combine_predicate(pass, result.work,
        result.forced.pre_scratch.lambda.size() == 128U);
    combine_predicate(pass, result.work,
        result.forced.pre_scratch.clamp_mask.size() == 128U);
    const bool qp_root_equal = result.qp_rejected_root
        == result.prior_root;
    const bool forced_root_equal = result.forced_rejected_root
        == result.prior_root;
    combine_predicate(pass, result.work, qp_root_equal);
    combine_predicate(pass, result.work, forced_root_equal);
    result.forced_hook_observed = counted_predicate(result.work,
        result.forced.transaction.hook_fired);
    pass = result.forced_hook_observed && pass;
    pass = qp_equal && forced_equal && qp_scratch_equal
        && forced_scratch_equal && result.qp.rejection.verifier.exact
        && result.forced.rejection.verifier.exact && pass;
    result.expected_work.lane_scalar_comparisons += 15U;
    result.qp.pass = qp_equal && qp_root_equal && qp_scratch_equal
        && result.qp.rejection.verifier.exact;
    result.forced.pass = forced_equal && forced_root_equal
        && forced_scratch_equal
        && result.forced.rejection.verifier.exact
        && result.forced.transaction.hook_fired;
    result.pass = pass;
    return result;
}

struct AdmissionMutation14 {
    std::uint32_t index = 0U;
    std::string name;
    std::uint64_t maximum_total_records = 100000U;
    std::uint64_t expected_ghost_count = 1608U;
    Admission14 expected_outcome = Admission14::InvalidProfile;
    Admission14 observed_outcome = Admission14::InvalidProfile;
    std::string field_name;
    std::array<std::uint64_t, 8> u64_payload{};
    std::array<std::uint32_t, 2> u32_payload{};
    std::string rejected_state_root;
    std::string root;
};

double binary64_from_bits(std::uint64_t bits) {
    double value = 0.0;
    static_assert(sizeof value == sizeof bits);
    std::memcpy(&value, &bits, sizeof value);
    return value;
}

AdmissionMutation14 admission_mutation14(std::uint32_t index) {
    AdmissionMutation14 result;
    result.index = index;
    switch (index) {
    case 1U:
        result.name = "WRONG_BASIN_X";
        result.expected_outcome = Admission14::InvalidProfile;
        result.field_name = "basin_extent.x";
        result.u64_payload[0] = UINT64_C(0x3fd0000000000000);
        break;
    case 2U:
        result.name = "MISSING_LAST_GHOST";
        result.expected_outcome = Admission14::InvalidProfile;
        result.u64_payload[0] = 1608U;
        result.u64_payload[1] = 1607U;
        break;
    case 3U:
        result.name = "WRONG_GHOST_LAYERS";
        result.expected_outcome = Admission14::InvalidProfile;
        result.field_name = "ghost_layers";
        result.u32_payload[0] = 2U;
        break;
    case 4U:
        result.name = "NON_BINARY32_CURRENT_X";
        result.expected_outcome = Admission14::NonBinary32;
        result.u64_payload[0] = 0U;
        result.field_name = "current.x";
        result.u64_payload[1] = UINT64_C(0x3fb999999999999a);
        break;
    case 5U:
        result.name = "DUPLICATE_LAST_ID";
        result.expected_outcome = Admission14::DuplicateId;
        result.u64_payload[0] = 127U;
        result.u64_payload[1] = 0U;
        break;
    case 6U:
        result.name = "NONFINITE_CURRENT_Z";
        result.expected_outcome = Admission14::Nonfinite;
        result.u64_payload[0] = 0U;
        result.field_name = "current.z";
        result.u64_payload[1] = UINT64_C(0x7ff0000000000000);
        break;
    case 7U:
        result.name = "DYNAMIC_CAPACITY_50001";
        result.expected_outcome = Admission14::CapacityExceeded;
        result.u64_payload[0] = 128U;
        result.u64_payload[1] = 50001U;
        result.u64_payload[2] = 0U;
        break;
    case 8U:
        result.name = "TOTAL_CAPACITY_100001";
        result.expected_outcome = Admission14::CapacityExceeded;
        result.expected_ghost_count = 100000U;
        result.u64_payload[0] = 0U;
        result.u64_payload[1] = 100000U;
        result.u32_payload[0] = 1000000U;
        result.u32_payload[1] = 1U;
        result.u64_payload[2] = UINT64_C(0x0000000000000000);
        result.u64_payload[3] = UINT64_C(0x0000000000000000);
        result.u64_payload[4] = UINT64_C(0xbf9999999999999a);
        break;
    case 9U:
        result.name = "ROW_CAPACITY_257";
        result.expected_outcome = Admission14::RowNeighborCapacityExceeded;
        result.expected_ghost_count = 0U;
        result.u64_payload[0] = 257U;
        result.u32_payload[0] = 1000U;
        result.u32_payload[1] = 17U;
        result.u64_payload[1] = UINT64_C(0x3fb3333340000000);
        result.u64_payload[2] = UINT64_C(0x3fb3333340000000);
        result.u64_payload[3] = UINT64_C(0x3fb3333340000000);
        result.u64_payload[4] = UINT64_C(0x0000000000000000);
        result.u64_payload[5] = UINT64_C(0x0000000000000000);
        result.u64_payload[6] = UINT64_C(0x0000000000000000);
        result.u64_payload[7] = 0U;
        break;
    default:
        throw std::logic_error("NCGP14 admission mutation index invalid");
    }
    return result;
}

struct AdmissionMutationInput14 {
    NonlocalGpuProfile profile;
    std::vector<NonlocalGpuSample> samples;
    std::vector<NonlocalGpuGhost> ghosts;
    bool check_rows = false;
};

AdmissionMutationInput14 admission_mutation_input14(
    const Fixture14& tight, const AdmissionMutation14& mutation) {
    AdmissionMutationInput14 result{
        tight.profile, tight.canonical_samples, tight.ghosts, false};
    switch (mutation.index) {
    case 1U:
        result.profile.basin_extent.x = binary64_from_bits(
            mutation.u64_payload[0]);
        break;
    case 2U:
        if (result.ghosts.size() != mutation.u64_payload[0]
            || mutation.u64_payload[1] >= result.ghosts.size()) {
            throw std::logic_error("NCGP14 missing-ghost prefix invalid");
        }
        result.ghosts.erase(result.ghosts.begin()
            + static_cast<std::ptrdiff_t>(mutation.u64_payload[1]));
        break;
    case 3U:
        result.profile.ghost_layers = mutation.u32_payload[0];
        break;
    case 4U:
        result.samples.at(mutation.u64_payload[0]).current.x =
            binary64_from_bits(mutation.u64_payload[1]);
        break;
    case 5U:
        result.samples.at(mutation.u64_payload[0]).sample_id =
            result.samples.at(mutation.u64_payload[1]).sample_id;
        break;
    case 6U:
        result.samples.at(mutation.u64_payload[0]).current.z =
            binary64_from_bits(mutation.u64_payload[1]);
        break;
    case 7U:
        result.samples.resize(mutation.u64_payload[1],
            result.samples.at(mutation.u64_payload[2]));
        break;
    case 8U: {
        const NonlocalGpuSample retained = result.samples.at(
            mutation.u64_payload[0]);
        result.samples = {retained};
        result.ghosts.clear();
        result.ghosts.reserve(mutation.u64_payload[1]);
        const Vec3d position{binary64_from_bits(mutation.u64_payload[2]),
            binary64_from_bits(mutation.u64_payload[3]),
            binary64_from_bits(mutation.u64_payload[4])};
        for (std::uint32_t index = 0U;
             index < mutation.u64_payload[1]; ++index) {
            result.ghosts.push_back({mutation.u32_payload[0]
                    + mutation.u32_payload[1] * index, position});
        }
        break;
    }
    case 9U: {
        result.samples.clear();
        result.samples.reserve(mutation.u64_payload[0]);
        const Vec3d position{binary64_from_bits(mutation.u64_payload[1]),
            binary64_from_bits(mutation.u64_payload[2]),
            binary64_from_bits(mutation.u64_payload[3])};
        const Vec3d velocity{binary64_from_bits(mutation.u64_payload[4]),
            binary64_from_bits(mutation.u64_payload[5]),
            binary64_from_bits(mutation.u64_payload[6])};
        for (std::uint32_t index = 0U;
             index < mutation.u64_payload[0]; ++index) {
            result.samples.push_back({mutation.u32_payload[0]
                    + mutation.u32_payload[1] * index,
                position, position, velocity});
        }
        result.ghosts.clear();
        result.check_rows = true;
        break;
    }
    default:
        throw std::logic_error("NCGP14 admission mutation index invalid");
    }
    return result;
}

constexpr std::array<std::uint64_t, 9> kAdmissionMutationPortableFields{
    11U, 11U, 11U, 12U, 11U, 12U, 12U, 16U, 19U};

std::string admission_mutation_root(std::string_view base_lane_root,
    const AdmissionMutation14& mutation) {
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.admission-mutation.v1");
    put_string(bytes, base_lane_root);
    put_u32(bytes, mutation.index);
    put_string(bytes, mutation.name);
    put_u64(bytes, mutation.maximum_total_records);
    put_u64(bytes, mutation.expected_ghost_count);
    put_string(bytes, admission_name(mutation.expected_outcome));
    put_string(bytes, admission_name(mutation.observed_outcome));
    put_string(bytes, mutation.rejected_state_root);
    switch (mutation.index) {
    case 1U:
        put_string(bytes, mutation.field_name);
        put_u64(bytes, mutation.u64_payload[0]);
        break;
    case 2U:
        put_u64(bytes, mutation.u64_payload[0]);
        put_u64(bytes, mutation.u64_payload[1]);
        break;
    case 3U:
        put_string(bytes, mutation.field_name);
        put_u32(bytes, mutation.u32_payload[0]);
        break;
    case 4U:
        put_u64(bytes, mutation.u64_payload[0]);
        put_string(bytes, mutation.field_name);
        put_u64(bytes, mutation.u64_payload[1]);
        break;
    case 5U:
        put_u64(bytes, mutation.u64_payload[0]);
        put_u64(bytes, mutation.u64_payload[1]);
        break;
    case 6U:
        put_u64(bytes, mutation.u64_payload[0]);
        put_string(bytes, mutation.field_name);
        put_u64(bytes, mutation.u64_payload[1]);
        break;
    case 7U:
        put_u64(bytes, mutation.u64_payload[0]);
        put_u64(bytes, mutation.u64_payload[1]);
        put_u64(bytes, mutation.u64_payload[2]);
        break;
    case 8U:
        put_u64(bytes, mutation.u64_payload[0]);
        put_u64(bytes, mutation.u64_payload[1]);
        put_u32(bytes, mutation.u32_payload[0]);
        put_u32(bytes, mutation.u32_payload[1]);
        put_u64(bytes, mutation.u64_payload[2]);
        put_u64(bytes, mutation.u64_payload[3]);
        put_u64(bytes, mutation.u64_payload[4]);
        break;
    case 9U:
        put_u64(bytes, mutation.u64_payload[0]);
        put_u32(bytes, mutation.u32_payload[0]);
        put_u32(bytes, mutation.u32_payload[1]);
        for (std::size_t index = 1U; index < mutation.u64_payload.size();
             ++index) {
            put_u64(bytes, mutation.u64_payload[index]);
        }
        break;
    default:
        throw std::logic_error("NCGP14 admission mutation index invalid");
    }
    return root_of(std::move(bytes));
}

struct AdmissionControls14 {
    bool pass = false;
    std::vector<std::string> state_roots;
    std::vector<std::string> outcomes;
    std::vector<AdmissionMutation14> mutations;
    std::vector<bool> outcome_exact;
    std::vector<bool> state_preserved;
    Work14 work;
    Work14 expected_work;
    std::vector<Work14> expected_children;
};

AdmissionControls14 admission_controls(const LanePolicy& base_policy) {
    AdmissionControls14 result;
    const Fixture14& tight = *base_policy.fixture;
    const State prior = working_state14(
        tight.canonical_samples, result.work);
    result.expected_work.records_canonicalized +=
        tight.canonical_samples.size();
    auto root_state = [&](const State& state) {
        ++result.work.parent.hash_derivations;
        result.work.portable_content_fields_serialized +=
            state14_portable_fields(state.id.size());
        result.state_roots.push_back(state14_root(base_policy.root, state));
    };
    root_state(prior);
    auto evaluate = [&](AdmissionMutation14 mutation) {
        const AdmissionMutationInput14 expected_input =
            admission_mutation_input14(tight, mutation);
        result.expected_children.push_back(expected_admission_work(
            expected_input.profile, expected_input.samples,
            expected_input.ghosts, expected_input.check_rows,
            &tight.profile, mutation.expected_ghost_count,
            mutation.maximum_total_records));
        const AdmissionMutationInput14 actual_input =
            admission_mutation_input14(tight, mutation);
        Work14 local;
        AdmissionTrace14 trace;
        const AdmissionDecision14 decision = production_admission14(prior,
            actual_input.profile, actual_input.samples, actual_input.ghosts,
            local, actual_input.check_rows, tight.profile,
            mutation.expected_ghost_count, mutation.maximum_total_records,
            trace);
        add_work14(result.work, local);
        result.outcomes.emplace_back(admission_name(decision.outcome));
        root_state(decision.accepted_state);
        mutation.observed_outcome = decision.outcome;
        mutation.rejected_state_root = result.state_roots.back();
        mutation.root = admission_mutation_root(base_policy.root, mutation);
        ++result.work.parent.hash_derivations;
        result.work.portable_content_fields_serialized +=
            kAdmissionMutationPortableFields.at(mutation.index - 1U);
        const bool outcome_equal = counted_predicate(
            result.work, decision.outcome == mutation.expected_outcome);
        const bool state_root_equal = counted_predicate(result.work,
            result.state_roots.back() == result.state_roots.front());
        result.outcome_exact.push_back(outcome_equal);
        result.state_preserved.push_back(state_root_equal);
        result.mutations.push_back(std::move(mutation));
        return outcome_equal && state_root_equal;
    };
    bool pass = true;
    for (std::uint32_t index = 1U; index <= 9U; ++index) {
        pass = evaluate(admission_mutation14(index)) && pass;
    }
    result.pass = pass;
    for (const Work14& child : result.expected_children) {
        add_work14(result.expected_work, child);
    }
    result.expected_work.parent.hash_derivations = 19U;
    result.expected_work.portable_content_fields_serialized +=
        10U * state14_portable_fields(prior.id.size())
        + std::accumulate(kAdmissionMutationPortableFields.begin(),
            kAdmissionMutationPortableFields.end(), UINT64_C(0));
    result.expected_work.lane_scalar_comparisons += 18U;
    return result;
}

struct WorkMutation14 {
    bool pass = false;
    std::vector<std::string> roots;
    std::vector<bool> root_changed;
    Work14 work;
    Work14 expected_work;
};
WorkMutation14 work_mutations(const Work14& baseline) {
    WorkMutation14 result;
    result.pass = true;
    for (std::size_t mutation = 0U; mutation < 5U; ++mutation) {
        Work14 changed = baseline;
        if (mutation == 0U) ++changed.parent.graph_builds;
        else if (mutation == 1U) ++changed.parent.qp_sweeps;
        else if (mutation == 2U) ++changed.parent.plane_tests;
        else if (mutation == 3U) ++changed.parent.analytic_jv_multiply_adds;
        else ++changed.parent.hash_derivations;
        const std::string before = work14_root(baseline);
        const std::string after = work14_root(changed);
        result.roots.push_back(before);
        result.roots.push_back(after);
        const bool changed_root = counted_predicate(result.work,
            before != after);
        result.root_changed.push_back(changed_root);
        result.pass = changed_root && result.pass;
        result.work.parent.hash_derivations += 2U;
        result.work.portable_content_fields_serialized += 2U * 43U;
    }
    result.expected_work.parent.hash_derivations = 10U;
    result.expected_work.portable_content_fields_serialized = 10U * 43U;
    result.expected_work.lane_scalar_comparisons = 6U;
    const bool root_count = counted_predicate(result.work,
        result.roots.size() == 10U);
    result.pass = root_count && result.pass;
    return result;
}

struct SurfaceControls14 {
    bool pass = false;
    std::array<Surface14, 3> candidate;
    std::array<Surface14, 3> oracle;
    std::array<Surface14, 3> zero;
    std::array<std::array<bool, 7>, 3> fixture_checks{};
    Surface14 wrong_branch;
    Surface14 wrong_sign;
    Surface14 half_force;
    std::array<bool, 4> mutation_checks{};
    Work14 work;
    Work14 expected_work;
};

SurfaceControls14 surface_controls(const std::array<const Fixture14*, 3>& fixtures,
    const std::array<NonlocalGpuProfile, 3>& census_profiles,
    const std::array<std::string, 3>& census_profile_roots) {
    SurfaceControls14 result;
    constexpr std::array<std::uint64_t, 3> expected_pairs{
        2976U, 17764U, 3216U};
    bool pass = true;
    for (std::size_t index = 0U; index < fixtures.size(); ++index) {
        result.candidate[index] = surface_candidate(*fixtures[index],
            census_profiles[index], census_profile_roots[index],
            SurfaceVariant::Correct);
        result.oracle[index] = surface_oracle(*fixtures[index],
            census_profiles[index], census_profile_roots[index]);
        NonlocalGpuProfile zero_profile = census_profiles[index];
        zero_profile.gamma = 0.0;
        result.zero[index] = surface_candidate(*fixtures[index], zero_profile,
            census_profile_roots[index], SurfaceVariant::ZeroGamma);
        add_work14(result.work, result.candidate[index].work);
        add_work14(result.work, result.oracle[index].work);
        add_work14(result.work, result.zero[index].work);
        add_work14(result.expected_work,
            result.candidate[index].expected_work);
        add_work14(result.expected_work,
            result.oracle[index].expected_work);
        add_work14(result.expected_work, result.zero[index].expected_work);
        bool zero_exact = true;
        for (const Vec3l value : result.zero[index].force) {
            combine_predicate(zero_exact, result.work, value.x == 0.0L);
            combine_predicate(zero_exact, result.work, value.y == 0.0L);
            combine_predicate(zero_exact, result.work, value.z == 0.0L);
        }
        result.fixture_checks[index] = std::array<bool, 7>{
                 result.candidate[index].active_pairs == expected_pairs[index],
                 result.oracle[index].active_pairs == expected_pairs[index],
                 result.zero[index].active_pairs == expected_pairs[index],
                 result.candidate[index].root == result.oracle[index].root,
                 force_relative_l2(result.candidate[index],
                     result.oracle[index], result.work) <= 2.0e-12L,
                 result.candidate[index].net_residual <= 1.0e-15L,
                 zero_exact};
        for (const bool predicate : result.fixture_checks[index]) {
            combine_predicate(pass, result.work, predicate);
        }
    }
    result.wrong_branch = surface_candidate(*fixtures[0], census_profiles[0],
        census_profile_roots[0], SurfaceVariant::WrongBranch);
    result.wrong_sign = surface_candidate(*fixtures[0], census_profiles[0],
        census_profile_roots[0], SurfaceVariant::WrongSign);
    result.half_force = surface_candidate(*fixtures[0], census_profiles[0],
        census_profile_roots[0], SurfaceVariant::HalfForce);
    add_work14(result.work, result.wrong_branch.work);
    add_work14(result.work, result.wrong_sign.work);
    add_work14(result.work, result.half_force.work);
    add_work14(result.expected_work, result.wrong_branch.expected_work);
    add_work14(result.expected_work, result.wrong_sign.expected_work);
    add_work14(result.expected_work, result.half_force.expected_work);
    const long double mean_velocity = (112.0L / 128.0L) * 2.0L * 9.81L
        * fixtures[0]->profile.dt;
    result.mutation_checks = std::array<bool, 4>{
             result.wrong_branch.root != result.candidate[0].root,
             result.wrong_sign.root != result.candidate[0].root,
             result.half_force.root != result.candidate[0].root,
             mean_velocity - result.candidate[0].delta_mean_bound
                 > kVelocityRmsLimit};
    for (const bool predicate : result.mutation_checks) {
        combine_predicate(pass, result.work, predicate);
    }
    result.expected_work.lane_scalar_comparisons = 4U;
    result.expected_work.surface_reduction_adds += 2304U;
    for (const Fixture14* fixture : fixtures) {
        result.expected_work.lane_scalar_comparisons +=
            7U + 3U * fixture->canonical_samples.size();
    }
    result.pass = pass;
    return result;
}

struct Identity14 {
    bool pass = false;
    std::string outcome = "IDENTITY_INVALID";
    BinaryRead binary;
    NonlocalGpuProfile open_profile;
    NonlocalGpuProfile tight_profile;
    NonlocalGpuProfile open_census_profile;
    NonlocalGpuProfile tight_census_profile;
    std::array<std::string, 4> profile_roots;
    Fixture14 open128;
    Fixture14 open512;
    Fixture14 tight128;
    std::array<LanePolicy, 5> policies;
    Work14 work;
    Work14 expected_work;
    WorkSeal14 verifier;
    std::string result_root;
};

Work14 expected_identity_work(const Identity14& identity) {
    Work14 result;
    result.parent.hash_derivations = 13U;
    result.fixture_lattice_sites_generated =
        identity.open128.canonical_samples.size()
        + identity.open512.canonical_samples.size()
        + identity.tight128.canonical_samples.size();
    result.ghost_cells_tested = extended_ghost_cells(identity.open_profile)
        + extended_ghost_cells(identity.open_profile)
        + extended_ghost_cells(identity.tight_profile);
    result.records_canonicalized =
        identity.open128.canonical_samples.size()
        + identity.open512.canonical_samples.size()
        + identity.tight128.canonical_samples.size()
        + identity.open128.ghosts.size() + identity.open512.ghosts.size()
        + identity.tight128.ghosts.size();
    result.portable_content_fields_serialized = 4U * 21U + 5U * 23U;
    for (const Fixture14* fixture : std::array<const Fixture14*, 3>{
             &identity.open128, &identity.open512, &identity.tight128}) {
        result.portable_content_fields_serialized += 5U
            + 10U * fixture->canonical_samples.size()
            + 4U * fixture->ghosts.size();
    }
    result.lane_scalar_comparisons = 18U
        + identity.open128.canonical_samples.size()
        + identity.open512.canonical_samples.size()
        + identity.tight128.canonical_samples.size();
    result.binary_file_reads = 1U;
    result.binary_bytes_hashed = identity.binary.bytes;
    return result;
}

std::string seal_identity14(Identity14& identity) {
    if (identity.verifier.expected_root.empty()) {
        identity.verifier = verify_work14(
            identity.work, identity.expected_work);
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.identity-envelope.v1");
    put_string(bytes, identity.outcome);
    put_string(bytes, NCGP14_CONTRACT_ROOT);
    put_string(bytes, NCGP14_SOURCE_ROOT);
    put_string(bytes, identity.binary.root);
    put_string(bytes, NCGP14_COMPILER_FAMILY);
    put_string(bytes, NCGP14_COMPILER_VERSION);
    put_string(bytes, NCGP14_COMPILER_FLAGS);
    put_string(bytes, kInvocation);
    for (const std::string& root : identity.profile_roots) {
        put_string(bytes, root);
    }
    put_string(bytes, identity.open128.fixture_root);
    put_string(bytes, identity.open512.fixture_root);
    put_string(bytes, identity.tight128.fixture_root);
    for (const LanePolicy& policy : identity.policies) {
        put_string(bytes, policy.root);
    }
    append_work_verifier(bytes, identity.verifier);
    identity.result_root = root_of(std::move(bytes));
    return identity.result_root;
}

Identity14 build_identity() {
    Identity14 result;
    result.binary = read_binary();
    result.work.binary_file_reads = 1U;
    result.work.binary_bytes_hashed = result.binary.bytes;
    ++result.work.parent.hash_derivations;
    result.open_profile = integrated_profile(false);
    result.tight_profile = integrated_profile(true);
    result.open_census_profile = census_profile(false);
    result.tight_census_profile = census_profile(true);
    result.profile_roots[0] = profile14_root(result.open_profile, result.work);
    result.profile_roots[1] = profile14_root(result.tight_profile, result.work);
    result.profile_roots[2] = profile14_root(
        result.open_census_profile, result.work);
    result.profile_roots[3] = profile14_root(
        result.tight_census_profile, result.work);
    result.open128 = make_fixture("OPEN-128", result.open_profile,
        4U, 4U, 8U, false, result.profile_roots[0], result.work);
    result.open512 = make_fixture("OPEN-512", result.open_profile,
        8U, 8U, 8U, false, result.profile_roots[0], result.work);
    result.tight128 = make_fixture("TIGHT-128", result.tight_profile,
        4U, 4U, 8U, true, result.profile_roots[1], result.work);
    result.policies[0] = {"OPEN-128-CAP4096", {}, &result.open128,
        2U, 8U, 4096U, false, true};
    result.policies[1] = {"OPEN-512-CAP4096", {}, &result.open512,
        2U, 8U, 4096U, false, false};
    result.policies[2] = {"TIGHT-128-CAP4096", {}, &result.tight128,
        2U, 8U, 4096U, true, false};
    result.policies[3] = {"TIGHT-128-CAP16384", {}, &result.tight128,
        2U, 8U, 16384U, true, false};
    result.policies[4] = {"TIGHT-128-CAP16384-R16", {}, &result.tight128,
        2U, 16U, 16384U, true, false};
    for (LanePolicy& policy : result.policies) {
        policy.root = lane14_root(policy.name,
            policy.fixture->fixture_root, policy.projection_cap,
            policy.qp_cap, result.work);
    }
    bool roots = true;
    for (std::size_t index = 0U; index < result.profile_roots.size(); ++index) {
        combine_predicate(roots, result.work,
            result.profile_roots[index] == kProfileRoots[index]);
    }
    combine_predicate(roots, result.work,
        result.open128.fixture_root == kFixtureRoots[0]);
    combine_predicate(roots, result.work,
        result.open512.fixture_root == kFixtureRoots[1]);
    combine_predicate(roots, result.work,
        result.tight128.fixture_root == kFixtureRoots[2]);
    combine_predicate(roots, result.work,
        result.open128.legacy_input_root == kOpen128Input);
    combine_predicate(roots, result.work,
        result.open512.legacy_input_root == kOpen512Input);
    for (std::size_t index = 0U; index < result.policies.size(); ++index) {
        combine_predicate(roots, result.work,
            result.policies[index].root == kLaneRoots[index]);
    }
    combine_predicate(roots, result.work,
        result.tight128.ghosts.size() == 1608U);
    combine_predicate(roots, result.work,
        result.work.parent.hash_derivations == 13U);
    combine_predicate(roots, result.work,
        result.work.binary_file_reads == 1U);
    combine_predicate(roots, result.work,
        result.work.binary_bytes_hashed == result.binary.bytes);
    result.expected_work = expected_identity_work(result);
    result.verifier = verify_work14(result.work, result.expected_work);
    result.pass = roots && result.verifier.exact;
    result.outcome = result.pass ? "PASS" : "IDENTITY_INVALID";
    seal_identity14(result);
    return result;
}

struct AdmissionReceipt14 {
    std::string name;
    Admission14 outcome = Admission14::InvalidProfile;
    Work14 work;
    Work14 expected_work;
    WorkSeal14 verifier;
    std::string result_root;
};

std::string seal_admission_receipt(AdmissionReceipt14& receipt) {
    receipt.verifier = verify_work14(receipt.work, receipt.expected_work);
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.admission-envelope.v1");
    put_string(bytes, admission_name(receipt.outcome));
    put_u64(bytes, 0U);
    append_work_verifier(bytes, receipt.verifier);
    receipt.result_root = root_of(std::move(bytes));
    return receipt.result_root;
}

struct AdmissionEnvelope14 {
    bool pass = false;
    std::string outcome = "ADMISSION_INVALID";
    std::array<AdmissionReceipt14, 6> children;
    std::size_t child_count = 0U;
    Work14 work;
    Work14 expected_work;
    WorkSeal14 verifier;
    std::string result_root;
};

std::string seal_admission_envelope(AdmissionEnvelope14& envelope) {
    if (envelope.verifier.expected_root.empty()) {
        envelope.verifier = verify_work14(
            envelope.work, envelope.expected_work);
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.admission-envelope.v1");
    put_string(bytes, envelope.outcome);
    put_u64(bytes, envelope.child_count);
    for (std::size_t index = 0U; index < envelope.child_count; ++index) {
        put_string(bytes, envelope.children[index].result_root);
    }
    append_work_verifier(bytes, envelope.verifier);
    envelope.result_root = root_of(std::move(bytes));
    return envelope.result_root;
}

AdmissionEnvelope14 run_raw_admission(const Identity14& identity) {
    AdmissionEnvelope14 result;
    result.child_count = result.children.size();
    const std::array<std::string_view, 6> names{"OPEN-128-CANONICAL",
        "OPEN-128-PERMUTED", "OPEN-512-CANONICAL",
        "OPEN-512-PERMUTED", "TIGHT-128-CANONICAL",
        "TIGHT-128-PERMUTED"};
    const std::array<const NonlocalGpuProfile*, 6> profiles{
        &identity.open_profile, &identity.open_profile,
        &identity.open_profile, &identity.open_profile,
        &identity.tight_profile, &identity.tight_profile};
    const std::array<const std::vector<NonlocalGpuSample>*, 6> samples{
        &identity.open128.canonical_samples, &identity.open128.permuted_samples,
        &identity.open512.canonical_samples, &identity.open512.permuted_samples,
        &identity.tight128.canonical_samples, &identity.tight128.permuted_samples};
    const std::array<const std::vector<NonlocalGpuGhost>*, 6> ghosts{
        &identity.open128.ghosts, &identity.open128.ghosts,
        &identity.open512.ghosts, &identity.open512.ghosts,
        &identity.tight128.ghosts, &identity.tight128.ghosts};
    bool pass = true;
    for (std::size_t index = 0U; index < result.children.size(); ++index) {
        AdmissionReceipt14& child = result.children[index];
        child.name = names[index];
        AdmissionTrace14 trace;
        child.outcome = admit14(*profiles[index], *samples[index],
            *ghosts[index], child.work, true, profiles[index],
            ghosts[index]->size(), 100000U, &trace);
        child.expected_work = expected_admission_work(*profiles[index],
            *samples[index], *ghosts[index], true, profiles[index],
            ghosts[index]->size());
        seal_admission_receipt(child);
        combine_predicate(pass, result.work,
            child.outcome == Admission14::Ok);
        add_child(result.work, child.work);
        add_child(result.expected_work, child.expected_work);
    }
    result.expected_work.lane_scalar_comparisons += result.children.size();
    result.verifier = verify_work14(result.work, result.expected_work);
    result.pass = pass && result.verifier.exact;
    result.outcome = result.pass ? "PASS" : "ADMISSION_INVALID";
    seal_admission_envelope(result);
    return result;
}

AdmissionEnvelope14 skipped_admission_envelope() {
    AdmissionEnvelope14 result;
    result.outcome = "NOT_RUN_BY_PRECEDENCE";
    result.pass = false;
    return result;
}

struct Selectors14 {
    bool invalid_hydrostatic_fixture_supported = false;
    bool tiny_open_support_artifact_supported = false;
    bool qp_budget_limited = false;
    bool projection_budget_limited = false;
    bool projection_cap_saturated = false;
    bool cause_not_unique = false;
    bool surface_direct_norm_insufficient = false;
};

LaneCategoryClosure14 lane_category14(const LaneResult14& lane,
    Work14* owner) {
    LaneCategoryClosure14 result;
    const auto evaluate = [&](bool predicate) {
        return owner == nullptr ? predicate
            : counted_predicate(*owner, predicate);
    };
    result.apparatus_was_valid = evaluate(lane.apparatus_valid);
    const std::array<bool, 4> flags{lane.both_steps_supported,
        lane.physical_gate_rejected, lane.qp_sweep_cap_exhausted,
        lane.projection_round_cap_exhausted};
    std::array<bool, 4> evaluated_flags{};
    for (std::size_t index = 0U; index < flags.size(); ++index) {
        evaluated_flags[index] = evaluate(flags[index]);
    }
    result.one_hot = evaluate(
        std::count(evaluated_flags.begin(), evaluated_flags.end(), true)
            == 1);
    if (!result.apparatus_was_valid || !result.one_hot) {
        return result;
    }
    if (evaluated_flags[0]) result.value = LaneCategory14::Supported;
    else if (evaluated_flags[1]) {
        result.value = LaneCategory14::PhysicalGateRejected;
    } else if (evaluated_flags[2]) {
        result.value = LaneCategory14::QpSweepCapExhausted;
    } else {
        result.value = LaneCategory14::ProjectionRoundCapExhausted;
    }
    return result;
}

LaneCategory14 select_decisive(LaneCategory14 r8,
    const std::optional<LaneCategory14>& r16, Work14* owner = nullptr) {
    const bool execute_r16 = r8
        == LaneCategory14::ProjectionRoundCapExhausted;
    if (owner != nullptr) {
        static_cast<void>(counted_predicate(*owner, execute_r16));
    }
    if (!execute_r16) return r8;
    return r16.value_or(LaneCategory14::ApparatusInvalid);
}

std::string_view category_name14(LaneCategory14 category) {
    switch (category) {
    case LaneCategory14::Supported: return "SUPPORTED";
    case LaneCategory14::PhysicalGateRejected:
        return "PHYSICAL_GATE_REJECTED";
    case LaneCategory14::QpSweepCapExhausted:
        return "QP_SWEEP_CAP_EXHAUSTED";
    case LaneCategory14::ProjectionRoundCapExhausted:
        return "PROJECTION_ROUND_CAP_EXHAUSTED";
    case LaneCategory14::ApparatusInvalid: return "APPARATUS_INVALID";
    }
    throw std::logic_error("unknown NCGP14 lane category");
}

std::array<bool, 7> selector_values(const Selectors14& value) {
    return {value.invalid_hydrostatic_fixture_supported,
        value.tiny_open_support_artifact_supported, value.qp_budget_limited,
        value.projection_budget_limited, value.projection_cap_saturated,
        value.cause_not_unique, value.surface_direct_norm_insufficient};
}

struct Finalization14 {
    std::string outcome = "NOT_RUN_BY_PRECEDENCE";
    std::string decisive_category;
    Selectors14 selectors;
    std::string status = "APPARATUS_INCONCLUSIVE";
    bool total_work_exact = false;
    std::string expected_total_work_root;
    std::string actual_total_work_root;
    Work14 work;
    Work14 expected_work;
    WorkSeal14 verifier;
    std::string result_root;
};

std::string seal_finalization14(Finalization14& finalization) {
    std::string bytes;
    put_string(bytes,
        "nextengine.nonlocal.ncgp14.finalization-envelope.v1");
    put_string(bytes, finalization.outcome);
    put_string(bytes, finalization.decisive_category);
    put_u64(bytes, 7U);
    for (const bool selector : selector_values(finalization.selectors)) {
        put_u8(bytes, selector ? 1U : 0U);
    }
    put_string(bytes, finalization.status);
    put_u8(bytes, finalization.total_work_exact ? 1U : 0U);
    put_string(bytes, finalization.expected_total_work_root);
    put_string(bytes, finalization.actual_total_work_root);
    append_work_verifier(bytes, finalization.verifier);
    finalization.result_root = root_of(std::move(bytes));
    return finalization.result_root;
}

std::string final14_root(const Identity14& identity,
    const AdmissionEnvelope14& admission,
    const EmbeddedParent& embedded,
    const std::array<LaneResult14, 10>& lanes,
    const std::array<Control14, 8>& controls,
    const Finalization14& finalization,
    std::string_view first_failure_stage,
    std::string_view first_failure_cause) {
    std::string bytes;
    for (const std::string_view value : std::array<std::string_view, 10>{
             kSchema, NCGP14_CONTRACT_ROOT, NCGP14_SOURCE_COMMIT,
             NCGP14_SOURCE_TREE, NCGP14_SOURCE_ROOT, identity.binary.root,
             NCGP14_COMPILER_FAMILY, NCGP14_COMPILER_VERSION,
             NCGP14_COMPILER_FLAGS, kInvocation}) {
        put_string(bytes, value);
    }
    put_string(bytes, first_failure_stage);
    put_string(bytes, first_failure_cause);
    put_u8(bytes, identity.pass ? 1U : 0U);
    put_u8(bytes, admission.pass ? 1U : 0U);
    put_u8(bytes, embedded.pass ? 1U : 0U);
    put_optional_root(bytes, true, identity.result_root);
    put_optional_root(bytes, true, admission.result_root);
    put_optional_root(bytes, true, embedded.result_root);
    for (const std::string_view value : std::array<std::string_view, 8>{
             kParentCommit, kParentTree, kParentSourceFile,
             kParentSourceAggregate, kParentContract, kParentBinary,
             kParentStdout, kParentResult}) {
        put_string(bytes, value);
    }
    put_optional_root(bytes, embedded.raw_stdout_root_present,
        embedded.raw_stdout_root);
    put_optional_root(bytes, embedded.recomputed_raw_result_root_present,
        embedded.recomputed_raw_result_root);
    put_optional_root(bytes, embedded.normalized_stdout_root_present,
        embedded.normalized_stdout_root);
    put_optional_root(bytes, embedded.normalized_result_root_present,
        embedded.recomputed_normalized_result_root);
    put_u64(bytes, identity.profile_roots.size());
    for (const std::string& root : identity.profile_roots) put_string(bytes, root);
    put_u64(bytes, 8U);
    put_string(bytes, identity.open128.fixture_root);
    put_string(bytes, identity.open512.fixture_root);
    put_string(bytes, identity.tight128.fixture_root);
    for (const LanePolicy& policy : identity.policies) put_string(bytes, policy.root);
    put_u64(bytes, lanes.size());
    for (const LaneResult14& lane : lanes) put_string(bytes, lane.result_root);
    put_u64(bytes, controls.size());
    for (const Control14& control : controls) put_string(bytes, control.result_root);
    put_u64(bytes, 7U);
    for (const bool selector : selector_values(finalization.selectors)) {
        put_u8(bytes, selector ? 1U : 0U);
    }
    put_string(bytes, finalization.status);
    put_string(bytes, finalization.result_root);
    put_u8(bytes, finalization.total_work_exact ? 1U : 0U);
    put_string(bytes, finalization.expected_total_work_root);
    put_string(bytes, finalization.actual_total_work_root);
    return root_of(std::move(bytes));
}

struct Report14 {
    Identity14 identity;
    EmbeddedParent embedded;
    std::array<LaneResult14, 10> lanes;
    std::array<Control14, 8> controls;
    Geometry14 geometry_candidate;
    Geometry14 geometry_oracle;
    Geometry14 geometry_mutated;
    GeometryMutation14 geometry_mutation;
    Transaction14 transaction;
    AdmissionEnvelope14 raw_admission;
    AdmissionControls14 admission;
    WorkMutation14 mutations;
    SideRemoval14 side;
    SurfaceControls14 surface;
    Finalization14 finalization;
    std::string first_failure_stage;
    std::string first_failure_cause;
    Work14 total_work;
    Work14 expected_total_work;
    WorkSeal14 total_verifier;
    std::string result_root;
};

void emit_report14(std::ostream& output, const Report14& report);

Control14 retained_control(const EmbeddedParent& embedded,
    const LaneResult14& canonical, const LaneResult14& permuted) {
    Control14 result;
    result.index = 1U;
    result.name = "open128-retained-route";
    bool pass = true;
    const bool parent_controls = retained_parent_controls_pass(
        embedded.validated.controls, &result.work);
    pass = parent_controls && pass;
    const ParentRetainedTrajectory14& parent_canonical =
        embedded.validated.retained[0];
    const ParentRetainedTrajectory14& parent_permuted =
        embedded.validated.retained[1];
    const bool parent_envelope_exact = counted_predicate(result.work,
        embedded.pass && embedded.inherited_parent_result_root_present
            && embedded.inherited_parent_result_root == kParentResult);
    const bool trial1_committed_exact = counted_predicate(result.work,
        canonical.attempted_trials == 2U
            && canonical.committed_trials == 1U
            && canonical.trial_1_committed
            && parent_canonical.steps.size() == 2U
            && parent_permuted.steps.size() == 2U
            && parent_canonical.steps[0].committed
            && parent_permuted.steps[0].committed);
    const bool trial2_rejected_exact = counted_predicate(result.work,
        canonical.trial_2_attempted && !canonical.trial_2_committed
            && canonical.physical_gate_rejected
            && !parent_canonical.steps[1].committed
            && !parent_permuted.steps[1].committed);
    const bool trial1_state_root_exact = counted_predicate(result.work,
        canonical.accepted_state_root == parent_canonical.state_root
            && canonical.accepted_state_root == parent_permuted.state_root
            && permuted.accepted_state_root == canonical.accepted_state_root);
    const bool trial2_state_root_exact = counted_predicate(result.work,
        canonical.failing_state_root
                == parent_canonical.failing_trial_state_root
            && canonical.failing_state_root
                == parent_permuted.failing_trial_state_root
            && permuted.failing_state_root == canonical.failing_state_root);
    const bool trajectory_root_exact = counted_predicate(result.work,
        canonical.trajectory_root == parent_canonical.trajectory_root
            && canonical.trajectory_root == parent_permuted.trajectory_root
            && permuted.trajectory_root == canonical.trajectory_root);
    const bool work_root_exact = counted_predicate(result.work,
        parent_canonical.work_root == parent_permuted.work_root);
    const bool canonical_permuted_exact = counted_predicate(result.work,
        canonical.result_root == permuted.result_root);
    for (const bool predicate : std::array<bool, 8>{parent_envelope_exact,
             trial1_committed_exact, trial2_rejected_exact,
             trial1_state_root_exact, trial2_state_root_exact,
             trajectory_root_exact, work_root_exact,
             canonical_permuted_exact}) {
        pass = predicate && pass;
    }
    result.expected_work.lane_scalar_comparisons = 64U;
    result.pass = pass;
    result.outcome = result.pass ? "PASS" : "CONTROL_INVALID";
    result.evidence_roots = {embedded.result_root, canonical.result_root,
        permuted.result_root, canonical.attempted_step_roots[0],
        canonical.rejected_step_roots[0], canonical.trajectory_root};
    result.scalars = {scalar_bool("parent_envelope_exact",
                          parent_envelope_exact),
        scalar_bool("trial1_committed_exact", trial1_committed_exact),
        scalar_bool("trial2_rejected_exact", trial2_rejected_exact),
        scalar_bool("trial1_state_root_exact", trial1_state_root_exact),
        scalar_bool("trial2_state_root_exact", trial2_state_root_exact),
        scalar_bool("trajectory_root_exact", trajectory_root_exact),
        scalar_bool("work_root_exact", work_root_exact),
        scalar_bool("canonical_permuted_exact", canonical_permuted_exact)};
    seal_control14(result);
    result.pass = result.pass && result.verifier.exact;
    return result;
}

Control14 geometry_control(Report14& report) {
    Control14 result;
    result.index = 3U;
    result.name = "independent-geometry-census";
    report.geometry_candidate = candidate_geometry(report.identity.tight128);
    report.geometry_oracle = oracle_geometry(report.identity.tight128,
        report.identity.tight128.ghosts);
    const auto mutated_ghosts = mutate_top_layer(report.identity.tight_profile,
        report.identity.tight128.ghosts, result.work);
    result.expected_work.lane_scalar_comparisons +=
        report.identity.tight128.ghosts.size();
    Fixture14 mutated_fixture = report.identity.tight128;
    mutated_fixture.ghosts = mutated_ghosts;
    report.geometry_mutated = candidate_geometry(mutated_fixture,
        &report.identity.tight128.ghosts);
    report.geometry_mutation = geometry_mutation_witness(
        report.identity.tight128, mutated_ghosts,
        report.geometry_candidate.root, report.geometry_mutated.root);
    add_work14(result.work, report.geometry_candidate.work);
    add_work14(result.work, report.geometry_oracle.work);
    add_work14(result.work, report.geometry_mutated.work);
    add_work14(result.work, report.geometry_mutation.work);
    add_work14(result.expected_work, report.geometry_candidate.expected_work);
    add_work14(result.expected_work, report.geometry_oracle.expected_work);
    add_work14(result.expected_work, report.geometry_mutated.expected_work);
    add_work14(result.expected_work, report.geometry_mutation.expected_work);
    result.work.parent.hash_derivations += 4U;
    result.expected_work.parent.hash_derivations += 4U;
    result.work.portable_content_fields_serialized += 4U * 43U;
    result.expected_work.portable_content_fields_serialized += 4U * 43U;
    const auto census_fields = [](const FaceCounts& counts) {
        return std::array<std::uint64_t, 13>{128U, 1608U,
            counts.dynamic_dynamic, counts.dynamic_ghost,
            counts.dynamic_dynamic + counts.dynamic_ghost,
            counts.lateral_unique, counts.membership[0],
            counts.membership[1], counts.membership[2],
            counts.membership[3], counts.membership[4],
            counts.membership[5], counts.maximum_row};
    };
    const std::array<std::uint64_t, 13> frozen{128U, 1608U, 6560U,
        6988U, 13548U, 6480U, 1809U, 1865U, 1809U, 1865U, 885U,
        0U, 117U};
    const auto candidate_fields = census_fields(
        report.geometry_candidate.counts);
    const auto oracle_fields = census_fields(report.geometry_oracle.counts);
    bool pass = true;
    std::array<bool, 13> field_exact{};
    for (std::size_t index = 0U; index < field_exact.size(); ++index) {
        const bool correspondence = counted_predicate(result.work,
            candidate_fields[index] == oracle_fields[index]);
        const bool frozen_exact = counted_predicate(result.work,
            oracle_fields[index] == frozen[index]);
        field_exact[index] = correspondence && frozen_exact;
        pass = field_exact[index] && pass;
    }
    const bool record_count_exact = counted_predicate(result.work,
        report.geometry_mutation.records.size() == 100U);
    bool records_exact = true;
    std::map<std::uint32_t, NonlocalGpuGhost> originals;
    for (const NonlocalGpuGhost& ghost : report.identity.tight128.ghosts) {
        originals.emplace(ghost.sample_id, ghost);
    }
    for (const GeometryMutationRecord14& record
         : report.geometry_mutation.records) {
        const auto found = originals.find(record.id);
        const NonlocalGpuGhost expected = found == originals.end()
            ? NonlocalGpuGhost{} : found->second;
        const auto expected_cell = ghost_cell(
            report.identity.tight_profile, expected);
        for (const bool predicate : std::array<bool, 10>{
                 found != originals.end(),
                 record.cell[0] == expected_cell[0],
                 record.cell[1] == expected_cell[1],
                 record.cell[2] == 12,
                 record.old_position.x == expected.position.x,
                 record.old_position.y == expected.position.y,
                 record.old_position.z == expected.position.z,
                 record.new_position.x == expected.position.x,
                 record.new_position.y == expected.position.y,
                 record.new_position.z
                     == static_cast<double>(static_cast<float>(0.5))}) {
            combine_predicate(records_exact, result.work, predicate);
        }
    }
    const bool mutated_top_pairs_exact = counted_predicate(result.work,
        report.geometry_mutated.counts.membership[5] == 144U);
    const bool mutation_changes_root = counted_predicate(result.work,
        report.geometry_mutated.root != report.geometry_candidate.root);
    pass = record_count_exact && records_exact && mutated_top_pairs_exact
        && mutation_changes_root && pass;
    result.expected_work.lane_scalar_comparisons += 1029U;
    result.pass = pass;
    result.outcome = result.pass ? "PASS" : "CONTROL_INVALID";
    result.evidence_roots = {report.geometry_candidate.graph_root,
        report.geometry_candidate.root, report.geometry_oracle.root,
        report.geometry_mutated.root, report.geometry_mutation.root};
    constexpr std::array<std::string_view, 13> field_names{
        "dynamic_count_exact", "ghost_count_exact",
        "dynamic_dynamic_pairs_exact", "dynamic_ghost_pairs_exact",
        "total_directed_pairs_exact", "lateral_unique_pairs_exact",
        "x_low_memberships_exact", "x_high_memberships_exact",
        "y_low_memberships_exact", "y_high_memberships_exact",
        "z_low_memberships_exact", "z_high_memberships_exact",
        "maximum_owner_row_degree_exact"};
    for (std::size_t index = 0U; index < field_names.size(); ++index) {
        result.scalars.push_back(scalar_bool(
            std::string(field_names[index]), field_exact[index]));
    }
    result.scalars.push_back(scalar_bool(
        "mutation_record_count_exact", record_count_exact));
    result.scalars.push_back(scalar_bool(
        "mutation_records_exact", records_exact));
    result.scalars.push_back(scalar_bool(
        "mutated_top_pairs_exact", mutated_top_pairs_exact));
    result.scalars.push_back(scalar_bool(
        "mutation_changes_census_root", mutation_changes_root));
    seal_control14(result);
    result.pass = result.pass && result.verifier.exact;
    return result;
}

Control14 admission_control(Report14& report) {
    report.admission = admission_controls(report.identity.policies[3]);
    Control14 result;
    result.index = 7U;
    result.name = "tight-profile-admission";
    result.pass = report.admission.pass;
    result.outcome = result.pass ? "PASS" : "CONTROL_INVALID";
    result.work = report.admission.work;
    result.expected_work = report.admission.expected_work;
    result.evidence_roots.push_back(report.admission.state_roots.front());
    for (std::size_t index = 0U; index < report.admission.mutations.size();
         ++index) {
        result.evidence_roots.push_back(
            report.admission.mutations[index].root);
        result.evidence_roots.push_back(
            report.admission.state_roots[index + 1U]);
        result.scalars.push_back(scalar_bool(
            "mutation_" + std::to_string(index + 1U)
                + "_outcome_exact",
            report.admission.outcome_exact[index]));
        result.scalars.push_back(scalar_bool(
            "mutation_" + std::to_string(index + 1U)
                + "_state_preserved",
            report.admission.state_preserved[index]));
    }
    seal_control14(result);
    result.pass = result.pass && result.verifier.exact;
    return result;
}

Control14 transaction_control_receipt(Report14& report) {
    report.transaction = transaction_control(report.identity.policies[3]);
    Control14 result;
    result.index = 5U;
    result.name = "transactional-failure";
    result.pass = report.transaction.pass;
    result.outcome = result.pass ? "PASS" : "CONTROL_INVALID";
    result.work = report.transaction.work;
    result.expected_work = report.transaction.expected_work;
    for (const std::string* root :
         std::array<const std::string*, 12>{
             &report.transaction.prior_root,
             &report.transaction.qp.pre_scratch_root,
             &report.transaction.qp.post_scratch_root,
             &report.transaction.qp_rejected_root,
             &report.transaction.qp.rejection.result_root,
             &report.transaction.qp.trial_skip.result_root,
             &report.transaction.forced.pre_scratch_root,
             &report.transaction.forced.post_scratch_root,
             &report.transaction.forced_rejected_root,
             &report.transaction.forced.rejection.result_root,
             &report.transaction.forced.trial_skip.result_root,
             &report.transaction.empty_scratch_root}) {
        if (root->empty()) break;
        result.evidence_roots.push_back(*root);
    }
    if (result.evidence_roots.size() == 12U) {
        result.scalars = {scalar_bool("QP_CAP_1_SUBROUTE",
                report.transaction.qp_cap_1_subroute),
            scalar_u64("effective_qp_cap", 1U),
            scalar_bool("FORCED_AFTER_PRIVATE_OBSERVABLE_SEAL",
                report.transaction.forced_hook_observed)};
    }
    seal_control14(result);
    result.pass = result.pass && result.verifier.exact;
    return result;
}

Control14 work_control(Report14& report) {
    report.mutations = work_mutations(report.identity.work);
    Control14 result;
    result.index = 6U;
    result.name = "work-cap-route-witness";
    result.work = report.mutations.work;
    result.expected_work = report.mutations.expected_work;
    constexpr std::array<std::string_view, 5> mutation_scalar_names{
        "graph_builds_root_changed", "qp_sweeps_root_changed",
        "plane_tests_root_changed",
        "analytic_jv_multiply_adds_root_changed",
        "hash_derivations_root_changed"};
    for (std::size_t index = 0U;
         index < mutation_scalar_names.size(); ++index) {
        result.scalars.push_back(scalar_bool(
            std::string(mutation_scalar_names[index]),
            report.mutations.root_changed[index]));
    }
    bool pass = true;
    for (const bool predicate : std::array<bool, 14>{
             report.identity.policies[2].fixture->profile_root
                 == report.identity.policies[3].fixture->profile_root,
             report.identity.policies[2].fixture->fixture_root
                 == report.identity.policies[3].fixture->fixture_root,
             report.identity.policies[2].maximum_steps
                 == report.identity.policies[3].maximum_steps,
             report.identity.policies[2].projection_cap
                 == report.identity.policies[3].projection_cap,
             report.identity.policies[2].qp_cap == 4096U,
             report.identity.policies[3].qp_cap == 16384U,
             report.identity.policies[2].tight,
             report.identity.policies[3].tight,
             !report.identity.policies[2].retained,
             !report.identity.policies[3].retained,
             !report.identity.policies[2].disable_xy_contact,
             !report.identity.policies[3].disable_xy_contact,
             report.identity.policies[2].name
                 != report.identity.policies[3].name,
             report.identity.policies[2].root
                 != report.identity.policies[3].root}) {
        combine_predicate(pass, result.work, predicate);
    }
    struct RouteRow14 {
        LaneCategory14 r8;
        std::optional<LaneCategory14> r16;
        LaneCategory14 expected;
    };
    const std::array<RouteRow14, 7> rows{{
        {LaneCategory14::Supported, std::nullopt,
            LaneCategory14::Supported},
        {LaneCategory14::PhysicalGateRejected, std::nullopt,
            LaneCategory14::PhysicalGateRejected},
        {LaneCategory14::QpSweepCapExhausted, std::nullopt,
            LaneCategory14::QpSweepCapExhausted},
        {LaneCategory14::ProjectionRoundCapExhausted,
            LaneCategory14::Supported, LaneCategory14::Supported},
        {LaneCategory14::ProjectionRoundCapExhausted,
            LaneCategory14::PhysicalGateRejected,
            LaneCategory14::PhysicalGateRejected},
        {LaneCategory14::ProjectionRoundCapExhausted,
            LaneCategory14::QpSweepCapExhausted,
            LaneCategory14::QpSweepCapExhausted},
        {LaneCategory14::ProjectionRoundCapExhausted,
            LaneCategory14::ProjectionRoundCapExhausted,
            LaneCategory14::ProjectionRoundCapExhausted}}};
    constexpr std::array<std::string_view, 7> route_scalar_names{
        "r8_supported", "r8_physical", "r8_qp_cap",
        "r16_supported", "r16_physical", "r16_qp_cap",
        "r16_projection_cap"};
    for (std::size_t index = 0U; index < rows.size(); ++index) {
        const RouteRow14& row = rows[index];
        const LaneCategory14 actual = select_decisive(
            row.r8, row.r16, &result.work);
        const bool row_pass = counted_predicate(result.work,
            actual == row.expected);
        pass = row_pass && pass;
        result.scalars.push_back(scalar_bool(
            std::string(route_scalar_names[index]), row_pass));
    }
    struct WitnessRow14 {
        bool tight;
        std::array<bool, 4> witnesses;
        bool expected;
    };
    const std::array<WitnessRow14, 9> witness_rows{{
        {false, {false, false, false, false}, false},
        {false, {false, true, false, false}, false},
        {false, {true, false, false, false}, false},
        {false, {true, true, false, false}, true},
        {true, {false, true, true, true}, false},
        {true, {true, false, true, true}, false},
        {true, {true, true, false, true}, false},
        {true, {true, true, true, false}, false},
        {true, {true, true, true, true}, true}}};
    constexpr std::array<std::string_view, 9> witness_scalar_names{
        "open_00", "open_01", "open_10", "open_11",
        "tight_step1_no_pressure", "tight_step1_no_contact",
        "tight_step2_no_pressure", "tight_step2_no_contact",
        "tight_all_present"};
    for (std::size_t index = 0U; index < witness_rows.size(); ++index) {
        const WitnessRow14& row = witness_rows[index];
        const bool actual = full_lane_acceptance14(row.tight, true,
            row.witnesses[0], row.witnesses[1], row.witnesses[2],
            row.witnesses[3], &result.work);
        const bool row_pass = counted_predicate(result.work,
            actual == row.expected);
        pass = row_pass && pass;
        result.scalars.push_back(scalar_bool(
            std::string(witness_scalar_names[index]), row_pass));
    }
    result.pass = report.mutations.pass && pass;
    result.outcome = result.pass ? "PASS" : "CONTROL_INVALID";
    result.evidence_roots = report.mutations.roots;
    result.expected_work.lane_scalar_comparisons += 46U;
    seal_control14(result);
    result.pass = result.pass && result.verifier.exact;
    return result;
}

Control14 side_control_receipt(Report14& report) {
    report.side = side_removal_control(report.identity.policies[3], {});
    Control14 result;
    result.index = 2U;
    result.name = "side-support-removal";
    result.pass = report.side.pass;
    result.outcome = result.pass ? "PASS" : "CONTROL_INVALID";
    result.work = report.side.work;
    result.expected_work = report.side.expected_work;
    result.evidence_roots.push_back(report.side.fixture.fixture_root);
    if (!report.side.analytic_velocity_root.empty()) {
        result.evidence_roots.push_back(report.side.analytic_velocity_root);
    }
    for (const Step14& step : report.side.steps) {
        result.evidence_roots.push_back(step.step.result_root);
    }
    for (const TrialObservables14& observables : report.side.observables) {
        result.evidence_roots.push_back(observables.root);
    }
    result.scalars.push_back(scalar_u64(
        "ghost_count", report.side.fixture.ghosts.size()));
    for (std::size_t index = 0U;
         index < report.side.velocity_rms.size(); ++index) {
        result.scalars.push_back(scalar_f64(
            "step" + std::to_string(index + 1U) + "_velocity_rms",
            report.side.velocity_rms[index]));
        result.scalars.push_back(scalar_f64(
            "step" + std::to_string(index + 1U)
                + "_analytic_velocity_rms",
            report.side.expected_velocity_rms[index]));
    }
    seal_control14(result);
    result.pass = result.pass && result.verifier.exact;
    return result;
}

Control14 surface_control_receipt(Report14& report) {
    const std::array<NonlocalGpuProfile, 3> profiles{
        report.identity.open_census_profile,
        report.identity.open_census_profile,
        report.identity.tight_census_profile};
    const std::array<std::string, 3> roots{
        report.identity.profile_roots[2], report.identity.profile_roots[2],
        report.identity.profile_roots[3]};
    report.surface = surface_controls({&report.identity.open128,
        &report.identity.open512, &report.identity.tight128}, profiles, roots);
    Control14 result;
    result.index = 8U;
    result.name = "surface-force-census";
    result.pass = report.surface.pass;
    result.outcome = result.pass ? "PASS" : "CONTROL_INVALID";
    result.work = report.surface.work;
    result.expected_work = report.surface.expected_work;
    for (std::size_t index = 0U; index < 3U; ++index) {
        result.evidence_roots.push_back(report.surface.candidate[index].root);
        result.evidence_roots.push_back(report.surface.oracle[index].root);
        result.evidence_roots.push_back(report.surface.zero[index].root);
    }
    result.evidence_roots.push_back(report.surface.wrong_branch.root);
    result.evidence_roots.push_back(report.surface.wrong_sign.root);
    result.evidence_roots.push_back(report.surface.half_force.root);
    constexpr std::array<std::string_view, 3> fixture_names{
        "open128", "open512", "tight128"};
    constexpr std::array<std::string_view, 7> check_names{
        "candidate_pair_count_exact", "oracle_pair_count_exact",
        "zero_pair_count_exact", "candidate_oracle_root_exact",
        "relative_l2_pass", "net_residual_pass", "zero_force_exact"};
    for (std::size_t fixture = 0U; fixture < fixture_names.size(); ++fixture) {
        for (std::size_t check = 0U; check < check_names.size(); ++check) {
            result.scalars.push_back(scalar_bool(
                std::string(fixture_names[fixture]) + "_"
                    + std::string(check_names[check]),
                report.surface.fixture_checks[fixture][check]));
        }
    }
    constexpr std::array<std::string_view, 4> mutation_names{
        "open128_wrong_branch_rejected", "open128_wrong_sign_rejected",
        "open128_half_force_rejected", "open128_mean_bound_pass"};
    for (std::size_t index = 0U; index < mutation_names.size(); ++index) {
        result.scalars.push_back(scalar_bool(
            std::string(mutation_names[index]),
            report.surface.mutation_checks[index]));
    }
    result.scalars.push_back(scalar_f64("open128_two_step_velocity_scale",
        report.surface.candidate[0].two_step_velocity_scale));
    result.scalars.push_back(scalar_f64("open128_delta_mean_bound",
        report.surface.candidate[0].delta_mean_bound));
    seal_control14(result);
    result.pass = result.pass && result.verifier.exact;
    return result;
}

struct PermutationControl14 {
    Control14 receipt;
    bool started = false;
    bool raw_valid = false;
    std::uint32_t lane_equalities = 0U;
    bool trigger_evaluated = false;
};

Work14 expected_permutation_work14(const Identity14& identity,
    const PermutationControl14& state) {
    Work14 result;
    if (!state.started) return result;
    result.parent.hash_derivations = 6U;
    result.raw_order_records_hashed = 2U
        * (identity.open128.canonical_samples.size()
            + identity.open128.ghosts.size()
            + identity.open512.canonical_samples.size()
            + identity.open512.ghosts.size()
            + identity.tight128.canonical_samples.size()
            + identity.tight128.ghosts.size());
    for (const Fixture14* fixture : std::array<const Fixture14*, 3>{
             &identity.open128, &identity.open512, &identity.tight128}) {
        result.portable_content_fields_serialized += 2U
            * (5U + 10U * fixture->canonical_samples.size()
                + 4U * fixture->ghosts.size());
    }
    result.lane_scalar_comparisons = 3U + state.lane_equalities
        + (state.trigger_evaluated ? 1U : 0U);
    return result;
}

PermutationControl14 begin_permutation_control14(
    const Identity14& identity) {
    PermutationControl14 state;
    Control14& result = state.receipt;
    state.started = true;
    result.index = 4U;
    result.name = "canonical-permutation";
    for (const Fixture14* fixture : std::array<const Fixture14*, 3>{
             &identity.open128, &identity.open512, &identity.tight128}) {
        result.evidence_roots.push_back(raw_order_root(fixture->name,
            fixture->profile_root, fixture->canonical_samples,
            fixture->ghosts, result.work));
        result.evidence_roots.push_back(raw_order_root(fixture->name,
            fixture->profile_root, fixture->permuted_samples,
            fixture->ghosts, result.work));
    }
    bool raw_valid = true;
    constexpr std::array<std::string_view, 3> names{
        "open128_raw_different", "open512_raw_different",
        "tight128_raw_different"};
    for (std::size_t index = 0U; index < names.size(); ++index) {
        const bool different = counted_predicate(result.work,
            result.evidence_roots[2U * index]
                != result.evidence_roots[2U * index + 1U]);
        raw_valid = different && raw_valid;
        result.scalars.push_back(scalar_bool(
            std::string(names[index]), different));
    }
    state.raw_valid = raw_valid;
    return state;
}

bool compare_permutation_lane14(PermutationControl14& state,
    const LaneResult14& canonical, const LaneResult14& permuted,
    std::string name) {
    const bool exact = counted_predicate(state.receipt.work,
        canonical.result_root == permuted.result_root);
    state.receipt.scalars.push_back(scalar_bool(std::move(name), exact));
    ++state.lane_equalities;
    return exact;
}

bool record_r16_trigger14(PermutationControl14& state,
    LaneCategory14 matched_r8_category) {
    const bool triggered = counted_predicate(state.receipt.work,
        matched_r8_category == LaneCategory14::ProjectionRoundCapExhausted);
    state.receipt.scalars.push_back(
        scalar_bool("r16_triggered", triggered));
    state.trigger_evaluated = true;
    return triggered;
}

Control14 close_permutation_control14(PermutationControl14& state,
    const Identity14& identity,
    const std::array<LaneResult14, 10>& lanes, std::string outcome) {
    Control14& result = state.receipt;
    for (const LaneResult14& lane : lanes) {
        result.evidence_roots.push_back(lane.result_root);
    }
    result.expected_work = expected_permutation_work14(identity, state);
    result.pass = outcome == "PASS";
    result.outcome = std::move(outcome);
    result.verifier = verify_work14(result.work, result.expected_work);
    if (!result.verifier.exact
        && result.outcome != "PREFIX_CLOSED_BY_PRECEDENCE") {
        result.pass = false;
        result.outcome = "CONTROL_INVALID";
    }
    std::string bytes;
    put_string(bytes, "nextengine.nonlocal.ncgp14.control.v1");
    put_u32(bytes, result.index);
    put_string(bytes, result.name);
    put_string(bytes, result.outcome);
    put_u64(bytes, result.evidence_roots.size());
    for (const std::string& root : result.evidence_roots) {
        put_string(bytes, root);
    }
    put_u64(bytes, result.scalars.size());
    for (const Scalar14& scalar : result.scalars) {
        put_string(bytes, scalar.name);
        put_u8(bytes, scalar.type);
        if (scalar.type == 0U) put_u8(bytes, scalar.boolean ? 1U : 0U);
        else if (scalar.type == 1U) put_u32(bytes, scalar.u32);
        else if (scalar.type == 2U) put_u64(bytes, scalar.u64);
        else if (scalar.type == 3U) put_f64(bytes, scalar.f64);
        else throw std::logic_error("NCGP14 scalar type invalid");
    }
    append_work_verifier(bytes, result.verifier);
    result.result_root = root_of(std::move(bytes));
    result.pass = result.pass && result.verifier.exact;
    return std::move(result);
}

Finalization14 evaluate_finalization(Report14& report, bool apparatus) {
    Finalization14 result;
    if (!apparatus) {
        result.outcome = "NOT_RUN_BY_PRECEDENCE";
        return result;
    }
    const LaneCategory14 r8 = report.lanes[6].category;
    const bool r16_executed = counted_predicate(result.work,
        report.lanes[8].attempted_trials != 0U);
    const std::optional<LaneCategory14> r16 = r16_executed
        ? std::optional<LaneCategory14>{report.lanes[8].category}
        : std::nullopt;
    const LaneCategory14 decisive = select_decisive(r8, r16, &result.work);
    bool supported = counted_predicate(result.work,
        decisive == LaneCategory14::Supported);
    const bool physical = counted_predicate(result.work,
        decisive == LaneCategory14::PhysicalGateRejected);
    const bool qp = counted_predicate(result.work,
        decisive == LaneCategory14::QpSweepCapExhausted);
    const bool projection = counted_predicate(result.work,
        decisive == LaneCategory14::ProjectionRoundCapExhausted);
    result.decisive_category = category_name14(decisive);
    if (supported) {
        result.status = "CONFINED_PRESSURE_CONTACT_SUPPORTED";
    } else if (physical) {
        result.status = "CONFINED_PRESSURE_CONTACT_REFUTED";
    } else if (qp || projection) {
        result.status = "SOLVER_WORK_CEILING_INCONCLUSIVE";
    } else {
        result.status = "APPARATUS_INCONCLUSIVE";
        supported = false;
    }
    bool invalid_hydrostatic = true;
    combine_predicate(invalid_hydrostatic, result.work, supported);
    combine_predicate(invalid_hydrostatic, result.work,
        report.lanes[0].physical_gate_rejected);
    combine_predicate(invalid_hydrostatic, result.work,
        report.lanes[2].physical_gate_rejected);
    result.selectors.invalid_hydrostatic_fixture_supported =
        invalid_hydrostatic;
    bool tiny = true;
    combine_predicate(tiny, result.work,
        report.lanes[0].physical_gate_rejected);
    combine_predicate(tiny, result.work,
        report.lanes[2].both_steps_supported);
    result.selectors.tiny_open_support_artifact_supported = tiny;
    bool qp_budget = true;
    combine_predicate(qp_budget, result.work,
        report.lanes[4].qp_sweep_cap_exhausted);
    combine_predicate(qp_budget, result.work, supported);
    result.selectors.qp_budget_limited = qp_budget;
    bool projection_budget = true;
    combine_predicate(projection_budget, result.work,
        report.lanes[6].projection_round_cap_exhausted);
    const bool projection_r16_executed = counted_predicate(
        result.work, r16_executed);
    combine_predicate(projection_budget, result.work,
        report.lanes[8].both_steps_supported);
    projection_budget = projection_r16_executed && projection_budget;
    result.selectors.projection_budget_limited = projection_budget;
    const bool saturation_r16_executed = counted_predicate(
        result.work, r16_executed);
    const bool selected_saturation = saturation_r16_executed
        ? report.lanes[8].projection_cap_saturated
        : report.lanes[6].projection_cap_saturated;
    result.selectors.projection_cap_saturated = counted_predicate(
        result.work, selected_saturation);
    bool cause_not_unique = true;
    combine_predicate(cause_not_unique, result.work,
        report.lanes[2].both_steps_supported);
    combine_predicate(cause_not_unique, result.work, supported);
    result.selectors.cause_not_unique = cause_not_unique;
    bool surface_insufficient = true;
    combine_predicate(surface_insufficient, result.work,
        report.controls[7].pass);
    const long double excess = std::max(
        report.embedded.validated.retained[0].failing_trial_velocity_rms
        - kVelocityRmsLimit, 0.0L);
    combine_predicate(surface_insufficient, result.work,
        report.surface.candidate[0].two_step_velocity_scale < excess);
    result.selectors.surface_direct_norm_insufficient = surface_insufficient;
    result.expected_work.lane_scalar_comparisons = 22U;
    result.outcome = "PASS";
    return result;
}

int run_successor() {
    Report14 report;
    constexpr std::array<std::string_view, 8> control_names{
        "open128-retained-route", "side-support-removal",
        "independent-geometry-census", "canonical-permutation",
        "transactional-failure", "work-cap-route-witness",
        "tight-profile-admission", "surface-force-census"};
    for (std::size_t index = 0U; index < report.controls.size(); ++index) {
        report.controls[index] = skipped_control14(
            static_cast<std::uint32_t>(index + 1U),
            std::string(control_names[index]));
    }
    report.embedded = skipped_embedded_parent();
    report.raw_admission = skipped_admission_envelope();
    report.identity = build_identity();
    report.identity.policies[0].fixture = &report.identity.open128;
    report.identity.policies[1].fixture = &report.identity.open512;
    report.identity.policies[2].fixture = &report.identity.tight128;
    report.identity.policies[3].fixture = &report.identity.tight128;
    report.identity.policies[4].fixture = &report.identity.tight128;
    const std::array<std::size_t, 10> lane_policy{
        0U, 0U, 1U, 1U, 2U, 2U, 3U, 3U, 4U, 4U};
    for (std::size_t index = 0U; index < report.lanes.size(); ++index) {
        report.lanes[index] = skipped_lane(
            report.identity.policies[lane_policy[index]],
            "NOT_RUN_APPARATUS_PRECEDENCE", false, false);
    }
    const auto fail = [&](std::string_view stage, std::string_view cause) {
        if (report.first_failure_stage.empty()) {
            report.first_failure_stage = stage;
            report.first_failure_cause = cause;
        }
    };
    PermutationControl14 permutation;
    const auto seal_missing_lanes = [&](std::string_view cause,
                                        bool lane_apparatus_valid) {
        for (std::size_t index = 0U; index < report.lanes.size(); ++index) {
            if (!report.lanes[index].result_root.empty()) continue;
            report.lanes[index] = skipped_lane(
                report.identity.policies[lane_policy[index]],
                std::string(cause), lane_apparatus_valid);
            lane_result_root(report.lanes[index]);
        }
    };
    const auto close_permutation = [&](std::string outcome,
                                       std::string_view skip_cause,
                                       bool lane_apparatus_valid) {
        seal_missing_lanes(skip_cause, lane_apparatus_valid);
        report.controls[3] = close_permutation_control14(permutation,
            report.identity, report.lanes, std::move(outcome));
    };
    bool apparatus = report.identity.pass;
    if (!apparatus) fail("IDENTITY", "IDENTITY_INVALID");
    if (apparatus) {
        report.raw_admission = run_raw_admission(report.identity);
        apparatus = report.raw_admission.pass;
        if (!apparatus) fail("ADMISSION", "ADMISSION_INVALID");
    }
    if (apparatus) {
        report.embedded = run_embedded_parent(report.identity.binary.root,
            report.identity.binary.bytes);
        apparatus = report.embedded.pass;
        if (!apparatus) {
            fail("EMBEDDED_PARENT", "EMBEDDED_PARENT_INVALID");
        }
    }
    if (apparatus) {
        LaneResult14 retained_canonical = retained_open128_lane(
            report.identity.policies[0],
            report.embedded.validated.retained[0]);
        LaneResult14 retained_permuted = retained_open128_lane(
            report.identity.policies[0],
            report.embedded.validated.retained[1]);
        Control14 retained = retained_control(report.embedded,
            retained_canonical, retained_permuted);
        report.lanes[0] = std::move(retained_canonical);
        report.lanes[1] = std::move(retained_permuted);
        report.controls[0] = std::move(retained);
        apparatus = report.controls[0].pass;
        if (!apparatus) fail("CONTROL_1", "CONTROL_INVALID");
    }
    if (apparatus) {
        permutation = begin_permutation_control14(report.identity);
        if (!permutation.raw_valid) {
            apparatus = false;
            fail("CONTROL_4", "CONTROL_INVALID");
            close_permutation("CONTROL_INVALID",
                "NOT_RUN_APPARATUS_PRECEDENCE", false);
        } else {
            const bool retained_exact = compare_permutation_lane14(
                permutation, report.lanes[0], report.lanes[1],
                "open128_result_exact");
            if (!retained_exact) {
                apparatus = false;
                fail("CONTROL_4", "CONTROL_INVALID");
                close_permutation("CONTROL_INVALID",
                    "NOT_RUN_APPARATUS_PRECEDENCE", false);
            }
        }
    }

    if (apparatus) {
        report.controls[2] = geometry_control(report);
        apparatus = report.controls[2].pass;
        if (!apparatus) {
            fail("CONTROL_3", "CONTROL_INVALID");
            close_permutation("PREFIX_CLOSED_BY_PRECEDENCE",
                "NOT_RUN_APPARATUS_PRECEDENCE", false);
        }
    }
    if (apparatus) {
        report.controls[6] = admission_control(report);
        apparatus = report.controls[6].pass;
        if (!apparatus) {
            fail("CONTROL_7", "CONTROL_INVALID");
            close_permutation("PREFIX_CLOSED_BY_PRECEDENCE",
                "NOT_RUN_APPARATUS_PRECEDENCE", false);
        }
    }
    if (apparatus) {
        report.controls[4] = transaction_control_receipt(report);
        apparatus = report.controls[4].pass;
        if (!apparatus) {
            fail("CONTROL_5", "CONTROL_INVALID");
            close_permutation("PREFIX_CLOSED_BY_PRECEDENCE",
                "NOT_RUN_APPARATUS_PRECEDENCE", false);
        }
    }
    if (apparatus) {
        report.controls[5] = work_control(report);
        apparatus = report.controls[5].pass;
        if (!apparatus) {
            fail("CONTROL_6", "CONTROL_INVALID");
            close_permutation("PREFIX_CLOSED_BY_PRECEDENCE",
                "NOT_RUN_APPARATUS_PRECEDENCE", false);
        }
    }

    if (apparatus) {
        LanePair14 open512 = run_lane_pair(report.identity.policies[1]);
        report.lanes[2] = std::move(open512.canonical);
        report.lanes[3] = std::move(open512.permuted);
        apparatus = report.lanes[2].apparatus_valid
            && report.lanes[3].apparatus_valid;
        if (!apparatus) {
            fail("OPEN_512", "LANE_APPARATUS_INVALID");
            close_permutation("PREFIX_CLOSED_BY_PRECEDENCE",
                "NOT_RUN_APPARATUS_PRECEDENCE", false);
        } else {
            const bool exact = compare_permutation_lane14(permutation,
                report.lanes[2], report.lanes[3],
                "open512_result_exact");
            if (!exact) {
                apparatus = false;
                fail("CONTROL_4", "CONTROL_INVALID");
                close_permutation("CONTROL_INVALID",
                    "NOT_RUN_APPARATUS_PRECEDENCE", false);
            }
        }
    }
    if (apparatus) {
        LanePair14 tight4096 = run_lane_pair(report.identity.policies[2]);
        report.lanes[4] = std::move(tight4096.canonical);
        report.lanes[5] = std::move(tight4096.permuted);
        apparatus = report.lanes[4].apparatus_valid
            && report.lanes[5].apparatus_valid;
        if (!apparatus) {
            fail("TIGHT_4096", "LANE_APPARATUS_INVALID");
            close_permutation("PREFIX_CLOSED_BY_PRECEDENCE",
                "NOT_RUN_APPARATUS_PRECEDENCE", false);
        } else {
            const bool exact = compare_permutation_lane14(permutation,
                report.lanes[4], report.lanes[5],
                "tight4096_result_exact");
            if (!exact) {
                apparatus = false;
                fail("CONTROL_4", "CONTROL_INVALID");
                close_permutation("CONTROL_INVALID",
                    "NOT_RUN_APPARATUS_PRECEDENCE", false);
            }
        }
    }
    bool r8_trigger_r16 = false;
    if (apparatus) {
        LanePair14 tight16384 = run_lane_pair(report.identity.policies[3]);
        report.lanes[6] = std::move(tight16384.canonical);
        report.lanes[7] = std::move(tight16384.permuted);
        apparatus = report.lanes[6].apparatus_valid
            && report.lanes[7].apparatus_valid;
        if (!apparatus) {
            fail("TIGHT_16384_R8", "LANE_APPARATUS_INVALID");
            close_permutation("PREFIX_CLOSED_BY_PRECEDENCE",
                "NOT_RUN_APPARATUS_PRECEDENCE", false);
        } else {
            const bool exact = compare_permutation_lane14(permutation,
                report.lanes[6], report.lanes[7],
                "tight_r8_result_exact");
            if (!exact) {
                apparatus = false;
                fail("CONTROL_4", "CONTROL_INVALID");
                close_permutation("CONTROL_INVALID",
                    "NOT_RUN_APPARATUS_PRECEDENCE", false);
            } else {
                r8_trigger_r16 = record_r16_trigger14(permutation,
                    report.lanes[6].category);
            }
        }
    }
    if (apparatus) {
        const LaneCategory14 canonical_r8 = report.lanes[6].category;
        if (r8_trigger_r16) {
            LanePair14 r16 = run_lane_pair(report.identity.policies[4]);
            report.lanes[8] = std::move(r16.canonical);
            report.lanes[9] = std::move(r16.permuted);
            apparatus = report.lanes[8].apparatus_valid
                && report.lanes[9].apparatus_valid;
            if (!apparatus) {
                fail("TIGHT_16384_R16", "LANE_APPARATUS_INVALID");
                close_permutation("PREFIX_CLOSED_BY_PRECEDENCE",
                    "NOT_RUN_APPARATUS_PRECEDENCE", false);
            }
        } else {
            std::string cause;
            switch (canonical_r8) {
            case LaneCategory14::Supported:
                cause = "NOT_RUN_R8_SUPPORTED";
                break;
            case LaneCategory14::PhysicalGateRejected:
                cause = "NOT_RUN_R8_PHYSICAL_GATE_REJECTED";
                break;
            case LaneCategory14::QpSweepCapExhausted:
                cause = "NOT_RUN_R8_QP_CAP_EXHAUSTED";
                break;
            case LaneCategory14::ProjectionRoundCapExhausted:
            case LaneCategory14::ApparatusInvalid:
                cause = "NOT_RUN_APPARATUS_PRECEDENCE";
                apparatus = false;
                break;
            }
            report.lanes[8] = skipped_lane(
                report.identity.policies[4], cause, apparatus);
            report.lanes[9] = skipped_lane(
                report.identity.policies[4], cause, apparatus);
            lane_result_root(report.lanes[8]);
            lane_result_root(report.lanes[9]);
            if (!apparatus) {
                fail("TIGHT_16384_R8", "LANE_APPARATUS_INVALID");
                close_permutation("PREFIX_CLOSED_BY_PRECEDENCE",
                    "NOT_RUN_APPARATUS_PRECEDENCE", false);
            }
        }
    }
    if (apparatus) {
        const bool exact = compare_permutation_lane14(permutation,
            report.lanes[8], report.lanes[9], "tight_r16_result_exact");
        if (!exact) {
            apparatus = false;
            fail("CONTROL_4", "CONTROL_INVALID");
            close_permutation("CONTROL_INVALID",
                "NOT_RUN_APPARATUS_PRECEDENCE", false);
        } else {
            close_permutation("PASS", "NOT_RUN_APPARATUS_PRECEDENCE",
                true);
            apparatus = report.controls[3].pass;
            if (!apparatus) fail("CONTROL_4", "CONTROL_INVALID");
        }
    }

    if (apparatus) {
        report.controls[1] = side_control_receipt(report);
        apparatus = report.controls[1].pass;
        if (!apparatus) fail("CONTROL_2", "CONTROL_INVALID");
    }
    if (apparatus) {
        report.controls[7] = surface_control_receipt(report);
        apparatus = report.controls[7].pass;
        if (!apparatus) fail("CONTROL_8", "CONTROL_INVALID");
    }
    if (report.raw_admission.result_root.empty()) {
        seal_admission_envelope(report.raw_admission);
    }
    if (report.embedded.result_root.empty()) {
        report.embedded.verifier = verify_work14(
            report.embedded.work, report.embedded.expected_work);
        seal_embedded_parent(report.embedded);
    }
    for (LaneResult14& lane : report.lanes) {
        if (lane.result_root.empty()) {
            if (lane.trial_skips.empty()) {
                lane.trial_skips.push_back(trial_skip14(
                    lane.policy.root, 2U, "NOT_RUN_BY_PRECEDENCE"));
                lane.trial_skip_roots.push_back(
                    lane.trial_skips.back().result_root);
            }
            lane_result_root(lane);
        }
    }
    for (Control14& control : report.controls) {
        if (control.result_root.empty()) seal_control14(control);
    }
    report.finalization = evaluate_finalization(report, apparatus);
    report.finalization.verifier = verify_work14(
        report.finalization.work, report.finalization.expected_work);
    add_child(report.total_work, report.identity.work);
    add_child(report.expected_total_work, report.identity.expected_work);
    add_child(report.total_work, report.raw_admission.work);
    add_child(report.expected_total_work,
        report.raw_admission.expected_work);
    add_child(report.total_work, report.embedded.work);
    add_child(report.expected_total_work, report.embedded.expected_work);
    for (std::size_t index = 2U; index < report.lanes.size(); ++index) {
        add_child(report.total_work, report.lanes[index].work);
        add_child(report.expected_total_work,
            report.lanes[index].expected_work);
    }
    for (const Control14& control : report.controls) {
        add_child(report.total_work, control.work);
        add_child(report.expected_total_work, control.expected_work);
    }
    add_child(report.total_work, report.finalization.work);
    add_child(report.expected_total_work, report.finalization.expected_work);
    report.total_verifier = verify_work14(report.total_work,
        report.expected_total_work);
    report.finalization.total_work_exact = report.total_verifier.exact;
    report.finalization.expected_total_work_root =
        report.total_verifier.expected_root;
    report.finalization.actual_total_work_root =
        report.total_verifier.actual_root;
    if (!report.finalization.verifier.exact) {
        report.finalization.outcome = "FINALIZATION_WORK_MISMATCH";
        fail("FINALIZATION", "FINALIZATION_WORK_MISMATCH");
        apparatus = false;
    } else if (!report.total_verifier.exact) {
        report.finalization.outcome = "TOTAL_WORK_MISMATCH";
        fail("TOTAL_WORK", "TOTAL_WORK_MISMATCH");
        apparatus = false;
    } else if (!apparatus) {
        report.finalization.outcome = "NOT_RUN_BY_PRECEDENCE";
    } else {
        report.finalization.outcome = "PASS";
    }
    if (!apparatus) {
        report.finalization.selectors = {};
        report.finalization.decisive_category.clear();
        report.finalization.status = "APPARATUS_INCONCLUSIVE";
    }
    seal_finalization14(report.finalization);
    report.result_root = final14_root(report.identity,
        report.raw_admission, report.embedded, report.lanes, report.controls,
        report.finalization, report.first_failure_stage,
        report.first_failure_cause);
    emit_report14(std::cout, report);
    return apparatus && report.total_verifier.exact ? 0 : 2;
}

void emit_work14(std::ostream& output, const Work14& work) {
    constexpr std::array<std::string_view, 42> names{
        "graph_builds", "graph_candidates", "accepted_pairs",
        "density_pairs", "derivative_pairs", "matrix_products",
        "qp_sweeps", "qp_updates", "gradient_recomputations",
        "finite_difference_candidates", "independent_density_candidates",
        "plane_tests", "plane_hits", "contact_projections",
        "projection_rounds", "analytic_jv_multiply_adds",
        "topology_distance_tests", "topology_discoveries",
        "hash_derivations", "penalty_work",
        "fixture_lattice_sites_generated", "ghost_cells_tested",
        "records_canonicalized", "profile_fields_checked",
        "dynamic_records_admitted", "ghost_records_admitted",
        "raw_order_records_hashed", "face_classifications",
        "ghost_pair_distance_tests", "ghost_pairs_accepted",
        "row_degree_checks", "surface_pair_distance_tests",
        "surface_active_pairs", "surface_force_evaluations",
        "surface_reduction_adds", "lane_scalar_comparisons",
        "transaction_values_compared", "scratch_values_compared",
        "receipt_children_aggregated",
        "portable_content_fields_serialized", "binary_file_reads",
        "binary_bytes_hashed"};
    const auto values = work14_values(work);
    output << '{';
    for (std::size_t index = 0U; index < values.size(); ++index) {
        if (index != 0U) output << ',';
        output << '"' << names[index] << "\":" << values[index];
    }
    output << '}';
}

void emit_work_seal14(std::ostream& output, const Work14& actual,
    const Work14& expected, const WorkSeal14& verifier) {
    output << "\"work_exact\":"
           << (verifier.exact ? "true" : "false")
           << ",\"expected_work_root\":\"" << verifier.expected_root
           << "\",\"actual_work_root\":\"" << verifier.actual_root
           << "\",\"expected_work\":";
    emit_work14(output, expected);
    output << ",\"actual_work\":";
    emit_work14(output, actual);
}

void emit_optional_root14(std::ostream& output, std::string_view name,
    bool present, std::string_view root) {
    output << '"' << name << "_present\":"
           << (present ? "true" : "false") << ",\"" << name << "\":";
    if (present) output << '"' << root << '"';
    else output << "null";
}

void emit_profile14(std::ostream& output, const NonlocalGpuProfile& profile,
    std::string_view root) {
    output << "{\"profile_root\":\"" << root << "\",\"id\":\""
           << profile.id << "\",\"dt\":" << profile.dt
           << ",\"spacing\":" << profile.spacing
           << ",\"horizon\":" << profile.horizon
           << ",\"mass\":" << profile.mass
           << ",\"rest_density\":" << profile.rest_density
           << ",\"kernel_scale\":" << profile.kernel_scale
           << ",\"kappa\":" << profile.kappa
           << ",\"lambda\":" << profile.lambda
           << ",\"mu\":" << profile.mu
           << ",\"gamma\":" << profile.gamma
           << ",\"gravity\":[" << profile.gravity.x << ','
           << profile.gravity.y << ',' << profile.gravity.z << ']'
           << ",\"basin_extent\":[" << profile.basin_extent.x << ','
           << profile.basin_extent.y << ',' << profile.basin_extent.z << ']'
           << ",\"ghost_layers\":" << profile.ghost_layers
           << ",\"maximum_dynamic_samples\":"
           << profile.maximum_dynamic_samples
           << ",\"maximum_neighbors\":" << profile.maximum_neighbors
           << '}';
}

void emit_sample14(std::ostream& output, const NonlocalGpuSample& sample) {
    output << "{\"id\":" << sample.sample_id << ",\"reference\":["
           << sample.reference.x << ',' << sample.reference.y << ','
           << sample.reference.z << "],\"current\":[" << sample.current.x
           << ',' << sample.current.y << ',' << sample.current.z
           << "],\"velocity\":[" << sample.velocity.x << ','
           << sample.velocity.y << ',' << sample.velocity.z << "]}";
}
void emit_ghost14(std::ostream& output, const NonlocalGpuGhost& ghost) {
    output << "{\"id\":" << ghost.sample_id << ",\"position\":["
           << ghost.position.x << ',' << ghost.position.y << ','
           << ghost.position.z << "]}";
}
void emit_fixture14(std::ostream& output, const Fixture14& fixture) {
    output << "{\"name\":\"" << fixture.name
           << "\",\"profile_root\":\"" << fixture.profile_root
           << "\",\"fixture_root\":\"" << fixture.fixture_root
           << "\",\"legacy_input_root\":\"" << fixture.legacy_input_root
           << "\",\"canonical_samples\":[";
    for (std::size_t index = 0U; index < fixture.canonical_samples.size();
         ++index) {
        if (index != 0U) output << ',';
        emit_sample14(output, fixture.canonical_samples[index]);
    }
    output << "],\"permuted_samples\":[";
    for (std::size_t index = 0U; index < fixture.permuted_samples.size();
         ++index) {
        if (index != 0U) output << ',';
        emit_sample14(output, fixture.permuted_samples[index]);
    }
    output << "],\"ghosts\":[";
    for (std::size_t index = 0U; index < fixture.ghosts.size(); ++index) {
        if (index != 0U) output << ',';
        emit_ghost14(output, fixture.ghosts[index]);
    }
    output << "]}";
}

void emit_face_counts(std::ostream& output, const FaceCounts& counts) {
    output << "{\"dynamic_dynamic_pairs_including_self\":"
           << counts.dynamic_dynamic << ",\"dynamic_ghost_pairs\":"
           << counts.dynamic_ghost << ",\"total_directed_pairs\":"
           << counts.dynamic_dynamic + counts.dynamic_ghost
           << ",\"lateral_unique_dynamic_ghost_pairs\":"
           << counts.lateral_unique << ",\"memberships\":[";
    for (std::size_t index = 0U; index < counts.membership.size(); ++index) {
        if (index != 0U) output << ',';
        output << counts.membership[index];
    }
    output << "],\"maximum_owner_row_degree\":" << counts.maximum_row
           << '}';
}

void emit_geometry14(std::ostream& output, const Geometry14& geometry) {
    output << "{\"root\":\"" << geometry.root << '"';
    if (!geometry.graph_root.empty()) {
        output << ",\"graph_root\":\"" << geometry.graph_root << '"';
    }
    output << ",\"counts\":";
    emit_face_counts(output, geometry.counts);
    output << ",\"actual_work_root\":\""
           << work14_root(geometry.work)
           << "\",\"expected_work\":";
    emit_work14(output, geometry.expected_work);
    output << ",\"actual_work\":";
    emit_work14(output, geometry.work);
    output << '}';
}

void emit_control14(std::ostream& output, const Control14& control) {
    output << "{\"index\":" << control.index << ",\"name\":\""
           << control.name << "\",\"typed_outcome\":\""
           << control.outcome << "\",\"pass\":"
           << (control.pass ? "true" : "false")
           << ",\"evidence_roots\":[";
    for (std::size_t index = 0U; index < control.evidence_roots.size();
         ++index) {
        if (index != 0U) output << ',';
        output << '"' << control.evidence_roots[index] << '"';
    }
    output << "],\"scalars\":[";
    for (std::size_t index = 0U; index < control.scalars.size(); ++index) {
        if (index != 0U) output << ',';
        const Scalar14& scalar = control.scalars[index];
        output << "{\"name\":\"" << scalar.name << "\",\"type\":"
               << static_cast<unsigned int>(scalar.type) << ",\"value\":";
        if (scalar.type == 0U) output << (scalar.boolean ? "true" : "false");
        else if (scalar.type == 1U) output << scalar.u32;
        else if (scalar.type == 2U) output << scalar.u64;
        else output << static_cast<double>(scalar.f64);
        output << '}';
    }
    output << "],\"result_root\":\"" << control.result_root << "\",";
    emit_work_seal14(output, control.work, control.expected_work,
        control.verifier);
    output << '}';
}

void emit_u64_array14(std::ostream& output,
    const std::array<std::uint64_t, 6>& values) {
    output << '[';
    for (std::size_t index = 0U; index < values.size(); ++index) {
        if (index != 0U) output << ',';
        output << values[index];
    }
    output << ']';
}

void emit_root_array14(std::ostream& output,
    const std::vector<std::string>& roots) {
    output << '[';
    for (std::size_t index = 0U; index < roots.size(); ++index) {
        if (index != 0U) output << ',';
        output << '"' << roots[index] << '"';
    }
    output << ']';
}

void emit_round_extra14(std::ostream& output, const RoundExtra14& extra) {
    output << "{\"lane_root\":\"" << extra.lane_root
           << "\",\"trial_index\":" << extra.trial_index
           << ",\"projection_round_index\":" << extra.round_index
           << ",\"inherited_round_root\":\""
           << extra.inherited_round_root
           << "\",\"finite_capacity_valid\":"
           << (extra.finite_capacity_valid ? "true" : "false")
           << ",\"assembly_valid\":"
           << (extra.assembly_valid ? "true" : "false")
           << ",\"jv_valid\":" << (extra.jv_ok ? "true" : "false")
           << ",\"symmetry_valid\":"
           << (extra.symmetry_valid ? "true" : "false")
           << ",\"qp_valid\":" << (extra.qp_valid ? "true" : "false")
           << ",\"qp_converged\":"
           << (extra.qp_converged ? "true" : "false")
           << ",\"contact_valid\":"
           << (extra.contact_valid ? "true" : "false")
           << ",\"candidate_density_valid\":"
           << (extra.candidate_density_valid ? "true" : "false")
           << ",\"independent_density_valid\":"
           << (extra.independent_density_valid ? "true" : "false")
           << ",\"density_correspondence_valid\":"
           << (extra.density_correspondence_valid ? "true" : "false")
           << ",\"pressure_balance_valid\":"
           << (extra.balance_ok ? "true" : "false")
           << ",\"inset_valid\":"
           << (extra.inset_ok ? "true" : "false")
           << ",\"round_apparatus_ok\":"
           << (extra.round_apparatus_ok ? "true" : "false")
           << ",\"round_closed\":"
           << (extra.round_closed ? "true" : "false")
           << ",\"jv_relative_l2\":"
           << static_cast<double>(extra.jv_relative_l2)
           << ",\"symmetry_relative_l2\":"
           << static_cast<double>(extra.symmetry_relative_l2)
           << ",\"pressure_balance\":"
           << static_cast<double>(extra.pressure_balance)
           << ",\"density_correspondence\":"
           << static_cast<double>(extra.density_correspondence)
           << ",\"maximum_positive_strain\":"
           << static_cast<double>(extra.maximum_positive_strain)
           << ",\"rms_positive_strain\":"
           << static_cast<double>(extra.rms_positive_strain)
           << ",\"penetration_m\":"
           << static_cast<double>(extra.penetration_m)
           << ",\"positive_multiplier_count\":"
           << extra.positive_multiplier_count << ",\"faces\":";
    emit_face_counts(output, extra.pre_faces);
    output << ",\"first_hit_faces\":";
    emit_u64_array14(output, extra.first_hit_faces);
    output << ",\"clamp_faces\":";
    emit_u64_array14(output, extra.clamp_faces);
    output << ",\"root\":\"" << extra.root << "\"}";
}

void emit_partial_round14(std::ostream& output, const RoundSummary& round) {
    output << "{\"assembly_valid\":"
           << (round.assembly_valid ? "true" : "false")
           << ",\"solve_valid\":"
           << (round.solve_valid ? "true" : "false")
           << ",\"converged\":" << (round.converged ? "true" : "false")
           << ",\"stale\":" << (round.stale ? "true" : "false")
           << ",\"expected_hash_derivations\":"
           << round.expected_hash_derivations
           << ",\"hash_accounting_exact\":"
           << (round.hash_accounting_exact ? "true" : "false")
           << ",\"sweeps\":" << round.sweeps
           << ",\"updates\":" << round.updates
           << ",\"positive\":" << round.positive
           << ",\"jacobian_relative_l2\":"
           << static_cast<double>(round.jacobian)
           << ",\"symmetry_relative_l2\":"
           << static_cast<double>(round.symmetry)
           << ",\"primal\":" << static_cast<double>(round.primal)
           << ",\"projected_kkt\":" << static_cast<double>(round.kkt)
           << ",\"complementarity_j\":"
           << static_cast<double>(round.complementarity)
           << ",\"pressure_balance\":"
           << static_cast<double>(round.pressure_balance)
           << ",\"maximum_positive_strain\":"
           << static_cast<double>(round.maximum_strain)
           << ",\"rms_positive_strain\":"
           << static_cast<double>(round.rms_strain)
           << ",\"penetration_m\":" << static_cast<double>(round.penetration)
           << ",\"density_correspondence\":"
           << static_cast<double>(round.density_correspondence)
           << ",\"assembly_input_root\":\"" << round.assembly_input_root
           << "\",\"graph_root\":\"" << round.graph_root
           << "\",\"density_root\":\"" << round.density_root
           << "\",\"jacobian_root\":\"" << round.jacobian_root
           << "\",\"matrix_root\":\"" << round.matrix_root
           << "\",\"post_graph_root\":\"" << round.post_graph_root
           << "\",\"post_density_root\":\"" << round.post_density_root
           << "\",\"independent_density_root\":\""
           << round.independent_density_root
           << "\",\"multiplier_root\":\"" << round.multiplier_root
           << "\",\"contact_root\":\"" << round.contact_root
           << "\",\"state_root\":\"" << round.state_root
           << "\",\"work_root\":\"" << work_root(round.work)
           << "\",\"result_root\":\"" << round.result_root
           << "\",\"work\":";
    emit_work(output, round.work);
    output << '}';
}

void emit_step14(std::ostream& output, const Step14& step) {
    output << "{\"typed_route\":\"" << step.typed_route
           << "\",\"maximum_qp_sweeps\":" << step.maximum_qp_sweeps
           << ",\"attempted_step_present\":"
           << (step.attempted_step_sealed ? "true" : "false")
           << ",\"step\":";
    if (step.attempted_step_sealed) emit_step(output, step.step);
    else output << "null";
    output << ",\"partial_rounds\":[";
    if (!step.attempted_step_sealed) {
        for (std::size_t index = 0U; index < step.step.rounds.size(); ++index) {
            if (index != 0U) output << ',';
            emit_partial_round14(output, step.step.rounds[index]);
        }
    }
    output << "],\"expected_parent_work\":";
    emit_work(output, step.expected_parent_work);
    output << ",\"actual_parent_work\":";
    emit_work(output, step.step.work);
    output << ",\"expected_extra_work\":";
    emit_work14(output, step.expected_extra_work);
    output << ",\"actual_extra_work\":";
    emit_work14(output, step.extra_work);
    output << ",\"round_extra\":[";
    for (std::size_t index = 0U; index < step.extra_rounds.size(); ++index) {
        if (index != 0U) output << ',';
        emit_round_extra14(output, step.extra_rounds[index]);
    }
    output << "]}";
}

void emit_trial_observables14(std::ostream& output,
    const TrialObservables14& observables) {
    output << "{\"lane_root\":\"" << observables.lane_root
           << "\",\"trial_index\":" << observables.trial_index
           << ",\"attempted_step_root\":\""
           << observables.attempted_step_root
           << "\",\"position_rmse_m\":"
           << static_cast<double>(observables.position_rms)
           << ",\"position_maximum_m\":"
           << static_cast<double>(observables.position_maximum)
           << ",\"velocity_rms_mps\":"
           << static_cast<double>(observables.velocity_rms)
           << ",\"maximum_speed_mps\":"
           << static_cast<double>(observables.maximum_speed)
           << ",\"energy_positive_excess\":"
           << static_cast<double>(observables.energy_excess)
           << ",\"momentum_residual\":"
           << static_cast<double>(observables.momentum_residual)
           << ",\"dynamic_count\":" << observables.dynamic_count
           << ",\"stable_id_count\":" << observables.stable_id_count
           << ",\"component_count\":" << observables.components
           << ",\"satellite_count\":" << observables.satellite_count
           << ",\"positive_multiplier_count\":"
           << observables.positive_multiplier_count
           << ",\"total_mass\":"
           << static_cast<double>(observables.total_mass)
           << ",\"lower_contact\":"
           << (observables.lower_contact ? "true" : "false")
           << ",\"trial_invariants_ok\":"
           << (observables.trial_invariants_ok ? "true" : "false")
           << ",\"predictor_first_hit_faces\":";
    emit_u64_array14(output, observables.predictor_first_hit_faces);
    output << ",\"predictor_clamp_faces\":";
    emit_u64_array14(output, observables.predictor_clamp_faces);
    output << ",\"round_extra_roots\":";
    emit_root_array14(output, observables.round_extra_roots);
    output << ",\"state_root\":\"" << observables.state_root
           << "\",\"velocity_root\":\"" << observables.velocity_root
           << "\",\"root\":\"" << observables.root << "\"}";
}

void emit_trial_skip14(std::ostream& output, const TrialSkip14& skip) {
    output << "{\"lane_root\":\"" << skip.lane_root
           << "\",\"trial_index\":" << skip.trial_index
           << ",\"cause\":\"" << skip.cause
           << "\",\"result_root\":\"" << skip.result_root << "\",";
    emit_work_seal14(output, skip.work, skip.expected_work, skip.verifier);
    output << '}';
}

void emit_lane14(std::ostream& output, const LaneResult14& lane) {
    output << "{\"name\":\"" << lane.policy.name
           << "\",\"lane_root\":\"" << lane.policy.root
           << "\",\"fixture_root\":\""
           << (lane.policy.fixture == nullptr
                   ? std::string_view{}
                   : std::string_view(lane.policy.fixture->fixture_root))
           << "\",\"profile_root\":\""
           << (lane.policy.fixture == nullptr
                   ? std::string_view{}
                   : std::string_view(lane.policy.fixture->profile_root))
           << "\",\"legacy_input_root\":\""
           << (lane.policy.fixture == nullptr
                   ? std::string_view{}
                   : std::string_view(lane.policy.fixture->legacy_input_root))
           << "\",\"profile\":";
    if (lane.policy.fixture == nullptr) {
        output << "null";
    } else {
        emit_profile14(output, lane.policy.fixture->profile,
            lane.policy.fixture->profile_root);
    }
    output << ",\"maximum_steps\":" << lane.policy.maximum_steps
           << ",\"projection_cap\":" << lane.policy.projection_cap
           << ",\"qp_sweep_cap\":" << lane.policy.qp_cap
           << ",\"tolerances\":{\"jv_relative_l2\":2e-7,"
           << "\"symmetry_relative_l2\":2e-12,"
           << "\"primal\":1e-8,\"projected_kkt\":1e-8,"
           << "\"complementarity_j\":1e-10,\"pressure_balance\":1e-8,"
           << "\"maximum_positive_strain\":1e-3,"
           << "\"rms_positive_strain\":2.5e-4,\"penetration_m\":0,"
           << "\"step_balance\":1e-8,\"position_rmse_m\":0.0025,"
           << "\"position_maximum_m\":0.005,"
           << "\"velocity_rms_mps\":0.05,\"maximum_speed_mps\":0.10,"
           << "\"energy_positive_excess\":0.01,"
           << "\"momentum_residual\":0.01,\"components\":1}"
           << ",\"typed_route\":\"" << lane.typed_route
           << "\",\"route_category\":\"" << lane.route_category
           << "\",\"failure_cause\":\"" << lane.failure_cause
           << "\",\"trial_2_execution\":\"" << lane.trial_2_execution
           << "\",\"attempted_trial_count\":" << lane.attempted_trials
           << ",\"committed_trial_count\":" << lane.committed_trials
           << ",\"apparatus_valid\":"
           << (lane.apparatus_valid ? "true" : "false")
           << ",\"both_steps_supported\":"
           << (lane.both_steps_supported ? "true" : "false")
           << ",\"physical_gate_rejected\":"
           << (lane.physical_gate_rejected ? "true" : "false")
           << ",\"qp_sweep_cap_exhausted\":"
           << (lane.qp_sweep_cap_exhausted ? "true" : "false")
           << ",\"projection_round_cap_exhausted\":"
           << (lane.projection_round_cap_exhausted ? "true" : "false")
           << ",\"trial_1_committed\":"
           << (lane.trial_1_committed ? "true" : "false")
           << ",\"trial_2_attempted\":"
           << (lane.trial_2_attempted ? "true" : "false")
           << ",\"trial_2_committed\":"
           << (lane.trial_2_committed ? "true" : "false")
           << ",\"projection_cap_saturated\":"
           << (lane.projection_cap_saturated ? "true" : "false")
           << ",\"accepted_state_root\":\"" << lane.accepted_state_root
           << "\",\"accepted_velocity_root\":\""
           << lane.accepted_velocity_root << "\",\"failing_state_root\":\""
           << lane.failing_state_root << "\",\"failing_velocity_root\":\""
           << lane.failing_velocity_root << "\",\"round_roots\":";
    emit_root_array14(output, lane.round_roots);
    output << ",\"round_extra_roots\":";
    emit_root_array14(output, lane.round_extra_roots);
    output << ",\"attempted_step_roots\":";
    emit_root_array14(output, lane.attempted_step_roots);
    output << ",\"rejected_step_roots\":";
    emit_root_array14(output, lane.rejected_step_roots);
    output << ",\"trial_observable_roots\":";
    emit_root_array14(output, lane.trial_observable_roots);
    output << ",\"trial_skip_roots\":";
    emit_root_array14(output, lane.trial_skip_roots);
    output << ",\"reached_children\":[";
    for (std::size_t index = 0U; index < lane.reached_children.size();
         ++index) {
        if (index != 0U) output << ',';
        const ReachedChild14& child = lane.reached_children[index];
        output << "{\"trial_index\":" << child.trial_index
               << ",\"round_index\":" << child.round_index
               << ",\"stage\":\"" << child.stage
               << "\",\"root\":\"" << child.root << "\"}";
    }
    output << "],\"trials\":[";
    for (std::size_t index = 0U; index < lane.trials.size(); ++index) {
        if (index != 0U) output << ',';
        emit_step14(output, lane.trials[index]);
    }
    output << "],\"trial_observables\":[";
    for (std::size_t index = 0U; index < lane.trial_observables.size();
         ++index) {
        if (index != 0U) output << ',';
        emit_trial_observables14(output, lane.trial_observables[index]);
    }
    output << "],\"trial_skips\":[";
    for (std::size_t index = 0U; index < lane.trial_skips.size(); ++index) {
        if (index != 0U) output << ',';
        emit_trial_skip14(output, lane.trial_skips[index]);
    }
    output << "],\"trajectory\":";
    if (!lane.inherited_trajectory_json.empty()) {
        output << lane.inherited_trajectory_json;
    } else {
        emit_trajectory(output, lane.trajectory);
    }
    output << ",\"trajectory_root\":\"" << lane.trajectory_root
           << "\",\"result_root\":\"" << lane.result_root << "\",";
    emit_work_seal14(output, lane.work, lane.expected_work, lane.verifier);
    output << '}';
}

void emit_surface14(std::ostream& output, const Surface14& surface) {
    output << "{\"variant\":\"" << surface_variant_name(surface.variant)
           << "\",\"active_pairs\":" << surface.active_pairs
           << ",\"acceleration_rms\":"
           << static_cast<double>(surface.acceleration_rms)
           << ",\"acceleration_maximum\":"
           << static_cast<double>(surface.acceleration_maximum)
           << ",\"net_force_residual\":"
           << static_cast<double>(surface.net_residual)
           << ",\"two_step_velocity_scale\":"
           << static_cast<double>(surface.two_step_velocity_scale)
           << ",\"delta_mean_bound\":"
           << static_cast<double>(surface.delta_mean_bound)
           << ",\"force_root\":\"" << surface.root
           << "\",\"expected_work\":";
    emit_work14(output, surface.expected_work);
    output << ",\"actual_work\":";
    emit_work14(output, surface.work);
    output << '}';
}

void emit_admission_receipt14(std::ostream& output,
    const AdmissionReceipt14& receipt) {
    output << "{\"name\":\"" << receipt.name
           << "\",\"typed_outcome\":\"" << admission_name(receipt.outcome)
           << "\",\"result_root\":\"" << receipt.result_root << "\",";
    emit_work_seal14(output, receipt.work, receipt.expected_work,
        receipt.verifier);
    output << '}';
}

void emit_admission_envelope14(std::ostream& output,
    const AdmissionEnvelope14& admission) {
    output << "{\"pass\":" << (admission.pass ? "true" : "false")
           << ",\"typed_outcome\":\"" << admission.outcome
           << "\",\"children\":[";
    if (admission.outcome != "NOT_RUN_BY_PRECEDENCE") {
        for (std::size_t index = 0U; index < admission.children.size();
             ++index) {
            if (index != 0U) output << ',';
            emit_admission_receipt14(output, admission.children[index]);
        }
    }
    output << "],\"result_root\":\"" << admission.result_root << "\",";
    emit_work_seal14(output, admission.work, admission.expected_work,
        admission.verifier);
    output << '}';
}

void emit_admission_mutation14(std::ostream& output,
    const AdmissionMutation14& mutation) {
    output << "{\"mutation_index\":" << mutation.index
           << ",\"mutation_name\":\"" << mutation.name
           << "\",\"maximum_total_records\":"
           << mutation.maximum_total_records
           << ",\"expected_ghost_count\":"
           << mutation.expected_ghost_count
           << ",\"expected_outcome\":\""
           << admission_name(mutation.expected_outcome)
           << "\",\"observed_outcome\":\""
           << admission_name(mutation.observed_outcome)
           << "\",\"post_rejection_state_root\":\""
           << mutation.rejected_state_root << "\",\"payload\":{";
    switch (mutation.index) {
    case 1U:
        output << "\"field_name\":\"" << mutation.field_name
               << "\",\"value_bits\":" << mutation.u64_payload[0];
        break;
    case 2U:
        output << "\"original_ghost_count\":" << mutation.u64_payload[0]
               << ",\"removed_index\":" << mutation.u64_payload[1];
        break;
    case 3U:
        output << "\"field_name\":\"" << mutation.field_name
               << "\",\"value\":" << mutation.u32_payload[0];
        break;
    case 4U:
        output << "\"dynamic_index\":" << mutation.u64_payload[0]
               << ",\"field_name\":\"" << mutation.field_name
               << "\",\"value_bits\":" << mutation.u64_payload[1];
        break;
    case 5U:
        output << "\"destination_dynamic_index\":"
               << mutation.u64_payload[0]
               << ",\"source_dynamic_index\":" << mutation.u64_payload[1];
        break;
    case 6U:
        output << "\"dynamic_index\":" << mutation.u64_payload[0]
               << ",\"field_name\":\"" << mutation.field_name
               << "\",\"value_bits\":" << mutation.u64_payload[1];
        break;
    case 7U:
        output << "\"original_dynamic_count\":" << mutation.u64_payload[0]
               << ",\"resulting_dynamic_count\":"
               << mutation.u64_payload[1]
               << ",\"append_source_dynamic_index\":"
               << mutation.u64_payload[2];
        break;
    case 8U:
        output << "\"retained_dynamic_index\":" << mutation.u64_payload[0]
               << ",\"generated_ghost_count\":" << mutation.u64_payload[1]
               << ",\"ghost_id_start\":" << mutation.u32_payload[0]
               << ",\"ghost_id_stride\":" << mutation.u32_payload[1]
               << ",\"position_bits\":[" << mutation.u64_payload[2] << ','
               << mutation.u64_payload[3] << ',' << mutation.u64_payload[4]
               << ']';
        break;
    case 9U:
        output << "\"dynamic_count\":" << mutation.u64_payload[0]
               << ",\"id_start\":" << mutation.u32_payload[0]
               << ",\"id_stride\":" << mutation.u32_payload[1]
               << ",\"position_bits\":[" << mutation.u64_payload[1] << ','
               << mutation.u64_payload[2] << ',' << mutation.u64_payload[3]
               << "],\"velocity_bits\":[" << mutation.u64_payload[4] << ','
               << mutation.u64_payload[5] << ',' << mutation.u64_payload[6]
               << "],\"ghost_count\":" << mutation.u64_payload[7];
        break;
    default:
        throw std::logic_error("NCGP14 admission mutation index invalid");
    }
    output << "},\"mutation_root\":\"" << mutation.root << "\"}";
}

void emit_scratch14(std::ostream& output, const Scratch14& scratch,
    std::string_view root) {
    output << "{\"root\":\"" << root << "\",\"owners\":[";
    for (std::size_t index = 0U; index < scratch.owner_ids.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"id\":" << scratch.owner_ids[index]
               << ",\"row_begin\":" << scratch.row_begin[index]
               << ",\"row_end\":" << scratch.row_end[index] << '}';
    }
    output << "],\"entries\":[";
    for (std::size_t index = 0U; index < scratch.entries.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"kind\":"
               << static_cast<unsigned int>(scratch.entries[index].first)
               << ",\"id\":" << scratch.entries[index].second << '}';
    }
    output << "],\"qp\":[";
    for (std::size_t index = 0U; index < scratch.lambda.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"id\":" << scratch.owner_ids[index]
               << ",\"lambda\":" << static_cast<double>(scratch.lambda[index])
               << ",\"gradient\":"
               << static_cast<double>(scratch.gradient[index]) << '}';
    }
    output << "],\"contact\":[";
    for (std::size_t index = 0U; index < scratch.clamp_mask.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"id\":" << scratch.owner_ids[index]
               << ",\"first_hit_mask\":"
               << static_cast<unsigned int>(scratch.first_mask[index])
               << ",\"clamp_mask\":"
               << static_cast<unsigned int>(scratch.clamp_mask[index])
               << ",\"correction\":["
               << static_cast<double>(scratch.correction[index].x) << ','
               << static_cast<double>(scratch.correction[index].y) << ','
               << static_cast<double>(scratch.correction[index].z)
               << "],\"impulse\":["
               << static_cast<double>(scratch.impulse[index].x) << ','
               << static_cast<double>(scratch.impulse[index].y) << ','
               << static_cast<double>(scratch.impulse[index].z) << "]}";
    }
    output << "]}";
}

void emit_transaction_subroute14(std::ostream& output,
    const TransactionSubroute14& subroute) {
    const TransactionRejection14& rejection = subroute.rejection;
    output << "{\"pass\":" << (subroute.pass ? "true" : "false")
           << ",\"pre_scratch\":";
    emit_scratch14(output, subroute.pre_scratch, subroute.pre_scratch_root);
    output << ",\"post_scratch\":";
    emit_scratch14(output, subroute.post_scratch, subroute.post_scratch_root);
    output << ",\"rejected_state_root\":\""
           << subroute.rejected_state_root
           << "\",\"private_step\":";
    emit_step14(output, subroute.private_step);
    output << ",\"trial_observables\":[";
    for (std::size_t index = 0U; index < subroute.observables.size();
         ++index) {
        if (index != 0U) output << ',';
        emit_trial_observables14(output, subroute.observables[index]);
    }
    output << ']';
    output << ",\"trial_skip\":";
    emit_trial_skip14(output, subroute.trial_skip);
    output << ",\"rejection_receipt\":{\"base_lane_root\":\""
           << rejection.base_lane_root << "\",\"subroute\":\""
           << rejection.subroute << "\",\"outcome\":\""
           << rejection.outcome << "\",\"attempted_trial_count\":"
           << rejection.attempted_trials
           << ",\"committed_trial_count\":"
           << rejection.committed_trials << ",\"child_roots\":";
    emit_root_array14(output, rejection.child_roots);
    output << ",\"post_rejection_state_root\":\""
           << rejection.rejected_state_root
           << "\",\"pre_reject_scratch_root\":\""
           << rejection.pre_scratch_root
           << "\",\"post_reject_scratch_root\":\""
           << rejection.post_scratch_root
           << "\",\"trial_2_skip_root\":\""
           << rejection.trial_skip_root << "\",\"hook_fired\":"
           << (rejection.hook_fired ? "true" : "false")
           << ",\"result_root\":\"" << rejection.result_root << "\",";
    emit_work_seal14(output, rejection.work, rejection.expected_work,
        rejection.verifier);
    output << '}';
    output << '}';
}

void emit_geometry_mutation14(std::ostream& output,
    const GeometryMutation14& mutation) {
    output << "{\"root\":\"" << mutation.root
           << "\",\"work_root\":\"" << mutation.work_root
           << "\",\"records\":[";
    for (std::size_t index = 0U; index < mutation.records.size(); ++index) {
        if (index != 0U) output << ',';
        const GeometryMutationRecord14& record = mutation.records[index];
        output << "{\"id\":" << record.id << ",\"cell\":["
               << record.cell[0] << ',' << record.cell[1] << ','
               << record.cell[2] << "],\"old_position\":["
               << record.old_position.x << ',' << record.old_position.y
               << ',' << record.old_position.z << "],\"new_position\":["
               << record.new_position.x << ',' << record.new_position.y
               << ',' << record.new_position.z << "]}";
    }
    output << "],\"expected_work\":";
    emit_work14(output, mutation.expected_work);
    output << ",\"actual_work\":";
    emit_work14(output, mutation.work);
    output << '}';
}

void emit_trial_observable_array14(std::ostream& output,
    const std::vector<TrialObservables14>& observables) {
    output << '[';
    for (std::size_t index = 0U; index < observables.size(); ++index) {
        if (index != 0U) output << ',';
        emit_trial_observables14(output, observables[index]);
    }
    output << ']';
}

void emit_report14(std::ostream& output, const Report14& report) {
    output << std::setprecision(17)
           << "{\"schema\":\"" << kSchema << "\",\"status\":\""
           << report.finalization.status << "\",\"contract_root\":\""
           << NCGP14_CONTRACT_ROOT << "\",\"source_commit\":\""
           << NCGP14_SOURCE_COMMIT << "\",\"source_tree\":\""
           << NCGP14_SOURCE_TREE << "\",\"source_root\":\""
           << NCGP14_SOURCE_ROOT << "\",\"binary_root\":\""
           << report.identity.binary.root << "\",\"compiler_family\":\""
           << NCGP14_COMPILER_FAMILY << "\",\"compiler_version\":\""
           << NCGP14_COMPILER_VERSION << "\",\"compiler_flags\":\""
           << NCGP14_COMPILER_FLAGS << "\",\"invocation\":\""
           << kInvocation << "\",\"first_failure_stage\":\""
           << report.first_failure_stage
           << "\",\"first_failure_cause\":\""
           << report.first_failure_cause << "\",\"identity_valid\":"
           << (report.identity.pass ? "true" : "false")
           << ",\"admission_valid\":"
           << (report.raw_admission.pass ? "true" : "false")
           << ",\"embedded_valid\":"
           << (report.embedded.pass ? "true" : "false")
           << ",\"identity\":{\"typed_outcome\":\""
           << report.identity.outcome << "\",\"binary_bytes\":"
           << report.identity.binary.bytes << ",\"profile_roots\":[";
    for (std::size_t index = 0U;
         index < report.identity.profile_roots.size(); ++index) {
        if (index != 0U) output << ',';
        output << '"' << report.identity.profile_roots[index] << '"';
    }
    output << "],\"fixture_roots\":[\""
           << report.identity.open128.fixture_root << "\",\""
           << report.identity.open512.fixture_root << "\",\""
           << report.identity.tight128.fixture_root
           << "\"],\"lane_roots\":[";
    for (std::size_t index = 0U; index < report.identity.policies.size();
         ++index) {
        if (index != 0U) output << ',';
        output << '"' << report.identity.policies[index].root << '"';
    }
    output << "],\"result_root\":\"" << report.identity.result_root
           << "\",";
    emit_work_seal14(output, report.identity.work,
        report.identity.expected_work, report.identity.verifier);
    output << "},\"admission\":";
    emit_admission_envelope14(output, report.raw_admission);
    output << ",\"detached_parent\":{\"expected_commit\":\""
           << kParentCommit << "\",\"expected_tree\":\"" << kParentTree
           << "\",\"expected_source_file_root\":\"" << kParentSourceFile
           << "\",\"expected_source_root\":\"" << kParentSourceAggregate
           << "\",\"expected_contract_root\":\"" << kParentContract
           << "\",\"expected_binary_root\":\"" << kParentBinary
           << "\",\"expected_stdout_root\":\"" << kParentStdout
           << "\",\"expected_result_root\":\"" << kParentResult
           << "\",";
    emit_optional_root14(output, "observed_raw_stdout_root",
        report.embedded.raw_stdout_root_present,
        report.embedded.raw_stdout_root);
    output << ',';
    emit_optional_root14(output, "recomputed_raw_result_root",
        report.embedded.recomputed_raw_result_root_present,
        report.embedded.recomputed_raw_result_root);
    output << ',';
    emit_optional_root14(output, "observed_normalized_stdout_root",
        report.embedded.normalized_stdout_root_present,
        report.embedded.normalized_stdout_root);
    output << ',';
    emit_optional_root14(output, "recomputed_normalized_result_root",
        report.embedded.normalized_result_root_present,
        report.embedded.recomputed_normalized_result_root);
    output << "},\"embedded_parent\":{\"pass\":"
           << (report.embedded.pass ? "true" : "false")
           << ",\"typed_outcome\":\"" << report.embedded.outcome
           << "\",\"exit_code\":" << report.embedded.exit_code
           << ",\"validation_stage\":"
           << static_cast<unsigned int>(report.embedded.validation_stage)
           << ',';
    emit_optional_root14(output, "raw_stdout_root",
        report.embedded.raw_stdout_root_present,
        report.embedded.raw_stdout_root);
    output << ',';
    emit_optional_root14(output, "raw_result_root",
        report.embedded.observed_raw_result_root_present,
        report.embedded.raw_result_root);
    output << ',';
    emit_optional_root14(output, "recomputed_raw_result_root",
        report.embedded.recomputed_raw_result_root_present,
        report.embedded.recomputed_raw_result_root);
    output << ',';
    emit_optional_root14(output, "normalized_stdout_root",
        report.embedded.normalized_stdout_root_present,
        report.embedded.normalized_stdout_root);
    output << ',';
    emit_optional_root14(output, "normalized_result_root",
        report.embedded.normalized_result_root_present,
        report.embedded.recomputed_normalized_result_root);
    output << ',';
    emit_optional_root14(output, "inherited_parent_result_root",
        report.embedded.inherited_parent_result_root_present,
        report.embedded.inherited_parent_result_root);
    output << ",\"result_root\":\"" << report.embedded.result_root
           << "\",";
    emit_work_seal14(output, report.embedded.work,
        report.embedded.expected_work, report.embedded.verifier);
    output << ",\"raw_json\":";
    if (report.embedded.validation_stage < 2U) output << "null";
    else output << report.embedded.raw_stdout;
    output << "},\"profiles\":[";
    emit_profile14(output, report.identity.open_profile,
        report.identity.profile_roots[0]);
    output << ',';
    emit_profile14(output, report.identity.tight_profile,
        report.identity.profile_roots[1]);
    output << ',';
    emit_profile14(output, report.identity.open_census_profile,
        report.identity.profile_roots[2]);
    output << ',';
    emit_profile14(output, report.identity.tight_census_profile,
        report.identity.profile_roots[3]);
    output << "],\"fixtures\":[";
    emit_fixture14(output, report.identity.open128);
    output << ',';
    emit_fixture14(output, report.identity.open512);
    output << ',';
    emit_fixture14(output, report.identity.tight128);
    output << "],\"lanes\":[";
    for (std::size_t index = 0U; index < report.lanes.size(); ++index) {
        if (index != 0U) output << ',';
        emit_lane14(output, report.lanes[index]);
    }
    output << "],\"geometry\":";
    if (report.controls[2].outcome == "NOT_RUN_BY_PRECEDENCE") {
        output << "null";
    } else {
        output << "{\"candidate\":";
        emit_geometry14(output, report.geometry_candidate);
        output << ",\"oracle\":";
        emit_geometry14(output, report.geometry_oracle);
        output << ",\"mutated_top\":";
        emit_geometry14(output, report.geometry_mutated);
        output << ",\"mutation\":";
        emit_geometry_mutation14(output, report.geometry_mutation);
        output << '}';
    }
    output << ",\"surface\":";
    if (report.controls[7].outcome == "NOT_RUN_BY_PRECEDENCE") {
        output << "null";
    } else {
        output << "{\"candidate\":[";
        for (std::size_t index = 0U; index < 3U; ++index) {
            if (index != 0U) output << ',';
            emit_surface14(output, report.surface.candidate[index]);
        }
        output << "],\"oracle\":[";
        for (std::size_t index = 0U; index < 3U; ++index) {
            if (index != 0U) output << ',';
            emit_surface14(output, report.surface.oracle[index]);
        }
        output << "],\"zero_gamma\":[";
        for (std::size_t index = 0U; index < 3U; ++index) {
            if (index != 0U) output << ',';
            emit_surface14(output, report.surface.zero[index]);
        }
        output << "],\"wrong_branch\":";
        emit_surface14(output, report.surface.wrong_branch);
        output << ",\"wrong_sign\":";
        emit_surface14(output, report.surface.wrong_sign);
        output << ",\"half_force\":";
        emit_surface14(output, report.surface.half_force);
        output << '}';
    }
    output << ",\"transaction\":";
    if (report.controls[4].outcome == "NOT_RUN_BY_PRECEDENCE") {
        output << "null";
    } else {
    output << "{\"pass\":"
           << (report.transaction.pass ? "true" : "false")
           << ",\"prior_state_root\":\"" << report.transaction.prior_root
           << "\",\"qp_rejected_state_root\":\""
           << report.transaction.qp_rejected_root
           << "\",\"forced_rejected_state_root\":\""
           << report.transaction.forced_rejected_root
           << "\",\"canonical_empty_scratch\":";
    emit_scratch14(output, report.transaction.canonical_empty,
        report.transaction.empty_scratch_root);
    output << ",\"qp_cap_1\":";
    emit_transaction_subroute14(output, report.transaction.qp);
    output << ",\"forced_after_private_observable_seal\":";
    emit_transaction_subroute14(output, report.transaction.forced);
    output << ",\"expected_work\":";
    emit_work14(output, report.transaction.expected_work);
    output << ",\"actual_work\":";
    emit_work14(output, report.transaction.work);
    output << '}';
    }
    output << ",\"admission_controls\":";
    if (report.controls[6].outcome == "NOT_RUN_BY_PRECEDENCE") {
        output << "null";
    } else {
    output << "{\"pass\":"
           << (report.admission.pass ? "true" : "false")
           << ",\"state_roots\":";
    emit_root_array14(output, report.admission.state_roots);
    output << ",\"mutations\":[";
    for (std::size_t index = 0U; index < report.admission.mutations.size();
         ++index) {
        if (index != 0U) output << ',';
        emit_admission_mutation14(output, report.admission.mutations[index]);
    }
    output << "],\"typed_outcomes\":[";
    for (std::size_t index = 0U; index < report.admission.outcomes.size();
         ++index) {
        if (index != 0U) output << ',';
        output << '"' << report.admission.outcomes[index] << '"';
    }
    output << "],\"expected_work\":";
    emit_work14(output, report.admission.expected_work);
    output << ",\"actual_work\":";
    emit_work14(output, report.admission.work);
    output << '}';
    }
    output << ",\"work_mutations\":";
    if (report.controls[5].outcome == "NOT_RUN_BY_PRECEDENCE") {
        output << "null";
    } else {
    output << "{\"pass\":"
           << (report.mutations.pass ? "true" : "false")
           << ",\"mutations\":[";
    constexpr std::array<std::string_view, 5> mutation_names{
        "graph_builds", "qp_sweeps", "plane_tests",
        "analytic_jv_multiply_adds", "hash_derivations"};
    for (std::size_t index = 0U; index < mutation_names.size(); ++index) {
        if (index != 0U) output << ',';
        output << "{\"field\":\"" << mutation_names[index]
               << "\",\"baseline_root\":\""
               << report.mutations.roots[2U * index]
               << "\",\"mutated_root\":\""
               << report.mutations.roots[2U * index + 1U]
               << "\",\"root_changed\":"
               << (report.mutations.root_changed[index]
                       ? "true" : "false") << '}';
    }
    output << "],\"ordered_roots\":";
    emit_root_array14(output, report.mutations.roots);
    output << ",\"expected_work\":";
    emit_work14(output, report.mutations.expected_work);
    output << ",\"actual_work\":";
    emit_work14(output, report.mutations.work);
    output << '}';
    }
    output << ",\"side_removal\":";
    if (report.controls[1].outcome == "NOT_RUN_BY_PRECEDENCE") {
        output << "null";
    } else {
    output << "{\"pass\":"
           << (report.side.pass ? "true" : "false")
           << ",\"fixture\":";
    emit_fixture14(output, report.side.fixture);
    output << ",\"analytic_velocity_root\":\""
           << report.side.analytic_velocity_root
           << "\",\"velocity_rms\":[";
    for (std::size_t index = 0U;
         index < report.side.velocity_rms.size(); ++index) {
        if (index != 0U) output << ',';
        output << static_cast<double>(report.side.velocity_rms[index]);
    }
    output << "],\"expected_velocity_rms\":[";
    for (std::size_t index = 0U;
         index < report.side.expected_velocity_rms.size(); ++index) {
        if (index != 0U) output << ',';
        output << static_cast<double>(
            report.side.expected_velocity_rms[index]);
    }
    output << "],\"steps\":[";
    for (std::size_t index = 0U; index < report.side.steps.size(); ++index) {
        if (index != 0U) output << ',';
        emit_step14(output, report.side.steps[index]);
    }
    output << "],\"trial_observables\":";
    emit_trial_observable_array14(output, report.side.observables);
    output << ",\"expected_work\":";
    emit_work14(output, report.side.expected_work);
    output << ",\"actual_work\":";
    emit_work14(output, report.side.work);
    output << '}';
    }
    output << ",\"controls\":[";
    for (std::size_t index = 0U; index < report.controls.size(); ++index) {
        if (index != 0U) output << ',';
        emit_control14(output, report.controls[index]);
    }
    const auto selectors = selector_values(report.finalization.selectors);
    output << "],\"selectors\":{\"invalid_hydrostatic_fixture_supported\":"
           << (selectors[0] ? "true" : "false")
           << ",\"tiny_open_support_artifact_supported\":"
           << (selectors[1] ? "true" : "false")
           << ",\"qp_budget_limited\":"
           << (selectors[2] ? "true" : "false")
           << ",\"projection_budget_limited\":"
           << (selectors[3] ? "true" : "false")
           << ",\"projection_cap_saturated\":"
           << (selectors[4] ? "true" : "false")
           << ",\"cause_not_unique\":"
           << (selectors[5] ? "true" : "false")
           << ",\"surface_direct_norm_insufficient\":"
           << (selectors[6] ? "true" : "false")
           << "},\"primary_route\":\""
           << report.finalization.decisive_category
           << "\",\"finalization\":{\"typed_outcome\":\""
           << report.finalization.outcome << "\",\"status\":\""
           << report.finalization.status << "\",\"total_work_exact\":"
           << (report.finalization.total_work_exact ? "true" : "false")
           << ",\"expected_total_work_root\":\""
           << report.finalization.expected_total_work_root
           << "\",\"actual_total_work_root\":\""
           << report.finalization.actual_total_work_root
           << "\",\"result_root\":\""
           << report.finalization.result_root << "\",";
    emit_work_seal14(output, report.finalization.work,
        report.finalization.expected_work, report.finalization.verifier);
    output << "},\"total_work\":{\"work_exact\":"
           << (report.total_verifier.exact ? "true" : "false")
           << ",\"expected_work_root\":\""
           << report.total_verifier.expected_root
           << "\",\"actual_work_root\":\""
           << report.total_verifier.actual_root
           << "\",\"expected_work\":";
    emit_work14(output, report.expected_total_work);
    output << ",\"actual_work\":";
    emit_work14(output, report.total_work);
    output << "},\"result_root\":\"" << report.result_root << "\"}\n";
}

} // namespace ncgp14

int main(int argc, char** argv) {
    try {
        if (argc != 2 || std::string_view(argv[1])
                != "--confined-pressure-contact-discriminator") {
            std::cerr << "usage: nonlocal-corrected-cpu-confined-pressure-contact "
                         "--confined-pressure-contact-discriminator\n";
            return 64;
        }
        return ncgp14::run_successor();
    } catch (const std::exception& error) {
        std::cerr << "NCGP14 apparatus failure: " << error.what() << '\n';
        return 3;
    }
}
