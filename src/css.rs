//! Shared cssparser syntax adapter. Property semantics remain in BoxStyle.
use crate::RenderError;
use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserState,
    QualifiedRuleParser, RuleBodyItemParser, RuleBodyParser, StyleSheetParser, ToCss, Token,
    TokenSerializationType,
};
const MAX_BYTES: usize = 1_048_576;
#[derive(Debug)]
pub(crate) struct Declaration {
    pub name: String,
    pub value: String,
}
pub(crate) struct Rule {
    pub selectors: Vec<String>,
    pub declarations: Vec<Declaration>,
}
struct Declarations;
impl<'i> DeclarationParser<'i> for Declarations {
    type Declaration = Declaration;
    type Error = ();
    fn parse_value(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i>,
        _: &ParserState,
    ) -> Result<Declaration, ParseError<()>> {
        let value = value(input)?;
        if value.is_empty() {
            return Err(ParseError::custom(()));
        }
        let name = if name.starts_with("--") {
            name.to_string()
        } else {
            name.to_ascii_lowercase()
        };
        Ok(Declaration { name, value })
    }
}
impl<'i> AtRuleParser<'i> for Declarations {
    type Prelude = ();
    type AtRule = Declaration;
    type Error = ();
}
impl<'i> QualifiedRuleParser<'i> for Declarations {
    type Prelude = ();
    type QualifiedRule = Declaration;
    type Error = ();
}
impl<'i> RuleBodyItemParser<'i, Declaration, ()> for Declarations {
    fn parse_declarations(&self) -> bool {
        true
    }
    fn parse_qualified(&self) -> bool {
        false
    }
}
fn declarations_from(input: &mut Parser<'_>) -> Result<Vec<Declaration>, ParseError<()>> {
    RuleBodyParser::new(input, &mut Declarations)
        .map(|item| item.map_err(|_| ParseError::custom(())))
        .collect()
}
/// cssparser owns token boundaries, escaping, strings, comments and nested blocks.
/// Whitespace between tokens is canonicalized; numbers retain original spelling
/// to avoid losing integer precision through CSS's floating-point token values.
fn value(input: &mut Parser<'_>) -> Result<String, ParseError<()>> {
    let mut parts = vec![];
    while !input.is_exhausted() {
        let start = input.position();
        let token = input.next_including_whitespace_and_comments()?.clone();
        if token.is_parse_error() {
            return Err(ParseError::custom(()));
        }
        let part = match token {
            Token::WhiteSpace(_) | Token::Comment(_) => continue,
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                let open = token.to_css_string();
                let close = match token {
                    Token::SquareBracketBlock => ']',
                    Token::CurlyBracketBlock => '}',
                    _ => ')',
                };
                let inner = input.parse_nested_block(value)?;
                format!("{open}{inner}{close}")
            }
            Token::Number { .. } | Token::Dimension { .. } | Token::Percentage { .. } => {
                input.slice_from(start).to_string()
            }
            _ => token.to_css_string(),
        };
        parts.push(part);
    }
    Ok(parts.join(" "))
}
struct Rules;
impl<'i> AtRuleParser<'i> for Rules {
    type Prelude = ();
    type AtRule = Rule;
    type Error = ();
}
impl<'i> QualifiedRuleParser<'i> for Rules {
    type Prelude = Vec<String>;
    type QualifiedRule = Rule;
    type Error = ();
    fn parse_prelude(&mut self, input: &mut Parser<'i>) -> Result<Vec<String>, ParseError<()>> {
        input.parse_comma_separated(|input| {
            let mut result = String::new();
            let mut previous = TokenSerializationType::default();
            while !input.is_exhausted() {
                let token = input.next_including_whitespace_and_comments()?.clone();
                if matches!(token, Token::Comment(_)) {
                    continue;
                }
                if token.is_parse_error()
                    || matches!(
                        token,
                        Token::Function(_)
                            | Token::ParenthesisBlock
                            | Token::SquareBracketBlock
                            | Token::CurlyBracketBlock
                    )
                {
                    return Err(ParseError::custom(()));
                }
                let kind = token.serialization_type();
                if previous.needs_separator_when_before(kind) {
                    result.push_str("/**/");
                }
                result.push_str(&token.to_css_string());
                previous = kind;
            }
            let result = result.trim().to_string();
            if result.is_empty() {
                Err(ParseError::custom(()))
            } else {
                Ok(result)
            }
        })
    }
    fn parse_block(
        &mut self,
        selectors: Vec<String>,
        _: &ParserState,
        input: &mut Parser<'i>,
    ) -> Result<Rule, ParseError<()>> {
        Ok(Rule {
            selectors,
            declarations: declarations_from(input)?,
        })
    }
}
fn check(source: &str) -> Result<(), RenderError> {
    if source.len() > MAX_BYTES {
        Err(RenderError("CSS byte limit exceeded"))
    } else {
        Ok(())
    }
}
pub(crate) fn declarations(source: &str) -> Result<Vec<Declaration>, RenderError> {
    check(source)?;
    declarations_from(&mut Parser::new(source))
        .map_err(|_| RenderError("invalid or unsupported CSS declaration"))
}
pub(crate) fn rules(source: &str) -> Result<Vec<Rule>, RenderError> {
    check(source)?;
    StyleSheetParser::new(&mut Parser::new(source), &mut Rules)
        .map(|item| item.map_err(|_| RenderError("invalid or unsupported stylesheet rule")))
        .collect()
}
