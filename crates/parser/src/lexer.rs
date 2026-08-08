use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, char, digit1, multispace0, space0},
    combinator::{map, map_res, opt},
    multi::many0,
    sequence::{delimited, preceded, tuple},
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
    P(f32),
    Comment(&'a str),
    LineNumber(u32),
}

fn parse_u16(input: &[u8]) -> IResult<&[u8], u16> {
    map_res(digit1, |s: &[u8]| core::str::from_utf8(s).unwrap().parse::<u16>())(input)
}

fn parse_f32(input: &[u8]) -> IResult<&[u8], f32> {
    map_res(
        tuple((opt(char('-')), digit1, opt(preceded(char('.'), digit1)))),
        |(sign, int_part, frac_part)| {
            let mut s = core::str::from_utf8(int_part).unwrap().to_string();
            if let Some(frac) = frac_part {
                s.push('.');
                s.push_str(core::str::from_utf8(frac).unwrap());
            }
            if sign.is_some() {
                s.insert(0, '-');
            }
            s.parse::<f32>()
        },
    )(input)
}

fn parse_u32(input: &[u8]) -> IResult<&[u8], u32> {
    map_res(digit1, |s: &[u8]| core::str::from_utf8(s).unwrap().parse::<u32>())(input)
}

fn g_code(input: &[u8]) -> IResult<&[u8], Token> {
    map(preceded(char('G'), parse_u16), Token::G)(input)
}

fn m_code(input: &[u8]) -> IResult<&[u8], Token> {
    map(preceded(char('M'), parse_u16), Token::M)(input)
}

fn axis(input: &[u8]) -> IResult<&[u8], Token> {
    map(
        tuple((
            alt((
                char('X'), char('Y'), char('Z'), char('A'), char('B'), char('C'), char('E'),
            )),
            parse_f32,
        )),
        |(axis, value)| Token::Axis(axis, value),
    )(input)
}

fn feedrate(input: &[u8]) -> IResult<&[u8], Token> {
    map(preceded(char('F'), parse_f32), Token::Feedrate)(input)
}

fn spindle_speed(input: &[u8]) -> IResult<&[u8], Token> {
    map(preceded(char('S'), parse_f32), Token::SpindleSpeed)(input)
}

fn p_value(input: &[u8]) -> IResult<&[u8], Token> {
    map(preceded(char('P'), parse_f32), Token::P)(input)
}

fn comment(input: &[u8]) -> IResult<&[u8], Token> {
    map(
        alt((
            delimited(char('('), is_not(")"), char(')')),
            preceded(char(';'), is_not("\r\n")),
        )),
        |s: &[u8]| Token::Comment(core::str::from_utf8(s).unwrap()),
    )(input)
}

fn line_number(input: &[u8]) -> IResult<&[u8], Token> {
    map(preceded(char('N'), parse_u32), Token::LineNumber)(input)
}

fn token(input: &[u8]) -> IResult<&[u8], Token> {
    preceded(
        space0,
        alt((
            g_code,
            m_code,
            axis,
            feedrate,
            spindle_speed,
            p_value,
            comment,
            line_number,
        )),
    )(input)
}

pub fn lexer(input: &[u8]) -> IResult<&[u8], Vec<Token, 64>> {
    preceded(multispace0, many0(token))(input)
}
