# Key-Value-Store Tests

## License

```
Copyright (c) 2025 Contributors to the Eclipse Foundation

See the NOTICE file(s) distributed with this work for additional
information regarding copyright ownership.

This program and the accompanying materials are made available under the
terms of the Apache License Version 2.0 which is available at
https://www.apache.org/licenses/LICENSE-2.0

SPDX-License-Identifier: Apache-2.0
```

## Separation Note

All tests are kept in separate files as the working directory is always
changed. If there is more than one test in a file both tests are run in
parallel and use the same working directory which will led to wrong results.
