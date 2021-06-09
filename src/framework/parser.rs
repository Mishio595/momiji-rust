use super::args::Args;
use peg;

peg::parser! { grammar command_parser() for str {
    pub rule command(del: &[String]) -> (String, Args)
        = comm:$(['a'..='z' | 'A'..='Z']+) [del]? args:($([_]+) ** [del]) {
            // TODO add Parse, ParseSlice, ParseElem, and ParseLiteral implementations for Args
            let args = Args::new(args.join(del[0].as_str()).as_str(), del);

            (comm.to_string(), args)
        }
}}

#[derive(Clone, Debug)]
pub struct Parser;

impl Parser {
    pub fn parse(&self, body: &str, possible_delimiters: &[String]) -> Option<(String, Args)> {
        command_parser::command(body, possible_delimiters).ok()
    }

    pub fn parse_with_prefix<'a>(&self, prefix: &'a str, body: &'a str, possible_delimiters: &[String]) -> Option<(String, Args)> {
        body.strip_prefix(prefix).and_then(|body| self.parse(body, possible_delimiters))
    }
}