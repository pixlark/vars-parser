#![allow(unused_variables)]
#![allow(dead_code)]

use std::str::Chars;
use std::iter::Peekable;
use std::collections::HashMap;

/// Useful wrapper for Peekable<Chars> which returns an EOF char
/// when the iterator is empty
struct Stream<'a> {
	stream: &'a mut Peekable<Chars<'a>>,
}

impl<'a> Stream<'a> {
	fn peek(&mut self) -> char
	{
		self.stream.peek().map(|c| *c).unwrap_or('\0')
	}

	fn next(&mut self) -> char
	{
		self.stream.next().unwrap_or('\0')
	}
}

#[derive(Debug)]
enum Token {
	EOF,
	Assignment,
	Name(String),
	String(String),
	Integer(i64),
	Float(f64),
}

impl From<Number> for Token {
	fn from(num: Number) -> Token {
		match num {
			Number::Integer(i) =>
				Token::Integer(i),
			Number::Float(f) =>
				Token::Float(f),
		}
	}
}

/// Pull from stream into buffer until name is terminated or EOF
/// reached
fn scan_name(stream: &mut Stream) -> String
{
	let mut string = String::new();
	while !stream.peek().is_whitespace() && stream.peek() != ':' && stream.peek() != '\0' {
		string.push(stream.next());
	}

	string
}

/// Pull from stream into buffer until string is terminated or EOF
/// reached
fn scan_string(stream: &mut Stream) -> String
{
	let mut string = String::new();
	while stream.peek() != '\0' && stream.peek() != '"' {
		string.push(stream.next());
	}

	string
}

#[derive(Debug)]
enum Number {
	Integer(i64),
	Float(f64),
}

/// Read int/float from stream. Returns Number::NotNumber if
/// scanning fails.
fn scan_number(stream: &mut Stream) -> Option<Number>
{
	let mut buffer = String::new();
	let mut fractional = false;

	if stream.peek() == '-' || stream.peek() == '+' {
		buffer.push(stream.next());
	}

	while stream.peek().is_numeric() || stream.peek() == '.' {
		if stream.peek() == '.' { fractional = true; }
		buffer.push(stream.next());
	}

	let number =
		if fractional {
			buffer.parse::<f64>().map(Number::Float).ok()
		} else {
			buffer.parse::<i64>().map(Number::Integer).ok()
		};

	number
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LexerError {
	InvalidLiteral,
	ExpectedEquals,
	UnterminatedString,
	UnrecognizedChar,
}

/// Central part of lexer. Advances stream by arbitrary amount
/// until the next token is lexed.
fn next_token(stream: &mut Stream) -> Result<Token, LexerError>
{
	let c = stream.peek();
	if c.is_whitespace() {
		stream.next();
		return next_token(stream);
	}
	if c.is_alphabetic() || c == '_' {
		return Ok(Token::Name(scan_name(stream)));
	}
	if c.is_numeric() || c == '.' || c == '-' || c == '+' {
		return scan_number(stream)
			.map(|n| n.into())
			.ok_or(LexerError::InvalidLiteral)
	}
	match c {
		'#' => {
			stream.next();
			let mut n = stream.next();
			while n != '\n' && n != '\0' {
				n = stream.next();
			}
			next_token(stream)
		}
		':' => {
			stream.next();
			if stream.next() == '=' {
				Ok(Token::Assignment)
			} else {
				Err(LexerError::ExpectedEquals)
			}
		},
		'"' => {
			stream.next();
			let s = Token::String(scan_string(stream));
			if stream.next() == '"' {
				Ok(s)
			} else {
				Err(LexerError::UnterminatedString)
			}
		},
		'\0' => {
			Ok(Token::EOF)
		},
		_ => {
			Err(LexerError::UnrecognizedChar)
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
	String(String),
	Integer(i64),
	Float(f64),
}

#[derive(Debug)]
struct Declaration {
	name: String,
	value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParseError {
	ExpectedName,
	ExpectedEquals,
	ExpectedLiteral,
	LexerError(LexerError),
}

impl From<LexerError> for ParseError {
	fn from(error: LexerError) -> ParseError {
		ParseError::LexerError(error)
	}
}

/// Variation on Result which can specify EOF reached to terminate
/// main loop
#[derive(Debug)]
enum ParseResult {
	Ok(Declaration),
	Err(ParseError),
	EOF,
}

/// Reads arbitrary amount of tokens from next_token() until a new
/// declaration is found
fn parse_declaration(stream: &mut Stream) -> ParseResult
{
	let mut decl = Declaration { name: "".to_string(), value: Value::Integer(0) };

	match next_token(stream) {
		Ok(ok) => {
			match ok {
				Token::Name(s) => decl.name = s,
				Token::EOF => return ParseResult::EOF,
				_ => return ParseResult::Err(ParseError::ExpectedName)
			}
		},
		Err(e) => return ParseResult::Err(e.into())
	}

	match next_token(stream) {
		Ok(ok) => {
			match ok {
				Token::Assignment => (),
				_ => return ParseResult::Err(ParseError::ExpectedEquals)
			}
		},
		Err(e) => return ParseResult::Err(e.into())
	}

	match next_token(stream) {
		Ok(ok) => {
			match ok {
				Token::String(s) => {
					decl.value = Value::String(s);
				},
				Token::Integer(n) => {
					decl.value = Value::Integer(n);
				},
				Token::Float(f) => {
					decl.value = Value::Float(f);
				},
				_ => return ParseResult::Err(ParseError::ExpectedLiteral)
			}
		},
		Err(e) => return ParseResult::Err(e.into())
	}

	ParseResult::Ok(decl)
}

/// Reads as many declarations from a source string as it can and
/// stores them in a HashMap
pub fn parse_vars(source: String) -> Result<HashMap<String, Value>, ParseError>
{
	let mut stream = Stream { stream: &mut source.chars().peekable() };
	let mut decls = HashMap::new();

	loop {
		let result = parse_declaration(&mut stream);
		match result {
			ParseResult::Ok(ok) => decls.insert(ok.name, ok.value),
			ParseResult::EOF => break,
			ParseResult::Err(e) => return Err(e)
		};
	}

	Ok(decls)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parsing()
	{
		let source: String = "
		# Comment
		variable_str   := \"string literal\"
		variable_int   := -15
		variable_float := 105.3".to_string();

		let vars = parse_vars(source).unwrap();

		let key = "variable_str".to_string();
		let val = vars.get(&key).unwrap();
		assert_eq!(&Value::String("string literal".into()), val);

		let key = "variable_int".to_string();
		let val = vars.get(&key).unwrap();
		assert_eq!(&Value::Integer(-15), val);

		let key = "variable_float".to_string();
		let val = vars.get(&key).unwrap();
		assert_eq!(&Value::Float(105.3), val);
	}
}
