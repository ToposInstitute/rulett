//! Helpers for pretty-printing via the [`pretty`] crate.

use pretty::RcDoc;
use std::fmt;

/// Default line width used when rendering [`Display`](fmt::Display) output.
pub const DEFAULT_WIDTH: usize = 80;

/// Render a [`pretty`] document to a formatter, honoring its width specifier.
pub fn render_doc(doc: RcDoc<'_>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let width = f.width().unwrap_or(DEFAULT_WIDTH);
    doc.render_fmt(width, f)
}

/// Comma-separated, line-breakable items wrapped in `open`/`close` delimiters.
pub fn bracketed<'a>(
    open: &'static str,
    close: &'static str,
    items: impl IntoIterator<Item = RcDoc<'a>>,
) -> RcDoc<'a> {
    let sep = RcDoc::text(",").append(RcDoc::line());
    RcDoc::text(open)
        .append(RcDoc::line_().append(RcDoc::intersperse(items, sep)).nest(2))
        .append(RcDoc::line_())
        .append(close)
        .group()
}

/// Pretty document for `<tm> : <ty> ⊢ <body>`, breakable after `⊢`.
pub fn judgment_doc<'a>(tm: RcDoc<'a>, ty: RcDoc<'a>, body: RcDoc<'a>) -> RcDoc<'a> {
    tm.append(RcDoc::text(" : "))
        .append(ty)
        .append(RcDoc::text(" ⊢"))
        .append(RcDoc::line().append(body).nest(2))
        .group()
}

/// Pretty document for `<mor> : <dom> → <cod>`, breakable before `:` and `→`.
pub fn mor_doc<'a>(mor: RcDoc<'a>, dom: RcDoc<'a>, cod: RcDoc<'a>) -> RcDoc<'a> {
    mor.append(RcDoc::line())
        .append(RcDoc::text(": "))
        .append(dom)
        .append(RcDoc::line())
        .append(RcDoc::text("→ "))
        .append(cod)
        .nest(2)
        .group()
}
