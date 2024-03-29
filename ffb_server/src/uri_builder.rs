use actix_web::http::Uri;
use std::collections::HashMap;
use std::fmt::Write;

lazy_static! {
    static ref RE_QUERY_PARAMS: regex::Regex =
        regex::Regex::new(r####"(?:(?P<key>[^=]+)(?:.)(?P<value>[^&]+)&{0,1})"####).unwrap();
}

pub struct UriBuilder {
    scheme: String,
    authority: String,
    path: String,
    query: HashMap<String, String>,
}

pub enum MessageType {
    Info,
    Error,
}

impl Default for UriBuilder {
    fn default() -> UriBuilder {
        UriBuilder {
            authority: "".into(),
            scheme: "https".into(),
            query: HashMap::new(),
            path: "".into(),
        }
    }
}

impl UriBuilder {
    pub fn from_existing_uri(uri: Uri) -> Self {
        Self::from(uri)
    }

    pub fn append_msg<'a>(&'a mut self, level: MessageType, msg: &str) -> &'a mut Self {
        self.query.remove("info");
        self.query.remove("error");
        match level {
            MessageType::Info => self.query.insert("info".into(), msg.to_string()),
            MessageType::Error => self.query.insert("error".into(), msg.to_string()),
        };
        self
    }

    pub fn build(&self) -> String {
        let query: String = if self.query.is_empty() {
            String::new()
        } else {
            let mut query_as_string: String = String::new();
            for (i, kv) in self.query.iter().enumerate() {
                if i == 0 {
                    let _ = write!(query_as_string, "?{}={}", kv.0, kv.1);
                } else {
                    let _ = write!(query_as_string, "&{}={}", kv.0, kv.1);
                }
            }
            query_as_string
        };
        format!("{}://{}{}{}", self.scheme, self.authority, self.path, query)
    }
}

impl From<Uri> for UriBuilder {
    fn from(uri: Uri) -> UriBuilder {
        let mut uri_builder = Self::default();
        if let Some(authority) = uri.authority() {
            uri_builder.authority = authority.to_string();
        }
        if let Some(scheme) = uri.scheme() {
            uri_builder.scheme = scheme.to_string();
        }
        uri_builder.path = uri.path().to_string();
        if let Some(query) = uri.query() {
            let captures = RE_QUERY_PARAMS.captures_iter(query);
            for capture in captures {
                uri_builder
                    .query
                    .insert(capture["key"].into(), capture["value"].into());
            }
        }

        uri_builder
    }
}
