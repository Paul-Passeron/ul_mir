use std::fmt::{self, Result};

use crate::{
    core::{
        Context,
        ctrl_flow::{
            BlockId,
            basic_blocks::{BasicBlock, Statement, Terminator},
            function::{FunctionData, MirFunId},
        },
    },
    output::dot::DotOutput,
};

pub struct DotFormatter<'a> {
    ctx: &'a Context,
    fun_id: MirFunId,
}

impl<'a> DotFormatter<'a> {
    pub fn new(ctx: &'a Context, fun_id: MirFunId) -> Self {
        Self { ctx, fun_id }
    }

    fn write_function<W: std::fmt::Write>(&self, f: &mut W) -> Result {
        let fun = &self.ctx.functions[&self.fun_id];
        let data = fun.get_function_data();
        if data.is_none() {
            return writeln!(f, "digraph \"{}\" {{}}", fun.name);
        }
        let data = data.unwrap();
        writeln!(f, "digraph \"{}\" {{", fun.name)?;
        writeln!(f, "  labeljust=l")?;
        writeln!(
            f,
            "  node [shape=box, style=filled, fillcolor=lightblue, fontname=\"monospace\"];"
        )?;
        writeln!(f, "  edge [fontname=\"monospace\"];")?;
        writeln!(f, "  rankdir=TB;")?;

        for (block_id, block_opt) in &data.blocks {
            if let Some(block) = block_opt {
                self.write_block(*block_id, block, data, f)?;
            }
        }

        for (block_id, block_opt) in &data.blocks {
            if let Some(block) = block_opt {
                self.write_edges(*block_id, &block.terminator, f)?;
            }
        }

        write!(f, "}}")?;
        Ok(())
    }

    fn write_block<W: std::fmt::Write>(
        &self,
        block_id: BlockId,
        block: &BasicBlock,
        data: &FunctionData,
        f: &mut W,
    ) -> fmt::Result {
        let is_entry = block_id == data.entry_block;
        let style = if is_entry {
            " style=filled, fillcolor=lightgreen"
        } else {
            ""
        };
        let mut label = format!("bb{}", block_id);
        for stmt in &block.statements {
            label.push_str("\\l");
            label.push_str(&self.format_statement(stmt));
        }
        label.push_str("\\l---\\n");
        label.push_str(&self.format_terminator(&block.terminator));
        writeln!(f, "  bb{} [label=\"{}\"{}];", block_id, label, style)?;
        Ok(())
    }

    fn write_edges<W: std::fmt::Write>(
        &self,
        from: BlockId,
        terminator: &Terminator,
        writer: &mut W,
    ) -> std::fmt::Result {
        match terminator {
            Terminator::Goto(target) => {
                writeln!(writer, "  bb{} -> bb{};", from, target)?;
            }
            Terminator::Br {
                then_dst, else_dst, ..
            } => {
                writeln!(writer, "  bb{} -> bb{} [label=\"true\"];", from, then_dst)?;
                writeln!(writer, "  bb{} -> bb{} [label=\"false\"];", from, else_dst)?;
            }
            Terminator::Return(_) => {}
        }
        Ok(())
    }

    fn format_statement(&self, stmt: &Statement) -> String {
        match stmt {
            Statement::Assign(place, rvalue) => format!("{} = {}", place, rvalue.display(self.ctx)),
            Statement::StorageLive(id) => format!("live(%{})", id),
            Statement::StorageDead(id) => format!("dead(%{})", id),
            Statement::Nop => format!("nop"),
        }
    }

    fn format_terminator(&self, term: &Terminator) -> String {
        match term {
            Terminator::Return(operand) => format!(
                "ret{}",
                match operand {
                    Some(value) => {
                        format!(" {}", value.display(self.ctx))
                    }
                    None => format!(""),
                }
            ),
            Terminator::Goto(id) => format!("goto bb{}", id),
            Terminator::Br { discriminant, .. } => {
                format!("br {}", discriminant.display(self.ctx))
            }
        }
    }
}

impl<'a> DotOutput for DotFormatter<'a> {
    fn write_dot<W: std::fmt::Write>(&self, writer: &mut W) -> std::fmt::Result {
        self.write_function(writer)
    }
}
