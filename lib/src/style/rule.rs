use super::style_tag::StyleTag;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops;
use std::str::FromStr;
use std::string::String as StdString;

pub trait Enumerable: Sized {
    fn enumerate_all() -> Vec<Self>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Language<T> {
    Alphabet(T),
    Any,
    Begin,
    KleenStar,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct String<T>(Vec<Language<T>>);

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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Op<T> {
    Alphabet(T),
    Any,
    Begin,
    End,
}

impl<T> Op<T>
where
    T: PartialEq,
{
    fn is_satisfied_by(&self, other: &Op<T>) -> bool {
        match self {
            Op::Alphabet(a) => match other {
                Op::Alphabet(b) => a == b,
                _ => false,
            },
            Op::Any => match other {
                Op::Alphabet(_) => true,
                _ => false,
            },
            Op::Begin => matches!(other, Op::Begin),
            Op::End => matches!(other, Op::End),
        }
    }
}

impl<T> Enumerable for Op<T>
where
    T: Enumerable,
{
    fn enumerate_all() -> Vec<Self> {
        let mut ops = T::enumerate_all()
            .into_iter()
            .map(|t| Op::Alphabet(t))
            .collect::<Vec<_>>();
        ops.push(Op::Any);
        ops.push(Op::Begin);
        ops.push(Op::End);
        ops
    }
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
    fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }
}

impl<T> ops::Deref for Set<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
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

impl<T> FromIterator<T> for Set<T>
where
    T: Ord,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
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

impl<T> Nfa<T>
where
    T: PartialEq,
{
    fn transition(&self, idx: usize, op: &Op<T>) -> NodeSet {
        self.nodes[idx]
            .iter()
            .filter_map(|(link_op, node_idx)| {
                link_op.as_ref().and_then(|link_op| {
                    if link_op.is_satisfied_by(op) {
                        Some(*node_idx)
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
struct Dfa<T> {
    nodes: HashMap<NodeSet, HashMap<Op<T>, NodeSet>>,
    start: NodeSet,
    accepting_idx: usize,
}

impl<T> Dfa<T>
where
    T: Ord + Clone + PartialEq + Hash + Enumerable,
{
    fn from_nfa(nfa: &Nfa<T>) -> Self {
        let ops = Op::enumerate_all();
        let epsilon_closures = nfa.epsilon_closures();
        let start = epsilon_closures[nfa.start].clone();
        let mut nodes = HashMap::new();
        let mut unexplored_nodes = Set::new();
        let mut explored_nodes = Set::new();
        unexplored_nodes.insert(start.clone());
        while let Some(node_set) = unexplored_nodes.pop() {
            explored_nodes.insert(node_set.clone());
            // collect edges
            let mut edges = HashMap::new();
            // for each opeartion get possible node set
            for op in ops.iter() {
                let mut d_node_set = Set::new();
                for idx in node_set.iter() {
                    d_node_set.merge(nfa.transition(*idx, op));
                }
                if d_node_set.is_empty() {
                    continue;
                }
                // extend node set by epsilon closures
                d_node_set = d_node_set
                    .iter()
                    .map(|idx| epsilon_closures[*idx].clone())
                    .reduce(|mut agg, next| {
                        agg.merge(next);
                        agg
                    })
                    .unwrap();
                // if unexplored add node set to the list of states to explore
                if !explored_nodes.contains(&d_node_set) {
                    unexplored_nodes.insert(d_node_set.clone());
                }
                edges.insert(op.clone(), d_node_set);
            }
            nodes.insert(node_set, edges.into_iter().collect());
        }
        Self {
            nodes,
            start,
            accepting_idx: nfa.end,
        }
    }

    fn start(&self) -> &NodeSet {
        &self.start
    }

    fn transition(&self, state: &NodeSet, op: &Op<T>) -> Option<&NodeSet> {
        self.nodes.get(state)?.get(op)
    }

    fn is_accepting(&self, state: &NodeSet) -> bool {
        state.contains(&self.accepting_idx)
    }
}

#[derive(Clone, Debug)]
pub struct Rule {
    loose: Dfa<StyleTag>,
    exact: Dfa<StyleTag>,
}

impl Rule {
    fn from_loose_nfa(loose_nfa: &Nfa<StyleTag>) -> Self {
        let exact_nfa = {
            let mut nfa = loose_nfa.clone();
            nfa.concat(Nfa::from_op(Op::End));
            nfa
        };
        Self {
            loose: Dfa::from_nfa(loose_nfa),
            exact: Dfa::from_nfa(&exact_nfa),
        }
    }

    pub fn matches_loose(&self, tree: &[StyleTag]) -> bool {
        Self::matches_dfa(&self.loose, tree)
    }

    pub fn matches_exact(&self, tree: &[StyleTag]) -> bool {
        Self::matches_dfa(&self.exact, tree)
    }

    fn matches_dfa(dfa: &Dfa<StyleTag>, tree: &[StyleTag]) -> bool {
        let mut state = Some(dfa.start());
        let mut ops = Self::ops(tree);
        while let Some(node_set) = state {
            if dfa.is_accepting(node_set) {
                return true;
            }
            if let Some(op) = ops.next() {
                state = dfa.transition(node_set, &op);
            } else {
                break;
            }
        }
        false
    }

    fn ops<'a>(tree: &'a [StyleTag]) -> impl Iterator<Item = Op<StyleTag>> + 'a {
        std::iter::once(Op::Begin)
            .chain(tree.iter().map(|tag| Op::Alphabet(tag.clone())))
            .chain(Some(Op::End))
    }
}

pub fn parse_rules(string: impl AsRef<str>) -> Result<Vec<Rule>> {
    string
        .as_ref()
        .split(",")
        .map(|string| {
            Ok(Rule::from_loose_nfa(&Nfa::from_string(
                string.parse::<String<StyleTag>>()?,
            )?))
        })
        .collect()
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
        assert_eq!(
            nfa.epsilon_closures(),
            vec![
                vec![0].into(),
                vec![1, 2, 4, 5].into(),
                vec![2].into(),
                vec![2, 3, 4].into(),
                vec![4].into(),
                vec![2, 4, 5].into(),
            ]
        );
    }

    #[test]
    fn nfa_to_dsa_test_1() {
        use StyleTag::*;
        let nfa: Nfa<StyleTag> = Nfa {
            nodes: vec![
                vec![(None, 1), (None, 2)],
                vec![(Some(Op::Begin), 3)],
                vec![(Some(Op::End), 3)],
                vec![(Some(Op::Alphabet(A)), 4)],
                vec![],
            ],
            start: 0,
            end: 4,
        };
        let dfa = Dfa::from_nfa(&nfa);
        let expected_dfa: Dfa<StyleTag> = {
            let mut nodes = HashMap::new();
            let state_a: NodeSet = vec![0, 1, 2].into();
            let state_b: NodeSet = vec![3].into();
            let state_c: NodeSet = vec![4].into();
            nodes.insert(
                state_a.clone(),
                vec![(Op::Begin, state_b.clone()), (Op::End, state_b.clone())]
                    .into_iter()
                    .collect(),
            );
            nodes.insert(
                state_b,
                vec![(Op::Alphabet(A), state_c.clone())]
                    .into_iter()
                    .collect(),
            );
            nodes.insert(state_c, HashMap::new());
            Dfa {
                nodes,
                start: state_a,
                accepting_idx: 4,
            }
        };
        assert_eq!(dfa.nodes, expected_dfa.nodes);
        assert_eq!(dfa.start, expected_dfa.start);
        assert_eq!(dfa.accepting_idx, expected_dfa.accepting_idx);
    }

    #[test]
    fn nfa_to_dsa_test_2() {
        use StyleTag::*;
        let nfa: Nfa<StyleTag> = Nfa {
            nodes: vec![
                vec![(Some(Op::Begin), 0), (None, 1)],
                vec![(Some(Op::Alphabet(A)), 1), (None, 2)],
                vec![(Some(Op::End), 2)],
            ],
            start: 0,
            end: 2,
        };
        let dfa = Dfa::from_nfa(&nfa);
        let expected_dfa: Dfa<StyleTag> = {
            let mut nodes = HashMap::new();
            let state_a: NodeSet = vec![0, 1, 2].into();
            let state_b: NodeSet = vec![1, 2].into();
            let state_c: NodeSet = vec![2].into();
            nodes.insert(
                state_a.clone(),
                vec![
                    (Op::Alphabet(A), state_b.clone()),
                    (Op::Begin, state_a.clone()),
                    (Op::End, state_c.clone()),
                ]
                .into_iter()
                .collect(),
            );
            nodes.insert(
                state_b.clone(),
                vec![
                    (Op::Alphabet(A), state_b.clone()),
                    (Op::End, state_c.clone()),
                ]
                .into_iter()
                .collect(),
            );
            nodes.insert(
                state_c.clone(),
                vec![(Op::End, state_c.clone())].into_iter().collect(),
            );
            Dfa {
                nodes,
                start: state_a,
                accepting_idx: 2,
            }
        };
        assert_eq!(dfa.nodes, expected_dfa.nodes);
        assert_eq!(dfa.start, expected_dfa.start);
        assert_eq!(dfa.accepting_idx, expected_dfa.accepting_idx);
    }

    #[test]
    fn rule_matches() -> Result<()> {
        use StyleTag::*;
        let rule = parse_rules("> a")?.pop().unwrap();
        assert!(rule.matches_loose(&[A]));
        assert!(rule.matches_exact(&[A]));
        assert!(rule.matches_loose(&[A, P]));
        assert!(!rule.matches_exact(&[A, P]));
        assert!(!rule.matches_loose(&[P, A]));
        assert!(!rule.matches_exact(&[P, A]));
        let rule = parse_rules("ul > li em")?.pop().unwrap();
        assert!(rule.matches_loose(&[Ul, Li, Em]));
        assert!(rule.matches_exact(&[Ul, Li, Em]));
        assert!(rule.matches_loose(&[Ul, Li, Em, A]));
        assert!(!rule.matches_exact(&[Ul, Li, Em, A]));
        assert!(rule.matches_loose(&[P, Ul, Li, Em]));
        assert!(rule.matches_exact(&[P, Ul, Li, Em]));
        assert!(!rule.matches_loose(&[P, Ul, A, Em]));
        assert!(rule.matches_exact(&[P, Ul, Li, A, Em]));
        assert!(!rule.matches_loose(&[P, Ul, A, Li, Em]));
        assert!(rule.matches_exact(&[P, Ul, Li, Em, Em]));
        Ok(())
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
