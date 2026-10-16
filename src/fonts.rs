//! Fonts bundled into the binary, registered with Iced at startup.
//!
//! cosmic-text only accepts a face whose weight equals the requested weight
//! (monospace faces excepted), and fontdb reads a variable font as a single
//! face at its default weight. So `Inter-Variable.ttf` serves weight 400 only;
//! the static instances below (cut from it at wght 500/600/700, subset to
//! Latin, Greek, Cyrillic, punctuation, arrows and math) serve the others.
//! Without them a bold or medium "Inter" run falls through to JetBrains Mono.
//! Inter is licensed under the SIL OFL 1.1; see `assets/fonts/Inter-OFL.txt`.
//!
//! `NotoSansTC-{Regular,Bold}.ttf` carry CJK text (Traditional forms), since
//! the macOS system fallback resolves to Arial Unicode MS. They are static
//! 400/700 instances of Noto Sans TC (SIL OFL 1.1, see
//! `assets/fonts/NotoSansTC-OFL.txt`) subset to Big5 common hanzi, GB2312
//! level-1 hanzi, CJK and fullwidth punctuation, kana and bopomofo; rarer
//! characters still fall back to the system. Each also holds half-width
//! copies of the brackets and stops (the font's own `halt` shapes) at
//! U+E000.., used by [`squeeze_fullwidth_punct`]. cosmic-text shapes every
//! punctuation mark as its own word, so OpenType pair kerning cannot do this.

pub const BUNDLED: [&[u8]; 8] = [
    include_bytes!("assets/fonts/Inter-Variable.ttf"),
    include_bytes!("assets/fonts/Inter-Medium.ttf"),
    include_bytes!("assets/fonts/Inter-SemiBold.ttf"),
    include_bytes!("assets/fonts/Inter-Bold.ttf"),
    include_bytes!("assets/fonts/JetBrainsMono-Regular.otf"),
    include_bytes!("assets/fonts/lucide.ttf"),
    include_bytes!("assets/fonts/NotoSansTC-Regular.ttf"),
    include_bytes!("assets/fonts/NotoSansTC-Bold.ttf"),
];

/// Characters that have a half-width copy in the bundled CJK face, in the
/// order of their private-use code points (`U+E000 + index`): opening
/// brackets, closing brackets, then stops.
const SQUEEZABLE: &str = "「『（【〔〈《［｛」』）】〕〉》］｝，。、．";
const OPENING: usize = 9;

fn is_opening(c: char) -> bool {
    SQUEEZABLE.chars().take(OPENING).any(|o| o == c)
}

fn squeezed_form(c: char) -> Option<char> {
    SQUEEZABLE
        .chars()
        .position(|o| o == c)
        .and_then(|i| char::from_u32(0xE000 + i as u32))
}

/// Squeeze adjacent fullwidth punctuation to half width (CSS
/// `text-spacing-trim: normal`): a closing bracket or stop followed by
/// another bracket or stop, and an opening bracket followed by an opening
/// bracket, are swapped for their half-width copy. Every swapped character
/// stays 3 UTF-8 bytes, so byte offsets into the text do not move.
pub fn squeeze_fullwidth_punct(text: &str) -> std::borrow::Cow<'_, str> {
    let mut out: Option<String> = None;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let next = chars.peek().map(|&(_, n)| n);
        let squeeze = match next {
            Some(n) if squeezed_form(n).is_some() => {
                if is_opening(c) {
                    is_opening(n)
                } else {
                    true
                }
            }
            _ => false,
        };
        let replacement = if squeeze { squeezed_form(c) } else { None };
        match (replacement, out.as_mut()) {
            (Some(r), None) => {
                let mut s = String::with_capacity(text.len());
                s.push_str(&text[..i]);
                s.push(r);
                out = Some(s);
            }
            (Some(r), Some(s)) => s.push(r),
            (None, Some(s)) => s.push(c),
            (None, None) => {}
        }
    }
    match out {
        Some(s) => s.into(),
        None => text.into(),
    }
}

/// Family name of the bundled CJK face (Noto Sans TC, Traditional forms).
pub const CJK_FAMILY: &str = "Noto Sans TC";

#[cfg(test)]
mod tests {
    use super::BUNDLED;
    use iced::advanced::graphics::text::cosmic_text::{
        fontdb, Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Weight,
    };
    use std::sync::Arc;

    /// Shape `text` the way Iced does (family "Inter" at `weight`, only the
    /// bundled fonts loaded) and return the family of each glyph's face.
    fn shaped_families(text: &str, weight: Weight) -> Vec<String> {
        shaped_families_in("Inter", text, weight)
    }

    fn shaped_families_in(family: &'static str, text: &str, weight: Weight) -> Vec<String> {
        let sources = BUNDLED
            .iter()
            .map(|bytes| fontdb::Source::Binary(Arc::new(*bytes)));
        let mut fonts = FontSystem::new_with_fonts(sources);
        let mut buffer = Buffer::new(&mut fonts, Metrics::new(16.0, 20.0));
        buffer.set_text(
            &mut fonts,
            text,
            &Attrs::new().family(Family::Name(family)).weight(weight),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut fonts, false);
        let mut out = Vec::new();
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let face = fonts.db().face(glyph.font_id).expect("face");
                out.push(face.families[0].0.clone());
            }
        }
        out
    }

    #[test]
    fn inter_weights_resolve_to_inter_not_the_monospace_fallback() {
        for weight in [
            Weight::NORMAL,
            Weight::MEDIUM,
            Weight::SEMIBOLD,
            Weight::BOLD,
        ] {
            let families = shaped_families("Plain bold words", weight);
            assert!(!families.is_empty());
            assert!(
                families.iter().all(|f| f == "Inter"),
                "weight {weight:?} shaped with {families:?}"
            );
        }
    }

    /// Sum of glyph advances (px at 16 px) for `text` shaped in the bundled CJK face.
    fn cjk_advance(text: &str) -> f32 {
        let text = &*super::squeeze_fullwidth_punct(text);
        let sources = BUNDLED
            .iter()
            .map(|bytes| fontdb::Source::Binary(Arc::new(*bytes)));
        let mut fonts = FontSystem::new_with_fonts(sources);
        let mut buffer = Buffer::new(&mut fonts, Metrics::new(16.0, 24.0));
        buffer.set_size(&mut fonts, Some(100_000.0), None);
        buffer.set_text(
            &mut fonts,
            text,
            &Attrs::new()
                .family(Family::Name(super::CJK_FAMILY))
                .weight(Weight::NORMAL),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut fonts, false);
        buffer
            .layout_runs()
            .flat_map(|run| run.glyphs.iter().map(|g| g.w).collect::<Vec<_>>())
            .sum()
    }

    #[test]
    fn cjk_text_resolves_to_the_bundled_noto_sans_tc() {
        for weight in [Weight::NORMAL, Weight::BOLD] {
            let families = shaped_families_in(super::CJK_FAMILY, "閱讀體驗，中文。", weight);
            assert!(!families.is_empty());
            assert!(
                families.iter().all(|f| f == "Noto Sans TC"),
                "weight {weight:?} shaped with {families:?}"
            );
        }
    }

    #[test]
    fn adjacent_fullwidth_brackets_and_stops_are_squeezed_to_half() {
        let eps = 0.01;
        // Closing + opening, closing + closing, opening + opening: first glyph halves.
        assert!(
            (cjk_advance("」「") - 24.0).abs() < eps,
            "{}",
            cjk_advance("」「")
        );
        assert!(
            (cjk_advance("）」") - 24.0).abs() < eps,
            "{}",
            cjk_advance("）」")
        );
        assert!(
            (cjk_advance("（「") - 24.0).abs() < eps,
            "{}",
            cjk_advance("（「")
        );
        assert!(
            (cjk_advance("。「") - 24.0).abs() < eps,
            "{}",
            cjk_advance("。「")
        );
        // Not adjacent to other punctuation: stay full width.
        assert!(
            (cjk_advance("「這」") - 48.0).abs() < eps,
            "{}",
            cjk_advance("「這」")
        );
        assert!(
            (cjk_advance("這。這") - 48.0).abs() < eps,
            "{}",
            cjk_advance("這。這")
        );
        assert!((cjk_advance("。") - 16.0).abs() < eps);
    }

    #[test]
    fn squeeze_only_touches_adjacent_punctuation_and_keeps_byte_length() {
        use super::squeeze_fullwidth_punct as sq;
        assert_eq!(sq("「這」"), "「這」");
        assert_eq!(sq("沒有標點"), "沒有標點");
        for s in ["」「", "）」", "（「", "。「", "他說：「好」。「對」"] {
            let out = sq(s);
            assert_eq!(out.len(), s.len(), "{s}");
            assert_ne!(out, s, "{s}");
        }
        // Opening followed by closing is left alone; colon is not squeezed.
        assert_eq!(sq("「」"), "「」");
        assert_eq!(sq("：「"), "：「");
    }
}
