## vars-parser
Parse variables files in Rust

### Usage

```
let vars = match parse_vars_file(source) {
	Ok(ok) => ok,
	Err(e) => panic!(e) // Err(e) contains error message as String
}
```

### Example vars file

```
# Comments start with '#'
variable_string := "string literal" # Represented as String
variable_integer := -1500           # Represented as i64
variable_float := 3.14159           # Represented as f64
```
