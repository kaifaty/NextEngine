#include "fixed_sha256.hpp"

#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <fcntl.h>
#include <span>
#include <unistd.h>

namespace {

constexpr std::size_t CACHE_BYTES = 1033625U;
constexpr std::array<std::uint8_t, 8U> MAGIC{{'N','E','F','P','R','B','1',0U}};
constexpr std::array<std::uint8_t, 45U> VECTOR_DOMAIN{{
    'n','e','x','t','e','n','g','i','n','e','.','n','o','n','l','o','c','a','l','.',
    'r','6','3','z','m','.','b','i','n','a','r','y','1','2','8','-','v','e','c','t',
    'o','r','.','v','1'}};
constexpr std::array<std::uint8_t, 44U> SELECTED_DOMAIN{{
    'n','e','x','t','e','n','g','i','n','e','.','n','o','n','l','o','c','a','l','.',
    'r','6','3','z','m','.','s','e','l','e','c','t','e','d','-','b','i','n','a','r',
    'y','.','v','1'}};

std::array<std::uint8_t, CACHE_BYTES> cache{};

struct View {
    const std::uint8_t* data = nullptr;
    std::size_t size = 0U;
};

struct VectorView {
    const std::uint8_t* data = nullptr;
    std::uint64_t count = 0U;
};

struct ParseWork final {
    std::uint64_t take_calls = 0U;
    std::uint64_t take_bytes = 0U;
    std::uint64_t u8_reads = 0U;
    std::uint64_t u32_reads = 0U;
    std::uint64_t u64_reads = 0U;
    std::uint64_t string_reads = 0U;
    std::uint64_t vector_reads = 0U;
    std::uint64_t skip_operations = 0U;
    std::uint64_t structural_predicates = 0U;
    std::uint64_t views_created = 0U;
};

class Cursor final {
public:
    explicit Cursor(std::span<const std::uint8_t> bytes,
        ParseWork* work = nullptr) : bytes_(bytes), work_(work) {}

    View take(std::size_t count) noexcept {
        if (work_ != nullptr) ++work_->take_calls;
        if (!ok_ || count > bytes_.size() - position_) {
            ok_ = false;
            return {};
        }
        const View result{bytes_.data() + position_, count};
        position_ += count;
        if (work_ != nullptr) {
            work_->take_bytes += count;
            ++work_->views_created;
        }
        return result;
    }

    std::uint8_t u8() noexcept {
        if (work_ != nullptr) ++work_->u8_reads;
        const View value = take(1U);
        return value.data == nullptr ? 0U : value.data[0U];
    }

    std::uint32_t u32() noexcept {
        if (work_ != nullptr) ++work_->u32_reads;
        const View value = take(4U);
        if (value.data == nullptr) return 0U;
        return static_cast<std::uint32_t>(value.data[0U])
            | (static_cast<std::uint32_t>(value.data[1U]) << 8U)
            | (static_cast<std::uint32_t>(value.data[2U]) << 16U)
            | (static_cast<std::uint32_t>(value.data[3U]) << 24U);
    }

    std::uint64_t u64() noexcept {
        if (work_ != nullptr) ++work_->u64_reads;
        const View value = take(8U);
        if (value.data == nullptr) return 0U;
        std::uint64_t result = 0U;
        for (std::size_t index = 0U; index < 8U; ++index)
            result |= static_cast<std::uint64_t>(value.data[index])
                << (8U * index);
        return result;
    }

    View string() noexcept {
        if (work_ != nullptr) ++work_->string_reads;
        const std::uint64_t count = u64();
        if (!expect(count <= CACHE_BYTES)) {
            ok_ = false;
            return {};
        }
        return take(static_cast<std::size_t>(count));
    }

    VectorView vector(std::size_t width) noexcept {
        if (work_ != nullptr) ++work_->vector_reads;
        const std::uint64_t count = u64();
        if (!expect(width != 0U
            && count <= (bytes_.size() - position_) / width)) {
            ok_ = false;
            return {};
        }
        const View value = take(static_cast<std::size_t>(count) * width);
        return {value.data, count};
    }

    void skip_string() noexcept {
        if (work_ != nullptr) ++work_->skip_operations;
        static_cast<void>(string());
    }
    void skip_vector(std::size_t width) noexcept {
        if (work_ != nullptr) ++work_->skip_operations;
        static_cast<void>(vector(width));
    }

    bool expect(bool value) noexcept {
        if (work_ != nullptr) ++work_->structural_predicates;
        if (!value) ok_ = false;
        return value;
    }

    bool ok() const noexcept { return ok_; }
    std::size_t position() const noexcept { return position_; }

private:
    std::span<const std::uint8_t> bytes_;
    ParseWork* work_ = nullptr;
    std::size_t position_ = 0U;
    bool ok_ = true;
};

bool equal(View left, std::span<const std::uint8_t> right) noexcept {
    if (left.size != right.size()) return false;
    for (std::size_t index = 0U; index < left.size; ++index)
        if (left.data[index] != right[index]) return false;
    return true;
}

void skip_certificate(Cursor& reader) noexcept {
    if (!reader.expect(reader.u8() <= 1U)) return;
    if (!reader.expect(reader.u8() <= 1U)) return;
    static_cast<void>(reader.take(3U * 8U));
    static_cast<void>(reader.take(8U));
    for (std::size_t index = 0U; index < 3U; ++index) reader.skip_string();
}

void skip_certificates(Cursor& reader) noexcept {
    const std::uint64_t count = reader.u64();
    if (!reader.expect(count <= 64U)) {
        static_cast<void>(reader.take(CACHE_BYTES));
        return;
    }
    for (std::uint64_t index = 0U; index < count && reader.ok(); ++index)
        skip_certificate(reader);
}

void skip_profile(Cursor& reader) noexcept {
    for (std::size_t index = 0U; index < 3U; ++index)
        if (!reader.expect(reader.u8() <= 1U)) return;
    static_cast<void>(reader.take(6U * 8U));
    for (std::size_t index = 0U; index < 4U; ++index)
        reader.skip_vector(8U);
    static_cast<void>(reader.take(3U * 8U));
    for (std::size_t index = 0U; index < 4U; ++index) reader.skip_string();
}

struct Projection {
    std::uint64_t dimension = 0U;
    std::uint64_t columns = 0U;
    std::array<VectorView, 6U> roles{};
    VectorView tangent;
    View sigma;
    View inverse;
    View projected_scale;
    View declared_tangent;
    View declared_original;
    View declared_projected;
};

Projection parse(Cursor& reader) noexcept {
    Projection value;
    if (!reader.expect(equal(reader.take(MAGIC.size()), MAGIC))) return value;
    if (!reader.expect(reader.u32() == 1U)) return value;
    if (!reader.expect(reader.u32() == 0x01020304U)) return value;
    if (!reader.expect(reader.u32() == 8U)) return value;
    if (!reader.expect(reader.u32() == 16U)) return value;
    if (!reader.expect(reader.u8() <= 1U)) return value;
    reader.skip_string();
    value.dimension = reader.u64();
    value.columns = reader.u64();
    for (std::size_t index = 0U; index < 9U; ++index) reader.skip_string();
    if (!reader.expect(reader.u64() == 3U)) return value;
    value.roles[2U] = reader.vector(16U);
    value.roles[3U] = reader.vector(16U);
    value.roles[4U] = reader.vector(16U);
    skip_certificates(reader);
    if (!reader.ok()) return value;
    value.tangent = reader.vector(16U);
    value.sigma = reader.take(16U);
    value.declared_tangent = reader.string();
    reader.skip_vector(8U);
    const std::uint64_t index_count = reader.u64();
    if (!reader.expect(index_count <= 4096U)) return value;
    for (std::uint64_t index = 0U; index < index_count; ++index)
        static_cast<void>(reader.u64());
    value.inverse = reader.take(16U);
    for (std::size_t index = 0U; index < 3U; ++index) reader.skip_string();
    value.roles[1U] = reader.vector(16U);
    value.roles[0U] = reader.vector(16U);
    value.projected_scale = reader.take(16U);
    value.declared_original = reader.string();
    value.declared_projected = reader.string();
    reader.skip_vector(8U);
    for (std::size_t index = 0U; index < 3U; ++index) reader.skip_string();
    skip_profile(reader);
    if (!reader.ok()) return value;
    reader.skip_string();
    reader.skip_string();
    if (!reader.expect(reader.u64() == 3U)) return value;
    reader.skip_vector(16U);
    reader.skip_vector(16U);
    value.roles[5U] = reader.vector(16U);
    skip_certificates(reader);
    if (!reader.ok()) return value;
    for (std::size_t index = 0U; index < 3U; ++index) reader.skip_string();
    return value;
}

void update_be64(r63zm::Sha256& hash, std::uint64_t value) noexcept {
    std::array<std::uint8_t, 8U> bytes{};
    for (std::size_t index = 0U; index < bytes.size(); ++index)
        bytes[index] = static_cast<std::uint8_t>(value >> (56U - 8U * index));
    hash.update(bytes);
}

void update_tlv_header(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t length) noexcept {
    hash.update_byte(tag);
    update_be64(hash, length);
}

void update_tlv(r63zm::Sha256& hash, std::uint8_t tag, View value) noexcept {
    update_tlv_header(hash, tag, value.size);
    hash.update(std::span<const std::uint8_t>(value.data, value.size));
}

template <std::size_t Size>
void update_tlv(r63zm::Sha256& hash, std::uint8_t tag,
    const std::array<std::uint8_t, Size>& value) noexcept {
    update_tlv_header(hash, tag, value.size());
    hash.update(value);
}

void update_tlv_be64(r63zm::Sha256& hash, std::uint8_t tag,
    std::uint64_t value) noexcept {
    update_tlv_header(hash, tag, 8U);
    update_be64(hash, value);
}

std::array<std::uint8_t, 16U> reversed(View value) noexcept {
    std::array<std::uint8_t, 16U> result{};
    if (value.size != result.size()) return result;
    for (std::size_t index = 0U; index < result.size(); ++index)
        result[index] = value.data[result.size() - 1U - index];
    return result;
}

r63zm::Digest vector_root(VectorView value) noexcept {
    r63zm::Sha256 hash;
    update_tlv(hash, 0x01U, VECTOR_DOMAIN);
    update_tlv_be64(hash, 0x02U, value.count);
    update_tlv_header(hash, 0x03U, value.count * 16U);
    for (std::uint64_t component = 0U; component < value.count; ++component) {
        const View raw{value.data + component * 16U, 16U};
        hash.update(reversed(raw));
    }
    return hash.finish();
}

[[maybe_unused]] r63zm::Digest selected_root(const Projection& value,
    const std::array<r63zm::Digest, 7U>& roots) noexcept {
    r63zm::Sha256 hash;
    update_tlv(hash, 0x01U, SELECTED_DOMAIN);
    update_tlv_be64(hash, 0x02U, value.dimension);
    update_tlv_be64(hash, 0x03U, value.columns);
    update_tlv(hash, 0x04U, reversed(value.inverse));
    update_tlv(hash, 0x05U, reversed(value.projected_scale));
    update_tlv(hash, 0x06U, reversed(value.sigma));
    update_tlv_be64(hash, 0x07U, value.tangent.count);
    update_tlv(hash, 0x08U, roots[0U]);
    for (std::size_t role = 0U; role < value.roles.size(); ++role) {
        const std::array<std::uint8_t, 1U> role_byte{{
            static_cast<std::uint8_t>(role)}};
        update_tlv(hash, static_cast<std::uint8_t>(0x10U + 3U * role),
            role_byte);
        update_tlv_be64(hash,
            static_cast<std::uint8_t>(0x11U + 3U * role),
            value.roles[role].count);
        update_tlv(hash, static_cast<std::uint8_t>(0x12U + 3U * role),
            roots[role + 1U]);
    }
    update_tlv(hash, 0x30U, value.declared_tangent);
    update_tlv(hash, 0x31U, value.declared_projected);
    update_tlv(hash, 0x32U, value.declared_original);
    return hash.finish();
}

std::array<char, 65U> hex(const r63zm::Digest& digest) noexcept {
    constexpr std::array<char, 16U> alphabet{{
        '0','1','2','3','4','5','6','7','8','9','a','b','c','d','e','f'}};
    std::array<char, 65U> result{};
    for (std::size_t index = 0U; index < digest.size(); ++index) {
        result[2U * index] = alphabet[digest[index] >> 4U];
        result[2U * index + 1U] = alphabet[digest[index] & 0x0fU];
    }
    return result;
}

bool digest_equal(const r63zm::Digest& digest, const char* expected) noexcept {
    const auto text = hex(digest);
    for (std::size_t index = 0U; index < 64U; ++index)
        if (text[index] != expected[index]) return false;
    return expected[64U] == '\0';
}

[[maybe_unused]] bool read_cache(const char* path) noexcept {
    const int descriptor = open(path, O_RDONLY);
    if (descriptor < 0) return false;
    std::size_t offset = 0U;
    while (offset < cache.size()) {
        const ssize_t count = read(descriptor, cache.data() + offset,
            cache.size() - offset);
        if (count <= 0) {
            static_cast<void>(close(descriptor));
            return false;
        }
        offset += static_cast<std::size_t>(count);
    }
    std::uint8_t extra = 0U;
    const ssize_t trailing = read(descriptor, &extra, 1U);
    const int closed = close(descriptor);
    return trailing == 0 && closed == 0;
}
} // namespace

#ifndef R63ZM_RAW_PROBE_LIBRARY
int main(int argc, char** argv) {
    if (argc != 2 || !read_cache(argv[1])) return 2;
    Cursor reader(cache);
    const Projection projection = parse(reader);
    std::array<r63zm::Digest, 7U> roots{};
    roots[0U] = vector_root(projection.tangent);
    for (std::size_t role = 0U; role < projection.roles.size(); ++role)
        roots[role + 1U] = vector_root(projection.roles[role]);
    const r63zm::Digest selected = selected_root(projection, roots);
    const r63zm::Digest source = r63zm::sha256(cache);
    constexpr std::array<const char*, 7U> EXPECTED_ROOTS{{
        "03e18bacbb09013618182985941b23496d9e9bab0ca95cb3f9edfe22193e1ada",
        "2cb58eea5f6b17bc666a5d4b1c0c9ef644b2c10f154f5d651bf24e446ad6a729",
        "1cabfd31c413d1a36fc299955739196bb50924bebe744ae57c1c99f31a9fc5b9",
        "950cd25039eafe48f7bb5d5cbdd8b049fe6d14cd0a178338fd50c9d797710a5b",
        "1573eeeb09ca81f104de3d56e4bf268ac0e9839ce83525472bb9c51d803bf622",
        "4d59cf5bcaaea440e3f061d00cbb05872d4be75d408e9927050a3ffd1efff392",
        "1b5581ab5871dcd60b5ff1ce180d0572fcb9725dffe40570595c70fda866bc49",
    }};
    bool exact = reader.ok() && reader.position() == cache.size()
        && projection.dimension == 102U && projection.columns == 315U
        && projection.tangent.count == 32130U;
    for (std::size_t index = 0U; index < roots.size(); ++index)
        exact = exact && digest_equal(roots[index], EXPECTED_ROOTS[index]);
    exact = exact && digest_equal(source,
        "23dbf605ad7b6ae12c4cf6a80404ead9617354ff2848bd010b52c7fa7f83bb84")
        && digest_equal(selected,
        "dc1b5328d2999dadafa8915ebe10d36029cc5b5ba13f2657670de75ee4a466a5");
    const auto source_hex = hex(source);
    const auto selected_hex = hex(selected);
    std::printf("{\"schema\":\"r63zm-fixed-raw-semantic-probe-v1\","
        "\"status\":\"%s\",\"cache_bytes\":%zu,\"parsed_bytes\":%zu,"
        "\"source_root\":\"%s\",\"selected_root\":\"%s\","
        "\"payload_roots\":[",
        exact ? "PASS" : "FAIL", cache.size(), reader.position(),
        source_hex.data(), selected_hex.data());
    for (std::size_t index = 0U; index < roots.size(); ++index) {
        const auto root_hex = hex(roots[index]);
        std::printf("%s\"%s\"", index == 0U ? "" : ",", root_hex.data());
    }
    std::printf("]}\n");
    return exact ? 0 : 1;
}
#endif
