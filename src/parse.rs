use crate::*;

impl Expr {
    pub fn parse(source: &str) -> Option<Self> {
        let source = source.trim();
        if let Some(Some(source)) = source.strip_prefix("(").map(|x| x.strip_suffix(")")) {
            Some(Expr::List(
                tokenize(source)?
                    .iter()
                    .map(|x| Expr::parse(x))
                    .collect::<Option<Vec<_>>>()?,
            ))
        } else {
            Some(Expr::Atom(Atom::parse(source)?))
        }
    }
}

impl Atom {
    pub fn parse(source: &str) -> Option<Self> {
        let source = source.trim();
        if let Ok(n) = source.parse::<i64>() {
            Some(Atom::Integer(n))
        } else {
            Some(Atom::Symbol("lisp_".to_string() + source))
        }
    }
}
