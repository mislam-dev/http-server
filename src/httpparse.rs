use std::collections::HashMap;

pub struct HttpParse;

impl HttpParse {
    pub fn parse(raw: &str) -> HashMap<&str, &str> {
        let mut headers: HashMap<&str, &str> = HashMap::new();
        if raw.len() <= 0 {
            return headers;
        }
        let mut values_iter = raw.split("\r\n").map(|r| r).into_iter();
        let first_line: Vec<&str> = values_iter.next().unwrap().split(" ").map(|r| r).collect();
        if first_line.len() >= 2 {
            headers.insert("method", first_line[0]);
            headers.insert("path", first_line[1]);
        }

        while let Some(value) = values_iter.next() {
            if value == "" {
                continue;
            }
            let items: Vec<&str> = value.split(": ").collect();
            if items.len() >= 2 {
                headers.insert(items[0], items[1]);
            }
        }
        headers
    }
}
