//! Copyright (c) 2025 Contributors to the Eclipse Foundation
//!
//! See the NOTICE file(s) distributed with this work for additional
//! information regarding copyright ownership.
//!
//! This program and the accompanying materials are made available under the
//! terms of the Apache License Version 2.0 which is available at
//! <https://www.apache.org/licenses/LICENSE-2.0>
//!
//! SPDX-License-Identifier: Apache-2.0
//!
//! # Key-Value and File Handling API and Implementation based on the Genivi Persistence Client Library
//!
//! ## Introduction
//!
//! This Rust Wrapper (library) provides Key-Value and File Handling using the Genivi [Persistence Client Library](https://github.com/GENIVI/persistence-client-library) to
//! persist the data. In order to load the library without the need to have it build inside your environment, [Libloading](https://github.com/nagisa/rust_libloading) is used. 
//! [once_cell](https://github.com/matklad/once_cell) is used to handle the loaded library thread safe. 
//! The code can drastically be simplified with static library linking when having the library inside the build environment. Currently the Wrapper searches on startup for the C-Library with the name "libpersistence_client_library.so.7" inside the "/usr/lib/" path.
//! Functions that are only used internally for loading specific C-API functions are not displayed inside this documentation. 
//! No other direct dependencies are used besides the Rust `std` library.
//!
//! The key-value-storage is opened or initialized with [`init_library`] and deinitialized with [`deinit_library`]
//! Data Storage and Verification is handled by the Persistence Client Library. 
//! Reading and writing of data is done via a u8 vector Bytestream.
//! 
//! Writing a Key-Value to the KVS can be done by calling [`write_key`] with `ldbid`, `resource_id` and `user_no` as specifying parameters and the Bytestream Vector 
//! as the fourth second parameter. 
//! 
//! Reading a Key-Value to the KVS can be done by calling [`read_key`] with the key specifying parameters
//! The function returns a u8 Vector containing the data on success and an error code on failure.
//! 
//! Furthermore you are able to get the Keysize performing [`get_key_size`] and delete a key with [`delete_key`] combined with the key identifying parameters.
//! 
//! The specific data management is provided by the Persistence Client Library, which provides different backend options modularly (e.g. SQLite).
//! Each Application has its own seperated Key-Value Databases. 
//!
//! ## Example Usage
//!
//! ```
//! use persistence_client_library_wrapper::*;
//!
//! fn main() -> Result<(), ErrorCode> {
//!     let app_name = "MyApp";
//!     init_library(app_name)?;
//! 
//!     //Key-Value-Handling
//!     let ldbid = 0xFF; 
//!     let resource_id = "MyKey";
//!     let user_no = 0; 
//!     let buffer = "Hello World!".as_bytes().to_vec(); //.as_bytes().to_vec() or .into() is used to provide a human-readable string as an example, but is intended to be used with any data as a "bytestream".
//!     write_key(ldbid, resource_id, user_no, buffer)?;
//! 
//!     let key_data = read_key(ldbid, resource_id, user_no)?;
//! 
//! 
//!     //File-Handling
//!     let fd = open_file(ldbid, resource_id, user_no)?;
//!     write_file(fd, buffer)?;
//!     let file_data = read_file(fd)?;
//!     
//!     close_file(fd)?;
//!     deinit_library()?;
//! 
//!     Ok(())
//! }
//! ```
//!  
//! ## Feature Coverage
//!
//! Feature and requirement definition:
//!   * [Features/Persistency/Key-Value-Storage](https://github.com/eclipse-score/score/blob/ulhu_persistency_kvs/docs/features/persistency/key-value-storage/index.rst#specification)
//!   * [Requirements/Stakeholder](https://github.com/eclipse-score/score/blob/ulhu_persistency_kvs/docs/requirements/stakeholder/index.rst)
//!
//! Supported features and requirements:
//!   * `FEAT_REQ__KVS__tooling`
//!   * `FEAT_REQ__KVS__maximum_size`
//!   * `FEAT_REQ__KVS__thread_safety`
//!   * `FEAT_REQ__KVS__supported_datatypes_keys`
//!   * `FEAT_REQ__KVS__supported_datatypes_values`
//!   * `FEAT_REQ__KVS__default_values`
//!   * `FEAT_REQ__KVS__default_value_reset`
//!   * `FEAT_REQ__KVS__default_value_retrieval`
//!   * `FEAT_REQ__KVS__persistency`
//!   * `FEAT_REQ__KVS__integrity_check`
//!   * `STKH_REQ__350`: Safe key-value-store
//!   * `STKH_REQ__30`: JSON storage format
//!   * `STKH_REQ__8`: Defaults stored in JSON format
//!   * `STKH_REQ__12`: Support storing data on non-volatile memory
//!   * `STKH_REQ__13`: POSIX portability
//!
//! Currently unsupported features:
//!   * `FEAT_REQ__KVS__cpp_rust_interoperability`
//!   * `FEAT_REQ__KVS__versioning`
//!   * `FEAT_REQ__KVS__update_mechanism`
//!   * `FEAT_REQ__KVS__snapshots`
//! 
//!

extern crate libloading;

use libloading::{Library, Symbol};
use std::ffi::CString;

use std::sync::Mutex; 
use once_cell::sync::Lazy;

mod persistence_client_library_ffi;

use crate::persistence_client_library_ffi::*; 

/// Keep loaded Lib static and secured 
static LIB: Lazy<Mutex<Option<Library>>> = Lazy::new(|| Mutex::new(None));

/// Runtime Error Codes
#[repr(i32)]
#[derive(Debug)]
pub enum ErrorCode {
    /// common error, for this error errno will be set
    EpersCommon = -1,
    /// file system is locked
    EpersLockfs = -2,
    /// bad storage policy
    EpersBadpol = -3,
    /// open handle limit reached
    EpersMaxhandle = -4,
    /// max buffer limit for persistence data
    EpersBuflimit = -5,
    /// persistence resource configuration table not found
    EpersNoprcTable = -6,
    /// key not found
    EpersNokey = -7,
    /// no data for key
    EpersNokeydata = -8,
    /// write of data failed
    EpersSetDtaFailed = -9,
    /// failed to open file
    EpersOpenfile = -10,
    /// invalid buffer or key
    EpersDeserBufOrKey = -11,
    /// can't allocate memory for deserialization of key/value
    EpersDeserAllocMem = -12,
    /// no policy available in data to serialize
    EpersDeserPolicy = -13,
    /// no store type available in data to serialize
    EpersDeserStore = -14,
    /// no permission available in data to serialize
    EpersDeserPerm = -15,
    /// no max size available in data to serialize
    EpersDeserMaxSize = -16,
    /// no responsibility available in data to serialize
    EpersDeserResp = -17,
    /// out of array bounds
    EpersOutOfBounds = -18,
    /// failed to map config file
    EpersConfigMapFailed = -19,
    /// config file is not available
    EpersConfigNotAvailable = -20,
    /// can't stat config file
    EpersConfigNoStat = -21,
    /// plugin function not found
    EpersNoPluginFcnT = -22,
    /// dlopen error
    EpersDlopenError = -23,
    /// plugin function not loaded
    EpersNoPluginFunct = -24,
    /// file remove error
    EpersFileRemove = -25,
    /// error code to signal last entry in DB
    EpersLastEntryInDb = -26,
    /// internal database error
    EpersDbErrorInternal = -27,
    /// db key size is too long
    EpersDbKeySize = -28,
    /// db value size is too long
    EpersDbValueSize = -29,
    /// resource is not a key
    EpersResNoKey = -30,
    /// change notification signal could not be sent
    EpersNotifySig = -31,
    /// client library has not been initialized
    EpersNotInitialized = -32,
    /// max buffer size
    EpersMaxBuffSize = -33,
    /// failed to setup dbus mainloop
    EpersDbusMainloop = -34,
    /// failed to register lifecycle dbus
    EpersRegisterLifecycle = -35,
    /// failed to register admin service dbus
    EpersRegisterAdmin = -36,
    /// registration on this key is not allowed
    EpersNotifyNotAllowed = -37,
    /// the requested resource is not a file
    EpersResourceNoFile = -38,
    /// write to requested resource failed, read-only resource
    EpersResourceReadOnly = -39,
    /// max number of cancel shutdown exceeded
    EpersShutdownMaxCancel = -40,
    /// not permitted to use this function
    EpersShutdownNoPermit = -42,
    /// not a trusted application, no access to persistence data
    EpersShutdownNoTrusted = -43,
    /// not the responsible application to modify shared data
    EpersNotRespApp = -44,
    /// plugin function not available
    EpersNoPluginFunctAvail = -45,
    /// plugin variable not available
    EpersNoPluginVar = -46,
    /// requested handle is not valid (since PCL v7.0.3)
    EpersNoRegToPas = -47,
    /// requested handle is not valid (since PCL v7.0.3)
    EpersInvalidHandle = -1000,

    /*Libloading ErrorCodes */
    /// a specific C function was not found
    FunctionNotFound,
    /// the library is not loaded
    LibraryNotLoaded,
    /// loading the library failed
    LoadLibraryFailed,
    /// loading a C function failed
    LoadFunctionFailed,
    /// a datatype conversion failed
    DatatypeConversionFailed,

    /// unknown error
    Unknown(i32),
}



impl From<i32> for ErrorCode {
    fn from(value: i32) -> Self {
        match value {
            -1 => Self::EpersCommon,
            -2 => Self::EpersLockfs,
            -3 => Self::EpersBadpol,
            -4 => Self::EpersMaxhandle,
            -5 => Self::EpersBuflimit,
            -6 => Self::EpersNoprcTable,
            -7 => Self::EpersNokey,
            -8 => Self::EpersNokeydata,
            -9 => Self::EpersSetDtaFailed,
            -10 => Self::EpersOpenfile,
            -11 => Self::EpersDeserBufOrKey,
            -12 => Self::EpersDeserAllocMem,
            -13 => Self::EpersDeserPolicy,
            -14 => Self::EpersDeserStore,
            -15 => Self::EpersDeserPerm,
            -16 => Self::EpersDeserMaxSize,
            -17 => Self::EpersDeserResp,
            -18 => Self::EpersOutOfBounds,
            -19 => Self::EpersConfigMapFailed,
            -20 => Self::EpersConfigNotAvailable,
            -21 => Self::EpersConfigNoStat,
            -22 => Self::EpersNoPluginFcnT,
            -23 => Self::EpersDlopenError,
            -24 => Self::EpersNoPluginFunct,
            -25 => Self::EpersFileRemove,
            -26 => Self::EpersLastEntryInDb,
            -27 => Self::EpersDbErrorInternal,
            -28 => Self::EpersDbKeySize,
            -29 => Self::EpersDbValueSize,
            -30 => Self::EpersResNoKey,
            -31 => Self::EpersNotifySig,
            -32 => Self::EpersNotInitialized,
            -33 => Self::EpersMaxBuffSize,
            -34 => Self::EpersDbusMainloop,
            -35 => Self::EpersRegisterLifecycle,
            -36 => Self::EpersRegisterAdmin,
            -37 => Self::EpersNotifyNotAllowed,
            -38 => Self::EpersResourceNoFile,
            -39 => Self::EpersResourceReadOnly,
            -40 => Self::EpersShutdownMaxCancel,
            -42 => Self::EpersShutdownNoPermit,
            -43 => Self::EpersShutdownNoTrusted,
            -44 => Self::EpersNotRespApp,
            -45 => Self::EpersNoPluginFunctAvail,
            -46 => Self::EpersNoPluginVar,
            -47 => Self::EpersNoRegToPas,
            -1000 => Self::EpersInvalidHandle,
            _ => Self::Unknown(value),
        }
    }
}

/// Method that dynamically loads the Persistence Client Library (C-API)
fn load_library() -> Result<(), ErrorCode> {
    
    let lib_name = CString::new("/usr/lib/libpersistence_client_library.so.7")
        .map_err(|_| ErrorCode::DatatypeConversionFailed)?;
    let lib = unsafe {
        Library::new(lib_name.to_str().map_err(|_| ErrorCode::DatatypeConversionFailed)?)
        .map_err(|_| ErrorCode::LoadLibraryFailed)?
    };
    let mut lib_lock = LIB.lock().unwrap();
    *lib_lock = Some(lib);

    Ok(())
}

/*-----------------------Load Library Functions-----------------------*/
/*-------Load Library Functions-------*/
#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_init_library functionality (Libloading)
fn load_pcl_init_library() -> Result<PclInitLibraryFn, ErrorCode> {
    
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_init_library: Symbol<PclInitLibraryFn> = lib.get(b"pclInitLibrary")
        .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_init_library)
    }
}

#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_deinit_library functionality (Libloading)
fn load_pcl_deinit_library() -> Result<PclDeinitLibraryFn, ErrorCode> {
    
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_deinit_library: Symbol<PclDeinitLibraryFn> = lib.get(b"pclDeinitLibrary")
            .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_deinit_library)
    }
}

/*-------Load Key Handling Functions-------*/

#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_key_read_data functionality (Libloading)
fn load_pcl_key_read_data() -> Result<PclKeyReadDataFn, ErrorCode> {
    
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_key_read_data: Symbol<PclKeyReadDataFn> = lib.get(b"pclKeyReadData")
            .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_key_read_data)
    }
}

#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_key_get_size functionality (Libloading)
fn load_pcl_key_get_size() -> Result<PclKeyGetSizeFn, ErrorCode> {
    
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_key_get_size: Symbol<PclKeyGetSizeFn> = lib.get(b"pclKeyGetSize")
            .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_key_get_size)
    }
}

#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_key_write_data functionality (Libloading)
fn load_pcl_key_write_data() -> Result<PclKeyWriteDataFn, ErrorCode> {
    
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_key_write_data: Symbol<PclKeyWriteDataFn> = lib.get(b"pclKeyWriteData")
            .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_key_write_data)
    }
}

#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_key_delete functionality (Libloading)
fn load_pcl_key_delete() -> Result<PclKeyDeleteFn, ErrorCode> {
    
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_key_delete: Symbol<PclKeyDeleteFn> = lib.get(b"pclKeyDelete")
            .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_key_delete)
    }
}

#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_file_close functionality (Libloading)
fn load_pcl_file_close() -> Result<PclFileCloseFn, ErrorCode> {
    
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_file_close: Symbol<PclFileCloseFn> = lib.get(b"pclFileClose")
            .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_file_close)
    }
}

#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_file_get_size functionality (Libloading)
fn load_pcl_file_get_size() -> Result<PclFileGetSizeFn, ErrorCode> {
    
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_file_get_size: Symbol<PclFileGetSizeFn> = lib.get(b"pclFileGetSize")
            .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_file_get_size)
    }
}

#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_file_open functionality (Libloading)
fn load_pcl_file_open() -> Result<PclFileOpenFn, ErrorCode> {
    
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_file_open: Symbol<PclFileOpenFn> = lib.get(b"pclFileOpen")
            .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_file_open)
    }
}

#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_file_read_data functionality (Libloading)
fn load_pcl_file_read_data() -> Result<PclFileReadDataFn, ErrorCode> {
    
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_file_read_data: Symbol<PclFileReadDataFn> = lib.get(b"pclFileReadData")
            .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_file_read_data)
    }
}

#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_file_write_data functionality (Libloading)
fn load_pcl_file_write_data() -> Result<PclFileWriteDataFn, ErrorCode> {
    
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_file_write_data: Symbol<PclFileWriteDataFn> = lib.get(b"pclFileWriteData")
            .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_file_write_data)
    }
}

#[doc(hidden)]
/// Method that dynamically provides the corresponding Persistence Client Library (C) pcl_file_remove functionality (Libloading)
fn load_pcl_file_remove() -> Result<PclFileRemoveFn, ErrorCode> {
    let lib_lock = LIB.lock().unwrap();
    let lib = lib_lock.as_ref().ok_or(ErrorCode::LibraryNotLoaded)?;

    unsafe {
        let pcl_file_remove: Symbol<PclFileRemoveFn> = lib.get(b"pclFileRemove")
            .map_err(|_| ErrorCode::FunctionNotFound)?;
        Ok(*pcl_file_remove)
    }
}



/// Initializes the library with the given application name and shutdown mode.
/// 
/// Features:
///   * Initializes the library only if it's not already loaded.
///   * Executes the initialization through the C API.
///
/// Parameter:
///   * `appname`: The application name as a byte slice.
pub fn init_library(appname: &str) -> Result<(), ErrorCode> {

    if LIB.lock().unwrap().is_none() {
        load_library().map_err(|_| {
            ErrorCode::LoadLibraryFailed
        })?;

    }
    let pcl_init_library = load_pcl_init_library()?;

    let c_appname = CString::new(appname).map_err(|_| ErrorCode::DatatypeConversionFailed)?;

    unsafe {
        let rval = pcl_init_library(c_appname.as_ptr(), 0);
        if rval >= 0 {
            Ok(())
        } else {
            Err(rval.into()) 
        }
    }
}

/// Deinitializes the library.
/// 
/// Features:
///   * Deinitializes the library if it was previously initialized.
///   * Handles C API call for deinitialization.
/// 
/// Return:
///   * `Ok(())` if deinitialization succeeds.
///   * `Err(Errorcode::<CODE>)` if deinitialization fails, with the error code.
pub fn deinit_library() -> Result<(), ErrorCode> {

    if LIB.lock().unwrap().is_none() {
        load_library().map_err(|_| {
            ErrorCode::LoadLibraryFailed 
        })?;
    }
    let pcl_deinit_library = load_pcl_deinit_library()?;

    unsafe {
        let rval = pcl_deinit_library();
        if rval >= 0 {
            Ok(())
        } else {
            Err(rval.into()) 
        }
    }
}

/*-----------------------Library Functions-----------------------*/
/*-------Key Handling Functions-------*/

/// Reads a key
///
/// Features:
///   * Reads the key data through the C API.
///
/// Parameters:
///   * `ldbid`: The database ID.
///   * `resource_id`: The ID of the resource to read.
///   * `user_no`: The user number.
///
/// Return:
///   * `Ok(buffer)` containing the key data if the read is successful.
///   * `Err(Errorcode::<CODE>)` with the error code if the read fails.
pub fn read_key(ldbid: u32, resource_id: &str, user_no: u32) -> Result<Vec<u8>, ErrorCode> {

    let size = get_key_size(ldbid, resource_id, user_no)? as i32;
    let mut buffer = vec![0u8; (size) as usize];
    let pcl_key_read_data = load_pcl_key_read_data()?;

    let c_resource_id = CString::new(resource_id).map_err(|_| ErrorCode::DatatypeConversionFailed)?;

    unsafe {
        let rval = pcl_key_read_data(
            ldbid,
            c_resource_id.as_ptr(),
            user_no,
            0, /*seat_no:  Jump over Seat-No from C-API*/
            buffer.as_mut_ptr(),
            size,
        );

        if rval >= 0 {
            Ok(buffer)
        } else {
            Err(rval.into())
        }
    }
}

/// Gets the size of a key
///
/// Features:
///   * Calls the C API to fetch the key size.
///
/// Parameters:
///   * `ldbid`: The database ID.
///   * `resource_id`: The ID of the resource.
///   * `user_no`: The user number.
///
/// Return:
///   * `Ok(rval)` containing the key size if successful.
///   * `Err(Errorcode::<CODE>)` with the error code if the operation fails.
pub fn get_key_size(ldbid: u32, resource_id: &str, user_no: u32) -> Result<i32, ErrorCode> {
    
    let pcl_key_get_size = load_pcl_key_get_size()?;
    let c_resource_id = CString::new(resource_id).map_err(|_| ErrorCode::DatatypeConversionFailed)?;

    unsafe {
        let rval = pcl_key_get_size(
            ldbid, 
            c_resource_id.as_ptr(), 
            user_no, 
            0 /*seat_no:  Jump over Seat-No from C-API*/
        );
        if rval >= 0 {
            Ok(rval)
        } else {
            Err(rval.into())
        }
    }
}

/// Writes a key
///
/// Features:
///   * Writes the key data through the C API.
///
/// Parameters:
///   * `ldbid`: The database ID.
///   * `resource_id`: The ID of the resource to write.
///   * `user_no`: The user number.
///   * `buffer`: The data buffer to write.
///
/// Return:
///   * `Ok(())` if successful.
///   * `Err(Errorcode::<CODE>)` with the error code if the write fails.
pub fn write_key(ldbid: u32, resource_id: &str, user_no: u32, buffer: Vec<u8>) -> Result<(), ErrorCode> {

    let pcl_key_write_data = load_pcl_key_write_data()?;
    let c_resource_id = CString::new(resource_id).map_err(|_| ErrorCode::DatatypeConversionFailed)?;

    unsafe {
        let rval = pcl_key_write_data(
            ldbid, 
            c_resource_id.as_ptr(), 
            user_no, 
            0, /*seat_no:  Jump over Seat-No from C-API*/
            buffer.as_ptr(), 
            buffer.len() as i32
        );
        if rval >= 0 {
            Ok(())
        } else {
            Err(rval.into())
        }
    }
}

/// Deletes a key
///
/// Features:
///   * Deletes the key through the C API.
///
/// Parameters:
///   * `ldbid`: The database ID.
///   * `resource_id`: The ID of the resource to delete.
///   * `user_no`: The user number.
///
/// Return:
///   * `Ok(())` if successful.
///   * `Err(Errorcode::<CODE>)` with the error code if the delete operation fails.
pub fn delete_key(ldbid: u32, resource_id: &str, user_no: u32) -> Result<(), ErrorCode> {
    
    let pcl_key_delete = load_pcl_key_delete()?;
    let c_resource_id = CString::new(resource_id).map_err(|_| ErrorCode::DatatypeConversionFailed)?;

    unsafe {
        let rval = pcl_key_delete(
            ldbid, 
            c_resource_id.as_ptr(), 
            user_no, 
            0 /*seat_no:  Jump over Seat-No from C-API*/
        );
        if rval >= 0 {
            Ok(())
        } else {
            Err(rval.into())
        }
    }
}

/*-------File Handling Functions-------*/
/// Reads data from a file
///
/// Features:
///   * Reads the file data based on the provided file descriptor.
///   * Uses the C API to read the file into a buffer.
///
/// Parameters:
///   * `fd`: The file descriptor to read from.
///
/// Return:
///   * `Ok(buffer)` containing the file data if successful.
///   * `Err(Errorcode::<CODE>)` with the error code if the read operation fails.
pub fn read_file(fd: i32) -> Result<Vec<u8>, ErrorCode> {

    let size = get_file_size(fd)? as i32;
    let mut buffer = vec![0u8; size as usize];
    let pcl_file_read_data = load_pcl_file_read_data()?;

    unsafe {
        let rval = pcl_file_read_data(
            fd,
            buffer.as_mut_ptr(),
            size,
        );

        if rval >= 0 {
            Ok(buffer)
        } else {
            Err(rval.into())
        }
    }
}

/// Gets the size of a file
///
/// Features:
///   * Uses the C API to fetch the file size based on the provided file descriptor..
///
/// Parameters:
///   * `fd`: The file descriptor.
///
/// Return:
///   * `Ok(rval)` containing the file size if successful.
///   * `Err(Errorcode::<CODE>)` with the error code if the operation fails.
pub fn get_file_size(fd: i32) -> Result<i32, ErrorCode> {
    
    let pcl_file_get_size = load_pcl_file_get_size()?;
    
    unsafe {
        let rval = pcl_file_get_size(fd);
        if rval >= 0 {
            Ok(rval)
        } else {
            Err(rval.into())
        }
    }
}

/// Opens a file in the library
///
/// Features:
///   * Uses the C API to open the file and return a file descriptor.
///
/// Parameters:
///   * `ldbid`: The database ID.
///   * `resource_id`: The ID of the resource to open.
///   * `user_no`: The user number.
///
/// Return:
///   * `Ok(fd)` containing the file descriptor if successful.
///   * `Err(Errorcode::<CODE>)` with the error code if the operation fails.
pub fn open_file(ldbid: u32, resource_id: &str, user_no: u32) -> Result<i32, ErrorCode> {
    
    let pcl_file_open = load_pcl_file_open()?;
    let c_resource_id = CString::new(resource_id).map_err(|_| ErrorCode::DatatypeConversionFailed)?;

    unsafe {
        let fd = pcl_file_open(
            ldbid,
            c_resource_id.as_ptr(),
            user_no,
            0, /*seat_no:  Jump over Seat-No from C-API*/
        );
        if fd >= 0 {
            Ok(fd)
        } else {
            Err(fd.into()) /*If fd < 0, it contains an error value */
        }
    }
}

/// Writes data to a file
///
/// Features:
///   * Writes the provided buffer data to the file associated with the given file descriptor.
///   * Uses the C API to perform the file write operation.
///
/// Parameters:
///   * `fd`: The file descriptor to write to.
///   * `buffer`: The data buffer to write to the file.
///
/// Return:
///   * `Ok(())` if the write is successful.
///   * `Err(Errorcode::<CODE>)` with the error code if the write operation fails.
pub fn write_file(fd: i32, buffer: Vec<u8>) -> Result<(), ErrorCode> {
    
    let pcl_file_write_data = load_pcl_file_write_data()?;

    unsafe {
        let rval = pcl_file_write_data(
            fd,
            buffer.as_ptr(),
            buffer.len() as i32,
        );
        if rval >= 0 {
            Ok(())
        } else {
            Err(rval.into())
        }
    }
}

/// Closes a file
///
/// Features:
///   * Uses the C API to close the file with the given file descriptor.
///
/// Parameters:
///   * `fd`: The file descriptor to close.
///
/// Return:
///   * `Ok(())` if the file is closed successfully.
///   * `Err(Errorcode::<CODE>)` with the error code if the close operation fails.
pub fn close_file(fd: i32) -> Result<(), ErrorCode> {
    
    let pcl_file_close = load_pcl_file_close()?;

    unsafe {
        let rval = pcl_file_close(fd);
        if rval >= 0 {
            Ok(())
        } else {
            Err(rval.into())
        }
    }
}

/// Removes a file
///
/// Features:
///   * Uses the C API to remove the file with a specific resource and user.
///
/// Parameters:
///   * `ldbid`: The database ID.
///   * `resource_id`: The ID of the resource to remove.
///   * `user_no`: The user number.
///
/// Return:
///   * `Ok(())` if the file is removed successfully.
///   * `Err(Errorcode::<CODE>)` with the error code if the remove operation fails.
pub fn remove_file(ldbid: u32, resource_id: &str, user_no: u32) -> Result<(), ErrorCode> {
    
    let pcl_file_remove = load_pcl_file_remove()?;
    let c_resource_id = CString::new(resource_id).map_err(|_| ErrorCode::DatatypeConversionFailed)?;

    unsafe {
        let rval = pcl_file_remove(
            ldbid,
            c_resource_id.as_ptr(),
            user_no,
            0 /*seat_no:  Jump over Seat-No from C-API*/
        );
        if rval >= 0 {
            Ok(())
        } else {
            Err(rval.into())
        }
    }
}
