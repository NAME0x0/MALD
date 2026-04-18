pub const FAVICON_SVG: &str = include_str!("../assets/favicon.svg");

pub fn favicon_data_uri() -> String {
    let mut encoded = String::with_capacity(FAVICON_SVG.len() * 3);

    for byte in FAVICON_SVG.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push('%');
                encoded.push_str(&format!("{byte:02X}"));
            }
        }
    }

    format!("data:image/svg+xml,{encoded}")
}
