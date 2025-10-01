use crate::*;

#[derive(Clone, Debug)]
pub struct Block(pub Vec<Stmt>);

impl Node for Block {
    fn parse(source: &str) -> Option<Block> {
        Some(Block(
            tokenize(source, &[";"], false, false, false)?
                .iter()
                .map(String::as_str)
                .map(Stmt::parse)
                .collect::<Option<Vec<_>>>()?,
        ))
    }

    fn compile(&self, ctx: &mut Compiler) -> Option<String> {
        let (mut result, Block(block)) = (vec![], self);
        for (n, line) in block.iter().enumerate() {
            let mut output = line.compile(ctx)?;
            if n != block.len() - 1 {
                if !matches!(line.infer(ctx)?, Type::Void) {
                    output.push_str("(drop)");
                }
            }
            result.push(output);
        }
        Some(join!(result))
    }

    fn infer(&self, ctx: &mut Compiler) -> Option<Type> {
        let var_ctx = ctx.variable.clone();
        let fun_ctx = ctx.function.clone();
        let mcr_ctx = ctx.r#macro.clone();

        let Block(block) = self.clone();
        let mut result = Type::Void;
        for line in block {
            result = line.infer(ctx)?;
        }

        ctx.variable = var_ctx;
        ctx.function = fun_ctx;
        ctx.r#macro = mcr_ctx;
        Some(result)
    }
}
