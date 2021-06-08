/*
ISC License (ISC)

Copyright (c) 2016, Zeyla Hellyer <hi@zeyla.me>

Permission to use, copy, modify, and/or distribute this software for any purpose
with or without fee is hereby granted, provided that the above copyright notice
and this permission notice appear in all copies.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND
FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS
OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER
TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF
THIS SOFTWARE.
*/

//Trimmed and lightly modified version of Args from the Serenity v0.5.11 StandardFramework


use std::{
    str::FromStr,
    error::Error as StdError,
    fmt
};

#[derive(Debug)]
pub enum Error<E: StdError> {
    Eos,
    Parse(E),
}

impl<E: StdError> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Error::Parse(e)
    }
}

impl<E: StdError> StdError for Error<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        use self::Error::*;

        match *self {
            Parse(ref e) => e.source(),
            _ => None,
        }
    }
}

impl<E: StdError> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match *self {
            Eos => write!(f, "end of string"),
            Parse(ref e) => fmt::Display::fmt(&e, f),
        }
    }
}

type Result<T, E> = std::result::Result<T, Error<E>>;

#[derive(Clone, Copy, Debug, PartialEq)]
enum TokenKind {
    Delimiter,
    Argument,
    QuotedArgument,
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    lit: String,
    pos: usize,
}

impl Token {
    fn new(kind: TokenKind, lit: &str, pos: usize) -> Self {
        Self {
            kind,
            lit: lit.to_string(),
            pos
        }
    }
}

fn find_end(s: &str, i: usize) -> Option<usize> {
    if i > s.len() {
        return None;
    }

    let mut end = i + 1;

    while !s.is_char_boundary(end) {
        end += 1;
    }

    Some(end)
}

#[derive(Debug)]
struct Lexer<'a> {
    msg: &'a str,
    delims: &'a [char],
    offset: usize,
}

impl<'a> Lexer<'a> {
    fn new(msg: &'a str, delims: &'a [char]) -> Self {
        Self {
            msg,
            delims,
            offset: 0,
        }
    }

    #[inline]
    fn at_end(&self) -> bool {
        self.offset >= self.msg.len()
    }

    fn current(&self) -> Option<&str> {
        if self.at_end() {
            return None;
        }

        let start = self.offset;

        let end = find_end(&self.msg, self.offset)?;

        Some(&self.msg[start..end])
    }

    fn next(&mut self) -> Option<()> {
        self.offset += self.current()?.len();

        Some(())
    }

    fn commit(&mut self) -> Option<Token> {
        if self.at_end() {
            return None;
        }

        if self.current()?.contains(self.delims) {
            let start = self.offset;
            self.next();
            return Some(Token::new(TokenKind::Delimiter, &self.msg[start..self.offset], start))
        }

        if self.current()? == "\"" {
            let start = self.offset;
            self.next();

            while !self.at_end() && self.current()? != "\"" {
                self.next();
            }

            let is_quote = self.current().map_or(false, |s | s == "\"");
            self.next();

            let end = self.offset;

            return Some(if is_quote {
                Token::new(TokenKind::QuotedArgument, &self.msg[start..end], start)
            } else {
                Token::new(TokenKind::Argument, &self.msg[start..], start)
            });
        }

        let start = self.offset;

        while !self.at_end() {
            if self.current()?.contains(self.delims) {
                break;
            }

            self.next();
        }

        Some(Token::new(TokenKind::Argument, &self.msg[start..self.offset], start))
    }
}

#[derive(Clone, Debug)]
pub struct Args {
    message: String,
    args: Vec<Token>,
    offset: usize,
}

impl Args {
    pub fn new(message: &str, possible_delimiters: &[String]) -> Self {
        let delims = possible_delimiters
            .iter()
            .filter(|d| message.contains(d.as_str()))
            .flat_map(|s| s.chars())
            .collect::<Vec<_>>();

        let mut args = Vec::new();

        if delims.is_empty() && !message.is_empty() {
            args.push(Token::new(TokenKind::Argument, &message[..], 0));
        } else {
            let mut lex = Lexer::new(message, &delims);

            while let Some(token) = lex.commit() {
                if token.kind == TokenKind::Delimiter {
                    continue;
                }

                args.push(token);
            }
        }

        Self {
            args,
            message: message.to_string(),
            offset: 0,
        }
    }

    pub fn current(&self) -> Option<&str> {
        self.args.get(self.offset).map(|t| t.lit.as_str())
    }

    pub fn trim(&mut self) -> &mut Self {
        if self.is_empty() {
            return self;
        }

        self.args[self.offset].lit = self.args[self.offset].lit.trim().to_string();

        self
    }

    pub fn trim_all(&mut self) {
        if self.is_empty() {
            return;
        }

        for token in &mut self.args[self.offset..] {
            token.lit = token.lit.trim().to_string();
        }
    }

    pub fn single<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let cur = &self.args[self.offset];

        let parsed = T::from_str(&cur.lit)?;
        self.offset += 1;
        Ok(parsed)
    }

    pub fn single_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let cur = &self.args[self.offset];

        Ok(T::from_str(&cur.lit)?)
    }

    pub fn skip(&mut self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        self.single::<String>().ok()
    }

    pub fn skip_for(&mut self, i: u32) -> Option<Vec<String>> {
        if self.is_empty() {
            return None;
        }

        let mut vec = Vec::with_capacity(i as usize);

        for _ in 0..i {
            vec.push(self.skip()?);
        }

        Some(vec)
    }

    pub fn iter<T: FromStr>(&mut self) -> Iter<T>
        where T::Err: StdError {
        Iter::new(self)
    }

    pub fn multiple<T: FromStr>(mut self) -> Result<Vec<T>, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        self.iter::<T>().collect()
    }

    pub fn current_quoted(&self) -> Option<&str> {
        self.args.get(self.offset).map(|t| quotes_extract(t))
    }

    pub fn single_quoted<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let current = &self.args[self.offset];

        // Discard quotations if present
        let lit = quotes_extract(current);

        let parsed = T::from_str(&lit)?;
        self.offset += 1;
        Ok(parsed)
    }

    pub fn single_quoted_n<T: FromStr>(&self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let current = &self.args[self.offset];

        let lit = quotes_extract(current);

        Ok(T::from_str(&lit)?)
    }

    pub fn iter_quoted<T: FromStr>(&mut self) -> IterQuoted<T>
        where T::Err: StdError {
        IterQuoted::new(self)
    }

    pub fn multiple_quoted<T: FromStr>(mut self) -> Result<Vec<T>, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        self.iter_quoted::<T>().collect()
    }

    pub fn find<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let pos = match self.args.iter().map(|t| quotes_extract(t)).position(|s| s.parse::<T>().is_ok()) {
            Some(p) => p,
            None => return Err(Error::Eos),
        };

        let parsed = T::from_str(quotes_extract(&self.args[pos]))?;
        self.args.remove(pos);
        self.rewind();

        Ok(parsed)
    }

    pub fn find_n<T: FromStr>(&mut self) -> Result<T, T::Err>
        where T::Err: StdError {
        if self.is_empty() {
            return Err(Error::Eos);
        }

        let pos = match self.args.iter().map(|t| quotes_extract(t)).position(|s| s.parse::<T>().is_ok()) {
            Some(p) => p,
            None => return Err(Error::Eos),
        };

        Ok(T::from_str(quotes_extract(&self.args[pos]))?)
    }

    pub fn full(&self) -> &str {
        &self.message
    }

    pub fn full_quoted(&self) -> &str {
        let s = &self.message;

        if !s.starts_with('"') {
            return s;
        }

        let end = s.rfind('"').unwrap();

        // If it got the quote at the start, then there's no closing quote.
        if end == 0 {
            return s;
        }

        &s[1..end]
    }

    pub fn rest(&self) -> &str {
        if self.is_empty() {
            return "";
        }

        let args = &self.args[self.offset..];

        if let Some(token) = args.get(0) {
            &self.message[token.pos..]
        } else {
            &self.message[..]
        }
    }

    pub fn len(&self) -> usize {
        self.args.len()
    }

    pub fn is_empty(&self) -> bool {
        self.offset >= self.args.len()
    }

    pub fn remaining(&self) -> usize {
        if self.is_empty() {
            return 0;
        }

        self.len() - self.offset
    }

    #[inline]
    pub fn next(&mut self) {
        self.offset += 1;
    }

    #[inline]
    pub fn rewind(&mut self) {
        if self.offset == 0 {
            return;
        }

        self.offset -= 1;
    }

    #[inline]
    pub fn restore(&mut self) {
        self.offset = 0;
    }
}

impl ::std::ops::Deref for Args {
    type Target = str;

    fn deref(&self) -> &Self::Target { self.full() }
}

impl PartialEq<str> for Args {
    fn eq(&self, other: &str) -> bool {
        self.message == other
    }
}

impl<'a> PartialEq<&'a str> for Args {
    fn eq(&self, other: &&'a str) -> bool {
        self.message == *other
    }
}

impl PartialEq for Args {
    fn eq(&self, other: &Self) -> bool {
        self.message == *other.message
    }
}

impl Eq for Args {}

use std::marker::PhantomData;

pub struct Iter<'a, T: FromStr> where T::Err: StdError {
    args: &'a mut Args,
    _marker: PhantomData<T>,
}

impl<'a, T: FromStr> Iter<'a, T> where T::Err: StdError {
    fn new(args: &'a mut Args) -> Self {
        Iter { args, _marker: PhantomData }
    }
}

impl<'a, T: FromStr> Iterator for Iter<'a, T> where T::Err: StdError  {
    type Item = Result<T, T::Err>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.args.is_empty() {
            None
        } else {
            Some(self.args.single::<T>())
        }
    }
}

pub struct IterQuoted<'a, T: FromStr> where T::Err: StdError {
    args: &'a mut Args,
    _marker: PhantomData<T>,
}

impl<'a, T: FromStr> IterQuoted<'a, T> where T::Err: StdError {
    fn new(args: &'a mut Args) -> Self {
        IterQuoted { args, _marker: PhantomData }
    }
}

impl<'a, T: FromStr> Iterator for IterQuoted<'a, T> where T::Err: StdError  {
    type Item = Result<T, T::Err>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.args.is_empty() {
            None
        } else {
            Some(self.args.single_quoted::<T>())
        }
    }
}

fn quotes_extract(token: &Token) -> &str {
    if token.kind == TokenKind::QuotedArgument {
        &token.lit[1..token.lit.len() - 1]
    } else {
        &token.lit
    }
}