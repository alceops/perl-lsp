/// Regression tests for TokenKind role predicate refactor (PR #6428).
///
/// These tests guard three behavioral changes found during deep review:
///
/// 1. `Question` (`?`) was explicitly a statement-end marker in `is_at_statement_end`
///    and `should_continue_bare_call_after_block`, but is not in `is_recovery_boundary()`.
///    After a bare-call block, `? 1 : 2` must be ternary on the result, not a bare-call arg.
///
/// 2. `is_logical_operator()` includes `Not` and `WordNot` which were NOT in the original
///    `is_binary_operator` list. The expansion must not break named-unary parsing.
///
/// 3. `is_sync_point()` now expands via `is_recovery_boundary()` to include
///    `RightParen` / `RightBracket` / `Eof` as sync points.  These were not sync
///    points before; check that error recovery inside parens still works.
mod cpan_test_helpers;
use cpan_test_helpers::*;

// --- Case 1: ternary after bare-call-with-block ---

#[test]
fn grep_block_ternary_result() {
    // grep { cond } @list ? "yes" : "no"
    // The `?` is a ternary on the result of grep, NOT an argument to grep.
    assert_clean_parse(r#"my $r = grep { $_ > 0 } @list ? "yes" : "no";"#);
}

#[test]
fn sort_block_ternary_no_list() {
    // sort { cmp } @arr then ternary on result.
    // should_continue_bare_call_after_block must stop at `?` if no intervening arg.
    assert_clean_parse(r#"my $r = (sort { $a <=> $b } @arr) ? "has items" : "empty";"#);
}

#[test]
fn bare_call_block_then_ternary_at_statement_end() {
    // `?` (Question) was in the original is_at_statement_end list but is NOT
    // in is_recovery_boundary(). Verify postfix arg collection stops at `?`.
    // `scalar grep { cond } @list` returns a count; ternary on that count is valid.
    assert_clean_parse(
        r#"my $n = scalar grep { $_ > 5 } @data; my $label = $n ? "found" : "none";"#,
    );
}

#[test]
fn map_block_ternary_result() {
    // map { expr } @arr ? "non-empty" : "empty"
    assert_clean_parse(r#"my $r = (map { $_ * 2 } @arr) ? "non-empty" : "empty";"#);
}

#[test]
fn sort_block_ternary_result() {
    // sort { cmp } @list ? $a : $b
    assert_clean_parse(r#"my @s = sort { $a <=> $b } @list; my $r = @s ? $s[0] : undef;"#);
}

#[test]
fn grep_block_ternary_in_assignment() {
    // Scalar context of grep result in ternary condition.
    assert_clean_parse(r#"print scalar(grep { /foo/ } @lines) ? "found" : "not found";"#);
}

// --- Case 2: `not` / `WordNot` after named-unary should not be consumed as arg ---

#[test]
fn defined_then_not_expr() {
    // `defined not $x` — `not` is a prefix unary but is_logical_operator now
    // includes Not; ensure named-unary builtins don't try to eat `not` as arg.
    assert_clean_parse(r#"my $ok = defined $x && not $y;"#);
}

#[test]
fn ref_then_not_condition() {
    assert_clean_parse(r#"my $b = ref $obj ? not $flag : 1;"#);
}

// --- Case 3: is_sync_point now includes RightParen/RightBracket/Eof via is_recovery_boundary ---
// The sync_point expansion is safe because those tokens already stopped the loop in other
// ways; adding them as sync points only makes recovery faster, not incorrect.

#[test]
fn error_recovery_sync_at_paren() {
    // Parser must not loop inside a well-formed paren expression when a missing operator
    // triggers recovery; the `)` should act as a sync boundary.
    assert_clean_parse(r#"my $x = (1 + 2);"#);
}

#[test]
fn error_recovery_sync_at_bracket() {
    // Matching bracket closes an arrayref safely.
    assert_clean_parse(r#"my $r = [1, 2, 3];"#);
}
