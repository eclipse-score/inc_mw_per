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
#ifndef SCORE_LIB_KVS_INTERNAL_ERROR_HPP
#define SCORE_LIB_KVS_INTERNAL_ERROR_HPP

#include "score/result/result.h"

namespace score::mw::per::kvs {

/* @brief */
enum class ErrorCode : score::result::ErrorCode {
    /* Error that was not yet mapped*/
    UnmappedError,

    /* File not found*/
    FileNotFound,

    /* KVS file read error*/
    KvsFileReadError,

    /* KVS hash file read error*/
    KvsHashFileReadError,

    /* JSON parser error*/
    JsonParserError,

    /* JSON generator error*/
    JsonGeneratorError,

    /* Physical storage failure*/
    PhysicalStorageFailure,

    /* Integrity corrupted*/
    IntegrityCorrupted,

    /* Validation failed*/
    ValidationFailed,

    /* Encryption failed*/
    EncryptionFailed,

    /* Resource is busy*/
    ResourceBusy,

    /* Out of storage space*/
    OutOfStorageSpace,

    /* Quota exceeded*/
    QuotaExceeded,

    /* Authentication failed*/
    AuthenticationFailed,

    /* Key not found*/
    KeyNotFound,
    
    /* Key default value not found*/
    KeyDefaultNotFound,

    /* Serialization failed*/
    SerializationFailed,

    /* Invalid snapshot ID*/
    InvalidSnapshotId,

    /* Conversion failed*/
    ConversionFailed,

    /* Mutex failed*/
    MutexLockFailed,

    /* Invalid value type*/
    InvalidValueType,
};

class MyErrorDomain final : public score::result::ErrorDomain
{
public:
    std::string_view MessageFor(score::result::ErrorCode const& code) const noexcept override;
};

constexpr MyErrorDomain my_error_domain;
score::result::Error MakeError(ErrorCode code, std::string_view user_message = "") noexcept;

} /* namespace score::mw::per::kvs */

#endif /* SCORE_LIB_KVS_INTERNAL_ERROR_HPP */
