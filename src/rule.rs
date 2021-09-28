use crate::error::Error;
use crate::style::StyleTag;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;
use std::str::FromStr;
use std::string::String as StdString;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Language<T> {
    Alphabet(T),
    Any,
    Begin,
    End,
    KleenStar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct String<T>(Vec<Language<T>>);

#[derive(Clone, Debug, Eq, PartialEq)]
enum Op<T> {
    Alphabet(T),
    Any,
    Begin,
    End,
}

type Edge<T> = (Option<Op<T>>, usize);
type Node<T> = Vec<Edge<T>>;

#[derive(Debug, Clone)]
struct Nfa<T> {
    nodes: Vec<Node<T>>,
    start: usize,
    end: usize,
}

impl<T> Nfa<T> {
    fn from_op(op: Op<T>) -> Self {
        Self {
            nodes: vec![vec![(Some(op), 1)], vec![]],
            start: 0,
            end: 1,
        }
    }
    fn kleen_star(&mut self) {
        let old_end_index = self.end;
        let old_start_index = self.start;
        self.nodes.push(vec![]);
        self.end = self.nodes.len() - 1;
        self.nodes[old_end_index].push((None, self.end));
        self.nodes[old_end_index].push((None, old_start_index));
        self.nodes
            .push(vec![(None, old_start_index), (None, self.end)]);
        self.start = self.nodes.len() - 1;
    }
    fn concat(&mut self, mut other: Nfa<T>) {
        let offset = self.nodes.len();
        for node in other.nodes.iter_mut() {
            for edge in node.iter_mut() {
                edge.1 += offset;
            }
        }
        self.nodes.append(&mut other.nodes);
        self.nodes[self.end].push((None, other.start + offset));
        self.end = other.end + offset;
    }
}

impl FromStr for StyleTag {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use StyleTag::*;
        Ok(match s {
            "p" => P,
            "h1" => H1,
            "h2" => H2,
            "h3" => H3,
            "h4" => H4,
            "h5" => H5,
            "blockquote" => Blockquote,
            "code" => Code,
            "codeblock" => Codeblock,
            "ul" => Ul,
            "ol" => Ol,
            "li" => Li,
            "em" => Em,
            "strong" => Strong,
            "strikethrough" => Strikethrough,
            "a" => A,
            "img" => Img,
            "imgcaption" => ImgCaption,
            _ => return Err(Error::InvalidRuleTag(s.to_string())),
        })
    }
}

impl FromStr for String<StyleTag> {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = vec![Language::Begin];
        let mut is_direct_child = false;
        let mut current_word = StdString::new();
        for ch in s.chars() {
            if ch.is_whitespace() {
                if current_word.is_empty() {
                    continue;
                } else {
                    if current_word == ">" {
                        is_direct_child = true;
                    } else {
                        if !is_direct_child {
                            out.push(Language::Any);
                            out.push(Language::KleenStar);
                        } else {
                            is_direct_child = false;
                        }
                        match current_word.as_str() {
                            "*" => out.push(Language::Any),
                            tag => out.push(Language::Alphabet(tag.parse()?)),
                        }
                    }
                    current_word = StdString::new();
                }
            } else {
                current_word.push(ch)
            }
        }
        if !current_word.is_empty() {
            if current_word == ">" {
                return Err(Error::DanglingDirectChild);
            } else {
                if !is_direct_child {
                    out.push(Language::Any);
                    out.push(Language::KleenStar);
                }
                match current_word.as_str() {
                    "*" => out.push(Language::Any),
                    tag => out.push(Language::Alphabet(tag.parse()?)),
                }
            }
        }
        Ok(Self(out))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[test]
    fn nfa() {
        let mut nfa = Nfa::<StyleTag>::from_op(Op::Any);
        nfa.kleen_star();
        nfa.concat(Nfa::<StyleTag>::from_op(Op::End));
        let mut other_nfa = Nfa::<StyleTag>::from_op(Op::Begin);
        other_nfa.concat(nfa);
        println!("{:#?}", other_nfa);
        todo!()
    }

    #[test]
    fn parse_language() -> Result<()> {
        use Language::{Any, *};
        use StyleTag::*;

        assert_eq!(
            "a".parse::<String<StyleTag>>()?,
            String(vec![Begin, Any, KleenStar, Alphabet(A)])
        );
        assert_eq!(
            "> a".parse::<String<StyleTag>>()?,
            String(vec![Begin, Alphabet(A)])
        );
        assert!("a >".parse::<String<StyleTag>>().is_err());
        assert_eq!(
            "a a".parse::<String<StyleTag>>()?,
            String(vec![
                Begin,
                Any,
                KleenStar,
                Alphabet(A),
                Any,
                KleenStar,
                Alphabet(A)
            ])
        );
        assert_eq!(
            "a > a".parse::<String<StyleTag>>()?,
            String(vec![Begin, Any, KleenStar, Alphabet(A), Alphabet(A)])
        );
        assert!("b".parse::<String<StyleTag>>().is_err());
        assert!("a b".parse::<String<StyleTag>>().is_err());
        assert_eq!(
            "ul > li em".parse::<String<StyleTag>>()?,
            String(vec![
                Begin,
                Any,
                KleenStar,
                Alphabet(Ul),
                Alphabet(Li),
                Any,
                KleenStar,
                Alphabet(Em)
            ])
        );
        Ok(())
    }
}
