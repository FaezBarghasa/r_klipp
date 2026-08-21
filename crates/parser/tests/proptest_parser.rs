//! Property-Based Testing & Fuzz Testing for Zero-Allocation G-Code Parser.

use parser::stream::GcodeLineParser;
use proptest::prelude::*;

proptest! {
    /// Invariant: Parser should NEVER panic on arbitrary byte inputs or corrupted G-Code strings.
    #[test]
    fn proptest_parser_never_panics_on_arbitrary_strings(s in "\\PC*") {
        let mut parser = GcodeLineParser::new();
        let _ = parser.parse_line(&s);
    }

    /// Invariant: Valid linear moves with any arbitrary float coordinates are reliably parsed.
    #[test]
    fn proptest_valid_g1_coordinates(
        x in -1000.0f32..1000.0f32,
        y in -1000.0f32..1000.0f32,
        z in 0.0f32..500.0f32,
        f in 1.0f32..20000.0f32
    ) {
        let mut parser = GcodeLineParser::new();
        let gcode = format!("G1 X{:.3} Y{:.3} Z{:.3} F{:.1}", x, y, z, f);
        let node = parser.parse_line(&gcode).expect("Should not error on valid format").expect("Should produce AST node");

        match node {
            parser::ast::AstNode::LinearMove { x: px, y: py, z: pz, feedrate: pf, .. } => {
                prop_assert!(px.is_some());
                prop_assert!(py.is_some());
                prop_assert!(pz.is_some());
                prop_assert!(pf.is_some());
            }
            _ => prop_assert!(false, "Expected LinearMove node"),
        }
    }
}
