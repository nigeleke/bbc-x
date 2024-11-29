use once_cell::sync::Lazy;

use std::collections::HashMap;

type Bits = u64;

static CHAR_TO_BITS: Lazy<HashMap<u8, Bits>> = Lazy::new(|| {
    HashMap::from_iter(vec![
        (b'\0', 0),
        (b'A', 1),
        (b'B', 2),
        (b'C', 3),
        (b'D', 4),
        (b'E', 5),
        (b'F', 6),
        (b'G', 7),
        (b'H', 8),
        (b'I', 9),
        (b'J', 10),
        (b'K', 11),
        (b'L', 12),
        (b'M', 13),
        (b'N', 14),
        (b'O', 15),
        (b'P', 16),
        (b'Q', 17),
        (b'R', 18),
        (b'S', 19),
        (b'T', 20),
        (b'U', 21),
        (b'V', 22),
        (b'W', 23),
        (b'X', 24),
        (b'Y', 25),
        (b'Z', 26),
        (b'\'', 27),
        (b'<', 28),
        (b'>', 29),
        // (b'<=', 30), // Not parsed
        // (b'>=', 31), // Not parsed
        (b'0', 32),
        (b'1', 33),
        (b'2', 34),
        (b'3', 35),
        (b'4', 36),
        (b'5', 37),
        (b'6', 38),
        (b'7', 39),
        (b'8', 40),
        (b'9', 41),
        (b'.', 42),
        (b'@', 43),
        (b'+', 44),
        (b'-', 45),
        (b'(', 46),
        (b')', 47),
        (b'[', 48),
        (b']', 49),
        (b'*', 50),
        (b'/', 51),
        (b'=', 52),
        // (b'|=', 53), // Not parsed
        (b'^', 54), // Re-interpreted from up-arrow
        // (b'<-', 55), // Not parsed
        (b'?', 56),
        (b'"', 57),
        (b':', 58),
        (b';', 59),
        (b',', 60),
        (b' ', 61),
        (b'\n', 62),
        // (b'<ctrl>', 63), // Ctrl-? escapes not parsed
    ])
});

static BITS_TO_CHAR: Lazy<HashMap<Bits, u8>> =
    Lazy::new(|| HashMap::from_iter(CHAR_TO_BITS.iter().map(|(c, b)| (*b, *c))));

pub struct CharSet {}

impl CharSet {
    pub fn char_to_bits(char: u8) -> Option<Bits> {
        CHAR_TO_BITS.get(&char).copied()
    }

    pub fn bits_to_char(bits: Bits) -> Option<u8> {
        BITS_TO_CHAR.get(&bits).copied()
    }
}
