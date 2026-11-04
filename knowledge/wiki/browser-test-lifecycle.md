# Bounded browser-test teardown

Trace: GitHub #435; R-210 / QAS-209 / DD-222.

A child process exit event is not replayed for listeners registered after termination.
The CDP driver used to attach a new exit listener on close, then kill and wait without
a deadline. An already-exited launcher therefore hung even though the UI work was done.

Check both exitCode and signalCode first. For a live Windows launcher, stop only its
owned PID tree with bounded taskkill /PID /T /F, never by executable name. Bound the
exit wait too, remove listeners/timers in finally, and report termination failure rather
than waiting indefinitely. Delete the unique test profile after teardown completes.

The regression exercises the actual CDP driver with an exited launcher and a fake
protocol transport; it fails before the fix without launching a desktop application.
Additional helper tests cover normal exit, Windows ownership, missing PID, failure and
missing exit events. Actual Windows CI must still verify native process-tree behavior.
