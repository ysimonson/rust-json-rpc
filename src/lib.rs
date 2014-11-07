extern crate serialize;
extern crate iron;

use serialize::json::{ToJson, Json, JsonList, JsonObject, Object, Null, encode, from_str};
use std::collections::TreeMap;
use std::io::net::ip::{IpAddr, Ipv4Addr};
use iron::{status, Iron, IronResult};
use iron::Request as IronRequest;
use iron::Response as IronResponse;
use std::str;

pub enum Id {
    StringBased(String),
    NumberBased(f64)
}

impl ToJson for Id {
    fn to_json(&self) -> Json {
        match *self {
            StringBased(ref s) => s.to_json(),
            NumberBased(ref n) => n.to_json()
        }
    }
}

pub enum Parameters {
    Positional(JsonList),
    Named(JsonObject)
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
        let mut obj: TreeMap<String, Json> = TreeMap::new();
        obj.insert("jsonrpc".to_string(), "2.0".to_string().to_json());
        obj.insert("id".to_string(), self.id.to_json());
        obj.insert("result".to_string(), self.result.to_json());
        Object(obj)
    }
}

pub struct ErrorResponse {
    id: Option<Id>,
    code: int,
    message: String,
    data: Json
}

impl ErrorResponse {
    pub fn new(id: Option<Id>, code: int, message: String, data: Json) -> ErrorResponse {
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
        let mut errorObj: TreeMap<String, Json> = TreeMap::new();
        errorObj.insert("code".to_string(), self.code.to_json());
        errorObj.insert("message".to_string(), self.message.to_json());
        errorObj.insert("data".to_string(), self.data.to_json());

        let mut obj: TreeMap<String, Json> = TreeMap::new();
        obj.insert("jsonrpc".to_string(), "2.0".to_string().to_json());
        obj.insert("id".to_string(), self.id.to_json());
        obj.insert("error".to_string(), errorObj.to_json());

        Object(obj)
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

    fn respond(&self, jrpc_res: Response) -> IronResult<IronResponse> {
        let res_str = encode(&jrpc_res.to_json());
        let res_bytes = res_str.as_bytes();
        let mut http_res = IronResponse::with(status::Ok, res_bytes);
        http_res.headers.content_type = Some(iron::headers::content_type::MediaType::new("application".to_string(), "json".to_string(), Vec::new()));
        Ok(http_res)
    }

    pub fn listener(&self, req: &mut IronRequest) -> IronResult<IronResponse> {
        match str::from_utf8(req.body.as_slice()).and_then(|body| from_str(body).ok()) {
            Some(body) => self.respond(Error(ErrorResponse::newParseError())),
            None => self.respond(Error(ErrorResponse::newParseError()))
        }
    }
}
