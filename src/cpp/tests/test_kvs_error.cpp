/********************************************************************************
* Copyright (c) 2025 Contributors to the Eclipse Foundation
*
* See the NOTICE file(s) distributed with this work for additional
* information regarding copyright ownership.
*
* This program and the accompanying materials are made available under the
* terms of the Apache License Version 2.0 which is available at
* https://www.apache.org/licenses/LICENSE-2.0
*
* SPDX-License-Identifier: Apache-2.0
********************************************************************************/
#include "test_kvs_general.hpp"


TEST(kvs_MessageFor, MessageFor) {
   struct {
        ErrorCode code;
        std::string_view expected_message;
    } test_cases[] = {
        {ErrorCode::UnmappedError,          "Error that was not yet mapped"},
        {ErrorCode::FileNotFound,           "File not found"},
        {ErrorCode::KvsFileReadError,       "KVS file read error"},
        {ErrorCode::KvsHashFileReadError,   "KVS hash file read error"},
        {ErrorCode::JsonParserError,        "JSON parser error"},
        {ErrorCode::JsonGeneratorError,     "JSON generator error"},
        {ErrorCode::PhysicalStorageFailure, "Physical storage failure"},
        {ErrorCode::IntegrityCorrupted,     "Integrity corrupted"},
        {ErrorCode::ValidationFailed,       "Validation failed"},
        {ErrorCode::EncryptionFailed,       "Encryption failed"},
        {ErrorCode::ResourceBusy,           "Resource is busy"},
        {ErrorCode::OutOfStorageSpace,      "Out of storage space"},
        {ErrorCode::QuotaExceeded,          "Quota exceeded"},
        {ErrorCode::AuthenticationFailed,   "Authentication failed"},
        {ErrorCode::KeyNotFound,            "Key not found"},
        {ErrorCode::KeyDefaultNotFound,     "Key default value not found"},
        {ErrorCode::SerializationFailed,    "Serialization failed"},
        {ErrorCode::InvalidSnapshotId,      "Invalid snapshot ID"},
        {ErrorCode::ConversionFailed,       "Conversion failed"},
        {ErrorCode::MutexLockFailed,        "Mutex failed"},
        {ErrorCode::InvalidValueType,       "Invalid value type"},
    };
    for (const auto& test : test_cases) {
        SCOPED_TRACE(static_cast<int>(test.code));
        score::result::ErrorCode code = static_cast<score::result::ErrorCode>(test.code);
        EXPECT_EQ(my_error_domain.MessageFor(code), test.expected_message);
    }

    score::result::ErrorCode invalid_code = static_cast<score::result::ErrorCode>(9999);
    EXPECT_EQ(my_error_domain.MessageFor(invalid_code), "Unknown Error!");

}
