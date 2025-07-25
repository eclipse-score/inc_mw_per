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

#ifndef SCORE_LIB_JSON_INTERNAL_MODEL_ERROR_H
#define SCORE_LIB_JSON_INTERNAL_MODEL_ERROR_H

#include <string_view>
#include <type_traits>
#include "score/result/error.h"
#include <score/string_view.hpp>
#include <score/utility.hpp>


namespace score {
namespace json {

/**
 * \brief JSON-specific error codes
 */
enum class Error : score::result::ErrorCode
{
    kUnknownError,
    kWrongType,
    kKeyNotFound,
    kParsingError,
    kInvalidFilePath,
};

/**
 * \brief ErrorDomain for score::json::Error
 */
class ErrorDomain final : public score::result::ErrorDomain
{
  public:
    /// Returns the human-readable message for an ErrorCode
    std::string_view MessageFor(const score::result::ErrorCode& code) const noexcept override
    {
        // Ensure that our ErrorCode type matches
        using ResultErrorCode = std::decay_t<decltype(code)>;
        static_assert(std::is_same_v<
          ResultErrorCode,
          std::underlying_type_t<score::json::Error>
        >, "ErrorCode mismatch");

        switch (code)
        {
            case score::cpp::to_underlying(Error::kWrongType):
                return "You tried to cast a Any JSON value into a type that it cannot be represented in!";
            case score::cpp::to_underlying(Error::kKeyNotFound):
                return "Your requested key was not found.";
            case score::cpp::to_underlying(Error::kParsingError):
                return "An error occurred during parsing.";
            case score::cpp::to_underlying(Error::kInvalidFilePath):
                return "The JSON file path is incorrect.";
            case score::cpp::to_underlying(Error::kUnknownError):
            default:
                return "Unknown Error";
        }
    }
};

/**
 * \brief Creates a score::result::Error object from a score::json::Error
 *
 * Inline to avoid requiring a separate .cpp file.
 */
inline score::result::Error MakeError(const score::json::Error code,
                                      const std::string_view user_message = {}) noexcept
{
    // Static domain object used once
    static const ErrorDomain domain{};
    return { static_cast<score::result::ErrorCode>(code),
             domain,
             user_message };
}

} // namespace json
} // namespace score

#endif // SCORE_LIB_JSON_INTERNAL_MODEL_ERROR_H
