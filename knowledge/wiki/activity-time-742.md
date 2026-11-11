# Assigned task activity time (#742)

The assigned task row shows the task's persisted `started_ms` in local HH:MM.
If no start time is recorded, the time column remains empty rather than displaying
the current time. The activity browser test covers a recorded start and a missing
start. Native visual acceptance and integration with #739 remain pending.
