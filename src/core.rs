use crate::expr::{nodeagg_expr, NodeaggExpr};
use crate::iter::NodeaggIterator;
use combine::stream::easy::ParseError;
use combine::stream::PointerOffset;
use combine::EasyParser;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseNodeaggError(String);

impl fmt::Display for ParseNodeaggError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ParseNodeaggError {
    /// Not supported. Alwyas return None
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug)]
pub struct Nodeagg(NodeaggExpr);

impl Nodeagg {
    pub fn iter(&self) -> NodeaggIterator<'_> {
        self.0.iter()
    }
}

impl std::str::FromStr for Nodeagg {
    type Err = ParseNodeaggError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Nodeagg::try_from(s) {
            Ok(x) => Ok(x),
            Err(e) => Err(ParseNodeaggError(format!("{:?}", e))),
        }
    }
}

impl<'a> TryFrom<&'a str> for Nodeagg {
    type Error = ParseError<&'a str>;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        match nodeagg_expr().easy_parse(s) {
            Ok((n, "")) => Ok(Nodeagg(n)),
            Ok((_, input)) => Err(Self::Error {
                position: PointerOffset::new(input.as_ptr() as usize),
                errors: vec![combine::stream::easy::Error::Message(
                    combine::stream::easy::Info::Owned(format!(
                        "string `{}' still exists after parse",
                        input
                    )),
                )],
            }),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str() {
        assert_eq!(
            Nodeagg::try_from("node1,,node2 ")
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            ["node1", "node2"]
        );

        assert_eq!(
            Nodeagg::try_from("node1,,node2\n\n")
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            ["node1", "node2"]
        );

        assert_eq!(
            Nodeagg::try_from("node1\nnode2")
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            ["node1", "node2"]
        );

        assert_eq!(
            Nodeagg::try_from("node1\n\nnode2")
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            ["node1", "node2"]
        );

        assert_eq!(
            Nodeagg::try_from("node1\n\nnode2\n#comment end")
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            ["node1", "node2"]
        );

        assert_eq!(
            Nodeagg::try_from("node1\nnode2 #comment\nnode3\n#comment only line\nnode4")
                .unwrap()
                .iter()
                .collect::<Vec<_>>(),
            ["node1", "node2", "node3", "node4"]
        );

        fn f1() -> Nodeagg {
            let a: String = "node[1-2]".to_string();
            Nodeagg::try_from(&a[..]).unwrap()
        }
        assert_eq!(f1().iter().collect::<Vec<_>>(), vec!["node1", "node2"]);

        fn f2(s: &str) -> Nodeagg {
            Nodeagg::try_from(s).unwrap()
        }

        assert_eq!(
            f2("node[01,2]").iter().collect::<Vec<_>>(),
            vec!["node01", "node02"]
        );
    }
}
