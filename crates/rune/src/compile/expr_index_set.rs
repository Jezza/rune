use crate::ast;
use crate::compiler::{Compiler, Needs};
use crate::error::CompileResult;
use crate::traits::Compile;
use runestick::Inst;

/// An expr index set operation.
impl Compile<(&ast::ExprIndexSet, Needs)> for Compiler<'_, '_> {
    fn compile(
        &mut self,
        (expr_index_set, needs): (&ast::ExprIndexSet, Needs),
    ) -> CompileResult<()> {
        let span = expr_index_set.span();
        log::trace!("ExprIndexSet => {:?}", self.source.source(span));

        self.compile((&*expr_index_set.value, Needs::Value))?;
        self.compile((&*expr_index_set.index, Needs::Value))?;
        self.compile((&*expr_index_set.target, Needs::Value))?;
        self.asm.push(Inst::IndexSet, span);

        // Encode a unit in case a value is needed.
        if needs.value() {
            self.asm.push(Inst::Unit, span);
        }

        Ok(())
    }
}
