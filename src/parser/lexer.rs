use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, multispace0, space0},
    combinator::{map, map_res, opt},
    sequence::{preceded, tuple},
    IResult,
};
use heapless::Vec;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    G(u16),
    M(u16),
    Axis(char, f32),
    Feedrate(f32),
    SpindleSpeed(f32),
    Comment(&'a str),
    LineNumber(u32),
}

fn is_axis_char(c: char) -> bool {
    matches!(c, 'X' | 'Y' | 'Z' | 'A' | 'B' | 'C' | 'U' | 'V' | 'W' | 'E')
}

fn parse_float(input: &str) -> IResult<&str, f32> {
    map_res(
        tuple((opt(char('-')), digit1, opt(preceded(char('.'), digit1)))),
        |(sign, int_part, frac_part)| {
            let mut s = String::new();
            if sign.is_some() { s.push('-'); }
            s.push_str(int_part);
            if let Some(frac) = frac_part {
                s.push('.');
                s.push_str(frac);
            }
            s.parse::<f32>()
        },
    )(input)
}

fn parse_g_code(input: &str) -> IResult<&str, Token> {
    map(preceded(char('G'), map_res(digit1, |s: &str| s.parse::<u16>())), Token::G)(input)
}

fn parse_m_code(input: &str) -> IResult<&str, Token> {
    map(preceded(char('M'), map_res(digit1, |s: &str| s.parse::<u16>())), Token::M)(input)
}

fn parse_axis(input: &str) -> IResult<&str, Token> {
    map(
        tuple((take_while1(is_axis_char), parse_float)),
        |(axis_char, val)| Token::Axis(axis_char.chars().next().unwrap(), val),
    )(input)
}

fn parse_feedrate(input: &str) -> IResult<&str, Token> {
    map(preceded(char('F'), parse_float), Token::Feedrate)(input)
}

fn parse_spindle_speed(input: &str) -> IResult<&str, Token> {
    map(preceded(char('S'), parse_float), Token::SpindleSpeed)(input)
}

fn parse_line_number(input: &str) -> IResult<&str, Token> {
    map(preceded(char('N'), map_res(digit1, |s: &str| s.parse::<u32>())), Token::LineNumber)(input)
}

fn parse_comment(input: &str) -> IResult<&str, Token> {
    map(
        alt((
            preceded(char(';'), nom::character::complete::not_line_ending),
            preceded(char('('), nom::bytes::complete::take_until(")")),
        )),
        Token::Comment,
    )(input)
}

fn parse_token(input: &str) -> IResult<&str, Token> {
    preceded(
        space0,
        alt((
            parse_g_code,
            parse_m_code,
            parse_axis,
            parse_feedrate,
            parse_spindle_speed,
            parse_line_number,
            parse_comment,
        )),
    )(input)
}

pub fn lexer(input: &str) -> IResult<&str, Vec<Token, 64>> {
    let mut tokens = Vec::new();
    let mut current_input = input;

    while !current_input.is_empty() {
        let (next_input, token) = parse_token(current_input)?;
        tokens.push(token).unwrap();
        current_input = next_input;
        // Handle end of line comments
        if let Ok((remaining, _)) = preceded(multispace0, nom::character::complete::line_ending)(current_input) {
            current_input = remaining;
            break;
        }
    }
    Ok((current_input, tokens))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::String; // Required for the test environment

    #[test]
    fn test_lexer() {
        let gcode = "G1 X10.5 Y-20 Z5 F500 ; this is a comment";
        let (_, tokens) = lexer(gcode).unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::G(1));
        assert_eq!(tokens[1], Token::Axis('X', 10.5));
        assert_eq!(tokens[2], Token::Axis('Y', -20.0));
        assert_eq!(tokens[3], Token::Axis('Z', 5.0));
        assert_eq!(tokens[4], Token::Feedrate(500.0));
    }
}
