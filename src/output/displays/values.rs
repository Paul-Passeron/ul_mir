use std::fmt::Display;

use escape_string::escape;

use crate::core::{
    Context,
    ctrl_flow::{BinOp, Constant, Operand, Place, ProjectionKind, RValue, UnOp},
    types::int::IntType,
};

pub struct OperandDisplayer<'a> {
    operand: &'a Operand,
    ctx: &'a Context,
}

impl<'a> OperandDisplayer<'a> {
    pub fn new(operand: &'a Operand, ctx: &'a Context) -> Self {
        Self { operand, ctx }
    }
}

impl Operand {
    pub fn display<'a>(&'a self, ctx: &'a Context) -> OperandDisplayer<'a> {
        OperandDisplayer::new(self, ctx)
    }
}

pub struct RValueDisplayer<'a> {
    rvalue: &'a RValue,
    ctx: &'a Context,
}

impl<'a> RValueDisplayer<'a> {
    pub fn new(rvalue: &'a RValue, ctx: &'a Context) -> Self {
        Self { rvalue, ctx }
    }
}

impl RValue {
    pub fn display<'a>(&'a self, ctx: &'a Context) -> RValueDisplayer<'a> {
        RValueDisplayer::new(self, ctx)
    }
}

impl<'a> Display for OperandDisplayer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.operand {
            Operand::Copy(place) => write!(f, "copy({place})"),
            Operand::Move(place) => write!(f, "move({place})"),
            Operand::Constant(constant) => write!(f, "{constant}"),
            Operand::Call { func, args } => write!(
                f,
                "call {}({})",
                self.ctx.functions[func].name,
                args.iter()
                    .map(|arg| format!("{}", OperandDisplayer::new(arg, self.ctx)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl Display for RValueDisplayer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.rvalue {
            RValue::Use(operand) => write!(f, "{}", operand.display(self.ctx)),
            RValue::BinOp(bin_op, lhs, rhs) => {
                write!(
                    f,
                    "({} {} {})",
                    lhs.display(self.ctx),
                    bin_op,
                    rhs.display(self.ctx)
                )
            }
            RValue::UnOp(un_op, operand) => write!(f, "{}{}", un_op, operand.display(self.ctx)),
            RValue::Cast(_, operand, mir_type) => {
                write!(
                    f,
                    "cast {} to {}",
                    operand.display(self.ctx),
                    mir_type.display(self.ctx)
                )
            }
            RValue::Ref(place) => write!(f, "&{place}"),
            RValue::Len(place) => write!(f, "len({place})"),
            RValue::Aggregate(operands) => write!(
                f,
                "aggregate ({})",
                operands
                    .iter()
                    .map(|x| format!("{}", x.display(self.ctx)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Not => write!(f, "!"),
            UnOp::Minus => write!(f, "-"),
        }
    }
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Mod => write!(f, "%"),
            BinOp::ShiftRight => write!(f, ">>"),
            BinOp::ShiftLeft => write!(f, "<<"),
            BinOp::Neq => write!(f, "!="),
            BinOp::Leq => write!(f, "<="),
        }
    }
}

impl Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Place::Local(id) => write!(f, "%{}", id),
            Place::Projection(place, projection_kind) => match projection_kind {
                ProjectionKind::Field(field) => write!(f, "{place}.{field}"),
                ProjectionKind::Index(index) => write!(f, "{place}[{index}]"),
                ProjectionKind::Deref => write!(f, "*{place}"),
            },
        }
    }
}

impl Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Constant::Int(value, mir_type) => match mir_type {
                IntType::I8 => write!(f, "{}", *value as i8),
                IntType::I16 => write!(f, "{}", *value as i16),
                IntType::I32 => write!(f, "{}", *value as i32),
                IntType::I64 => write!(f, "{}", *value as i64),
                IntType::U8 => write!(f, "{}", *value as u8),
                IntType::U16 => write!(f, "{}", *value as u16),
                IntType::U32 => write!(f, "{}", *value as u32),
                IntType::U64 => write!(f, "{}", *value as u64),
                IntType::Bool => write!(f, "{}", *value == 0),
            },
            Constant::Bool(b) => write!(f, "{b}"),
            Constant::Str(s) => write!(f, "\\\"{}\\\"", escape(&escape(s)).replace("\\ ", " ")),
            Constant::Null => write!(f, "NULL"),
        }
    }
}
