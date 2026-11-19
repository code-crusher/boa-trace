use crate::{
    lexer::{InputElement, Token, TokenKind},
    parser::cursor::BufferedLexer,
};
use boa_ast::Punctuator;
use boa_interner::Interner;
use boa_macros::utf16;

#[test]
fn peek_skip_ascending() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
}

#[test]
fn peek_skip_next() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("d", utf16!("d")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("e", utf16!("e")))
    );
    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("f", utf16!("f")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("g", utf16!("g")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("h", utf16!("h")))
    );
}

#[test]
fn peek_skip_next_alternating() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("a", utf16!("a")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("b", utf16!("b")))
    );
    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("d", utf16!("d")))
    );
    assert_eq!(
        *cur.next(false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("c", utf16!("c")))
    );
    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("f", utf16!("f")))
    );
}

#[test]
fn peek_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    loop {
        let peek = cur.peek(0, false, interner).unwrap().cloned();
        let next = cur.next(false, interner).unwrap();

        assert_eq!(peek, next);

        if peek.is_none() {
            break;
        }
    }
}

#[test]
fn peek_skip_next_till_end() {
    let mut cur = BufferedLexer::from(&b"a b c d e f g h i"[..]);
    let interner = &mut Interner::default();

    let mut peeked: [Option<Token>; super::MAX_PEEK_SKIP + 1] =
        [None::<Token>, None::<Token>, None::<Token>, None::<Token>];

    loop {
        for (i, peek) in peeked.iter_mut().enumerate() {
            *peek = cur.peek(i, false, interner).unwrap().cloned();
        }

        for peek in &peeked {
            assert_eq!(&cur.next(false, interner).unwrap(), peek);
        }

        if peeked[super::MAX_PEEK_SKIP - 1].is_none() {
            break;
        }
    }
}

#[test]
fn skip_peeked_terminators() {
    let mut cur = BufferedLexer::from(&b"A \n B"[..]);
    let interner = &mut Interner::default();

    assert_eq!(
        *cur.peek(0, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("A", utf16!("A")))
    );
    assert_eq!(
        *cur.peek(0, true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("A", utf16!("A")))
    );

    assert_eq!(
        *cur.peek(1, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::LineTerminator,
    );
    assert_eq!(
        *cur.peek(1, true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("B", utf16!("B"))) /* This value is after the line terminator */
    );

    assert_eq!(
        *cur.peek(2, false, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("B", utf16!("B")))
    );
    // End of stream
    assert!(cur.peek(2, true, interner).unwrap().is_none());
}

#[test]
fn issue_1768() {
    let mut cur = BufferedLexer::from(&b"\n(\nx\n)\n"[..]);
    let interner = &mut Interner::default();

    assert!(cur.peek(3, true, interner).unwrap().is_none());
}

#[test]
#[cfg(feature = "annex-b")]
fn html_close_comment_at_start() {
    let mut cur = BufferedLexer::from(&b"--> comment\ntrue"[..]);
    let interner = &mut Interner::default();
    cur.set_goal(InputElement::HashbangOrRegExp);

    assert_eq!(
        *cur.peek(0, true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::boolean_literal(true),
    );
    assert_eq!(
        *cur.next(true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::boolean_literal(true),
    );
    // End of stream
    assert!(cur.peek(0, true, interner).unwrap().is_none());
}

#[test]
#[cfg(feature = "annex-b")]
fn html_close_comment_at_start_with_whitespace() {
    let mut cur = BufferedLexer::from(&b"   \t --> comment\ntrue"[..]);
    let interner = &mut Interner::default();
    cur.set_goal(InputElement::HashbangOrRegExp);

    assert_eq!(
        *cur.peek(0, true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::boolean_literal(true),
    );
}

#[test]
#[cfg(feature = "annex-b")]
fn html_close_comment_at_start_with_comments() {
    let mut cur = BufferedLexer::from(&b"/* a */ /* b */--> comment\ntrue"[..]);
    let interner = &mut Interner::default();
    cur.set_goal(InputElement::HashbangOrRegExp);

    assert_eq!(
        *cur.peek(0, true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::boolean_literal(true),
    );
}

#[test]
#[cfg(feature = "annex-b")]
fn html_close_comment_not_at_start() {
    // `-->` in the middle of a source text is not an HTML close comment; it must be lexed
    // as the `--` and `>` punctuators instead.
    let mut cur = BufferedLexer::from(&b"true --> comment"[..]);
    let interner = &mut Interner::default();
    cur.set_goal(InputElement::HashbangOrRegExp);

    assert_eq!(
        *cur.next(true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::boolean_literal(true),
    );
    assert_eq!(
        *cur.next(true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::Punctuator(Punctuator::Dec),
    );
    assert_eq!(
        *cur.next(true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::Punctuator(Punctuator::GreaterThan),
    );
    assert_eq!(
        *cur.next(true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::identifier(interner.get_or_intern_static("comment", utf16!("comment"))),
    );
}

#[test]
#[cfg(not(feature = "annex-b"))]
fn html_close_comment_at_start_without_annex_b() {
    let mut cur = BufferedLexer::from(&b"--> comment\ntrue"[..]);
    let interner = &mut Interner::default();
    cur.set_goal(InputElement::HashbangOrRegExp);

    assert_eq!(
        *cur.next(true, interner)
            .unwrap()
            .expect("Some value expected")
            .kind(),
        TokenKind::Punctuator(Punctuator::Dec),
    );
}
