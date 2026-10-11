# Enterprise quota budgets

Trace: R-74 / UC-63 / QAS-68 / DD-81. A time window is not an organization allocation. Do not relabel secondary/weekly quota as Total or infer a company budget from Enterprise planType.

The installed Codex 0.162.1 generated app-server schema contains individualLimit (limit, used, remainingPercent, resetsAt) and credits (balance, hasCredits, unlimited). Preserve these independently of primary/secondary windows. Bounded exact decimal subtraction derives remaining from reported limit/used. Keep scope explicit: individual allocation is not organization-wide allocation. Balance-only credits have an unknown denominator, not 100%.

[Official app-server documentation](https://learn.chatgpt.com/docs/app-server) describes credits as optional provider metadata. [Claude Enterprise documentation](https://support.claude.com/en/articles/11845131-use-claude-code-with-your-team-or-enterprise-plan) distinguishes seat-based and usage-based plans. Actual Windows Enterprise responses have not been supplied; fixtures do not prove real account access or the Claude display format. No Admin API key, inference prompt or HTTP quota endpoint was added.
