use combine::parser::char::newline;
use combine::parser::repeat::skip_until;
use combine::{eof, satisfy, skip_count, skip_many, token, Stream};

parser! {
    pub fn tabspace[Input]()(Input) -> char
        where [Input: Stream<Token = char>]
{
    satisfy(|ch: char| ch == '\t' || ch == ' ')
}}

parser! {
    pub fn tabspaces[Input]()(Input) -> ()
        where [Input: Stream<Token = char>]
{
    skip_many(tabspace())
}}

parser! {
pub fn comment[Input]()(Input) -> ()
    where [Input: Stream<Token = char>]
{
    skip_count(1, token('#').and(skip_until(
                eof().or(newline().map(|_|()))
                ))).map(|_|())
}}

#[cfg(test)]
mod tests {
    use super::*;
    use combine::Parser;

    #[test]
    fn test_comment() {
        assert_eq!(comment().parse(""), Ok(((), "")));
        assert_eq!(comment().parse("#comment"), Ok(((), "")));
        assert_eq!(comment().parse("#comment\n"), Ok(((), "\n")));
    }
}
