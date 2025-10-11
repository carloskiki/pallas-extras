use crate::TermType;

pub fn term(s: &str) -> Option<(&str, &str, TermType)> {
    match s.as_bytes()[0] {
        b'(' => stripped_group(&s[1..]).map(|(a, b)| (a, b, TermType::Group)),
        b'[' => s[1..]
            .split_once(']')
            .map(|(a, b)| (a, b.trim_start(), TermType::Application)),
        _ => s
            .split_once(char::is_whitespace)
            .map(|(a, b)| (a, b.trim_start(), TermType::Variable)),
    }
}

pub fn constant_type(s: &str) -> Option<(&str, &str)> {
    if s.as_bytes()[0] == b'(' {
        stripped_group(&s[1..])
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

pub fn list(s: &str) -> Option<(&str, &str)> {
    if !s.starts_with('[') {
        return None;
    }
    s[1..].split_once(']').map(|(a, b)| (a.trim(), b.trim_start()))
}

pub fn group(s: &str) -> Option<(&str, &str)> {
    if !s.starts_with('(') {
        return None;
    }
    stripped_group(&s[1..])
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
pub fn stripped_group(s: &str) -> Option<(&str, &str)> {
    let mut depth = 1;
    for (i, c) in s.as_bytes().iter().enumerate() {
        match c {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    let rest = &s[i + 1..].trim_start();
                    let extracted = s[..i].trim();

                    return Some((extracted, rest));
                }
            }
            _ => {}
        }
    }
    None
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
            let (e, r) = super::stripped_group(input).unwrap_or_else(|| panic!("failed to parse: {input}"));
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
