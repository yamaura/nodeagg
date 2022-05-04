use crate::iter::ConcatIterator;
use crate::parser::{comment, tabspaces};
use combine::parser::char::{alpha_num, char, digit, newline, spaces};
use combine::parser::combinator::attempt;
use combine::sep_end_by;
use combine::stream::Stream;
use combine::token;
use combine::{between, choice, eof, many, many1, sep_by1};

#[allow(non_camel_case_types)]
type number = u64;

/// node range expression
///
/// [00X-00Y] from x to y (including also x and y).
///
/// Currently it does not support skip like: [X-Y/Z]
#[derive(Default, Debug, PartialEq, Clone)]
pub struct NodeaggSubRange {
    pub start: number,
    pub stop: number,
    // TODO: skip is not supported yet.
    // TODO: impl PartialEq<NodeaggSubNumber>
}

impl NodeaggSubRange {
    pub fn iter(&self) -> Box<dyn Iterator<Item = number> + '_> {
        Box::new(self.start..(self.stop + 1))
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct NodeaggSubNumber {
    pub value: number,
}

impl NodeaggSubNumber {
    pub fn iter(&self) -> Box<dyn Iterator<Item = number> + '_> {
        Box::new(std::iter::once(self.value))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeaggSubArrayElementKind {
    Range(NodeaggSubRange),
    Number(NodeaggSubNumber),
}

#[derive(Debug, PartialEq, Clone)]
pub struct NodeaggSubArrayElement {
    pub width: usize,
    pub inner: NodeaggSubArrayElementKind,
}

impl NodeaggSubArrayElement {
    pub fn iter(&self) -> Box<dyn Iterator<Item = number> + '_> {
        match &self.inner {
            NodeaggSubArrayElementKind::Range(v) => v.iter(),
            NodeaggSubArrayElementKind::Number(v) => v.iter(),
        }
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct NodeaggSubArray {
    values: Vec<NodeaggSubArrayElement>,
}

impl Extend<NodeaggSubArrayElement> for NodeaggSubArray {
    fn extend<T: IntoIterator<Item = NodeaggSubArrayElement>>(&mut self, iter: T) {
        self.values.extend(iter);
    }
}

impl NodeaggSubArray {
    pub fn iter(&self) -> Box<dyn Iterator<Item = String> + '_> {
        // TODO: reduce copy
        // TODO: should use itertools::concat?
        let width = self.values.iter().map(|x| x.width).max().unwrap();
        Box::new(
            (ConcatIterator {
                itrs: self.values.iter().map(|e| e.iter()).collect::<Vec<_>>(),
            })
            .map(move |x| format!("{0:0>1$}", x, width)),
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeaggSubExpr {
    NodeaggSub(String),
    Array(NodeaggSubArray),
}

#[derive(Default, Debug, PartialEq)]
pub struct NodeaggBaseExpr(Vec<NodeaggSubExpr>);

impl Extend<NodeaggSubExpr> for NodeaggBaseExpr {
    fn extend<T: IntoIterator<Item = NodeaggSubExpr>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl NodeaggSubExpr {
    pub fn iter(&self) -> Box<dyn Iterator<Item = String> + '_> {
        match self {
            NodeaggSubExpr::NodeaggSub(s) => Box::new(std::iter::once(s.clone())),
            NodeaggSubExpr::Array(a) => a.iter(),
        }
    }
}

impl NodeaggBaseExpr {
    pub fn iter(&self) -> Box<dyn Iterator<Item = String> + '_> {
        use itertools::Itertools;
        if self.0.len() == 0 {
            Box::new(std::iter::empty::<String>())
        } else {
            // TODO: reduce clone
            Box::new(
                self.0
                    .iter()
                    .clone()
                    .map(|e| e.iter().map(|s| s.to_string()).collect::<Vec<String>>())
                    .multi_cartesian_product()
                    .map(|v| {
                        v.iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>()
                            .join("")
                    }),
            )
        }
    }
}

#[derive(Default, Debug)]
pub struct NodeaggExpr(Vec<NodeaggBaseExpr>);

impl Extend<NodeaggBaseExpr> for NodeaggExpr {
    fn extend<T: IntoIterator<Item = NodeaggBaseExpr>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl NodeaggExpr {
    pub fn iter(&self) -> itertools::Unique<ConcatIterator<'_, String>> {
        // TODO: reduce copy
        // TODO: should use itertools::concat?
        use itertools::Itertools;
        (ConcatIterator {
            itrs: self.0.iter().map(|e| e.iter()).collect::<Vec<_>>(),
        })
        .unique()
    }
}

parser! {
    pub fn nodeagg_sub_expr[Input]()(Input) -> NodeaggSubExpr
        where [Input: Stream<Token = char>]
{
    use std::cmp::max;
    //let skip_spaces = || spaces().silent();

    // TODO: shold relaxing characters and shold not use parser::char::alpha_num?
    let word = many1(alpha_num().or(char('.')).or(char('-')));
    let number = many1(digit()).map(|t: String|
        NodeaggSubArrayElement {
            width: t.len(),
            inner: NodeaggSubArrayElementKind::Number (
                NodeaggSubNumber {
                    value: t.parse::<number>().unwrap()
                }
            )}
    );
    let range = (many1(digit()), char('-'), many1(digit())).map(|t: (String, _, String)|
        NodeaggSubArrayElement {
            width: max(t.0.len(), t.2.len()),
            inner: NodeaggSubArrayElementKind::Range (
                NodeaggSubRange {
                    start: t.0.parse::<number>().unwrap(),
                    stop: t.2.parse::<number>().unwrap(),
                })}
        );

    let comma_list = sep_by1::<NodeaggSubArray, _, _, _>(
        attempt(range).or(number)
        , char(','));
    let array = between(char('['), char(']'), comma_list);
    choice((
            word.map(NodeaggSubExpr::NodeaggSub),
            array.map(NodeaggSubExpr::Array),
    ))
}
}

parser! {
    pub fn nodeagg_base_expr[Input]()(Input) -> NodeaggBaseExpr
        where [Input: Stream<Token = char>]
{
        many(nodeagg_sub_expr()).skip(tabspaces())
}
}

parser! {
pub fn nodeagg_expr[Input]()(Input) -> NodeaggExpr
    where [Input: Stream<Token = char>]
{
    //spaces().with(sep_end_by(nodeagg_base_expr().skip(comment()), token(',').or(newline()).skip(spaces()).skip(comment()))).skip(spaces().skip(comment()).with(eof()))
    spaces().with(sep_end_by(nodeagg_base_expr().skip(comment()),
            (token(',').or(newline())).skip(tabspaces()).skip(comment())
            )).skip(tabspaces().skip(comment()).with(eof()))
}}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::EasyParser;
    use combine::Parser;

    // for test use
    impl From<Vec<NodeaggSubExpr>> for NodeaggBaseExpr {
        fn from(item: Vec<NodeaggSubExpr>) -> Self {
            NodeaggBaseExpr(item)
        }
    }

    // for test use
    impl From<NodeaggExpr> for Vec<NodeaggBaseExpr> {
        fn from(item: NodeaggExpr) -> Self {
            item.0
        }
    }

    impl From<Vec<NodeaggBaseExpr>> for NodeaggExpr {
        fn from(item: Vec<NodeaggBaseExpr>) -> Self {
            NodeaggExpr(item)
        }
    }

    impl PartialEq for NodeaggExpr {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0
        }
    }

    #[test]
    fn _nodeagg_sub_expr() {
        assert!(nodeagg_sub_expr().parse("").is_err());
        assert_eq!(
            nodeagg_sub_expr().parse("node"),
            Ok((NodeaggSubExpr::NodeaggSub("node".to_string()), ""))
        );
        assert_eq!(
            nodeagg_sub_expr().parse("123"),
            Ok((NodeaggSubExpr::NodeaggSub("123".to_string()), ""))
        );
        assert!(nodeagg_sub_expr().parse("[]").is_err());
        assert!(nodeagg_sub_expr().parse("[1]").is_ok());
        assert!(nodeagg_sub_expr().parse("[01,02]").is_ok());
        assert!(nodeagg_sub_expr().parse("[01-02]").is_ok());

        assert_eq!(
            nodeagg_sub_expr()
                .parse("[1,2,3]")
                .unwrap()
                .0
                .iter()
                .collect::<String>()
                .len(),
            3
        );
    }

    #[test]
    fn _nodeagg_base_expr() {
        assert_eq!(nodeagg_base_expr().parse(""), Ok((vec![].into(), "")));
        assert_eq!(
            nodeagg_base_expr().parse("123"),
            Ok((
                vec![NodeaggSubExpr::NodeaggSub("123".to_string())].into(),
                ""
            ))
        );
        assert_eq!(
            nodeagg_base_expr().parse("node  "),
            Ok((
                vec![NodeaggSubExpr::NodeaggSub("node".to_string())].into(),
                ""
            ))
        );
        assert!(nodeagg_base_expr().parse("[]").is_err());
        assert!(nodeagg_base_expr().parse("[1]").is_ok());
        assert!(nodeagg_base_expr().parse("node1,node2").is_ok());
        assert!(nodeagg_base_expr().parse("[1,2]").is_ok());
        assert!(nodeagg_base_expr().parse("[1-2]").is_ok());
        assert!(nodeagg_base_expr().parse("anode[01-02]").is_ok());
    }

    #[test]
    fn _nodeagg_expr() {
        assert_eq!(
            nodeagg_expr().easy_parse("node1,node2"),
            Ok((
                vec![
                    nodeagg_base_expr().parse("node1").unwrap().0,
                    nodeagg_base_expr().parse("node2").unwrap().0
                ]
                .into(),
                ""
            ))
        );

        assert_eq!(
            nodeagg_expr()
                .parse("anode[01-02],bnode2")
                .unwrap()
                .0
                .iter()
                .collect::<Vec<_>>(),
            vec!["anode01", "anode02", "bnode2"]
        );
        assert!(nodeagg_expr().parse("node1, node2").is_ok());
        assert!(nodeagg_expr().parse(" node1,node2").is_ok());
        assert!(nodeagg_expr().parse("node1,node2 ").is_ok());
        assert!(nodeagg_expr().parse("node1\nnode2").is_ok());
        assert!(nodeagg_expr().parse("node1 a").is_err());
    }
}
