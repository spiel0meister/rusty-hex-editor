#![allow(unused_imports)] //! Temporary
use std::{
    io::{Result, Error, ErrorKind, Write, stdout, Stdout},
    time::Duration,
    thread::sleep,
    fs::read,
    cmp::{min, max},
    slice::Chunks
};
use crossterm::{
    QueueableCommand,
    ExecutableCommand,
    terminal::{size, Clear, ClearType, enable_raw_mode, disable_raw_mode},
    cursor::{self, MoveTo, SetCursorStyle},
    style::{Print, PrintStyledContent, Stylize, Color, Attribute},
    event::{poll, Event, KeyCode, read as read_event, KeyModifiers}
};

struct CursorPosition(u16, u16);

impl CursorPosition {
    fn to_moveto(&self) -> MoveTo {
        MoveTo(self.0, self.1)
    }
}

fn draw_bytes(out: &mut Stdout, chunks: Chunks<'_, u8>, size_of_chunks: usize, literal_part_offset: usize, cursor_pos: &CursorPosition) -> Result<()> {
    let highlighted_byte_row = cursor_pos.1;
    let highlighted_byte_col = (cursor_pos.0 as f32 / 3f32).floor() as usize;
    for (row, byte_chunk) in chunks.enumerate() {
        for i in 0..size_of_chunks {
            out.queue(MoveTo(i as u16 * 3, row as u16))?;
            if size_of_chunks - byte_chunk.len() > 0 && i >= byte_chunk.len() {
                out.queue(Print("  "))?;
            } else {
                let byte = &byte_chunk[i];
                if cursor_pos.0 % 3 != 2 && highlighted_byte_row == row as u16 && highlighted_byte_col == i {
                    out.queue(PrintStyledContent(format!("{:02x}", byte).with(Color::Black).on(Color::Blue)))?;
                } else {
                    out.queue(Print(format!("{:02x}", byte)))?;
                }
            }
        }

        out.queue(MoveTo(literal_part_offset as u16 - 3u16, row as u16))?;
        out.queue(Print(" | "))?;

        for i in 0..size_of_chunks {
            out.queue(MoveTo(literal_part_offset as u16 + i as u16, row as u16))?;
            if size_of_chunks - byte_chunk.len() > 0 && i >= byte_chunk.len() {
                out.queue(Print(" "))?;
            } else {
                let byte = &byte_chunk[i];
                let byte_as_char = char::from_u32(*byte as u32).ok_or(Error::new(ErrorKind::InvalidInput, "Unsupported UTF32 code"))?;
                if cursor_pos.0 % 3 != 2 && highlighted_byte_row == row as u16 && highlighted_byte_col == i {
                    out.queue(PrintStyledContent(byte_as_char.with(Color::Black).on(Color::Blue)))?;
                } else {
                    out.queue(Print(byte_as_char))?;
                }
            }
        }
    }

    Ok(())
}

static FPS: u8 = 15;

fn main() -> Result<()> {
    let mut out = stdout();
    let delay = (1000 as f32 / FPS as f32).floor() as u64;
    let bytes = read("dummy.txt")?;
    let mut cursor_position = CursorPosition(0, 0);

    enable_raw_mode()?;
    out.execute(Clear(ClearType::All))?;

    loop {
        out.queue(Clear(ClearType::All))?;
        out.queue(SetCursorStyle::SteadyBlock)?;
        let (width, height) = size()?;
        let size_of_chunks = ((width - 2) / 4) as usize;
        let literal_part_offset = 3 * size_of_chunks + 2;

        draw_bytes(&mut out, bytes.chunks(size_of_chunks), size_of_chunks, literal_part_offset, &cursor_position)?;

        if poll(Duration::ZERO)? {
            match read_event()? {
                Event::Key(event) => {
                    match event.code {
                        KeyCode::Char('c') => {
                            if event.modifiers.contains(KeyModifiers::CONTROL) {
                                break;
                            }
                        },
                        KeyCode::Char('j') => cursor_position.1 = min(cursor_position.1 + 1, height - 1),
                        KeyCode::Char('k') => cursor_position.1 = cursor_position.1.checked_sub(1).unwrap_or(0),
                        KeyCode::Char('h') => cursor_position.0 = cursor_position.0.checked_sub(1).unwrap_or(0),
                        KeyCode::Char('l') => cursor_position.0 = min(cursor_position.0 + 1, width - 1),
                        _ => {}
                    }
                },
                _ => {}
            }
        }


        out.queue(cursor_position.to_moveto())?;

        out.flush()?;
        sleep(Duration::from_millis(delay));
    }
    out.execute(Clear(ClearType::All))?;
    out.execute(MoveTo(0, 0))?;

    disable_raw_mode()?;
    Ok(())
}
