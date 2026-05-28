use std::time::Instant;
use typespec_rs::checker::Checker;
use typespec_rs::diagnostics::DiagnosticSeverity;
use typespec_rs::parser;
use typespec_rs::scanner::Lexer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("/tmp/cdp_clean.tsp");
    let source = std::fs::read_to_string(path).unwrap();
    println!(
        "Source: {} chars, {} lines",
        source.len(),
        source.lines().count()
    );

    // Phase 0: Scanner only
    let t0 = Instant::now();
    let mut lexer = Lexer::new(&source);
    let mut token_count = 0u64;
    loop {
        let tok = lexer.scan();
        token_count += 1;
        if tok == typespec_rs::scanner::TokenKind::EndOfFile {
            break;
        }
    }
    let scan_time = t0.elapsed();
    println!("Scanner only: {:.2?} ({} tokens)", scan_time, token_count);

    // Phase 1a: Parse without stdlib
    let t1 = Instant::now();
    let parse_result_no_stdlib = parser::parse(&source);
    let parse_only_time = t1.elapsed();
    println!("Parse (no stdlib): {:.2?}", parse_only_time);
    println!(
        "  AST nodes: {}",
        parse_result_no_stdlib.builder.nodes.len()
    );
    println!(
        "  Diagnostics: {}",
        parse_result_no_stdlib.diagnostics.len()
    );

    // Phase 1b: Parse with stdlib
    let t2 = Instant::now();
    let parse_result =
        parser::parse_with_libraries(&source, parser::ParseOptions::default().libraries);
    let parse_with_stdlib_time = t2.elapsed();
    println!("Parse (with stdlib): {:.2?}", parse_with_stdlib_time);
    println!("  AST nodes: {}", parse_result.builder.nodes.len());
    println!("  Diagnostics: {}", parse_result.diagnostics.len());

    // Phase 2: Check
    let t3 = Instant::now();
    let mut checker = Checker::new();
    checker.set_parse_result(parse_result.root_id, parse_result.builder);
    checker.check_program();
    let check_time = t3.elapsed();
    println!("Check: {:.2?}", check_time);

    let diags = checker.diagnostics();
    let errors = diags
        .iter()
        .filter(|d| matches!(d.severity, DiagnosticSeverity::Error))
        .count();
    let warnings = diags
        .iter()
        .filter(|d| matches!(d.severity, DiagnosticSeverity::Warning))
        .count();
    println!("  Errors: {}, Warnings: {}", errors, warnings);
    println!(
        "Total (parse+check): {:.2?}",
        parse_with_stdlib_time + check_time
    );
}
