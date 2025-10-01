pub const BYTES: i32 = 4;
pub const SPACE: [&str; 5] = [" ", "　", "\n", "\t", "\r"];
pub const OPERATOR: [&str; 23] = [
    "+", "-", "*", "/", "%", "==", "=", "!=", "<<", ">>", "<=", ">=", "<", ">", "&&", "||", "&",
    "|", "^", ":", "!", "?", "~",
];
pub const RESERVED: [&str; 15] = [
    "pub", "let", "type", "if", "then", "else", "while", "loop", "break", "next", "return",
    "import", "overload", "try", "catch",
];

#[macro_export]
macro_rules! expand_local {
    ($ctx: expr) => {
        join!(
            $ctx.variable
                .clone()
                .iter()
                .map(|(name, typ)| Some(format!("(local ${name} {})", typ.compile($ctx)?)))
                .collect::<Option<Vec<String>>>()?
        )
    };
}

#[macro_export]
macro_rules! expand_global {
    ($ctx: expr) => {
        join!(
            $ctx.global
                .clone()
                .iter()
                .map(|(name, typ)| {
                    Some(format!(
                        "(global ${name} (mut {typ}) ({typ}.const 0))",
                        typ = typ.compile($ctx)?
                    ))
                })
                .collect::<Option<Vec<String>>>()?
        )
    };
}

#[macro_export]
macro_rules! is_ptr {
    ($typ: expr, $ctx: expr) => {
        matches!(
            $typ.type_infer($ctx)?,
            Type::String | Type::Array(_) | Type::Dict(_)
        )
    };
}

#[macro_export]
macro_rules! type_check {
    ($lhs: expr, $rhs: expr, $ctx: expr) => {{
        let lhs = $lhs.type_infer($ctx)?.type_infer($ctx)?;
        let rhs = $rhs.type_infer($ctx)?.type_infer($ctx)?;
        if lhs.compare(&rhs, $ctx) {
            Some(lhs.clone())
        } else {
            $ctx.error = Some(format!(
                "type mismatch between {} and {}",
                lhs.format(),
                rhs.format()
            ));
            None
        }
    }};
}

#[macro_export]
macro_rules! correct {
    ($lhs: expr, $rhs: expr , $ctx: expr, $pat: pat) => {{
        let ret = type_check!($lhs, $rhs, $ctx)?;
        if let $pat = ret {
            Some(ret)
        } else {
            let msg = format!(
                "can't this operation between {} and {}",
                $lhs.type_infer($ctx)?.format(),
                $rhs.type_infer($ctx)?.format()
            );
            $ctx.error = Some(msg);
            None
        }
    }};
}

#[macro_export]
macro_rules! compile_return {
    ($ret: expr, $ctx: expr) => {{
        let ret = $ret.type_infer($ctx)?;
        if let Type::Void = ret {
            String::new()
        } else {
            format!("(result {})", ret.compile($ctx)?)
        }
    }};
}

#[macro_export]
macro_rules! compile_args {
    ($function: expr, $ctx: expr) => {
        format!(
            "(param {})",
            join!(
                $function
                    .arguments
                    .iter()
                    .map(|(_, typ)| typ.compile($ctx))
                    .collect::<Option<Vec<_>>>()?
            )
        )
    };
}

#[macro_export]
macro_rules! compile_op {
    ($oper: expr, $ctx: expr, $lhs: expr, $rhs: expr) => {{
        let mut oper = $oper.to_string();
        let ret = type_check!($lhs, $rhs, $ctx)?.compile($ctx)?;
        let [lhs, rhs] = [$lhs.compile($ctx)?, $rhs.compile($ctx)?];
        if ret == "f32" {
            oper = $oper.replace("_s", "")
        }
        format!("({ret}.{oper} {lhs} {rhs})",)
    }};
    ($oper: expr, $ctx: expr, $term: expr) => {{
        let ret = $term.type_infer($ctx)?.compile($ctx)?;
        format!("({}.{} {})", ret, $oper, $term.compile($ctx)?)
    }};
}

#[macro_export]
macro_rules! offset_calc {
    ($dict: expr, $offset: expr, $typ: expr) => {
        Expr::Operator(Box::new(Op::Add(
            Expr::Operator(Box::new(Op::Transmute(*$dict.clone(), Type::Integer))),
            Expr::Literal(Value::Integer($offset.clone())),
        )))
    };
}

#[macro_export]
macro_rules! address_calc {
    ($array: expr, $index: expr, $typ: expr) => {
        Expr::Operator(Box::new(Op::Add(
            Expr::Operator(Box::new(Op::Add(
                Expr::Literal(Value::Integer(BYTES)),
                Expr::Operator(Box::new(Op::Transmute(*$array.clone(), Type::Integer))),
            ))),
            Expr::Operator(Box::new(Op::Mul(
                Expr::Operator(Box::new(Op::Mod(
                    *$index.clone(),
                    Expr::Peek($array.clone(), Type::Integer),
                ))),
                Expr::Literal(Value::Integer(BYTES)),
            ))),
        )))
    };
}

#[macro_export]
macro_rules! overload {
    ($self: expr, $ctx: expr, $method: ident) => {{
        let mut overload = || {
            if let Some((lhs, rhs)) = $self.binop_term() {
                let lhs_typ = lhs.type_infer($ctx)?.compress_alias($ctx);
                let rhs_typ = rhs.type_infer($ctx)?.compress_alias($ctx);
                let key = ($self.get_overload_id()?, (lhs_typ, rhs_typ));
                let func = key_from_value!(&$ctx.overload, key)?;
                Some(Expr::Call(func.clone(), vec![lhs, rhs]).$method($ctx))
            } else if let Op::Cast(lhs, rhs) = $self.clone() {
                let lhs_typ = lhs.type_infer($ctx)?.compress_alias($ctx);
                let rhs_typ = rhs.type_infer($ctx)?.compress_alias($ctx);
                let key = ($self.get_overload_id()?, (lhs_typ, rhs_typ));
                let func = key_from_value!(&$ctx.overload, key)?;
                Some(Expr::Call(func.clone(), vec![lhs]).$method($ctx))
            } else {
                None
            }
        };
        if let Some(overloaded) = overload() {
            return overloaded;
        }
    }};
}

#[macro_export]
macro_rules! check_args {
    ($args: expr, $ctx: expr) => {
        for arg in $args {
            let Expr::Operator(oper) = arg else {
                let msg = "function argument definition needs type annotation";
                $ctx.error = Some(msg.to_string());
                return None;
            };
            let Op::Cast(Expr::Variable(name), typ) = *oper.clone() else {
                let msg = "function argument name should be identifier";
                $ctx.error = Some(msg.to_string());
                return None;
            };
            if let Some(typ) = typ.type_infer($ctx) {
                $ctx.argument.insert(name.to_string(), typ);
            } else {
                $ctx.argument.insert(name.to_string(), typ);
            }
        }
    };
}

#[macro_export]
macro_rules! key_from_value {
    ($map: expr, $value: expr) => {
        $map.into_iter()
            .find_map(|(k, v)| if *v == $value { Some(k) } else { None })
    };
}

#[macro_export]
macro_rules! ok {
    ($result:expr) => {
        if let Ok(val) = $result {
            Some(val)
        } else {
            None
        }
    };
}

#[macro_export]
macro_rules! join {
    ($x:expr) => {
        $x.join(&SPACE[0].to_string())
    };
}
