use crate::TermType;

pub fn term(s: &str) -> Option<(&str, &str, TermType)> {
    match s.as_bytes()[0] {
        b'(' => stripped_group::<b'(', b')'>(&s[1..]).map(|(a, b)| (a, b, TermType::Group)),
        b'[' => stripped_group::<b'[', b']'>(&s[1..]).map(|(a, b)| (a, b, TermType::Application)),
        _ => s
            .split_once(char::is_whitespace)
            .map(|(a, b)| (a, b.trim_start(), TermType::Variable)),
    }
}

pub fn constant_type(s: &str) -> Option<(&str, &str)> {
    if s.as_bytes()[0] == b'(' {
        stripped_group::<b'(', b')'>(&s[1..])
    } else {
        Some(word(s))
    }
}

// Any non-whitespace sequence of characters
pub fn word(s: &str) -> (&str, &str) {
    s.split_once(char::is_whitespace)
        .map(|(a, b)| (a, b.trim_start()))
        .unwrap_or((s, ""))
}

pub fn group<const OPEN: u8, const CLOSE: u8>(s: &str) -> Option<(&str, &str)> {
    if !s.as_bytes().starts_with(&[OPEN]) {
        return None;
    }
    stripped_group::<OPEN, CLOSE>(&s[1..])
}

pub fn right_term(s: &str) -> Option<(&str, &str)> {
    match s.as_bytes().last() {
        Some(bracket @ (b')' | b']')) => {
            let mut depth = 1;
            let opposite_bracket = if *bracket == b')' { b'(' } else { b'[' };

            for (i, c) in s.as_bytes()[..s.len() - 1].iter().enumerate().rev() {
                match c {
                    b if *b == opposite_bracket => {
                        depth -= 1;
                        if depth == 0 {
                            let rest = s[..i].trim_end();
                            let extracted = s[i..].trim();

                            return Some((rest, extracted));
                        }
                    }

                    b if b == bracket => depth += 1,
                    _ => {}
                }
            }
            None
        }
        Some(_) => Some(
            s.rsplit_once(char::is_whitespace)
                .map(|(a, b)| (a.trim_end(), b.trim_start()))
                .unwrap_or(("", s)),
        ),
        None => None,
    }
}

/// Parse a group in parentheses, with the first `(` already stripped.
pub fn stripped_group<const OPEN: u8, const CLOSE: u8>(s: &str) -> Option<(&str, &str)> {
    let mut depth = 1;
    for (i, c) in s.as_bytes().iter().enumerate() {
        if *c == OPEN {
            depth += 1
        } else if *c == CLOSE {
            depth -= 1;
            if depth == 0 {
                let rest = &s[i + 1..].trim_start();
                let extracted = s[..i].trim();

                return Some((extracted, rest));
            }
        }
    }
    None
}

pub fn string(s: &str) -> Option<(String, &str)> {
    let stripped = s.strip_prefix('"')?;
    let mut string_chars = stripped.char_indices().peekable();
    let mut string = String::new();
    let mut escape = false;
    let last_char = loop {
        let (i, c) = string_chars.next()?;
        if escape {
            match c {
                'n' => string.push('\n'),
                'r' => string.push('\r'),
                't' => string.push('\t'),
                '\\' => string.push('\\'),
                'a' => string.push(0x7 as char),
                'b' => string.push(0x8 as char),
                'v' => string.push(0xB as char),
                'f' => string.push(0xC as char),
                // "\DEL"
                'D' => {
                    let full = stripped[i..].get(..3)?;
                    if full != "DEL" {
                        return None;
                    }

                    string.push(0x7F as char);
                    string_chars.next();
                    string_chars.next();
                }
                '"' => string.push('"'),
                'x' => {
                    let hex = stripped.get(i + 1..i + 3)?;
                    let byte = u8::from_str_radix(hex, 16).ok()?;
                    string.push(byte as char);

                    string_chars.next();
                    string_chars.next();
                }
                'o' => {
                    let oct = stripped.get(i + 1..i + 4)?;
                    let byte = u8::from_str_radix(oct, 8).ok()?;
                    string.push(byte as char);

                    string_chars.next();
                    string_chars.next();
                    string_chars.next();
                }
                c if c.is_ascii_digit() => {
                    let mut num = c.to_digit(10).unwrap();
                    // Can be up to 5 digits
                    for _ in 0..5 {
                        if let Some((_, c)) = string_chars.peek() {
                            if let Some(digit) = c.to_digit(10) {
                                num = num * 10 + digit;
                                string_chars.next();
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    string.push(char::from_u32(num)?);
                }
                _ => return None,
            }
            escape = false;
        } else if c == '\\' {
            escape = true;
        } else if c == '"' {
            break i;
        } else {
            string.push(c);
        }
    };

    // We know last char is 1 byte since it's a `"`
    Some((string, stripped[last_char + 1..].trim_start()))
}

#[cfg(test)]
mod tests {
    // (Input, Extracted, Rest)
    const GROUP_VECTORS: &[(&str, &str, &str)] = &[
        ("a b c)", "a b c", ""),
        ("a b c) d e f", "a b c", "d e f"),
        ("a (b c) d) e f", "a (b c) d", "e f"),
        ("a (b (c d)) e) f g", "a (b (c d)) e", "f g"),
        ("a (b (c d)) e)f g", "a (b (c d)) e", "f g"),
        ("a (b (c d)) e)   f g", "a (b (c d)) e", "f g"),
        ("(a b c)) d e f", "(a b c)", "d e f"),
        ("(a b c) d e f)", "(a b c) d e f", ""),
        ("  a b c  ) d e f", "a b c", "d e f"),
        ("a b c d e f)", "a b c d e f", ""),
        ("a b c d e f)   ", "a b c d e f", ""),
        ("a b (c d) e f)", "a b (c d) e f", ""),
        ("a b (c d) e f)   ", "a b (c d) e f", ""),
        ("(a b (c d) e f))   ", "(a b (c d) e f)", ""),
        (
            "     a  b  (  c  d  )  e  f  )   ",
            "a  b  (  c  d  )  e  f",
            "",
        ),
        ("())   ", "()", ""),
        (" () )   ", "()", ""),
        (" ( ) )   ", "( )", ""),
        (" (()) )   ", "(())", ""),
        (" (()()) )   ", "(()())", ""),
        ("( (()()) )) extra stuff ", "( (()()) )", "extra stuff "),
    ];

    #[test]
    fn test_group() {
        for (input, extracted, rest) in GROUP_VECTORS {
            let (e, r) = super::stripped_group::<b'(', b')'>(input)
                .unwrap_or_else(|| panic!("failed to parse: {input}"));
            assert_eq!(e, *extracted, "input: {input}");
            assert_eq!(r, *rest, "input: {input}");
        }
    }

    const RIGHT_TERM_VECTORS: &[(&str, &str, &str)] = &[
        ("var1 var2 var3", "var1 var2", "var3"),
        (
            "[app (con T t)] (force a) b",
            "[app (con T t)] (force a)",
            "b",
        ),
        (
            "(lam x (var x)) b (con T t)",
            "(lam x (var x)) b",
            "(con T t)",
        ),
        ("[ [ [ [ app ] ] ] ]", "", "[ [ [ [ app ] ] ] ]"),
        ("( ( ( ( group ) ) ) )", "", "( ( ( ( group ) ) ) )"),
        ("singlevar", "", "singlevar"),
        ("test a [[app x1] x2]", "test a", "[[app x1] x2]"),
    ];

    #[test]
    fn test_right_term() {
        for (input, rest, extracted) in RIGHT_TERM_VECTORS {
            let (r, e) =
                super::right_term(input).unwrap_or_else(|| panic!("failed to parse: {input}"));
            assert_eq!(e, *extracted, "input: {input}");
            assert_eq!(r, *rest, "input: {input}");
        }
    }
}
