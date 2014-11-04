extern crate serialize;
extern crate iron;

use serialize::json;
use std::collections::TreeMap;
use std::io::net::ip::{IpAddr, Ipv4Addr};
use iron::{status, Iron, IronResult};
use iron::Request as IronRequest;
use iron::Response as IronResponse;

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

impl Request {
    pub fn new(id: Option<Id>, method: String, params: Parameters) -> Request {
        return Request {
            id: id,
            method: method,
            params: params,
        }
    }
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

impl NormalResponse {
    pub fn new(id: Id, result: json::Json) -> NormalResponse {
        return NormalResponse {
            id: id,
            result: result,
        }
    }
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

impl ErrorResponse {
    pub fn new(id: Option<Id>, code: int, message: String, data: json::Json) -> ErrorResponse {
        ErrorResponse {
            id: id,
            code: code,
            message: message,
            data: data,
        }
    }

    fn newParseError() -> ErrorResponse {
        ErrorResponse::new(None, -32700, "Parse error".to_string(), json::Null)
    }

    fn newInvalidRequest(data: json::Json) -> ErrorResponse {
        ErrorResponse::new(None, -32600, "Invalid Request".to_string(), data)
    }

    pub fn newMethodNotFound(id: Id, data: json::Json) -> ErrorResponse {
        ErrorResponse::new(Some(id), -32601, "Method not found".to_string(), data)
    }

    pub fn newInvalidParams(id: Id, data: json::Json) -> ErrorResponse {
        ErrorResponse::new(Some(id), -32602, "Invalid params".to_string(), data)
    }

    fn newInternalError(id: Id, data: json::Json) -> ErrorResponse {
        ErrorResponse::new(Some(id), -32603, "Internal error".to_string(), data)
    }
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

pub struct Server {
    requestSender: Sender<(Vec<Request>, Sender<Vec<Response>>)>,
    requests: Receiver<(Vec<Request>, Sender<Vec<Response>>)>
}

impl Server {
    pub fn new() -> Server {
        let (tx, rx) = channel();

        Server {
            requestSender: tx,
            requests: rx,
        }
    }

    pub fn listener(&self, req: &mut IronRequest) -> IronResult<IronResponse> {
        Ok(IronResponse::with(status::Ok, "Hello, world"))
    }
}

