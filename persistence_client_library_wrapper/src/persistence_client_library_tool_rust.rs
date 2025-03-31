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
//! # Key-Value and File Handling Command Line Tool based on the Genivi Persistence Client Library Rust Wrapper
//!
//! ## Introduction
//!
//! This Command Line Tool provides Key-Value and File Handling using the Persistence Client Library Rust Wrapper (`FEAT_REQ__KVS__tooling`).
//! For Command Line Argument parsing the crate [clap](https://docs.rs/clap/latest/clap/) is used. 
//! No other direct dependencies are used besides the Rust `std` library.
//! This tool and its syntax is inspired by the [persistence client library tool](https://github.com/GENIVI/persistence-client-library/blob/master/tools/persistence_client_tool.c) from Genivi.
//! 
//! 
//! ## Example Usage
//!
//! ```text
//! 1.) Read a Key and show value as HexDump:
//! persistence_client_tool_rust -o readkey -a MyApplication -r MyKey                     optional parameters: [-l 0xFF -u 0]
//! 
//! 2.) Write a Key and use the <payload> as the data source:
//! persistence_client_tool_rust -o writekey -a MyApplication -r MyKey -p 'Hello World'   optional parameters: [-l 0xFF -u 0]
//! 
//! 3.) Get the size of a key [bytes]:
//! persistence_client_tool_rust -o getkeysize -a MyApplication -r MyKey                  optional parameters: [-l 0xFF -u 0]
//! 
//! 4.) Delete a key:
//! persistence_client_tool_rust -o deletekey -a MyApplication -r MyKey                   optional parameters: [-l 0xFF -u 0]
//! 
//! 5.) Read a File and show value as HexDump:
//! persistence_client_tool_rust -o readfile -a MyApplication -r MyFile                   optional parameters: [-l 0xFF -u 0]
//! 
//! 6.) Write a File and use the <payload> as the data source:
//! persistence_client_tool_rust -o writefile -a MyApplication -r MyFile -p 'Hello World' optional parameters: [-l 0xFF -u 0]
//! 
//! 7.) Get the size of a File [bytes]:
//! persistence_client_tool_rust -o getfilesize -a MyApplication -r MyFile                optional parameters: [-l 0xFF -u 0]
//! 
//! 8.) Delete a File:
//! persistence_client_tool_rust -o deletefile -a MyApplication -r MyFile                 optional parameters: [-l 0xFF -u 0]
//! ```
//! 

use persistence_client_library_wrapper::*;
use clap::*;
use std::process::exit;

/// Defines the available operation modes for key and file management.
enum OperationMode {
    Invalid,
    ReadKey,
    WriteKey,
    DeleteKey,
    GetKeySize,
    ReadFile,
    WriteFile,
    DeleteFile,
    GetFileSize
}
/*--------------------Key Handling--------------------*/
/// Reads data from a key
///
/// Behavior:
///   * Prints the `resource_id` in readable ASCII form (non-ASCII as `.`).
///   * Displays which key is being read, including `user_no` and `ldbid`.
///   * Calls the `read_key()` function from the wrapper.
///   * Prints the key data in a hex dump format using `print_buffer()`.
fn _read_key(ldbid: u32, resource_id: &str, user_no: u32) {
    print!("Resource_id: ");
    for c in resource_id.chars() {
        print!("{}", if c.is_ascii() { c } else { '.' });
    }
    println!();  
    println!("Reading key with resource_id: user_no: {}, ldbid: {:X}",user_no, ldbid);
    
    let buffer = match read_key(ldbid, resource_id, user_no) {
        Ok(buffer) => buffer,
        Err(err) => {
            println!("Failed to read key! Error: {:?}", err);
            return;
        }
    };
    println!("Key Data:");
    print_buffer(&buffer);

}

/// Writes a key
///
/// Behavior:
///   * Prints the `resource_id` in readable ASCII form (non-ASCII as `.`).
///   * Displays which key is being written, including `user_no` and `ldbid`.
///   * Calls the `write_key()` function from the wrapper.
///   * Prints success or error message depending on the result.
///   * Shows the contents of the `buffer` using `print_buffer()`.
fn _write_key(ldbid: u32, resource_id: &str, user_no: u32, buffer: Vec<u8>) {
    print!("Resource_id: ");
    for c in resource_id.chars() {
        print!("{}", if c.is_ascii() { c } else { '.' });
    }
    println!(); 
    println!("Writing key: user_no: {}, ldbid: {:X}", user_no, ldbid);
    print_buffer(&buffer);

    if let Err(err) = write_key(ldbid, resource_id, user_no, buffer) {
        println!("Failed to write key! Error: {:?}", err);
        return;
    }
    println!("Key successfully written!");

}

/// Deletes a key
///
/// Behavior:
///   * Prints the `resource_id` in readable ASCII form (non-ASCII as `.`).
///   * Displays which key is being deleted, including `user_no` and `ldbid`.
///   * Calls the `delete_key()` function from the wrapper.
///   * Prints success or error message depending on the result.
fn _delete_key( ldbid: u32, resource_id: &str, user_no: u32) {
    print!("Resource_id: ");
    for c in resource_id.chars() {
        print!("{}", if c.is_ascii() { c } else { '.' });
    }
    println!();  
    println!("Deleting key with resource_id: user_no: {}, ldbid: {:X}",user_no, ldbid);

    if let Err(err) = delete_key(ldbid, resource_id, user_no) {
        println!("Failed to delete key! Error: {:?}", err);
        return;
    }
    println!("Key successfully deleted!");
}

/// Determines the size of a key
///
/// Behavior:
///   * Prints the `resource_id` in readable ASCII form (non-ASCII as `.`).
///   * Displays which key is being queried, including `user_no` and `ldbid`.
///   * Retrieves the key size using the`get_key_size()` function from the wrapper.
///   * Prints the key size or error message.
fn _get_key_size(ldbid: u32, resource_id: &str, user_no: u32) {
    print!("Resource_id: ");
    for c in resource_id.chars() {
        print!("{}", if c.is_ascii() { c } else { '.' });
    }
    println!(); 
    println!("Getting key size for resource_id: user_no: {},  ldbid: {:X}", user_no, ldbid);
    
    let size = match get_key_size(ldbid, resource_id, user_no) {
        Ok(size) => size,
        Err(err) => {
            println!("Failed to get key size! Return Value: {:?}", err);
            return;
        }
    };
    println!("Key size: {}", size);
}


/*--------------------File Handling--------------------*/
/// Reads data from a file
///
/// Behavior:
///   * Prints the `resource_id` in readable ASCII form (non-ASCII as `.`).
///   * Displays which file is being read, including `user_no` and `ldbid`.
///   * Opens the file using the `open_file()` function from the wrapper.
///   * Reads the file content using the `read_file()` function from the wrapper..
///   * Prints success or error messages.
///   * Closes the file using the`close_file()` function from the wrapper.
///   * Dumps the file content using `print_buffer()`.
fn _read_file(ldbid: u32, resource_id: &str, user_no: u32){
    print!("Resource_id: ");
    for c in resource_id.chars() {
        print!("{}", if c.is_ascii() { c } else { '.' });
    }
    println!(); 
    println!("Reading file: user_no: {}, ldbid: {:X}", user_no, ldbid);

    let fd = match open_file(ldbid, resource_id, user_no) {
        Ok(fd) => fd,
        Err(err) => {
            println!("Failed to open file! Error: {:?}", err);
            return;
        }
    };

    let file_data = match read_file(fd) {
        Ok(data) => data,
        Err(err) => {
            println!("Failed to read file! Error: {:?}", err);
            return;
        }
    };

    println!("File successfully read!");
    if let Err(err) = close_file(fd) {
        println!("Failed to close file! Error: {:?}", err);
    }
    
    print_buffer(&file_data);
}

/// Determines the size of a file
///
/// Behavior:
///   * Prints the `resource_id` in readable ASCII form (non-ASCII as `.`).
///   * Displays target info (`user_no`, `ldbid`).
///   * Opens the file using the `open_file()` function from the wrapper.
///   * Gets the file size using the `get_file_size()` function from the wrapper.
///   * Closes the file using the `close_file()` function from the wrapper.
///   * Prints the file size or error messages.
fn _get_file_size(ldbid: u32, resource_id: &str, user_no: u32){
    print!("Resource_id: ");
    for c in resource_id.chars() {
        print!("{}", if c.is_ascii() { c } else { '.' });
    }
    println!(); 
    println!("Getting file size for resource_id: user_no: {}, ldbid: {:X}", user_no, ldbid);

    let fd = match open_file(ldbid, resource_id, user_no) {
        Ok(fd) => fd,
        Err(err) => {
            println!("Failed to open file! Error: {:?}", err);
            return;
        }
    };

    let size = match get_file_size(fd) {
        Ok(size) => size,
        Err(err) => {
            println!("Failed to get file size! Error: {:?}", err);
            return;
        }
    };

    println!("File size: {}", size);
    if let Err(err) = close_file(fd) {
        println!("Failed to close file! Error: {:?}", err);
    }
}

/// Writes data to a file
///
/// Behavior:
///   * Prints the `resource_id` in readable ASCII form (non-ASCII as `.`).
///   * Displays target info (`user_no`, `ldbid`) and buffer content using `print_buffer()`.
///   * Opens the file using the `open_file()` function from the wrapper.
///   * Writes `buffer` content using the `write_file()` function from the wrapper.
///   * Closes the file using the `close_file()` function from the wrapper.
///   * Prints success or error messages for each step.
fn _write_file(ldbid: u32, resource_id: &str, user_no: u32, buffer: Vec<u8>) {
    print!("Resource_id: ");
    for c in resource_id.chars() {
        print!("{}", if c.is_ascii() { c } else { '.' });
    }
    println!(); 
    println!("Writing to file: user_no: {}, ldbid: {:X}", user_no, ldbid);
    print_buffer(&buffer);

    let fd = match open_file(ldbid, resource_id, user_no) {
        Ok(fd) => fd,
        Err(err) => {
            println!("Failed to open file! Error: {:?}", err);
            return;
        }
    };

    if let Err(err) = write_file(fd, buffer) {
        println!("Failed to write to file! Error: {:?}", err);
    } else {
        println!("File successfully written!");
    }

    if let Err(err) = close_file(fd) {
        println!("Failed to close file! Error: {:?}", err);
    }
}

/// Removes a file
///
/// Behavior:
///   * Prints the `resource_id` in readable ASCII form (non-ASCII as `.`).
///   * Displays which file is being removed, including `user_no` and `ldbid`.
///   * Calls the `remove_file()` function from the wrapper.
///   * Prints success or error message depending on the result.
fn _remove_file(ldbid: u32, resource_id: &str, user_no: u32) {
    print!("Resource_id: ");
    for c in resource_id.chars() {
        print!("{}", if c.is_ascii() { c } else { '.' });
    }
    println!(); 
    println!("Removing file: user_no: {}, ldbid: {:X}", user_no, ldbid);

    if let Err(err) = remove_file(ldbid, resource_id, user_no) {
        println!("Failed to remove file! Error: {:?}", err);
        return;
    }
    println!("File successfully removed!");
}

/*--------------------Print Buffer--------------------*/
/// Prints the contents of a buffer as a hex dump and ASCII representation.
///
/// Arguments:
///   * `buffer`: The data buffer to print.
///
/// Output:
///   * Prints a formatted hex dump to the console.
///   * Prints the corresponding ASCII output (non-printable characters are replaced by `.`).
///
/// Details:
///   * If the buffer is smaller than 120 bytes, 8 bytes are shown per row.
///   * If the buffer is 120 bytes or larger, 16 bytes are shown per row.
///   * Each row shows the offset and the bytes in hexadecimal.
///   * After the hex dump, an ASCII representation of the buffer is printed.
fn print_buffer(buffer: &Vec<u8>) {
    let len = buffer.len();
    let mut num_per_row = 8;
    
    if len >= 120 {
        num_per_row = 16;
    }

    if len != 0 {
        let mut j = 0;
        println!("\nHEXDUMP:");
        println!("---------------------------------------------------------");
        print!("[{:3}] - ", num_per_row * j);

        for (i, &byte) in buffer.iter().enumerate() {
            print!("{:02x} ", byte);
            if (i + 1) % num_per_row == 0 {
                j += 1;
                println!();
                print!("[{:3}] - ", num_per_row * j);
            }
        }
        println!();
        println!("---------------------------------------------------------\n");

       
        println!("---------------------------------------------------------");
        println!("ASCII Output:");
        for &byte in buffer.iter() {
            print!("{}", if byte >= 32 && byte <= 126 { byte as char } else { '.' });
        }
        println!("\n---------------------------------------------------------\n");
    }
}


/// The entry point of the CLI tool for interacting with the Persistence Client Library Wrapper.
///
/// Parses command-line arguments and dispatches to the appropriate operation.
///
/// Arguments:
///   * `-o`, `--operation`: Operation mode (readkey, writekey, deletekey, getkeysize, readfile, writefile, deletefile, getfilesize).
///   * `-a`, `--app_name`: Name of the application.
///   * `-r`, `--resource_id`: ID of the resource (key or file).
///   * `-p`, `--payload`: Payload to write (used for write operations).
///   * `-u`, `--user_no`: Optional user number (default is `0`).
///   * `-l`, `--ldbid`: Optional LDB ID in hexadecimal (default is `0xFF`).
///   * `-h`, `--help`: Prints manual on how to use the CLI Tool.
///
/// Description:
///   * Initializes the persistence library.
///   * Performs the specified operation.
///   * Deinitializes the library.
///
fn main() -> Result<(), ErrorCode> {
    const LONG_ABOUT: &str = "

    ---------------------------------------
    Persistence_Client_Library_Tool in Rust
    ---------------------------------------

    Version 1.0
    Author: Joshua Licht, Continental AG

    ---------------------------------------

    Usage Examples:
    1.) Read a Key and show value as HexDump:
        persistence_client_tool_rust -o readkey -a MyApplication -r MyKey                     optional parameters: [-l 0xFF -u 0]

    2.) Write a Key and use the <payload> as the data source:
        persistence_client_tool_rust -o writekey -a MyApplication -r MyKey -p 'Hello World'   optional parameters: [-l 0xFF -u 0]

    3.) Get the size of a key [bytes]:
        persistence_client_tool_rust -o getkeysize -a MyApplication -r MyKey                  optional parameters: [-l 0xFF -u 0]

    4.) Delete a key:
        persistence_client_tool_rust -o deletekey -a MyApplication -r MyKey                   optional parameters: [-l 0xFF -u 0]

    5.) Read a File and show value as HexDump:
        persistence_client_tool_rust -o readfile -a MyApplication -r MyFile                   optional parameters: [-l 0xFF -u 0]

    6.) Write a File and use the <payload> as the data source:
        persistence_client_tool_rust -o writefile -a MyApplication -r MyFile -p 'Hello World' optional parameters: [-l 0xFF -u 0]

    7.) Get the size of a File [bytes]:
        persistence_client_tool_rust -o getfilesize -a MyApplication -r MyFile                optional parameters: [-l 0xFF -u 0]

    8.) Delete a File:
        persistence_client_tool_rust -o deletefile -a MyApplication -r MyFile                 optional parameters: [-l 0xFF -u 0]
        
    ---------------------------------------
    ";

    let matches = Command::new("persistence_client_tool_rust")
        .version("1.0")
        .about("Persistence_Client_Library_Tool in Rust")
        .long_about(LONG_ABOUT)
        .arg(
            Arg::new("operation")
                .short('o')
                .long("operation")
                .help("Specify the operation mode")
        )
        .arg(
            Arg::new("app_name")
                .short('a')
                .long("app_name")
                .help("Application name"),
        )
        .arg(
            Arg::new("resource_id")
                .short('r')
                .long("resource_id")
                .help("Resource ID"),
        )
        .arg(
            Arg::new("payload")
                .short('p')
                .long("payload")
                .help("Payload to write"),
        )
        .arg(
            Arg::new("user_no")
                .short('u')
                .long("user_no")
                .help("User number"),
        )
        .arg(
            Arg::new("ldbid")
                .short('l')
                .long("ldbid")
                .help("LDB ID (hex)"),
        )
        .get_matches();


    /*Parse Data */
    let op_mode = match matches.get_one::<String>("operation") {
        Some(op) => match op.as_str() {
            "readkey" => OperationMode::ReadKey,
            "writekey" => OperationMode::WriteKey,
            "deletekey" => OperationMode::DeleteKey,
            "getkeysize" => OperationMode::GetKeySize,
            "readfile" => OperationMode::ReadFile,
            "writefile" => OperationMode::WriteFile,
            "deletefile" => OperationMode::DeleteFile,
            "getfilesize" => OperationMode::GetFileSize,
            _ => OperationMode::Invalid,
        },
        None => OperationMode::Invalid,};
    let app_name = matches.get_one::<String>("app_name");
    let resource_id = matches.get_one::<String>("resource_id");
    let payload = matches.get_one::<String>("payload");
    let user_no: Option<u32> = matches
        .get_one::<String>("user_no")
        .and_then(|s| s.parse::<u32>().ok()); 
    let ldbid: Option<u32> = matches
        .get_one::<String>("ldbid")
        .and_then(|s| u32::from_str_radix(s, 16).ok()); 


    /*Set Default Values for user_no and ldbid if empty*/
    let user_no = user_no.unwrap_or(0); 
    let ldbid = ldbid.unwrap_or(0xFF); 

    let app_name = app_name.unwrap_or_else(|| {
        println!("Application name is required");
        exit(1);
    });

    let resource_id = resource_id.unwrap_or_else(|| {
        println!("Resource ID is required");
        exit(1);
    });


    /* Initialize Library */
    init_library(app_name)?;

    match op_mode {
        OperationMode::ReadKey => {
            _read_key(ldbid, resource_id, user_no);
        }
        OperationMode::WriteKey => {
            let buffer = payload.as_ref().map_or("", |s| s.as_str()).as_bytes().to_vec();
            _write_key(ldbid, resource_id, user_no, buffer);
        }
        OperationMode::DeleteKey => {
            _delete_key(ldbid, resource_id, user_no);
        }
        OperationMode::GetKeySize => {
            _get_key_size(ldbid, resource_id, user_no);
        }
        OperationMode::ReadFile => {
            _read_file(ldbid, resource_id, user_no);
        }
        OperationMode::WriteFile => {
            let buffer = payload.as_ref().map_or("", |s| s.as_str()).as_bytes().to_vec();
            _write_file(ldbid, resource_id, user_no, buffer);
        }
        OperationMode::DeleteFile => {
            _remove_file(ldbid, resource_id, user_no);
        }
        OperationMode::GetFileSize => {
            _get_file_size(ldbid, resource_id, user_no);
        }
        OperationMode::Invalid => {
            println!("Unsupported operation mode");
            deinit_library()?;
        }
    }

    /* Deinitialize Library */
    deinit_library()?;
    Ok(())

}
   