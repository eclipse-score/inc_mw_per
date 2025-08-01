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
#ifndef SCORE_LIB_KVS_KVSBUILDER_HPP
#define SCORE_LIB_KVS_KVSBUILDER_HPP

#include <string>
#include "kvs.hpp"

namespace score::mw::pers::kvs {

/**
 * @class KvsBuilder
 * @brief Builder for opening a KVS object.
 * This class allows configuration of various options for opening a KVS instance,
 * such as whether default values are required, whether the KVS data must already exist.
 * 
 * Important: You don't need to include any other header files to use the KVS.
 * For documentation of the KVS Functions, refer to the kvs.hpp documentation.
 * 
 * \brief Example Usage
 * \code
 *  #include <iostream>
 *  #include "kvsbuilder.hpp"
 *
 *  int main() {
 *    // Open kvs
 *    auto open_res = KvsBuilder(0)
 *                        .need_defaults_flag(true)
 *                        .need_kvs_flag(true)
 *                        .build();
 *    if (!open_res) return 1;
 *    Kvs kvs = std::move(open_res.value());
 *
 *    // Set and get a value
 *    kvs.set_value("pi", KvsValue(3.14));
 *    auto get_res = kvs.get_value("pi");
 *
 *    // Delete a key
 *    kvs.remove_key("pi");
 *    std::cout << "has pi? " << (kvs.key_exists("pi").value_or(false) ? "yes" : "no") << "\n";
 *
 *
 *    return 0;
 *  }
 * \endcode
 */
class KvsBuilder final {
public:
    /**
     * @brief Constructs a KvsBuilder for the given KVS instance.
     * @param instance_id Unique identifier for the KVS instance.
     */
    explicit KvsBuilder(const InstanceId& instance_id);

    /**
     * @brief Specify whether default values must be loaded.
     * @param flag True to require default values; false to make them optional.
     * @return Reference to this builder (for chaining).
     */
    KvsBuilder& need_defaults_flag(bool flag);

    /**
     * @brief Configure if KVS must exist when opening the KVS.
     * @param flag True to require an existing store; false to allow starting empty.
     * @return Reference to this builder (for chaining).
     */
    KvsBuilder& need_kvs_flag(bool flag);

    /**
     * @brief Specify the directory where KVS files are stored.
     * @param dir The directory path as a string.
     * Use "" or "." for the current directory.
     *
     * @return Reference to this builder (for chaining).
     */
    KvsBuilder& dir(std::string&& dir_path);

    /**
     * @brief Builds and opens the Kvs instance with the configured options.
     *
     * Internally calls Kvs::open() with the selected flags and directory.
     *
     * @return A score::Result<Kvs> containing the opened store or an ErrorCode.
     */
    score::Result<Kvs> build();

private:
    InstanceId                         instance_id;   ///< ID of the KVS instance
    bool                               need_defaults; ///< Whether default values are required
    bool                               need_kvs;      ///< Whether an existing KVS is required
    std::string                        directory;     ///< Directory where to store the KVS Files
};

} /* namespace score::mw::pers::kvs */

#endif /* SCORE_LIB_KVS_KVSBUILDER_HPP */
