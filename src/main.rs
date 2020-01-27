#![feature(trait_alias)]
mod parsec;

use parsec::{
    character::{digit, label, string},
    multi::{many, many1},
    whitespace::ws,
    JsonError, Parser, ParserError, Remaining,
};

#[derive(Debug)]
pub struct JsonObject {
    members: Vec<Member>,
}
#[derive(Debug)]
pub struct Member {
    identifier: String,
    value: JsonValue,
}
impl Member {
    pub fn new(identifier: String, value: JsonValue) -> Self {
        Self { identifier, value }
    }
}
#[derive(Debug)]
pub enum JsonValue {
    String(String),
    Number(isize),
    Array(Vec<JsonValue>),
    True,
    False,
    Null,
    Object(JsonObject),
}
pub fn number<'a>() -> impl Parser<'a, JsonValue> {
    |s| {
        many1(digit(10))(s).map(|(remaining, num)| {
            (
                remaining,
                JsonValue::Number(num.into_iter().collect::<String>().parse().unwrap()),
            )
        })
    }
}
pub fn json_string<'a>() -> impl Parser<'a, JsonValue> {
    |s| string()(s).map(|(remaining, val)| (remaining, JsonValue::String(val.to_string())))
}
pub fn value<'a>() -> impl Parser<'a, JsonValue> {
    |s| {
        number()(s)
            .or_else(|error| match error {
                JsonError::Unsavable(_, _) => return Err(error),
                _ => json_string()(error.rem()),
            })
            .or_else(|error| match error {
                JsonError::Unsavable(_, _) => return Err(error),
                _ => array()(error.rem()),
            })
            .or_else(|error| match error {
                JsonError::Unsavable(_, _) => return Err(error),
                _ => keyword()(error.rem()),
            })
            .or_else(|error| match error {
                JsonError::Unsavable(_, _) => return Err(error),
                _ => object()(error.rem())
                    .map(|(remaining, object)| (remaining, JsonValue::Object(object))),
            })
            .or_else(|error| match error {
                JsonError::Unsavable(_, _) => return Err(error),
                _ => {
                    return Err(JsonError::Failure(
                        error.rem(),
                        ParserError::new(0..0, format!("Invalid value `{:#?}`", error.rem().rem)),
                    ))
                }
            })
    }
}
pub fn array<'a>() -> impl Parser<'a, JsonValue> {
    |s| {
        label("[")(s)
            .and_then(|(remaining, _)| {
                many(|s| {
                    ws()(s)
                        .and_then(|(remaining, _)| value()(remaining))
                        .and_then(|(remaining, val)| {
                            ws()(remaining)
                                .and_then(|(remaining, _)| label(",")(remaining))
                                .map(|(remaining, _)| (remaining, val))
                                .map_err(|error| {
                                    JsonError::Failure(
                                        error.rem(),
                                        ParserError::new(
                                            0..1,
                                            format!(
                                                "Unexpected character {:#?}",
                                                &error.rem().rem[0..1]
                                            ),
                                        ),
                                    )
                                })
                        })
                })(remaining)
                .and_then(|(remaining, mut vec)| {
                    ws()(remaining)
                        .and_then(|(remaining, _)| value()(remaining))
                        .map(|(remaining, val)| {
                            let (remaining, _) = ws()(remaining).unwrap();
                            vec.push(val);
                            (remaining, vec)
                        })
                })
                .map(|(remaining, vec)| (remaining, JsonValue::Array(vec)))
            })
            .and_then(|(remaining, vec)| {
                label("]")(remaining)
                    .and_then(|(remaining, _)| Ok((remaining, vec)))
                    .or_else(|error| {
                        Err(JsonError::Failure(
                            error.rem(),
                            ParserError::new(
                                0..1,
                                format!("Unexpected character {:#?}", &error.rem().rem[0..1]),
                            ),
                        ))
                    })
            })
    }
}
pub fn keyword<'a>() -> impl Parser<'a, JsonValue> {
    |s| {
        label("true")(s)
            .map(|(remaining, _)| (remaining, JsonValue::True))
            .or_else(|error| {
                label("false")(error.rem()).map(|(remaining, _)| (remaining, JsonValue::False))
            })
            .or_else(|error| {
                label("null")(error.rem()).map(|(remaining, _)| (remaining, JsonValue::Null))
            })
            .or_else(|error| {
                if let Some(c) = error.rem().rem.chars().nth(0) {
                    if c.is_alphabetic() {
                        if let JsonError::Failure(rem, mut reason) = error {
                            reason.set_reason(format!(
                                "Expected either true, false or null, found {}",
                                &rem.rem[0..rem
                                    .rem
                                    .find(|c| c == '\n' || c == ',')
                                    .unwrap_or(rem.rem.len())]
                            ));
                            return Err(JsonError::Unsavable(rem.pos, reason));
                        }
                    }
                }
                Err(error)
            })
    }
}
pub fn member<'a>() -> impl Parser<'a, Member> {
    |s| {
        ws()(s)
            .and_then(|(remaining, _)| string()(remaining))
            .or_else(|error| {
                if let JsonError::Failure(rem, mut error) = error {
                    if let Ok((_, val)) = value()(rem) {
                        match val {
                            JsonValue::Array(_) => error.set_reason(format!(
                                "Expected a string, found a string\n
Help: member identifier can only be a string"
                            )),
                            JsonValue::Number(_) => error.set_reason(format!(
                                "Expected a string, found a number\n
Help: member identifier can only be a string"
                            )),
                            JsonValue::Object(_) => error.set_reason(format!(
                                "Expected a string, found an object\n
Help: member identifier can only be a string"
                            )),
                            JsonValue::True => error.set_reason(format!(
                                "Expected a string, found keyword `true` \n
Help: member identifier can only be a string"
                            )),
                            JsonValue::False => error.set_reason(format!(
                                "Expected a string, found keyword `false` \n
Help: member identifier can only be a string"
                            )),
                            JsonValue::Null => error.set_reason(format!(
                                "Expected a string, found keyword `null` \n
Help: member identifier can only be a string"
                            )),
                            _ => unreachable!(),
                        }
                        Err(JsonError::Unsavable(rem.pos, error))
                    } else if label("}")(rem).is_ok() {
                        error.set_reason(format!(
                            "Expected a string`, found `}}`\n
Help: Trailing comma aren't allowed in json"
                        ));
                        Err(JsonError::Unsavable(rem.pos, error))
                    } else {
                        error.set_reason(format!("Expected a string`, found `}}`"));
                        Err(JsonError::Unsavable(rem.pos, error))
                    }
                } else {
                    unreachable!()
                }
            })
            .and_then(|(remaining, identifier)| {
                ws()(remaining)
                    .and_then(|(remaining, _)| label(":")(remaining))
                    .or_else(|error| match error {
                        JsonError::Failure(rem, mut error) => {
                            error.set_reason(format!("Expected a `:`"));
                            Err(JsonError::Unsavable(rem.pos, error))
                        }
                        JsonError::Unsavable(_, _) => Err(error),
                        _ => unreachable!(),
                    })
                    .and_then(|(remaining, _)| ws()(remaining))
                    .and_then(|(remaining, _)| value()(remaining))
                    .or_else(|error| match error {
                        JsonError::Failure(rem, mut error) => {
                            error.set_reason(format!("Missing a value after `:`"));
                            Err(JsonError::Unsavable(rem.pos, error))
                        }
                        JsonError::Unsavable(_, _) => Err(error),
                        _ => unreachable!(),
                    })
                    .and_then(|(remaining, value)| {
                        Ok((remaining, Member::new(identifier.to_string(), value)))
                    })
            })
    }
}
pub fn object<'a>() -> impl Parser<'a, JsonObject> {
    |s| {
        label("{")(s)
            .and_then(|(remaining, _)| {
                many(|s| {
                    member()(s)
                        .and_then(|(remaining, member)| {
                            label(",")(remaining).and_then(|(remaining, _)| Ok((remaining, member)))
                        })
                        .or_else(|error| match error {
                            JsonError::Failure(rem, mut error) => {
                                error.set_reason(format!("Missing a value after `:`"));
                                Err(JsonError::Unsavable(rem.pos, error))
                            }
                            JsonError::Unsavable(_, _) => Err(error),
                            _ => unreachable!(),
                        })
                })(remaining)
            })
            .and_then(|(remaining, mut members)| {
                member()(remaining).and_then(|(remaining, member)| {
                    members.push(member);
                    Ok((remaining, members))
                })
            })
            .and_then(|(remaining, members_vec)| {
                let (remaining, _) = ws()(remaining).unwrap();
                label("}")(remaining)
                    .or_else(|error| match error {
                        JsonError::Failure(rem, mut error) => {
                            if json_string()(rem).is_ok() {
                                error.set_reason(
                                    "Expected a `}}`, found a string\n
Help:You probably forgot a `,` here"
                                        .to_string(),
                                );
                                Err(JsonError::Unsavable(rem.pos, error))
                            } else if label(",")(rem).is_ok() {
                                error.set_reason(
                                    "Expected a `}}`, found a `,`\n
Help: Trailing comma aren't allowed in json"
                                        .to_string(),
                                );
                                Err(JsonError::Unsavable(rem.pos, error))
                            } else {
                                let reason = format!(
                                    "Expected a `}}`, found `{}`",
                                    &rem.rem[0..rem
                                        .rem
                                        .find(|c| c == '\n' || c == ',')
                                        .unwrap_or(rem.rem.len())]
                                );
                                error.set_reason(reason);
                                Err(JsonError::Unsavable(rem.pos, error))
                            }
                        }
                        _ => unreachable!(),
                    })
                    .and_then(|(remaining, _)| {
                        Ok((
                            remaining,
                            JsonObject {
                                members: members_vec,
                            },
                        ))
                    })
            })
    }
}
/*
*/
pub fn json<'a>(input: &'a str) -> Option<JsonObject> {
    let input = input.trim();
    match object()(Remaining::new(input, 0)) {
        Ok((remaining, obj)) => {
            if remaining.rem.is_empty() {
                Some(obj)
            } else {
                println!("remaining: {}", remaining);
                None
            }
        }
        Err(err) => {
            println!("error: {:#?}", err);
            None
        }
    }
}
const CODE: &str = r#"
{
    "num": fals,
    "str": "abc",
    "obj": {
        "array": [1,2,3]
    }
}"#;

fn main() {
    println!("{:#?}", json(CODE));
}
