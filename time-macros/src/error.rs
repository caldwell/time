use std::fmt;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::format_description::error::InvalidFormatDescription;

trait WithSpan {
    fn with_span(self, span: Span) -> Self;
}

impl WithSpan for TokenTree {
    fn with_span(mut self, span: Span) -> Self {
        self.set_span(span);
        self
    }
}

pub(crate) enum Error {
    MissingComponent {
        name: &'static str,
        span_start: Option<Span>,
        span_end: Option<Span>,
    },
    InvalidComponent {
        name: &'static str,
        value: String,
        span_start: Option<Span>,
        span_end: Option<Span>,
    },
    ExpectedString,
    UnexpectedToken {
        tree: TokenTree,
    },
    UnexpectedEndOfInput,
    InvalidFormatDescription {
        error: InvalidFormatDescription,
        span_start: Option<Span>,
        span_end: Option<Span>,
    },
    Custom {
        message: String,
        span_start: Option<Span>,
        span_end: Option<Span>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingComponent { name, .. } => write!(f, "missing component: {}", name),
            Self::InvalidComponent { name, value, .. } => {
                write!(f, "invalid component: {} was {}", name, value)
            }
            Self::ExpectedString => f.write_str("expected string"),
            Self::UnexpectedToken { tree } => write!(f, "unexpected token: {}", tree),
            Self::UnexpectedEndOfInput => f.write_str("unexpected end of input"),
            Self::InvalidFormatDescription { error, .. } => error.fmt(f),
            Self::Custom { message, .. } => f.write_str(message),
        }
    }
}

impl Error {
    fn span_start(&self) -> Span {
        match self {
            Self::MissingComponent { span_start, .. }
            | Self::InvalidComponent { span_start, .. }
            | Self::InvalidFormatDescription { span_start, .. }
            | Self::Custom { span_start, .. } => *span_start,
            Self::UnexpectedToken { tree } => Some(tree.span()),
            Self::ExpectedString | Self::UnexpectedEndOfInput => Some(Span::mixed_site()),
        }
        .unwrap_or_else(Span::mixed_site)
    }

    fn span_end(&self) -> Span {
        match self {
            Self::MissingComponent { span_end, .. }
            | Self::InvalidComponent { span_end, .. }
            | Self::InvalidFormatDescription { span_end, .. }
            | Self::Custom { span_end, .. } => *span_end,
            Self::UnexpectedToken { tree, .. } => Some(tree.span()),
            Self::ExpectedString | Self::UnexpectedEndOfInput => Some(Span::mixed_site()),
        }
        .unwrap_or_else(|| self.span_start())
    }

    pub(crate) fn to_compile_error(&self) -> TokenStream {
        let (start, end) = (self.span_start(), self.span_end());

        [
            TokenTree::from(Punct::new(':', Spacing::Joint)).with_span(start),
            TokenTree::from(Punct::new(':', Spacing::Alone)).with_span(start),
            TokenTree::from(Ident::new("core", start)),
            TokenTree::from(Punct::new(':', Spacing::Joint)).with_span(start),
            TokenTree::from(Punct::new(':', Spacing::Alone)).with_span(start),
            TokenTree::from(Ident::new("compile_error", start)),
            TokenTree::from(Punct::new('!', Spacing::Alone)).with_span(start),
            TokenTree::from(Group::new(
                Delimiter::Parenthesis,
                TokenStream::from(
                    TokenTree::from(Literal::string(&self.to_string())).with_span(end),
                ),
            ))
            .with_span(end),
        ]
        .iter()
        .cloned()
        .collect()
    }
}