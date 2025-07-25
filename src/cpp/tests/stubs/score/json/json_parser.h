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

#ifndef SCORE_LIB_JSON_JSON_PARSER_H
#define SCORE_LIB_JSON_JSON_PARSER_H

#include "score/json/internal/model/any.h"
#include "score/json/internal/model/error.h"
#include "score/result/result.h"

namespace score
{
namespace json
{
// Global control flags for unittests
extern bool g_JsonParserShouldFail; // Global flag to control failure of parsing
extern std::string g_JsonParserReceivedValue; // Global variable to store the received value for testing
extern score::json::Any g_JsonParserReturnValue; // Global variable to store the return value of the parser


class JsonParser
{
  public:
    /// \brief Parses the underlying buffer and creates a tree of JSON data
    /// \param buffer The string_view that shall be parsed
    /// \return root to the tree of JSON data, error on error
    score::Result<Any> FromBuffer(const std::string_view buffer){ // std::string_view is used instead of score ::string_view for easy std::string conversion (unittests only) 
        if (g_JsonParserShouldFail) {
            return score::MakeUnexpected(Error::kInvalidFilePath);
        }else{
            g_JsonParserReceivedValue = std::string(buffer); // Store the received value for testing purposes
            score::Result<Any> return_value;    
            return_value = g_JsonParserReturnValue;           
            return return_value;
        }
    }
};

/// \brief Parses the underlying buffer and creates a tree of JSON data
/// \return root to the tree of JSON data, on error will terminate
Any operator"" _json(const char* const data, const size_t size);

}  // namespace json
}  // namespace score

#endif  // SCORE_LIB_JSON_JSON_PARSER_H
