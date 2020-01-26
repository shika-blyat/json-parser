use crate::{Parser, ParserError, Remaining};

pub fn label<'a>(str_to_match: &'a str) -> impl Parser<'a, &'a str> {
    move |s: Remaining<'a>| {
        if str_to_match.len() > s.rem_len() {
            return Err((
                s,
                ParserError::new(
                    s.rem_len()..s.pos,
                    format!("Expected `{}` found `{}`", str_to_match, s),
                ),
            ));
        }
        let mut schars = s.rem.chars();
        let chars_to_match = str_to_match.chars();
        for i in chars_to_match.into_iter() {
            if i != schars.next().unwrap() {
                return Err((
                    s,
                    ParserError::new(
                        s.pos..s.rem_len(),
                        format!(
                            "Expected `{}` found `{}`",
                            str_to_match,
                            &s.rem[..str_to_match.len() - 1]
                        ),
                    ),
                ));
            }
        }
        Ok((
            Remaining::new(&s.rem[str_to_match.len()..], s.pos + str_to_match.len()),
            str_to_match,
        ))
    }
}

pub fn digit<'a>(base: u32) -> impl Parser<'a, char> {
    move |s: Remaining<'a>| {
        if let Some(c) = s.rem.chars().nth(0) {
            if c.is_digit(base) {
                return Ok((Remaining::new(&s.rem[c.len_utf8()..], s.pos + 1), c));
            }
        }
        Err((s, ParserError::new_empty()))
    }
}
pub fn string<'a>() -> impl Parser<'a, &'a str> {
    // This code is clearly not elegant. Any improvements are welcome
    move |s| {
        label("\"")(s)
            .and_then(|(remaining, _)| {
                let mut last_escaped = false;
                for (k, i) in remaining.rem.chars().enumerate() {
                    if i == '\"' && !last_escaped {
                        return Ok((
                            Remaining::new(&remaining.rem[k..], remaining.pos + k),
                            &remaining.rem[..k],
                        ));
                    } else if i == '\\' {
                        last_escaped = true;
                    } else if last_escaped {
                        last_escaped = false;
                    }
                }
                Err((
                    remaining,
                    ParserError::new(
                        remaining.pos - 1..remaining.rem.len(),
                        "Unclosed string delimiter".to_string(),
                    ),
                ))
            })
            .and_then(|(remaining, val)| {
                label("\"")(remaining).map(|(remaining, _)| (remaining, val))
            })
    }
}
