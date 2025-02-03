use flate2::read::GzDecoder;
use std::error::Error;
use std::io::Read;

/// Represents a complete STOMP frame with owned header and body.
#[derive(Debug)]
pub(crate) struct StompFrame {
    pub headers: String,
    pub body: Vec<u8>,
}

/// Looks for a header line starting with "content-length:" and returns its value.
fn get_content_length(headers: &str) -> Option<usize> {
    headers
        .lines()
        .find_map(|line| {
            line.strip_prefix("content-length:")
                .and_then(|s| s.trim().parse().ok())
        })
}

/// Parses a complete STOMP frame from `data` and returns a tuple:
/// (total number of bytes consumed, parsed StompFrame).
///
/// A frame is defined as:
/// - Headers terminated by "\n\n", followed by either:
///   - A body of length given by a content-length header and a trailing null byte, or
///   - A body terminated by a null byte.
/// Returns `None` if a complete frame isn’t yet available.
pub(crate) fn parse_stomp_frame(data: &[u8]) -> Option<(usize, StompFrame)> {
    let (header_len, header_end) = find_header_end(data)?;
    let headers = String::from_utf8_lossy(&data[..header_len]).to_string();
    parse_body(data, header_end, &headers)
}

fn find_header_end(data: &[u8]) -> Option<(usize, usize)> {
    let sep = b"\n\n";
    let pos = data.windows(2).position(|w| w == sep)?;
    Some((pos, pos + sep.len()))
}

fn parse_body(data: &[u8], header_end: usize, headers: &str) -> Option<(usize, StompFrame)> {
    if let Some(len) = get_content_length(headers) {
        parse_fixed_length_body(data, header_end, len, headers)
    } else {
        parse_null_terminated_body(data, header_end, headers)
    }
}

fn parse_fixed_length_body(
    data: &[u8],
    header_end: usize,
    body_length: usize,
    headers: &str,
) -> Option<(usize, StompFrame)> {
    let total_length = header_end + body_length + 1;
    if data.len() < total_length {
        return None;
    }
    let body = data[header_end..header_end + body_length].to_vec();
    Some((
        total_length,
        StompFrame {
            headers: headers.to_string(),
            body,
        },
    ))
}

fn parse_null_terminated_body(
    data: &[u8],
    header_end: usize,
    headers: &str,
) -> Option<(usize, StompFrame)> {
    let null_pos = data[header_end..].iter().position(|&b| b == 0)?;
    let total_length = header_end + null_pos + 1;
    let body = data[header_end..header_end + null_pos].to_vec();
    Some((
        total_length,
        StompFrame {
            headers: headers.to_string(),
            body,
        },
    ))
}

/// Decompresses gzipped data into a String using GzDecoder.
/// If decompression fails, you might want to fallback to interpreting the bytes directly.
pub(crate) fn decompress_gzipped_data(compressed: &[u8]) -> Result<String, std::io::Error> {
    let mut gz = GzDecoder::new(compressed);
    let mut decompressed = String::new();
    gz.read_to_string(&mut decompressed)?;
    Ok(decompressed)
}
