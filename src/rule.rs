use crate::error::{Error, Result};
use crate::style::StyleTag;
use std::collections::HashMap;
use std::str::FromStr;
use std::string::String as StdString;

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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Op<T> {
    Alphabet(T),
    Any,
    Begin,
    End,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Set<T>(Vec<T>);

impl<T> Set<T>
where
    T: Ord,
{
    fn new() -> Self {
        Self(Vec::new())
    }
    fn insert(&mut self, item: T) {
        if let Err(pos) = self.0.binary_search(&item) {
            self.0.insert(pos, item)
        }
    }
    fn merge(&mut self, other: Set<T>) {
        for item in other.0 {
            self.insert(item)
        }
    }
    fn contains(&self, item: &T) -> bool {
        self.0.binary_search(&item).is_ok()
    }
    fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

impl<T> From<Vec<T>> for Set<T>
where
    T: Ord,
{
    fn from(mut list: Vec<T>) -> Self {
        list.sort();
        Self(list)
    }
}

type NfaEdge<T> = (Option<Op<T>>, usize);
type NfaNode<T> = Vec<NfaEdge<T>>;
type NodeSet = Set<usize>;

#[derive(Debug, Clone)]
struct Nfa<T> {
    nodes: Vec<NfaNode<T>>,
    start: usize,
    end: usize,
}

impl<T> Nfa<T> {
    fn from_string(string: String<T>) -> Result<Self> {
        let mut stack = Vec::new();
        for symbol in string.0.into_iter() {
            match symbol {
                Language::Any => stack.push(Self::from_op(Op::Any)),
                Language::Begin => stack.push(Self::from_op(Op::Begin)),
                Language::End => stack.push(Self::from_op(Op::End)),
                Language::Alphabet(t) => stack.push(Self::from_op(Op::Alphabet(t))),
                Language::KleenStar => {
                    stack
                        .last_mut()
                        .ok_or_else(|| Error::EmptyRuleString)?
                        .kleen_star();
                }
            }
        }
        stack
            .into_iter()
            .reduce(|mut nfa, next| {
                nfa.concat(next);
                nfa
            })
            .ok_or_else(|| Error::EmptyRuleString)
    }
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
    fn epsilon_closures(&self) -> Vec<Set<usize>> {
        let mut out = Vec::new();
        for (idx, _) in self.nodes.iter().enumerate() {
            let mut unexplored_nodes: Set<usize> = vec![idx].into();
            let mut explored_nodes = Set::new();
            while let Some(node_idx) = unexplored_nodes.pop() {
                explored_nodes.insert(node_idx);
                for (transition, link_idx) in &self.nodes[node_idx] {
                    if transition.is_some() || explored_nodes.contains(link_idx) {
                        continue;
                    }
                    unexplored_nodes.insert(*link_idx)
                }
            }
            out.push(explored_nodes)
        }
        out
    }
}

type DfaEdge<T> = (Set<Op<T>>, NodeSet);

struct Dfa<T> {
    nodes: HashMap<NodeSet, DfaEdge<T>>,
    start: NodeSet,
    end: Vec<NodeSet>,
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
    fn nfa() -> Result<()> {
        use Language::{Any, *};
        use StyleTag::*;

        let nfa = Nfa::from_string(String(vec![
            Begin,
            Any,
            KleenStar,
            Alphabet(Ul),
            Alphabet(Li),
            Any,
            KleenStar,
            Alphabet(Em),
        ]))?;

        let expected_nfa = {
            let mut n = Nfa::from_op(Op::Begin);
            let any_star = {
                let mut n = Nfa::from_op(Op::Any);
                n.kleen_star();
                n
            };
            n.concat(any_star.clone());
            n.concat(Nfa::from_op(Op::Alphabet(Ul)));
            n.concat(Nfa::from_op(Op::Alphabet(Li)));
            n.concat(any_star.clone());
            n.concat(Nfa::from_op(Op::Alphabet(Em)));
            n
        };

        assert_eq!(nfa.nodes, expected_nfa.nodes);
        assert_eq!(nfa.start, expected_nfa.start);
        assert_eq!(nfa.end, expected_nfa.end);

        Ok(())
    }

    #[test]
    fn epsilon_closures() {
        let mut nfa = Nfa::<StyleTag>::from_op(Op::Begin);
        let any_star = {
            let mut n = Nfa::from_op(Op::Any);
            n.kleen_star();
            n
        };
        nfa.concat(any_star);
        let expected_epsilon = vec![
            vec![0].into(),
            vec![1, 2, 4, 5].into(),
            vec![2].into(),
            vec![2, 3, 4].into(),
            vec![4].into(),
            vec![2, 4, 5].into(),
        ];

        assert_eq!(nfa.epsilon_closures(), expected_epsilon);
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
