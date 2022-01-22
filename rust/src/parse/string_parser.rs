pub fn parse_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut escaped = false;
    for c in input.chars() {
        if !escaped {
            match c {
                '\\' => escaped = true,
                _ => out += &c.to_string(),
            };
        } else {
            escaped = false;
            match c {
                '\\' => out +="\\",
                'n' => out += "\n",
                _ => unimplemented!(),
            };
        }
    }
    out
}

