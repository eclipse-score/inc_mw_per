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

#include <benchmark/benchmark.h>
#include <string>

#define private public
#define final 
#include "kvsbuilder.hpp"
#undef private
#undef final
#include "internal/kvs_helper.hpp"
using namespace score::mw::pers::kvs;

static void BM_get_hash_bytes(benchmark::State& state) {
    // Prepare a test string of configurable size
    std::string data(state.range(0), 'a');
    for (auto _ : state) {
        benchmark::DoNotOptimize(get_hash_bytes(data));
    }
    state.SetBytesProcessed(int64_t(state.iterations()) * int64_t(data.size()));
}

// Register the function as a benchmark with different input sizes
BENCHMARK(BM_get_hash_bytes)->Range(16, 16<<10);

BENCHMARK_MAIN();
