use crate::parser::tests::{check_invalid_script, check_module_parser, check_script_parser};
use crate::{Parser, Source};
use boa_ast::{
    Declaration, ModuleItem, Span, Statement,
    declaration::{
        ExportDeclaration, ExportSpecifier, ImportAttribute, ImportDeclaration, ImportKind,
        LexicalDeclaration, ModuleSpecifier, ReExportKind, VarDeclaration, Variable,
    },
    expression::{
        Identifier,
        literal::{Literal, LiteralKind},
    },
};
use boa_interner::{Interner, Sym};
use boa_macros::utf16;
use indoc::indoc;

/// Checks `var` declaration parsing.
#[test]
fn var_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a = 5;",
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(5, Span::new((1, 9), (1, 10))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `var` declaration parsing with reserved words.
#[test]
fn var_declaration_keywords() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var yield = 5;",
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::YIELD, Span::new((1, 5), (1, 10))),
                    Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );

    let interner = &mut Interner::default();
    check_script_parser(
        "var await = 5;",
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::AWAIT, Span::new((1, 5), (1, 10))),
                    Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `var` declaration parsing with no spaces.
#[test]
fn var_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a=5;",
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(5, Span::new((1, 7), (1, 8))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks empty `var` declaration parsing.
#[test]
fn empty_var_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a;",
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    None,
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks multiple `var` declarations.
#[test]
fn multiple_var_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "var a = 5, b, c = 6;",
        vec![
            Statement::Var(VarDeclaration(
                vec![
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("a", utf16!("a")),
                            Span::new((1, 5), (1, 6)),
                        ),
                        Some(Literal::new(5, Span::new((1, 9), (1, 10))).into()),
                    ),
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("b", utf16!("b")),
                            Span::new((1, 12), (1, 13)),
                        ),
                        None,
                    ),
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("c", utf16!("c")),
                            Span::new((1, 15), (1, 16)),
                        ),
                        Some(Literal::new(6, Span::new((1, 19), (1, 20))).into()),
                    ),
                ]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `let` declaration parsing.
#[test]
fn let_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(5, Span::new((1, 9), (1, 10))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `let` declaration parsing with reserved words.
#[test]
fn let_declaration_keywords() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let yield = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::YIELD, Span::new((1, 5), (1, 10))),
                    Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );

    let interner = &mut Interner::default();
    check_script_parser(
        "let await = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::AWAIT, Span::new((1, 5), (1, 10))),
                    Some(Literal::new(5, Span::new((1, 13), (1, 14))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `let` declaration parsing with no spaces.
#[test]
fn let_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a=5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    Some(Literal::new(5, Span::new((1, 7), (1, 8))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `let` declaration with a reserved keyword as identifier.
#[test]
fn let_declaration_reserved_keyword_identifier() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let of = 1;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("of", utf16!("of")),
                        Span::new((1, 5), (1, 7)),
                    ),
                    Some(Literal::new(1, Span::new((1, 10), (1, 11))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks empty `let` declaration parsing.
#[test]
fn empty_let_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 5), (1, 6)),
                    ),
                    None,
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks multiple `let` declarations.
#[test]
fn multiple_let_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "let a = 5, b, c = 6;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Let(
                vec![
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("a", utf16!("a")),
                            Span::new((1, 5), (1, 6)),
                        ),
                        Some(Literal::new(5, Span::new((1, 9), (1, 10))).into()),
                    ),
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("b", utf16!("b")),
                            Span::new((1, 12), (1, 13)),
                        ),
                        None,
                    ),
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("c", utf16!("c")),
                            Span::new((1, 15), (1, 16)),
                        ),
                        Some(Literal::new(6, Span::new((1, 19), (1, 20))).into()),
                    ),
                ]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `const` declaration parsing.
#[test]
fn const_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "const a = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 7), (1, 8)),
                    ),
                    Some(Literal::new(5, Span::new((1, 11), (1, 12))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `const` declaration parsing with reserved words.
#[test]
fn const_declaration_keywords() {
    let interner = &mut Interner::default();
    check_script_parser(
        "const yield = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::YIELD, Span::new((1, 7), (1, 12))),
                    Some(Literal::new(5, Span::new((1, 15), (1, 16))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );

    let interner = &mut Interner::default();
    check_script_parser(
        "const await = 5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    Identifier::new(Sym::AWAIT, Span::new((1, 7), (1, 12))),
                    Some(Literal::new(5, Span::new((1, 15), (1, 16))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `const` declaration parsing with no spaces.
#[test]
fn const_declaration_no_spaces() {
    let interner = &mut Interner::default();
    check_script_parser(
        "const a=5;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("a", utf16!("a")),
                        Span::new((1, 7), (1, 8)),
                    ),
                    Some(Literal::new(5, Span::new((1, 9), (1, 10))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks empty `const` declaration parsing.
#[test]
fn empty_const_declaration() {
    check_invalid_script("const a;");
}

/// Checks multiple `const` declarations.
#[test]
fn multiple_const_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "const a = 5, c = 6;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Const(
                vec![
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("a", utf16!("a")),
                            Span::new((1, 7), (1, 8)),
                        ),
                        Some(Literal::new(5, Span::new((1, 11), (1, 12))).into()),
                    ),
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("c", utf16!("c")),
                            Span::new((1, 14), (1, 15)),
                        ),
                        Some(Literal::new(6, Span::new((1, 18), (1, 19))).into()),
                    ),
                ]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `LexicalDeclaration` early errors.
#[test]
fn lexical_declaration_early_errors() {
    check_invalid_script("let let = 0");
    check_invalid_script("let a = 0, a = 0");
    check_invalid_script("const a = 0, a = 0");
    check_invalid_script("for (let let = 0; ; ) {}");
    check_invalid_script("for (let a = 0, a = 0; ; ) {}");
    check_invalid_script("for (const a = 0, a = 0; ; ) {}");
}

/// Checks module exports with reserved keywords
#[test]
fn module_export_reserved() {
    let interner = &mut Interner::default();
    let val = interner.get_or_intern_static("val", utf16!("val"));
    check_module_parser(
        indoc! {"
            const val = null;
            export { val as null, val as true, val as false };
        "},
        vec![
            ModuleItem::StatementListItem(
                Declaration::Lexical(LexicalDeclaration::Const(
                    vec![Variable::from_identifier(
                        Identifier::new(val, Span::new((1, 7), (1, 10))),
                        Some(Literal::new(LiteralKind::Null, Span::new((1, 13), (1, 17))).into()),
                    )]
                    .try_into()
                    .unwrap(),
                ))
                .into(),
            ),
            ModuleItem::ExportDeclaration(
                ExportDeclaration::List(
                    vec![
                        ExportSpecifier::new(Sym::NULL, val, false),
                        ExportSpecifier::new(Sym::TRUE, val, false),
                        ExportSpecifier::new(Sym::FALSE, val, false),
                    ]
                    .into(),
                )
                .into(),
            ),
        ],
        interner,
    );
}

/// Checks import declaration with a single attribute.
#[test]
fn import_with_single_attribute() {
    let interner = &mut Interner::default();
    let json = interner.get_or_intern_static("json", utf16!("json"));
    let foo_json = interner.get_or_intern_static("./foo.json", utf16!("./foo.json"));
    let type_sym = interner.get_or_intern_static("type", utf16!("type"));

    check_module_parser(
        r#"import json from "./foo.json" with { type: "json" };"#,
        vec![ModuleItem::ImportDeclaration(ImportDeclaration::new(
            Some(Identifier::new(json, Span::new((1, 8), (1, 12)))),
            ImportKind::DefaultOrUnnamed,
            ModuleSpecifier::new(foo_json),
            vec![ImportAttribute::new(type_sym, json)].into(),
        ))],
        interner,
    );
}

/// Checks import declaration with multiple attributes.
#[test]
fn import_with_multiple_attributes() {
    let interner = &mut Interner::default();
    let json = interner.get_or_intern_static("json", utf16!("json"));
    let foo_json = interner.get_or_intern_static("./foo.json", utf16!("./foo.json"));
    let type_sym = interner.get_or_intern_static("type", utf16!("type"));
    let integrity = interner.get_or_intern_static("integrity", utf16!("integrity"));
    let hash = interner.get_or_intern_static("sha384-abc123", utf16!("sha384-abc123"));

    check_module_parser(
        r#"import json from "./foo.json" with { type: "json", integrity: "sha384-abc123" };"#,
        vec![ModuleItem::ImportDeclaration(ImportDeclaration::new(
            Some(Identifier::new(json, Span::new((1, 8), (1, 12)))),
            ImportKind::DefaultOrUnnamed,
            ModuleSpecifier::new(foo_json),
            vec![
                ImportAttribute::new(type_sym, json),
                ImportAttribute::new(integrity, hash),
            ]
            .into(),
        ))],
        interner,
    );
}

/// Checks import declaration with trailing comma in attributes.
#[test]
fn import_with_trailing_comma_attribute() {
    let interner = &mut Interner::default();
    let json = interner.get_or_intern_static("json", utf16!("json"));
    let foo_json = interner.get_or_intern_static("./foo.json", utf16!("./foo.json"));
    let type_sym = interner.get_or_intern_static("type", utf16!("type"));

    check_module_parser(
        r#"import json from "./foo.json" with { type: "json", };"#,
        vec![ModuleItem::ImportDeclaration(ImportDeclaration::new(
            Some(Identifier::new(json, Span::new((1, 8), (1, 12)))),
            ImportKind::DefaultOrUnnamed,
            ModuleSpecifier::new(foo_json),
            vec![ImportAttribute::new(type_sym, json)].into(),
        ))],
        interner,
    );
}

/// Checks re-export with attributes.
#[test]
fn reexport_with_attributes() {
    let interner = &mut Interner::default();
    let foo_js = interner.get_or_intern_static("./foo.js", utf16!("./foo.js"));
    let type_sym = interner.get_or_intern_static("type", utf16!("type"));
    let json = interner.get_or_intern_static("json", utf16!("json"));

    check_module_parser(
        r#"export * from "./foo.js" with { type: "json" };"#,
        vec![ModuleItem::ExportDeclaration(Box::new(
            ExportDeclaration::ReExport {
                kind: ReExportKind::Namespaced { name: None },
                specifier: ModuleSpecifier::new(foo_js),
                attributes: vec![ImportAttribute::new(type_sym, json)].into(),
            },
        ))],
        interner,
    );
}

/// Checks import attributes with string literal key.
#[test]
fn import_with_string_literal_key() {
    let interner = &mut Interner::default();
    let json = interner.get_or_intern_static("json", utf16!("json"));
    let foo_json = interner.get_or_intern_static("./foo.json", utf16!("./foo.json"));
    let type_sym = interner.get_or_intern_static("type", utf16!("type"));

    check_module_parser(
        r#"import json from "./foo.json" with { "type": "json" };"#,
        vec![ModuleItem::ImportDeclaration(ImportDeclaration::new(
            Some(Identifier::new(json, Span::new((1, 8), (1, 12)))),
            ImportKind::DefaultOrUnnamed,
            ModuleSpecifier::new(foo_json),
            vec![ImportAttribute::new(type_sym, json)].into(),
        ))],
        interner,
    );
}

/// Checks that duplicate attribute keys are rejected.
#[test]
fn import_duplicate_attribute_key() {
    assert!(
        Parser::new(Source::from_bytes(
            r#"import json from "./foo.json" with { type: "json", type: "css" };"#
        ))
        .parse_module(
            &boa_ast::scope::Scope::new_global(),
            &mut Interner::default()
        )
        .is_err()
    );
}

/// Checks that non-string attribute values are rejected.
#[test]
fn import_non_string_attribute_value() {
    let scope = boa_ast::scope::Scope::new_global();

    assert!(
        Parser::new(Source::from_bytes(
            r#"import json from "./foo.json" with { type: json };"#
        ))
        .parse_module(&scope, &mut Interner::default())
        .is_err()
    );
    assert!(
        Parser::new(Source::from_bytes(
            r#"import json from "./foo.json" with { type: 123 };"#
        ))
        .parse_module(&scope, &mut Interner::default())
        .is_err()
    );
    assert!(
        Parser::new(Source::from_bytes(
            r#"import json from "./foo.json" with { type: true };"#
        ))
        .parse_module(&scope, &mut Interner::default())
        .is_err()
    );
}

/// Checks `using` declaration parsing.
#[test]
fn using_declaration() {
    let interner = &mut Interner::default();
    check_script_parser(
        "using x = resource;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Using(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("x", utf16!("x")),
                        Span::new((1, 7), (1, 8)),
                    ),
                    Some(
                        Identifier::new(
                            interner.get_or_intern_static("resource", utf16!("resource")),
                            Span::new((1, 11), (1, 19)),
                        )
                        .into(),
                    ),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks `using` declaration with multiple bindings.
#[test]
fn using_declaration_multiple() {
    let interner = &mut Interner::default();
    check_script_parser(
        "using a = res1, b = res2;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Using(
                vec![
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("a", utf16!("a")),
                            Span::new((1, 7), (1, 8)),
                        ),
                        Some(
                            Identifier::new(
                                interner.get_or_intern_static("res1", utf16!("res1")),
                                Span::new((1, 11), (1, 15)),
                            )
                            .into(),
                        ),
                    ),
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("b", utf16!("b")),
                            Span::new((1, 17), (1, 18)),
                        ),
                        Some(
                            Identifier::new(
                                interner.get_or_intern_static("res2", utf16!("res2")),
                                Span::new((1, 21), (1, 25)),
                            )
                            .into(),
                        ),
                    ),
                ]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks that `using` declaration without initializer fails.
#[test]
fn using_declaration_no_init() {
    check_invalid_script("using x;");
}

/// Checks `await using` declaration parsing in async function.
#[test]
fn await_using_declaration() {
    let interner = &mut Interner::default();
    // await using is only valid in async contexts, so we test it inside an async function
    // We just verify it parses without error
    let source = Source::from_bytes("async function f() { await using x = resource; }");
    let mut parser = Parser::new(source);
    let scope = boa_ast::scope::Scope::new_global();
    let result = parser.parse_script(&scope, interner);
    assert!(
        result.is_ok(),
        "Failed to parse await using in async function: {:?}",
        result.err()
    );
}

/// Checks that `await using` declaration without initializer fails.
#[test]
fn await_using_declaration_no_init() {
    // Test in async function context
    check_invalid_script("async function f() { await using x; }");
}

/// Checks that `await using` is only valid in async contexts.
#[test]
fn await_using_requires_async_context() {
    // Should fail in non-async context (top-level script)
    check_invalid_script("await using x = resource;");
}

/// Checks that line terminator between `await` and `using` is rejected.
/// Per spec: <https://arai-a.github.io/ecma262-compare/snapshot.html?pr=3000#prod-AwaitUsingDeclarationHead>
/// There must be [no `LineTerminator` here] between the keywords.
#[test]
fn await_using_no_line_terminator() {
    // Line terminator between await and using should fail
    check_invalid_script("async function f() { await\nusing x = resource; }");

    // Without line terminator should succeed
    let interner = &mut Interner::default();
    let source = Source::from_bytes("async function f() { await using x = resource; }");
    let mut parser = Parser::new(source);
    let scope = boa_ast::scope::Scope::new_global();
    let result = parser.parse_script(&scope, interner);
    assert!(
        result.is_ok(),
        "Failed to parse await using without line terminator: {:?}",
        result.err()
    );
}

/// Checks that `using` declarations cannot use object destructuring patterns.
/// Per spec: <https://tc39.es/proposal-explicit-resource-management/>
/// The grammar uses ~Pattern parameter which means destructuring is NOT allowed.
#[test]
fn using_no_destructuring_object() {
    check_invalid_script("using {x, y} = resource;");
}

/// Checks that `using` declarations cannot use array destructuring patterns.
#[test]
fn using_no_destructuring_array() {
    check_invalid_script("export using [a, b] = resource;");
}

/// Checks that destructuring patterns are rejected for `await using` declarations.
#[test]
fn await_using_no_destructuring_object() {
    check_invalid_script("async function f() { await using {x, y} = resource; }");
}

/// Checks that destructuring patterns are rejected for `await using` declarations.
#[test]
fn await_using_no_destructuring_array() {
    check_invalid_script("async function f() { await using [a, b] = resource; }");
}

/// Checks that `using let` is rejected (let is not allowed as a bound name).
#[test]
fn using_let_rejected() {
    check_invalid_script("using let = resource;");
}

/// Checks that duplicate names in `using` declarations are rejected.
#[test]
fn using_duplicate_names() {
    check_invalid_script("using x = r1, x = r2;");
}

/// Checks that `await using` with duplicate names is rejected.
#[test]
fn await_using_duplicate_names() {
    check_invalid_script("async function f() { await using x = r1, x = r2; }");
}

/// Checks that `using` works with valid identifiers.
#[test]
fn using_valid_identifiers() {
    let interner = &mut Interner::default();
    check_script_parser(
        "using x = resource, y = resource2;",
        vec![
            Declaration::Lexical(LexicalDeclaration::Using(
                vec![
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("x", utf16!("x")),
                            Span::new((1, 7), (1, 8)),
                        ),
                        Some(
                            Identifier::new(
                                interner.get_or_intern_static("resource", utf16!("resource")),
                                Span::new((1, 11), (1, 19)),
                            )
                            .into(),
                        ),
                    ),
                    Variable::from_identifier(
                        Identifier::new(
                            interner.get_or_intern_static("y", utf16!("y")),
                            Span::new((1, 21), (1, 22)),
                        ),
                        Some(
                            Identifier::new(
                                interner.get_or_intern_static("resource2", utf16!("resource2")),
                                Span::new((1, 25), (1, 34)),
                            )
                            .into(),
                        ),
                    ),
                ]
                .try_into()
                .unwrap(),
            ))
            .into(),
        ],
        interner,
    );
}

/// Checks that `await using` works with valid identifiers in async context.
#[test]
fn await_using_valid_identifiers() {
    let interner = &mut Interner::default();
    let source = Source::from_bytes("async function f() { await using x = r1, y = r2; }");
    let mut parser = Parser::new(source);
    let scope = boa_ast::scope::Scope::new_global();
    let result = parser.parse_script(&scope, interner);
    assert!(
        result.is_ok(),
        "Failed to parse await using with multiple bindings: {:?}",
        result.err()
    );
}

/// Checks that `using` is still parsed as an identifier when it cannot start a
/// `using` declaration (`using` is a contextual keyword).
/// See: <https://github.com/boa-dev/boa/issues/5602>
#[test]
fn using_as_identifier() {
    // `using` declarations need a binding identifier on the same line; everything
    // else starting with `using` is an expression statement.
    let interner = &mut Interner::default();
    check_script_parser(
        "var using = 0; using;",
        vec![
            Statement::Var(VarDeclaration(
                vec![Variable::from_identifier(
                    Identifier::new(
                        interner.get_or_intern_static("using", utf16!("using")),
                        Span::new((1, 5), (1, 10)),
                    ),
                    Some(Literal::new(0, Span::new((1, 13), (1, 14))).into()),
                )]
                .try_into()
                .unwrap(),
            ))
            .into(),
            Statement::Expression(
                Identifier::new(
                    interner.get_or_intern_static("using", utf16!("using")),
                    Span::new((1, 16), (1, 21)),
                )
                .into(),
            )
            .into(),
        ],
        interner,
    );
}

/// Checks that `using` is a valid identifier reference in expressions.
#[test]
fn using_as_identifier_expressions() {
    // All of these must parse: `using` is only a keyword where a `using`
    // declaration can start.
    for js in [
        "var using = {}; !using['b'];",
        "var using = {}; delete using['a'];",
        "var using = {}; using.a = 1;",
        "var using = {}; using;",
        "var using = 0; using = 1;",
        "var using = 0; using + 1;",
        "var using = 0; typeof using;",
        "using: 1;",
        "var using = {}; for (using.a of []) {}",
        "var using = {}; for (using['a'] of []) {}",
        "var using = {}; for (using in {}) {}",
        "var using = {}; ({using: 1});",
        "var using = {}; ({using});",
    ] {
        let result = Parser::new(Source::from_bytes(js)).parse_script(
            &boa_ast::scope::Scope::new_global(),
            &mut Interner::default(),
        );
        assert!(result.is_ok(), "failed to parse {js:?}: {:?}", result.err());
    }
}

/// Checks that `using` declarations still work when a binding identifier follows.
#[test]
fn using_declaration_still_parses() {
    for js in [
        "using x = resource;",
        "using x = r1, y = r2;",
        "for (using x of []) {}",
        "for (using x = r;;) { break; }",
    ] {
        let result = Parser::new(Source::from_bytes(js)).parse_script(
            &boa_ast::scope::Scope::new_global(),
            &mut Interner::default(),
        );
        assert!(result.is_ok(), "failed to parse {js:?}: {:?}", result.err());
    }
}

/// Checks the `[no LineTerminator here]` restriction of `using` declarations.
#[test]
fn using_declaration_no_line_terminator() {
    // `using` followed by a line terminator is an identifier reference, so this
    // is two statements: `using;` and `x = 1;`.
    let interner = &mut Interner::default();
    let result = Parser::new(Source::from_bytes("using\nx = 1;"))
        .parse_script(&boa_ast::scope::Scope::new_global(), interner);
    assert!(result.is_ok(), "failed to parse: {:?}", result.err());
}

/// Checks that `using` declarations are not allowed in the head of `for-in` loops.
#[test]
fn using_declaration_invalid_in_for_in() {
    check_invalid_script("for (using x in {}) {}");
}

/// Checks that `using of` is not a `using` declaration in a for-of head.
#[test]
fn using_of_is_not_a_declaration_in_for_of() {
    // `for (using of it)` iterates over `it`, assigning to the variable `using`
    // (`[lookahead ≠ using of]` in the `ForInOfStatement` grammar).
    let interner = &mut Interner::default();
    let result = Parser::new(Source::from_bytes("for (using of it) {}"))
        .parse_script(&boa_ast::scope::Scope::new_global(), interner);
    assert!(result.is_ok(), "failed to parse: {:?}", result.err());
}

/// Checks that contextual keywords can be bound by a `using` declaration.
#[test]
fn using_declaration_contextual_keyword_bindings() {
    // `of`, `async` and `using` itself are valid binding identifiers.
    for js in [
        "using of = r;",
        "using using = r;",
        "using async = r;",
        "using yield = r;",
    ] {
        let result = Parser::new(Source::from_bytes(js)).parse_script(
            &boa_ast::scope::Scope::new_global(),
            &mut Interner::default(),
        );
        assert!(result.is_ok(), "failed to parse {js:?}: {:?}", result.err());
    }
}

/// Checks that `using` can be used as a binding identifier in other declarations.
#[test]
fn using_as_binding_identifier() {
    for js in [
        "var using = 1;",
        "let using = 1;",
        "const using = 1;",
        "function using() {}",
        "function f(using) {}",
        "class using {}",
        "try {} catch (using) {}",
        "for (var using of []) {}",
        "var [using] = [];",
        "var { using } = {};",
        // Single-parameter arrow functions.
        "using => 1;",
        "async using => 1;",
    ] {
        let result = Parser::new(Source::from_bytes(js)).parse_script(
            &boa_ast::scope::Scope::new_global(),
            &mut Interner::default(),
        );
        assert!(result.is_ok(), "failed to parse {js:?}: {:?}", result.err());
    }
}
