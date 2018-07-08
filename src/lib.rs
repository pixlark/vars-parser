#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]

#[cfg(test)]
mod vars_parser {
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
			let c: Option<&char> = self.stream.peek();
			match c {
				Some(c) => *c,
				None => '\0'
			}
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
		String_Literal(String),
		Int_Literal(i64),
		Float_Literal(f64),
	}

	/// Pull from stream into buffer until name is terminated or EOF
	/// reached
	fn scan_name(stream: &mut Stream) -> String
	{
		let mut string = String::new();
		while !stream.peek().is_whitespace() && stream.peek() != ':' && stream.peek() != '\0' {
			string.push(stream.next());
		}
		return string;
	}

	/// Pull from stream into buffer until string is terminated or EOF
	/// reached
	fn scan_string(stream: &mut Stream) -> String
	{
		let mut string = String::new();
		while stream.peek() != '\0' && stream.peek() != '"' {
			string.push(stream.next());
		}
		return string;
	}

	#[derive(Debug)]
	enum Number {
		Integer(i64),
		Float(f64),
		Not_A_Number,
	}

	/// Read int/float from stream. Returns Number::Not_A_Number if
	/// scanning fails.
	fn scan_number(stream: &mut Stream) -> Number
	{
		let mut buffer = String::new();
		let mut fractional: bool = false;
		if stream.peek() == '-' || stream.peek() == '+' {
			buffer.push(stream.next());
		}
		while stream.peek().is_numeric() || stream.peek() == '.' {
			if stream.peek() == '.' { fractional = true; }
			buffer.push(stream.next());
		}
		if fractional {
			let result = buffer.parse::<f64>();
			match result {
				Ok(ok) => Number::Float(ok),
				Err(_) => Number::Not_A_Number
			}
		} else {
			let result = buffer.parse::<i64>();
			match result {
				Ok(ok) => Number::Integer(ok),
				Err(_) => Number::Not_A_Number
			}
		}
	}

	/// Central part of lexer. Advances stream by arbitrary amount
	/// until the next token is lexed.
	fn next_token(stream: &mut Stream) -> Result<Token, String>
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
			let num = scan_number(stream);
			return match num {
				Number::Integer(n) => Ok(Token::Int_Literal(n)),
				Number::Float(f) => Ok(Token::Float_Literal(f)),
				Number::Not_A_Number => Err("Unable to parse literal".to_string())
			}
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
					Err("Expected = after :".to_string())
				}
			},
			'"' => {
				stream.next();
				let s = Token::String_Literal(scan_string(stream));
				if stream.next() == '"' {
					Ok(s)
				} else {
					Err("String literal unterminated".to_string())
				}
			},
			'\0' => {
				Ok(Token::EOF)
			},
			_ => {
				Err("Unrecognized char".to_string())
			}
		}
	}

	#[derive(Debug)]
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

	/// Variation on Result which can specify EOF reached to terminate
	/// main loop
	#[derive(Debug)]
	enum Parse_Result {
		Ok(Declaration),
		Err(String),
		EOF,
	}

	/// Reads arbitrary amount of tokens from next_token() until a new
	/// declaration is found
	fn parse_declaration(stream: &mut Stream) -> Parse_Result
	{
		let mut decl = Declaration { name: "".to_string(), value: Value::Integer(0) };
		{
			let token = next_token(stream);
			match token {
				Ok(ok) => {
					match ok {
						Token::Name(s) => decl.name = s,
						Token::EOF => return Parse_Result::EOF,
						_ => return Parse_Result::Err("Expected name at beginning of declaration".to_string())
					}
				},
				Err(e) => return Parse_Result::Err(e)
			}
		}
		{
			let token = next_token(stream);
			match token {
				Ok(ok) => {
					match ok {
						Token::Assignment => (),
						_ => return Parse_Result::Err("Expected := after name in declaration".to_string())
					}
				},
				Err(e) => return Parse_Result::Err(e)
			}
		}
		{
			let token = next_token(stream);
			match token {
				Ok(ok) => {
					match ok {
						Token::String_Literal(s) => {
							decl.value = Value::String(s);
						},
						Token::Int_Literal(n) => {
							decl.value = Value::Integer(n);
						},
						Token::Float_Literal(f) => {
							decl.value = Value::Float(f);
						},
						_ => return Parse_Result::Err("Expected literal at end of declaration".to_string())
					}
				},
				Err(e) => return Parse_Result::Err(e)
			}	
		}
		Parse_Result::Ok(decl)
	}

	/// Reads as many declarations from a source string as it can and
	/// stores them in a HashMap
	pub fn parse_vars(source: String) -> Result<HashMap<String, Value>, String>
	{
		let mut stream = Stream { stream: &mut source.chars().peekable() };
		let mut decls: HashMap<String, Value> = HashMap::new();
		loop {
			let result = parse_declaration(&mut stream);
			match result {
				Parse_Result::Ok(ok) => decls.insert(ok.name, ok.value),
				Parse_Result::EOF => break,
				Parse_Result::Err(e) => return Err(e)
			};
		}
		Ok(decls)
	}

	#[test]
	fn test_parsing()
	{
		let source: String = "
		# Comment
		variable_str   := \"string literal\"
		variable_int   := -15
		variable_float := 105.3".to_string();
		let vars = match parse_vars(source) {
			Ok(ok) => ok,
			Err(e) => panic!(e)
		};
		{
			let key: String = "variable_str".to_string();
			match vars.get(&key) {
				Some(val) => {
					match val {
						Value::String(s) => {
							assert_eq!(s, "string literal");
						},
						_ => panic!("String literal didn't parse correctly")
					}
				},
				None => panic!("Name didn't get parsed correctly")
			}
		}
		{
			let key: String = "variable_int".to_string();
			match vars.get(&key) {
				Some(val) => {
					match val {
						Value::Integer(n) => {
							assert_eq!(*n, -15);
						},
						_ => panic!("Integer literal didn't parse correctly")
					}
				},
				None => panic!("Name didn't get parsed correctly")
			}
		}
		{
			let key: String = "variable_float".to_string();
			match vars.get(&key) {
				Some(val) => {
					match val {
						Value::Float(f) => {
							assert_eq!(*f, 105.3);
						},
						_ => panic!("Float literal didn't parse correctly")
					}
				},
				None => panic!("Name didn't get parsed correctly")
			}
		}
	}
}
