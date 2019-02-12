
extern crate quicksilver;

use quicksilver::{
    Future, Result,
    combinators::result,
    geom::{Shape, Rectangle, Vector},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image},
    input::{MouseCursor, MouseButton, ButtonState},
    lifecycle::{Asset, Event, Settings, State, Window, run}
};

struct MallocState {
    coalesce: bool,
    allocations: Vec<(Rectangle, bool)>,
    sbrk: Asset<Image>,
    end_of_heap_px: u32,
}

static total_memory: i32 = 1024;
static px_per_byte: i32 = 1;
static sbrk_menu_px: i32 = 100;

impl State for MallocState {
    fn new() -> Result<Self> {
        let sbrk_asset = Asset::new(Font::load("mononoki-Regular.ttf")
            .and_then(|font| {
                let style = FontStyle::new(36.0, Color::BLACK);
                result(font.render("SBRK", &style))
            }));
        Ok(MallocState {
            coalesce: false,
            allocations: vec![],
            sbrk: sbrk_asset,
            end_of_heap_px: sbrk_menu_px as u32,
        })
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {

        match event {
            Event::MouseButton(button, state) => {

                if button == &MouseButton::Left &&
                        state == &ButtonState::Pressed {
                    let mut input_processed = false;
                    for (i, alloc) in (&self.allocations).iter().enumerate() {
                        let rect = alloc.0;
                        if window.mouse().pos().overlaps_rectangle(&rect) {
                            self.allocations[i] = (alloc.0, !alloc.1);
                            input_processed = true;
                            break;
                        }
                    }
                    if !input_processed {
                        &self.allocations.push(
                            (Rectangle::new((self.end_of_heap_px,0), (50, 100)), true));
                        self.end_of_heap_px += 60;
                    }
                }
            }
            _=> {}
        }
        Ok(())
    }
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;
        let sbrk_rect = Rectangle::new((0,0), (100, 100));
        window.draw(&sbrk_rect, Col(Color::CYAN));
        self.sbrk.execute(|image| {
            window.draw(&image.area().with_center((50, 50)), Img(&image));
            Ok(())
        });

        for (i, alloc) in (&self.allocations).iter().enumerate() {
            let rect = alloc.0;
            if alloc.1 {
                window.draw(&rect, Col(Color::RED));
            } else {
                window.draw(&rect, Col(Color::BLUE));
            }
        }

        Ok(())
    }
}

fn main() {
    run::<MallocState>(
        "Malloc Visualization",
        Vector::new(total_memory*px_per_byte + sbrk_menu_px, 800),
        Settings::default()
    );
}
