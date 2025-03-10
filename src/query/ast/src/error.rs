// Copyright 2022 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::Write;
use std::num::IntErrorKind;
use std::num::ParseIntError;

use itertools::Itertools;
use logos::Span;
use pratt::NoError;
use pratt::PrattError;

use crate::input::Input;
use crate::parser::token::*;

const MAX_DISPLAY_ERROR_COUNT: usize = 6;

/// This error type accumulates errors and their position when backtracking
/// through a parse tree. This take a deepest error at `alt` combinator.
#[derive(Clone, Debug)]
pub struct Error<'a> {
    /// The next token when encountering an error.
    span: Token<'a>,
    /// List of errors tried in various branches that consumed
    /// the same (farthest) length of input.
    errors: Vec<ErrorKind>,
    /// The backtrace stack of the error.
    contexts: Vec<(Token<'a>, &'static str)>,
    /// The extra backtrace of error in optional branches.
    backtrace: &'a Backtrace<'a>,
}

/// ErrorKind is the error type returned from parser.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Error generated by `match_token` function
    ExpectToken(TokenKind),
    /// Error generated by `match_text` function
    ExpectText(&'static str),
    /// Plain text description of an error
    Other(&'static str),
}

/// Record the farthest position in the input before encountering an error.
///
/// This is similar to the `Error`, but the information will not get lost
/// even the error is from a optional branch.
#[derive(Debug, Clone, Default)]
pub struct Backtrace<'a> {
    inner: RefCell<Option<BacktraceInner<'a>>>,
}

impl<'a> Backtrace<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&self) {
        self.inner.replace(None);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BacktraceInner<'a> {
    /// The next token when encountering an error.
    span: Token<'a>,
    /// List of errors tried in various branches that consumed
    /// the same (farthest) length of input.
    errors: Vec<ErrorKind>,
}

impl<'a> nom::error::ParseError<Input<'a>> for Error<'a> {
    fn from_error_kind(i: Input<'a>, _: nom::error::ErrorKind) -> Self {
        Error {
            span: i[0].clone(),
            errors: vec![],
            contexts: vec![],
            backtrace: i.2,
        }
    }

    fn append(_: Input<'a>, _: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(_: Input<'a>, _: char) -> Self {
        unreachable!()
    }

    fn or(mut self, mut other: Self) -> Self {
        match self.span.span.start.cmp(&other.span.span.start) {
            Ordering::Equal => {
                self.errors.append(&mut other.errors);
                self.contexts.clear();
                self
            }
            Ordering::Less => other,
            Ordering::Greater => self,
        }
    }
}

impl<'a> nom::error::ContextError<Input<'a>> for Error<'a> {
    fn add_context(input: Input<'a>, ctx: &'static str, mut other: Self) -> Self {
        other.contexts.push((input.0[0].clone(), ctx));
        other
    }
}

impl<'a> Error<'a> {
    pub fn from_error_kind(input: Input<'a>, kind: ErrorKind) -> Self {
        let mut inner = input.2.inner.borrow_mut();
        if let Some(ref mut inner) = *inner {
            match input.0[0].span.start.cmp(&inner.span.span.start) {
                Ordering::Equal => {
                    inner.errors.push(kind);
                }
                Ordering::Less => (),
                Ordering::Greater => {
                    *inner = BacktraceInner {
                        span: input.0[0].clone(),
                        errors: vec![kind],
                    };
                }
            }
        } else {
            *inner = Some(BacktraceInner {
                span: input.0[0].clone(),
                errors: vec![kind],
            })
        }

        Error {
            span: input.0[0].clone(),
            errors: vec![kind],
            contexts: vec![],
            backtrace: input.2,
        }
    }
}

impl From<fast_float::Error> for ErrorKind {
    fn from(_: fast_float::Error) -> Self {
        ErrorKind::Other("unable to parse float number")
    }
}

impl From<ParseIntError> for ErrorKind {
    fn from(err: ParseIntError) -> Self {
        let msg = match err.kind() {
            IntErrorKind::InvalidDigit => {
                "unable to parse number because it contains invalid characters"
            }
            IntErrorKind::PosOverflow => "unable to parse number because it positively overflowed",
            IntErrorKind::NegOverflow => "unable to parse number because it negatively overflowed",
            _ => "unable to parse number",
        };
        ErrorKind::Other(msg)
    }
}

/// DisplayError is used to display errors in user-friendly way.
pub trait DisplayError {
    type Message;

    fn display_error(&self, message: Self::Message) -> String;
}

impl DisplayError for std::ops::Range<usize> {
    type Message = (String, String);

    fn display_error(&self, (source, message): Self::Message) -> String {
        pretty_print_error(&source, vec![(self.clone(), message)])
    }
}

impl<'a> DisplayError for Token<'a> {
    type Message = String;

    fn display_error(&self, message: Self::Message) -> String {
        pretty_print_error(self.source, vec![(self.span.clone(), message)])
    }
}

impl<'a> DisplayError for &'a [Token<'a>] {
    type Message = String;

    fn display_error(&self, message: Self::Message) -> String {
        assert!(!self.is_empty());
        let source = self.first().unwrap().source;
        let span_start = self.first().unwrap().span.start;
        let span_end = self.last().unwrap().span.end;
        pretty_print_error(source, vec![(span_start..span_end, message)])
    }
}

impl<'a> DisplayError for Error<'a> {
    type Message = ();

    fn display_error(&self, _: Self::Message) -> String {
        let inner = &*self.backtrace.inner.borrow();
        let inner = match inner {
            Some(inner) => inner,
            None => return String::new(),
        };

        let mut lables = vec![];

        // Plain text error has the hightest priority. Only display it if exists.
        for kind in &inner.errors {
            if let ErrorKind::Other(msg) = kind {
                lables = vec![(inner.span.span.clone(), msg.to_string())];
                break;
            }
        }

        // List all expected tokens in alternative branches.
        if lables.is_empty() {
            let expected_tokens = self
                .errors
                .iter()
                .chain(&inner.errors)
                .filter_map(|kind| match kind {
                    ErrorKind::ExpectToken(EOI) => None,
                    ErrorKind::ExpectToken(token) if token.is_keyword() => {
                        Some(format!("`{:?}`", token))
                    }
                    ErrorKind::ExpectToken(token) => Some(format!("<{:?}>", token)),
                    ErrorKind::ExpectText(text) => Some(format!("`{}`", text)),
                    _ => None,
                })
                .unique()
                .collect::<Vec<_>>();

            let mut msg = String::new();
            let mut iter = expected_tokens.iter().enumerate().peekable();
            while let Some((i, error)) = iter.next() {
                if i == MAX_DISPLAY_ERROR_COUNT {
                    let more = expected_tokens
                        .len()
                        .saturating_sub(MAX_DISPLAY_ERROR_COUNT);
                    write!(msg, ", or {} more ...", more).unwrap();
                    break;
                } else if i == 0 {
                    msg += "expected ";
                } else if iter.peek().is_none() && i == 1 {
                    msg += " or ";
                } else if iter.peek().is_none() {
                    msg += ", or ";
                } else {
                    msg += ", ";
                }
                msg += error;
            }

            lables = vec![(inner.span.span.clone(), msg)];
        }

        // Append contexts as secondary lables.
        lables.extend(
            self.contexts
                .iter()
                .map(|(span, msg)| (span.span.clone(), format!("while parsing {}", msg))),
        );

        pretty_print_error(self.span.source, lables)
    }
}

fn pretty_print_error(source: &str, lables: Vec<(Span, String)>) -> String {
    use codespan_reporting::diagnostic::Diagnostic;
    use codespan_reporting::diagnostic::Label;
    use codespan_reporting::files::SimpleFile;
    use codespan_reporting::term;
    use codespan_reporting::term::termcolor::Buffer;
    use codespan_reporting::term::Chars;
    use codespan_reporting::term::Config;

    let mut writer = Buffer::no_color();
    let file = SimpleFile::new("SQL", source);
    let config = Config {
        chars: Chars::ascii(),
        before_label_lines: 3,
        ..Default::default()
    };

    let lables = lables
        .into_iter()
        .enumerate()
        .map(|(i, (span, msg))| {
            if i == 0 {
                Label::primary((), span).with_message(msg)
            } else {
                Label::secondary((), span).with_message(msg)
            }
        })
        .collect();

    let diagnostic = Diagnostic::error().with_labels(lables);

    term::emit(&mut writer, &config, &file, &diagnostic).unwrap();

    std::str::from_utf8(&writer.into_inner())
        .unwrap()
        .to_string()
}

impl<T: std::fmt::Debug> From<PrattError<T, pratt::NoError>> for ErrorKind {
    fn from(err: PrattError<T, NoError>) -> Self {
        match err {
            PrattError::EmptyInput => ErrorKind::Other("expected more tokens for expression"),
            PrattError::UnexpectedNilfix(_) => {
                ErrorKind::Other("unable to parse the expression value")
            }
            PrattError::UnexpectedPrefix(_) => {
                ErrorKind::Other("unable to parse the prefix operator")
            }
            PrattError::UnexpectedInfix(_) => {
                ErrorKind::Other("unable to parse the binary operator")
            }
            PrattError::UnexpectedPostfix(_) => {
                ErrorKind::Other("unable to parse the postfix operator")
            }
            PrattError::UserError(_) => unreachable!(),
        }
    }
}
