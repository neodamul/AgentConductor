# Windows UI Automation Text Pacing

The packaged Windows acceptance driver uses Unicode `SendInput` to type into native text controls. Sending every character in one tight loop can report successful input while the packaged app persists an empty value. Pause briefly after each Unicode key-down/key-up pair, then allow the existing settle interval before reading the control and verifying the saved workspace data.

The source-contract test checks that each character pair is followed by a 100 ms pause. This protects the test driver only; it does not prove that the packaged Windows application accepted the input. Exact packaged execution and persisted-state inspection remain required for acceptance.
