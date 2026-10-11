# Custom CLI model catalogs

User confirmed that aipro executes tasks but does not expose `/models` or another catalog command. An OpenAI-compatible serving protocol does not imply CLI metadata discovery. Automatic catalog retrieval cannot be claimed without a supported CLI contract.

The Models editor accepts company-provided IDs one per line or observed JSON. Manual IDs carry an operator-configured source and unavailable CLI version. They remain local profile metadata and survive reopening. No endpoint, API key, unsupported slash command or generation prompt is sent. A selected execution model still needs the real CLI argument containing `{model}`.

Trace: R-73 / UC-62 / QAS-67 / DD-80. Evidence: `tests/test_custom_model_catalog.py` exercises the actual JavaScript parser; `operator_catalog_is_durable_without_a_discovery_job` checks persistence and absence of a subprocess job.

The Korean native dialog was verified on the exact macOS CI binary in stable v0.1.2; cancellation preserved the existing goal and reviewed plan. Both native platform validation suites include parser/persistence contracts. IDs remain operator assertions, and the working aipro CLI was not invoked with an unsupported catalog command.
