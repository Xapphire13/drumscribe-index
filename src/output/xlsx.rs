use anyhow::Result;
use rust_xlsxwriter::{Color, Format, Workbook};

use crate::{
    group_songs,
    models::song::{Difficulty, Song},
};

pub struct XlsxFormatter;

impl XlsxFormatter {
    pub fn format_to_file(&self, songs: &[Song], path: &str) -> Result<()> {
        let groups = group_songs(songs);

        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Create formats
        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x000000))
            .set_font_color(Color::White);

        let artist_header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x000000))
            .set_font_color(Color::White)
            .set_font_size(14);

        let alternate_row_format = Format::new().set_background_color(Color::RGB(0xF5F5F5));

        // Set column widths
        worksheet.set_column_width(0, 40)?; // Title
        worksheet.set_column_width(1, 20)?; // Difficulty
        worksheet.set_column_width(2, 12)?; // Sequence Number

        // Write header row
        worksheet.write_with_format(0, 0, "Title", &header_format)?;
        worksheet.write_with_format(0, 1, "Difficulty", &header_format)?;
        worksheet.write_with_format(0, 2, "Sequence #", &header_format)?;

        let mut current_row = 1u32;

        for group in &groups {
            // Write artist header
            worksheet.merge_range(
                current_row,
                0,
                current_row,
                2,
                &group.artist,
                &artist_header_format,
            )?;
            current_row += 1;

            // Write songs for this artist
            for (idx, song) in group.songs.iter().enumerate() {
                let difficulty_str = match song.difficulty {
                    Difficulty::Beginner => "★",
                    Difficulty::Intermediate => "★★",
                    Difficulty::Advanced => "★★★",
                    Difficulty::Expert => "★★★★",
                    Difficulty::Master => "★★★★★",
                    Difficulty::Unrated => "—",
                };

                // Alternate row backgrounds
                if idx % 2 == 1 {
                    worksheet.write_with_format(
                        current_row,
                        0,
                        &song.title,
                        &alternate_row_format,
                    )?;
                    worksheet.write_with_format(
                        current_row,
                        1,
                        difficulty_str,
                        &alternate_row_format,
                    )?;
                    worksheet.write_with_format(
                        current_row,
                        2,
                        &song.sequence_number,
                        &alternate_row_format,
                    )?;
                } else {
                    worksheet.write(current_row, 0, &song.title)?;
                    worksheet.write(current_row, 1, difficulty_str)?;
                    worksheet.write(current_row, 2, &song.sequence_number)?;
                }

                current_row += 1;
            }

            // Add a blank row between artist groups
            current_row += 1;
        }

        workbook.save(path)?;

        Ok(())
    }
}
