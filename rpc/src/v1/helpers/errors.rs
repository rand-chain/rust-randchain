///! RPC Error codes and error objects

mod codes {
    // NOTE [ToDr] Codes from [-32099, -32000]
    pub const UNKNOWN: i64 = -32000;
    pub const EXECUTION_ERROR: i64 = -32015;
    pub const BLOCK_NOT_FOUND: i64 = -32099;
    pub const NODE_ALREADY_ADDED: i64 = -32150;
    pub const NODE_NOT_ADDED: i64 = -32151;
    pub const TOO_MANY_BLOCKS: i64 = -32152;
}

use jsonrpc_core::{Error, ErrorCode, Value};
use std::fmt;

pub fn unimplemented(details: Option<String>) -> Error {
    Error {
        code: ErrorCode::InternalError,
        message: "This request is not implemented yet. Please create an issue on Github repo."
            .into(),
        data: details.map(Value::String),
    }
}

pub fn invalid_params<T: fmt::Debug>(param: &str, details: T) -> Error {
    Error {
        code: ErrorCode::InvalidParams,
        message: format!("Couldn't parse parameters: {}", param),
        data: Some(Value::String(format!("{:?}", details))),
    }
}

pub fn execution<T: fmt::Debug>(data: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::EXECUTION_ERROR),
        message: "Execution error.".into(),
        data: Some(Value::String(format!("{:?}", data))),
    }
}

pub fn block_not_found<T: fmt::Debug>(data: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::BLOCK_NOT_FOUND),
        message: "Block with given hash is not found".into(),
        data: Some(Value::String(format!("{:?}", data))),
    }
}

pub fn block_at_height_not_found<T: fmt::Debug>(data: T) -> Error {
    Error {
        code: ErrorCode::ServerError(codes::BLOCK_NOT_FOUND),
        message: "Block at given height is not found".into(),
        data: Some(Value::String(format!("{:?}", data))),
    }
}

pub fn node_already_added() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::NODE_ALREADY_ADDED),
        message: "Node already added to the node table".into(),
        data: None,
    }
}

pub fn node_not_added() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::NODE_NOT_ADDED),
        message: "Node not added to the node table".into(),
        data: None,
    }
}

pub fn unknown() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::UNKNOWN),
        message: "Unknown error has occurred".into(),
        data: None,
    }
}

pub fn too_many_blocks() -> Error {
    Error {
        code: ErrorCode::ServerError(codes::TOO_MANY_BLOCKS),
        message: "Too many blocks to respond, use smaller `num`".into(),
        data: None,
    }
}
