#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

/// Useful wrapper for Peekable<Chars> which returns an EOF char
/// when the iterator is empty
struct Stream<'a> {
    stream: &'a mut Peekable<Chars<'a>>,
}

impl<'a> Stream<'a> {
    fn peek(&mut self) -> char {
        let c: Option<&char> = self.stream.peek();
        match c {
            Some(c) => *c,
            None => '\0',
        }
    }
    fn next(&mut self) -> char {
        self.stream.next().unwrap_or('\0')
    }
}

#[derive(Debug)]
enum Token {
    EOF,
    Assignment,
    Name(String),
    StringLiteral(String),
    IntLiteral(i64),
    FloatLiteral(f64),
}

/// Pull from stream into buffer until name is terminated or EOF
/// reached
fn scan_name(stream: &mut Stream) -> String {
    let mut string = String::new();
    while !stream.peek().is_whitespace() && stream.peek() != ':' && stream.peek() != '\0' {
        string.push(stream.next());
    }
    return string;
}

/// Pull from stream into buffer until string is terminated or EOF
/// reached
fn scan_string(stream: &mut Stream) -> String {
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
    NotANumber,
}

/// Read int/float from stream. Returns Number::NotANumber if
/// scanning fails.
fn scan_number(stream: &mut Stream) -> Number {
    let mut buffer = String::new();
    let mut fractional: bool = false;
    if stream.peek() == '-' || stream.peek() == '+' {
        buffer.push(stream.next());
    }
    while stream.peek().is_numeric() || stream.peek() == '.' {
        if stream.peek() == '.' {
            fractional = true;
        }
        buffer.push(stream.next());
    }
    if fractional {
        let result = buffer.parse::<f64>();
        match result {
            Ok(ok) => Number::Float(ok),
            Err(_) => Number::NotANumber,
        }
    } else {
        let result = buffer.parse::<i64>();
        match result {
            Ok(ok) => Number::Integer(ok),
            Err(_) => Number::NotANumber,
        }
    }
}

/// Central part of lexer. Advances stream by arbitrary amount
/// until the next token is lexed.
fn next_token(stream: &mut Stream) -> Result<Token, String> {
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
            Number::Integer(n) => Ok(Token::IntLiteral(n)),
            Number::Float(f) => Ok(Token::FloatLiteral(f)),
            Number::NotANumber => Err("Unable to parse literal".to_string()),
        };
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
        }
        '"' => {
            stream.next();
            let s = Token::StringLiteral(scan_string(stream));
            if stream.next() == '"' {
                Ok(s)
            } else {
                Err("String literal unterminated".to_string())
            }
        }
        '\0' => Ok(Token::EOF),
        _ => Err("Unrecognized char".to_string()),
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
enum ParseResult {
    Ok(Declaration),
    Err(String),
    EOF,
}

/// Reads arbitrary amount of tokens from next_token() until a new
/// declaration is found
fn parse_declaration(stream: &mut Stream) -> ParseResult {
    let mut decl = Declaration {
        name: "".to_string(),
        value: Value::Integer(0),
    };
    {
        let token = next_token(stream);
        match token {
            Ok(ok) => match ok {
                Token::Name(s) => decl.name = s,
                Token::EOF => return ParseResult::EOF,
                _ => {
                    return ParseResult::Err("Expected name at beginning of declaration".to_string())
                }
            },
            Err(e) => return ParseResult::Err(e),
        }
    }
    {
        let token = next_token(stream);
        match token {
            Ok(ok) => match ok {
                Token::Assignment => (),
                _ => return ParseResult::Err("Expected := after name in declaration".to_string()),
            },
            Err(e) => return ParseResult::Err(e),
        }
    }
    {
        let token = next_token(stream);
        match token {
            Ok(ok) => match ok {
                Token::StringLiteral(s) => {
                    decl.value = Value::String(s);
                }
                Token::IntLiteral(n) => {
                    decl.value = Value::Integer(n);
                }
                Token::FloatLiteral(f) => {
                    decl.value = Value::Float(f);
                }
                _ => return ParseResult::Err("Expected literal at end of declaration".to_string()),
            },
            Err(e) => return ParseResult::Err(e),
        }
    }
    ParseResult::Ok(decl)
}

/// Reads as many declarations from a source string as it can and
/// stores them in a HashMap
pub fn parse_vars(source: &str) -> Result<HashMap<String, Value>, String> {
    let mut stream = Stream {
        stream: &mut source.chars().peekable(),
    };
    let mut decls: HashMap<String, Value> = HashMap::new();
    loop {
        let result = parse_declaration(&mut stream);
        match result {
            ParseResult::Ok(ok) => decls.insert(ok.name, ok.value),
            ParseResult::EOF => break,
            ParseResult::Err(e) => return Err(e),
        };
    }
    Ok(decls)
}

#[test]
fn test_parsing() {
    let source = "
		# Comment
		variable_str   := \"string literal\"
		variable_int   := -15
		variable_float := 105.3";
    let vars = match parse_vars(source) {
        Ok(ok) => ok,
        Err(e) => panic!(e),
    };
    {
        let key: String = "variable_str".to_string();
        match vars.get(&key).expect("Name didn't get parsed correctly") {
            Value::String(s) => assert_eq!(s, "string literal"),
            _ => panic!("String literal didn't parse correctly"),
        }
    }
    {
        let key: String = "variable_int".to_string();
        match vars.get(&key).expect("Name didn't get parsed correctly") {
            Value::Integer(n) => assert_eq!(*n, -15),
            _ => panic!("Integer literal didn't parse correctly"),
        }
    }
    {
        let key: String = "variable_float".to_string();
        match vars.get(&key).expect("Name didn't get parsed correctly") {
            Value::Float(f) => assert_eq!(*f, 105.3),
            _ => panic!("Float literal didnt' parse correctly"),
        }
    }
}
