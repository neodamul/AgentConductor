# Interface message catalogs

All app-owned interface wording is referenced by a stable message ID. English and Korean text live in [en.json](../src/ui/messages/en.json) and [ko.json](../src/ui/messages/ko.json). The catalogs are UTF-8 JSON data and are exempt from the 500-line code limit. IDs are grouped by feature: startup, room, mcp, native, quota, state and common. Shared wording may reuse an existing ID across screens.

To change wording, edit the values in both language files and keep the ID unchanged. IDs are explicit identifiers, not an English-text lookup; changing the English value does not change its Korean association. For a new message, add a descriptive feature ID to both files. Keep the same named placeholders in each language. Values passed into placeholders are substituted once and are never interpreted as markup or translated as user content.

Static HTML uses data-i18n for text and data-i18n-placeholder, data-i18n-title or data-i18n-aria-label for attributes. Bind option labels on the option itself. JavaScript uses acMessage(id, parameters). AC_I18N.bind(node, () => acMessage(...)) keeps text or an attribute current when the language changes. Node creation helpers bind their translated labels as text nodes so appended icons are retained. Use AC_I18N.replace to replace rendered children and discard obsolete text bindings.

Native app messages use ui::messages::text and ui::messages::render. The saved interface language selects the catalog, with English as the fallback. System/provider errors, source documents, typed values, goals, code, model IDs, paths and raw logs retain their original content. Windows picker captions and filters travel in environment values rather than being interpolated into PowerShell code.

Checks: cargo test --test messages --test interface_language --test source_layout, node tests/i18n_runtime.cjs, and python -B -m unittest discover -s tests -p 'test_*.py'. The checks detect missing IDs, duplicate entries, mismatched placeholders, untranslated static UI text, language-switch regressions and changes to user content.
