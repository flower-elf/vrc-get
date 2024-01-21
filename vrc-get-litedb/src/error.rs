use std::str;
use crate::lowlevel::{FFISlice, GcHandle};

#[derive(Debug)]
pub struct LiteDbError {
    message: Box<str>,
    code: LiteDBErrorCode,
}

impl LiteDbError {
    pub(crate) unsafe fn from_ffi(error: LiteDbErrorFFI) -> Self {
        let message=  str::from_boxed_utf8_unchecked(error.message.as_boxed_byte_slice());
        if error.code == -1 {
            // -1 means unexpected error in C# code so panic here
            panic!("{}", message);
        }

        Self {
            // SAFETY: C# guarantees the safety.
            code: std::mem::transmute(error.code),
            message,
        }
    }
}

#[cfg_attr(not(doc), repr(i32))]
#[derive(Debug, Eq, PartialEq)]
pub enum LiteDBErrorCode {
    Uncategorized = 0,
    FileNotFound = 101,
    DatabaseShutdown = 102,
    InvalidDatabase = 103,
    FileSizeExceeded = 105,
    CollectionLimitExceeded = 106,
    IndexDropId = 108,
    IndexDuplicateKey = 110,
    InvalidIndexKey = 111,
    IndexNotFound = 112,
    InvalidDbref = 113,
    LockTimeout = 120,
    InvalidCommand = 121,
    AlreadyExistsCollectionName = 122,
    AlreadyOpenDatafile = 124,
    InvalidTransactionState = 126,
    IndexNameLimitExceeded = 128,
    InvalidIndexName = 129,
    InvalidCollectionName = 130,
    TempEngineAlreadyDefined = 131,
    InvalidExpressionType = 132,
    CollectionNotFound = 133,
    CollectionAlreadyExist = 134,
    IndexAlreadyExist = 135,
    InvalidUpdateField = 136,
    InvalidFormat = 200,
    DocumentMaxDepth = 201,
    InvalidCtor = 202,
    UnexpectedToken = 203,
    InvalidDataType = 204,
    PropertyNotMapped = 206,
    InvalidTypedName = 207,
    PropertyReadWrite = 209,
    InitialSizeCryptoNotSupported = 210,
    InvalidInitialSize = 211,
    InvalidNullCharString = 212,
    InvalidFreeSpacePage = 213,
    DataTypeNotAssignable = 214,
    AvoidUseOfProcess = 215,
    NotEncrypted = 216,
    InvalidPassword = 217,
}

#[repr(C)]
pub(crate) struct LiteDbErrorFFI {
    // must be 
    message: FFISlice<u8>,
    code: i32,
}

#[repr(C)]
pub(crate) struct HandleErrorResult {
    pub result: Option<GcHandle>,
    pub error: LiteDbErrorFFI,
}

impl HandleErrorResult {
    pub unsafe fn into_result(self) -> super::Result<GcHandle> {
        if let Some(result) = self.result {
            Ok(result)
        } else {
            Err(LiteDbError::from_ffi(self.error))
        }
    }
}
