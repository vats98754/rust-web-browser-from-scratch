// extern crate getopts;
// extern crate image;

use std::default::Default;
use std::io::{Read, BufWriter};
use std::fs::File;

pub mod css;
pub mod dom;
pub mod html;
pub mod layout;
pub mod style;
pub mod painting;
pub mod pdf;

fn main() {
    // Simplified version - using hardcoded defaults due to dependency issues
    let args: Vec<String> = std::env::args().collect();
    
    // Default file paths
    let html_file = if args.len() > 1 { &args[1] } else { "examples/test.html" };
    let css_file = if args.len() > 2 { &args[2] } else { "examples/test.css" };
    let output_file = if args.len() > 3 { &args[3] } else { "output.pdf" };
    
    println!("Parsing HTML: {}", html_file);
    println!("Parsing CSS: {}", css_file);
    println!("Output: {}", output_file);

    // Read input files:
    let html = read_source(html_file.to_string());
    let css  = read_source(css_file.to_string());

    // Since we don't have an actual window, hard-code the "viewport" size.
    let mut viewport: layout::Dimensions = Default::default();
    viewport.content.width  = 800.0;
    viewport.content.height = 600.0;

    // Parsing and rendering:
    let root_node = html::parse(html);
    let stylesheet = css::parse(css, css::Origin::Author);
    let stylesheets = [stylesheet];
    let style_root = style::style_tree(&root_node, &stylesheets);
    let layout_root = layout::layout_tree(&style_root, viewport);

    // Create the output file:
    let png = output_file.ends_with(".png");
    let filename = output_file.to_string();
    let mut file = BufWriter::new(File::create(&filename).unwrap());

    // Write to the file:
    let ok = if png {
        // Temporarily disabled PNG output due to image crate dependency issues
        println!("PNG output temporarily disabled - use PDF format instead");
        false
        // let canvas = painting::paint(&layout_root, viewport.content);
        // let (w, h) = (canvas.width as u32, canvas.height as u32);
        // let img = image::ImageBuffer::from_fn(w, h, move |x, y| {
        //     let color = canvas.pixels[(y * w + x) as usize];
        //     image::Rgba([color.r, color.g, color.b, color.a])
        // });
        // image::DynamicImage::ImageRgba8(img).write_to(&mut file, image::ImageFormat::Png).is_ok()
    } else {
        pdf::render(&layout_root, viewport.content, &mut file).is_ok()
    };
    if ok {
        println!("Saved output as {}", filename)
    } else {
        println!("Error saving output as {}", filename)
    }
}

fn read_source(filename: String) -> String {
    let mut str = String::new();
    File::open(filename).unwrap().read_to_string(&mut str).unwrap();
    str
}