# Native AC application identity

## Decision

R-66 / DD-72 use the existing lowercase AC titlebar mark, forest #23352f, paper
#f3f5ed and sage #bfd4ad, as the project-owned launcher identity. Titlebars and
native shells share derived committed assets under src/ui/assets. The root MIT
license covers this artwork; no provider logo or font file is distributed with it.

## Asset generation

The 1024px master was drawn locally with Pillow, a rounded rectangle and the two
letters rendered using the host Arial Bold font with negative tracking. Pillow
was used only to prepare the committed assets; it is not an application or build
dependency. Derived PNG, ICNS, ICO and 64px RGBA bytes are committed.

Cocoa creates NSImage from include_bytes and sets the application icon even for
a standalone binary launch. The optional Finder .app wrapper includes ac.icns.
Windows embeds ICO through the installed SDK resource compiler and uses RGBA
for the native window/taskbar icon. No image service, model prompt or runtime
asset generation is required. Windows resource and visual behavior still require
Windows CI/host verification; macOS is verified separately.

## Evidence

Container/header regression checks, macOS release build and native screenshot
inspection establish the asset and titlebar behavior. See the current UI assets
and DD-72 checklist for acceptance evidence.
