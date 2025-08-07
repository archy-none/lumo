use crate::*;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i32),
    Number(f32),
    Bool(bool),
    Array(Vec<Expr>),
    Dict(IndexMap<String, Expr>),
    Enum(Type, String),
    String(String),
}

impl Node for Value {
    fn parse(source: &str) -> Option<Self> {
        // Integer literal
        if let Ok(n) = source.parse::<i32>() {
            Some(Value::Integer(n))
        // Number literal
        } else if let Ok(n) = source.parse::<f32>() {
            Some(Value::Number(n))
        // Boolean literal `true | false`
        } else if let Ok(n) = source.parse::<bool>() {
            Some(Value::Bool(n))
        // String literal `"..."`
        } else if source.starts_with("\"") && source.ends_with("\"") {
            let source = source.get(1..source.len() - 1)?;
            Some(Value::String(source.to_string()))
        // Array `[expr, ...]`
        } else if source.starts_with("[") && source.ends_with("]") {
            let source = source.get(1..source.len() - 1)?.trim();
            let elms = tokenize(source, &[","], false, true, false)?;
            let elms = elms.iter().map(|i| Expr::parse(&i));
            Some(Value::Array(elms.collect::<Option<Vec<_>>>()?))
        // Dict `@{ field: expr, ... }`
        } else if source.starts_with("@{") && source.ends_with("}") {
            let token = source.get(2..source.len() - 1)?.trim();
            let mut result = IndexMap::new();
            for line in tokenize(token, &[","], false, true, false)? {
                let (name, value) = line.trim().split_once(":")?;
                let name = name.trim().to_string();
                if !is_identifier(&name) {
                    return None;
                };
                result.insert(name, Expr::parse(value)?);
            }
            Some(Value::Dict(result))
        } else {
            None
        }
    }

    fn compile(&self, ctx: &mut Compiler) -> Option<String> {
        let value = |n| Box::new(Expr::Literal(Value::Integer(n)));
        Some(match self {
            Value::Number(n) => format!("(f32.const {n})"),
            Value::Integer(n) => format!("(i32.const {n})"),
            Value::Bool(n) => value(if *n { 1 } else { 0 }).compile(ctx)?,
            Value::String(str) => {
                let result = value(ctx.allocator).compile(ctx)?;
                let code = format!(r#"(data {result} "{str}\00")"#);
                ctx.allocator += str.len() as i32 + 1;
                ctx.static_data.push(code);
                result
            }
            Value::Array(array) => {
                let Type::Array(inner_type) = self.type_infer(ctx)? else {
                    return None;
                };
                let array = array.clone();
                let mut result: Vec<_> = vec![];
                let pointer;

                if is_ptr!(inner_type, ctx) {
                    let mut inner_codes = vec![];
                    for elm in array.clone() {
                        inner_codes.push(elm.compile(ctx)?)
                    }
                    pointer = ctx.allocator;
                    let poke = Expr::Poke(value(ctx.allocator), value(array.len() as i32));
                    result.push(poke.compile(ctx)?);
                    ctx.allocator += BYTES;
                    for code in inner_codes {
                        result.push(format!(
                            "({typ}.store {addr} {code})",
                            typ = inner_type.compile(ctx)?,
                            addr = value(ctx.allocator).compile(ctx)?,
                        ));
                        ctx.allocator += BYTES;
                    }
                } else {
                    pointer = ctx.allocator;
                    let poke = Expr::Poke(value(ctx.allocator), value(array.len() as i32));
                    result.push(poke.compile(ctx)?);
                    ctx.allocator += BYTES;
                    for elm in array {
                        type_check!(inner_type, elm.type_infer(ctx)?, ctx)?;
                        let poke = Expr::Poke(value(ctx.allocator), Box::new(elm));
                        result.push(poke.compile(ctx)?);
                        ctx.allocator += BYTES
                    }
                }
                join!([value(pointer).compile(ctx)?, join!(result)])
            }
            Value::Dict(dict) => {
                let mut result: Vec<_> = vec![];
                let Type::Dict(_) = self.type_infer(ctx)? else {
                    return None;
                };

                let mut prestore = IndexMap::new();
                for (name, elm) in dict {
                    let typ = elm.type_infer(ctx)?;
                    if is_ptr!(typ, ctx) {
                        prestore.insert(name, elm.compile(ctx)?);
                    }
                }

                let pointer = ctx.allocator;
                for (name, elm) in dict {
                    let typ = elm.type_infer(ctx)?;
                    result.push(format!(
                        "({typ}.store {addr} {value})",
                        typ = typ.clone().compile(ctx)?,
                        addr = value(ctx.allocator).compile(ctx)?,
                        value = prestore.get(name).cloned().or_else(|| elm.compile(ctx))?
                    ));
                    ctx.allocator += BYTES;
                }

                join!([value(pointer).compile(ctx)?, join!(result)])
            }
            Value::Enum(typ, key) => {
                let typ = typ.type_infer(ctx)?;
                let Type::Enum(enum_type) = typ.clone() else {
                    let error_message = format!("can't access enumerator to {}", typ.format());
                    ctx.occurred_error = Some(error_message);
                    return None;
                };
                let Some(variant) = enum_type.iter().position(|item| item == key) else {
                    let error_message = format!("`{key}` is invalid variant of {}", typ.format());
                    ctx.occurred_error = Some(error_message);
                    return None;
                };
                value(variant as i32).compile(ctx)?
            }
        })
    }

    fn type_infer(&self, ctx: &mut Compiler) -> Option<Type> {
        Some(match self {
            Value::Number(_) => Type::Number,
            Value::Integer(_) => Type::Integer,
            Value::Bool(_) => Type::Bool,
            Value::String(_) => Type::String,
            Value::Array(e) => {
                let origin = e.first()?.type_infer(ctx)?;
                for e in e.iter().skip(1) {
                    let typ = e.type_infer(ctx)?;
                    if typ != origin {
                        let errmsg = "array elements must be of the same type";
                        ctx.occurred_error = Some(errmsg.to_owned());
                        return None;
                    }
                }
                Type::Array(Box::new(origin))
            }
            Value::Dict(dict) => {
                let mut result = IndexMap::new();
                let mut index: i32 = 0;
                for (name, elm) in dict {
                    let typ = elm.type_infer(ctx)?;
                    result.insert(name.to_string(), (index, typ.clone()));
                    index += BYTES;
                }
                Type::Dict(result)
            }
            Value::Enum(typ, _) => typ.type_infer(ctx)?,
        })
    }
}
