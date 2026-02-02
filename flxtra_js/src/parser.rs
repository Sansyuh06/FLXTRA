//! JavaScript parser

use crate::ast::*;
use crate::lexer::{Lexer, Token};
use tracing::debug;

/// JavaScript parser
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        debug!("Parsed {} tokens", tokens.len());
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        let token = self.current().clone();
        self.pos += 1;
        self.tokens.get(self.pos - 1).unwrap_or(&Token::Eof)
    }

    fn expect(&mut self, expected: Token) -> bool {
        if std::mem::discriminant(self.current()) == std::mem::discriminant(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn matches(&self, token: &Token) -> bool {
        std::mem::discriminant(self.current()) == std::mem::discriminant(token)
    }

    pub fn parse(&mut self) -> AstNode {
        let mut statements = Vec::new();
        
        while !matches!(self.current(), Token::Eof) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
        }
        
        AstNode::Program(statements)
    }

    fn parse_statement(&mut self) -> Option<AstNode> {
        match self.current() {
            Token::Var | Token::Let | Token::Const => self.parse_variable_declaration(),
            Token::Function => self.parse_function_declaration(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            Token::For => self.parse_for_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Break => {
                self.advance();
                self.expect(Token::Semicolon);
                Some(AstNode::BreakStatement)
            }
            Token::Continue => {
                self.advance();
                self.expect(Token::Semicolon);
                Some(AstNode::ContinueStatement)
            }
            Token::LBrace => self.parse_block_statement(),
            Token::Semicolon => {
                self.advance();
                Some(AstNode::EmptyStatement)
            }
            Token::Try => self.parse_try_statement(),
            Token::Throw => self.parse_throw_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_variable_declaration(&mut self) -> Option<AstNode> {
        let kind = match self.current() {
            Token::Var => VarKind::Var,
            Token::Let => VarKind::Let,
            Token::Const => VarKind::Const,
            _ => return None,
        };
        self.advance();

        let mut declarations = Vec::new();

        loop {
            if let Token::Identifier(name) = self.current().clone() {
                self.advance();
                
                let init = if self.matches(&Token::Eq) {
                    self.advance();
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };

                declarations.push(VariableDeclarator { name, init });

                if !self.matches(&Token::Comma) {
                    break;
                }
                self.advance();
            } else {
                break;
            }
        }

        self.expect(Token::Semicolon);
        Some(AstNode::VariableDeclaration { kind, declarations })
    }

    fn parse_function_declaration(&mut self) -> Option<AstNode> {
        self.advance(); // function

        let name = if let Token::Identifier(n) = self.current().clone() {
            self.advance();
            n
        } else {
            return None;
        };

        self.expect(Token::LParen);
        let params = self.parse_params();
        self.expect(Token::RParen);

        let body = self.parse_block_statement()?;

        Some(AstNode::FunctionDeclaration {
            name,
            params,
            body: Box::new(body),
        })
    }

    fn parse_params(&mut self) -> Vec<String> {
        let mut params = Vec::new();

        while let Token::Identifier(name) = self.current().clone() {
            params.push(name);
            self.advance();
            
            if !self.matches(&Token::Comma) {
                break;
            }
            self.advance();
        }

        params
    }

    fn parse_block_statement(&mut self) -> Option<AstNode> {
        self.expect(Token::LBrace);
        
        let mut statements = Vec::new();
        while !self.matches(&Token::RBrace) && !matches!(self.current(), Token::Eof) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
        }
        
        self.expect(Token::RBrace);
        Some(AstNode::BlockStatement(statements))
    }

    fn parse_if_statement(&mut self) -> Option<AstNode> {
        self.advance(); // if
        self.expect(Token::LParen);
        let test = self.parse_expression()?;
        self.expect(Token::RParen);
        
        let consequent = self.parse_statement()?;
        
        let alternate = if self.matches(&Token::Else) {
            self.advance();
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Some(AstNode::IfStatement {
            test: Box::new(test),
            consequent: Box::new(consequent),
            alternate,
        })
    }

    fn parse_while_statement(&mut self) -> Option<AstNode> {
        self.advance(); // while
        self.expect(Token::LParen);
        let test = self.parse_expression()?;
        self.expect(Token::RParen);
        let body = self.parse_statement()?;

        Some(AstNode::WhileStatement {
            test: Box::new(test),
            body: Box::new(body),
        })
    }

    fn parse_for_statement(&mut self) -> Option<AstNode> {
        self.advance(); // for
        self.expect(Token::LParen);

        let init = if !self.matches(&Token::Semicolon) {
            if matches!(self.current(), Token::Var | Token::Let | Token::Const) {
                self.parse_variable_declaration().map(Box::new)
            } else {
                Some(Box::new(self.parse_expression()?))
            }
        } else {
            self.advance();
            None
        };

        if !matches!(self.tokens.get(self.pos - 1), Some(Token::Semicolon)) {
            self.expect(Token::Semicolon);
        }

        let test = if !self.matches(&Token::Semicolon) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        self.expect(Token::Semicolon);

        let update = if !self.matches(&Token::RParen) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        self.expect(Token::RParen);

        let body = self.parse_statement()?;

        Some(AstNode::ForStatement {
            init,
            test,
            update,
            body: Box::new(body),
        })
    }

    fn parse_return_statement(&mut self) -> Option<AstNode> {
        self.advance(); // return

        let argument = if !self.matches(&Token::Semicolon) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.expect(Token::Semicolon);
        Some(AstNode::ReturnStatement(argument))
    }

    fn parse_try_statement(&mut self) -> Option<AstNode> {
        self.advance(); // try
        let block = self.parse_block_statement()?;

        let handler = if self.matches(&Token::Catch) {
            self.advance();
            let param = if self.matches(&Token::LParen) {
                self.advance();
                let name = if let Token::Identifier(n) = self.current().clone() {
                    self.advance();
                    Some(n)
                } else {
                    None
                };
                self.expect(Token::RParen);
                name
            } else {
                None
            };
            let body = self.parse_block_statement()?;
            Some(CatchClause {
                param,
                body: Box::new(body),
            })
        } else {
            None
        };

        let finalizer = if self.matches(&Token::Finally) {
            self.advance();
            Some(Box::new(self.parse_block_statement()?))
        } else {
            None
        };

        Some(AstNode::TryStatement {
            block: Box::new(block),
            handler,
            finalizer,
        })
    }

    fn parse_throw_statement(&mut self) -> Option<AstNode> {
        self.advance(); // throw
        let argument = self.parse_expression()?;
        self.expect(Token::Semicolon);
        Some(AstNode::ThrowStatement(Box::new(argument)))
    }

    fn parse_expression_statement(&mut self) -> Option<AstNode> {
        let expr = self.parse_expression()?;
        self.expect(Token::Semicolon);
        Some(AstNode::ExpressionStatement(Box::new(expr)))
    }

    fn parse_expression(&mut self) -> Option<AstNode> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Option<AstNode> {
        let left = self.parse_ternary()?;

        let op = match self.current() {
            Token::Eq => AssignOp::Assign,
            Token::PlusEq => AssignOp::AddAssign,
            Token::MinusEq => AssignOp::SubAssign,
            Token::StarEq => AssignOp::MulAssign,
            Token::SlashEq => AssignOp::DivAssign,
            _ => return Some(left),
        };

        self.advance();
        let right = self.parse_assignment()?;

        Some(AstNode::AssignmentExpression {
            operator: op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_ternary(&mut self) -> Option<AstNode> {
        let test = self.parse_logical_or()?;

        if self.matches(&Token::Question) {
            self.advance();
            let consequent = self.parse_assignment()?;
            self.expect(Token::Colon);
            let alternate = self.parse_assignment()?;

            return Some(AstNode::ConditionalExpression {
                test: Box::new(test),
                consequent: Box::new(consequent),
                alternate: Box::new(alternate),
            });
        }

        Some(test)
    }

    fn parse_logical_or(&mut self) -> Option<AstNode> {
        let mut left = self.parse_logical_and()?;

        while self.matches(&Token::Or) {
            self.advance();
            let right = self.parse_logical_and()?;
            left = AstNode::LogicalExpression {
                operator: LogicalOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_logical_and(&mut self) -> Option<AstNode> {
        let mut left = self.parse_equality()?;

        while self.matches(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            left = AstNode::LogicalExpression {
                operator: LogicalOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_equality(&mut self) -> Option<AstNode> {
        let mut left = self.parse_comparison()?;

        loop {
            let op = match self.current() {
                Token::EqEq => BinaryOp::Eq,
                Token::EqEqEq => BinaryOp::StrictEq,
                Token::NotEq => BinaryOp::NotEq,
                Token::NotEqEq => BinaryOp::StrictNotEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            left = AstNode::BinaryExpression {
                operator: op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_comparison(&mut self) -> Option<AstNode> {
        let mut left = self.parse_additive()?;

        loop {
            let op = match self.current() {
                Token::Lt => BinaryOp::Lt,
                Token::LtEq => BinaryOp::LtEq,
                Token::Gt => BinaryOp::Gt,
                Token::GtEq => BinaryOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = AstNode::BinaryExpression {
                operator: op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_additive(&mut self) -> Option<AstNode> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.current() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = AstNode::BinaryExpression {
                operator: op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_multiplicative(&mut self) -> Option<AstNode> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.current() {
                Token::Star => BinaryOp::Mul,
                Token::Slash => BinaryOp::Div,
                Token::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = AstNode::BinaryExpression {
                operator: op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Some(left)
    }

    fn parse_unary(&mut self) -> Option<AstNode> {
        let op = match self.current() {
            Token::Not => UnaryOp::Not,
            Token::Minus => UnaryOp::Neg,
            Token::Plus => UnaryOp::Pos,
            Token::Typeof => UnaryOp::TypeOf,
            Token::Void => UnaryOp::Void,
            Token::Delete => UnaryOp::Delete,
            _ => return self.parse_update(),
        };

        self.advance();
        let argument = self.parse_unary()?;

        Some(AstNode::UnaryExpression {
            operator: op,
            argument: Box::new(argument),
            prefix: true,
        })
    }

    fn parse_update(&mut self) -> Option<AstNode> {
        // Prefix update
        if matches!(self.current(), Token::PlusPlus | Token::MinusMinus) {
            let op = match self.current() {
                Token::PlusPlus => UpdateOp::Increment,
                Token::MinusMinus => UpdateOp::Decrement,
                _ => unreachable!(),
            };
            self.advance();
            let argument = self.parse_call()?;
            return Some(AstNode::UpdateExpression {
                operator: op,
                argument: Box::new(argument),
                prefix: true,
            });
        }

        let expr = self.parse_call()?;

        // Postfix update
        if matches!(self.current(), Token::PlusPlus | Token::MinusMinus) {
            let op = match self.current() {
                Token::PlusPlus => UpdateOp::Increment,
                Token::MinusMinus => UpdateOp::Decrement,
                _ => unreachable!(),
            };
            self.advance();
            return Some(AstNode::UpdateExpression {
                operator: op,
                argument: Box::new(expr),
                prefix: false,
            });
        }

        Some(expr)
    }

    fn parse_call(&mut self) -> Option<AstNode> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.matches(&Token::LParen) {
                self.advance();
                let arguments = self.parse_arguments();
                self.expect(Token::RParen);
                expr = AstNode::CallExpression {
                    callee: Box::new(expr),
                    arguments,
                };
            } else if self.matches(&Token::Dot) {
                self.advance();
                if let Token::Identifier(name) = self.current().clone() {
                    self.advance();
                    expr = AstNode::MemberExpression {
                        object: Box::new(expr),
                        property: Box::new(AstNode::Identifier(name)),
                        computed: false,
                    };
                }
            } else if self.matches(&Token::LBracket) {
                self.advance();
                let property = self.parse_expression()?;
                self.expect(Token::RBracket);
                expr = AstNode::MemberExpression {
                    object: Box::new(expr),
                    property: Box::new(property),
                    computed: true,
                };
            } else {
                break;
            }
        }

        Some(expr)
    }

    fn parse_arguments(&mut self) -> Vec<AstNode> {
        let mut args = Vec::new();

        if !self.matches(&Token::RParen) {
            if let Some(arg) = self.parse_expression() {
                args.push(arg);
            }

            while self.matches(&Token::Comma) {
                self.advance();
                if let Some(arg) = self.parse_expression() {
                    args.push(arg);
                }
            }
        }

        args
    }

    fn parse_primary(&mut self) -> Option<AstNode> {
        match self.current().clone() {
            Token::Number(n) => {
                self.advance();
                Some(AstNode::Literal(Literal::Number(n)))
            }
            Token::String(s) => {
                self.advance();
                Some(AstNode::Literal(Literal::String(s)))
            }
            Token::Boolean(b) => {
                self.advance();
                Some(AstNode::Literal(Literal::Boolean(b)))
            }
            Token::Null => {
                self.advance();
                Some(AstNode::Literal(Literal::Null))
            }
            Token::Undefined => {
                self.advance();
                Some(AstNode::Literal(Literal::Undefined))
            }
            Token::Identifier(name) => {
                self.advance();
                Some(AstNode::Identifier(name))
            }
            Token::This => {
                self.advance();
                Some(AstNode::ThisExpression)
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen);
                Some(expr)
            }
            Token::LBracket => self.parse_array(),
            Token::LBrace => self.parse_object(),
            Token::Function => self.parse_function_expression(),
            Token::New => self.parse_new_expression(),
            _ => None,
        }
    }

    fn parse_array(&mut self) -> Option<AstNode> {
        self.advance(); // [
        let mut elements = Vec::new();

        while !self.matches(&Token::RBracket) && !matches!(self.current(), Token::Eof) {
            if let Some(elem) = self.parse_expression() {
                elements.push(elem);
            }
            if !self.matches(&Token::Comma) {
                break;
            }
            self.advance();
        }

        self.expect(Token::RBracket);
        Some(AstNode::ArrayExpression(elements))
    }

    fn parse_object(&mut self) -> Option<AstNode> {
        self.advance(); // {
        let mut properties = Vec::new();

        while !self.matches(&Token::RBrace) && !matches!(self.current(), Token::Eof) {
            if let Token::Identifier(key) = self.current().clone() {
                self.advance();
                
                if self.matches(&Token::Colon) {
                    self.advance();
                    let value = self.parse_expression()?;
                    properties.push(Property {
                        key,
                        value: Box::new(value),
                        computed: false,
                        shorthand: false,
                    });
                } else {
                    // Shorthand property
                    properties.push(Property {
                        key: key.clone(),
                        value: Box::new(AstNode::Identifier(key)),
                        computed: false,
                        shorthand: true,
                    });
                }
            } else if let Token::String(key) = self.current().clone() {
                self.advance();
                self.expect(Token::Colon);
                let value = self.parse_expression()?;
                properties.push(Property {
                    key,
                    value: Box::new(value),
                    computed: false,
                    shorthand: false,
                });
            }

            if !self.matches(&Token::Comma) {
                break;
            }
            self.advance();
        }

        self.expect(Token::RBrace);
        Some(AstNode::ObjectExpression(properties))
    }

    fn parse_function_expression(&mut self) -> Option<AstNode> {
        self.advance(); // function

        let name = if let Token::Identifier(n) = self.current().clone() {
            self.advance();
            Some(n)
        } else {
            None
        };

        self.expect(Token::LParen);
        let params = self.parse_params();
        self.expect(Token::RParen);
        let body = self.parse_block_statement()?;

        Some(AstNode::FunctionExpression {
            name,
            params,
            body: Box::new(body),
        })
    }

    fn parse_new_expression(&mut self) -> Option<AstNode> {
        self.advance(); // new
        let callee = self.parse_call()?;
        
        // Arguments are parsed in parse_call, but we need to handle `new Foo` without parens
        if let AstNode::CallExpression { callee, arguments } = callee {
            Some(AstNode::NewExpression { callee, arguments })
        } else {
            Some(AstNode::NewExpression {
                callee: Box::new(callee),
                arguments: Vec::new(),
            })
        }
    }
}
