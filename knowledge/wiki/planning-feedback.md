# Windows picker and planning feedback

Windows PowerShell writes picker output in the OEM code page and may add a UTF-8 BOM, so a picked Korean path broke while a typed one worked. The owner script forces UTF-8 without BOM and `picked()` strips a leading BOM (DD-90).

Generate plan copies an ordinary folder before replying; the 15 s bridge default produced a false timeout. Planning-entry actions now wait up to 300 s. The review area reports the coordinator's real output (bytes and last write of its stdout/stderr) and warns after two minutes of silence; reduced motion keeps an opacity-only blink (DD-91).

Trace: R-82, R-83, DD-90, DD-91, QAS-78, QAS-79. Evidence: tests/picker_output.rs, tests/windows_console.rs, tests/planning_progress.rs, tests/test_planning_feedback.py.
