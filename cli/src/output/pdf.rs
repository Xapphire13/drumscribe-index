use std::{
    fs::File,
    io::{BufWriter, Cursor},
    path::Path,
};

use ab_glyph::{Font as _, FontVec, PxScale, ScaleFont as _};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use printpdf::{
    Color, IndirectFontRef, Mm, PdfDocument, PdfLayerReference, Point, Polygon,
    PolygonMode, Rgb, WindingOrder,
};

use crate::{
    group_songs,
    models::song::{Difficulty, Song},
};

// Page dimensions (US Letter)
const PAGE_W: f64 = 215.9;
const PAGE_H: f64 = 279.4;

// Margins and column layout
const MARGIN: f64 = 15.0;
const COL_GAP: f64 = 8.0;
const COL_W: f64 = (PAGE_W - 2.0 * MARGIN - COL_GAP) / 2.0;

// Row heights
const ARTIST_H: f64 = 8.0; // artist header bar
const ROW_H: f64 = 5.5; // one song row (base height, multiplied for wrapped lines)

// Gap between consecutive artist groups (CSS: margin-top: 12pt ≈ 4.23mm)
const GROUP_GAP: f64 = 4.0;

// Corner radius for artist header bars (CSS: border-radius: 4pt ≈ 1.41mm)
const CORNER_RADIUS: f64 = 1.5;
// Bezier approximation constant for quarter-circle
const BEZIER_K: f64 = 0.552_284_75;

// Song row column zones (within COL_W)
const SEQ_ZONE: f64 = 12.0; // "#437" — controls gap between stars and seq number
const STARS_ZONE: f64 = 14.0; // "★★★" right-aligned
const TITLE_ZONE: f64 = COL_W - STARS_ZONE - SEQ_ZONE;

// Extra space below artist header bar before first song row
const ARTIST_BOTTOM_PAD: f64 = 1.5;

// Font sizes (points)
const PT_TITLE: f32 = 20.0;
const PT_SUBTITLE: f32 = 9.0;
const PT_ARTIST: f32 = 10.0;
const PT_SONG: f32 = 9.0;

// System font paths (macOS)
const FONT_REGULAR: &str = "/System/Library/Fonts/Supplemental/Arial.ttf";
const FONT_BOLD: &str = "/System/Library/Fonts/Supplemental/Arial Bold.ttf";
const FONT_SYMBOL: &str = "/System/Library/Fonts/SFCompact.ttf"; // supports ★

// Padding within cells
const H_PAD: f64 = 1.5; // horizontal left padding for text

// Page 1 header section height (title + subtitle + gap before columns)
const PAGE1_HEADER_H: f64 = 22.0;

enum LayoutItem {
    ArtistHeader {
        artist: String,
    },
    SongRow {
        title_lines: Vec<String>,
        stars: &'static str,
        /// True when stars is "—" (use regular font instead of symbol)
        is_unrated: bool,
        seq_num: String,
        /// Offset from column left where "#" should be drawn, so the widest
        /// number in the group ends H_PAD from the right edge.
        seq_col_offset: f64,
        is_alternate: bool,
        height_mm: f64,
    },
}

impl LayoutItem {
    fn height_mm(&self) -> f64 {
        match self {
            Self::ArtistHeader { .. } => ARTIST_H + ARTIST_BOTTOM_PAD,
            Self::SongRow { height_mm, .. } => *height_mm,
        }
    }
}

struct Fonts {
    regular: IndirectFontRef,
    bold: IndirectFontRef,
    symbol: IndirectFontRef,
    ab_regular: FontVec,
    ab_symbol: FontVec,
}

struct LayoutCursor {
    column: u8,        // 0 = left, 1 = right
    y_mm: f64,         // y from page top for next item
    content_top: f64,  // top of content area for current page (changes after page 1)
    at_col_top: bool,  // true when nothing has been placed in the current column yet
}

impl LayoutCursor {
    fn col_x(&self) -> f64 {
        match self.column {
            0 => MARGIN,
            _ => MARGIN + COL_W + COL_GAP,
        }
    }

    fn remaining(&self) -> f64 {
        (PAGE_H - MARGIN) - self.y_mm
    }

    /// Advance to next column. Returns true if a new page is needed.
    fn advance(&mut self) -> bool {
        self.at_col_top = true;
        if self.column == 0 {
            self.column = 1;
            self.y_mm = MARGIN; // right column always starts at top of page
            false
        } else {
            self.column = 0;
            self.content_top = MARGIN; // pages 2+ have no header
            self.y_mm = MARGIN;
            true
        }
    }
}

/// Convert y-from-page-top to printpdf's bottom-origin coordinate.
fn pdf_y(y_from_top: f64) -> Mm {
    Mm((PAGE_H - y_from_top) as f32)
}

fn load_font(path: &str) -> Result<Vec<u8>> {
    std::fs::read(path).with_context(|| format!("Failed to load font: {path}"))
}

fn measure_mm(ab_font: &FontVec, text: &str, pt: f32) -> f64 {
    let scale = PxScale::from(pt);
    let scaled = ab_font.as_scaled(scale);
    let mut width: f32 = 0.0;
    let mut prev_glyph = None;
    for ch in text.chars() {
        let gid = ab_font.glyph_id(ch);
        if let Some(prev) = prev_glyph {
            width += scaled.kern(prev, gid);
        }
        width += scaled.h_advance(gid);
        prev_glyph = Some(gid);
    }
    // PxScale at value N means N px/em at 72dpi → 1px = 1pt = 0.352778mm
    width as f64 * 0.352778
}

fn wrap_text(ab_font: &FontVec, text: &str, pt: f32, max_mm: f64) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in words {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };

        if measure_mm(ab_font, &candidate, pt) <= max_mm {
            current = candidate;
        } else if current.is_empty() {
            // Single word too long — just use it anyway
            lines.push(word.to_string());
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

fn difficulty_stars(d: &Difficulty) -> (&'static str, bool) {
    match d {
        Difficulty::Beginner => ("★", false),
        Difficulty::Intermediate => ("★★", false),
        Difficulty::Advanced => ("★★★", false),
        Difficulty::Expert => ("★★★★", false),
        Difficulty::Master => ("★★★★★", false),
        Difficulty::Unrated => ("—", true), // use regular font for dash
    }
}

fn build_layout(songs: &[Song], fonts: &Fonts) -> Vec<LayoutItem> {
    let groups = group_songs(songs);
    let mut items = Vec::new();

    for group in groups {
        items.push(LayoutItem::ArtistHeader {
            artist: group.artist.clone(),
        });

        // Find the widest "#NNN" string in this group to align all seq numbers.
        let max_seq_width = group
            .songs
            .iter()
            .map(|s| measure_mm(&fonts.ab_regular, &format!("#{}", s.sequence_number), PT_SONG))
            .fold(0.0_f64, f64::max);
        let seq_col_offset = COL_W - H_PAD - max_seq_width;

        for (i, song) in group.songs.iter().enumerate() {
            let title_lines =
                wrap_text(&fonts.ab_regular, &song.title, PT_SONG, TITLE_ZONE - H_PAD);
            let line_count = title_lines.len().max(1);
            let (stars, is_unrated) = difficulty_stars(&song.difficulty);
            items.push(LayoutItem::SongRow {
                height_mm: ROW_H * line_count as f64,
                title_lines,
                stars,
                is_unrated,
                seq_num: song.sequence_number.clone(),
                seq_col_offset,
                is_alternate: i % 2 == 1,
            });
        }
    }

    items
}

/// Draw a filled rectangle with rounded corners (radius = CORNER_RADIUS).
/// Uses cubic Bézier curves for each corner.
fn filled_rounded_rect(layer: &PdfLayerReference, x: f64, y_top: f64, w: f64, h: f64) {
    let r = CORNER_RADIUS;
    let k = BEZIER_K * r;

    let x1 = x as f32;
    let x2 = (x + w) as f32;
    // PDF y-axis is bottom-up; y_tp > y_bp
    let y_tp = (PAGE_H - y_top) as f32;
    let y_bp = (PAGE_H - (y_top + h)) as f32;

    let r = r as f32;
    let k = k as f32;

    // Points for a rounded rectangle, going counterclockwise in PDF coords
    // (clockwise visually). Each corner: anchor(true) + cp1(true) + cp2(false) + end(false).
    let pts: Vec<(Point, bool)> = vec![
        // Start: top edge, after top-left arc
        (Point::new(Mm(x1 + r), Mm(y_tp)), false),
        // Top edge →
        (Point::new(Mm(x2 - r), Mm(y_tp)), false),
        // Top-right corner arc (→ then ↓)
        (Point::new(Mm(x2 - r), Mm(y_tp)), true),
        (Point::new(Mm(x2 - r + k), Mm(y_tp)), true),
        (Point::new(Mm(x2), Mm(y_tp - r + k)), false),
        (Point::new(Mm(x2), Mm(y_tp - r)), false),
        // Right edge ↓
        (Point::new(Mm(x2), Mm(y_bp + r)), false),
        // Bottom-right corner arc (↓ then ←)
        (Point::new(Mm(x2), Mm(y_bp + r)), true),
        (Point::new(Mm(x2), Mm(y_bp + r - k)), true),
        (Point::new(Mm(x2 - r + k), Mm(y_bp)), false),
        (Point::new(Mm(x2 - r), Mm(y_bp)), false),
        // Bottom edge ←
        (Point::new(Mm(x1 + r), Mm(y_bp)), false),
        // Bottom-left corner arc (← then ↑)
        (Point::new(Mm(x1 + r), Mm(y_bp)), true),
        (Point::new(Mm(x1 + r - k), Mm(y_bp)), true),
        (Point::new(Mm(x1), Mm(y_bp + r - k)), false),
        (Point::new(Mm(x1), Mm(y_bp + r)), false),
        // Left edge ↑
        (Point::new(Mm(x1), Mm(y_tp - r)), false),
        // Top-left corner arc (↑ then →)
        (Point::new(Mm(x1), Mm(y_tp - r)), true),
        (Point::new(Mm(x1), Mm(y_tp - r + k)), true),
        (Point::new(Mm(x1 + r - k), Mm(y_tp)), false),
        (Point::new(Mm(x1 + r), Mm(y_tp)), false),
    ];

    let polygon = Polygon {
        rings: vec![pts],
        mode: PolygonMode::Fill,
        winding_order: WindingOrder::NonZero,
    };
    layer.add_polygon(polygon);
}

fn filled_rect(layer: &PdfLayerReference, x: f64, y_top: f64, w: f64, h: f64) {
    let x1 = x as f32;
    let x2 = (x + w) as f32;
    let y_top_pdf = (PAGE_H - y_top) as f32;
    let y_bot_pdf = (PAGE_H - (y_top + h)) as f32;

    let polygon = Polygon {
        rings: vec![vec![
            (Point::new(Mm(x1), Mm(y_top_pdf)), false),
            (Point::new(Mm(x2), Mm(y_top_pdf)), false),
            (Point::new(Mm(x2), Mm(y_bot_pdf)), false),
            (Point::new(Mm(x1), Mm(y_bot_pdf)), false),
        ]],
        mode: PolygonMode::Fill,
        winding_order: WindingOrder::NonZero,
    };
    layer.add_polygon(polygon);
}

fn draw_page_header(layer: &PdfLayerReference, fonts: &Fonts, last_indexed: DateTime<Utc>) {
    // Title: "Drumscribe Index" bold 20pt
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    layer.use_text(
        "Drumscribe Index",
        PT_TITLE,
        Mm(MARGIN as f32),
        pdf_y(MARGIN + 7.5),
        &fonts.bold,
    );

    // Subtitle: "Last indexed: ..." regular 9pt
    let subtitle = format!(
        "Last indexed: {}",
        last_indexed.format("%-d-%b-%Y %-I:%M:%S %p %Z")
    );
    layer.use_text(
        subtitle,
        PT_SUBTITLE,
        Mm(MARGIN as f32),
        pdf_y(MARGIN + 14.5),
        &fonts.regular,
    );
}

fn draw_artist_header(
    layer: &PdfLayerReference,
    fonts: &Fonts,
    cursor: &LayoutCursor,
    artist: &str,
) {
    let x = cursor.col_x();
    let y_top = cursor.y_mm;

    // Dark background with rounded corners
    layer.set_fill_color(Color::Rgb(Rgb::new(0.2, 0.2, 0.2, None)));
    filled_rounded_rect(layer, x, y_top, COL_W, ARTIST_H);

    // White artist name — vertically centered at ~62% from top of bar
    let text_y = y_top + ARTIST_H * 0.62;
    layer.set_fill_color(Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None)));
    layer.use_text(
        artist,
        PT_ARTIST,
        Mm((x + H_PAD) as f32),
        pdf_y(text_y),
        &fonts.bold,
    );
}

fn draw_song_row(
    layer: &PdfLayerReference,
    fonts: &Fonts,
    cursor: &LayoutCursor,
    title_lines: &[String],
    stars: &str,
    is_unrated: bool,
    seq_num: &str,
    seq_col_offset: f64,
    is_alternate: bool,
    height_mm: f64,
) {
    let x = cursor.col_x();
    let y_top = cursor.y_mm;

    // Alternating row background
    if is_alternate {
        layer.set_fill_color(Color::Rgb(Rgb::new(0.961, 0.961, 0.961, None)));
        filled_rect(layer, x, y_top, COL_W, height_mm);
    }

    // Draw each wrapped title line
    layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
    for (i, line) in title_lines.iter().enumerate() {
        let line_y = y_top + ROW_H * i as f64 + ROW_H * 0.65;
        layer.use_text(
            line.as_str(),
            PT_SONG,
            Mm((x + H_PAD) as f32),
            pdf_y(line_y),
            &fonts.regular,
        );
    }

    let first_line_y = y_top + ROW_H * 0.65;

    // Stars / dash — right-aligned in stars zone (first line only).
    // Use regular font for "—" (em dash is in Arial); symbol font for ★.
    let stars_font = if is_unrated { &fonts.regular } else { &fonts.symbol };
    let stars_ab = if is_unrated {
        &fonts.ab_regular
    } else {
        &fonts.ab_symbol
    };
    let stars_zone_x = x + TITLE_ZONE;
    let stars_width = measure_mm(stars_ab, stars, PT_SONG);
    let stars_x = stars_zone_x + STARS_ZONE - stars_width - 0.5;
    layer.use_text(
        stars,
        PT_SONG,
        Mm(stars_x as f32),
        pdf_y(first_line_y),
        stars_font,
    );

    // Sequence number — left-aligned so "#" lines up and widest number ends H_PAD from edge; color #666
    layer.set_fill_color(Color::Rgb(Rgb::new(0.4, 0.4, 0.4, None)));
    let seq_x = x + seq_col_offset;
    layer.use_text(
        format!("#{seq_num}"),
        PT_SONG,
        Mm(seq_x as f32),
        pdf_y(first_line_y),
        &fonts.regular,
    );
}

pub struct PdfFormatter;

impl PdfFormatter {
    pub fn format_to_file(songs: &[Song], last_indexed: DateTime<Utc>, path: &str) -> Result<()> {
        // Load font bytes
        let regular_bytes = load_font(FONT_REGULAR)?;
        let bold_bytes = load_font(FONT_BOLD)?;
        let symbol_bytes = load_font(FONT_SYMBOL)?;

        // Load ab_glyph fonts for text measurement
        let ab_regular =
            FontVec::try_from_vec(regular_bytes.clone()).context("Failed to parse regular font")?;
        let ab_symbol =
            FontVec::try_from_vec(symbol_bytes.clone()).context("Failed to parse symbol font")?;

        // Create PDF document
        let (doc, page1, layer1) =
            PdfDocument::new("Drumscribe Index", Mm(PAGE_W as f32), Mm(PAGE_H as f32), "Layer 1");

        // Embed fonts into PDF
        let regular = doc
            .add_external_font(&mut Cursor::new(&regular_bytes))
            .context("Failed to embed regular font")?;
        let bold = doc
            .add_external_font(&mut Cursor::new(&bold_bytes))
            .context("Failed to embed bold font")?;
        let symbol = doc
            .add_external_font(&mut Cursor::new(&symbol_bytes))
            .context("Failed to embed symbol font")?;

        let fonts = Fonts {
            regular,
            bold,
            symbol,
            ab_regular,
            ab_symbol,
        };

        // Pre-calculate layout
        let items = build_layout(songs, &fonts);

        // Draw page 1 header
        let layer = doc.get_page(page1).get_layer(layer1);
        draw_page_header(&layer, &fonts, last_indexed);

        // Cursor starts below the header on page 1
        let page1_content_top = MARGIN + PAGE1_HEADER_H;
        let mut cursor = LayoutCursor {
            column: 0,
            y_mm: page1_content_top,
            content_top: page1_content_top,
            at_col_top: true,
        };
        let mut current_layer = layer;

        for item in &items {
            let item_h = item.height_mm();

            // For artist headers: add group gap when not at the top of a column,
            // and ensure the header isn't orphaned (needs at least one song row below it).
            let (gap, min_needed) = match item {
                LayoutItem::ArtistHeader { .. } => {
                    let g = if cursor.at_col_top { 0.0 } else { GROUP_GAP };
                    (g, g + ARTIST_H + ROW_H)
                }
                LayoutItem::SongRow { .. } => (0.0, item_h),
            };

            if cursor.remaining() < min_needed {
                let needs_new_page = cursor.advance();
                if needs_new_page {
                    let (new_page, new_layer) =
                        doc.add_page(Mm(PAGE_W as f32), Mm(PAGE_H as f32), "Layer 1");
                    current_layer = doc.get_page(new_page).get_layer(new_layer);
                }
                // Gap doesn't apply at the top of a new column
            } else {
                cursor.y_mm += gap;
            }

            match item {
                LayoutItem::ArtistHeader { artist } => {
                    draw_artist_header(&current_layer, &fonts, &cursor, artist);
                }
                LayoutItem::SongRow {
                    title_lines,
                    stars,
                    is_unrated,
                    seq_num,
                    seq_col_offset,
                    is_alternate,
                    height_mm,
                } => {
                    draw_song_row(
                        &current_layer,
                        &fonts,
                        &cursor,
                        title_lines,
                        stars,
                        *is_unrated,
                        seq_num,
                        *seq_col_offset,
                        *is_alternate,
                        *height_mm,
                    );
                }
            }

            cursor.at_col_top = false;
            cursor.y_mm += item_h;
        }

        // Save to file
        let file = File::create(Path::new(path))
            .with_context(|| format!("Failed to create output file: {path}"))?;
        doc.save(&mut BufWriter::new(file))
            .context("Failed to save PDF")?;

        Ok(())
    }
}
