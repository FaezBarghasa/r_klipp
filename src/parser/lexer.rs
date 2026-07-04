//! Zero-allocation G-code lexer and tokenizer for r_klipp.
//! This module uses `nom` to parse G-code lines into a sequence of tokens.
//! This file corresponds to Task 2.1 of the development plan.

#![cfg_attr(not(test), no_std)]

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, multispace0, one_of},
    combinator::{map, map_res, opt, recognize},
    multi::many0,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};
use heapless::Vec;
use lexical_core::{parse, FromLexical};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    G(u16),
    M(u16),
    Axis(char, f32),
    Feedrate(f32),
    SpindleSpeed(f32),
    Comment(&'a str),
    LineNumber(u32),
    Checksum(u8),
    Tool(u16),
}

/// Parses a floating-point number using lexical-core for `no_std` compatibility.
fn parse_f32(input: &str) -> IResult<&str, f32> {
    map_res(
        recognize(
            pair(
                opt(char('-')),
                alt((
                    map(pair(take_while1(|c: char| c.is_digit(10)), opt(preceded(char('.'), take_while1(|c: char| c.is_digit(10))))), |_| ()),
                    map(preceded(char('.'), take_while1(|c: char| c.is_digit(10))), |_| ()),
                )),
            )
        ),
        |s: &str| f32::from_lexical(s.as_bytes()),
    )(input)
}

/// Parses an integer number.
fn parse_u16(input: &str) -> IResult<&str, u16> {
    map_res(take_while1(|c: char| c.is_digit(10)), |s: &str| {
        u16::from_lexical(s.as_bytes())
    })(input)
}

fn parse_u32(input: &str) -> IResult<&str, u32> {
    map_res(take_while1(|c: char| c.is_digit(10)), |s: &str| {
        u32::from_lexical(s.as_bytes())
    })(input)
}

/// Parses a G-code word (e.g., 'G90', 'X10.5').
fn parse_word(input: &str) -> IResult<&str, Token> {
    let (input, code) = one_of("GMXYZABCDEFSTN*")(input)?;
    let (input, value) = parse_f32(input)?;

    let token = match code {
        'G' => Token::G(value as u16),
        'M' => Token::M(value as u16),
        'F' => Token::Feedrate(value),
        'S' => Token::SpindleSpeed(value),
        'T' => Token::Tool(value as u16),
        'N' => Token::LineNumber(value as u32),
        '*' => Token::Checksum(value as u8),
        'X' | 'Y' | 'Z' | 'A' | 'B' | 'C' | 'E' => Token::Axis(code, value),
        // This case should not be reached due to the `one_of` parser.
        // In a production system, we might log an error or handle it more gracefully.
        _ => return Err(nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Char))),
    };
    Ok((input, token))
}

/// Parses a G-code comment in parentheses, e.g., '(this is a comment)'.
fn parse_comment_paren(input: &str) -> IResult<&str, Token> {
    map(
        delimited(char('('), take_while(|c| c != ')'), char(')')),
        Token::Comment
    )(input)
}

/// Parses a G-code comment starting with a semicolon, e.g., '; this is a comment'
fn parse_comment_semicolon(input: &str) -> IResult<&str, Token> {
    map(
        preceded(char(';'), take_while(|c| c != '\n' && c != '\r')),
        Token::Comment
    )(input)
}

/// Parses either type of comment.
fn parse_comment(input: &str) -> IResult<&str, Token> {
    alt((parse_comment_paren, parse_comment_semicolon))(input)
}


/// Parses a single G-code line into a vector of tokens.
/// The vector has a fixed capacity of 64, which should be sufficient for any reasonable G-code line.
pub fn tokenize_line(input: &str) -> IResult<&str, Vec<Token, 64>> {
    many0(preceded(
        multispace0,
        alt((parse_word, parse_comment)),
    ))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_parse_simple_g_code() {
        let line = "G1 X10.5 Y-20 F300";
        let (_rem, tokens) = tokenize_line(line).unwrap();

        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0], Token::G(1)));

        match tokens[1] {
            Token::Axis(c, v) => {
                assert_eq!(c, 'X');
                assert_relative_eq!(v, 10.5);
            },
            _ => panic!("Expected Axis token for X"),
        }
        match tokens[2] {
            Token::Axis(c, v) => {
                assert_eq!(c, 'Y');
                assert_relative_eq!(v, -20.0);
            },
            _ => panic!("Expected Axis token for Y"),
        }
        match tokens[3] {
            Token::Feedrate(v) => assert_relative_eq!(v, 300.0),
            _ => panic!("Expected Feedrate token"),
        }
    }

    #[test]
    fn test_parse_with_paren_comment() {
        let line = "G90 (Absolute Positioning)";
        let (_rem, tokens) = tokenize_line(line).unwrap();

        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::G(90)));
        assert!(matches!(tokens[1], Token::Comment("Absolute Positioning")));
    }

    #[test]
    fn test_parse_with_semicolon_comment() {
        let line = "G91 ;Relative Positioning";
        let (_rem, tokens) = tokenize_line(line).unwrap();

        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::G(91)));
        assert!(matches!(tokens[1], Token::Comment("Relative Positioning")));
    }

    #[test]
    fn test_parse_line_number_and_checksum() {
        let line = "N10 G0 X0 Y0 *69";
        let (_rem, tokens) = tokenize_line(line).unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], Token::LineNumber(10)));
        assert!(matches!(tokens[1], Token::G(0)));
        assert!(matches!(tokens[4], Token::Checksum(69)));
    }

    #[test]
    fn test_float_parsing() {
        let line = "G1 X.5 Y-.25 Z0.0";
        let (_rem, tokens) = tokenize_line(line).unwrap();
        assert_eq!(tokens.len(), 4);
        match tokens[1] {
            Token::Axis(c, v) => { assert_eq!(c, 'X'); assert_relative_eq!(v, 0.5); },
            _ => panic!("Incorrect token"),
        }
        match tokens[2] {
            Token::Axis(c, v) => { assert_eq!(c, 'Y'); assert_relative_eq!(v, -0.25); },
            _ => panic!("Incorrect token"),
        }
        match tokens[3] {
            Token::Axis(c, v) => { assert_eq!(c, 'Z'); assert_relative_eq!(v, 0.0); },
            _ => panic!("Incorrect token"),
        }
    }

    #[test]
    fn test_tool_change() {
        let line = "T1 M6";
        let (_rem, tokens) = tokenize_line(line).unwrap();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::Tool(1)));
        assert!(matches!(tokens[1], Token::M(6)));
    }

    #[test]
    fn test_empty_line() {
        let line = "";
        let (_rem, tokens) = tokenize_line(line).unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_line_with_only_whitespace() {
        let line = "   \t ";
        let (rem, tokens) = tokenize_line(line).unwrap();
        assert!(tokens.is_empty());
        assert!(rem.trim().is_empty());
    }
}