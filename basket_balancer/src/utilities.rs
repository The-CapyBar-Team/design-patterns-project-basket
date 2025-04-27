pub(crate) const fn parse_u8(s: &str) -> u8 {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut result = 0u8;

    while i < bytes.len() {
        let b = bytes[i];
        if b < b'0' || b > b'9' {
            panic!("Invalid digit in const parse");
        }
        result = result * 10 + (b - b'0');
        i += 1;
    }

    result
}

pub(crate) async fn retry_and_report_error<F, R, E>(mut action: F) -> R
where
    F: AsyncFn() -> Result<R, E>,
    E: std::error::Error,
{
    loop {
        match action().await {
            Ok(result) => return result,
            Err(error) => eprintln!("Retry Error: {:?}", error),
        }
    }
}
