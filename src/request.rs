use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpRequest<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub version: &'a str,
    pub headers: HashMap<&'a str, &'a str>,
    pub body: &'a str,
}

impl<'a> HttpRequest<'a> {
    pub fn from(buf: &'a [u8]) -> Result<Self, &'a str> {
        let buffer_as_str = str::from_utf8(buf).map_err(|_| "Request is not valid UTF-8")?;
        let (headers_lines, body) = buffer_as_str
            .split_once("\r\n\r\n")
            .ok_or("Failed to separ headers and body")?;
        let mut headers_lines = headers_lines.lines();
        let mut request_line = headers_lines
            .next()
            .ok_or("Empty request")?
            .split_whitespace();
        let method = request_line.next().ok_or("Missing Method")?;
        let path = request_line.next().ok_or("Missing Path")?;
        let version = request_line.next().ok_or("Missing Version")?;
        let mut headers = HashMap::new();
        for line in headers_lines {
            let (head, value) = line.split_once(":").ok_or("Failed to split headers")?;
            headers.insert(head.trim(), value.trim());
        }
        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}

#[derive(Debug, Default)]
pub struct HttpResponse<'a> {
    pub status_line: &'a str,
    pub headers: HashMap<&'a str, &'a str>,
    pub body: &'a str,
}

impl<'a> HttpResponse<'a> {
    pub fn new(body: &'a str) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type", "text/html");
        Self {
            status_line: "HTTP/1.1 200 OK",
            headers,
            body,
        }
    }

    pub fn change_status_line(&mut self, status: &'a str) {
        self.status_line = status;
    }

    pub fn add_header(&mut self, k: &'a str, v: &'a str) {
        let _ = &self.headers.insert(k, v);
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        let result = format!(
            "{}\r\n{}\r\n\r\n{}",
            &self.status_line,
            &self
                .headers
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<String>>()
                .join("\r\n"),
            &self.body
        );
        result.into_bytes()
    }
}
