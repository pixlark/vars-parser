## vars-parser
Parse variables files in rust

### Usage

`let vars: HashMap<String, vars_parser::Value> = parse_vars_parser(source);`

### Vars files

Example file:
```
# Comments start with '#'
variable_string := "string literal" # Represented as String()
variable_integer := -1500           # Represented as i64
variable_float := 3.14159           # Represented as f64
```
