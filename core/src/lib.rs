mod block;
mod expr;
mod lexer;
mod op;
mod stmt;
mod r#type;
mod utils;
mod value;

use crate::utils::expand_global;
use indexmap::IndexMap;
use unicode_xid::UnicodeXID;

pub use {
    block::Block,
    expr::Expr,
    lexer::{is_identifier, str_format, tokenize},
    op::Op,
    stmt::Stmt,
    r#type::{Dict, Enum, Type},
    utils::{BYTES, OPERATOR, RESERVED, SPACE, expand_local},
    value::Value,
};

pub trait Node {
    fn compile(&self, ctx: &mut Compiler) -> Option<String>;
    fn type_infer(&self, ctx: &mut Compiler) -> Option<Type>;
    fn parse(source: &str) -> Option<Self>
    where
        Self: Node + Sized;
}

/// Function includes local variables, arguments, and returns
#[derive(Debug, Clone)]
pub struct Function {
    pub variables: IndexMap<String, Type>,
    pub arguments: IndexMap<String, Type>,
    pub returns: Type,
}

/// Context in compiling
#[derive(Debug, Clone)]
pub struct Compiler {
    /// Address tracker
    pub allocator: i32,
    /// Code that imports external module
    pub import: Vec<String>,
    /// Static string data
    pub data: Vec<String>,
    /// Set of function declare code
    pub declare: Vec<String>,
    /// Macro code that's processing in compile time
    pub r#macro: IndexMap<String, (Vec<String>, Expr)>,
    /// Operator overload code that's processing in compile time
    pub overload: IndexMap<(usize, (String, String)), String>,
    /// Type alias that's defined by user
    pub type_alias: IndexMap<String, Type>,
    /// Errors that occurred during compilation
    pub error: Option<String>,
    /// Type environment for variable
    pub variable_type: IndexMap<String, Type>,
    /// Type environment for global varibale
    pub global_type: IndexMap<String, Type>,
    /// Type environment for argument
    pub argument_type: IndexMap<String, Type>,
    /// Type environment for function
    pub function_type: IndexMap<String, Function>,
    /// Type environment for exported function
    pub export_type: IndexMap<String, Function>,
    /// Type of main program returns
    pub program_return: Type,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            allocator: 0,
            import: vec![],
            data: vec![],
            declare: vec![],
            error: None,
            r#macro: IndexMap::new(),
            overload: IndexMap::new(),
            type_alias: IndexMap::new(),
            variable_type: IndexMap::new(),
            global_type: IndexMap::new(),
            argument_type: IndexMap::new(),
            function_type: IndexMap::new(),
            export_type: IndexMap::new(),
            program_return: Type::Void,
        }
    }

    pub fn build(&mut self, source: &str) -> Option<String> {
        let ast = Block::parse(source)?;
        self.program_return = ast.type_infer(self)?;
        Some(format!(
            "(module {import} {memory} {tag} {memcpy} {strings} {declare} {global} (func (export \"_start\") {ret} {locals} {code}))",
            code = ast.compile(self)?,
            ret = compile_return!(self.program_return.clone(), self),
            import = join!(self.import),
            strings = join!(self.data),
            declare = join!(self.declare),
            global = expand_global(self)?,
            locals = expand_local(self)?,
            tag = "(tag $err)",
            memory = "(memory $mem (export \"mem\") 64)",
            memcpy = &format!(
                "(global $allocator (export \"allocator\") (mut i32) (i32.const {allocator})) {}",
                format!(
                    "(func $malloc (export \"malloc\") (param $size i32) (result i32) (global.get $allocator) {}",
                    "(global.set $allocator (i32.add (global.get $allocator) (local.get $size))))"
                ),
                allocator = self.allocator
            ),
        ))
    }
}
