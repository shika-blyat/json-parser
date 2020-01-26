#![feature(trait_alias)]
mod parsec;

use parsec::{
    character::{digit, label, string},
    multi::{many, many1},
    whitespace::ws,
    Parser, ParserError, Remaining,
};
const CODE: &str = "
{
    \"num\": 15,
    \"str\": \"abc\",
    \"obj\": {
        \"array\": [1,2,3]
    }
}";

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
            .or_else(|(remaining, _)| json_string()(remaining))
            .or_else(|(remaining, _)| array()(remaining))
            .or_else(|(remaining, _)| keyword()(remaining))
            .or_else(|(remaining, _)| {
                object()(remaining)
                    .map(|(remaining, object)| (remaining, JsonValue::Object(object)))
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
                                .map_err(|(remaining, _)| {
                                    (
                                        remaining,
                                        ParserError::new(
                                            0..1,
                                            format!(
                                                "Unexpected character {:#?}",
                                                &remaining.rem[0..1]
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
                    .or_else(|(remaining, _)| {
                        Err((
                            remaining,
                            ParserError::new(
                                0..1,
                                format!("Unexpected character {:#?}", &remaining.rem[0..1]),
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
            .or_else(|(remaining, _)| {
                label("false")(remaining).map(|(remaining, _)| (remaining, JsonValue::False))
            })
            .or_else(|(remaining, _)| {
                label("null")(remaining).map(|(remaining, _)| (remaining, JsonValue::Null))
            })
    }
}

pub fn object<'a>() -> impl Parser<'a, JsonObject> {
    |s| {
        label("{")(s)
            .and_then(|(remaining, _)| {
                many(|s| {
                    let (remaining, _) = ws()(s).unwrap();
                    string()(remaining).and_then(|(remaining, identifier)| {
                        ws()(remaining)
                            .and_then(|(remaining, _)| label(":")(remaining))
                            .and_then(|(remaining, _)| ws()(remaining))
                            .and_then(|(remaining, _)| value()(remaining))
                            .and_then(|(remaining, value)| {
                                label(",")(remaining).and_then(|(remaining, _)| {
                                    Ok((remaining, Member::new(identifier.to_string(), value)))
                                })
                            })
                    })
                })(remaining)
            })
            .and_then(|(remaining, mut members)| {
                let (remaining, _) = ws()(remaining).unwrap();
                string()(remaining).and_then(|(remaining, identifier)| {
                    ws()(remaining)
                        .and_then(|(remaining, _)| label(":")(remaining))
                        .and_then(|(remaining, _)| ws()(remaining))
                        .and_then(|(remaining, _)| value()(remaining))
                        .and_then(|(remaining, value)| {
                            members.push(Member::new(identifier.to_string(), value));
                            Ok((remaining, members))
                        })
                })
            })
            .and_then(|(remaining, members_vec)| {
                let (remaining, _) = ws()(remaining).unwrap();
                label("}")(remaining).and_then(|(remaining, _)| {
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
pub fn json<'a>(input: &'a str) -> Option<JsonObject> {
    match object()(Remaining::new(input.trim(), 0)) {
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
fn main() {
    println!("{:#?}", json(CODE));
}
