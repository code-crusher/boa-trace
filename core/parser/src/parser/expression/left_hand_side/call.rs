//! Call expression parsing.
//!
//! More information:
//!  - [MDN documentation][mdn]
//!  - [ECMAScript specification][spec]
//!
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Functions
//! [spec]: https://tc39.es/ecma262/#prod-CallExpression

use super::arguments::Arguments;
use crate::{
    Error,
    lexer::{Token, TokenKind},
    parser::{
        AllowAwait, AllowYield, Cursor, OrAbrupt, ParseResult, TokenParser,
        expression::{
            Expression, FormalParameterListOrExpression, expression_to_formal_parameters,
            left_hand_side::template::TaggedTemplateLiteral,
        },
    },
    source::ReadChar,
};
use ast::function::PrivateName;
use boa_ast::{
    self as ast, Punctuator, Span, Spanned,
    declaration::Variable,
    expression::{
        Call, Identifier,
        access::{PrivatePropertyAccess, SimplePropertyAccess},
    },
    function::{FormalParameter, FormalParameterList},
    operations::bound_names,
};
use boa_interner::{Interner, Sym};

/// Parses a call expression.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-CallExpression
#[derive(Debug)]
pub(super) struct CallExpression {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    first_member_expr: ast::Expression,
}

impl CallExpression {
    /// Creates a new `CallExpression` parser.
    pub(super) fn new<Y, A>(
        allow_yield: Y,
        allow_await: A,
        first_member_expr: ast::Expression,
    ) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            first_member_expr,
        }
    }
}

impl<R> TokenParser<R> for CallExpression
where
    R: ReadChar,
{
    type Output = FormalParameterListOrExpression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let token = cursor.peek(0, interner).or_abrupt()?;

        if token.kind() != &TokenKind::Punctuator(Punctuator::OpenParen) {
            let next_token = cursor.next(interner)?.expect("token vanished");
            return Err(Error::expected(
                ["(".to_owned()],
                next_token.to_string(interner),
                next_token.span(),
                "call expression",
            ));
        }

        // `AsyncArrowFunction` requires [no LineTerminator here] between `async` and `(`.
        let no_lt_before_args = !cursor.peek_is_line_terminator(0, interner)?.unwrap_or(true);
        let (args, args_span, has_trailing_comma) =
            Arguments::new(self.allow_yield, self.allow_await).parse(cursor, interner)?;

        // `async(...)` is ambiguous: it can be a call to a function named `async` or the
        // parameter list of an async arrow function. It is only the latter when the `async`
        // is an unescaped identifier followed by `=>` with no line terminator in between
        // (`async(a) => a`). An escaped `async` (e.g. `as\u{79}nc`) is never an arrow head,
        // and its span is necessarily wider than the 5 columns of the plain identifier.
        let is_async = match &self.first_member_expr {
            ast::Expression::Identifier(ident) => {
                ident.sym() == Sym::ASYNC
                    && ident
                        .span()
                        .end()
                        .column_number()
                        .saturating_sub(ident.span().start().column_number())
                        == 5
            }
            _ => false,
        };

        // `AsyncArrowFunction` also requires [no LineTerminator here] between `)` and `=>`.
        let no_lt_before_arrow = !cursor.peek_is_line_terminator(0, interner)?.unwrap_or(true);

        let is_async_arrow_head = is_async
            && no_lt_before_args
            && no_lt_before_arrow
            && cursor.peek(0, interner)?.map(Token::kind)
                == Some(&TokenKind::Punctuator(Punctuator::Arrow));

        if is_async_arrow_head {
            let fpl = arguments_to_formal_parameters(
                &args,
                has_trailing_comma,
                cursor.strict(),
                args_span,
            )?;
            return Ok(FormalParameterListOrExpression::AsyncArrowHead {
                fpl,
                params_start_position: args_span.start(),
            });
        }

        let lhs = Call::new(self.first_member_expr, args, args_span).into();
        let expr = CallExpressionTail::new(self.allow_yield, self.allow_await, lhs)
            .parse(cursor, interner)?;
        Ok(FormalParameterListOrExpression::Expression(expr))
    }
}

/// Parses the tail parts of a call expression (property access, successive call, array access).
#[derive(Debug)]
pub(super) struct CallExpressionTail {
    allow_yield: AllowYield,
    allow_await: AllowAwait,
    call: ast::Expression,
}

impl CallExpressionTail {
    /// Creates a new `CallExpressionTail` parser.
    pub(super) fn new<Y, A>(allow_yield: Y, allow_await: A, call: ast::Expression) -> Self
    where
        Y: Into<AllowYield>,
        A: Into<AllowAwait>,
    {
        Self {
            allow_yield: allow_yield.into(),
            allow_await: allow_await.into(),
            call,
        }
    }
}

impl<R> TokenParser<R> for CallExpressionTail
where
    R: ReadChar,
{
    type Output = ast::Expression;

    fn parse(self, cursor: &mut Cursor<R>, interner: &mut Interner) -> ParseResult<Self::Output> {
        let mut lhs = self.call;

        while let Some(token) = cursor.peek(0, interner)?.cloned() {
            let lhs_span_start = lhs.span().start();
            match token.kind() {
                TokenKind::Punctuator(Punctuator::OpenParen) => {
                    let (args, args_span, _) = Arguments::new(self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    lhs = Call::new(lhs, args, args_span).into();
                }
                TokenKind::Punctuator(Punctuator::Dot) => {
                    cursor.advance(interner);

                    let token = cursor.next(interner).or_abrupt()?;
                    let access = match token.kind() {
                        TokenKind::IdentifierName((name, _)) => {
                            SimplePropertyAccess::new(lhs, Identifier::new(*name, token.span()))
                                .into()
                        }
                        TokenKind::Keyword((kw, _)) => SimplePropertyAccess::new(
                            lhs,
                            Identifier::new(kw.to_sym(), token.span()),
                        )
                        .into(),
                        TokenKind::BooleanLiteral((true, _)) => {
                            SimplePropertyAccess::new(lhs, Identifier::new(Sym::TRUE, token.span()))
                                .into()
                        }
                        TokenKind::BooleanLiteral((false, _)) => SimplePropertyAccess::new(
                            lhs,
                            Identifier::new(Sym::FALSE, token.span()),
                        )
                        .into(),
                        TokenKind::NullLiteral(_) => {
                            SimplePropertyAccess::new(lhs, Identifier::new(Sym::NULL, token.span()))
                                .into()
                        }
                        TokenKind::PrivateIdentifier(name) => PrivatePropertyAccess::new(
                            lhs,
                            PrivateName::new(*name, token.span()),
                            Span::new(lhs_span_start, token.span().end()),
                        )
                        .into(),
                        _ => {
                            return Err(Error::expected(
                                ["identifier".to_owned()],
                                token.to_string(interner),
                                token.span(),
                                "call expression",
                            ));
                        }
                    };

                    lhs = ast::Expression::PropertyAccess(access);
                }
                TokenKind::Punctuator(Punctuator::OpenBracket) => {
                    cursor.advance(interner);
                    let idx = Expression::new(true, self.allow_yield, self.allow_await)
                        .parse(cursor, interner)?;
                    cursor.expect(Punctuator::CloseBracket, "call expression", interner)?;
                    lhs =
                        ast::Expression::PropertyAccess(SimplePropertyAccess::new(lhs, idx).into());
                }
                TokenKind::TemplateNoSubstitution { .. } | TokenKind::TemplateMiddle { .. } => {
                    lhs = TaggedTemplateLiteral::new(
                        self.allow_yield,
                        self.allow_await,
                        token.start_group(),
                        lhs,
                    )
                    .parse(cursor, interner)?
                    .into();
                }
                _ => break,
            }
        }

        Ok(lhs)
    }
}

/// Converts the arguments of an `async(...)` call into the formal parameter list of an
/// async arrow function, when the call is really an `AsyncArrowFunction` head.
///
/// More information:
///  - [ECMAScript specification][spec]
///
/// [spec]: https://tc39.es/ecma262/#prod-CoverCallExpressionAndAsyncArrowHead
fn arguments_to_formal_parameters(
    args: &[ast::Expression],
    has_trailing_comma: bool,
    strict: bool,
    args_span: Span,
) -> ParseResult<FormalParameterList> {
    let mut parameters = Vec::new();
    let num_args = args.len();

    for (i, arg) in args.iter().enumerate() {
        if let ast::Expression::Spread(spread) = arg {
            if i != num_args - 1 {
                return Err(Error::general(
                    "rest parameter must be last formal parameter",
                    spread.span().start(),
                ));
            }
            if has_trailing_comma {
                return Err(Error::general(
                    "rest parameter must be last formal parameter",
                    args_span.end(),
                ));
            }

            let declaration = match spread.target() {
                ast::Expression::Identifier(ident) => {
                    check_strict_parameter_name(ident, strict, spread.span())?;
                    Variable::from_identifier(*ident, None)
                }
                ast::Expression::ObjectLiteral(object) => {
                    let pattern = object.to_pattern(strict).ok_or_else(|| {
                        Error::general(
                            "invalid object binding pattern in formal parameter list",
                            spread.span().start(),
                        )
                    })?;
                    Variable::from_pattern(pattern.into(), None)
                }
                ast::Expression::ArrayLiteral(array) => {
                    let pattern = array.to_pattern(strict).ok_or_else(|| {
                        Error::general(
                            "invalid array binding pattern in formal parameter list",
                            spread.span().start(),
                        )
                    })?;
                    Variable::from_pattern(pattern.into(), None)
                }
                _ => {
                    return Err(Error::unexpected(
                        ")".to_string(),
                        spread.span(),
                        "parenthesized expression with non-binding expression",
                    ));
                }
            };

            parameters.push(FormalParameter::new(declaration, true));
        } else {
            expression_to_formal_parameters(arg, &mut parameters, strict, args_span)?;
        }
    }

    let parameters = FormalParameterList::from(parameters);

    // `await` is never a valid binding name inside an async arrow function's parameters.
    if bound_names(&parameters).contains(&Sym::AWAIT) {
        return Err(Error::general(
            "keyword `await` not allowed in this context",
            args_span.start(),
        ));
    }

    Ok(parameters)
}

/// Checks that `ident` is a valid parameter name in strict mode.
fn check_strict_parameter_name(ident: &Identifier, strict: bool, span: Span) -> ParseResult<()> {
    if strict {
        match ident.sym() {
            Sym::EVAL => {
                return Err(Error::general(
                    "parameter name 'eval' not allowed in strict mode",
                    span.start(),
                ));
            }
            Sym::ARGUMENTS => {
                return Err(Error::general(
                    "parameter name 'arguments' not allowed in strict mode",
                    span.start(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}
