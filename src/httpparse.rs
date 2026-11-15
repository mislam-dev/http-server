use std::collections::HashMap;

#[derive(Debug)]
pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub headers: HashMap<&'a str, &'a str>,
    pub body: &'a str,
}

pub struct HttpParse;

impl HttpParse {
    pub fn parse<'a>(raw: &'a str) -> Request<'a> {
        let sp: Vec<&str> = raw.split("\r\n\r\n").collect();

        let ((method, path), headers) = Self::parse_headers(sp[0]);

        let req: Request<'a> = Request {
            method,
            path,
            headers,
            body: sp[1],
        };

        req
    }
    pub fn parse_body(raw: &str) -> HashMap<&str, &str> {
        println!("{raw:#?}");
        let mut body: HashMap<&str, &str> = HashMap::new();
        if raw.len() <= 0 {
            return body;
        }

        let mut values_iter = raw.split("\r\n").map(|r| r).into_iter();

        while let Some(value) = values_iter.next() {
            if value == "" {
                continue;
            }
            let items: Vec<&str> = value.split(": ").collect();
            if items.len() >= 2 {
                body.insert(items[0], items[1]);
            }
        }
        body
    }

    fn parse_headers(raw: &str) -> ((&str, &str), HashMap<&str, &str>) {
        let mut headers: HashMap<&str, &str> = HashMap::new();
        let mut method = "GET";
        let mut path = "/";
        if raw.len() <= 0 {
            return ((method, path), headers);
        }

        let mut values_iter = raw.split("\r\n").map(|r| r).into_iter();
        let first_line: Vec<&str> = values_iter.next().unwrap().split(" ").map(|r| r).collect();

        if first_line.len() >= 2 {
            method = first_line[0];
            path = first_line[1];
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
        ((method, path), headers)
    }
}
