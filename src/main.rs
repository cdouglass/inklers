#![allow(deprecated)]
#![feature(collections)]

extern crate rustbox;

use std::default::Default;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::env;

use rustbox::{Color, RustBox};
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

fn col_to_string(color: Color) -> String {
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

fn str_to_col(s: &str) -> Color {
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

fn col_to_pixel(color: Color) -> &'static str { 
  match color {
    Color::Red     => "255   0   0 ",
    Color::Yellow  => "255 255   0 ",
    Color::Green   => "  0 255   0 ",
    Color::Blue    => "  0   0 255 ",
    Color::White   => "255 255 255 ",
    Color::Cyan    => "  0 255 255 ",
    Color::Magenta => "255   0 255 ",
    _              => "  0   0   0 ",
  }
}

// files

fn save_image(t: &Threading, n: &str) {
  let f = n.to_string() + ".ppm";
  let path = Path::new(&f);
  let mut file = File::create(&path).unwrap();

  let width = (t.odd.len() + t.even.len()) * 2 + 2;
  let w = width.to_string();
  let height = width * 4;
  let h = height.to_string();

  file.write((format!("P3\n{} {}\n255\n", w, h)).as_bytes()).unwrap(); // header

  for i in (0..height) {
    for z in (0..8) {
      let weft = col_to_pixel(t.weft);
      match i % 2 {
        0 => { for k in 0..(t.even.len()) { // TODO if one warp set is short, fill in with warp to constant width
            let p = col_to_pixel(t.even[k]);
            file.write(p.as_bytes()).unwrap();
            file.write(p.as_bytes()).unwrap();
            file.write(p.as_bytes()).unwrap();
            file.write(p.as_bytes()).unwrap();
          }
          file.write(weft.as_bytes()).unwrap();
          file.write(weft.as_bytes()).unwrap();
        },
        _ => { 
          file.write(weft.as_bytes()).unwrap();
          file.write(weft.as_bytes()).unwrap();
          for k in 0..(t.odd.len()) {
            let p = col_to_pixel(t.odd[k]);
            file.write(p.as_bytes()).unwrap();
            file.write(p.as_bytes()).unwrap();
            file.write(p.as_bytes()).unwrap();
            file.write(p.as_bytes()).unwrap();
          }
        }
      }
      file.write("\n".as_bytes()).unwrap();
    }
  }
}

fn save_threading(t: &Threading, n: &str) {
  let path = Path::new(n);
  let mut file = File::create(&path).unwrap();
  
  let mut out = vec![col_to_string(t.weft)];

  for shed in [&t.even, &t.odd].iter() {
    out.push("\n".to_string());
    for i in (0..shed.len()) {
      let c = col_to_string(shed[i]);
      out.push(c + ",");
    }
  }

  for i in out {
    file.write_all(i.as_bytes()).unwrap();
  }
}

fn read_threading(filename: &str) -> Option<Threading> {
  let path = Path::new(filename);
  let file = File::open(&path);
  let mut temp: Vec<String> = vec![];

  match file {
    Ok(f) => {
      let reader = BufReader::new(&f);
      for line in reader.lines() {
        match line {
          Ok(s) => { temp.push(s) },
          Err(_) => {},
        }
      } 
    },
    _ => {}
  }

  if temp.len() > 2 {
    let w = str_to_col(&temp[0]);
    let mut even: Vec<Color> = temp[1].split(",").map(str_to_col).collect();
    even.pop(); // last elt in the split doesn't match a color; remove that here
    let mut odd: Vec<Color> = temp[2].split(",").map(str_to_col).collect();
    odd.pop();
    let t = Threading { weft: w, even: even, odd: odd };
    return Some(t);
  }
  else { return None; }
}

// data manipulation

fn change_position(x: &mut i32, lim: usize, inc: i32) {
  let new_lim = lim as i32;
  let new = *x + inc;
  if new >= 0 { *x = new % new_lim; }
  else { *x = new_lim + new }
}

fn threading_to_row(t: &Threading, i: usize) -> Vec<Color> {
  let mut row: Vec<Color>;
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
  let num: Option<usize>;
    if { offset == 1 && *x < 1 } { num = None }
    else { num = Some((*x - offset) / 2) }
  (offset,num)
}

fn change_threading_color(threading: &mut Threading, color: Color, x: &i32, y: &i32) {
  let ux = *x as usize;
  let uy = *y as usize;
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

fn print_box(rb: &RustBox, x: usize, y: usize, color: Color) {
  rb.print_char(x, y, rustbox::RB_BOLD, color, color, ' ');
}

fn print_row(rb: &RustBox, y: usize, colors: &Vec<Color>) {
  for i in (0..colors.len()) {
    print_box(rb, i, y, colors[i]);
  }
}

fn print_cursor(rb: &RustBox, ix: i32, iy: i32) {
  let ux = ix as usize;
  let uy = iy as usize;
  rb.print_char(ux % rb.width(), uy % rb.height(), rustbox::RB_NORMAL, Color::Black, Color::White, 'X');
}

fn print_dash(rb: &RustBox, mode: Mode, msg: &str) {
  let x: usize = rb.width() - 55; 
  let mut y = 0;
  let mut blanks = " ".to_string();
  for i in (0..x+2) {
    blanks = blanks + " "
  }
  for i in (0..12) {
    rb.print(x-2, i, rustbox::RB_NORMAL, Color::White, Color::Black, &blanks);
  }
  let color_lst = [(Color::Red, "r"), (Color::Yellow, "y"), (Color::Green, "g"), (Color::Blue, "u"), (Color::White, "w"), (Color::Cyan, "c"), (Color::Magenta, "m")];
  let mut out: Vec<&str> = vec!["'h', 'j', 'k', and 'l' move the cursor."];
  match mode {
    Mode::Coloring => {
      out.push("'q' exits coloring mode.");
      for i in (0..out.len()) {
        rb.print(x, y, rustbox::RB_NORMAL, Color::White, Color::Black, out[i]);
        y = y + 1;
      }
      y = y + 1;
      for pair in color_lst.iter() {
        let (c, ch) = *pair;
        let text = format!("Press '{}' to set current color to {colorname}", ch, colorname = col_to_string(c));
        rb.print(x, y, rustbox::RB_NORMAL, c, Color::Black, &text);
        y = y + 1;
      }
      rb.print(x, y, rustbox::RB_NORMAL, Color::Black, Color::White, "Press 'b' to set current color to Black");
    },
    Mode::Save => {
      let f = format!("Press 's' to save. Filename will be '{}'.", msg);
      rb.print(x, y, rustbox::RB_NORMAL, Color::White, Color::Black, &f);
      y = y + 1;
      out.push("To select a new output name,");
      out.push("  press 'n', then a new name, then <Enter>.");
      out.push("Press 'q' to return to normal mode without saving.");
      out.push("");
      for i in (1..out.len()) {
        rb.print(x, y, rustbox::RB_NORMAL, Color::White, Color::Black, out[i]);
        y = y + 1;
      }
    },
    _ => {
      out.push("Press 'c' to enter coloring mode. Press 'q' to quit.");
      out.push("Press 's' to enter save mode.");
      out.push("Press 'q' to quit.");
      out.push("Other modes will be implemented soon.");
      out.push("Keep checking for updates!");
      for i in (0..out.len()) {
        rb.print(x, y, rustbox::RB_NORMAL, Color::White, Color::Black, out[i]);
        y = y + 1;
      }
    },
  }
}

fn draw(rb: &RustBox, t: &Threading, x: i32, y: i32, mode: Mode, msg: &str) {
  let row_0 = threading_to_row(t, 0);
  let row_1 = threading_to_row(t, 1);
  for i in (0..rb.height()) {
    if i % 2 == 0 { print_row(rb, i, &row_0) }
    else { print_row(rb, i, &row_1) }
  }
  print_cursor(rb, x, y);
  print_dash(rb, mode, msg);
}


fn color_key(rb: &RustBox, t: &mut Threading, x: &mut i32, y: &mut i32, c: Color) {
  change_threading_color(t, c, x, y);
  navigate(rb, x, y, 'l');
}

// interactive

fn pick_color(rb: &RustBox, t: &mut Threading, x: &mut i32, y: &mut i32) {
  loop {
    rb.clear();
    draw(rb, t, *x, *y, Mode::Coloring, "");
    rb.present(); 
    let event: rustbox::EventResult = (*rb).poll_event(false);
    match event { 
      Ok(rustbox::Event::KeyEvent(key)) => {
        match key {
          Some(Key::Char('q')) => { break; },
          Some(Key::Char('r')) => { color_key(rb, t, x, y, Color::Red) }
          Some(Key::Char('y')) => { color_key(rb, t, x, y, Color::Yellow) }
          Some(Key::Char('g')) => { color_key(rb, t, x, y, Color::Green) }
          Some(Key::Char('u')) => { color_key(rb, t, x, y, Color::Blue) }
          Some(Key::Char('c')) => { color_key(rb, t, x, y, Color::Cyan) }
          Some(Key::Char('w')) => { color_key(rb, t, x, y, Color::White) }
          Some(Key::Char('m')) => { color_key(rb, t, x, y, Color::Magenta) }
          Some(Key::Char('b')) => { color_key(rb, t, x, y, Color::Black) }
          Some(Key::Char(k)) => { navigate(rb, x, y, k) },
          _ => {},
        }
      },
      Err(e) => panic!("{:?}", e), 
      _ => { }
    }
  }
}

fn save(rb: &RustBox, t: &Threading, name: String) {
  let mut filename = name;
  loop {
    rb.clear();
    draw(rb, t, 0, 0, Mode::Save, &filename);
    rb.present();
    let event: rustbox::EventResult = (*rb).poll_event(false);
    match event {
      Ok(rustbox::Event::KeyEvent(key)) => {
        match key {
          Some(Key::Char('n')) => { filename = get_name(rb).clone(); },
          Some(Key::Char('s')) => {
            save_threading(t, &filename);
            save_image(t, &filename);
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

fn get_name(rb: &RustBox) -> String {
  let mut name = "".to_string();
  loop {
    let event: rustbox::EventResult = (*rb).poll_event(false);
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

fn navigate(rb: &RustBox, x: &mut i32, y: &mut i32, ch: char) {
  match ch {
    'h' => { change_position(x, rb.width(), -2) },
    'l' => { change_position(x, rb.width(), 2) },
    'k' => { change_position(y, rb.height(), -1) },
    'j' => { change_position(y, rb.height(), 1) },
    _ => {},
  }
}

fn main() {
  let rustbox = match RustBox::init(Default::default()) {
    Result::Ok(v) => v,
    Result::Err(e) => panic!("{}", e),
  };

  let mut cursor_x: i32;
  let mut cursor_y: i32;
  cursor_x = 0;
  cursor_y = 0;
  
  let input_file = env::args().nth(1).unwrap_or("output".to_string());

  let default_threading = Threading { even: vec![Color::Blue;rustbox.width() / 2 - 1], odd: vec![Color::White;rustbox.width() / 2 - 1], weft: Color::White };

  let mut my_threading = read_threading(&input_file).unwrap_or(default_threading);

  rustbox.present();
  loop {
    rustbox.clear();
    draw(&rustbox, &my_threading, cursor_x, cursor_y, Mode::Normal, "");
    rustbox.present();
    match rustbox.poll_event(false) {
      Ok(rustbox::Event::KeyEvent(key)) => {
        match key {
          Some(Key::Char('q')) => { break; }
          Some(Key::Char('c')) => { pick_color(&rustbox, &mut my_threading, &mut cursor_x, &mut cursor_y); },
          Some(Key::Char('s')) => { save(&rustbox, &my_threading, input_file.clone()) },
          Some(Key::Char(k)) => { navigate(&rustbox, &mut cursor_x, &mut cursor_y, k) },
          _ => { }
        }
      },
      Err(e) => panic!("{:?}", e),
      _ => { }
    }
  }
}
