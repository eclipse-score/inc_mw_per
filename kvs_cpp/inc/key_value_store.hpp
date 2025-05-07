// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0

#include <string>
#include <unordered_map>
#include <mutex>
#include <atomic>
#include <variant>
#include <vector>

struct InstanceId {
    size_t id;
    new(size_t id){this->id = id;}
}
struct SnapshotId {
    size_t id;
    new(size_t id){this->id = id;}
}

// Need-Defaults flag
enum class OpenNeedDefaults{
    Optional, // Optional: Open defaults only if available
    Required // Required: Defaults must be available
}

// Need-KVS flag
enum class OpenNeedKvs {
    Optional, // Optional: Use an empty KVS if no KVS is available
    Required // Required: KVS must be already exist
}

// Define the KvsValue class
/**
 * @class KvsValue
 * @brief Represents a flexible value type that can hold various data types, 
 *        including numbers, booleans, strings, null, arrays, and objects.
 * 
 * The KvsValue class provides a type-safe way to store and retrieve values of 
 * different types. It uses a std::variant to hold the underlying value and an 
 * enum to track the type of the value.
 * 
 * ## Supported Types:
 * - Number (double)
 * - Boolean (bool)
 * - String (std::string)
 * - Null (std::nullptr_t)
 * - Array (std::vector<KvsValue>)
 * - Object (std::unordered_map<std::string, KvsValue>)
 * 
 * ## Public Methods:
 * - `KvsValue(double number)`: Constructs a KvsValue holding a number.
 * - `KvsValue(bool boolean)`:
 * - Access the underlying value using `getValue()` and `std::get`.
 *
 * ## Example:
 * @code
 * KvsValue numberValue(42.0);
 * KvsValue stringValue("Hello, World!");
 * KvsValue arrayValue(KvsValue::Array{numberValue, stringValue});
 *
 * if (numberValue.getType() == KvsValue::Type::Number) {
 *     double number = std::get<double>(numberValue.getValue());
 * }
 * @endcode
 */
class KvsValue {
public:
    // Define the possible types for KvsValue
    using Array = std::vector<KvsValue>;
    using Object = std::unordered_map<std::string, KvsValue>;

    // Enum to represent the type of the value
    enum class Type {
        Number,
        Boolean,
        String,
        Null,
        Array,
        Object
    };

    // Constructors for each type
    KvsValue(double number) : value(number), type(Type::Number) {}
    KvsValue(bool boolean) : value(boolean), type(Type::Boolean) {}
    KvsValue(const std::string& str) : value(str), type(Type::String) {}
    KvsValue(std::nullptr_t) : value(nullptr), type(Type::Null) {}
    KvsValue(const Array& array) : value(array), type(Type::Array) {}
    KvsValue(const Object& object) : value(object), type(Type::Object) {}

    // Get the type of the value
    Type getType() const { return type; }

    // Access the underlying value (use std::get to retrieve the value)
    const std::variant<double, bool, std::string, std::nullptr_t, Array, Object>& getValue() const {
        return value;
    }

private:
    // The underlying value
    std::variant<double, bool, std::string, std::nullptr_t, Array, Object> value;

    // The type of the value
    Type type;
};

// @brief 
enum class ErrorCode {
    // Error that was not yet mapped
    UnmappedError = 0,

    // File not found
    FileNotFound = 1,

    // KVS file read error
    KvsFileReadError = 2,

    // KVS hash file read error
    KvsHashFileReadError = 3,

    // JSON parser error
    JsonParserError = 4,

    // JSON generator error
    JsonGeneratorError = 5,

    // Physical storage failure
    PhysicalStorageFailure = 6,

    // Integrity corrupted
    IntegrityCorrupted = 7,

    // Validation failed
    ValidationFailed = 8,

    // Encryption failed
    EncryptionFailed = 9,

    // Resource is busy
    ResourceBusy = 10,

    // Out of storage space
    OutOfStorageSpace = 11,

    // Quota exceeded
    QuotaExceeded = 12,

    // Authentication failed
    AuthenticationFailed = 13,

    // Key not found
    KeyNotFound = 14,

    // Serialization failed
    SerializationFailed = 15,

    // Invalid snapshot ID
    InvalidSnapshotId = 16,

    // Conversion failed
    ConversionFailed = 17,

    // Mutex failed
    MutexLockFailed = 18
};

/**
 * @class Kvs
 * @brief A thread-safe key-value store (KVS) with support for default values, snapshots, and persistence.
 * 
 * The Kvs class provides an interface for managing a key-value store with features such as:
 * - Thread safety using a mutex.
 * - Support for default values.
 * - Snapshot management for persistence and restoration.
 * - Configurable flush-on-exit behavior.
 * 
 * Features:
 * - `FEAT_REQ__KVS__thread_safety`: Ensures thread safety using a mutex.
 * - `FEAT_REQ__KVS__default_values`: Allows optional default values for keys.
 * 
 * Public Methods:
 * - `open`: Opens the KVS with a specified instance ID and flags.
 * - `set_flush_on_exit`: Configures whether the KVS should flush to storage on exit.
 * - `get_all_keys`: Retrieves all keys stored in the KVS.
 * - `key_exists`: Checks if a specific key exists in the KVS.
 * - `get_value`: Retrieves the value associated with a specific key.
 * - `get_default_value`: Retrieves the default value associated with a specific key.
 * - `has_default_value`: Checks if a default value exists for a specific key.
 * - `set_value`: Sets the value for a specific key in the KVS.
 * - `remove_key`: Removes a specific key from the KVS.
 * - `flush`: Flushes the KVS to storage.
 * - `snapshot_count`: Retrieves the number of available snapshots.
 * - `max_snapshot_count`: Retrieves the maximum number of snapshots allowed.
 * - `snapshot_restore`: Restores the KVS from a specified snapshot.
 * - `get_kvs_filename`: Retrieves the filename associated with a snapshot.
 * - `get_kvs_hash_filename`: Retrieves the hash filename associated with a snapshot.
 * 
 * Private Members:
 * - `kvs_mutex`: A mutex for ensuring thread safety.
 * - `kvs`: An unordered map for storing key-value pairs.
 * - `default_values`: An unordered map for storing optional default values.
 * - `filename_prefix`: A string prefix for filenames associated with snapshots.
 * - `flush_on_exit`: An atomic boolean flag indicating whether to flush on exit.
 */
class Kvs {
    public:
        /**
         * @brief Opens the key-value store with the specified instance ID and flags.
         * 
         * This function initializes and opens the key-value store (KVS) for a given instance ID. 
         * It allows the caller to specify whether default values and an existing KVS are required 
         * or optional during the opening process.
         * 
         * @param id The instance ID of the KVS. This uniquely identifies the KVS instance.
         * @param need_defaults A flag of type OpenNeedDefaults indicating whether default values 
         *                      are required or optional.
         *                      - OpenNeedDefaults::Required: Default values must be available.
         *                      - OpenNeedDefaults::Optional: Default values are optional.
         * @param need_kvs A flag of type OpenNeedKvs indicating whether the KVS is required or optional.
         *                 - OpenNeedKvs::Required: The KVS must already exist.
         *                 - OpenNeedKvs::Optional: An empty KVS will be used if no KVS exists.
         * 
         * @return A Result object containing either:
         *         - A Kvs object if the operation is successful.
         *         - An ErrorCode if an error occurs during the operation.
         * 
         * Possible Error Codes:
         * - ErrorCode::FileNotFound: The KVS file was not found.
         * - ErrorCode::KvsFileReadError: An error occurred while reading the KVS file.
         * - ErrorCode::IntegrityCorrupted: The KVS integrity is corrupted.
         * - ErrorCode::ValidationFailed: Validation of the KVS data failed.
         * - ErrorCode::ResourceBusy: The KVS resource is currently in use.
         */
        Result<Kvs, ErrorCode> open(const InstanceId& id, OpenNeedDefaults need_defaults, OpenNeedKvs need_kvs);

        /**
         * @brief Sets whether the key-value store should flush its contents to
         *        persistent storage upon program exit.
         * 
         * @param flush A boolean value indicating whether to enable or disable
         *              flushing on exit. If true, the store will flush its
         *              contents to persistent storage when the program exits.
         *              If false, the store will not perform this operation.
         */
        void set_flush_on_exit(bool flush);

        /**
         * @brief Retrieves all keys stored in the key-value store.
         * 
         * @return Result<std::vector<std::string_view>, ErrorCode> 
         *         A result object containing a vector of all keys on success, 
         *         or an error code on failure.
         */
        Result<std::vector<std::string_view>, ErrorCode> get_all_keys();


        /**
         * @brief Checks if a key exists in the key-value store.
         * 
         * @param key The key to check for existence, provided as a std::string_view.
         * @return Result<bool, ErrorCode> 
         *         - On success: A Result containing `true` if the key exists, or `false` if it does not.
         *         - On failure: A Result containing an appropriate ErrorCode.
         */
        Result<bool, ErrorCode> key_exists(const std::string_view key);


        /**
         * @brief Retrieves the value associated with the specified key from the key-value store.
         * 
         * @param key The key for which the value is to be retrieved. It is passed as a string view to avoid unnecessary copying.
         * @return A Result object containing either the retrieved value (of type KvsValue) or an error code (of type ErrorCode) 
         *         if the operation fails.
         */
        Result<KvsValue, ErrorCode> get_value(const std::string_view key);


        /**
         * @brief Retrieves the default value associated with the specified key.
         * 
         * This function attempts to fetch the default value for a given key from the
         * key-value store. If the key exists, the corresponding value is returned.
         * Otherwise, an error code is provided to indicate the failure reason.
         * 
         * @param key The key for which the default value is to be retrieved.
         *            It is passed as a string view to avoid unnecessary string copies.
         * 
         * @return A Result object containing either the default value (KvsValue) if
         *         the operation is successful, or an ErrorCode indicating the error
         *         if the operation fails.
         */
        Result<KvsValue, ErrorCode> get_default_value(const std::string_view key);


        /**
         * @brief Checks if the specified key has a default value.
         * 
         * This function determines whether the given key is associated with a default
         * value in the key-value store.
         * 
         * @param key The key to check, provided as a string view.
         * @return Result<bool, ErrorCode> 
         *         - `true` if the key has a default value.
         *         - `false` if the key does not have a default value.
         *         - An `ErrorCode` if an error occurs during the operation.
         */
        Result<bool, ErrorCode> has_default_value(const std::string_view key);



        /**
         * @brief Stores a key-value pair in the key-value store.
         * 
         * @param key The key associated with the value to be stored. 
         *            It is represented as a string view to avoid unnecessary copying.
         * @param value The value to be stored, represented as a KvsValue object.
         * 
         * @return Result<bool, ErrorCode> 
         *         - On success: Returns a Result containing `true` if the value was successfully set.
         *         - On failure: Returns a Result containing an appropriate ErrorCode.
         */
        Result<bool, ErrorCode> set_value(const std::string_view key, const KvsValue& value);


        /**
         * @brief Removes a key-value pair from the store based on the specified key.
         * 
         * @param key The key of the key-value pair to be removed. It is passed as a 
         *            std::string_view to avoid unnecessary copying.
         * @return Result<void, ErrorCode> Returns a Result object. If the operation 
         *         is successful, it contains no value (void). If an error occurs, 
         *         it contains an appropriate ErrorCode.
         */
        Result<void, ErrorCode> remove_key(const std::string_view key);


        /**
         * @brief Flushes the key-value store, ensuring that all pending changes 
         *        are written to the underlying storage.
         * 
         * @return A Result object that indicates the success or failure of the operation.
         *         - On success: Returns a void Result.
         *         - On failure: Returns an ErrorCode describing the error.
         */
        Result<void, ErrorCode> flush();


        /**
         * @brief Retrieves the number of snapshots currently stored in the key-value store.
         * 
         * @return The total count of snapshots as a size_t value.
         */
        size_t snapshot_count() const;


        /**
         * @brief Retrieves the maximum number of snapshots that can be stored.
         * 
         * This function returns the upper limit on the number of snapshots
         * that the key-value store can maintain at any given time.
         * 
         * @return The maximum count of snapshots as a size_t value.
         */
        size_t max_snapshot_count() const;


        /**
         * @brief Restores the state of the key-value store from a specified snapshot.
         * 
         * This function attempts to restore the key-value store to the state
         * captured in the snapshot identified by the given snapshot ID. If the
         * restoration process fails, an appropriate error code is returned.
         * 
         * @param snapshot_id The identifier of the snapshot to restore from.
         * @return Result<void, ErrorCode> 
         *         - On success: An empty result indicating the restoration was successful.
         *         - On failure: An error code describing the reason for the failure.
         */
        Result<void, ErrorCode> snapshot_restore(const SnapshotId& snapshot_id);


        /**
         * @brief Retrieves the filename associated with a given snapshot ID in the key-value store.
         * 
         * @param snapshot_id The identifier of the snapshot for which the filename is to be retrieved.
         * @return A string view representing the filename corresponding to the provided snapshot ID.
         */
        const std::string_view get_kvs_filename(const SnapshotId& snapshot_id) const;


        /**
         * @brief Retrieves the filename of the hash file associated with a given snapshot ID.
         * 
         * This function returns a string view representing the filename of the hash file
         * corresponding to the provided snapshot ID. The hash file is typically used to
         * store metadata or integrity information for the snapshot.
         * 
         * @param snapshot_id The identifier of the snapshot for which the hash filename is requested.
         * @return A string view representing the hash filename associated with the given snapshot ID.
         */
        const std::string_view get_kvs_hash_filename(const SnapshotId& snapshot_id) const;

    private:
        // Internal storage and configuration details.
        std::mutex kvs_mutex;
        std::unordered_map<std::string, KvsValue> kvs;
    
        // Optional default values
        std::unordered_map<std::string, KvsValue> default_values;
    
        // Filename prefix
        std::string filename_prefix;
    
        // Flush on exit flag
        std::atomic<bool> flush_on_exit;
};