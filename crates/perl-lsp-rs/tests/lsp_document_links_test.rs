//! UX-focused behavioral coverage for document links.
//!
//! Exercises the real JSON-RPC workflow used by editors:
//! 1) open document
//! 2) request deferred links
//! 3) resolve a chosen link

mod support;

use serde_json::{Value, json};
use support::lsp_harness::LspHarness;

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Scenario {
    name: &'static str,
}

impl Scenario {
    fn new(name: &'static str) -> Self {
        eprintln!("Scenario: {name}");
        Self { name }
    }

    fn given(&self, step: &str) {
        eprintln!("[{}] Given {step}", self.name);
    }

    fn when(&self, step: &str) {
        eprintln!("[{}] When {step}", self.name);
    }

    fn then(&self, step: &str) {
        eprintln!("[{}] Then {step}", self.name);
    }
}

fn first_link_of_type<'a>(links: &'a [Value], kind: &str) -> Option<&'a Value> {
    links.iter().find(|link| link.pointer("/data/type").and_then(Value::as_str) == Some(kind))
}

#[test]
fn bdd_document_links_emit_deferred_data_and_resolve_module_targets() -> TestResult {
    let scenario = Scenario::new("Document link module flow uses deferred resolve");
    scenario.given("a Perl document contains both pragma and module imports");

    let mut harness = LspHarness::new();
    let _ = harness.initialize(None)?;

    let uri = "file:///workspace/lib/main.pl";
    harness.open(uri, "use strict;\nuse Foo::Bar;\nrequire Data::Dumper;\n")?;

    scenario.when("the client requests textDocument/documentLink");
    let links_value = harness.document_links(uri)?;
    let links = links_value.as_array().ok_or("documentLink should return an array")?;

    scenario.then("module imports produce deferred links while pragmas are excluded");
    assert!(links.len() >= 2, "expected links for Foo::Bar and Data::Dumper, got {links_value:?}");

    let strict_link = links
        .iter()
        .find(|link| link.pointer("/data/module").and_then(Value::as_str) == Some("strict"));
    assert!(strict_link.is_none(), "pragma 'strict' must not produce a document link");

    let module_link =
        first_link_of_type(links, "module").ok_or("expected at least one module link")?;
    assert!(
        module_link.get("target").is_none(),
        "deferred module links should not set target before resolve: {module_link:?}"
    );

    scenario.when("the client resolves the deferred module link");
    let resolved = harness.resolve_document_link(module_link.clone())?;

    scenario.then("documentLink/resolve sets a concrete target URI");
    let target = resolved
        .get("target")
        .and_then(Value::as_str)
        .ok_or("resolved link should include target")?;
    assert!(
        target.starts_with("file://") || target.starts_with("https://metacpan.org/pod/"),
        "resolved module link should point to local file or MetaCPAN, got {target}"
    );

    Ok(())
}

#[test]
fn bdd_document_link_resolve_normalizes_mixed_separators_for_file_paths() -> TestResult {
    let scenario = Scenario::new("File link resolve normalizes mixed path separators");
    scenario.given("a deferred file link carries mixed slash and backslash separators");

    let mut harness = LspHarness::new();
    let _ = harness.initialize(None)?;

    let unresolved = json!({
        "range": {
            "start": {"line": 0, "character": 8},
            "end": {"line": 0, "character": 28}
        },
        "tooltip": "Open lib/Foo/Bar.pm",
        "data": {
            "type": "file",
            "path": "lib\\Foo//Bar.pm",
            "baseUri": "file:///workspace/script/main.pl"
        }
    });

    scenario.when("the client requests documentLink/resolve for the deferred file link");
    let resolved = harness.resolve_document_link(unresolved)?;

    scenario.then("the resolved target is a normalized file URI");
    let target = resolved
        .get("target")
        .and_then(Value::as_str)
        .ok_or("resolved file link should include a target")?;
    assert!(target.starts_with("file:///workspace/script/lib/Foo/Bar.pm"));

    Ok(())
}
