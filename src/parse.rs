//! Parsing logic

use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, token, Expr, ExprLet, Ident, Result, Token, Type};

#[derive(Clone)]
pub struct Program {
    pub relations: Vec<Relation>,
    pub rules: Vec<Rule>,
}

impl Parse for Program {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut relations: Vec<Relation> = Vec::new();
        let mut rules: Vec<Rule> = Vec::new();

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Token![@]) || lookahead.peek(Token![struct]) {
                relations.push(input.parse()?);
            } else if lookahead.peek(Ident) {
                rules.push(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(Self { relations, rules })
    }
}

#[derive(Clone)]
pub struct Relation {
    pub attribute: Option<Ident>,
    pub struct_token: Token![struct],
    pub name: Ident,
    pub paren_token: token::Paren,
    pub fields: Punctuated<Type, Token![,]>,
    pub semi_token: Token![;],
}

impl Parse for Relation {
    fn parse(input: ParseStream) -> Result<Self> {
        let attribute = if input.peek(Token![@]) {
            let _: Token![@] = input.parse()?; // @
            Some(input.parse()?) // Ident
        } else {
            None
        };
        let content;
        #[allow(clippy::eval_order_dependence)]
        Ok(Self {
            attribute,
            struct_token: input.parse()?,
            name: input.parse()?,
            paren_token: parenthesized!(content in input),
            fields: content.parse_terminated(Type::parse)?,
            semi_token: input.parse()?,
        })
    }
}

#[derive(Clone)]
pub struct Rule {
    pub goal: Fact,
    pub arrow_token: Token![<-],
    pub clauses: Punctuated<Clause, Token![,]>,
    pub semi_token: Token![;],
}

impl Parse for Rule {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            goal: input.parse()?,
            arrow_token: input.parse()?,
            clauses: Punctuated::parse_separated_nonempty(input)?,
            semi_token: input.parse()?,
        })
    }
}

#[derive(Clone)]
pub enum Clause {
    Fact(Fact),
    Expr(Expr),
    Let(ExprLet),
}

impl Parse for Clause {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Paren) {
            input.parse().map(Clause::Expr)
        } else if lookahead.peek(Token![let]) {
            input.parse().map(Clause::Let)
        } else if lookahead.peek(Ident) || lookahead.peek(Token![!]) {
            input.parse().map(Clause::Fact)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Clone)]
pub struct Fact {
    pub negate: Option<Token![!]>,
    pub relation: Ident,
    pub paren_token: token::Paren,
    pub fields: Punctuated<Option<Expr>, Token![,]>,
}

impl Parse for Fact {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        #[allow(clippy::eval_order_dependence)]
        Ok(Self {
            negate: input.parse()?,
            relation: input.parse()?,
            paren_token: parenthesized!(content in input),
            fields: content.parse_terminated(|input| {
                if input.peek(Token![_]) {
                    let _: Token![_] = input.parse()?;
                    Ok(None)
                } else {
                    input.parse().map(Some)
                }
            })?,
        })
    }
}
