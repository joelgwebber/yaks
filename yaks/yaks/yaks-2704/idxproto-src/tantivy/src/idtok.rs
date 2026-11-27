//! Identifier-aware tokenizer for tantivy.
//!
//! A *run* is word characters joined by single `-`, `_` or `/` (a joiner must sit between two word
//! characters). Each run emits, in position order:
//! - every part (`yaks`, `ca59`), one position each;
//! - for a `/`-component with several parts (`yaks-ca59`), the whole component, colocated with its last part;
//! - for a run with several components (`issues/123`, `/yaks/yaks/yaks-ca59`), the whole run, colocated with its last part.
//!
//! Anything else is split like `SimpleTokenizer`. Case folding and accent folding are filters on top.
use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

#[derive(Clone, Default)]
pub struct IdentTokenizer {
    tokens: Vec<Token>,
}

pub struct IdentStream<'a> {
    tokens: &'a mut [Token],
    i: usize,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric()
}
fn is_joiner(c: char) -> bool {
    matches!(c, '-' | '_' | '/')
}

impl Tokenizer for IdentTokenizer {
    type TokenStream<'a> = IdentStream<'a>;
    fn token_stream<'a>(&'a mut self, text: &'a str) -> IdentStream<'a> {
        self.tokens.clear();
        let cs: Vec<(usize, char)> = text.char_indices().collect();
        let end_of = |k: usize| cs.get(k + 1).map_or(text.len(), |x| x.0);
        let mut pos = 0usize;
        let mut k = 0;
        while k < cs.len() {
            if !is_word(cs[k].1) {
                k += 1;
                continue;
            }
            // collect the run as components of parts: Vec<Vec<(start,end)>> with joiner kinds
            let run_start = cs[k].0;
            let mut comps: Vec<Vec<(usize, usize)>> = vec![vec![]];
            let mut run_end = run_start;
            loop {
                // a part
                let s = cs[k].0;
                let mut e = end_of(k);
                k += 1;
                while k < cs.len() && is_word(cs[k].1) {
                    e = end_of(k);
                    k += 1;
                }
                comps.last_mut().unwrap().push((s, e));
                run_end = e;
                // joiner followed by a word char continues the run
                if k + 1 < cs.len() && is_joiner(cs[k].1) && is_word(cs[k + 1].1) {
                    if cs[k].1 == '/' {
                        comps.push(vec![]);
                    }
                    k += 1;
                } else {
                    break;
                }
            }
            for comp in &comps {
                for &(a, b) in comp {
                    self.tokens.push(Token { offset_from: a, offset_to: b, position: pos, text: text[a..b].to_string(), position_length: 1 });
                    pos += 1;
                }
                if comp.len() > 1 {
                    let (a, b) = (comp[0].0, comp[comp.len() - 1].1);
                    self.tokens.push(Token { offset_from: a, offset_to: b, position: pos - 1, text: text[a..b].to_string(), position_length: comp.len() });
                }
            }
            if comps.len() > 1 {
                let total: usize = comps.iter().map(|c| c.len()).sum();
                self.tokens.push(Token { offset_from: run_start, offset_to: run_end, position: pos - 1, text: text[run_start..run_end].to_string(), position_length: total });
            }
        }
        IdentStream { tokens: &mut self.tokens, i: 0 }
    }
}

impl TokenStream for IdentStream<'_> {
    fn advance(&mut self) -> bool {
        if self.i < self.tokens.len() {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn token(&self) -> &Token {
        &self.tokens[self.i - 1]
    }
    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.i - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn toks(s: &str) -> Vec<(String, usize)> {
        let mut t = IdentTokenizer::default();
        let mut st = t.token_stream(s);
        let mut v = vec![];
        while st.advance() {
            v.push((st.token().text.clone(), st.token().position));
        }
        v
    }
    #[test]
    fn idents() {
        let t = |a: &[(&str, usize)]| a.iter().map(|(s, p)| (s.to_string(), *p)).collect::<Vec<_>>();
        assert_eq!(toks("see issues/123."), t(&[("see", 0), ("issues", 1), ("123", 2), ("issues/123", 2)]));
        assert_eq!(toks("(yaks-ca59)"), t(&[("yaks", 0), ("ca59", 1), ("yaks-ca59", 1)]));
        assert_eq!(
            toks("/yaks/yaks/yaks-ca59.md"),
            t(&[("yaks", 0), ("yaks", 1), ("yaks", 2), ("ca59", 3), ("yaks-ca59", 3), ("yaks/yaks/yaks-ca59", 3), ("md", 4)])
        );
        assert_eq!(toks("a - b"), t(&[("a", 0), ("b", 1)]));
    }
}
