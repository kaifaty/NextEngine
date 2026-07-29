use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_RUST_SOURCE_LINES: usize = 1_000;
const MAX_SOURCE_ATTRIBUTE_NESTING: u32 = 64;

pub(super) fn validate_source_layout(root: &Path) -> Result<(), String> {
    let mut source_sizes = BTreeMap::new();
    for relative_root in ["apps", "crates", "tools"] {
        let source_root = root.join(relative_root);
        if !source_root.is_dir() {
            return Err(format!("SOURCE_ROOT_MISSING: {}", source_root.display()));
        }
        let mut files = Vec::new();
        collect_strict_source_files(&source_root, &mut files)?;
        files.sort();
        for file in files {
            let relative = workspace_relative_path(root, &file)?;
            let body = read(&file)?;
            if contains_source_layout_escape_hatch(&body) {
                return Err(format!("SOURCE_LAYOUT_ESCAPE_HATCH: {relative}"));
            }
            let line_count = body.lines().count();
            if source_sizes.insert(relative.clone(), line_count).is_some() {
                return Err(format!("SOURCE_FILE_DUPLICATE: {relative}"));
            }
        }
    }
    validate_source_size_inventory(&source_sizes)
}

fn validate_source_size_inventory(source_sizes: &BTreeMap<String, usize>) -> Result<(), String> {
    for (path, line_count) in source_sizes {
        if *line_count > MAX_RUST_SOURCE_LINES {
            return Err(format!(
                "SOURCE_FILE_TOO_LARGE: {path} has {line_count} lines; limit is {MAX_RUST_SOURCE_LINES}"
            ));
        }
    }
    Ok(())
}

fn contains_source_layout_escape_hatch(body: &str) -> bool {
    let code = rust_code_without_comments_and_literals(body);
    let mut index = 0;
    while index < code.len() {
        if is_ascii_identifier_start(code[index]) {
            let start = index;
            index += 1;
            while index < code.len() && is_ascii_identifier_continue(code[index]) {
                index += 1;
            }
            let is_complete_token = ascii_identifier_is_complete_token(&code, start, index);
            if is_complete_token && &code[start..index] == b"include" {
                let next = skip_rust_whitespace(&code, index);
                if code.get(next) == Some(&b'!') {
                    return true;
                }
            } else if is_complete_token
                && &code[start..index] == b"use"
                && !has_raw_identifier_prefix(&code, start)
                && use_statement_imports_include(&code, index)
            {
                return true;
            }
            continue;
        }
        if code[index] == b'#' {
            let attribute_open = skip_rust_whitespace(&code, index + 1);
            if code.get(attribute_open) == Some(&b'[')
                && let Some(attribute_close) =
                    matching_delimiter(&code, attribute_open, b'[', b']', code.len())
                && attribute_contains_path_directive(&code, attribute_open + 1, attribute_close)
            {
                return true;
            }
        }
        index += 1;
    }
    false
}

fn has_raw_identifier_prefix(code: &[u8], identifier_start: usize) -> bool {
    identifier_start >= 2 && code.get(identifier_start - 2..identifier_start) == Some(b"r#")
}

fn use_statement_imports_include(code: &[u8], mut index: usize) -> bool {
    while index < code.len() && code[index] != b';' {
        if is_ascii_identifier_start(code[index]) {
            let start = index;
            index += 1;
            while index < code.len() && is_ascii_identifier_continue(code[index]) {
                index += 1;
            }
            if ascii_identifier_is_complete_token(code, start, index)
                && &code[start..index] == b"include"
            {
                return true;
            }
        } else {
            index += 1;
        }
    }
    false
}

fn attribute_contains_path_directive(code: &[u8], start: usize, end: usize) -> bool {
    attribute_contains_path_directive_at_depth(code, start, end, 0)
}

fn attribute_contains_path_directive_at_depth(
    code: &[u8],
    start: usize,
    end: usize,
    depth: u32,
) -> bool {
    let mut identifier = skip_rust_whitespace(code, start);
    if code.get(identifier..identifier.saturating_add(2)) == Some(b"r#") {
        identifier = skip_rust_whitespace(code, identifier + 2);
    }
    let identifier_start = identifier;
    while identifier < end && is_ascii_identifier_continue(code[identifier]) {
        identifier += 1;
    }
    if !ascii_identifier_is_complete_token(code, identifier_start, identifier) {
        return false;
    }
    match code.get(identifier_start..identifier) {
        Some(b"path") => true,
        Some(b"cfg_attr") if depth >= MAX_SOURCE_ATTRIBUTE_NESTING => true,
        Some(b"cfg_attr") => cfg_attr_contains_path_directive(code, identifier, end, depth + 1),
        _ => false,
    }
}

fn cfg_attr_contains_path_directive(code: &[u8], start: usize, end: usize, depth: u32) -> bool {
    let arguments_open = skip_rust_whitespace(code, start);
    if code.get(arguments_open) != Some(&b'(') {
        return false;
    }
    let Some(arguments_close) = matching_delimiter(code, arguments_open, b'(', b')', end) else {
        return false;
    };

    let mut delimiters = Vec::new();
    let mut argument_start = arguments_open + 1;
    let mut argument_index = 0_u32;
    let mut index = argument_start;
    while index <= arguments_close {
        let byte = code[index];
        match byte {
            b'(' => delimiters.push(b')'),
            b'[' => delimiters.push(b']'),
            b'{' => delimiters.push(b'}'),
            b')' | b']' | b'}' if delimiters.last() == Some(&byte) => {
                delimiters.pop();
            }
            b',' if delimiters.is_empty() => {
                if argument_index > 0
                    && attribute_contains_path_directive_at_depth(
                        code,
                        argument_start,
                        index,
                        depth,
                    )
                {
                    return true;
                }
                argument_index += 1;
                argument_start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    argument_index > 0
        && attribute_contains_path_directive_at_depth(code, argument_start, arguments_close, depth)
}

fn matching_delimiter(
    code: &[u8],
    open_index: usize,
    open: u8,
    close: u8,
    end: usize,
) -> Option<usize> {
    let mut depth = 0_u32;
    for (offset, byte) in code.get(open_index..end)?.iter().copied().enumerate() {
        if byte == open {
            depth += 1;
        } else if byte == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(open_index + offset);
            }
        }
    }
    None
}

fn rust_code_without_comments_and_literals(body: &str) -> Vec<u8> {
    let bytes = body.as_bytes();
    let mut code = bytes.to_vec();
    let source_start = if code.starts_with(&[0xef, 0xbb, 0xbf]) {
        code[..3].fill(b' ');
        3
    } else {
        0
    };
    if bytes.get(source_start..source_start + 2) == Some(b"#!")
        && bytes.get(source_start + 2) != Some(&b'[')
    {
        let shebang_end = bytes[source_start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(bytes.len(), |offset| source_start + offset);
        code[source_start..shebang_end].fill(b' ');
    }
    let mut index = 0;
    while index < bytes.len() {
        if bytes.get(index..index + 2) == Some(b"//") {
            let start = index;
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            code[start..index].fill(b' ');
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            let start = index;
            index += 2;
            let mut depth = 1_u32;
            while index < bytes.len() && depth > 0 {
                if bytes.get(index..index + 2) == Some(b"/*") {
                    depth += 1;
                    index += 2;
                } else if bytes.get(index..index + 2) == Some(b"*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
            code[start..index].fill(b' ');
            continue;
        }
        if bytes[index] == b'\''
            && let Some(end) = char_literal_end(bytes, index)
        {
            code[index..end].fill(b' ');
            index = end;
            continue;
        }
        if bytes[index] == b'r'
            && let Some((hashes, content_start)) = raw_string_start(bytes, index)
        {
            let start = index;
            index = content_start;
            while index < bytes.len() {
                if bytes[index] == b'"'
                    && bytes.get(index + 1..index + 1 + hashes)
                        == Some(&bytes[start + 1..start + 1 + hashes])
                {
                    index += hashes + 1;
                    break;
                }
                index += 1;
            }
            code[start..index].fill(b' ');
            continue;
        }
        if bytes[index] == b'"' {
            let start = index;
            index += 1;
            let mut escaped = false;
            while index < bytes.len() {
                let byte = bytes[index];
                index += 1;
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    break;
                }
            }
            code[start..index].fill(b' ');
            continue;
        }
        index += 1;
    }
    code
}

fn char_literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    let content = start + 1;
    if bytes.get(content) == Some(&b'\\') {
        let mut index = content;
        let mut escaped = false;
        while index < bytes.len() {
            let byte = bytes[index];
            index += 1;
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'\'' {
                return Some(index);
            }
        }
        return None;
    }

    let character = std::str::from_utf8(bytes.get(content..)?)
        .ok()?
        .chars()
        .next()?;
    let closing = content + character.len_utf8();
    (bytes.get(closing) == Some(&b'\'')).then_some(closing + 1)
}

fn raw_string_start(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    let mut delimiter = start + 1;
    while bytes.get(delimiter) == Some(&b'#') {
        delimiter += 1;
    }
    (bytes.get(delimiter) == Some(&b'"')).then_some((delimiter - start - 1, delimiter + 1))
}

fn skip_rust_whitespace(bytes: &[u8], mut index: usize) -> usize {
    while let Some(width) = rust_whitespace_width(bytes, index) {
        index += width;
    }
    index
}

fn rust_whitespace_width(bytes: &[u8], index: usize) -> Option<usize> {
    match bytes.get(index..)? {
        [byte, ..] if byte.is_ascii_whitespace() => Some(1),
        [0xc2, 0x85, ..] => Some(2),
        [0xe2, 0x80, 0x8e | 0x8f | 0xa8 | 0xa9, ..] => Some(3),
        _ => None,
    }
}

fn ascii_identifier_is_complete_token(bytes: &[u8], start: usize, end: usize) -> bool {
    let starts_at_boundary = match start.checked_sub(1).and_then(|index| bytes.get(index)) {
        None => true,
        Some(byte) if byte.is_ascii() => !is_ascii_identifier_continue(*byte),
        Some(_) => rust_whitespace_ends_at(bytes, start),
    };
    let ends_at_boundary = match bytes.get(end) {
        None => true,
        Some(byte) if byte.is_ascii() => !is_ascii_identifier_continue(*byte),
        Some(_) => rust_whitespace_width(bytes, end).is_some(),
    };
    starts_at_boundary && ends_at_boundary
}

fn rust_whitespace_ends_at(bytes: &[u8], end: usize) -> bool {
    (end >= 2 && bytes.get(end - 2..end) == Some(&[0xc2, 0x85]))
        || (end >= 3
            && matches!(
                bytes.get(end - 3..end),
                Some([0xe2, 0x80, 0x8e | 0x8f | 0xa8 | 0xa9])
            ))
}

const fn is_ascii_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

const fn is_ascii_identifier_continue(byte: u8) -> bool {
    is_ascii_identifier_start(byte) || byte.is_ascii_digit()
}

fn workspace_relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|_| {
        format!(
            "SOURCE_FILE_OUTSIDE_WORKSPACE: {} is not under {}",
            path.display(),
            root.display()
        )
    })?;
    Ok(relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

pub(super) fn collect_strict_source_files(
    root: &Path,
    output: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let root_metadata =
        fs::symlink_metadata(root).map_err(|error| format!("{}: {error}", root.display()))?;
    if root_metadata.file_type().is_symlink() {
        return Err(format!("SOURCE_SYMLINK_FORBIDDEN: {}", root.display()));
    }
    for entry in fs::read_dir(root).map_err(|error| format!("{}: {error}", root.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if file_type.is_symlink() {
            return Err(format!("SOURCE_SYMLINK_FORBIDDEN: {}", path.display()));
        }
        if file_type.is_dir() {
            collect_strict_source_files(&path, output)?;
        } else if file_type.is_file() && path.extension() == Some(OsStr::new("rs")) {
            output.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        MAX_RUST_SOURCE_LINES, MAX_SOURCE_ATTRIBUTE_NESTING, contains_source_layout_escape_hatch,
        validate_source_size_inventory,
    };

    #[test]
    fn source_size_inventory_rejects_a_new_oversized_file() {
        for accepted_lines in [MAX_RUST_SOURCE_LINES - 1, MAX_RUST_SOURCE_LINES] {
            let source_sizes =
                BTreeMap::from([("crates/example/src/lib.rs".to_owned(), accepted_lines)]);
            validate_source_size_inventory(&source_sizes)
                .expect("a source at or below the hard limit must pass");
        }

        let source_sizes = BTreeMap::from([(
            "crates/example/src/lib.rs".to_owned(),
            MAX_RUST_SOURCE_LINES + 1,
        )]);

        let error = validate_source_size_inventory(&source_sizes)
            .expect_err("an oversized source must fail");

        assert!(error.starts_with("SOURCE_FILE_TOO_LARGE: crates/example/src/lib.rs"));
    }

    #[test]
    fn source_layout_scan_rejects_split_escape_hatches_outside_literals_and_comments() {
        assert!(contains_source_layout_escape_hatch(
            "include /* deliberate spacing */ ! (\"part.rs\");"
        ));
        assert!(contains_source_layout_escape_hatch(
            "include\u{200e}!(\"hidden.not-rs\");"
        ));
        assert!(contains_source_layout_escape_hatch(
            "\u{feff}include!(\"hidden.not-rs\");"
        ));
        assert!(contains_source_layout_escape_hatch(
            "# [ path = \"hidden.rs\" ] mod hidden;"
        ));
        assert!(contains_source_layout_escape_hatch(
            "#\u{200f}[\u{2028}path = \"hidden.not-rs\"] mod hidden;"
        ));
        assert!(contains_source_layout_escape_hatch(
            "\u{feff}#[path = \"hidden.not-rs\"] mod hidden;"
        ));
        assert!(contains_source_layout_escape_hatch(
            "#[r#path = \"hidden.rs\"] mod hidden;"
        ));
        assert!(contains_source_layout_escape_hatch(
            "#[cfg_attr(all(), path = \"hidden.not-rs\")] mod hidden;"
        ));
        assert!(contains_source_layout_escape_hatch(
            "#[cfg_attr(all(), cfg_attr(any(), path = \"hidden.not-rs\"))] mod hidden;"
        ));
        assert!(contains_source_layout_escape_hatch(
            "use std::include as merge; merge!(\"hidden.not-rs\");"
        ));
        assert!(contains_source_layout_escape_hatch(
            "use std::{include as merge}; merge!(\"hidden.not-rs\");"
        ));
        assert!(contains_source_layout_escape_hatch(
            "use\u{2029}std::include as merge; merge!(\"hidden.not-rs\");"
        ));
        assert!(contains_source_layout_escape_hatch(
            "const QUOTE: char = '\"'; include!(\"hidden.not-rs\");"
        ));
        assert!(!contains_source_layout_escape_hatch(
            "const NOTE: &str = \"include!(part.rs) #[path]\";"
        ));
        assert!(!contains_source_layout_escape_hatch(
            "/* include!(\"part.rs\"); /* #[path] */ */"
        ));
        assert!(!contains_source_layout_escape_hatch(
            r##"const NOTE: &str = r#"include!("part.rs"); #[path]"#;"##
        ));
        assert!(!contains_source_layout_escape_hatch(
            "include_bytes!(\"fixture.bin\");"
        ));
        assert!(!contains_source_layout_escape_hatch(
            "#[cfg_attr(path = \"predicate\", derive(Clone))] struct Example;"
        ));
        assert!(!contains_source_layout_escape_hatch(
            "#[serde(path = \"domain::Type\")] struct Example;"
        ));
        assert!(!contains_source_layout_escape_hatch(
            "#[cfg_attr(any(), pathα = \"value\")] struct Example;"
        ));
        assert!(!contains_source_layout_escape_hatch(
            "fn example() { let include = 1; let r#use = include; }"
        ));
        assert!(!contains_source_layout_escape_hatch(
            "#!/usr/bin/include!\nfn example() {}"
        ));
        assert!(!contains_source_layout_escape_hatch(
            "\u{feff}#!/usr/bin/include!\nfn example() {}"
        ));
        assert!(!contains_source_layout_escape_hatch(
            "αinclude!(); let useα = include;"
        ));

        let nested_cfg_attr = |depth| {
            let mut attribute = "derive(Clone)".to_owned();
            for _ in 0..depth {
                attribute = format!("cfg_attr(all(), {attribute})");
            }
            format!("#[{attribute}] struct Example;")
        };
        assert!(!contains_source_layout_escape_hatch(&nested_cfg_attr(
            MAX_SOURCE_ATTRIBUTE_NESTING
        )));
        assert!(contains_source_layout_escape_hatch(&nested_cfg_attr(
            MAX_SOURCE_ATTRIBUTE_NESTING + 1
        )));
    }
}
