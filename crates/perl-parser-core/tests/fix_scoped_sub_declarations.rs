//! Regression tests for scoped subroutine declarations.
//!
//! Perl allows `my sub`, `our sub`, and `state sub` declarations.
//! These should parse as subroutine statements rather than variable declarations.

mod cpan_test_helpers;

use cpan_test_helpers::assert_clean_parse;
use perl_parser_core::{NodeKind, Parser};

#[test]
fn parses_my_sub_declaration() {
    assert_clean_parse("my sub helper ($x) { $x }");
}

#[test]
fn parses_our_sub_declaration() {
    assert_clean_parse("our sub helper ($x) { $x }");
}

#[test]
fn parses_state_sub_declaration() {
    assert_clean_parse("state sub memo { state $x = 1; $x }");
}

#[test]
fn parses_scoped_sub_forward_declaration() -> Result<(), Box<dyn std::error::Error>> {
    let source = "my sub helper; my $x = 1;";
    let mut parser = Parser::new(source);
    let ast = parser.parse()?;

    match &ast.kind {
        NodeKind::Program { statements } => {
            assert_eq!(statements.len(), 2, "expected two top-level statements");
            assert!(
                matches!(statements[0].kind, NodeKind::Subroutine { .. }),
                "first statement should stay a subroutine declaration"
            );
            assert!(
                matches!(statements[1].kind, NodeKind::VariableDeclaration { .. }),
                "second statement should stay the following declaration"
            );
        }
        other => return Err(format!("expected Program AST, got {}", other.kind_name()).into()),
    }

    assert!(
        parser.get_errors().is_empty(),
        "expected no diagnostics for valid scoped forward declaration, got: {:?}",
        parser.get_errors()
    );

    Ok(())
}

#[test]
fn recovers_scoped_sub_missing_name_without_losing_following_declaration()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "my sub ; my $x = 1;";
    let mut parser = Parser::new(source);
    let ast = parser.parse()?;

    match &ast.kind {
        NodeKind::Program { statements } => {
            assert_eq!(statements.len(), 2, "expected two top-level statements");
            assert!(
                matches!(statements[0].kind, NodeKind::Subroutine { .. }),
                "recovery should preserve malformed scoped sub as a subroutine node"
            );
            assert!(
                matches!(statements[1].kind, NodeKind::VariableDeclaration { .. }),
                "recovery should keep the following declaration intact"
            );
        }
        other => return Err(format!("expected Program AST, got {}", other.kind_name()).into()),
    }

    let diagnostics = format!("{:?}", parser.get_errors());
    assert!(
        diagnostics.contains("subroutine name"),
        "expected missing-name diagnostic, got: {diagnostics}"
    );

    Ok(())
}
