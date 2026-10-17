use agentconductor::manual::{Block, blocks, render};
use std::{fs, path::Path};

#[test]
fn offline_manual_rendering_preserves_safe_tables_code_and_local_images() {
    let root = std::env::temp_dir().join(format!("ac-manual-{}", std::process::id()));
    let assets = root.join("assets");
    fs::create_dir_all(&assets).unwrap();
    fs::write(assets.join("test.png"), b"png").unwrap();
    fs::write(assets.join("test.jpg"), b"jpg").unwrap();
    let html = render(
        "| Command | Purpose |\n|---|---|\n| `status RUN` | **Inspect** [docs](https://example.com) |\n\n<script>bad()</script>\n\n[bad](javascript:alert)\n\n![Screen](assets/test.png)\n\n![Photo](assets/test.jpg)\n\n```rust\nlet x = \"<script>\";\n```\n",
        Path::new(&root),
    )
    .unwrap();
    assert!(html.contains("<table>") && html.contains("<th>Command</th>"));
    assert!(html.contains("<td><code>status RUN</code></td>"));
    assert!(
        html.contains("<strong>Inspect</strong>") && html.contains("href=\"https://example.com\"")
    );
    assert!(html.contains("data:image/png;base64,cG5n"));
    assert!(html.contains("data:image/jpeg;base64,anBn"));
    assert!(!html.contains("<script>bad()") && !html.contains("href=\"javascript:"));
    assert!(html.contains("&lt;script&gt;"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn renderer_handles_escaped_table_pipes_and_rejects_images_outside_assets() {
    let root = std::env::temp_dir().join(format!("ac-manual-escape-{}", std::process::id()));
    let assets = root.join("assets");
    fs::create_dir_all(&assets).unwrap();
    fs::write(root.join("secret.png"), b"private").unwrap();
    fs::write(assets.join("image.svg"), b"<svg/>").unwrap();

    let html = render(
        "| Command | Purpose |\n|---|---|\n| `one\\|two` | inspect |\n",
        Path::new(&root),
    )
    .unwrap();
    assert!(html.contains("<td><code>one|two</code></td>"));
    assert!(render("![Bad](secret.png)", Path::new(&root)).is_err());
    assert!(render("![Bad](assets/image.svg)", Path::new(&root)).is_err());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn committed_html_is_generated_from_both_manual_sources() {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs");
    for name in ["user-manual", "user-manual.ko"] {
        let source = fs::read_to_string(docs.join(format!("{name}.md"))).unwrap();
        let expected = fs::read_to_string(docs.join(format!("{name}.html"))).unwrap();
        let actual = render(&source, &docs).unwrap();
        let expected = expected.replace("\r\n", "\n");
        if expected != actual {
            let (line, before, after) = expected
                .lines()
                .zip(actual.lines())
                .enumerate()
                .find(|(_, (a, b))| a != b)
                .map(|(i, (a, b))| (i + 1, a, b))
                .unwrap_or((0, "<eof>", "<eof>"));
            panic!("{name} differs at line {line}: expected {before:?}; got {after:?}");
        }
    }
}

#[test]
fn native_manual_blocks_preserve_headings_lists_tables_and_code() {
    let parsed = blocks(
        "# Guide\n\nChoose one.\n\n- First\n- Second\n\n| Command | Purpose |\n|---|---|\n| `status RUN` | Inspect |\n\n```sh\nagentconductor status RUN\n```\n",
    );
    assert_eq!(
        parsed,
        vec![
            Block::Heading {
                level: 1,
                text: "Guide".into()
            },
            Block::Paragraph("Choose one.".into()),
            Block::List(vec!["First".into(), "Second".into()]),
            Block::Table {
                headers: vec!["Command".into(), "Purpose".into()],
                rows: vec![vec!["`status RUN`".into(), "Inspect".into()]],
            },
            Block::Code("agentconductor status RUN".into()),
        ]
    );
}
