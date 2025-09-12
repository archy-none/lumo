use crate::*;

#[derive(Debug, Clone)]
pub enum Op {
    Add(Expr, Expr),
    Sub(Expr, Expr),
    Mul(Expr, Expr),
    Div(Expr, Expr),
    Mod(Expr, Expr),
    Shr(Expr, Expr),
    Shl(Expr, Expr),
    Eql(Expr, Expr),
    Neq(Expr, Expr),
    Lt(Expr, Expr),
    Gt(Expr, Expr),
    LtEq(Expr, Expr),
    GtEq(Expr, Expr),
    BAnd(Expr, Expr),
    BOr(Expr, Expr),
    BNot(Expr),
    XOr(Expr, Expr),
    LAnd(Expr, Expr),
    LOr(Expr, Expr),
    LNot(Expr),
    Cast(Expr, Type),
    NullCheck(Expr),
    Nullable(Type),
    Transmute(Expr, Type),
}

impl Node for Op {
    fn parse(source: &str) -> Option<Self> {
        let tokens: Vec<String> = tokenize(source, SPACE.as_ref(), true, true, false)?;
        // Parsing is from right to left because operator is left-associative
        let binopergen = |n: usize| {
            let operator = tokens.get(n)?;
            let lhs = &join!(tokens.get(..n)?);
            let rhs = &join!(tokens.get(n + 1..)?);
            Some(match operator.as_str() {
                "+" => Op::Add(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "-" => Op::Sub(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "*" => Op::Mul(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "/" => Op::Div(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "%" => Op::Mod(Expr::parse(lhs)?, Expr::parse(rhs)?),
                ">>" => Op::Shr(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "<<" => Op::Shl(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "==" => Op::Eql(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "!=" => Op::Neq(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "<" => Op::Lt(Expr::parse(lhs)?, Expr::parse(rhs)?),
                ">" => Op::Gt(Expr::parse(lhs)?, Expr::parse(rhs)?),
                ">=" => Op::GtEq(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "<=" => Op::LtEq(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "&" => Op::BAnd(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "|" => Op::BOr(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "^" => Op::XOr(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "&&" => Op::LAnd(Expr::parse(lhs)?, Expr::parse(rhs)?),
                "||" => Op::LOr(Expr::parse(lhs)?, Expr::parse(rhs)?),
                ":" => Op::Cast(Expr::parse(lhs)?, Type::parse(rhs)?),
                _ => return None,
            })
        };
        let unaryopergen = || {
            let op = tokens.first()?.trim();
            let token = &join!(tokens.get(1..)?);
            Some(match op {
                "~" => Op::BNot(Expr::parse(token)?),
                "!" => Op::LNot(Expr::parse(token)?),
                "-" => {
                    let token = Expr::parse(token)?;
                    Op::Sub(
                        Expr::Operator(Box::new(Op::Sub(token.clone(), token.clone()))),
                        token,
                    )
                }
                _ => return None,
            })
        };
        let suffixopergen = || {
            let op = tokens.last()?.trim();
            let token = &join!(tokens.get(..tokens.len() - 1)?);
            Some(match op {
                "?" => Op::NullCheck(Expr::parse(token)?),
                "!" => Op::Nullable(Type::parse(token)?),
                _ => return None,
            })
        };
        if let Some(op) = unaryopergen() {
            return Some(op);
        }
        if let Some(op) = suffixopergen() {
            return Some(op);
        }
        for i in 2..tokens.len() {
            if let Some(op) = binopergen(tokens.len().checked_sub(i)?) {
                return Some(op);
            }
        }
        None
    }

    fn compile(&self, ctx: &mut Compiler) -> Option<String> {
        overload!(self, ctx, compile);
        Some(match self {
            Op::Add(lhs, rhs) => compile_op!("add", ctx, lhs, rhs),
            Op::Sub(lhs, rhs) => compile_op!("sub", ctx, lhs, rhs),
            Op::Mul(lhs, rhs) => compile_op!("mul", ctx, lhs, rhs),
            Op::Div(lhs, rhs) => compile_op!("div_s", ctx, lhs, rhs),
            Op::Shr(lhs, rhs) => compile_op!("shr", ctx, lhs, rhs),
            Op::Shl(lhs, rhs) => compile_op!("shl", ctx, lhs, rhs),
            Op::BAnd(lhs, rhs) => compile_op!("and", ctx, lhs, rhs),
            Op::BOr(lhs, rhs) => compile_op!("or", ctx, lhs, rhs),
            Op::XOr(lhs, rhs) => compile_op!("xor", ctx, lhs, rhs),
            Op::LNot(term) => compile_op!("eqz", ctx, term),
            Op::Neq(lhs, rhs) => compile_op!("ne", ctx, lhs, rhs),
            Op::Lt(lhs, rhs) => compile_op!("lt_s", ctx, lhs, rhs),
            Op::Gt(lhs, rhs) => compile_op!("gt_s", ctx, lhs, rhs),
            Op::LtEq(lhs, rhs) => compile_op!("le_s", ctx, lhs, rhs),
            Op::GtEq(lhs, rhs) => compile_op!("ge_s", ctx, lhs, rhs),
            Op::LAnd(lhs, rhs) => compile_op!("and", ctx, lhs, rhs),
            Op::LOr(lhs, rhs) => compile_op!("or", ctx, lhs, rhs),
            Op::Eql(lhs, rhs) => compile_op!("eq", ctx, lhs, rhs),
            Op::Mod(lhs, rhs) => {
                let typ = lhs.type_infer(ctx)?.compile(ctx)?;
                let (lhs, rhs) = (lhs.compile(ctx)?, rhs.compile(ctx)?);
                if typ == "i32" {
                    format!("(i32.rem_s (i32.add (i32.rem_s {lhs} {rhs}) {rhs}) {rhs})")
                } else {
                    format!("(f32.sub {lhs} (f32.mul (f32.floor (f32.div {lhs} {rhs})) {rhs}))")
                }
            }
            Op::BNot(lhs) => {
                let minus_one = Expr::Literal(Value::Integer(-1));
                compile_op!("xor", ctx, lhs, minus_one)
            }
            Op::Cast(lhs, rhs) => {
                let rhs = rhs.type_infer(ctx)?;
                match (lhs.type_infer(ctx)?, &rhs) {
                    (Type::Number | Type::Integer, Type::String) => {
                        let numized = Expr::Operator(Box::new(Op::Cast(lhs.clone(), Type::Number)));
                        Expr::Call("to_str".to_owned(), vec![numized]).compile(ctx)?
                    }
                    (Type::String, Type::Number | Type::Integer) => {
                        let parse = Expr::Call("to_num".to_owned(), vec![lhs.clone()]);
                        Op::Cast(parse, rhs).compile(ctx)?
                    }
                    (Type::Integer, Type::Number) => {
                        format!("(f32.convert_i32_s {})", lhs.compile(ctx)?)
                    }
                    (Type::Number, Type::Integer) => {
                        format!("(i32.trunc_f32_s {})", lhs.compile(ctx)?)
                    }
                    (lhs, rhs) if lhs == *rhs => lhs.compile(ctx)?,
                    _ => return None,
                }
            }
            Op::Transmute(lhs, _) => lhs.compile(ctx)?,
            Op::NullCheck(expr) => Op::Neq(
                Expr::Operator(Box::new(Op::Transmute(expr.clone(), Type::Integer))),
                Expr::Literal(Value::Integer(-1)),
            )
            .compile(ctx)?,
            Op::Nullable(_) => Value::Integer(-1).compile(ctx)?,
        })
    }

    fn type_infer(&self, ctx: &mut Compiler) -> Option<Type> {
        overload!(self, ctx, type_infer);
        match self {
            Op::Add(lhs, rhs)
            | Op::Sub(lhs, rhs)
            | Op::Mul(lhs, rhs)
            | Op::Div(lhs, rhs)
            | Op::Mod(lhs, rhs)
            | Op::BAnd(lhs, rhs)
            | Op::BOr(lhs, rhs)
            | Op::XOr(lhs, rhs) => correct!(lhs, rhs, ctx, Type::Number | Type::Integer),
            Op::Shr(lhs, rhs) | Op::Shl(lhs, rhs) => correct!(lhs, rhs, ctx, Type::Integer),
            Op::Eql(lhs, rhs) | Op::Neq(lhs, rhs) => {
                correct!(lhs, rhs, ctx, Type::Number | Type::Integer | Type::Enum(_))?;
                Some(Type::Bool)
            }
            Op::Lt(lhs, rhs) | Op::Gt(lhs, rhs) | Op::LtEq(lhs, rhs) | Op::GtEq(lhs, rhs) => {
                correct!(lhs, rhs, ctx, Type::Number | Type::Integer)?;
                Some(Type::Bool)
            }
            Op::LAnd(lhs, rhs) | Op::LOr(lhs, rhs) => {
                type_check!(lhs, Type::Bool, ctx)?;
                type_check!(rhs, Type::Bool, ctx)?;
                Some(Type::Bool)
            }
            Op::LNot(lhs) => {
                type_check!(lhs, Type::Bool, ctx)?;
                Some(Type::Bool)
            }
            Op::Cast(lhs, rhs) => {
                let lhs = lhs.type_infer(ctx)?;
                let rhs = rhs.type_infer(ctx)?;
                match (lhs.clone(), rhs.clone()) {
                    (Type::Number, Type::Integer) => Some(Type::Integer),
                    (Type::Integer, Type::Number) => Some(Type::Number),
                    (Type::String, Type::Integer | Type::Number) => Some(rhs),
                    (Type::Integer | Type::Number, Type::String) => Some(Type::String),
                    (lhs, rhs) if lhs == rhs => Some(lhs),
                    _ => {
                        let [lhs, rhs] = [lhs.format(), rhs.format()];
                        let msg = format!("type {lhs} can't convert to {rhs}");
                        ctx.error = Some(msg);
                        return None;
                    }
                }
            }
            Op::BNot(lhs) => {
                type_check!(lhs, Type::Integer, ctx)?;
                Some(Type::Integer)
            }
            Op::Transmute(lhs, rhs) => {
                lhs.type_infer(ctx)?;
                rhs.type_infer(ctx)
            }
            Op::NullCheck(expr) => {
                if is_ptr!(expr.type_infer(ctx)?, ctx) {
                    Some(Type::Bool)
                } else {
                    let errmsg = format!("can't null-check primitive typed value");
                    ctx.error = Some(errmsg);
                    return None;
                }
            }
            Op::Nullable(typ) => {
                if is_ptr!(typ, ctx) {
                    Some(typ.clone())
                } else {
                    let errmsg = format!("primitive types are not nullable");
                    ctx.error = Some(errmsg);
                    return None;
                }
            }
        }
    }
}

impl Op {
    pub fn overload_id_table() -> IndexMap<String, usize> {
        IndexMap::from([
            ("+".to_owned(), 1),
            ("-".to_owned(), 2),
            ("*".to_owned(), 3),
            ("/".to_owned(), 4),
            ("%".to_owned(), 5),
            (">>".to_owned(), 6),
            ("<<".to_owned(), 7),
            ("==".to_owned(), 8),
            ("!=".to_owned(), 9),
            ("<".to_owned(), 10),
            (">".to_owned(), 11),
            ("<=".to_owned(), 13),
            (">=".to_owned(), 12),
            ("&".to_owned(), 14),
            ("|".to_owned(), 15),
            ("^".to_owned(), 17),
            ("&&".to_owned(), 18),
            ("||".to_owned(), 19),
        ])
    }

    pub fn get_overload_id(&self) -> Option<usize> {
        Some(match self {
            Op::Add(_, _) => 1,
            Op::Sub(_, _) => 2,
            Op::Mul(_, _) => 3,
            Op::Div(_, _) => 4,
            Op::Mod(_, _) => 5,
            Op::Shr(_, _) => 6,
            Op::Shl(_, _) => 7,
            Op::Eql(_, _) => 8,
            Op::Neq(_, _) => 9,
            Op::Lt(_, _) => 10,
            Op::Gt(_, _) => 11,
            Op::LtEq(_, _) => 12,
            Op::GtEq(_, _) => 13,
            Op::BAnd(_, _) => 14,
            Op::BOr(_, _) => 15,
            Op::XOr(_, _) => 17,
            Op::LAnd(_, _) => 18,
            Op::LOr(_, _) => 19,
            _ => return None,
        })
    }

    pub fn binop_term(&self) -> Option<(Expr, Expr)> {
        Some(match self.clone() {
            Op::Add(lhs, rhs) => (lhs, rhs),
            Op::Sub(lhs, rhs) => (lhs, rhs),
            Op::Mul(lhs, rhs) => (lhs, rhs),
            Op::Div(lhs, rhs) => (lhs, rhs),
            Op::Mod(lhs, rhs) => (lhs, rhs),
            Op::Shr(lhs, rhs) => (lhs, rhs),
            Op::Shl(lhs, rhs) => (lhs, rhs),
            Op::Eql(lhs, rhs) => (lhs, rhs),
            Op::Neq(lhs, rhs) => (lhs, rhs),
            Op::Lt(lhs, rhs) => (lhs, rhs),
            Op::Gt(lhs, rhs) => (lhs, rhs),
            Op::LtEq(lhs, rhs) => (lhs, rhs),
            Op::GtEq(lhs, rhs) => (lhs, rhs),
            Op::BAnd(lhs, rhs) => (lhs, rhs),
            Op::BOr(lhs, rhs) => (lhs, rhs),
            Op::XOr(lhs, rhs) => (lhs, rhs),
            Op::LAnd(lhs, rhs) => (lhs, rhs),
            Op::LOr(lhs, rhs) => (lhs, rhs),
            _ => return None,
        })
    }
}
