// Couchbase Lite error classs
//
// Copyright (c) 2020 Couchbase, Inc All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

#![allow(non_upper_case_globals)]

use crate::c_api::{
    CBLError, CBLErrorDomain, CBLError_Message, FLError, kCBLDomain, kCBLFleeceDomain,
    kCBLNetworkDomain, kCBLPOSIXDomain, kCBLSQLiteDomain, kCBLWebSocketDomain,
};
use crate::error;
use enum_primitive::FromPrimitive;
use std::fmt;

//////// ERROR STRUCT:

/** Error type. Wraps multiple types of errors in an enum. */
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Error {
    pub code: ErrorCode,
    pub(crate) internal_info: Option<u32>,
}

/** The enum that stores the error domain and code for an Error. */
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ErrorCode {
    CouchbaseLite(CouchbaseLiteError),
    POSIX(i32),
    SQLite(i32),
    Fleece(FleeceError),
    Network(NetworkError),
    WebSocket(i32),
}

// Redefine `Result` to assume our `Error` type
pub type Result<T> = std::result::Result<T, Error>;

enum_from_primitive! {
    /** Couchbase Lite error codes. */
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum CouchbaseLiteError {
        AssertionFailed = 1,    // Internal assertion failure
        Unimplemented,          // Oops, an unimplemented API call
        UnsupportedEncryption,  // Unsupported encryption algorithm
        BadRevisionID,          // Invalid revision ID syntax
        CorruptRevisionData,    // Revision contains corrupted/unreadable data
        NotOpen,                // Database/KeyStore/index is not open
        NotFound,               // Document not found
        Conflict,               // Document update conflict
        InvalidParameter,       // Invalid function parameter or struct value
        UnexpectedError, /*10*/ // Internal unexpected C++ exception
        CantOpenFile,           // Database file can't be opened; may not exist
        IOError,                // File I/O error
        MemoryError,            // Memory allocation failed (out of memory?)
        NotWriteable,           // File is not writeable
        CorruptData,            // Data is corrupted
        Busy,                   // Database is busy/locked
        NotInTransaction,       // Function must be called while in a transaction
        TransactionNotClosed,   // Database can't be closed while a transaction is open
        Unsupported,            // Operation not supported in this database
        NotADatabaseFile,/*20*/ // File is not a database, or encryption key is wrong
        WrongFormat,            // Database exists but not in the format/storage requested
        Crypto,                 // Encryption/decryption error
        InvalidQuery,           // Invalid query
        MissingIndex,           // No such index, or query requires a nonexistent index
        InvalidQueryParam,      // Unknown query param name, or param number out of range
        RemoteError,            // Unknown error from remote server
        DatabaseTooOld,         // Database file format is older than what I can open
        DatabaseTooNew,         // Database file format is newer than what I can open
        BadDocID,               // Invalid document ID
        CantUpgradeDatabase,/*30*/ // DB can't be upgraded (might be unsupported dev version)

        UntranslatableError = 1000,  // Can't translate native error (unknown domain or code)
    }
}

enum_from_primitive! {
    /** Fleece error codes. */
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum FleeceError {
        MemoryError = 1,    // Out of memory, or allocation failed
        OutOfRange,         // Array index or iterator out of range
        InvalidData,        // Bad input data (NaN, non-string key, etc.)
        EncodeError,        // Structural error encoding (missing value, too many ends, etc.)
        JSONError,          // Error parsing JSON
        UnknownValue,       // Unparseable data in a Value (corrupt? Or from some distant future?)
        InternalError,      // Something that shouldn't happen
        NotFound,           // Key not found
        SharedKeysStateError, // Misuse of shared keys (not in transaction, etc.)
        POSIXError,         // Something went wrong at the OS level (file I/O, etc.)
        Unsupported,        // Operation is unsupported
    }
}

enum_from_primitive! {
    /** Network error codes defined by Couchbase Lite. */
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum NetworkError {
        DNSFailure = 1,            // DNS lookup failed
        UnknownHost,               // DNS server doesn't know the hostname
        Timeout,                   // No response received before timeout
        InvalidURL,                // Invalid URL
        TooManyRedirects,          // HTTP redirect loop
        TLSHandshakeFailed,        // Low-level error establishing TLS
        TLSCertExpired,            // Server's TLS certificate has expired
        TLSCertUntrusted,          // Cert isn't trusted for other reason
        TLSClientCertRequired,     // Server requires client to have a TLS certificate
        TLSClientCertRejected,     // Server rejected my TLS client certificate
        TLSCertUnknownRoot,        // Self-signed cert, or unknown anchor cert
        InvalidRedirect,           // Attempted redirect to invalid URL
        Unknown,                   // Unknown networking error
        TLSCertRevoked,            // Server's cert has been revoked
        TLSCertNameMismatch,       // Server cert's name does not match DNS name
    }
}

impl Error {
    pub fn default() -> Self {
        Self::new(&CBLError::default())
    }

    pub(crate) fn new(err: &CBLError) -> Self {
        Self {
            code: ErrorCode::new(err),
            internal_info: Some(err.internal_info),
        }
    }

    pub(crate) const fn cbl_error(e: CouchbaseLiteError) -> Self {
        Self {
            code: ErrorCode::CouchbaseLite(e),
            internal_info: None,
        }
    }

    pub(crate) fn fleece_error(e: FLError) -> Self {
        Self {
            code: ErrorCode::from_fleece(e as i32),
            internal_info: None,
        }
    }

    pub(crate) fn as_cbl_error(&self) -> CBLError {
        let domain: u32;
        let code: i32;
        match &self.code {
            ErrorCode::CouchbaseLite(e) => {
                domain = kCBLDomain;
                code = *e as i32;
            }
            ErrorCode::Fleece(e) => {
                domain = kCBLFleeceDomain;
                code = *e as i32;
            }
            ErrorCode::Network(e) => {
                domain = kCBLNetworkDomain;
                code = *e as i32;
            }
            ErrorCode::POSIX(e) => {
                domain = kCBLPOSIXDomain;
                code = *e;
            }
            ErrorCode::SQLite(e) => {
                domain = kCBLSQLiteDomain;
                code = *e;
            }
            ErrorCode::WebSocket(e) => {
                domain = kCBLWebSocketDomain;
                code = *e;
            }
        }
        CBLError {
            domain: domain as CBLErrorDomain,
            code,
            internal_info: self.internal_info.unwrap_or(0),
        }
    }

    pub fn message(&self) -> String {
        if let ErrorCode::CouchbaseLite(e) = self.code {
            if e == CouchbaseLiteError::UntranslatableError {
                return "Unknown error".to_string();
            }
        }
        unsafe {
            CBLError_Message(&self.as_cbl_error())
                .to_string()
                .unwrap_or_else(|| {
                    error!("Generating the error message for error ({:?}) and internal info ({:?}) failed", self.code, self.internal_info);
                    "Unknown error".to_string()
                })
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("{:?}: {})", self.code, self.message()))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        fmt.write_str(&self.message())
    }
}

impl ErrorCode {
    fn new(err: &CBLError) -> Self {
        match u32::from(err.domain) {
            kCBLDomain => CouchbaseLiteError::from_i32(err.code)
                .map_or(Self::untranslatable(), Self::CouchbaseLite),
            kCBLNetworkDomain => {
                NetworkError::from_i32(err.code).map_or(Self::untranslatable(), Self::Network)
            }
            kCBLPOSIXDomain => Self::POSIX(err.code),
            kCBLSQLiteDomain => Self::SQLite(err.code),
            kCBLFleeceDomain => Self::from_fleece(err.code),
            kCBLWebSocketDomain => Self::WebSocket(err.code),
            _ => Self::untranslatable(),
        }
    }

    fn from_fleece(fleece_error: i32) -> Self {
        FleeceError::from_i32(fleece_error).map_or(Self::untranslatable(), Self::Fleece)
    }

    const fn untranslatable() -> Self {
        Self::CouchbaseLite(CouchbaseLiteError::UntranslatableError)
    }
}

//////// CBLERROR UTILITIES:
#[allow(clippy::derivable_impls)]
impl Default for CBLError {
    fn default() -> Self {
        Self {
            domain: 0,
            code: 0,
            internal_info: 0,
        }
    }
}

impl std::ops::Not for CBLError {
    type Output = bool;
    fn not(self) -> bool {
        self.code == 0
    }
}

impl std::ops::Not for &CBLError {
    type Output = bool;
    fn not(self) -> bool {
        self.code == 0
    }
}

// Convenient way to return a Result from a failed CBLError
pub(crate) fn failure<T>(err: CBLError) -> Result<T> {
    assert!(err.code != 0);
    Err(Error::new(&err))
}

pub(crate) fn check_failure(status: bool, err: &CBLError) -> Result<()> {
    if status {
        return Ok(());
    }
    assert!(err.code != 0);
    Err(Error::new(err))
}

pub(crate) fn check_error(err: &CBLError) -> Result<()> {
    if err.domain == 0 || err.code == 0 {
        Ok(())
    } else {
        Err(Error::new(err))
    }
}

pub(crate) fn check_bool<F>(func: F) -> Result<()>
where
    F: Fn(*mut CBLError) -> bool,
{
    let mut error = CBLError::default();
    let ok = func(&mut error);
    check_failure(ok, &error)
}

// The first parameter is a function that returns a non-null pointer or sets the error.
// The second parameter is a function that takes the returned pointer and returns the final result.
pub(crate) fn check_ptr<PTR, F, MAPF, RESULT>(func: F, map: MAPF) -> Result<RESULT>
where
    F: Fn(*mut CBLError) -> *mut PTR,
    MAPF: FnOnce(*mut PTR) -> RESULT,
{
    let mut error = CBLError::default();
    let ptr = func(&mut error);
    if ptr.is_null() {
        failure(error)
    } else {
        Ok(map(ptr))
    }
}

// The first parameter is a function that returns a non-null pointer or sets the error.
// The second parameter is a function that takes the returned pointer and returns the final result.
pub(crate) fn check_io<F>(mut func: F) -> std::io::Result<usize>
where
    F: FnMut(*mut CBLError) -> i32,
{
    let mut error = CBLError::default();
    let n = func(&mut error);
    if n < 0 {
        // TODO: Better error mapping!
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            Error::new(&error),
        ));
    }
    #[allow(clippy::cast_sign_loss)]
    Ok(n as usize)
}
