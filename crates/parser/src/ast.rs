use crate::lexer::Token;
use crate::modal::ModalState;
use heapless::Vec;

#[derive(Debug, PartialEq)]
pub enum AstNode {
    LinearMove {
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
        a: Option<f32>,
        b: Option<f32>,
        c: Option<f32>,
        e: Option<f32>,
        feedrate: Option<f32>,
    },
    RapidMove {
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
        a: Option<f32>,
        b: Option<f32>,
        c: Option<f32>,
        e: Option<f32>,
    },
    ArcMove {
        // ...
    },
    Dwell(f32),
    ToolChange(u16),
    SpindleControl(f32),
    SetModalState(ModalState),
    Comment(heapless::String<64>),
}

pub fn parse_line<'a>(tokens: &Vec<Token<'a>, 64>, state: &mut ModalState) -> Result<Option<AstNode>, ()> {
    let mut g_code = None;
    let mut m_code = None;
    let mut x = None;
    let mut y = None;
    let mut z = None;
    let mut a = None;
    let mut b = None;
    let mut c = None;
    let mut e = None;
    let mut feedrate = None;
    let mut spindle_speed = None;
    let mut p_val = None;
    let mut comment = None;

    for token in tokens {
        match *token {
            Token::G(code) => g_code = Some(code),
            Token::M(code) => m_code = Some(code),
            Token::Axis(axis, value) => match axis {
                'X' => x = Some(value),
                'Y' => y = Some(value),
                'Z' => z = Some(value),
                'A' => a = Some(value),
                'B' => b = Some(value),
                'C' => c = Some(value),
                'E' => e = Some(value),
                _ => {}
            },
            Token::Feedrate(value) => feedrate = Some(value),
            Token::SpindleSpeed(value) => spindle_speed = Some(value),
            Token::P(value) => p_val = Some(value),
            Token::Comment(text) => comment = Some(text),
            Token::LineNumber(_) => {}
        }
    }

    if let Some(text) = comment {
        let mut s = heapless::String::new();
        s.push_str(text).unwrap();
        return Ok(Some(AstNode::Comment(s)));
    }

    match g_code {
        Some(0) => Ok(Some(AstNode::RapidMove { x, y, z, a, b, c, e })),
        Some(1) => Ok(Some(AstNode::LinearMove { x, y, z, a, b, c, e, feedrate })),
        Some(4) => {
            if let Some(p) = p_val {
                Ok(Some(AstNode::Dwell(p)))
            } else {
                Err(())
            }
        }
        Some(17) => {
            state.plane = crate::modal::Plane::XY;
            Ok(Some(AstNode::SetModalState(*state)))
        }
        Some(18) => {
            state.plane = crate::modal::Plane::XZ;
            Ok(Some(AstNode::SetModalState(*state)))
        }
        Some(19) => {
            state.plane = crate::modal::Plane::YZ;
            Ok(Some(AstNode::SetModalState(*state)))
        }
        // ... and so on for all G-codes
        _ => Ok(None),
    }
}
