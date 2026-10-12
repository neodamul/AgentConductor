# Native window ownership

DD-84 uses an app-owned NSOpenPanel on the AppKit main queue. External script activation can create a picker owned by another process and obscure it behind the desktop app. Windows uses GUI subsystem plus CREATE_NO_WINDOW for background children; existing CLI terminal attachment remains separate from allocation. No extra daemon or dependencies are introduced.

Verification distinguishes native foreground/cancellation checks, PE subsystem and actual console-child pipe tests, and packaged workflow execution.
