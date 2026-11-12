# Keep MCP registration separate from automatic document reads

Source: [fingerprinted verification](../raw/mcp-server-settings-882.json), GitHub #882. Trace: R-134 / UC-124 / QAS-130 / DD-143.

Normal Settings registers MCP servers and shows discovery status/count/errors. A duplicate textual tool catalog and manual Jira/Confluence binding forms are unnecessary: the configured CLI coordinator already selects discovered read tools and schema-valid arguments for Goal references.

Remove the presentation and obsolete handlers, not the internal configuration compatibility or read safety checks. Preserve legacy `goal_sources` values in unrelated server saves and advanced JSON round trips. Settings removal must not introduce automatic writes or external transmissions.

The product and approved reference now omit manual bindings. Browser regressions cover both themes at840/1280px; native tests cover coordinator-selected reads, rejected write/undiscovered tools and legacy round trips. No live Jira/Confluence requests were made.
