extern crate serialize;
extern crate iron;

use serialize::json::{ToJson, Json, JsonList, JsonObject, Object, Null, encode, from_str};
use std::collections::TreeMap;
use std::io::net::ip::{IpAddr, Ipv4Addr};
use iron::{status, Iron, IronResult};
use iron::Request as IronRequest;
use iron::Response as IronResponse;
use std::str;
use std::string;

pub enum Id {
    String(string::String),
    I64(i64),
    U64(u64),
    F64(f64)
}

impl ToJson for Id {
    fn to_json(&self) -> Json {
        match *self {
            String(ref s) => s.to_json(),
            I64(ref n) => n.to_json(),
            U64(ref n) => n.to_json(),
            F64(ref n) => n.to_json()
        }
    }
}

pub enum Parameters {
    Positional(JsonList),
    Named(JsonObject)
}

pub struct Request {
    id: Option<Id>,
    method: string::String,
    params: Parameters
}

impl Request {
    pub fn new(id: Option<Id>, method: string::String, params: Parameters) -> Request {
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

impl ToJson for Response {
    fn to_json(&self) -> Json {
        match *self {
            Normal(ref r) => r.to_json(),
            Error(ref e) => e.to_json()
        }
    }
}

pub struct NormalResponse {
    id: Id,
    result: Json
}

impl NormalResponse {
    pub fn new(id: Id, result: Json) -> NormalResponse {
        return NormalResponse {
            id: id,
            result: result,
        }
    }
}

impl ToJson for NormalResponse {
    fn to_json(&self) -> Json {
        let mut obj: TreeMap<string::String, Json> = TreeMap::new();
        obj.insert("jsonrpc".to_string(), "2.0".to_string().to_json());
        obj.insert("id".to_string(), self.id.to_json());
        obj.insert("result".to_string(), self.result.to_json());
        Object(obj)
    }
}

pub struct ErrorResponse {
    id: Option<Id>,
    code: int,
    message: string::String,
    data: Json
}

impl ErrorResponse {
    pub fn new(id: Option<Id>, code: int, message: string::String, data: Json) -> ErrorResponse {
        ErrorResponse {
            id: id,
            code: code,
            message: message,
            data: data,
        }
    }

    fn newParseError() -> ErrorResponse {
        ErrorResponse::new(None, -32700, "Parse error".to_string(), Null)
    }

    fn newInvalidRequest(data: Json) -> ErrorResponse {
        ErrorResponse::new(None, -32600, "Invalid Request".to_string(), data)
    }

    pub fn newMethodNotFound(id: Id, data: Json) -> ErrorResponse {
        ErrorResponse::new(Some(id), -32601, "Method not found".to_string(), data)
    }

    pub fn newInvalidParams(id: Id, data: Json) -> ErrorResponse {
        ErrorResponse::new(Some(id), -32602, "Invalid params".to_string(), data)
    }

    fn newInternalError(id: Id, data: Json) -> ErrorResponse {
        ErrorResponse::new(Some(id), -32603, "Internal error".to_string(), data)
    }
}

impl ToJson for ErrorResponse {
    fn to_json(&self) -> Json {
        let mut errorObj: TreeMap<string::String, Json> = TreeMap::new();
        errorObj.insert("code".to_string(), self.code.to_json());
        errorObj.insert("message".to_string(), self.message.to_json());
        errorObj.insert("data".to_string(), self.data.to_json());

        let mut obj: TreeMap<string::String, Json> = TreeMap::new();
        obj.insert("jsonrpc".to_string(), "2.0".to_string().to_json());
        obj.insert("id".to_string(), self.id.to_json());
        obj.insert("error".to_string(), errorObj.to_json());

        Object(obj)
    }
}

pub struct Server {
    request_sender: Sender<(Vec<Request>, Sender<Vec<Response>>)>,
    requests: Receiver<(Vec<Request>, Sender<Vec<Response>>)>
}

impl Server {
    pub fn new() -> Server {
        let (tx, rx) = channel();

        Server {
            request_sender: tx,
            requests: rx,
        }
    }

    fn parse_json_request(&self, req: TreeMap<string::String, Json>) -> Result<Request, string::String> {
        let version = req.find(&"jsonrpc".to_string());

        if !version.is_some() || version.unwrap() != &Json::String("2.0".to_string()) {
            return Err("Invalid version specified".to_string());
        }

        let method = match req.find(&"method".to_string()) {
            Some(&Json::String(ref s)) => Some(s),
            _ => None
        };

        if !method.is_some() {
            return Err("Invalid method specified".to_string());
        }

        let params = match req.find(&"params".to_string()) {
            Some(&Json::Object(ref o)) => Some(Parameters::Named(o.clone())),
            Some(&Json::List(ref l)) => Some(Parameters::Positional(l.clone())),
            _ => None
        };

        if !params.is_some() {
            return Err("Invalid params specified".to_string());
        }

        let (id, is_invalid_id) = match req.find(&"id".to_string()) {
            Some(&Json::String(ref s)) => (Some(Id::String(s.clone())), false),
            Some(&Json::I64(n)) => (Some(Id::I64(n)), false),
            Some(&Json::U64(n)) => (Some(Id::U64(n)), false),
            Some(&Json::F64(n)) => (Some(Id::F64(n)), false),
            Some(&Json::Null) => (None, false),
            _ => (None, true)
        };

        if is_invalid_id {
            return Err("Invalid id specified".to_string());
        }

        return Ok(Request::new(id, method.unwrap().clone(), params.unwrap()))
    }

    fn process_request(&self, request_json: Vec<Json>) -> Vec<Json> {
        return Vec::new()
    }

    fn single_request(&self, request_json: TreeMap<string::String, Json>) -> Json {
        let mut wrapped_request_json: Vec<Json> = Vec::with_capacity(1);
        wrapped_request_json.push(Json::Object(request_json));
        let mut wrapped_response_json = self.process_request(wrapped_request_json);
        wrapped_response_json.pop().unwrap()
    }

    pub fn listener(&self, req: &mut IronRequest) -> IronResult<IronResponse> {
        let response_json = match str::from_utf8(req.body.as_slice()).and_then(|body| from_str(body).ok()) {
            Some(Json::Object(body)) => self.single_request(body),
            _ => Error(ErrorResponse::newParseError()).to_json()
        };

        let response_str = encode(&response_json.to_json());
        let response_bytes = response_str.as_bytes();
        let mut response_http = IronResponse::with(status::Ok, response_bytes);
        response_http.headers.content_type = Some(iron::headers::content_type::MediaType::new("application".to_string(), "json".to_string(), Vec::new()));
        Ok(response_http)
    }
}
