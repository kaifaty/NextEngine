//! LLVM const-TLS initializer validation for the allocator counter.

pub(super) fn validate_const_tls_definition(definition: &str) -> Result<(), String> {
    if !definition.contains("internal thread_local") || !definition.contains(" global ") {
        return Err("expected one internal thread-local LLVM global".to_owned());
    }
    if !definition.trim_end().ends_with(", align 8") {
        return Err("expected the 64-byte TLS state to have align 8".to_owned());
    }
    let (_, initializer_and_tail) = definition
        .split_once("}> <{")
        .ok_or_else(|| "missing aggregate const TLS initializer".to_owned())?;
    let (initializer, _) = initializer_and_tail
        .rsplit_once(" }>, align 8")
        .ok_or_else(|| "missing aligned aggregate const TLS initializer".to_owned())?;

    let mut bytes = Vec::new();
    for segment in initializer.split(", ") {
        append_tls_initializer_segment(segment, &mut bytes)?;
    }
    let initialized = bytes.iter().flatten().count();
    let undefined = bytes.len().saturating_sub(initialized);
    let zeroes = bytes.iter().filter(|byte| matches!(byte, Some(0))).count();
    let ff_offsets = bytes
        .iter()
        .enumerate()
        .filter_map(|(offset, byte)| (*byte == Some(u8::MAX)).then_some(offset))
        .collect::<Vec<_>>();
    let unexpected = bytes
        .iter()
        .flatten()
        .filter(|byte| **byte != 0 && **byte != u8::MAX)
        .count();
    let max_u32_is_exact = ff_offsets.len() == 4
        && ff_offsets.windows(2).all(|pair| pair[1] == pair[0] + 1)
        && ff_offsets[0] % std::mem::align_of::<u32>() == 0;
    if bytes.len() != 64
        || initialized != 61
        || undefined != 3
        || zeroes != 57
        || unexpected != 0
        || !max_u32_is_exact
    {
        return Err(format!(
            "const TLS bytes: total={}, initialized={initialized}, zero={zeroes}, ff={}, undef={undefined}, unexpected={unexpected}, ff_offsets={ff_offsets:?}",
            bytes.len(),
            ff_offsets.len(),
        ));
    }
    Ok(())
}

fn append_tls_initializer_segment(
    segment: &str,
    output: &mut Vec<Option<u8>>,
) -> Result<(), String> {
    let segment = segment.trim();
    let shape_end = segment
        .find("] ")
        .ok_or_else(|| format!("unsupported TLS initializer segment: {segment}"))?;
    let shape = segment
        .get(1..shape_end)
        .ok_or_else(|| format!("invalid TLS initializer segment: {segment}"))?;
    let (element_count, bit_width) = shape
        .split_once(" x i")
        .ok_or_else(|| format!("invalid TLS array shape: {shape}"))?;
    let element_count = element_count
        .parse::<usize>()
        .map_err(|_| format!("invalid TLS array length: {element_count}"))?;
    let bit_width = bit_width
        .parse::<usize>()
        .map_err(|_| format!("invalid TLS element width: {bit_width}"))?;
    if bit_width == 0 || bit_width % 8 != 0 {
        return Err(format!("unsupported TLS element width: i{bit_width}"));
    }
    let byte_count = element_count
        .checked_mul(bit_width / 8)
        .ok_or_else(|| "TLS initializer extent overflow".to_owned())?;
    let value = segment
        .get(shape_end + 2..)
        .ok_or_else(|| format!("missing TLS initializer value: {segment}"))?;
    if value == "undef" {
        output.extend(std::iter::repeat_n(None, byte_count));
    } else if value == "zeroinitializer" {
        output.extend(std::iter::repeat_n(Some(0), byte_count));
    } else if let Some(literal) = value
        .strip_prefix("c\"")
        .and_then(|value| value.strip_suffix('"'))
    {
        if bit_width != 8 {
            return Err("LLVM c-string used for a non-i8 TLS array".to_owned());
        }
        let decoded = decode_llvm_byte_string(literal)?;
        if decoded.len() != byte_count {
            return Err(format!(
                "TLS c-string extent mismatch: declared={byte_count}, decoded={}",
                decoded.len()
            ));
        }
        output.extend(decoded.into_iter().map(Some));
    } else {
        return Err(format!("unsupported TLS initializer value: {value}"));
    }
    Ok(())
}

fn decode_llvm_byte_string(literal: &str) -> Result<Vec<u8>, String> {
    let input = literal.as_bytes();
    let mut decoded = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if input[index] == b'\\' {
            let high = input
                .get(index + 1)
                .and_then(|value| hex_nibble(*value))
                .ok_or_else(|| "invalid LLVM byte-string escape".to_owned())?;
            let low = input
                .get(index + 2)
                .and_then(|value| hex_nibble(*value))
                .ok_or_else(|| "invalid LLVM byte-string escape".to_owned())?;
            decoded.push((high << 4) | low);
            index += 3;
        } else if input[index].is_ascii() {
            decoded.push(input[index]);
            index += 1;
        } else {
            return Err("non-ASCII LLVM byte-string content".to_owned());
        }
    }
    Ok(decoded)
}

const fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}
