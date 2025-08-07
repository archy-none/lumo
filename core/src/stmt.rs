use crate::*;

/// Import function signature: name, arguments, return, alias
type Signature = (String, Vec<(String, Type)>, Type);
#[derive(Clone, Debug)]
pub enum Stmt {
    Expr(Expr),
    Let(Scope, Expr, Expr),
    If(Expr, Expr, Option<Box<Stmt>>),
    While(Expr, Expr),
    Type(String, Type),
    Try(Expr, Box<Stmt>),
    Macro(String, Vec<String>, Expr),
    Overload(usize, (Type, Type), String),
    Import(Option<String>, Signature),
    Return(Option<Expr>),
    Break,
    Next,
}

#[derive(Clone, Copy, Debug)]
pub enum Scope {
    Global,
    Local,
}

impl Node for Stmt {
    fn parse(source: &str) -> Option<Self> {
        let source = source.trim();
        if let Some(source) = source.strip_prefix("if ") {
            let tokens = tokenize(source, SPACE.as_ref(), false, true, false)?;
            let then = tokens.iter().position(|i| i == "then")?;
            if let Some(r#else) = tokens.iter().position(|i| i == "else") {
                let cond = Expr::parse(&join!(tokens.get(0..then)?))?;
                let then = Expr::parse(&join!(tokens.get(then + 1..r#else)?))?;
                let r#else = Stmt::parse(&join!(tokens.get(r#else + 1..)?))?;
                Some(Stmt::If(cond, then, Some(Box::new(r#else))))
            } else {
                let cond = Expr::parse(&join!(tokens.get(0..then)?))?;
                let then = Expr::parse(&join!(tokens.get(then + 1..)?))?;
                Some(Stmt::If(cond, then, None))
            }
        } else if let Some(source) = source.strip_prefix("while ") {
            let tokens = tokenize(source, SPACE.as_ref(), false, true, false)?;
            let r#loop = tokens.iter().position(|i| i == "loop")?;
            let cond = Expr::parse(&join!(tokens.get(0..r#loop)?))?;
            let body = Expr::parse(&join!(tokens.get(r#loop + 1..)?))?;
            Some(Stmt::While(cond, body))
        } else if let Some(source) = source.strip_prefix("try ") {
            let tokens = tokenize(source, SPACE.as_ref(), false, true, false)?;
            let r#catch = tokens.iter().position(|i| i == "catch")?;
            let expr = Expr::parse(&join!(tokens.get(0..r#catch)?))?;
            let r#catch = Stmt::parse(&join!(tokens.get(r#catch + 1..)?))?;
            Some(Stmt::Try(expr, Box::new(r#catch)))
        } else if let Some(token) = source.strip_prefix("let ") {
            if let Some((name, value)) = token.split_once("=") {
                let (name, value) = (Expr::parse(name)?, Expr::parse(value)?);
                Some(Stmt::Let(Scope::Local, name, value))
            } else {
                let source = Op::parse(token)?;
                macro_rules! assign_with {
                    ($op: ident) => {
                        if let Op::$op(name, value) = source {
                            let value = Expr::Operator(Box::new(Op::$op(name.clone(), value)));
                            return Some(Stmt::Let(Scope::Local, name, value));
                        }
                    };
                }
                assign_with!(Add);
                assign_with!(Sub);
                assign_with!(Mul);
                assign_with!(Div);
                assign_with!(Mod);
                None
            }
        } else if let Some(token) = source.strip_prefix("pub ") {
            let Stmt::Let(Scope::Local, name, value) = Stmt::parse(token)? else {
                return None;
            };
            Some(Stmt::Let(Scope::Global, name, value))
        } else if let Some(source) = source.strip_prefix("type ") {
            let (name, value) = source.split_once("=")?;
            let Some(Expr::Variable(name)) = Expr::parse(name) else {
                return None;
            };
            Some(Stmt::Type(name, Type::parse(value)?))
        } else if let Some(source) = source.strip_prefix("macro ") {
            let (head, value) = source.split_once("=")?;
            let Expr::Call(name, args) = Expr::parse(head)? else {
                return None;
            };
            let func = |x: &Expr| {
                let Expr::Variable(x) = x else { return None };
                Some(x.clone())
            };
            let args = args.iter().map(func).collect::<Option<Vec<_>>>()?;
            Some(Stmt::Macro(name, args, Expr::parse(value)?))
        } else if let Some(source) = source.strip_prefix("overload ") {
            let (name, value) = source.split_once("=")?;
            let tokens = tokenize(value, SPACE.as_ref(), true, true, false)?;
            let [lhs, op, rhs] = tokens.as_slice() else {
                return None;
            };
            Some(Stmt::Overload(
                Op::parse(&format!("0 {op} 0"))?.overload_id()?,
                (Type::parse(lhs)?, Type::parse(rhs)?),
                name.trim().to_owned(),
            ))
        } else if let Some(after) = source.strip_prefix("load ") {
            let rest = after.trim_start();
            if let Some((module, sigs)) = rest.split_once(".") {
                let module = module.trim().to_string();
                Some(Stmt::Import(Some(module), import_args!(sigs)))
            } else {
                Some(Stmt::Import(None, import_args!(after)))
            }
        } else if let Some(source) = source.strip_prefix("return ") {
            Some(Stmt::Return(Some(Expr::parse(source)?)))
        } else if source == "return" {
            Some(Stmt::Return(None))
        } else if source == "next" {
            Some(Stmt::Next)
        } else if source == "break" {
            Some(Stmt::Break)
        } else {
            Some(Stmt::Expr(Expr::parse(source)?))
        }
    }

    fn compile(&self, ctx: &mut Compiler) -> Option<String> {
        Some(match self {
            Stmt::Expr(expr) => expr.compile(ctx)?,
            Stmt::If(cond, then, r#else) => {
                format!(
                    "(if {} {} (then {}) {})",
                    compile_return!(self.type_infer(ctx)?, ctx),
                    cond.compile(ctx)?,
                    then.compile(ctx)?,
                    if let Some(r#else) = r#else {
                        format!("(else {})", r#else.compile(ctx)?)
                    } else {
                        String::new()
                    },
                )
            }
            Stmt::While(cond, body) => {
                format!(
                    "(block $outer (loop $while_start (br_if $outer (i32.eqz {})) {} {}))",
                    cond.compile(ctx)?,
                    body.compile(ctx)?,
                    Stmt::Next.compile(ctx)?
                )
            }
            Stmt::Next => "(br $while_start)".to_string(),
            Stmt::Break => "(br $outer)".to_string(),
            Stmt::Let(scope, name, value) => match name {
                Expr::Variable(name) => match scope {
                    Scope::Local => {
                        let typ = value.type_infer(ctx)?;
                        if !ctx.argument_type.contains_key(name) {
                            ctx.variable_type.insert(name.to_string(), typ);
                        }
                        format!("(local.set ${name} {})", value.compile(ctx)?)
                    }
                    Scope::Global => {
                        let typ = value.type_infer(ctx)?;
                        if !ctx.global_type.contains_key(name) {
                            ctx.global_type.insert(name.to_string(), typ);
                        }
                        format!("(global.set ${name} {})", value.compile(ctx)?)
                    }
                },
                Expr::Call(name, _) => {
                    self.type_infer(ctx);
                    let var_ctx = ctx.variable_type.clone();
                    let arg_ctx = ctx.argument_type.clone();
                    let function = ctx
                        .function_type
                        .get(name)
                        .or(ctx.export_type.get(name))?
                        .clone();
                    ctx.variable_type = function.variables.clone();
                    ctx.argument_type = function.arguments.clone();
                    let code = format!(
                        "(func ${name} {pub} {args} {ret} {locals} {body})",
                        args = join!(
                            &function
                                .arguments
                                .iter()
                                .map(|(name, typ)| Some(format!(
                                    "(param ${name} {})",
                                    typ.type_infer(ctx)?.compile(ctx)?
                                )))
                                .collect::<Option<Vec<_>>>()?
                        ),
                        ret = compile_return!(function.returns, ctx),
                        pub = if let Scope::Global = scope { format!("(export \"{name}\")") } else { String::new() },
                        body = value.compile(ctx)?, locals = expand_local(ctx)?
                    );
                    ctx.declare_code.push(code);
                    ctx.variable_type = var_ctx;
                    ctx.argument_type = arg_ctx;
                    String::new()
                }
                Expr::Operator(oper) => {
                    self.type_infer(ctx)?;
                    let Op::Cast(func, _) = *oper.clone() else {
                        return None;
                    };
                    Stmt::Let(*scope, func, value.clone()).compile(ctx)?
                }
                Expr::Index(array, index) => {
                    let Type::Array(typ) = array.type_infer(ctx)? else {
                        return None;
                    };
                    type_check!(typ, value.type_infer(ctx)?, ctx)?;
                    let addr = Box::new(address_calc!(array, index, typ));
                    Expr::Poke(addr, Box::new(value.clone())).compile(ctx)?
                }
                Expr::Field(expr, key) => {
                    let Type::Dict(dict) = expr.type_infer(ctx)? else {
                        return None;
                    };
                    let (offset, typ) = dict.get(key)?.clone();
                    type_check!(typ, value.type_infer(ctx)?, ctx)?;
                    let addr = Box::new(offset_calc!(expr, offset));
                    Expr::Poke(addr, Box::new(value.clone())).compile(ctx)?
                }
                _ => return None,
            },
            Stmt::Try(expr, catch) => expr.compile(ctx).or(catch.compile(ctx))?,
            Stmt::Import(module, funcs) => {
                let (name, args, ret_typ) = funcs.clone();
                let mut export = name.clone();
                if let Some(module) = module {
                    export = format!("{module}.{name}")
                };
                let function = Function {
                    variables: IndexMap::new(),
                    arguments: args.into_iter().collect(),
                    returns: ret_typ.clone(),
                };
                let sig = compile_args_type!(function, ctx);
                let ret = compile_return!(ret_typ, ctx);
                ctx.import_code.push(format!(
                    "(import \"env\" \"{export}\" (func ${name} {sig} {ret}))"
                ));
                String::new()
            }
            Stmt::Return(Some(expr)) => {
                format!("(return {})", expr.compile(ctx)?)
            }
            Stmt::Return(_) => "(return)".to_string(),
            Stmt::Type(_, _) | Stmt::Macro(_, _, _) | Stmt::Overload(_, (_, _), _) => String::new(),
        })
    }

    fn type_infer(&self, ctx: &mut Compiler) -> Option<Type> {
        Some(match self {
            Stmt::Expr(expr) => expr.type_infer(ctx)?,
            Stmt::If(cond, then, r#else) => {
                type_check!(cond, Type::Bool, ctx)?;
                if let Some(r#else) = r#else {
                    type_check!(then, r#else, ctx)?
                } else {
                    then.type_infer(ctx)?
                }
            }
            Stmt::While(cond, body) => {
                type_check!(cond, Type::Bool, ctx)?;
                body.type_infer(ctx)?;
                Type::Void
            }
            Stmt::Break => Type::Void,
            Stmt::Next => Type::Void,
            Stmt::Let(scope, name, value) => {
                match name {
                    Expr::Variable(name) => match scope {
                        Scope::Local => {
                            if !ctx.argument_type.contains_key(name) {
                                let value_type = value.type_infer(ctx)?;
                                if let Some(exist_val) = ctx.clone().variable_type.get(name) {
                                    type_check!(exist_val, value_type, ctx)?;
                                } else {
                                    ctx.variable_type.insert(name.to_string(), value_type);
                                }
                            } else {
                                let msg = format!("can't reassign value to argument");
                                ctx.occurred_error = Some(msg);
                                return None;
                            }
                        }
                        Scope::Global => {
                            let value_type = value.type_infer(ctx)?;
                            if let Some(exist_val) = ctx.clone().global_type.get(name) {
                                type_check!(exist_val, value_type, ctx)?;
                            } else {
                                ctx.global_type.insert(name.to_string(), value_type);
                            }
                        }
                    },
                    Expr::Call(name, args) => {
                        let var_ctx = ctx.variable_type.clone();
                        let arg_ctx = ctx.argument_type.clone();
                        ctx.variable_type.clear();
                        ctx.argument_type.clear();
                        compile_args!(args, ctx);
                        let frame = Function {
                            returns: value.type_infer(ctx)?,
                            variables: ctx.variable_type.clone(),
                            arguments: ctx.argument_type.clone(),
                        };
                        if let Scope::Global = scope {
                            &mut ctx.export_type
                        } else {
                            &mut ctx.function_type
                        }
                        .insert(name.to_owned(), frame);
                        ctx.variable_type = var_ctx;
                        ctx.argument_type = arg_ctx;
                    }
                    Expr::Operator(oper) => match *oper.clone() {
                        Op::Cast(Expr::Call(name, args), ret) => {
                            let var_ctx = ctx.variable_type.clone();
                            let arg_ctx = ctx.argument_type.clone();
                            ctx.variable_type.clear();
                            ctx.argument_type.clear();
                            compile_args!(args.clone(), ctx);
                            ctx.function_type.insert(
                                name.to_owned(),
                                Function {
                                    variables: ctx.variable_type.clone(),
                                    arguments: ctx.argument_type.clone(),
                                    returns: ret.clone(),
                                },
                            );
                            type_check!(value.type_infer(ctx)?, ret, ctx);
                            ctx.variable_type = var_ctx;
                            ctx.argument_type = arg_ctx;
                        }
                        _ => return None,
                    },
                    _ => {
                        value.type_infer(ctx);
                    }
                }
                Type::Void
            }
            Stmt::Type(name, value) => {
                ctx.type_alias.insert(name.to_string(), value.clone());
                Type::Void
            }
            Stmt::Macro(name, args, expr) => {
                ctx.macro_code
                    .insert(name.to_owned(), (args.clone(), expr.clone()));
                Type::Void
            }
            Stmt::Try(expr, catch) => expr.type_infer(ctx).or(catch.type_infer(ctx))?,
            Stmt::Import(_module, funcs) => {
                let (fn_name, args, ret_typ) = funcs;
                let mut arg_map = IndexMap::new();
                for (name, typ) in args.iter() {
                    arg_map.insert(name.to_string(), typ.clone());
                }
                ctx.function_type.insert(
                    fn_name.clone(),
                    Function {
                        variables: IndexMap::new(),
                        arguments: arg_map,
                        returns: ret_typ.clone(),
                    },
                );
                Type::Void
            }
            Stmt::Overload(id, (arg1, arg2), name) => {
                let key = (*id, (arg1.format(), arg2.format()));
                ctx.overload.insert(key, name.clone());
                Type::Void
            }
            Stmt::Return(_) => Type::Void,
        })
    }
}
