//! Links (MDV-048 P3, group 2): where a link goes, what hovering one shows,
//! and images wrapped in links.

use super::*;
use std::time::Duration;

/// A workspace with `a.md` and files of each kind a link can point at.
fn fixture(tag: &str) -> (App, PathBuf) {
    let dir = std::env::temp_dir().join(format!("rmdv-links-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("sub")).expect("fixture dir");
    std::fs::write(dir.join("a.md"), "# A\n\nText.\n").expect("a.md");
    std::fs::write(dir.join("data.json"), "{\"k\": 1}\n").expect("data.json");
    std::fs::write(dir.join("conf.yaml"), "k: 1\n").expect("conf.yaml");
    std::fs::write(dir.join("sub").join("NOTES.MD"), "# Notes\n").expect("NOTES.MD");
    let dir = canonicalize_existing_path(dir);
    let mut app = App::default();
    app.set_workspace(dir.clone(), false);
    (app, dir)
}

/// Runs a task the way the runtime does, applying the messages it yields.
fn run(app: &mut App, task: Task<Message>) {
    use iced::futures::StreamExt;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let Some(mut stream) = iced_runtime::task::into_stream(task) else {
        return;
    };
    loop {
        let next = runtime.block_on(async {
            tokio::time::timeout(Duration::from_millis(200), stream.next()).await
        });
        match next {
            Ok(Some(iced_runtime::Action::Output(message))) => {
                let task = app.update(message);
                run(app, task);
            }
            Ok(Some(_)) => {}
            Ok(None) | Err(_) => break,
        }
    }
}

fn send(app: &mut App, message: Message) {
    let task = app.update(message);
    run(app, task);
}

fn toast_text(app: &App) -> String {
    app.toast
        .as_ref()
        .map(|t| t.text.clone())
        .unwrap_or_default()
}

// --- 2. Link targets -----------------------------------------------------------

#[test]
fn links_to_files_the_sidebar_opens_open_inside_rmdv() {
    let (mut app, dir) = fixture("targets");
    let a = dir.join("a.md");
    send(&mut app, Message::Open(a.clone()));
    assert_eq!(app.file.as_deref(), Some(a.as_path()));

    send(&mut app, Message::OpenLink("data.json".into()));
    assert_eq!(
        app.file.as_deref(),
        Some(dir.join("data.json").as_path()),
        "a JSON link opens in rmdv"
    );
    send(&mut app, Message::OpenLink("conf.yaml".into()));
    assert_eq!(
        app.file.as_deref(),
        Some(dir.join("conf.yaml").as_path()),
        "a YAML link opens in rmdv"
    );
    send(&mut app, Message::OpenLink("sub/NOTES.MD".into()));
    assert_eq!(
        app.file.as_deref(),
        Some(dir.join("sub").join("NOTES.MD").as_path()),
        "an upper-case .MD link still opens in rmdv"
    );
    assert!(app.external_opens.is_empty(), "{:?}", app.external_opens);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_link_decision_is_the_sidebar_decision() {
    for name in [
        "a.md",
        "a.markdown",
        "a.tex",
        "a.json",
        "a.yaml",
        "a.yml",
        "a.toml",
    ] {
        assert!(link_opens_in_app(Path::new(name)), "{name}");
        assert_eq!(
            link_opens_in_app(Path::new(name)),
            crate::picker::is_markdown_path(Path::new(name))
        );
    }
    assert!(link_opens_in_app(Path::new("A.JSON")));
    assert_eq!(
        link_opens_in_app(Path::new("a.pdf")),
        cfg!(feature = "pdf"),
        "PDFs open in rmdv only with the pdf feature"
    );
    for name in ["a.png", "a.zip", "Makefile"] {
        assert!(!link_opens_in_app(Path::new(name)), "{name}");
    }
}

#[test]
fn a_link_to_a_missing_file_says_file_not_found() {
    let (mut app, dir) = fixture("missing");
    let a = dir.join("a.md");
    send(&mut app, Message::Open(a.clone()));
    send(&mut app, Message::OpenLink("gone.md#part".into()));
    assert_eq!(app.file.as_deref(), Some(a.as_path()), "a.md stays open");
    assert_eq!(toast_text(&app), "File not found: gone.md");
    assert!(
        app.external_opens.is_empty(),
        "nothing is handed to the OS: {:?}",
        app.external_opens
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_external_link_opens_in_the_background_with_command_held() {
    let (mut app, dir) = fixture("external");
    send(&mut app, Message::Open(dir.join("a.md")));
    send(&mut app, Message::OpenLink("https://example.com/x".into()));
    app.quick_slots_modifier_held = true;
    send(&mut app, Message::OpenLink("https://example.com/y".into()));
    assert_eq!(
        app.external_opens,
        vec![
            ("https://example.com/x".to_string(), false),
            // Only macOS has a background open (`open -g`).
            (
                "https://example.com/y".to_string(),
                cfg!(target_os = "macos")
            ),
        ]
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_local_file_rmdv_cannot_show_goes_to_the_os_by_its_full_path() {
    let (mut app, dir) = fixture("other");
    std::fs::write(dir.join("pic.png"), b"not really").expect("pic.png");
    send(&mut app, Message::Open(dir.join("a.md")));
    send(&mut app, Message::OpenLink("pic.png".into()));
    assert_eq!(
        app.external_opens,
        vec![(dir.join("pic.png").to_string_lossy().into_owned(), false)]
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(target_os = "macos")]
#[test]
fn the_background_open_is_open_dash_g() {
    let command = background_open_command("https://example.com").expect("a command");
    assert_eq!(command.get_program(), "open");
    let args: Vec<_> = command.get_args().collect();
    assert_eq!(args, ["-g", "https://example.com"]);
    assert!(
        background_open_command("-a://evil").is_none(),
        "a target that reads as a flag is not passed to open"
    );
}
