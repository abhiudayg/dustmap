/// Metrics computation. Placeholder — extended post-M1.
pub fn format_size(bytes: u64) -> String {
    bytesize::ByteSize(bytes).to_string()
}
