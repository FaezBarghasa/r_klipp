//! Abstract Syntax Tree (AST) for parsed G-Code commands.

use crate::lexer::Token;
use crate::modal::ModalState;

#[derive(Debug, Clone, PartialEq)]
pub enum AstNode<'a> {
    RapidMove {
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
        e: Option<f32>,
        feedrate: Option<f32>,
    },
    LinearMove {
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
        e: Option<f32>,
        feedrate: Option<f32>,
    },
    ArcMove {
        clockwise: bool,
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
        i: Option<f32>,
        j: Option<f32>,
        r: Option<f32>,
        feedrate: Option<f32>,
    },
    Dwell(f32),
    Home {
        x: bool,
        y: bool,
        z: bool,
    },
    SetPosition {
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
        e: Option<f32>,
    },
    SetHotendTemp {
        temp: f32,
        wait: bool,
    },
    SetBedTemp {
        temp: f32,
        wait: bool,
    },
    SetFanSpeed(f32),
    FanOff,
    EmergencyStop,
    ToolChange(u8),
    SetModalState(ModalState),
    Comment(&'a str),
    UnknownG(u16),
    UnknownM(u16),
}

/// Parses a tokenized slice into an `AstNode`.
pub fn parse_tokens<'a>(tokens: &[Token<'a>], state: &mut ModalState) -> Result<Option<AstNode<'a>>, ()> {
    if tokens.is_empty() {
        return Ok(None);
    }

    let mut g_code = None;
    let mut m_code = None;
    let mut t_code = None;
    let mut x = None;
    let mut y = None;
    let mut z = None;
    let mut e = None;
    let mut i_offset = None;
    let mut j_offset = None;
    let mut r_radius = None;
    let mut feedrate = None;
    let mut s_val = None;
    let mut p_val = None;
    let mut comment_text = None;

    for token in tokens {
        match *token {
            Token::G(code) => g_code = Some(code),
            Token::M(code) => m_code = Some(code),
            Token::T(tool) => t_code = Some(tool),
            Token::Axis(axis, value) => match axis {
                'X' => x = Some(value),
                'Y' => y = Some(value),
                'Z' => z = Some(value),
                'E' => e = Some(value),
                'I' => i_offset = Some(value),
                'J' => j_offset = Some(value),
                'R' => r_radius = Some(value),
                _ => {}
            },
            Token::Feedrate(value) => feedrate = Some(value),
            Token::SpindleSpeed(value) => s_val = Some(value),
            Token::P(value) => p_val = Some(value),
            Token::Comment(text) => comment_text = Some(text),
            Token::LineNumber(_) | Token::Checksum(_) => {}
        }
    }

    // Pure comment line
    if g_code.is_none() && m_code.is_none() && t_code.is_none() && x.is_none() && y.is_none() && z.is_none() && e.is_none() {
        if let Some(text) = comment_text {
            return Ok(Some(AstNode::Comment(text)));
        }
        return Ok(None);
    }

    if let Some(t) = t_code {
        return Ok(Some(AstNode::ToolChange(t)));
    }

    if let Some(g) = g_code {
        match g {
            0 => Ok(Some(AstNode::RapidMove { x, y, z, e, feedrate })),
            1 => Ok(Some(AstNode::LinearMove { x, y, z, e, feedrate })),
            2 => Ok(Some(AstNode::ArcMove { clockwise: true, x, y, z, i: i_offset, j: j_offset, r: r_radius, feedrate })),
            3 => Ok(Some(AstNode::ArcMove { clockwise: false, x, y, z, i: i_offset, j: j_offset, r: r_radius, feedrate })),
            4 => {
                let dwell_time = p_val.or(s_val.map(|s| s * 1000.0)).unwrap_or(0.0);
                Ok(Some(AstNode::Dwell(dwell_time)))
            }
            28 => {
                let home_all = x.is_none() && y.is_none() && z.is_none();
                Ok(Some(AstNode::Home {
                    x: home_all || x.is_some(),
                    y: home_all || y.is_some(),
                    z: home_all || z.is_some(),
                }))
            }
            90 => {
                state.distance_mode = crate::modal::DistanceMode::Absolute;
                Ok(Some(AstNode::SetModalState(*state)))
            }
            91 => {
                state.distance_mode = crate::modal::DistanceMode::Relative;
                Ok(Some(AstNode::SetModalState(*state)))
            }
            92 => Ok(Some(AstNode::SetPosition { x, y, z, e })),
            _ => Ok(Some(AstNode::UnknownG(g))),
        }
    } else if let Some(m) = m_code {
        match m {
            104 => Ok(Some(AstNode::SetHotendTemp { temp: s_val.unwrap_or(0.0), wait: false })),
            109 => Ok(Some(AstNode::SetHotendTemp { temp: s_val.unwrap_or(0.0), wait: true })),
            140 => Ok(Some(AstNode::SetBedTemp { temp: s_val.unwrap_or(0.0), wait: false })),
            190 => Ok(Some(AstNode::SetBedTemp { temp: s_val.unwrap_or(0.0), wait: true })),
            106 => Ok(Some(AstNode::SetFanSpeed(s_val.unwrap_or(255.0)))),
            107 => Ok(Some(AstNode::FanOff)),
            112 => Ok(Some(AstNode::EmergencyStop)),
            _ => Ok(Some(AstNode::UnknownM(m))),
        }
    } else if x.is_some() || y.is_some() || z.is_some() || e.is_some() {
        // Modal linear move
        Ok(Some(AstNode::LinearMove { x, y, z, e, feedrate }))
    } else {
        Ok(None)
    }
}
