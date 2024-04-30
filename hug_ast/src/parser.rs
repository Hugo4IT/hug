use std::{collections::HashMap, vec::IntoIter};

use hug_lexer::{
    parser::TokenPair,
    tokenizer::{AnnotationKind, KeywordKind, LiteralKind, TokenKind, TypeKind},
    FilterUseless,
};
use hug_lib::{function::HugFunctionArgument, value::HugValue, Ident};

use crate::{scope::HugScope, Expression, HugTree, HugTreeEntry};

pub trait TypedDefinition {
    fn parse_from_type(_type: TypeKind, value: String) -> Self;
}

impl TypedDefinition for HugValue {
    fn parse_from_type(_type: TypeKind, value: String) -> Self {
        match _type {
            TypeKind::Int8 => HugValue::from(
                value
                    .parse::<i8>()
                    .unwrap_or_else(|_| panic!("Cannot parse Int8 from {}", value)),
            ),
            TypeKind::Int16 => HugValue::from(
                value
                    .parse::<i16>()
                    .unwrap_or_else(|_| panic!("Cannot parse Int16 from {}", value)),
            ),
            TypeKind::Int32 => HugValue::from(
                value
                    .parse::<i32>()
                    .unwrap_or_else(|_| panic!("Cannot parse Int32 from {}", value)),
            ),
            TypeKind::Int64 => HugValue::from(
                value
                    .parse::<i64>()
                    .unwrap_or_else(|_| panic!("Cannot parse Int64 from {}", value)),
            ),
            TypeKind::Int128 => HugValue::from(
                value
                    .parse::<i128>()
                    .unwrap_or_else(|_| panic!("Cannot parse Int128 from {}", value)),
            ),
            TypeKind::UInt8 => HugValue::from(
                value
                    .parse::<u8>()
                    .unwrap_or_else(|_| panic!("Cannot parse UInt8 from {}", value)),
            ),
            TypeKind::UInt16 => HugValue::from(
                value
                    .parse::<u16>()
                    .unwrap_or_else(|_| panic!("Cannot parse UInt16 from {}", value)),
            ),
            TypeKind::UInt32 => HugValue::from(
                value
                    .parse::<u32>()
                    .unwrap_or_else(|_| panic!("Cannot parse UInt32 from {}", value)),
            ),
            TypeKind::UInt64 => HugValue::from(
                value
                    .parse::<u64>()
                    .unwrap_or_else(|_| panic!("Cannot parse UInt64 from {}", value)),
            ),
            TypeKind::UInt128 => HugValue::from(
                value
                    .parse::<u128>()
                    .unwrap_or_else(|_| panic!("Cannot parse UInt128 from {}", value)),
            ),
            TypeKind::Float32 => HugValue::from(
                value
                    .parse::<f32>()
                    .unwrap_or_else(|_| panic!("Cannot parse Float32 from {}", value)),
            ),
            TypeKind::Float64 => HugValue::from(
                value
                    .parse::<f64>()
                    .unwrap_or_else(|_| panic!("Cannot parse Float64 from {}", value)),
            ),
            TypeKind::String => HugValue::from(value[1..(value.len() - 1)].to_string()),
            TypeKind::Other(_) => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct HugTreeAnnotationState {
    is_extern: bool,
    extern_location: String,
    custom: HashMap<Ident, HashMap<String, (LiteralKind, String)>>,
}

impl HugTreeAnnotationState {
    pub fn new() -> HugTreeAnnotationState {
        HugTreeAnnotationState {
            is_extern: false,
            extern_location: String::new(),
            custom: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.is_extern = false;
        self.extern_location.clear();
        self.custom.clear();
    }

    #[inline]
    pub fn push_custom(&mut self, key: Ident, value: HashMap<String, (LiteralKind, String)>) {
        self.custom.insert(key, value).unwrap();
    }

    #[inline]
    pub fn get_custom(&mut self, key: Ident) -> Option<&HashMap<String, (LiteralKind, String)>> {
        self.custom.get(&key)
    }

    pub fn set_extern(&mut self, location: String) {
        self.is_extern = true;
        self.extern_location = location;
    }

    pub fn get_extern(&self) -> Option<String> {
        if self.is_extern {
            if !self.extern_location.is_empty() {
                Some(self.extern_location.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct HugTreeParser {
    tree: HugTree,
    pairs: IntoIter<TokenPair>,
    annotation_state: HugTreeAnnotationState,
}

impl HugTreeParser {
    pub fn new(pairs: Vec<TokenPair>) -> HugTreeParser {
        HugTreeParser {
            annotation_state: HugTreeAnnotationState::new(),
            pairs: pairs.filter_useless().into_iter(),
            tree: HugTree::new(),
        }
    }

    pub fn next(&mut self) -> TokenPair {
        self.pairs.next().unwrap_or_else(TokenPair::null)
    }

    pub fn peek_next(&self) -> TokenPair {
        self.pairs.clone().next().unwrap_or_else(TokenPair::null)
    }

    pub fn peek_next_is(&self, kind: TokenKind) -> bool {
        self.peek_next().token.kind == kind
    }

    pub fn annotation(&mut self, kind: AnnotationKind) -> bool {
        self.next();

        let mut vars: HashMap<String, (LiteralKind, String)> = HashMap::new();

        if self.peek_next_is(TokenKind::OpenParenthesis) {
            self.next(); // (

            loop {
                let name_pair = self.next();
                name_pair.token.kind.expect_ident().unwrap();

                let name = name_pair.text;

                self.next()
                    .token
                    .kind
                    .expect_kind(TokenKind::Assign)
                    .unwrap();

                let value_pair = self.next();
                let value_kind = value_pair.token.kind.expect_literal().unwrap();
                let value = value_pair.text;
                let value = value[1..value.len() - 1].to_string();

                vars.insert(name, (value_kind, value));

                if self.next().token.kind == TokenKind::CloseParenthesis {
                    break;
                }
            }
        }

        if vars.keys().len() > 0 {
            match kind {
                AnnotationKind::Extern => self
                    .annotation_state
                    .set_extern(vars.remove("location").unwrap().1),
                AnnotationKind::Other(id) => self.annotation_state.push_custom(id, vars),
            }
        } else {
            match kind {
                AnnotationKind::Extern => self.annotation_state.set_extern("".to_string()),
                AnnotationKind::Other(id) => self.annotation_state.push_custom(id, vars),
            }
        }

        true // An annotation isn't an AST entry by itself, it supports the following entry
    }

    pub fn parse_argument_list(&mut self) -> Vec<HugFunctionArgument> {
        self.next()
            .token
            .kind
            .expect_kind(TokenKind::OpenParenthesis)
            .expect("Expected (");

        if self.peek_next_is(TokenKind::CloseParenthesis) {
            self.next();

            return Vec::with_capacity(0);
        }

        let mut arguments = Vec::new();

        while !self.peek_next_is(TokenKind::CloseParenthesis) {
            let ident = self
                .next()
                .token
                .kind
                .expect_ident()
                .expect("Expected identifier");

            let _type = if self.peek_next_is(TokenKind::Colon) {
                self.next();

                Some(self.next().token.kind.expect_type().unwrap())
            } else {
                None
            };

            let default_value = if self.peek_next_is(TokenKind::Assign) {
                self.next();

                let expression = self.expression();

                if !expression.is_constant() {
                    panic!("Invalid default value for argument, must be constant");
                }

                expression.get_constant_value()
            } else {
                None
            };

            arguments.push(HugFunctionArgument {
                ident,
                default_value,
            });

            match self.peek_next().token.kind {
                TokenKind::Comma => {
                    self.next();
                }
                TokenKind::CloseParenthesis => (),
                _ => {
                    panic!("Syntax error.");
                }
            }
        }

        self.next();

        arguments
    }

    pub fn keyword(&mut self, scope: &mut HugScope, kind: KeywordKind) -> bool {
        self.next();

        match kind {
            // KeywordKind::Enum => todo!(),
            KeywordKind::Fn => {
                let ident = self
                    .next()
                    .token
                    .kind
                    .expect_ident()
                    .expect("Expected identifier");

                let arguments = self.parse_argument_list();

                if self.peek_next_is(TokenKind::Arrow) {
                    self.next();

                    let return_type = self.next().token.kind.expect_type().expect("Expected type");
                }

                let function_body = self.scope();

                let ident = scope.idents.ident(&ident);
                scope
                    .members
                    .set(ident, HugValue::Function)
                    .push(HugTreeEntry::FunctionDefinition { ident, arguments });

                true
            }
            KeywordKind::Let => self.variable_definition(),
            KeywordKind::Module => {
                if let Some(location) = self.annotation_state.get_extern() {
                    let module = self.next().token.kind.expect_ident().unwrap();

                    self.tree
                        .entries
                        .push(HugTreeEntry::ExternalModuleDefinition { location, module });

                    true
                } else {
                    todo!() // TODO: Non-@extern modules not implemented yet.
                }
            }
            // TODO: KeywordKind::Private => todo!(),
            // TODO: KeywordKind::Public => todo!(),
            KeywordKind::Type => {
                if self.annotation_state.is_extern {
                    let _type = self.next().token.kind.expect_ident().unwrap();

                    self.tree
                        .entries
                        .push(HugTreeEntry::ExternalTypeDefinition { _type });

                    true
                } else {
                    todo!() // TODO: Write non-extern type
                }
            }
            KeywordKind::Use => {
                let mut path = Vec::new();
                path.push(self.next().token.kind.expect_ident().unwrap());

                while self.peek_next_is(TokenKind::Dot) {
                    self.next(); // .

                    path.push(self.next().token.kind.expect_ident().unwrap());
                }

                self.tree.entries.push(HugTreeEntry::Import { path });

                true
            }
            KeywordKind::Return => {
                self.tree.entries.push(self.expression());
            }
            _ => false,
        }
    }

    pub fn identifier(&mut self, _id: Ident) -> bool {
        let next = self.peek_next();

        match next.token.kind {
            TokenKind::Assign => {
                // TODO: Assigning values to existing variables
                todo!()
            }
            _ => {
                let expression = self.expression();

                self.tree.entries.push(HugTreeEntry::Expression(expression));

                true
            }
        }
    }

    pub fn variable_definition(&mut self) -> bool {
        let name = self.next();
        let name = name.token.kind.expect_ident().unwrap();

        let next = self.next();

        match next.token.kind {
            TokenKind::Assign => {
                let value = self.expression();

                self.tree.entries.push(HugTreeEntry::VariableDefinition {
                    variable: name,
                    value,
                });

                true
            }
            TokenKind::Colon => {
                let _type = self.next();
                let _type = _type.token.kind.expect_type().unwrap();

                self.next()
                    .token
                    .kind
                    .expect_kind(TokenKind::Assign)
                    .unwrap();

                let value = self.expression();

                // let value = self.next().unwrap().text;
                // let value = HugValue::parse_from_type(_type, value);
                self.tree.entries.push(HugTreeEntry::VariableDefinition {
                    variable: name,
                    value,
                });

                true
            }
            _ => panic!("Unexpected token at variable definition: {:?}", next),
        }
    }

    pub fn expression(&mut self) -> Expression {
        match self.peek_next().token.kind {
            TokenKind::Literal(_) => Expression::Literal(self.next().parse_literal().unwrap()),
            TokenKind::Identifier(ident) => {
                self.next();

                match self.peek_next().token.kind {
                    TokenKind::Dot => {
                        // TODO: Accessing fields
                        todo!()
                    }
                    TokenKind::OpenParenthesis => {
                        self.next();

                        let mut args = Vec::new();

                        while !matches!(self.peek_next().token.kind, TokenKind::CloseParenthesis) {
                            args.push(self.expression());

                            match self.peek_next().token.kind {
                                TokenKind::Comma => {
                                    self.next();
                                }
                                TokenKind::CloseParenthesis => (),
                                _ => {
                                    panic!("Syntax error.");
                                }
                            }
                        }

                        self.next();

                        Expression::Call {
                            function: ident,
                            args,
                        }
                    }
                    _ => Expression::Variable(ident),
                }
            }
            other => panic!("Invalid expression {other:?}"),
        }
    }

    pub fn visit_next_pair(&mut self, scope: &mut HugScope) -> bool {
        let pair = self.peek_next();

        match pair.token.kind {
            // TokenKind::Literal(_) => todo!(),
            TokenKind::Keyword(kind) => self.keyword(scope, kind),
            TokenKind::Identifier(id) => self.identifier(id),
            TokenKind::Annotation(kind) => self.annotation(kind),
            // TokenKind::Dot => todo!(),
            // TokenKind::OpenParenthesis => todo!(),
            // TokenKind::CloseParenthesis => todo!(),
            // TokenKind::OpenBrace => todo!(),
            // TokenKind::CloseBrace => todo!(),
            // TokenKind::OpenBracket => todo!(),
            // TokenKind::CloseBracket => todo!(),
            // TokenKind::Colon => todo!(),
            // TokenKind::Assign => todo!(),
            // TokenKind::Add => todo!(),
            // TokenKind::Subtract => todo!(),
            // TokenKind::Multiply => todo!(),
            // TokenKind::Divide => todo!(),
            // TokenKind::Modulus => todo!(),
            // TokenKind::AddAssign => todo!(),
            // TokenKind::SubtractAssign => todo!(),
            // TokenKind::MultiplyAssign => todo!(),
            // TokenKind::DivideAssign => todo!(),
            // TokenKind::ModulusAssign => todo!(),
            // TokenKind::Not => todo!(),
            // TokenKind::And => todo!(),
            // TokenKind::Or => todo!(),
            // TokenKind::IsEqualTo => todo!(),
            // TokenKind::IsNotEqualTo => todo!(),
            // TokenKind::LessThan => todo!(),
            // TokenKind::GreaterThan => todo!(),
            // TokenKind::LessThanOrEquals => todo!(),
            // TokenKind::GreaterThanOrEquals => todo!(),
            // TokenKind::BinaryAnd => todo!(),
            // TokenKind::BinaryOr => todo!(),
            // TokenKind::BinaryNot => todo!(),
            // TokenKind::BinaryXOr => todo!(),
            // TokenKind::BinaryAndAssign => todo!(),
            // TokenKind::BinaryOrAssign => todo!(),
            // TokenKind::BinaryNotAssign => todo!(),
            // TokenKind::BinaryXOrAssign => todo!(),
            // TokenKind::ShiftLeft => todo!(),
            // TokenKind::ShiftRight => todo!(),
            // TokenKind::ShiftLeftOverflow => todo!(),
            // TokenKind::ShiftRightOverflow => todo!(),
            TokenKind::LineComment | TokenKind::BlockComment => {
                self.next();

                true
            }
            TokenKind::Unknown => panic!("Unknown token: {}!", pair.text),
            _ => false,
        }
    }

    pub fn scope(&mut self) -> HugScope {
        self.next()
            .token
            .kind
            .expect_kind(TokenKind::OpenBrace)
            .unwrap(); // {

        let mut scope = HugScope::new();

        while !self.peek_next_is(TokenKind::CloseBrace) {
            if !self.visit_next_pair(&mut scope) {
                panic!("Syntax error");
            }
        }

        self.next(); // }

        scope
    }

    pub fn parse(mut self) -> HugTree {
        self.annotation_state.reset();

        while !self.pairs.as_slice().is_empty() {
            self.visit_next_pair(&mut self.tree.root);
        }

        self.tree
    }
}
