//! Zero-allocation G-Code lexer for no_std environments.

use heapless::Vec;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    G(u16),
    M(u16),
    T(u8),
    Axis(char, f32),
    Feedrate(f32),
    SpindleSpeed(f32),
    P(f32),
    Comment(&'a str),
    LineNumber(u32),
    Checksum(u8),
}

/// Zero-allocation ASCII float parsing for no_std
pub fn parse_float(bytes: &[u8]) -> Option<(f32, usize)> {
    if bytes.is_empty() {
        return None;
    }

    let mut i = 0;
    let mut negative = false;
    if bytes[i] == b'-' {
        negative = true;
        i += 1;
    } else if bytes[i] == b'+' {
        i += 1;
    }

    if i >= bytes.len() {
        return None;
    }

    let mut int_val: u32 = 0;
    let mut has_digits = false;

    while i < bytes.len() && bytes[i].is_ascii_digit() {
        has_digits = true;
        int_val = int_val.saturating_mul(10).saturating_add((bytes[i] - b'0') as u32);
        i += 1;
    }

    let mut frac_val: f32 = 0.0;
    let mut frac_div: f32 = 1.0;

    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            has_digits = true;
            frac_div *= 10.0;
            frac_val += (bytes[i] - b'0') as f32 / frac_div;
            i += 1;
        }
    }

    if !has_digits {
        return None;
    }

    let mut val = (int_val as f32) + frac_val;
    if negative {
        val = -val;
    }

    Some((val, i))
}

pub fn parse_uint(bytes: &[u8]) -> Option<(u32, usize)> {
    if bytes.is_empty() || !bytes[0].is_ascii_digit() {
        return None;
    }

    let mut val: u32 = 0;
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        val = val.saturating_mul(10).saturating_add((bytes[i] - b'0') as u32);
        i += 1;
    }

    Some((val, i))
}

/// Tokenizes a single line of G-Code without heap allocation.
pub fn tokenize_line<'a, const N: usize>(line: &'a str) -> Result<Vec<Token<'a>, N>, ()> {
    let mut tokens = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Skip whitespace
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\r' || bytes[i] == b'\n') {
            i += 1;
        }

        if i >= bytes.len() {
            break;
        }

        let b = bytes[i];

        // Semicolon comment: everything to end of line
        if b == b';' {
            let comment_text = line[i + 1..].trim();
            let _ = tokens.push(Token::Comment(comment_text));
            break;
        }

        // Parentheses comment: ( ... )
        if b == b'(' {
            if let Some(close_idx) = line[i..].find(')') {
                let comment_text = line[i + 1..i + close_idx].trim();
                let _ = tokens.push(Token::Comment(comment_text));
                i += close_idx + 1;
                continue;
            } else {
                let comment_text = line[i + 1..].trim();
                let _ = tokens.push(Token::Comment(comment_text));
                break;
            }
        }

        // Checksum: *123
        if b == b'*' {
            i += 1;
            if let Some((cs, consumed)) = parse_uint(&bytes[i..]) {
                let _ = tokens.push(Token::Checksum(cs as u8));
                i += consumed;
                continue;
            }
        }

        let letter = (b as char).to_ascii_uppercase();
        i += 1;

        match letter {
            'G' => {
                if let Some((code, consumed)) = parse_uint(&bytes[i..]) {
                    tokens.push(Token::G(code as u16)).map_err(|_| ())?;
                    i += consumed;
                }
            }
            'M' => {
                if let Some((code, consumed)) = parse_uint(&bytes[i..]) {
                    tokens.push(Token::M(code as u16)).map_err(|_| ())?;
                    i += consumed;
                }
            }
            'T' => {
                if let Some((tool, consumed)) = parse_uint(&bytes[i..]) {
                    tokens.push(Token::T(tool as u8)).map_err(|_| ())?;
                    i += consumed;
                }
            }
            'N' => {
                if let Some((num, consumed)) = parse_uint(&bytes[i..]) {
                    tokens.push(Token::LineNumber(num)).map_err(|_| ())?;
                    i += consumed;
                }
            }
            'F' => {
                if let Some((f, consumed)) = parse_float(&bytes[i..]) {
                    tokens.push(Token::Feedrate(f)).map_err(|_| ())?;
                    i += consumed;
                }
            }
            'S' => {
                if let Some((s, consumed)) = parse_float(&bytes[i..]) {
                    tokens.push(Token::SpindleSpeed(s)).map_err(|_| ())?;
                    i += consumed;
                }
            }
            'P' => {
                if let Some((p, consumed)) = parse_float(&bytes[i..]) {
                    tokens.push(Token::P(p)).map_err(|_| ())?;
                    i += consumed;
                }
            }
            'X' | 'Y' | 'Z' | 'E' | 'A' | 'B' | 'C' | 'I' | 'J' | 'K' | 'R' => {
                if let Some((val, consumed)) = parse_float(&bytes[i..]) {
                    tokens.push(Token::Axis(letter, val)).map_err(|_| ())?;
                    i += consumed;
                }
            }
            _ => {
                // Ignore unrecognized character
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_alloc_lexer() {
        let line = "N12 G1 X10.5 Y-20.25 Z0.5 E1.2 F3000 ; move command";
        let tokens: Vec<Token, 32> = tokenize_line(line).expect("tokenization failed");

        assert_eq!(tokens[0], Token::LineNumber(12));
        assert_eq!(tokens[1], Token::G(1));
        assert_eq!(tokens[2], Token::Axis('X', 10.5));
        assert_eq!(tokens[3], Token::Axis('Y', -20.25));
        assert_eq!(tokens[4], Token::Axis('Z', 0.5));
        assert_eq!(tokens[5], Token::Axis('E', 1.2));
        assert_eq!(tokens[6], Token::Feedrate(3000.0));
        assert_eq!(tokens[7], Token::Comment("move command"));
    }
}
