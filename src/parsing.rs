use super::Rule;
use pest::{
    error::{Error as PestError, ErrorVariant},
    iterators::{Pair, Pairs},
};

#[derive(Debug)]
pub enum Type {
    List(Vec<TypeExpr>),
    SimpleType(String),
    Tuple(Vec<TypeExpr>),
    AsType {
        name: String,
        types: Vec<TypeExpr>,
        target: Box<TypeExpr>,
    },
    GenericType {
        name: String,
        types: Vec<TypeExpr>,
    },
}

impl Default for Type {
    fn default() -> Self {
        Self::SimpleType(String::default())
    }
}

#[derive(Default, Debug)]
pub struct TypeExpr {
    original: String,
    reference: Option<String>,
    impl_marker: Option<String>,
    r#type: Type,
    as_target: Option<Box<Type>>,
}

pub enum ParseError {
    EOI,
}

macro_rules! fields {
    ($pair:ident |> $children:ident $(: $($field:ident),*)? $(:> $drain:ident)?) => {
        #[allow(unused_mut)]
        let mut $children = $pair.clone().into_inner();

        $(
            $(
                let $field: Pair<Rule> = $children
                    .next()
                    .ok_or_else(||
                        PestError::new_from_span(
                            ErrorVariant::ParsingError {
                                positives: vec![$pair.as_rule()],
                                negatives: vec![]
                            },
                            $pair.as_span()
                        )
                    )?;
            )*
        )?

        $(
            let $drain: Vec<Pair<Rule>> = $children.collect();
        )?
    };
}

pub fn parse_type_tuple(
    pair: &Pair<Rule>,
    rstack: &mut Vec<Rule>,
) -> Result<TypeExpr, PestError<Rule>> {
    fields!(pair |> children :> types);

    Ok(TypeExpr {
        r#type: Type::List(
            types
                .into_iter()
                .map(|pair| -> Result<TypeExpr, PestError<Rule>> {
                    rstack.push(pair.as_rule());

                    let res = match pair.as_rule() {
                        Rule::r#type => todo!(),
                        Rule::tuple => {
                            parse_type_tuple(&pair.into_inner().next().unwrap(), rstack)?
                        }
                        _ => unreachable!(),
                    };

                    rstack.pop();
                    Ok(res)
                })
                .collect::<Result<Vec<TypeExpr>, PestError<Rule>>>()?,
        ),
        ..Default::default()
    })
}

fn parse_type_expr(pair: &Pair<Rule>, rstack: &mut Vec<Rule>) -> Result<TypeExpr, PestError<Rule>> {
    fields!(pair |> children :> drain);

    let (reference, impl_marker, inner, as_target) = {
        match drain.len() {
            1 => (None, None, drain.get(0).unwrap(), None),
            2 => {
                let first = drain.get(0).unwrap();
                let second = drain.get(1).unwrap();

                match first.as_rule() {
                    Rule::reference => (Some(first), None, second, None),
                    Rule::impl_marker => (None, Some(first), second, None),
                    Rule::regular_type | Rule::turbofish_type => (None, None, first, Some(second)),
                    _ => unreachable!(),
                }
            }
            3 => {
                let first = drain.get(0).unwrap();
                let second = drain.get(1).unwrap();
                let third = drain.get(2).unwrap();

                match first.as_rule() {
                    Rule::reference => match second.as_rule() {
                        Rule::impl_marker => (Some(first), Some(second), third, None),
                        _ => unreachable!(),
                    },
                    Rule::impl_marker => (None, Some(first), second, Some(third)),
                    _ => unreachable!(),
                }
            }
            4 => (
                drain.get(0),
                drain.get(1),
                drain.get(2).unwrap(),
                drain.get(3),
            ),
            _ => unreachable!(),
        }
    };

    rstack.push(pair.as_rule());

    let res = Ok(TypeExpr {
        original: pair.as_str().to_owned(),
        reference: reference.map(|reference| reference.as_str().to_owned()),
        impl_marker: impl_marker.map(|impl_marker| impl_marker.as_str().to_owned()),
        r#type: parse_type(inner, rstack)?,
        as_target: {
            if let Some(as_target) = as_target {
                let r#type = parse_type(as_target, rstack)?;
                Some(Box::new(r#type))
            } else {
                None
            }
        },
    });

    rstack.pop();

    res
}

fn parse_type(pair: &Pair<Rule>, rstack: &mut Vec<Rule>) -> Result<Type, PestError<Rule>> {
    rstack.push(pair.as_rule());
    let res = match pair.as_rule() {
        Rule::regular_type => parse_regular_type(pair, rstack),
        Rule::turbofish_type => parse_turbofish_type(pair, rstack),
        Rule::closure_type => parse_closure_type(pair, rstack),
        _ => unreachable!(),
    };
    rstack.pop();
    res
}

fn parse_regular_type(pair: &Pair<Rule>, rstack: &mut Vec<Rule>) -> Result<Type, PestError<Rule>> {
    fields!(pair |> children: typename :> generics);

    let typename = typename.as_str().trim().to_owned();

    if let Some(generic_type) = generics.get(0) {
        let Type::List(types) = parse_type_tuple(generic_type, rstack)?.r#type 
            else { 
                return Err(
                    PestError::new_from_span(
                        ErrorVariant::ParsingError { 
                            positives: vec![], 
                            negatives: vec![pair.as_rule()] 
                        }, 
                        pair.as_span()
                    )
                );
            };

        Ok(Type::GenericType {
            name: typename,
            types,
        })
    } else {
        Ok(Type::SimpleType(typename))
    }
}

fn parse_turbofish_type(
    pair: &Pair<Rule>,
    rstack: &mut Vec<Rule>,
) -> Result<Type, PestError<Rule>> {
    todo!()
}

fn parse_closure_type(pair: &Pair<Rule>, rstack: &mut Vec<Rule>) -> Result<Type, PestError<Rule>> {
    todo!()
}
