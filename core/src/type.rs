use crate::*;

pub type Dict = IndexMap<String, Type>;
pub type Enum = IndexSet<String>;
#[derive(Clone, Debug)]
pub enum Type {
    Integer,
    Number,
    Bool,
    String,
    Array(Box<Type>),
    Dict(Dict),
    Enum(Enum),
    Alias(String),
    Any,
    Void,
}

impl Node for Type {
    fn parse(source: &str) -> Option<Type> {
        match source.trim() {
            "int" => Some(Type::Integer),
            "num" => Some(Type::Number),
            "bool" => Some(Type::Bool),
            "str" => Some(Type::String),
            "void" => Some(Type::Void),
            "any" => Some(Type::Any),
            source => {
                let mut source = source.trim().to_owned();
                if source.starts_with("[") && source.ends_with("]") {
                    let source = source.get(1..source.len() - 1)?.trim();
                    Some(Type::Array(Box::new(Type::parse(source)?)))
                } else if source.starts_with("@{") && source.ends_with("}") {
                    let source = source.get(2..source.len() - 1)?.trim();
                    let mut result = IndexMap::new();
                    for line in tokenize(source, &[","], false, true, false)? {
                        let (name, value) = line.split_once(":")?;
                        let mut name = name.trim().to_owned();
                        if !is_identifier(&mut name) {
                            return None;
                        };
                        result.insert(name, Type::parse(value)?);
                    }
                    Some(Type::Dict(result))
                } else if source.starts_with("(") && source.ends_with(")") {
                    let source = source.get(1..source.len() - 1)?.trim();
                    let tokens = tokenize(source, &["|"], false, true, false)?;
                    let mut result: IndexSet<String> = IndexSet::new();
                    for key in tokens {
                        let mut value = key.trim().to_owned();
                        if !is_identifier(&mut value) {
                            return None;
                        };
                        result.insert(value);
                    }
                    Some(Type::Enum(result))
                } else if is_identifier(&mut source) {
                    Some(Type::Alias(source.to_string()))
                } else {
                    None
                }
            }
        }
    }

    fn compile(&self, ctx: &mut Compiler) -> Option<String> {
        Some(
            match self.infer(ctx)? {
                Type::Number => "f32",
                Type::Integer
                | Type::Bool
                | Type::String
                | Type::Array(_)
                | Type::Dict(_)
                | Type::Enum(_) => "i32",
                _ => return None,
            }
            .to_string(),
        )
    }

    fn infer(&self, ctx: &mut Compiler) -> Option<Type> {
        self.solve_alias(ctx, vec![])
    }
}

impl Type {
    pub fn solve_alias(&self, ctx: &mut Compiler, xpct: Vec<Type>) -> Option<Type> {
        for x in &xpct {
            if x.restore_alias(ctx) == self.restore_alias(ctx) {
                return Some(self.restore_alias(ctx));
            }
        }
        match self {
            Type::Alias(name) => {
                let Some(typ) = ctx.type_alias.get(name).cloned() else {
                    let msg = format!("undefined type alias `{name}`");
                    ctx.error = Some(msg);
                    return None;
                };
                typ.solve_alias(ctx, xpct.clone())
            }
            Type::Array(typ) => Some(Type::Array(Box::new(
                typ.solve_alias(ctx, [xpct.clone(), vec![self.clone()]].concat())?,
            ))),
            Type::Dict(dict) => {
                let mut a = IndexMap::new();
                for (name, typ) in dict {
                    let typ = typ.solve_alias(ctx, [xpct.clone(), vec![self.clone()]].concat())?;
                    a.insert(name.clone(), typ.clone());
                }
                Some(Type::Dict(a))
            }
            _ => Some(self.clone()),
        }
    }

    pub fn restore_alias(&self, ctx: &Compiler) -> Type {
        let typ = match self {
            Type::Array(typ) => Type::Array(Box::new(typ.restore_alias(ctx))),
            Type::Dict(dict) => Type::Dict(
                dict.iter()
                    .map(|(key, typ)| (key.clone(), (typ.restore_alias(ctx))))
                    .collect(),
            ),
            _ => self.clone(),
        };
        let mut aliases = ctx.type_alias.iter();
        if let Some((alias, _)) = aliases.find(|(_, v)| **v == typ) {
            if *alias != Type::Any.format() {
                Type::Alias(alias.clone())
            } else {
                typ
            }
        } else {
            typ
        }
    }

    pub fn compare(&self, other: &Self, ctx: &mut Compiler) -> bool {
        match (self, other) {
            (Type::Integer, Type::Integer) => true,
            (Type::Number, Type::Number) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::String, Type::String) => true,
            (Type::Void, Type::Void) => true,
            (Type::Any, typ) | (typ, Type::Any) => {
                if let Some(any) = ctx.type_alias.get(&Type::Any.format()) {
                    typ == any
                } else {
                    ctx.type_alias.insert(Type::Any.format(), typ.clone());
                    true
                }
            }
            (Type::Dict(a), Type::Dict(b)) => {
                a.iter().zip(b).all(|((_, a), (_, b))| a.compare(b, ctx))
            }
            (Type::Enum(a), Type::Enum(b)) => a == b,
            (Type::Array(a), Type::Array(b)) => a.clone().compare(b, ctx),
            (Type::Alias(a), Type::Alias(b)) => a == b,
            _ => false,
        }
    }

    pub fn polymorphism(&self, ctx: &mut Compiler) -> Option<Type> {
        match self {
            Type::Any => ctx.type_alias.swap_remove(&Type::Any.format()),
            Type::Dict(dict) => Some(Type::Dict(
                dict.iter()
                    .map(|(key, typ)| Some((key.clone(), typ.polymorphism(ctx)?)))
                    .collect::<Option<IndexMap<String, Type>>>()?,
            )),
            Type::Array(typ) => Some(Type::Array(Box::new(typ.polymorphism(ctx)?))),
            primitive => Some(primitive.clone()),
        }
    }

    pub fn format(&self) -> String {
        match self {
            Type::Integer => "int".to_string(),
            Type::Number => "num".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "str".to_string(),
            Type::Void => "void".to_string(),
            Type::Any => "any".to_string(),
            Type::Dict(dict) => format!(
                "@{{ {} }}",
                dict.iter()
                    .map(|(key, typ)| format!("{key}: {}", typ.format()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Enum(e) => format!(
                "( {} )",
                e.iter().cloned().collect::<Vec<String>>().join(" | ")
            ),
            Type::Array(typ) => format!("[{}]", typ.format()),
            Type::Alias(name) => name.to_string(),
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Integer, Type::Integer) => true,
            (Type::Number, Type::Number) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::String, Type::String) => true,
            (Type::Void, Type::Void) => true,
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Dict(a), Type::Dict(b)) => a == b,
            (Type::Enum(a), Type::Enum(b)) => a == b,
            (Type::Array(a), Type::Array(b)) => a == b,
            (Type::Alias(a), Type::Alias(b)) => a == b,
            _ => false,
        }
    }
}
