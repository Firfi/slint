// Copyright © SixtyFPS GmbH <info@slint-ui.com>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-commercial

// cspell:ignore Noto fontconfig

use femtovg::TextContext;
use i_slint_core::graphics::euclid;
use i_slint_core::graphics::FontRequest;
use i_slint_core::items::{TextHorizontalAlignment, TextOverflow, TextVerticalAlignment, TextWrap};
use i_slint_core::lengths::{LogicalLength, LogicalSize, ScaleFactor, SizeLengths};
use i_slint_core::{SharedString, SharedVector};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use super::{PhysicalLength, PhysicalPoint, PhysicalSize};

pub const DEFAULT_FONT_SIZE: LogicalLength = LogicalLength::new(12.);
pub const DEFAULT_FONT_WEIGHT: i32 = 400; // CSS normal

#[cfg(not(any(
    target_family = "windows",
    target_os = "macos",
    target_os = "ios",
    target_arch = "wasm32"
)))]
mod fontconfig;

/// This function can be used to register a custom TrueType font with Slint,
/// for use with the `font-family` property. The provided slice must be a valid TrueType
/// font.
pub fn register_font_from_memory(data: &'static [u8]) -> Result<(), Box<dyn std::error::Error>> {
    FONT_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .available_fonts
            .load_font_source(fontdb::Source::Binary(std::sync::Arc::new(data)))
    });
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn register_font_from_path(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let requested_path = path.canonicalize().unwrap_or_else(|_| path.to_owned());
    FONT_CACHE.with(|cache| {
        for face_info in cache.borrow().available_fonts.faces() {
            match &face_info.source {
                fontdb::Source::Binary(_) => {}
                fontdb::Source::File(loaded_path) | fontdb::Source::SharedFile(loaded_path, ..) => {
                    if *loaded_path == requested_path {
                        return Ok(());
                    }
                }
            }
        }

        cache.borrow_mut().available_fonts.load_font_file(requested_path).map_err(|e| e.into())
    })
}

#[cfg(target_arch = "wasm32")]
pub fn register_font_from_path(_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    return Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Registering fonts from paths is not supported in WASM builds",
    )
    .into());
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct FontCacheKey {
    family: SharedString,
    weight: i32,
}

#[derive(Clone)]
pub struct Font {
    fonts: SharedVector<femtovg::FontId>,
    pixel_size: PhysicalLength,
    text_context: TextContext,
}

impl Font {
    pub fn init_paint(
        &self,
        letter_spacing: PhysicalLength,
        mut paint: femtovg::Paint,
    ) -> femtovg::Paint {
        paint.set_font(&self.fonts);
        paint.set_font_size(self.pixel_size.get());
        paint.set_text_baseline(femtovg::Baseline::Top);
        paint.set_letter_spacing(letter_spacing.get());
        paint
    }

    pub fn text_size(
        &self,
        letter_spacing: PhysicalLength,
        text: &str,
        max_width: Option<PhysicalLength>,
    ) -> PhysicalSize {
        let paint = self.init_paint(letter_spacing, femtovg::Paint::default());
        let font_metrics = self.text_context.measure_font(paint).unwrap();
        let mut lines = 0;
        let mut width = 0.;
        let mut start = 0;
        if let Some(max_width) = max_width {
            while start < text.len() {
                let index =
                    self.text_context.break_text(max_width.get(), &text[start..], paint).unwrap();
                if index == 0 {
                    break;
                }
                let index = start + index;
                let measure =
                    self.text_context.measure_text(0., 0., &text[start..index], paint).unwrap();
                start = index;
                lines += 1;
                width = measure.width().max(width);
            }
        } else {
            for line in text.lines() {
                let measure = self.text_context.measure_text(0., 0., line, paint).unwrap();
                lines += 1;
                width = measure.width().max(width);
            }
        }
        euclid::size2(width, lines as f32 * font_metrics.height())
    }
}

pub(crate) fn text_size(
    font_request: &i_slint_core::graphics::FontRequest,
    scale_factor: ScaleFactor,
    text: &str,
    max_width: Option<LogicalLength>,
) -> LogicalSize {
    let font =
        FONT_CACHE.with(|cache| cache.borrow_mut().font(font_request.clone(), scale_factor, text));
    let letter_spacing = font_request.letter_spacing.unwrap_or_default();
    font.text_size(letter_spacing * scale_factor, text, max_width.map(|x| x * scale_factor))
        / scale_factor
}

#[derive(Copy, Clone)]
struct LoadedFont {
    femtovg_font_id: femtovg::FontId,
    fontdb_face_id: fontdb::ID,
}

struct SharedFontData(std::sync::Arc<dyn AsRef<[u8]>>);
impl AsRef<[u8]> for SharedFontData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
}

#[derive(Default)]
struct GlyphCoverage {
    // Used to express script support for all scripts except Unknown, Common and Inherited
    // For those the detailed glyph_coverage is used instead
    supported_scripts: HashMap<unicode_script::Script, bool>,
    // Especially in characters mapped to the common script, the support varies. For example
    // '✓' and the digit '1' map to Common, but not all fonts providing digits also support the
    // check mark glyph.
    exact_glyph_coverage: HashMap<char, bool>,
}

enum GlyphCoverageCheckResult {
    Incomplete,
    Improved,
    Complete,
}

pub struct FontCache {
    loaded_fonts: HashMap<FontCacheKey, LoadedFont>,
    // for a given fontdb face id, this tells us what we've learned about the script
    // coverage of the font.
    loaded_font_coverage: HashMap<fontdb::ID, GlyphCoverage>,
    pub(crate) text_context: TextContext,
    pub(crate) available_fonts: fontdb::Database,
    available_families: HashSet<SharedString>,
    #[cfg(not(any(
        target_family = "windows",
        target_os = "macos",
        target_os = "ios",
        target_arch = "wasm32"
    )))]
    fontconfig_fallback_families: Vec<SharedString>,
}

impl Default for FontCache {
    fn default() -> Self {
        let mut font_db = fontdb::Database::new();

        #[cfg(not(any(
            target_family = "windows",
            target_os = "macos",
            target_os = "ios",
            target_arch = "wasm32"
        )))]
        let mut fontconfig_fallback_families;

        #[cfg(target_arch = "wasm32")]
        {
            let data = include_bytes!("fonts/DejaVuSans.ttf");
            font_db.load_font_data(data.to_vec());
            font_db.set_sans_serif_family("DejaVu Sans");
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            font_db.load_system_fonts();
            #[cfg(any(
                target_family = "windows",
                target_os = "macos",
                target_os = "ios",
                target_arch = "wasm32"
            ))]
            let default_sans_serif_family = "Arial";
            #[cfg(not(any(
                target_family = "windows",
                target_os = "macos",
                target_os = "ios",
                target_arch = "wasm32"
            )))]
            let default_sans_serif_family = {
                fontconfig_fallback_families = fontconfig::find_families("sans-serif")
                    .into_iter()
                    .map(|s| s.into())
                    .collect::<Vec<SharedString>>();
                fontconfig_fallback_families.remove(0)
            };
            font_db.set_sans_serif_family(default_sans_serif_family);
        }
        let available_families =
            font_db.faces().iter().map(|face_info| face_info.family.as_str().into()).collect();

        Self {
            loaded_fonts: HashMap::new(),
            loaded_font_coverage: HashMap::new(),
            text_context: Default::default(),
            available_fonts: font_db,
            available_families,
            #[cfg(not(any(
                target_family = "windows",
                target_os = "macos",
                target_os = "ios",
                target_arch = "wasm32"
            )))]
            fontconfig_fallback_families,
        }
    }
}

thread_local! {
    pub static FONT_CACHE: RefCell<FontCache> = RefCell::new(Default::default())
}

impl FontCache {
    fn load_single_font(&mut self, family: Option<&SharedString>, weight: i32) -> LoadedFont {
        let text_context = self.text_context.clone();
        let cache_key = FontCacheKey { family: family.cloned().unwrap_or_default(), weight };

        if let Some(loaded_font) = self.loaded_fonts.get(&cache_key) {
            return *loaded_font;
        }

        let family = family
            .as_ref()
            .map_or(fontdb::Family::SansSerif, |family| fontdb::Family::Name(family));

        //let now = std::time::Instant::now();
        let query = fontdb::Query {
            families: &[family],
            weight: fontdb::Weight(weight as u16),
            ..Default::default()
        };

        let fontdb_face_id = self
            .available_fonts
            .query(&query)
            .or_else(|| {
                // If the requested family could not be found, fall back to *some* family that must exist
                let mut fallback_query = query;
                fallback_query.families = &[fontdb::Family::SansSerif];
                self.available_fonts.query(&fallback_query)
            })
            .expect("there must be a sans-serif font face registered");

        // Safety: We map font files into memory that - while we never unmap them - may
        // theoretically get corrupted/truncated by another process and then we'll crash
        // and burn. In practice that should not happen though, font files are - at worst -
        // removed by a package manager and they unlink instead of truncate, even when
        // replacing files. Unlinking OTOH is safe and doesn't destroy the file mapping,
        // the backing file becomes an orphan in a special area of the file system. That works
        // on Unixy platforms and on Windows the default file flags prevent the deletion.
        #[cfg(not(target_arch = "wasm32"))]
        let (shared_data, face_index) = unsafe {
            self.available_fonts.make_shared_face_data(fontdb_face_id).expect("unable to mmap font")
        };
        #[cfg(target_arch = "wasm32")]
        let (shared_data, face_index) = self
            .available_fonts
            .face_source(fontdb_face_id)
            .map(|(source, face_index)| {
                (
                    match source {
                        fontdb::Source::Binary(data) => data.clone(),
                        // We feed only Source::Binary into fontdb on wasm
                        #[allow(unreachable_patterns)]
                        _ => unreachable!(),
                    },
                    face_index,
                )
            })
            .expect("invalid fontdb face id");

        let femtovg_font_id = text_context
            .add_shared_font_with_index(SharedFontData(shared_data), face_index)
            .unwrap();

        //println!("Loaded {:#?} in {}ms.", request, now.elapsed().as_millis());
        let new_font = LoadedFont { femtovg_font_id, fontdb_face_id };
        self.loaded_fonts.insert(cache_key, new_font);
        new_font
    }

    pub fn font(
        &mut self,
        font_request: FontRequest,
        scale_factor: ScaleFactor,
        reference_text: &str,
    ) -> Font {
        let pixel_size = font_request.pixel_size.unwrap_or(DEFAULT_FONT_SIZE) * scale_factor;
        let weight = font_request.weight.unwrap_or(DEFAULT_FONT_WEIGHT);

        let primary_font = self.load_single_font(font_request.family.as_ref(), weight);

        use unicode_script::{Script, UnicodeScript};
        // map from required script to sample character
        let mut scripts_required: HashMap<unicode_script::Script, char> = Default::default();
        let mut chars_required: HashSet<char> = Default::default();
        for ch in reference_text.chars() {
            if ch.is_control() || ch.is_whitespace() {
                continue;
            }
            let script = ch.script();
            if script == Script::Common || script == Script::Inherited || script == Script::Unknown
            {
                chars_required.insert(ch);
            } else {
                scripts_required.insert(script, ch);
            }
        }

        let mut coverage_result = self.check_and_update_script_coverage(
            &mut scripts_required,
            &mut chars_required,
            primary_font.fontdb_face_id,
        );

        //eprintln!(
        //    "coverage for {} after checking primary font: {:#?}",
        //    reference_text, scripts_required
        //);

        let fallbacks = if !matches!(coverage_result, GlyphCoverageCheckResult::Complete) {
            self.font_fallbacks_for_request(
                font_request.family.as_ref(),
                pixel_size,
                &primary_font,
                reference_text,
            )
        } else {
            Vec::new()
        };

        let fonts = core::iter::once(primary_font.femtovg_font_id)
            .chain(fallbacks.iter().filter_map(|fallback_family| {
                if matches!(coverage_result, GlyphCoverageCheckResult::Complete) {
                    return None;
                }

                let fallback_font = self.load_single_font(Some(fallback_family), weight);

                coverage_result = self.check_and_update_script_coverage(
                    &mut scripts_required,
                    &mut chars_required,
                    fallback_font.fontdb_face_id,
                );

                if matches!(coverage_result, GlyphCoverageCheckResult::Improved) {
                    Some(fallback_font.femtovg_font_id)
                } else {
                    None
                }
            }))
            .collect::<SharedVector<_>>();

        Font { fonts, text_context: self.text_context.clone(), pixel_size }
    }

    #[cfg(target_os = "macos")]
    fn font_fallbacks_for_request(
        &self,
        _family: Option<&SharedString>,
        _pixel_size: PhysicalLength,
        _primary_font: &LoadedFont,
        _reference_text: &str,
    ) -> Vec<SharedString> {
        let requested_font = match core_text::font::new_from_name(
            &_family.as_ref().map_or_else(|| "", |s| s.as_str()),
            _pixel_size.get() as f64,
        ) {
            Ok(f) => f,
            Err(_) => return vec![],
        };

        core_text::font::cascade_list_for_languages(
            &requested_font,
            &core_foundation::array::CFArray::from_CFTypes(&[]),
        )
        .iter()
        .map(|fallback_descriptor| fallback_descriptor.family_name().into())
        .filter(|family| self.is_known_family(family))
        .collect::<Vec<_>>()
    }

    #[cfg(target_os = "windows")]
    fn font_fallbacks_for_request(
        &self,
        _family: Option<&SharedString>,
        _pixel_size: PhysicalLength,
        _primary_font: &LoadedFont,
        reference_text: &str,
    ) -> Vec<SharedString> {
        let system_font_fallback = match dwrote::FontFallback::get_system_fallback() {
            Some(fallback) => fallback,
            None => return Vec::new(),
        };
        let font_collection = dwrote::FontCollection::get_system(false);
        let base_family = Some(_family.as_ref().map_or_else(|| "", |s| s.as_str()));

        let reference_text_utf16: Vec<u16> = reference_text.encode_utf16().collect();

        // Hack to implement the minimum interface for direct write. We have yet to provide the correct
        // locale (but return an empty string for now). This struct stores the number of utf-16 characters
        // so that in get_locale_name it can return that the (empty) locale applies all the characters after
        // `text_position`, by returning the count.
        struct TextAnalysisHack(u32);
        impl dwrote::TextAnalysisSourceMethods for TextAnalysisHack {
            fn get_locale_name<'a>(
                &'a self,
                text_position: u32,
            ) -> (std::borrow::Cow<'a, str>, u32) {
                ("".into(), self.0 - text_position)
            }

            // We should do better on this one, too...
            fn get_paragraph_reading_direction(
                &self,
            ) -> winapi::um::dwrite::DWRITE_READING_DIRECTION {
                winapi::um::dwrite::DWRITE_READING_DIRECTION_LEFT_TO_RIGHT
            }
        }

        let text_analysis_source = dwrote::TextAnalysisSource::from_text_and_number_subst(
            Box::new(TextAnalysisHack(reference_text_utf16.len() as u32)),
            std::borrow::Cow::Borrowed(&reference_text_utf16),
            dwrote::NumberSubstitution::new(
                winapi::um::dwrite::DWRITE_NUMBER_SUBSTITUTION_METHOD_NONE,
                "",
                true,
            ),
        );

        let mut fallback_fonts = Vec::new();

        let mut utf16_pos = 0;

        while utf16_pos < reference_text_utf16.len() {
            let fallback_result = system_font_fallback.map_characters(
                &text_analysis_source,
                utf16_pos as u32,
                (reference_text_utf16.len() - utf16_pos) as u32,
                &font_collection,
                base_family,
                dwrote::FontWeight::Regular,
                dwrote::FontStyle::Normal,
                dwrote::FontStretch::Normal,
            );

            if let Some(fallback_font) = fallback_result.mapped_font {
                let family = fallback_font.family_name().into();
                if self.is_known_family(&family) {
                    fallback_fonts.push(family)
                }
            } else {
                break;
            }

            utf16_pos += fallback_result.mapped_length;
        }

        fallback_fonts
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows"), not(target_arch = "wasm32")))]
    fn font_fallbacks_for_request(
        &self,
        _family: Option<&SharedString>,
        _pixel_size: PhysicalLength,
        _primary_font: &LoadedFont,
        _reference_text: &str,
    ) -> Vec<SharedString> {
        self.fontconfig_fallback_families
            .iter()
            .filter(|family_name| self.is_known_family(family_name))
            .cloned()
            .collect()
    }

    #[cfg(target_arch = "wasm32")]
    fn font_fallbacks_for_request(
        &self,
        _family: Option<&SharedString>,
        _pixel_size: PhysicalLength,
        _primary_font: &LoadedFont,
        _reference_text: &str,
    ) -> Vec<SharedString> {
        ["DejaVu Sans".into()]
            .iter()
            .filter(|family_name| self.is_known_family(family_name))
            .cloned()
            .collect()
    }

    fn is_known_family(&self, family: &SharedString) -> bool {
        self.available_families.contains(family)
    }

    // From the set of script without coverage, remove all entries that are known to be covered by
    // the given face_id. Any yet unknown script coverage for the face_id is updated (hence
    // mutable self).
    fn check_and_update_script_coverage(
        &mut self,
        scripts_without_coverage: &mut HashMap<unicode_script::Script, char>,
        chars_without_coverage: &mut HashSet<char>,
        face_id: fontdb::ID,
    ) -> GlyphCoverageCheckResult {
        //eprintln!("required scripts {:#?}", required_scripts);
        let coverage = self.loaded_font_coverage.entry(face_id).or_default();

        let mut scripts_that_need_checking = Vec::new();
        let mut chars_that_need_checking = Vec::new();

        let old_uncovered_scripts_count = scripts_without_coverage.len();
        let old_uncovered_chars_count = chars_without_coverage.len();

        scripts_without_coverage.retain(|script, sample| {
            coverage.supported_scripts.get(script).map_or_else(
                || {
                    scripts_that_need_checking.push((*script, *sample));
                    true // this may or may not be supported, so keep it in scripts_without_coverage
                },
                |has_coverage| !has_coverage,
            )
        });

        chars_without_coverage.retain(|ch| {
            coverage.exact_glyph_coverage.get(ch).map_or_else(
                || {
                    chars_that_need_checking.push(*ch);
                    true // this may or may not be supported, so keep it in chars_without_coverage
                },
                |has_coverage| !has_coverage,
            )
        });

        if !scripts_that_need_checking.is_empty() || !chars_that_need_checking.is_empty() {
            self.available_fonts.with_face_data(face_id, |face_data, face_index| {
                let face = ttf_parser::Face::parse(face_data, face_index).unwrap();

                for (unchecked_script, sample_char) in scripts_that_need_checking {
                    let glyph_coverage = face.glyph_index(sample_char).is_some();
                    coverage.supported_scripts.insert(unchecked_script, glyph_coverage);

                    if glyph_coverage {
                        scripts_without_coverage.remove(&unchecked_script);
                    }
                }

                for unchecked_char in chars_that_need_checking {
                    let glyph_coverage = face.glyph_index(unchecked_char).is_some();
                    coverage.exact_glyph_coverage.insert(unchecked_char, glyph_coverage);

                    if glyph_coverage {
                        chars_without_coverage.remove(&unchecked_char);
                    }
                }
            });
        }

        let remaining_required_script_coverage = scripts_without_coverage.len();
        let remaining_required_char_coverage = chars_without_coverage.len();

        if scripts_without_coverage.is_empty() && chars_without_coverage.is_empty() {
            GlyphCoverageCheckResult::Complete
        } else if remaining_required_script_coverage < old_uncovered_scripts_count
            || remaining_required_char_coverage < old_uncovered_chars_count
        {
            GlyphCoverageCheckResult::Improved
        } else {
            GlyphCoverageCheckResult::Incomplete
        }
    }
}

/// Layout the given string in lines, and call the `layout_line` callback with the line to draw at position y.
/// The signature of the `layout_line` function is: `(text, pos, start_index, line_metrics)`.
/// start index is the starting byte of the text in the string.
/// Returns the y coordinate of where to place the cursor if it is at the end of the text
pub(crate) fn layout_text_lines(
    string: &str,
    font: &Font,
    max_size: PhysicalSize,
    (horizontal_alignment, vertical_alignment): (TextHorizontalAlignment, TextVerticalAlignment),
    wrap: TextWrap,
    overflow: TextOverflow,
    single_line: bool,
    paint: femtovg::Paint,
    mut layout_line: impl FnMut(&str, PhysicalPoint, usize, &femtovg::TextMetrics),
) -> PhysicalLength {
    let wrap = wrap == TextWrap::WordWrap;
    let elide = overflow == TextOverflow::Elide;

    let max_width = max_size.width_length();
    let max_height = max_size.height_length();

    let text_context = FONT_CACHE.with(|cache| cache.borrow().text_context.clone());
    let font_metrics = text_context.measure_font(paint).unwrap();
    let font_height = PhysicalLength::new(font_metrics.height());

    let text_height = || {
        if single_line {
            font_height
        } else {
            // Note: this is kind of doing twice the layout because text_size also does it
            font.text_size(
                PhysicalLength::new(paint.letter_spacing()),
                string,
                if wrap { Some(max_width) } else { None },
            )
            .height_length()
        }
    };

    let mut process_line =
        |text: &str, y: PhysicalLength, start: usize, line_metrics: &femtovg::TextMetrics| {
            let x = match horizontal_alignment {
                TextHorizontalAlignment::Left => PhysicalLength::default(),
                TextHorizontalAlignment::Center => {
                    max_width / 2. - max_width.min(PhysicalLength::new(line_metrics.width())) / 2.
                }
                TextHorizontalAlignment::Right => {
                    max_width - max_width.min(PhysicalLength::new(line_metrics.width()))
                }
            };
            layout_line(text, PhysicalPoint::from_lengths(x, y), start, line_metrics);
        };

    let baseline_y = match vertical_alignment {
        TextVerticalAlignment::Top => PhysicalLength::default(),
        TextVerticalAlignment::Center => max_height / 2. - text_height() / 2.,
        TextVerticalAlignment::Bottom => max_height - text_height(),
    };
    let mut y = baseline_y;
    let mut start = 0;
    'lines: while start < string.len() && y + font_height <= max_height {
        if wrap && (!elide || y + font_height * 2. <= max_height) {
            let index = text_context.break_text(max_width.get(), &string[start..], paint).unwrap();
            if index == 0 {
                // FIXME the word is too big to be shown, but we should still break, ideally
                break;
            }
            let index = start + index;
            let line = &string[start..index];
            let text_metrics = text_context.measure_text(0., 0., line, paint).unwrap();
            process_line(line, y, start, &text_metrics);
            y += font_height;
            start = index;
        } else {
            let index = if single_line {
                string.len()
            } else {
                string[start..].find('\n').map_or(string.len(), |i| start + i + 1)
            };
            let line = &string[start..index];
            let text_metrics = text_context.measure_text(0., 0., line, paint).unwrap();
            let elide_last_line =
                elide && index < string.len() && y + font_height * 2. > max_height;
            if text_metrics.width() > max_width.get() || elide_last_line {
                let w = max_width
                    - if elide {
                        PhysicalLength::new(
                            text_context.measure_text(0., 0., "…", paint).unwrap().width(),
                        )
                    } else {
                        PhysicalLength::default()
                    };
                let mut current_x = 0.;
                for glyph in &text_metrics.glyphs {
                    current_x += glyph.advance_x;
                    if current_x >= w.get() {
                        let txt = &line[..glyph.byte_index];
                        if elide {
                            let elided = format!("{}…", txt);
                            process_line(&elided, y, start, &text_metrics);
                        } else {
                            process_line(txt, y, start, &text_metrics);
                        }
                        y += font_height;
                        start = index;
                        continue 'lines;
                    }
                }
                if elide_last_line {
                    let elided = format!("{}…", line);
                    process_line(&elided, y, start, &text_metrics);
                    y += font_height;
                    start = index;
                    continue 'lines;
                }
            }
            process_line(line, y, start, &text_metrics);
            y += font_height;
            start = index;
        }
    }
    y
}
