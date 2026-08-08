use crate::parser::modal::ModalState;
use crate::parser::lexer::Token;
use heapless::Vec;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AstNode {
    LinearMove { x: Option<f32>, y: Option<f32>, z: Option<f32>, e: Option<f32>, f: Option<f32> },
    RapidMove { x: Option<f32>, y: Option<f32>, z: Option<f32>, e: Option<f32> },
    ArcMove { x: Option<f32>, y: Option<f32>, i: Option<f32>, j: Option<f32>, f: Option<f32> },
    Dwell { p: u32 },
    ToolChange { t: u16 },
    SetUnits(super::modal::Units),
    SetPositioning(super::modal::Positioning),
    // ... other AST nodes
}

pub fn build_ast<'a>(tokens: &Vec<Token<'a>, 64>, state: &mut ModalState) -> Result<Option<AstNode>, &'static str> {
    let mut g_code = None;
    let mut m_code = None;
    let mut x = None;
    let mut y = None;
    let mut z = None;
    let mut e = None;
    let mut f = None;
    let mut i = None;
    let mut j = None;
    let mut p = None;
    let mut t = None;

    for token in tokens.iter() {
        match *token {
            Token::G(code) => g_code = Some(code),
            Token::M(code) => m_code = Some(code),
            Token::Axis('X', val) => x = Some(val),
            Token::Axis('Y', val) => y = Some(val),
            Token::Axis('Z', val) => z = Some(val),
            Token::Axis('E', val) => e = Some(val),
            Token::Feedrate(val) => f = Some(val),
            // ... handle other tokens
            _ => {}
        }
    }

    let g = g_code.unwrap_or(state.motion_mode);

    match g {
        0 => Ok(Some(AstNode::RapidMove { x, y, z, e })),
        1 => Ok(Some(AstNode::LinearMove { x, y, z, e, f })),
        2 | 3 => Ok(Some(AstNode::ArcMove { x, y, i, j, f })),
        4 => Ok(Some(AstNode::Dwell { p: p.unwrap_or(0) })),
        20 => { state.units = super::modal::Units::Inches; Ok(Some(AstNode::SetUnits(super::modal::Units::Inches))) },
        21 => { state.units = super::modal::Units::Millimeters; Ok(Some(AstNode::SetUnits(super::modal::Units::Millimeters))) },
        90 => { state.positioning = super::modal::Positioning::Absolute; Ok(Some(AstNode::SetPositioning(super::modal::Positioning::Absolute))) },
        91 => { state.positioning = super::modal::Positioning::Relative; Ok(Some(AstNode::SetPositioning(super::modal::Positioning::Relative))) },
        _ => Ok(None),
    }
}
