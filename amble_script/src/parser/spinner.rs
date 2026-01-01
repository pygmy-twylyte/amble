use crate::{SpinnerAst, SpinnerWedgeAst};

use super::helpers::unquote;
use super::{AstError, Rule};

pub(super) fn parse_spinner_pair(
    sp: pest::iterators::Pair<Rule>,
    _source: &str,
) -> Result<SpinnerAst, AstError> {
    let (src_line, _src_col) = sp.as_span().start_pos().line_col();
    let mut it = sp.into_inner();
    let id = it
        .next()
        .ok_or(AstError::Shape("expected spinner ident"))?
        .as_str()
        .to_string();
    let block = it.next().ok_or(AstError::Shape("expected spinner block"))?;
    let mut wedges = Vec::new();
    for w in block.into_inner() {
        let mut wi = w.into_inner();
        let text_pair = wi.next().ok_or(AstError::Shape("wedge text"))?;
        let text = unquote(text_pair.as_str());
        // width is optional; default to 1
        let width: usize = if let Some(width_pair) = wi.next() {
            let width = width_pair
                .as_str()
                .parse()
                .map_err(|_| AstError::Shape("invalid wedge width"))?;
            if width == 0 {
                return Err(AstError::Shape("wedge width must be at least 1"));
            }
            width
        } else {
            1
        };
        wedges.push(SpinnerWedgeAst { text, width });
    }
    Ok(SpinnerAst { id, wedges, src_line })
}
