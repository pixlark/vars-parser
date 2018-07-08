## vars-parser
Parse variables files in Rust

### Usage

```
let vars = match parse_vars(source) {
	Ok(ok) => ok,
	Err(e) => panic!(e) // Err(e) contains error message as String
}
```

parse_vars_file returns a `HashMap<String, Value>`, where Value is an enum with three possible values:
* `Value::String(String)`
* `Value::Integer(i64)`
* `Value::Float(f64)`

### Example vars file

```
# Comments start with '#'
variable_string := "string literal"
variable_integer := -1500
variable_float := 3.14159
```
