use crate::SpinnerAst;

use super::helpers::unquote;
use super::{AstError, Rule};

pub(super) fn parse_spinner_pair(sp: pest::iterators::Pair<Rule>, _source: &str) -> Result<SpinnerAst, AstError> {
    let (src_line, _src_col) = sp.as_span().start_pos().line_col();
    let mut it = sp.into_inner();
    let id = it
        .next()
        .ok_or(AstError::Shape("expected spinner ident"))?
        .as_str()
        .to_string();
    let block = it.next().ok_or(AstError::Shape("expected spinner block"))?;
    let mut entries = Vec::new();
    for entry in block.into_inner() {
        let mut inner = entry.into_inner();
        let text_pair = inner.next().ok_or(AstError::Shape("spinner entry text"))?;
        let text = unquote(text_pair.as_str());
        entries.push(text);
    }
    Ok(SpinnerAst { id, entries, src_line })
}
