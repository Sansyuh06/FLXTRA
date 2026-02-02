//! JavaScript runtime

use crate::ast::*;
use crate::builtins;
use crate::context::ExecutionContext;
use crate::parser::Parser;
use crate::value::{FunctionBody, JsFunction, JsValue};
use std::sync::Arc;
use tracing::debug;

pub struct JsRuntime {
    ctx: ExecutionContext,
    timeout_ms: u64,
}

impl JsRuntime {
    pub fn new(timeout_ms: u64) -> Self {
        let mut runtime = Self {
            ctx: ExecutionContext::new(),
            timeout_ms,
        };
        runtime.init_globals();
        runtime
    }

    fn init_globals(&mut self) {
        // Console
        self.ctx.set_global("console", JsValue::object());
        
        // Global functions
        self.register_native("parseInt", builtins::parse_int);
        self.register_native("parseFloat", builtins::parse_float);
        self.register_native("isNaN", builtins::is_nan);
        self.register_native("isFinite", builtins::is_finite);
    }

    fn register_native(&mut self, name: &str, func: fn(Vec<JsValue>) -> JsValue) {
        let js_func = JsFunction::native(name, func);
        self.ctx.set_global(name, JsValue::Function(Arc::new(js_func)));
    }

    pub fn execute(&mut self, source: &str) -> JsValue {
        debug!("Executing JS ({} bytes)", source.len());
        let mut parser = Parser::new(source);
        let ast = parser.parse();
        self.eval_node(&ast)
    }

    fn eval_node(&mut self, node: &AstNode) -> JsValue {
        match node {
            AstNode::Program(stmts) => {
                let mut result = JsValue::Undefined;
                for stmt in stmts { result = self.eval_node(stmt); }
                result
            }
            AstNode::Literal(lit) => self.eval_literal(lit),
            AstNode::Identifier(name) => self.ctx.get(name),
            AstNode::BinaryExpression { operator, left, right } => {
                self.eval_binary(*operator, left, right)
            }
            AstNode::UnaryExpression { operator, argument, .. } => {
                self.eval_unary(*operator, argument)
            }
            AstNode::AssignmentExpression { operator, left, right } => {
                self.eval_assignment(*operator, left, right)
            }
            AstNode::VariableDeclaration { declarations, .. } => {
                for decl in declarations {
                    let value = decl.init.as_ref().map(|e| self.eval_node(e)).unwrap_or(JsValue::Undefined);
                    self.ctx.declare(&decl.name, value);
                }
                JsValue::Undefined
            }
            AstNode::BlockStatement(stmts) => {
                self.ctx.enter_scope();
                let mut result = JsValue::Undefined;
                for stmt in stmts { result = self.eval_node(stmt); }
                self.ctx.exit_scope();
                result
            }
            AstNode::ExpressionStatement(expr) => self.eval_node(expr),
            AstNode::IfStatement { test, consequent, alternate } => {
                if self.eval_node(test).to_boolean() {
                    self.eval_node(consequent)
                } else if let Some(alt) = alternate {
                    self.eval_node(alt)
                } else {
                    JsValue::Undefined
                }
            }
            AstNode::WhileStatement { test, body } => {
                while self.eval_node(test).to_boolean() { self.eval_node(body); }
                JsValue::Undefined
            }
            AstNode::ReturnStatement(arg) => {
                arg.as_ref().map(|e| self.eval_node(e)).unwrap_or(JsValue::Undefined)
            }
            AstNode::CallExpression { callee, arguments } => {
                self.eval_call(callee, arguments)
            }
            AstNode::ArrayExpression(elements) => {
                let values: Vec<JsValue> = elements.iter().map(|e| self.eval_node(e)).collect();
                JsValue::array(values)
            }
            AstNode::LogicalExpression { operator, left, right } => {
                let left_val = self.eval_node(left);
                match operator {
                    LogicalOp::And => if left_val.to_boolean() { self.eval_node(right) } else { left_val },
                    LogicalOp::Or => if left_val.to_boolean() { left_val } else { self.eval_node(right) },
                    LogicalOp::NullCoalesce => {
                        if matches!(left_val, JsValue::Null | JsValue::Undefined) {
                            self.eval_node(right)
                        } else { left_val }
                    }
                }
            }
            AstNode::ConditionalExpression { test, consequent, alternate } => {
                if self.eval_node(test).to_boolean() { self.eval_node(consequent) }
                else { self.eval_node(alternate) }
            }
            _ => JsValue::Undefined,
        }
    }

    fn eval_literal(&self, lit: &Literal) -> JsValue {
        match lit {
            Literal::Number(n) => JsValue::Number(*n),
            Literal::String(s) => JsValue::String(s.clone()),
            Literal::Boolean(b) => JsValue::Boolean(*b),
            Literal::Null => JsValue::Null,
            Literal::Undefined => JsValue::Undefined,
            _ => JsValue::Undefined,
        }
    }

    fn eval_binary(&mut self, op: BinaryOp, left: &AstNode, right: &AstNode) -> JsValue {
        let l = self.eval_node(left);
        let r = self.eval_node(right);
        match op {
            BinaryOp::Add => {
                if matches!(&l, JsValue::String(_)) || matches!(&r, JsValue::String(_)) {
                    JsValue::String(format!("{}{}", l.to_string(), r.to_string()))
                } else { JsValue::Number(l.to_number() + r.to_number()) }
            }
            BinaryOp::Sub => JsValue::Number(l.to_number() - r.to_number()),
            BinaryOp::Mul => JsValue::Number(l.to_number() * r.to_number()),
            BinaryOp::Div => JsValue::Number(l.to_number() / r.to_number()),
            BinaryOp::Mod => JsValue::Number(l.to_number() % r.to_number()),
            BinaryOp::Lt => JsValue::Boolean(l.to_number() < r.to_number()),
            BinaryOp::LtEq => JsValue::Boolean(l.to_number() <= r.to_number()),
            BinaryOp::Gt => JsValue::Boolean(l.to_number() > r.to_number()),
            BinaryOp::GtEq => JsValue::Boolean(l.to_number() >= r.to_number()),
            BinaryOp::StrictEq => JsValue::Boolean(l.strict_equals(&r)),
            BinaryOp::StrictNotEq => JsValue::Boolean(!l.strict_equals(&r)),
            _ => JsValue::Undefined,
        }
    }

    fn eval_unary(&mut self, op: UnaryOp, arg: &AstNode) -> JsValue {
        let val = self.eval_node(arg);
        match op {
            UnaryOp::Neg => JsValue::Number(-val.to_number()),
            UnaryOp::Pos => JsValue::Number(val.to_number()),
            UnaryOp::Not => JsValue::Boolean(!val.to_boolean()),
            UnaryOp::TypeOf => JsValue::String(val.type_of().to_string()),
            _ => JsValue::Undefined,
        }
    }

    fn eval_assignment(&mut self, _op: AssignOp, left: &AstNode, right: &AstNode) -> JsValue {
        let value = self.eval_node(right);
        if let AstNode::Identifier(name) = left {
            self.ctx.assign(name, value.clone());
        }
        value
    }

    fn eval_call(&mut self, callee: &AstNode, args: &[AstNode]) -> JsValue {
        let func = self.eval_node(callee);
        let arg_values: Vec<JsValue> = args.iter().map(|a| self.eval_node(a)).collect();
        
        if let JsValue::Function(f) = func {
            match &f.body {
                FunctionBody::Native(native_fn) => native_fn(arg_values),
                FunctionBody::Interpreted(_) => JsValue::Undefined,
            }
        } else { JsValue::Undefined }
    }
}
