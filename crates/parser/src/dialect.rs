use crate::ast::AstNode;
use crate::modal::ModalState;

pub enum MachineCommand {
    LinearMove,
    RapidMove,
    Dwell,
    ToolChange,
    SpindleControl,
    // ... and so on
}

pub enum ParseError {
    UnsupportedCommand,
    InvalidParameters,
}

pub trait MachineDialect {
    fn interpret(&self, node: &AstNode, state: &mut ModalState) -> Result<MachineCommand, ParseError>;
}

pub struct FdmPrinterDialect;

impl MachineDialect for FdmPrinterDialect {
    fn interpret(&self, node: &AstNode, state: &mut ModalState) -> Result<MachineCommand, ParseError> {
        match node {
            AstNode::LinearMove { .. } => Ok(MachineCommand::LinearMove),
            _ => Err(ParseError::UnsupportedCommand),
        }
    }
}

pub struct CncRouterDialect;

impl MachineDialect for CncRouterDialect {
    fn interpret(&self, node: &AstNode, state: &mut ModalState) -> Result<MachineCommand, ParseError> {
        match node {
            AstNode::LinearMove { .. } => Ok(MachineCommand::LinearMove),
            _ => Err(ParseError::UnsupportedCommand),
        }
    }
}

pub struct WireEdmDialect;

impl MachineDialect for WireEdmDialect {
    fn interpret(&self, node: &AstNode, state: &mut ModalState) -> Result<MachineCommand, ParseError> {
        match node {
            AstNode::LinearMove { .. } => Ok(MachineCommand::LinearMove),
            _ => Err(ParseError::UnsupportedCommand),
        }
    }
}
