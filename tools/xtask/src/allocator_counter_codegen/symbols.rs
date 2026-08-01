pub(super) fn semantic_symbol_match(line: &str, symbol: &str) -> bool {
    line.match_indices(symbol).any(|(index, _)| {
        let before = line[..index].chars().next_back();
        let after = line[index + symbol.len()..].chars().next();
        (before.is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
            && after.is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_'))
            || rust_legacy_symbol_component_match(line, index, symbol)
            || rust_v0_symbol_component_match(line, index, symbol)
    })
}

fn rust_legacy_symbol_component_match(line: &str, index: usize, symbol: &str) -> bool {
    let before = &line[..index];
    let digit_start = before
        .char_indices()
        .rev()
        .find(|(_, character)| !character.is_ascii_digit())
        .map_or(0, |(position, character)| position + character.len_utf8());
    if before[digit_start..] != symbol.len().to_string() {
        return false;
    }

    let suffix = &line[index + symbol.len()..];
    let Some(hash_and_end) = suffix.strip_prefix("17h") else {
        return false;
    };
    let mut characters = hash_and_end.chars();
    (0..16).all(|_| {
        characters
            .next()
            .is_some_and(|character| character.is_ascii_hexdigit())
    }) && characters.next() == Some('E')
        && characters
            .next()
            .is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
}

fn rust_v0_symbol_component_match(line: &str, index: usize, symbol: &str) -> bool {
    // Rust v0 inserts an underscore separator before identifiers that start
    // with `_`. Compiler allocator shims therefore encode `__rust_dealloc`
    // as `14___rust_dealloc`: length 14, separator, then the 14-byte name.
    let Some(component) = symbol.strip_prefix('_') else {
        return false;
    };
    let before = &line[..index];
    let digit_start = before
        .char_indices()
        .rev()
        .find(|(_, character)| !character.is_ascii_digit())
        .map_or(0, |(position, character)| position + character.len_utf8());
    if before[digit_start..] != component.len().to_string() || !before[..digit_start].contains("_R")
    {
        return false;
    }
    line[index + symbol.len()..]
        .chars()
        .next()
        .is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
}
