# Screen fixes need approved states as well as runtime tests

Source: [fingerprinted findings](../raw/design-evidence-890.json), #890.

Runtime behavior tests alone missed gaps in the approved reference for Phase history, multiline raw logs and notification save states. Add reachable reference states, interaction checks and explicit parity entries with the product fix. Capture and inspect the reference rather than treating a passing DOM assertion as visual acceptance.

The follow-up found a real styling defect: the local settings error inherited the global toast's fixed positioning. Keep it in flow inside the still-open editor, verify its parent width, and retain draft/retry behavior. Match the state on both sides of parity: an undiscovered MCP server must not be compared against a successfully discovered one.
