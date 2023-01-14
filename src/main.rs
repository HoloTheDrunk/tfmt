use clap::Parser as CParser;
use pest::{iterators::Pair, pratt_parser::PrattParser, Parser};

#[derive(CParser, Debug)]
#[command(author, version, about, long_about = None)]
/// Rust type prettifier
struct Args {
    /// Type to prettify
    input: String,
}

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct TParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::Op;
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            .op(Op::prefix(type_tuple))
    };
}

enum Type {
    SimpleType(String),
    Tuple(Vec<Type>),
    AsType {
        name: String,
        types: Vec<Type>,
        target: Box<Type>,
    },
    GenericType {
        name: String,
        types: Vec<Type>,
    },
}

struct TypeExpr {
    lifetime: Option<String>,
    impl_marker: Option<String>,
    r#type: Type,
}

fn main() {
    let args = Args::parse();

    let parsed = TParser::parse(Rule::ast, args.input.as_ref())
        .expect("Unsuccessful parse")
        .next()
        .unwrap();

    recursive_print(Some(&parsed), 0);
}

fn recursive_print(cur: Option<&Pair<Rule>>, depth: u8) {
    if let Some(node) = cur {
        let rule = node.as_rule();

        let indent = (0..depth)
            .map(|_| "\x1b[32m|   \x1b[0m")
            .collect::<String>();

        println!(
            "{}\x1b[1;33m{:?}\x1b[0m:'{}'",
            indent,
            rule,
            node.as_span()
                .as_str()
                .lines()
                .map(|line| line.trim())
                .collect::<String>()
        );

        for pair in node.clone().into_inner() {
            recursive_print(Some(&pair), depth + 1);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use paste::paste;

    macro_rules! test {
        ($name:ident, $input:literal, $rule:ident, $check:ident) => {
            #[test]
            fn $name() {
                paste! {
                    assert!(
                        TParser::parse(Rule::$rule, $input).[<is_ $check>](),
                        "Unsuccessful parse"
                    );
                }
            }
        };

        ($name:ident, $input:literal, $check:ident) => {
            test!($name, $input, ast, $check);
        };

        ($name:ident, $input:literal) => {
            test!($name, $input, ast, ok);
        };
    }

    test!(simple, "String");
    test!(lifetime_simple, "&String");
    test!(generic, "Vec<String>");
    test!(generic_lifetime, "Vec<&String>");
    test!(lifetime_generic_lifetime, "&Vec<&String>");
    test!(generic_tuple_simple, "Result<String, (String, String)>");
    test!(
        complex,
        "MyStruct<<&'_ Thing<&'s str>>::String as Other<String>, ()>"
    );
}
