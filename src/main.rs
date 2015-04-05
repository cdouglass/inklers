#![feature(old_io)]
#![feature(core)]
#![allow(deprecated)]

extern crate rustbox;

use std::old_io::stdio;
use std::default::Default;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::env;
use std::num::ToPrimitive;

use rustbox::{Color, RustBox, InitOptions};
use rustbox::Key;

struct Threading {
  even: Vec<Color>, // lengths should always be equal
  odd: Vec<Color>,
  weft: Color
}

enum Mode {
  Coloring,
  Float, // TODO warp float patterning mode
  Double, // TODO double weave mode
  Normal,
  Save,
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

fn str_to_col(s:&str) -> Color {
  match s {
    "Red" => Color::Red,
    "Yellow" => Color::Yellow,
    "Green" => Color::Green,
    "Blue" => Color::Blue,
    "Black" => Color::Black,
    "White" => Color::White,
    "Cyan" => Color::Cyan,
    "Magenta" => Color::Magenta,
    _ => Color::Black, // so I don't have to make this give an Option
  }
}

// files

fn save_threading(t:&Threading, n:&str) {
  let path = Path::new(n);
  let display = path.display();
  let mut file = match File::create(&path) {
    Err(reason) => panic!("Couldn't create {}: {:?}", display, reason),
    Ok(file) => file,
  };
  
  let mut out = vec![col_to_string(t.weft)];

  for shed in [&t.even, &t.odd].iter() {
    out.push("\n".to_string());
    for i in (0..shed.len()) {
      let c = col_to_string(shed[i]);
      out.push(c + ",");
    }
  }

  for i in out {
    match file.write_all(i.as_bytes()) {
      Err(reason) => { panic!("Couldn't write to {}: {:?}", display, reason) },
      Ok(_) => {},
    }
  }
}

fn read_threading(filename:&str) -> Option<Threading> {
  let path = Path::new(filename);
  let file = match File::open(&path) {
    Err(_) => { return None; }
    Ok(file) => file,
  };
  
  let reader = BufReader::new(&file);

  let mut temp:Vec<String> = vec![];

  for line in reader.lines() {
    match line {
      Ok(s) => { temp.push(s) },
      Err(_) => {},
    }
  }

  if temp.len() > 2 {
    let w = str_to_col(temp[0].as_slice());
    let mut even:Vec<Color> = temp[1].split(",").map(str_to_col).collect();
    even.pop(); // last elt in the split doesn't match a color; remove that here
    let mut odd:Vec<Color> = temp[2].split(",").map(str_to_col).collect();
    odd.pop();
    let t = Threading { weft: w, even: even, odd: odd };
    return Some(t);
  }
  else { return None; }
}

// data manipulation

fn change_position(x:&mut i32, lim: usize, inc:i32) {
  let new_lim = my_to_i32(lim);
  let new = *x + inc;
  if new >= 0 { *x = new % new_lim; }
  else { *x = new_lim + new }
}

fn threading_to_row(t:&Threading, i: usize) -> Vec<Color> {
  let mut row:Vec<Color>;
  row = vec![t.weft];
  match i % 2 {
    0 => { for j in (0..t.even.len()) {
             row.push(t.even[j]);
             row.push(t.even[j]);
           }
           row.push(t.weft);
           row.remove(0);
         },
    1 => { for j in (0..t.odd.len()) {
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

fn my_to_usize(i:i32) -> usize {
  match i.to_usize() {
                       Some(n) => n,
                       _ => 0,
                     }
}

fn my_to_i32(u:usize) -> i32 {
  match u.to_i32() {
                   Some(n) => n,
                   _ => 0,
                 }
}

fn change_threading_color(threading: &mut Threading, color:Color, x:&i32, y:&i32) {
  let ux = my_to_usize(*x);
  let uy = my_to_usize(*y);
  let warp: usize;
  match pos_to_warp(&ux, &uy) {
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
  for i in (0..colors.len()) {
    print_box(rb, i, y, colors[i]);
  }
}

fn print_cursor(rb:&RustBox, ix: i32, iy: i32) {
  let ux = my_to_usize(ix);
  let uy = my_to_usize(iy);
  rb.print_char(ux % rb.width(), uy % rb.height(), rustbox::RB_NORMAL, Color::Black, Color::White, 'X');
}

fn print_dash(rb:&RustBox, color:Color, mode:Mode, msg:&str) {
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
                        for i in (0..out.len()) {
                          rb.print(x, y, rustbox::RB_NORMAL, Color::White, Color::Black, out[i]);
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
    Mode::Save => {  let f = format!("Press 's' to save. Filename will be '{}'.", msg);
                     rb.print(x, y, rustbox::RB_NORMAL, Color::White, Color::Black, f.as_slice());
                     y = y + 1;
                     out.push("Press 'n', then a new name, then <Enter> to select a new output name.");
                     out.push("Press 'q' to return to normal mode without saving.");
                     for i in (1..out.len()) {
                       rb.print(x, y, rustbox::RB_NORMAL, Color::White, Color::Black, out[i]);
                       y = y + 1;
                     }
                  },
    _ => { out.push("Press 'c' to enter coloring mode. Press 'q' to quit.");
           out.push("Press 's' to enter save mode.");
           out.push("Press 'q' to quit.");
           out.push("Other modes will be implemented soon. Keep checking for updates!");
           for i in (0..out.len()) {
             rb.print(x, y, rustbox::RB_NORMAL, Color::White, Color::Black, out[i]);
             y = y + 1;
           }
    },
  }
}

fn draw(rb:&RustBox, t:&Threading, c:Color, x:i32, y:i32, mode:Mode, msg:&str) {
  let row_0 = threading_to_row(t, 0);
  let row_1 = threading_to_row(t, 1);
  for i in (0..rb.height()) {
    if i % 2 == 0 { print_row(rb, i, &row_0) }
    else { print_row(rb, i, &row_1) }
  }
  print_cursor(rb, x, y);
  print_dash(rb, c, mode, msg);
}

// interactive

fn pick_color(rb:&RustBox, t:&mut Threading, x:&mut i32, y:&mut i32) -> Option<Color> {
  let mut c: Option<Color> = Some(Color::Red); 
  loop {
    rb.clear();
    let color = match c {
                  Some(col) => col,
                  _ => Color::Red,
    };
    draw(rb, t, color, *x, *y, Mode::Coloring, "");
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
      Err(e) => panic!("{:?}", e), 
      _ => { }
    }
  }
  c
}

fn save(rb:&RustBox, t:&Threading) {
  let mut filename = "output".to_string();
  loop {
    rb.clear();
    draw(rb, t, Color::White, 0, 0, Mode::Save, filename.as_slice());
    rb.present();
    let event:rustbox::EventResult<rustbox::Event> = (*rb).poll_event(false);
    match event {
      Ok(rustbox::Event::KeyEvent(key)) => {
        match key {
          Some(Key::Char('n')) => { filename = get_name(rb).clone(); },
          Some(Key::Char('s')) => { save_threading(t, filename.as_slice());
                                    break;
                                  },
          Some(Key::Char('q')) => { break; },
          _ => {},
        }
      }
      Err(e) => panic!("{:?}", e),
      _ => {}
    }
  }
}

fn get_name(rb:&RustBox) -> String {
  let mut name = "".to_string();
  loop {
    let event:rustbox::EventResult<rustbox::Event> = (*rb).poll_event(false);
    match event {
      Ok(rustbox::Event::KeyEvent(key)) => {
        match key {
          Some(Key::Char(k)) => { name.push(k); },
          Some(_) => { break; }, // any key that's not a character, including but not limited to <Enter>
          _ => {},
          }
        }
      Err(e) => panic!("{:?}", e),
      _ => {},
    }
  }
  name
}

fn navigate(rb:&RustBox, x:&mut i32, y:&mut i32, ch:char) {
  match ch {
    'h' => { change_position(x, rb.width(), -1) },
    'l' => { change_position(x, rb.width(), 1) },
    'k' => { change_position(y, rb.height(), -1) },
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

  let mut cursor_x: i32;
  let mut cursor_y: i32;
  cursor_x = 0;
  cursor_y = 0;
  
  let input_file = match env::args().nth(1) {
    Some(n) => n.clone(), // if i take a slice here then it goes out of scope when i need it later
    _ => "input".to_string(),
  };

  let mut my_threading = match read_threading(input_file.as_slice()) {
    Some(t) => t,
    None => Threading { even: vec![Color::Blue;rustbox.width() / 3 - 1], odd: vec![Color::White;rustbox.width() / 3 - 1], weft: Color::White },
    };

  rustbox.present();
  loop {
    rustbox.clear();
    draw(&rustbox, &my_threading, Color::White, cursor_x, cursor_y, Mode::Normal, "");
    rustbox.present();
    match rustbox.poll_event(false) {
      Ok(rustbox::Event::KeyEvent(key)) => {
        match key {
          Some(Key::Char('q')) => { break; }
          Some(Key::Char('c')) => { pick_color(&rustbox, &mut my_threading, &mut cursor_x, &mut cursor_y); },
          Some(Key::Char('s')) => { save(&rustbox, &my_threading) },
          Some(Key::Char(k)) => { navigate(&rustbox, &mut cursor_x, &mut cursor_y, k) },
          _ => { }
        }
      },
      Err(e) => panic!("{:?}", e),
      _ => { }
    }
  }
}
