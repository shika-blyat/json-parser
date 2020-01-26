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
                        println!("{:#?}", error);
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
                label("}")(remaining)
                    .or_else(|error| {
                        if let JsonError::Failure(rem, mut error) = error {
                            if json_string()(rem).is_ok() {
                                error.set_reason(format!(
                                    "Expected a `}}`, found a string\n Help: You probably forgot a `,` here"
                                ));
                                Err(JsonError::Unsavable(rem.pos, error))
                            }else if label(",")(rem).is_ok(){
                                error.set_reason(format!(
                                    "Expected a `}}`, found a string\n Help: Trailing comma aren't allowed in json"
                                ));
                                Err(JsonError::Unsavable(rem.pos, error))
                            }
                            else {
                                error.set_reason(format!(
                                    "Expected a `}}`, found `{}`", 
                                    &rem.rem[0..rem.rem.find(|c| c == '\n' || c == ',').unwrap_or(rem.rem.len())]
                                ));
                                Err(JsonError::Unsavable(rem.pos, error))
                            }
                        } else {
                            unreachable!()
                        }
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
const CODE: &str = "
{
    \"num\": false,
    \"str\": \"abc\",
    \"obj\": {
        \"array\": [1,2,3]
    
}";

fn main() {
    println!("{:#?}", json(CODE));
}
