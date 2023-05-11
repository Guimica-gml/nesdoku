use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::rwops::RWops;
use sdl2::ttf;
use sdl2::ttf::Font;
use sdl2::video::{Window, WindowContext};
use std::{env, process};

mod sudoku;
use sudoku::*;

const COLOR_STATIC: Color = Color::RGB(29, 32, 33);
const COLOR_CERTAIN: Color = Color::RGB(0, 131, 176);
const COLOR_UNCERTAIN: Color = Color::RGB(81, 132, 113);
const COLOR_BACKGROUD: Color = Color::WHITE;

const WINDOW_DIM: u32 = 900;
const FONT_TFF_BYTES: &[u8] = include_bytes!("../fnt/Iosevka.ttf");

macro_rules! point {
    ($x: expr, $y: expr) => {
        ($x as i32, $y as i32)
    };
}

fn draw_text(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font: &Font,
    text: &str,
    pos: (i32, i32),
    color: Color,
) -> Result<(), String> {
    let surface = font
        .render(text)
        .blended(color)
        .map_err(|e| e.to_string())?;

    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    let (w, h) = surface.rect().size();
    let target = Rect::new(pos.0 - w as i32 / 2, pos.1 - h as i32 / 2, w, h);

    canvas.copy(&texture, None, Some(target))?;
    Ok(())
}

pub fn draw_line_thicc(
    canvas: &mut Canvas<Window>,
    start: (i32, i32),
    end: (i32, i32),
    thicc: i32,
) -> Result<(), String> {
    for i in (-thicc / 2).min(0)..(thicc / 2).max(1) {
        if start.0 != end.0 {
            canvas.draw_line((start.0, start.1 + i), (end.0, end.1 + i))?;
        } else if start.1 != end.1 {
            canvas.draw_line((start.0 + i, start.1), (end.0 + i, end.1))?;
        } else {
            canvas.draw_line((start.0 + i, start.1 + i), (end.0 + i, end.1 + i))?;
        }
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let sudoku_file = match args.next() {
        Some(v) => v,
        None => {
            eprintln!("Error: Expected sudoku file");
            process::exit(1);
        }
    };

    let sdl_context = sdl2::init()?;
    let ttf_context = ttf::init().map_err(|e| e.to_string())?;

    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Sudoku", WINDOW_DIM, WINDOW_DIM)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    let texture_creator = canvas.texture_creator();
    let field_dim = WINDOW_DIM / Sudoku::BOARD_DIM as u32;

    let initial_board = match Sudoku::from_file(&sudoku_file) {
        Ok(v) => v,
        Err(message) => {
            eprintln!("Error: Could not read file `{}`: {}", sudoku_file, message);
            process::exit(1);
        }
    };

    let mut boards = vec![initial_board];

    let font_size = (field_dim as f32 * 0.4) as u16;
    let font = ttf_context.load_font_from_rwops(RWops::from_bytes(FONT_TFF_BYTES)?, font_size)?;

    let small_font_size = (field_dim as f32 * 0.25) as u16;
    let small_font = ttf_context.load_font_from_rwops(RWops::from_bytes(FONT_TFF_BYTES)?, small_font_size)?;

    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {keycode: Some(Keycode::R), .. } => {
                    boards[0].reset_board();
                    boards.drain(1..);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } if !boards[0].complete() => {
                    boards[0].update_possible_values();
                    let (x, y) = boards[0].find_less_entropy();

                    match boards[0].collapse_cell(x, y) {
                        Ok(other_possibilities) => {
                            for board in other_possibilities {
                                boards.insert(1, board);
                            }
                        }
                        Err(_) => _ = boards.remove(0),
                    }

                    assert!(boards.len() > 0, "It's a bug... No! Feature");
                }
                Event::Quit { .. } => break 'gameloop,
                _ => {}
            }
        }

        canvas.clear();
        canvas.set_draw_color(COLOR_STATIC);

        for y in 0..Sudoku::BOARD_DIM {
            for x in 0..Sudoku::BOARD_DIM {
                let nums_len = boards[0].get_cell(x, y).value().as_vec().len();
                if nums_len == 0 {
                    continue;
                }

                let (cols_amount, rows_amount) = if nums_len >= 7 {
                    (3, 3)
                } else if nums_len >= 5 {
                    (3, 2)
                } else if nums_len >= 3 {
                    (2, 2)
                } else if nums_len == 2 {
                    (2, 1)
                } else {
                    (1, 1)
                };

                let font = if nums_len == 1 { &font } else { &small_font };

                let xspace = field_dim / cols_amount;
                let yspace = field_dim / rows_amount;

                let mut xcurr: u32 = 0;
                let mut ycurr: u32 = 0;

                for num in boards[0].get_cell(x, y).value().as_vec() {
                    let posx = (x as u32 * field_dim + xspace / 2 + xspace * xcurr) as i32;
                    let posy = (y as u32 * field_dim + yspace / 2 + yspace * ycurr) as i32;

                    let color = if boards[0].get_cell(x, y).is_static() {
                        COLOR_STATIC
                    } else if boards[0].get_cell(x, y).value().is_certain() {
                        COLOR_CERTAIN
                    } else {
                        COLOR_UNCERTAIN
                    };

                    draw_text(&mut canvas, &texture_creator, font, &num.to_string(), (posx, posy), color)?;

                    xcurr += 1;
                    if xcurr >= cols_amount {
                        ycurr += 1;
                        xcurr = 0;
                    }
                }
            }
        }

        for i in 1..Sudoku::BOARD_DIM as u32 {
            let pos = i * field_dim;
            let thicc = if i % 3 == 0 { 5 } else { 1 };
            draw_line_thicc(&mut canvas, point!(pos, 0), point!(pos, WINDOW_DIM), thicc)?;
            draw_line_thicc(&mut canvas, point!(0, pos), point!(WINDOW_DIM, pos), thicc)?;
        }

        canvas.set_draw_color(COLOR_BACKGROUD);
        canvas.present();
    }

    Ok(())
}
