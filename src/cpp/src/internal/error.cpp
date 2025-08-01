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
#include "error.hpp"

namespace score::mw::pers::kvs {

/*********************** Error Implementation *********************/
std::string_view MyErrorDomain::MessageFor(score::result::ErrorCode const& code) const noexcept
{
    std::string_view msg;
    switch (static_cast<ErrorCode>(code))
    {
        case ErrorCode::UnmappedError:
            msg = "Error that was not yet mapped";
            break;
        case ErrorCode::FileNotFound:
            msg = "File not found";
            break;
        case ErrorCode::KvsFileReadError:
            msg = "KVS file read error";
            break;
        case ErrorCode::KvsHashFileReadError:
            msg = "KVS hash file read error";
            break;
        case ErrorCode::JsonParserError:
            msg = "JSON parser error";
            break;
        case ErrorCode::JsonGeneratorError:
            msg = "JSON generator error";
            break;
        case ErrorCode::PhysicalStorageFailure:
            msg = "Physical storage failure";
            break;
        case ErrorCode::IntegrityCorrupted:
            msg = "Integrity corrupted";
            break;
        case ErrorCode::ValidationFailed:
            msg = "Validation failed";
            break;
        case ErrorCode::EncryptionFailed:
            msg = "Encryption failed";
            break;
        case ErrorCode::ResourceBusy:
            msg = "Resource is busy";
            break;
        case ErrorCode::OutOfStorageSpace:
            msg = "Out of storage space";
            break;
        case ErrorCode::QuotaExceeded:
            msg = "Quota exceeded";
            break;
        case ErrorCode::AuthenticationFailed:
            msg = "Authentication failed";
            break;
        case ErrorCode::KeyNotFound:
            msg = "Key not found";
            break;
        case ErrorCode::KeyDefaultNotFound:
            msg = "Key default value not found";
            break;
        case ErrorCode::SerializationFailed:
            msg = "Serialization failed";
            break;
        case ErrorCode::InvalidSnapshotId:
            msg = "Invalid snapshot ID";
            break;
        case ErrorCode::ConversionFailed:
            msg = "Conversion failed";
            break;
        case ErrorCode::MutexLockFailed:
            msg = "Mutex failed";
            break;
        case ErrorCode::InvalidValueType:
            msg = "Invalid value type";
            break;
        default:
            msg = "Unknown Error!";
            break;
    }

    return msg;
}

score::result::Error MakeError(ErrorCode code, std::string_view user_message) noexcept
{
    return {static_cast<score::result::ErrorCode>(code), my_error_domain, user_message};
}

} /* namespace score::mw::pers::kvs */
