//! Cost of selectable reader text against Iced's plain `rich_text`.
//!
//! Simulates the reader scrolling a 10k-line document: each frame builds the
//! visible window of paragraphs, diffs it against the previous frame's tree,
//! lays it out, and feeds a cursor move. Run by hand:
//!
//! `ICED_TEST_BACKEND=tiny-skia cargo test --release --test selection_cost -- --ignored --nocapture`
//!
//! This file owns its binary, so its counting allocator sees only this test.

use iced::advanced::clipboard;
use iced::advanced::renderer::Headless;
use iced::advanced::text::Span;
use iced::mouse;
use iced::widget::{column, span};
use iced::{Element, Event, Font, Pixels, Point, Size};
use iced_runtime::user_interface::{Cache, UserInterface};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

struct Counting;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let live = LIVE.fetch_add(layout.size(), Ordering::SeqCst) + layout.size();
            PEAK.fetch_max(live, Ordering::SeqCst);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        LIVE.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

const DOC_PARAGRAPHS: usize = 5_000; // ~10k source lines with blank separators
const WINDOW: usize = 60; // visible blocks plus virt padding
const FRAMES: usize = 400;

fn paragraphs() -> Vec<String> {
    (0..DOC_PARAGRAPHS)
        .map(|i| {
            format!(
                "Paragraph {i}: the quick brown fox jumps over the lazy dog while \
                 rmdv renders markdown with virtual scrolling and rich text spans. "
            )
            .repeat(2)
        })
        .collect()
}

fn spans_for(text: &str) -> Vec<Span<'_, (), Font>> {
    let (a, b) = text.split_at(text.len() / 2);
    vec![span(a), span(b).font(Font::MONOSPACE).link(())]
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Plain,
    Selectable,
}

fn window_view(doc: &[String], start: usize, kind: Kind) -> Element<'_, ()> {
    let mut col = column![].spacing(12).width(720);
    for text in &doc[start..start + WINDOW] {
        let el: Element<'_, ()> = match kind {
            Kind::Plain => iced::widget::rich_text(spans_for(text))
                .on_link_click(|m| m)
                .into(),
            Kind::Selectable => iced_selection::rich_text(spans_for(text))
                .on_link_click(|m| m)
                .into(),
        };
        col = col.push(el);
    }
    col.into()
}

fn measure(kind: Kind, doc: &[String]) -> (Duration, usize, usize) {
    let mut renderer = iced::futures::executor::block_on(<iced::Renderer as Headless>::new(
        Font::DEFAULT,
        Pixels(16.0),
        Some("tiny-skia"),
    ))
    .expect("headless renderer");
    let size = Size::new(800.0, 900.0);
    let mut cache = Cache::default();
    let mut messages = Vec::new();
    let base = LIVE.load(Ordering::SeqCst);
    PEAK.store(base, Ordering::SeqCst);
    let started = Instant::now();
    for frame in 0..FRAMES {
        let start = (frame * 7) % (DOC_PARAGRAPHS - WINDOW);
        let mut ui =
            UserInterface::build(window_view(doc, start, kind), size, cache, &mut renderer);
        let at = Point::new(100.0, 300.0 + (frame % 50) as f32);
        let _ = ui.update(
            &[Event::Mouse(mouse::Event::CursorMoved { position: at })],
            mouse::Cursor::Available(at),
            &mut renderer,
            &mut clipboard::Null,
            &mut messages,
        );
        cache = ui.into_cache();
    }
    let elapsed = started.elapsed();
    let retained = LIVE.load(Ordering::SeqCst).saturating_sub(base);
    let peak = PEAK.load(Ordering::SeqCst).saturating_sub(base);
    drop(cache);
    (elapsed, retained, peak)
}

#[test]
#[ignore = "manual measurement"]
fn selectable_text_costs_about_the_same_as_plain_rich_text() {
    let doc = paragraphs();
    // Warm the font system and glyph caches so the first variant is not
    // charged for them.
    let _ = measure(Kind::Plain, &doc);
    for kind in [Kind::Plain, Kind::Selectable, Kind::Plain, Kind::Selectable] {
        let (t, retained, peak) = measure(kind, &doc);
        println!(
            "{kind:?}: {FRAMES} frames in {:.1} ms ({:.2} ms/frame), retained {:.1} KiB, peak {:.1} KiB",
            t.as_secs_f64() * 1e3,
            t.as_secs_f64() * 1e3 / FRAMES as f64,
            retained as f64 / 1024.0,
            peak as f64 / 1024.0,
        );
    }
}
