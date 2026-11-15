//! Rules of the spike records: each crate of the `spikes/` workspace has one
//! decision page, the page has the four sections in order, and its Source
//! section names the issue and the pull request of the spike. Each rule is a
//! function over text, so a test can also pass it a text that breaks the rule.

use std::fmt::Write as _;

mod support;

const DECISIONS: &str = "docs/src/content/docs/decisions";
const SECTIONS: [&str; 4] = ["Decision", "Reason", "Not tested", "Source"];

/// The parts of a decision page that the rules read.
struct Page {
    name: String,
    headings: Vec<String>,
    source: String,
}

impl Page {
    /// Reads the headings and the Source text of a page.
    fn parse(name: &str, text: &str) -> Self {
        let sections = support::h2_sections(text);
        let source = sections
            .iter()
            .filter(|(heading, _)| heading == "Source")
            .map(|(_, body)| body.as_str())
            .collect();
        Self {
            name: name.to_owned(),
            headings: sections.into_iter().map(|(heading, _)| heading).collect(),
            source,
        }
    }
}

/// True if `text` names `spikes/<krate>` as a whole path, or a path under it.
fn names_crate(text: &str, krate: &str) -> bool {
    let needle = format!("spikes/{krate}");
    text.match_indices(&needle).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let rest = &text[start + needle.len()..];
        let ends = |c: Option<char>| matches!(c, None | Some('`' | ' ' | '\n'));
        let after = match rest.chars().next() {
            // A full stop ends the path only when the sentence ends with it.
            Some('.') => ends(rest.chars().nth(1)),
            next => next == Some('/') || ends(next),
        };
        ends(before) && after
    })
}

/// Each crate whose name is in the Source section of no page or of more than one page.
fn crates_without_one_page(crates: &[String], pages: &[Page]) -> Vec<String> {
    crates
        .iter()
        .filter(|krate| {
            pages
                .iter()
                .filter(|page| names_crate(&page.source, krate))
                .count()
                != 1
        })
        .cloned()
        .collect()
}

/// The pages whose Source section names one of the crates.
fn spike_pages<'a>(crates: &[String], pages: &'a [Page]) -> Vec<&'a Page> {
    pages
        .iter()
        .filter(|page| crates.iter().any(|krate| names_crate(&page.source, krate)))
        .collect()
}

/// True if the page has exactly the four sections, in order.
fn has_sections_in_order(page: &Page) -> bool {
    page.headings == SECTIONS
}

/// True if `text` holds `<words> <n>` with a number, as whole words. A line end counts as a space.
fn names_numbered(text: &str, words: &str) -> bool {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let needle = format!("{words} ");
    text.match_indices(&needle).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let after = text[start + needle.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && after.is_some_and(|c| c.is_ascii_digit())
    })
}

/// True if the Source section names an issue and a pull request, each with a number.
fn source_names_issue_and_pull_request(page: &Page) -> bool {
    names_numbered(&page.source, "issue") && names_numbered(&page.source, "pull request")
}

/// The path of each spike crate under `spikes/`, from the directory of its manifest.
fn real_crates() -> Vec<String> {
    let metadata =
        support::workspace_metadata(&support::repo_root().join("spikes").join("Cargo.toml"));
    metadata
        .workspace_packages()
        .iter()
        .filter_map(|package| {
            let directory = package.manifest_path.parent()?;
            Some(
                directory
                    .strip_prefix(&metadata.workspace_root)
                    .ok()?
                    .to_string(),
            )
        })
        .collect()
}

fn real_pages() -> Vec<Page> {
    let root = support::repo_root();
    let mut pages: Vec<Page> = support::files_under(&root.join(DECISIONS))
        .into_iter()
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .map(|path| {
            let relative = path
                .strip_prefix(root)
                .expect("the page is in the repository");
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            Page::parse(&name, &support::repo_file(&relative.to_string_lossy()))
        })
        .collect();
    pages.sort_by(|a, b| a.name.cmp(&b.name));
    pages
}

/// A decision page with front matter and the given sections.
fn page(name: &str, sections: &[(&str, &str)]) -> Page {
    let mut text = String::from("---\ntitle: T\ndescription: D.\nsidebar:\n  order: 1\n---\n\n");
    for (heading, body) in sections {
        write!(text, "## {heading}\n\n{body}\n\n").expect("a String takes the text");
    }
    Page::parse(name, &text)
}

fn record(source: &str) -> Page {
    page(
        "record.md",
        &[
            ("Decision", "D."),
            ("Reason", "R."),
            ("Not tested", "- N."),
            ("Source", source),
        ],
    )
}

fn crates(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn spike_crates_have_decision_pages() {
    let real = real_crates();
    assert!(!real.is_empty(), "the spikes workspace has crates");
    assert_eq!(
        crates_without_one_page(&real, &real_pages()),
        Vec::<String>::new()
    );

    let one = crates(&["pane"]);
    for source in [
        "The code is `spikes/pane`.",
        "The code is in spikes/pane and spikes/pane/data.",
        "spikes/pane",
        "See `spikes/pane/results`.",
        "The code is spikes/pane.",
        "The code is in\nspikes/pane.",
        "`spikes/pane`",
        "- The code is spikes/pane\n- The data is elsewhere.",
        "### Code\n\nThe code is `spikes/pane`.",
        "| Code |\n|---|\n|spikes/pane|",
    ] {
        assert!(
            crates_without_one_page(&one, &[record(source)]).is_empty(),
            "{source:?} names the crate"
        );
    }

    for (case, pages) in [
        ("no page", vec![record("The code is `spikes/other`.")]),
        (
            "two pages",
            vec![record("`spikes/pane`"), record("`spikes/pane`")],
        ),
        (
            "a longer name",
            vec![record("The code is `spikes/pane-v2`.")],
        ),
        (
            "a dotted name",
            vec![record("The code is `spikes/pane.v2`.")],
        ),
        (
            "a name in emphasis",
            vec![record("The code is *spikes/pane*x.")],
        ),
        (
            "a longer parent",
            vec![record("The code is `oldspikes/pane`.")],
        ),
        (
            "the name only in Reason",
            vec![page(
                "p.md",
                &[
                    ("Decision", "D."),
                    ("Reason", "The code is `spikes/pane`."),
                    ("Not tested", "- N."),
                    ("Source", "Issue 1."),
                ],
            )],
        ),
        (
            "the name in a section after Source",
            vec![page(
                "p.md",
                &[
                    ("Decision", "D."),
                    ("Reason", "R."),
                    ("Not tested", "- N."),
                    ("Source", "Issue 1."),
                    ("Notes", "The code is `spikes/pane`."),
                ],
            )],
        ),
    ] {
        assert_eq!(
            crates_without_one_page(&one, &pages),
            one,
            "{case} breaks the rule"
        );
    }
}

#[test]
fn spike_pages_have_sections_in_order() {
    let real = real_crates();
    let pages = real_pages();
    let spike = spike_pages(&real, &pages);
    assert_eq!(spike.len(), real.len(), "each spike crate has a page");
    for page in spike {
        assert!(
            has_sections_in_order(page),
            "{} has the sections in order",
            page.name
        );
    }

    assert!(has_sections_in_order(&record("S.")));
    for (case, sections) in [
        (
            "no Not tested",
            vec![("Decision", "D."), ("Reason", "R."), ("Source", "S.")],
        ),
        (
            "another order",
            vec![
                ("Decision", "D."),
                ("Not tested", "- N."),
                ("Reason", "R."),
                ("Source", "S."),
            ],
        ),
        (
            "a fifth heading",
            vec![
                ("Decision", "D."),
                ("Reason", "R."),
                ("Results", "X."),
                ("Not tested", "- N."),
                ("Source", "S."),
            ],
        ),
        (
            "Source twice",
            vec![
                ("Decision", "D."),
                ("Reason", "R."),
                ("Not tested", "- N."),
                ("Source", "S."),
                ("Source", "S."),
            ],
        ),
    ] {
        assert!(
            !has_sections_in_order(&page("p.md", &sections)),
            "{case} breaks the rule"
        );
    }
}

#[test]
fn spike_sources_name_issue_and_pull_request() {
    let real = real_crates();
    let pages = real_pages();
    for page in spike_pages(&real, &pages) {
        assert!(
            source_names_issue_and_pull_request(page),
            "the Source section of {} names the issue and the pull request",
            page.name
        );
    }

    for source in [
        "The spike of issue 3, merged in pull request 29.",
        "The spike of issue 3, merged in pull request\n29.",
        "The spike of issue 3, merged in pull\nrequest 29.",
    ] {
        assert!(
            source_names_issue_and_pull_request(&record(source)),
            "{source:?} names both"
        );
    }
    for source in [
        "The spike of issue 3.",
        "Merged in pull request 29.",
        "The spike of the issue, merged in pull request 5.",
        "The spike of issue 5, merged in a pull request.",
        "The spike of reissue 3, merged in pull request 29.",
    ] {
        assert!(
            !source_names_issue_and_pull_request(&record(source)),
            "{source:?} breaks the rule"
        );
    }
    let outside = page(
        "p.md",
        &[
            ("Decision", "D."),
            ("Reason", "Merged in pull request 5."),
            ("Not tested", "- N."),
            ("Source", "The spike of issue 3."),
        ],
    );
    assert!(
        !source_names_issue_and_pull_request(&outside),
        "a pull request outside Source breaks the rule"
    );
}
