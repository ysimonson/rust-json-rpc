extern crate serialize;

use serialize::json;

pub enum Id {
    StringBased(String),
    NumberBased(f64)
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

pub struct NormalResponse {
    id: Id,
    result: json::Json
}

pub struct ErrorResponse {
    id: Option<Id>,
    code: int,
    message: String,
    data: json::Json
}
