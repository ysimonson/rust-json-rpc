extern crate serialize;

use serialize::json;
use std::collections::TreeMap;

pub enum Id {
    StringBased(String),
    NumberBased(f64)
}

impl json::ToJson for Id {
    fn to_json(&self) -> json::Json {
        match *self {
            StringBased(ref s) => s.to_json(),
            NumberBased(ref n) => n.to_json()
        }
    }
}

pub enum Parameters {
    Positional(json::JsonList),
    Named(json::JsonObject)
}

pub struct Request {
    id: Option<Id>,
    method: String,
    params: Parameters
}

pub enum Response {
    Normal(NormalResponse),
    Error(ErrorResponse)
}

impl json::ToJson for Response {
    fn to_json(&self) -> json::Json {
        match *self {
            Normal(ref r) => r.to_json(),
            Error(ref e) => e.to_json()
        }
    }
}

pub struct NormalResponse {
    id: Id,
    result: json::Json
}

impl json::ToJson for NormalResponse {
    fn to_json(&self) -> json::Json {
        let mut obj: TreeMap<String, json::Json> = TreeMap::new();
        obj.insert("jsonrpc".to_string(), "2.0".to_string().to_json());
        obj.insert("id".to_string(), self.id.to_json());
        obj.insert("result".to_string(), self.result.to_json());
        json::Object(obj)
    }
}

pub struct ErrorResponse {
    id: Option<Id>,
    code: int,
    message: String,
    data: json::Json
}

impl json::ToJson for ErrorResponse {
    fn to_json(&self) -> json::Json {
        let mut errorObj: TreeMap<String, json::Json> = TreeMap::new();
        errorObj.insert("code".to_string(), self.code.to_json());
        errorObj.insert("message".to_string(), self.message.to_json());
        errorObj.insert("data".to_string(), self.data.to_json());

        let mut obj: TreeMap<String, json::Json> = TreeMap::new();
        obj.insert("jsonrpc".to_string(), "2.0".to_string().to_json());
        obj.insert("id".to_string(), self.id.to_json());
        obj.insert("error".to_string(), errorObj.to_json());

        json::Object(obj)
    }
}

