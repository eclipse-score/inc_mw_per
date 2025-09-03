..
   # *******************************************************************************
   # Copyright (c) 2025 Contributors to the Eclipse Foundation
   #
   # See the NOTICE file(s) distributed with this work for additional
   # information regarding copyright ownership.
   #
   # This program and the accompanying materials are made available under the
   # terms of the Apache License Version 2.0 which is available at
   # https://www.apache.org/licenses/LICENSE-2.0
   #
   # SPDX-License-Identifier: Apache-2.0
   # *******************************************************************************

:orphan:

.. document:: Persistency Software Verification Plan
    :id: doc__persistency_software_verification_plan
    :status: valid
    :security: NO
    :safety: QM
    :realizes: PROCESS_wp__verification__plan

Software Verification Plan
**************************
.. This document provides a template for a software verification plan.
.. It should be adapted to the specific needs of project.

Persistency Key-Value Storage (KVS) Software Verification Plan

Purpose
=======
.. This section should briefly describe the overall goal of the verification plan.
.. It should state the plan's intended audience and the information it aims to provide.
.. This might include clarifying the scope of the verification activities and linking to
.. other relevant documents.

This document describes the test plan for the Persistency Key-Value Storage (KVS) module. The plan outlines the scope, approach, resources, and schedule of all testing activities.
The goal is to verify that the KVS implementation meets the specified feature and module-level requirements and is robust for use in an automotive context.



Objectives and Scope
====================

Objectives
----------
.. This section outlines the key objectives of the software verification effort.
.. Examples include correctness, completeness, reliability, performance, maintainability,
.. compliance, and traceability. Each objective should be clearly defined and measurable.

The primary objectives of this verification plan are to ensure the Key-Value Store (KVS) module is fit for its intended purpose in a safety critical environment. Each objective is defined as follows:

* **Correctness**: To verify that the KVS module's implementation strictly adheres to the `feature requirements`_ and `module requirements`_. All API functions must behave as specified under all documented conditions.
* **Completeness**: To ensure that all specified requirements for the KVS module have corresponding verification tests.
* **Reliability**: To demonstrate that the KVS module operates dependably over extended periods and recovers gracefully from faults, ensuring data integrity and system consistency.
* **Performance**: To validate that the KVS module meets the timing and resource usage constraints specified in the non-functional requirements, ensuring it does not negatively impact the overall system performance.
* **Maintainability**: To ensure that the test suite is well-documented, automated, and easy to modify, facilitating future updates and regression testing.
* **Compliance**: To ensure the verification process and its work products adhere to the standards outlined in the project's quality plan.
* **Traceability**: To maintain a clear and auditable link between requirements, design documents, source code and verification test cases.

.. LINKS
.. _feature requirements: https://eclipse-score.github.io/score/main/features/persistency/kvs/requirements/index.html
.. _module requirements: https://eclipse-score.github.io/score/main/modules/persistency/kvs/docs/requirements/index.html

Verification Scope and Constraints
----------------------------------
.. This section details what software components and functionalities are included in the
.. verification process. It should also clearly specify any limitations or exclusions.
.. This section should address external dependencies and integrations.

This plan details the verification of the KVS software module.

In Scope:

* **Components**: The KVS module source code, including all public API functions and internal logic.
* **Functionalities**:

  * All Create, Read, Update, Delete (CRUD) operations.
  * Error handling and reporting mechanisms.
  * Data persistence across program cycles.
  * Atomic write operations.
* **External Dependencies**: The interaction between the KVS module and the underlying file system is in scope, but only to the extent that it affects KVS functionality. 
  Verification will ensure that the KVS module's assumptions about the storage layer are valid.

Out of Scope (Exclusions):

* **Hardware-Specific Validation**: Testing of the physical storage medium's endurance, wear-leveling, or low-level hardware error correction is not covered.
* **Underlying System Verification**: The operating system, its file system implementation, and device drivers are considered pre-validated and are not part of this verification effort.
* **End-to-End Application Logic**: System-level tests that validate the business logic of applications using the KVS for storage are out of scope.


Risks and Mitigation
--------------------
.. This section identifies potential risks associated with the verification activities and outlines
.. strategies to mitigate those risks. This may involve referencing the :need:`wp__platform_mgmt`.

Potential risks that derived from the verification activities and their respective mitigation measures are assessed based on Risk Management Matrix and handled by
the project management. Risks are classified based on their likelihood of occurrence and severity of impact, as shown in the table below.

+------------+------------+------------+------------+------------+------------+
| Likelihood | Severity 1 | Severity 2 | Severity 3 | Severity 4 | Severity 5 |
+============+============+============+============+============+============+
| Very High  | Low        | Medium     | High       | Very High  | Very High  |
+------------+------------+------------+------------+------------+------------+
| High       | Low        | Medium     | High       | High       | Very High  |
+------------+------------+------------+------------+------------+------------+
| Medium     | Low        | Medium     | Medium     | High       | High       |
+------------+------------+------------+------------+------------+------------+
| Low        | Very Low   | Low        | Medium     | Medium     | High       |
+------------+------------+------------+------------+------------+------------+
| Very Low   | Very Low   | Very Low   | Low        | Low        | Medium     |
+------------+------------+------------+------------+------------+------------+

Schedules
---------
.. This section defines the timeline for different verification activities.
.. It might include milestones, deadlines, and dependencies between tasks.

The verification activities are planned as follows. This schedule assumes that the entry criteria are met on time.

.. list-table:: Verification Schedule
    :header-rows: 1

    * - Activity
      - Start Date
      - End Date
    * - Verification Plan Finalized 
      - 
      - 
    * - Finalized Requirements
      - 
      - 
    * - Test Environment Ready
      - 
      - 
    * - Unit Test Suite Complete
      - 
      - 
    * - Code complete for KVS module
      - 
      - 
    * - Component Integration Test Suite Complete
      - 
      - 
    * - Feature Integration Test Suite Complete
      - 
      - 
    * - Platform Test Suite Complete
      - 
      - 
    * - Reference Hardware Available
      - 
      - 
    * - Final Verification Report
      - 
      - 



Approach
========

General Approach
----------------
.. This section provides a high-level overview of the verification strategy.
.. It should describe the overall methodology (e.g., Continuous Integration),
.. approaches used, and rationale behind the choices made.

The verification strategy employs a Continuous Integration (CI) methodology. Every code change pushed to the version control system will automatically trigger a build and the execution of a suite of automated tests.
This provides rapid feedback to developers and ensures that regressions are caught early. The approach is layered, starting from developer-led unit tests and progressing to component integration tests maintained by test engineers.

Software Integration
--------------------
.. This section details how software components are integrated into the system.
.. It should describe the integration process, including procedures for handling new features,
.. bug fixes, and code changes.

The integration process follows a structured approach:

#. **Feature Branching**: Developers create feature branches for new functionality or bug fixes. This isolates changes until they are ready for integration.
#. **Code Reviews**: Before merging changes into the main branch, code reviews are conducted to ensure quality and adherence to coding standards.
#. **Automated Testing**: Each integration is accompanied by automated tests that validate the changes. This includes unit tests and component integration tests.
#. **Continuous Integration**: The CI system automatically builds the code and runs tests on each commit. This provides immediate feedback on the impact of changes.
#. **Staging Environment**: Once changes pass automated tests and are approved by reviewers, they can be merged into the main branch.
#. **SCORE Reference**: After successful merge in feature repository, changes can be introduced in reference repository as new reference version to be a part of SCORE. 
   After successful verification including review and automatically executed feature integration tests, changes can be a part of SCORE.

Levels of Integration and Verification
--------------------------------------
.. This section defines the different levels of integration and verification that will be performed
.. (e.g., unit, component, system). Each level should be clearly defined, with associated criteria
.. for successful completion.

#. :need:`PROCESS_wp__verification__sw_unit_test`: Focuses on testing individual functions of the KVS module in isolation from the rest of the system. The goal is to verify the logical correctness of the code.
#. :need:`PROCESS_wp__verification__comp_int_test`: Tests the KVS module along with the storage driver or file system it directly depends on. The goal is to verify the interfaces and interactions between these closely coupled components.
#. :need:`PROCESS_wp__verification__feat_int_test`: Tests the KVS module in conjunction with other system components to validate end-to-end functionality. The goal is to ensure that the KVS module works correctly within the context of the overall system.
#. :need:`PROCESS_wp__verification__platform_test`: Tests the fully integrated KVS module on the reference hardware. The goal is to verify non-functional requirements like performance and robustness in a production-representative environment.

Verification Methods
--------------------
.. This section lists the specific verification methods used, such as static analysis,
.. dynamic testing, reviews, and inspections. Each method should be briefly described,
.. including its purpose and applicability at different levels of verification.
.. Reference tables can list methods, identifiers, applicable levels and ASIL relevance.

All verification methods listed in :need:`PROCESS_gd_meth__verification__methods` are applicable. The following methods will be primarily used:

#. **Fault Injection** - Most effective for identifying edge cases and potential failure points by simulating faults in the system.
#. **Interface Test** - Validates the interactions between the KVS module and its dependencies, ensuring correct data exchange and protocol adherence.
#. **Structural Coverage** - Assesses the codebase to ensure all branches and conditions are tested, providing insights into untested areas.

Test Derivation Methods
^^^^^^^^^^^^^^^^^^^^^^^
.. This section details the techniques used to derive test cases (e.g., boundary value analysis,
.. equivalence partitioning, requirements tracing). It should clarify which techniques are used
.. at each level of testing and for different ASIL levels.  Again, a reference table is recommended.

All derivation methods listed in :need:`PROCESS_gd_meth__verification__derivation` are applicable. In addition, the following methods will be primarily used:

#. Analysis of requirements (requirements-analysis)
#. Analysis of design (design-analysis)
#. Analysis of boundary values (boundary-values)
#. Analysis of equivalence classes (equivalence-classes)
#. Error guessing based on knowledge or experience (error-guessing)
#. Explorative testing (explorative-testing)

Some derivation methods such as fuzzy testing can be not used directly but they can be used in higher testing levels such as Feature Integration Tests.

Quality Criteria
----------------
.. This section specifies the quality criteria that must be met for successful verification.
.. These criteria might include code coverage metrics, defect density, or other relevant measures.
.. The criteria should be defined with quantifiable goals for different ASIL levels.

.. list-table:: Quality Criteria
    :header-rows: 1

    * - Quality Criterion
      - Target Value
    * - Code Coverage (Line)
      - > 98%
    * - Code Coverage (Branch)
      - > 95%
    * - Static Analysis Defects
      - Zero warnings of any level
    * - Requirements Coverage
      - 100% of requirements covered by test cases
    * - Test Pass Rate (for release)
      - 100% of planned tests executed, 100% pass rate

Test Development
----------------
.. This section describes the process for developing and maintaining test cases.
.. It should cover aspects such as test automation, test data management, and version control.

Test cases will be developed in parallel with the software development. All test cases and test scenarios (if applicable) will be stored in the project's version control system (Git) alongside the source code. 
Test automation is mandatory for all unit and integration tests.

.. Pre-existing test cases
.. -----------------------
.. This section describes how pre-existing test cases are handled which are e.g. available
.. from an OSS component. It should be stated how they are reviewed, integrated, extended
.. (e.g. with respective documentation), and adopted to the needs described in the project
.. (e.g. usage of documentation templates and traceability)
.. 
.. N/A

Test Execution and Result Analysis
----------------------------------
.. This section describes how tests will be executed and the procedures for analyzing the results.
.. It should outline the tools and processes used for test execution and reporting.

Tests will be executed automatically via the CI pipeline for every commit. 
Test results will be automatically collected and visible directly in Pull Request. 
The results of integrated SCORE components will be published to a sphinx-needs dashboard showing metrics and bidirectional traceability between requirements and tests.

Test Selection and Regression Testing
-------------------------------------
.. This section describes the approach to selecting test cases for execution and the strategy for
.. regression testing to ensure that new changes don't introduce regressions.

* **CI Builds**: Automated subset of tests (unit tests and component integration tests) will run on every commit.
* **Nightly Builds**: The full suite of unit, component integration, feature integration and platform tests will be executed in a loop to catch sporadic errors.
* **Release Candidates**: The goal is to any version integrated in SCORE to be a release candidate verified by the full suite of unit, component integration, feature integration and platform tests.
* **Regression strategy**: The approach is to re-run all tests that verify functionality that could reasonably be affected by a code change.

Work Products and Traceability
------------------------------
.. This section lists all the key deliverables related to the verification process.
.. It should also describe how traceability between requirements, design, code, and test
.. cases is maintained.

.. list-table:: Work Products and Traceability
    :header-rows: 1
  
    * - Work Product
      - Description
      - Location
    * - :need:`S-CORE_doc__verification_plan`
      - This document.
      - Github Pages
    * - Test Specification
      - Detailed descriptions of all test cases as described in :need:`PROCESS_gd_req__verification__link_tests`.
      - Github Pages, Github Repository
    * - Test Implementation
      - The automated test code.
      - Github Repository
    * - Test Reports
      - The results of each test execution cycle. - :need:`PROCESS_wp__verification__module_ver_report`, :need:`PROCESS_wp__verification__platform_ver_report`
      - Github Actions, Github Pages
    * - Defect Reports
      - Detailed reports for each failed test or identified issue.
      - Github Issues

Environments and Resources
==========================

Roles
-----
.. This section defines the roles and responsibilities of individuals involved in the
.. verification process. It can refer and should be based on the definition in the
.. verification process :ref:`verification_roles`.

.. list-table:: Roles and Responsibilities
    :header-rows: 1

    * - Role
      - Responsibility
    * - Test Lead
      - Owns the verification plan, coordinates all testing activities, and is responsible for the final test report.
    * - Test Engineer
      - Designs, develops, and maintains automated test cases; executes system and robustness tests; analyzes results.
    * - Developer
      - Develops and executes unit tests for their own code; performs code reviews; fixes defects found during verification.


Tools
-----
.. This section lists the tools used for verification, including build systems, test frameworks,
.. static analysis tools, and other relevant software.

List of tools used in the verification process:

#. Sphinx-needs
#. Bazel
#. Cargo Test
#. Google Test
#. Python Pytest

.. #. S-CORE ITF


Verification Setups and Variants
--------------------------------
.. This section describes the different test environments and configurations used for verification.

#. **Developer PC**: Windows/Linux PC with development tools and simulators for local testing and debugging.
#. **CI Environment**: A dedicated CI server that runs automated tests on every commit, ensuring that the codebase remains stable and functional.
#. **Reference Hardware**: External hardware board connected to a PC via debugging probes and a controllable power supply for automated system and robustness testing. Emulations created with QEMU and QNX image.


Test Execution Environment and Reference Hardware
-------------------------------------------------
.. This section describes the hardware and software environments used for test execution.
.. It should include information about any specific hardware platforms or simulators used.
.. It should also define how the verification environment interacts with the CI system, including
.. access control and maintenance.


| **HW**: Qualcomm SA8650 running QNX7.1 / QNX8.0