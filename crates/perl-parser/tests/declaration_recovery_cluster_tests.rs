use perl_parser::Parser;
use perl_parser_core::engine::ast::NodeKind;
use perl_tdd_support::must;

#[test]
fn recovers_scoped_sub_forward_decl_without_semicolon() {
    let mut parser = Parser::new("my sub helper my $x = 1;");
    let ast = must(parser.parse());
    let sexp = ast.to_sexp();

    assert!(
        matches!(ast.kind, NodeKind::Program { .. }),
        "expected Program root, got {:?}",
        ast.kind
    );
    if let NodeKind::Program { statements } = &ast.kind {
        assert!(
            statements.len() >= 2,
            "expected recovered sub declaration and trailing variable declaration, got: {sexp}"
        );
    }

    assert!(sexp.contains("(sub "), "recovered AST should retain subroutine shape: {sexp}");
    assert!(
        sexp.contains("my_declaration"),
        "recovered AST should retain trailing declaration: {sexp}"
    );
}
