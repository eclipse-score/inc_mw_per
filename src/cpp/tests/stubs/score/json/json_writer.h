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

#ifndef SCORE_LIB_JSON_JSON_WRITER_H
#define SCORE_LIB_JSON_JSON_WRITER_H


#include "score/json/internal/model/any.h"
#include "score/json/internal/model/error.h"
#include "score/result/result.h"
#include <score/string_view.hpp>

namespace score
{
namespace json
{

extern bool g_JsonWriterShouldFail; /* Global flag to control failure of ToBuffer */
extern std::string g_JsonWriterReturnValue; /* Global return value for ToBuffer */
extern score::json::Object g_JsonWriterReceivedValue; /* Global variable to store the received value for testing */

class JsonWriter
{
  public:
    score::Result<std::string> ToBuffer(const score::json::Object& json_data){
        g_JsonWriterReceivedValue = json_data; // Store the received value for testing purposes
        if (g_JsonWriterShouldFail) {
            return score::MakeUnexpected(Error::kUnknownError);
        }else{
            // Simulate writing JSON data to a buffer
            return g_JsonWriterReturnValue; // Return the stubbed JSON string
        }
    }
};

}  // namespace json
}  // namespace score

#endif  // SCORE_LIB_JSON_JSON_PARSER_H
