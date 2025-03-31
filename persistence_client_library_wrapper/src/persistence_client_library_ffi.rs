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
//! # This Foreign Function Interface provides the possibility to call the C-functions from the Persistence Client Library.
//!

use std::os::raw::c_char;

/* Define external c functions */

/* ----Library Functions--- */
pub type PclInitLibraryFn = unsafe extern "C" fn(
    appname: *const c_char,
    shutdown_mode: i32,
) -> i32;

pub type PclDeinitLibraryFn = unsafe extern "C" fn(
) -> i32;

/* ----Key Handling Functions--- */
pub type PclKeyGetSizeFn = unsafe extern "C" fn(
    ldbid: u32,
    resource_id: *const c_char,
    user_no: u32,
    seat_no: u32,
) -> i32;

pub type PclKeyReadDataFn = unsafe extern "C" fn(
    ldbid: u32,
    resource_id: *const c_char,
    user_no: u32,
    seat_no: u32,
    buffer: *mut u8,
    buffer_size: i32,
) -> i32;

pub type PclKeyWriteDataFn = unsafe extern "C" fn(
    ldbid: u32,
    resource_id: *const c_char,
    user_no: u32,
    seat_no: u32,
    buffer: *const u8,
    buffer_size: i32,
) -> i32;

pub type PclKeyDeleteFn = unsafe extern "C" fn(
    ldbid: u32,
    resource_id: *const c_char,
    user_no: u32,
    seat_no: u32,
) -> i32;

/* ----File Handling Functions--- */
pub type PclFileCloseFn = unsafe extern "C" fn(fd: i32) -> i32;

pub type PclFileGetSizeFn = unsafe extern "C" fn(fd: i32) -> i32;

pub type PclFileOpenFn = unsafe extern "C" fn(
    ldbid: u32,
    resource_id: *const c_char,
    user_no: u32,
    seat_no: u32,
) -> i32;

pub type PclFileReadDataFn = unsafe extern "C" fn(
    fd: i32,
    buffer: *mut u8,
    buffer_size: i32,
) -> i32;

pub type PclFileWriteDataFn = unsafe extern "C" fn(
    fd: i32,
    buffer: *const u8,
    buffer_size: i32,
) -> i32;

pub type PclFileRemoveFn = unsafe extern "C" fn(
    ldbid: u32, 
    resource_id: *const c_char,
    user_no: u32, 
    seat_no: u32
) -> i32;

