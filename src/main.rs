#![feature(old_io)]
#![feature(core)]

extern crate rustbox;

use std::old_io::stdio;
use std::error::Error;
use std::default::Default;
use std::iter;

use rustbox::{Color, RustBox, InitOptions};
use rustbox::Key;

struct Threading {
  even: Vec<Color>, // lengths should always be equal
  odd: Vec<Color>,
  weft: Color
}

enum Mode {
  Coloring,
  Float,
  Double,
  Normal,
}

fn col_to_string(color:Color) -> String {
  match color {
    Color::Red => "Red".to_string(),
    Color::Yellow => "Yellow".to_string(),
    Color::Green => "Green".to_string(),
    Color::Blue => "Blue".to_string(),
    Color::Black => "Black".to_string(),
    Color::White => "White".to_string(),
    Color::Cyan => "Cyan".to_string(),
    Color::Magenta => "Magenta".to_string(),
    _ => unreachable!()
  }
}

// data manipulation

fn change_position(x:&mut usize, lim: usize, inc:usize) {
  match inc {
    1 => { if *x < lim - 1 {
             *x = *x + 1;
           } else {
             *x = 0;
           }
         },
   _ => { if *x > 0 {
            *x = *x - 1;
          } else {
            *x = lim - 1;
          }
        },
  }
}

fn threading_to_row(t:&Threading, i: usize) -> Vec<Color> {
  let mut row:Vec<Color>;
  row = vec![t.weft];
  match i % 2 {
    0 => { for j in iter::range(0,t.even.len()) {
             row.push(t.even[j]);
             row.push(t.even[j]);
           }
           row.push(t.weft);
           row.remove(0);
         },
    1 => { for j in iter::range(0,t.odd.len()) {
             row.push(t.odd[j]);
             row.push(t.odd[j]);
           }
         },
    _ => unreachable!()
  };
  row
}

fn pos_to_warp(x: &usize, y: &usize) -> (usize, Option<usize>) {
  let offset = *y % 2;
  let num:Option<usize>;
    if { offset == 1 && *x < 1 } { num = None }
    else { num = Some((*x - offset) / 2) }
  (offset,num)
}

fn change_threading_color(threading: &mut Threading, color:Color, x:&usize, y:&usize) {
  let warp: usize;
  match pos_to_warp(x, y) {
    (i, Some(j)) => { warp = j;
                      match i {
                        0 => { if warp >= threading.even.len() { threading.weft = color }
                               else { threading.even[warp] = color }
                             },
                        1 => { if warp >= threading.odd.len() { threading.weft = color }
                               else { threading.odd[warp] = color }
                             },
                        _ => unreachable!(),
                      }
                    },
    _ => { threading.weft = color },
  };
}

// display

fn print_box(rb:&RustBox, x: usize, y: usize, color:Color) {
  rb.print_char(x, y, rustbox::RB_BOLD, color, color, ' ');
}

fn print_row(rb:&RustBox, y: usize, colors:&Vec<Color>) {
  for i in iter::range(0, colors.len()) {
    print_box(rb, i, y, colors[i]);
  }
}

fn print_cursor(rb:&RustBox, x: usize, y: usize) {
  rb.print_char(x, y, rustbox::RB_NORMAL, Color::Black, Color::White, 'X');
}

fn print_dash(rb:&RustBox, color:Color, mode:Mode) {
  let bg = match color {
    Color::Black => Color::White,
    _ => Color::Black,
  };
  let x: usize = rb.width() - 70; 
  let mut y = 0;
  let color_lst = [(Color::Red, "r"), (Color::Yellow, "y"), (Color::Green, "g"), (Color::Blue, "u"), (Color::White, "w"), (Color::Cyan, "c"), (Color::Magenta, "m")];
  let mut out:Vec<&str> = vec!["Press 'h', 'j', 'k', or 'l' to move the cursor."];
  match mode {
    Mode::Coloring => { out.push("Press 'a' to apply the current color to the selected thread.");
                        out.push("Press 'q' to exit coloring mode.");
                        for i in iter::range(0,out.len()) {
                          rb.print(x, i, rustbox::RB_NORMAL, Color::White, Color::Black, out[i]);
                          y = y + 1;
                        }
                        let c = format!("The current color is {}", col_to_string(color));
                        rb.print(x, out.len(), rustbox::RB_NORMAL, color, bg, c.as_slice());
                        y = y + 1;
                          for pair in color_lst.iter() {
                            let (c, ch) = *pair;
                            let text = format!("Press '{}' to set current color to {colorname}",ch, colorname = col_to_string(c));
                            rb.print(x, y, rustbox::RB_NORMAL, c, Color::Black, text.as_slice() );
                            y = y + 1;
                          }
                          rb.print(x, y, rustbox::RB_NORMAL, Color::Black, Color::White, "Press 'b' to set current color to Black");
                      },
    _ => { out.push("Press 'c' to enter coloring mode. Press 'q' to quit.");
           out.push("Other modes will be implemented soon. Keep checking for updates!");
           for i in iter::range(0,out.len()) {
             rb.print(x, i, rustbox::RB_NORMAL, Color::White, Color::Black, out[i])
           }
    },
  }
}

fn draw(rb:&RustBox, t:&Threading, c:Color, x:usize, y:usize, mode:Mode) {
  let row_0 = threading_to_row(t, 0);
  let row_1 = threading_to_row(t, 1);
  for i in iter::range(0, rb.height()) {
    if i % 2 == 0 { print_row(rb, i, &row_0) }
    else { print_row(rb, i, &row_1) }
  }
  print_cursor(rb, x, y);
  print_dash(rb, c, mode);
}

// interactive

fn pick_color(rb:&RustBox, t:&mut Threading, x:&mut usize, y:&mut usize) -> Option<Color> {
  let mut c: Option<Color> = Some(Color::Red); 
  loop {
    rb.clear();
    let color = match c {
                  Some(col) => col,
                  _ => Color::Red,
    };
    draw(rb, t, color, *x, *y, Mode::Coloring);
    rb.present(); 
    let event:rustbox::EventResult<rustbox::Event> = (*rb).poll_event(false);
    match event { 
      Ok(rustbox::Event::KeyEvent(key)) => {
        match key {
        Some(Key::Char('r')) => { c = Some(Color::Red) },
        Some(Key::Char('y')) => { c = Some(Color::Yellow) },
        Some(Key::Char('g')) => { c = Some(Color::Green) },
        Some(Key::Char('u')) => { c = Some(Color::Blue) },
        Some(Key::Char('c')) => { c = Some(Color::Cyan) },
        Some(Key::Char('w')) => { c = Some(Color::White) },
        Some(Key::Char('m')) => { c = Some(Color::Magenta) },
        Some(Key::Char('b')) => { c = Some(Color::Black) },
        Some(Key::Char('q')) => { break; },
        Some(Key::Char('a')) => { match c {
                                    Some(color) => { change_threading_color(t, color, x, y) }
                                    _ => { },
                                   }
                                },
        Some(Key::Char(k)) => { navigate(rb, x, y, k) },
        _ => { },
        }
      },
      Err(e) => panic!("{}", e.description()), 
      _ => { }
    }
  }
  c
}

fn navigate(rb:&RustBox, x:&mut usize, y:&mut usize, ch:char) {
  match ch {
    'h' => { change_position(x, rb.width(), 0) },
    'l' => { change_position(x, rb.width(), 1) },
    'k' => { change_position(y, rb.height(), 0) },
    'j' => { change_position(y, rb.height(), 1) },
    _ => { },
  }
}

fn main() {
  let rustbox = match RustBox::init(InitOptions {
    buffer_stderr: stdio::stderr_raw().isatty(),
    ..Default::default()
  }) {
    Result::Ok(v) => v,
    Result::Err(e) => panic!("{}", e),
  };

  let mut cursor_x: usize;
  let mut cursor_y: usize;
  cursor_x = 0;
  cursor_y = 0;

  let mut my_threading = Threading { even: vec![Color::Red], odd: vec![Color::Cyan], weft: Color::White }; // can't use shortcut vec![Color::Red; 20] because clone is not implemented for color
  for i in iter::range(0,rustbox.width() / 3 - 1) {
    my_threading.even.push(Color::Red);
    my_threading.odd.push(Color::Cyan);
  }

  let mut current_color = Color::Red;
  
  rustbox.present();
  loop {
    rustbox.clear();
    draw(&rustbox, &my_threading, Color::White, cursor_x, cursor_y, Mode::Normal);
    rustbox.present();
    match rustbox.poll_event(false) {
      Ok(rustbox::Event::KeyEvent(key)) => {
        match key {
          Some(Key::Char('q')) => { break; }
          Some(Key::Char('c')) => { match pick_color(&rustbox, &mut my_threading, &mut cursor_x, &mut cursor_y) {
                                      Some(color) => { current_color = color },
                                      _ => { },
                                      }
                                  },
          Some(Key::Char('a')) => { change_threading_color(&mut my_threading, current_color, &cursor_x, &cursor_y) },
          Some(Key::Char(k)) => { navigate(&rustbox, &mut cursor_x, &mut cursor_y, k) },
          _ => { }
        }
      },
      Err(e) => panic!("{}", e.description()),
      _ => { }
    }
  }
}

// TODO (easy, do tonight)
// better determination of where to end warp and where to start dash
// github link in Cargo.toml - test it actually works
