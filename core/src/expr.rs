use crate::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Variable(String),
    Operator(Box<Op>),
    Call(String, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Field(Box<Expr>, String),
    Block(Block),
    Clone(Box<Expr>),
    Peek(Box<Expr>, Type),
    Poke(Box<Expr>, Box<Expr>),
}

impl Node for Expr {
    fn parse(source: &str) -> Option<Expr> {
        let token = source.trim();
        // Operator
        if let Some(literal) = Op::parse(&token) {
            Some(Expr::Operator(Box::new(literal)))
        // Literal value
        } else if let Some(literal) = Value::parse(&token) {
            Some(Expr::Literal(literal))
        // Formatted string (f-string)
        } else if token.starts_with("f\"") && token.ends_with('"') {
            let str = str_format(token.get(2..token.len() - 1)?)?;
            let mut result: Option<Expr> = None;
            macro_rules! concat {
                ($expr: expr) => {
                    Some(if let Some(result) = result {
                        Expr::Operator(Box::new(Op::Add(result, $expr.clone())))
                    } else {
                        $expr
                    })
                };
            }
            for elm in str {
                result = if elm.starts_with("{") && elm.ends_with("}") {
                    let elm = elm.get(1..elm.len() - 1)?.trim();
                    concat!(Expr::Operator(Box::new(Op::Cast(
                        Expr::Block(Block::parse(elm)?),
                        Type::String,
                    ))))
                } else {
                    concat!(Expr::Literal(Value::String(elm)))
                }
            }
            result.clone()
        // Prioritize expression `(expr)`
        } else if token.starts_with("(") && token.ends_with(")") {
            let token = token.get(1..token.len() - 1)?.trim();
            Some(Expr::parse(token)?)
        // Code block `{ stmt; ... }`
        } else if token.starts_with("{") && token.ends_with("}") {
            let token = token.get(1..token.len() - 1)?.trim();
            Some(Expr::Block(Block::parse(token)?))
        // Index access `array[index]`
        } else if token.contains("[") && token.ends_with("]") {
            let token = tokenize(token, &["["], false, true, true)?;
            let array = Expr::parse(&token.get(..token.len() - 1)?.concat())?;
            let index = token.last()?.get(1..token.last()?.len() - 1)?;
            Some(Expr::Index(Box::new(array), Box::new(Expr::parse(index)?)))
        // Function call `name(args, ...)`
        } else if token.contains("(") && token.ends_with(")") {
            let token = tokenize(token, &["("], false, true, true)?;
            let args = token.last()?.get(1..token.last()?.len() - 1)?;
            let args = tokenize(args, &[","], false, true, false)?;
            let args = args
                .iter()
                .map(|i| Expr::parse(&i))
                .collect::<Option<Vec<_>>>()?;
            let name = token.get(..token.len() - 1)?.concat();
            let (name, args) = match Expr::parse(&name)? {
                Expr::Variable(name) => (name, args),
                Expr::Field(obj, name) => (name, [vec![*obj], args].concat()),
                _ => return None,
            };
            if name == "memcpy" {
                let obj = args.first()?.clone();
                return Some(Expr::Clone(Box::new(obj)));
            }
            Some(Expr::Call(name, args))
        // Dictionary access `dict.field`
        } else if token.contains(".") {
            let (dict, field) = token.rsplit_once(".")?;
            let field = field.trim();
            if !is_identifier(field) {
                return None;
            };
            Some(Expr::Field(Box::new(Expr::parse(dict)?), field.to_owned()))
        // Enumerate access `( a | b )#a`
        } else if source.contains("#") {
            let (typ, enum_) = source.rsplit_once("#")?;
            let enum_ = Value::Enum(Type::parse(typ)?, enum_.to_owned());
            Some(Expr::Literal(enum_))
        // Variable reference
        } else if is_identifier(token) {
            Some(Expr::Variable(token.to_string()))
        } else {
            None
        }
    }

    fn compile(&self, ctx: &mut Compiler) -> Option<String> {
        Some(match self {
            Expr::Operator(oper) => oper.compile(ctx)?,
            Expr::Variable(name) => {
                let is_global = ctx.global_type.contains_key(name);
                let scope = if is_global { "global" } else { "local" };
                format!("({scope}.get ${name})",)
            }
            Expr::Literal(literal) => literal.compile(ctx)?,
            Expr::Call(name, args) => {
                if ctx.function_type.contains_key(name) || ctx.export_type.contains_key(name) {
                    let args = args
                        .iter()
                        .map(|x| x.compile(ctx))
                        .collect::<Option<Vec<_>>>()?;
                    format!("(call ${name} {})", join!(args))
                } else if let Some((params, expr)) = ctx.macro_code.get(name).cloned() {
                    let mut old_types = IndexMap::new();
                    for (param, arg) in params.iter().zip(args) {
                        let typ = arg.type_infer(ctx)?;
                        if let Some(original_var) = ctx.variable_type.get(param).cloned() {
                            old_types.insert(param.to_owned(), original_var);
                        }
                        ctx.variable_type.insert(param.to_owned(), typ);
                    }
                    let mut body = expr.compile(ctx)?;
                    for (param, arg) in params.iter().zip(args) {
                        let var = Expr::Variable(param.to_owned()).compile(ctx)?;
                        body = body.replace(&var, &arg.compile(ctx)?);
                    }
                    ctx.variable_type.extend(old_types);
                    body
                } else {
                    return None;
                }
            }
            Expr::Index(array, index) => {
                let Type::Array(typ) = array.type_infer(ctx)?.type_infer(ctx)? else {
                    return None;
                };
                let addr = Box::new(address_calc!(array, index, typ.clone()));
                Expr::Peek(addr, *typ).compile(ctx)?
            }
            Expr::Field(expr, key) => {
                let typ = expr.type_infer(ctx)?.type_infer(ctx)?;
                let Type::Dict(dict) = typ else {
                    return None;
                };
                let (offset, typ) = dict.get(key)?.clone();
                let addr = offset_calc!(expr, offset, typ.clone());
                Expr::Peek(Box::new(addr), typ).compile(ctx)?
            }
            Expr::Block(block) => block.compile(ctx)?,
            Expr::Clone(from) => {
                let size = from.object_size(ctx)?.compile(ctx)?;
                format!(
                    "(memory.copy (global.get $allocator) {object} {size}) (call $malloc {size})",
                    object = from.compile(ctx)?,
                )
            }
            Expr::Peek(expr, typ) => {
                let safeguard = {
                    let opgen = |x| Expr::Operator(Box::new(x));
                    let is_null = opgen(Op::LNot(opgen(Op::NullCheck(*expr.clone()))));
                    Stmt::If(is_null, Expr::Block(Block(vec![Stmt::Error])), None)
                };
                let loader = format!("({}.load {})", typ.compile(ctx)?, expr.compile(ctx)?);
                safeguard.compile(ctx)? + &loader
            }
            Expr::Poke(addr, expr) => {
                let safeguard = {
                    let opgen = |x| Expr::Operator(Box::new(x));
                    let is_null = opgen(Op::LNot(opgen(Op::NullCheck(*expr.clone()))));
                    Stmt::If(is_null, Expr::Block(Block(vec![Stmt::Error])), None)
                };
                let typ = expr.type_infer(ctx)?.compile(ctx)?;
                let [addr, expr] = [addr.compile(ctx)?, expr.compile(ctx)?];
                let loader = format!("({typ}.store {addr} {expr})");
                safeguard.compile(ctx)? + &loader
            }
        })
    }

    fn type_infer(&self, ctx: &mut Compiler) -> Option<Type> {
        Some(match self {
            Expr::Operator(oper) => oper.type_infer(ctx)?,
            Expr::Variable(name) => {
                if let Some(global) = ctx.global_type.get(name) {
                    global.clone()
                } else if let Some(local) = ctx.variable_type.get(name) {
                    local.clone()
                } else if let Some(arg) = ctx.argument_type.get(name) {
                    arg.clone()
                } else {
                    ctx.occurred_error = Some(format!("undefined variable `{name}`"));
                    return None;
                }
            }
            Expr::Literal(literal) => literal.type_infer(ctx)?,
            Expr::Call(name, args) => {
                macro_rules! arglen_check {
                    ($params: expr, $typ: literal) => {
                        if args.len() != $params.len() {
                            let (typ, paramlen, arglen) = ($typ, $params.len(), args.len());
                            let errmsg = format!("arguments of {typ} `{name}` length should be {paramlen}, but passed {arglen} values");
                            ctx.occurred_error = Some(errmsg);
                            return None;
                        }
                    };
                }
                if let Some(function) = ctx
                    .function_type
                    .get(name)
                    .or(ctx.export_type.get(name))
                    .cloned()
                {
                    arglen_check!(function.arguments, "function");
                    let func = |(arg, typ): (&Expr, &Type)| type_check!(arg, typ, ctx);
                    let ziped = args.iter().zip(function.arguments.values());
                    ziped.map(func).collect::<Option<Vec<_>>>()?;
                    function.returns.type_infer(ctx)?
                } else if let Some((params, expr)) = ctx.macro_code.get(name).cloned() {
                    arglen_check!(params, "macro");
                    let var_ctx = ctx.variable_type.clone();
                    for (params, arg) in params.iter().zip(args) {
                        let typ = arg.type_infer(ctx)?;
                        ctx.variable_type.insert(params.to_owned(), typ);
                    }
                    let typ = expr.type_infer(ctx)?;
                    ctx.variable_type = var_ctx;
                    typ
                } else {
                    ctx.occurred_error = Some(format!(
                        "function or macro `{name}` you want to call is not defined"
                    ));
                    return None;
                }
            }
            Expr::Index(arr, _) => {
                let infered = arr.type_infer(ctx)?;
                let Some(Type::Array(typ)) = infered.type_infer(ctx) else {
                    let error_message = format!("can't index access to {}", infered.format());
                    ctx.occurred_error = Some(error_message);
                    return None;
                };
                typ.type_infer(ctx)?
            }
            Expr::Field(dict, key) => {
                let infered = dict.type_infer(ctx)?.type_infer(ctx)?;
                if let Type::Dict(dict) = infered.clone() {
                    let Some((_offset, typ)) = dict.get(key) else {
                        let error_message = format!("{} haven't field `{key}`", infered.format());
                        ctx.occurred_error = Some(error_message);
                        return None;
                    };
                    typ.type_infer(ctx)?
                } else {
                    let error_message = format!("can't field access to {}", infered.format());
                    ctx.occurred_error = Some(error_message);
                    return None;
                }
            }
            Expr::Block(block) => block.type_infer(ctx)?,
            Expr::Clone(from) => {
                let typ = from.type_infer(ctx)?;
                if is_ptr!(typ, ctx) {
                    typ
                } else {
                    let errmsg = "can't memory copy primitive typed value";
                    ctx.occurred_error = Some(errmsg.to_string());
                    return None;
                }
            }
            Expr::Peek(expr, typ) => {
                expr.type_infer(ctx)?;
                typ.clone()
            }
            Expr::Poke(addr, expr) => {
                addr.type_infer(ctx)?;
                expr.type_infer(ctx)?;
                Type::Void
            }
        })
    }
}

impl Expr {
    pub fn object_size(&self, ctx: &mut Compiler) -> Option<Expr> {
        match self.type_infer(ctx)? {
            Type::Dict(dict) => Some(Expr::Literal(Value::Integer(dict.len() as i32 * BYTES))),
            Type::Array(_) => Some(Expr::Operator(Box::new(Op::Add(
                Expr::Operator(Box::new(Op::Mul(
                    Expr::Literal(Value::Integer(BYTES)),
                    Expr::Peek(Box::new(self.clone()), Type::Integer),
                ))),
                Expr::Literal(Value::Integer(BYTES)),
            )))),
            _ => None,
        }
    }
}
