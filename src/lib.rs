#![no_std]
#![no_main]

pub fn convert_int_to_str(asd: u8) -> &'static str {
    match asd {
        1 => "hi",
        2 => "darien",
        _ => ":(",
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn test_convert() {
//         assert!(super::convert_int_to_str(1) != "hi")
//     }
// }
