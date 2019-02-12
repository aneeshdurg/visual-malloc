
extern crate quicksilver;

use quicksilver::{
    Future, Result,
    combinators::result,
    geom::{Shape, Rectangle, Vector},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image},
    input::{MouseButton, ButtonState},
    lifecycle::{Asset, Event, Settings, State, Window, run}
};

struct SbrkDescriptor {
    end_of_heap_bytes: i32,
    sbrk: Asset<Image>,
    sbrk_rect: Rectangle,
    selected: bool,
    old_mouse_pos: Option<Vector>,
}

struct AllocationMenu {
    free_button: Rectangle,
    free_text: Asset<Image>,

    font_size: Vector,
    font_num_map: Asset<Image>,
    // TODO
    //coalesce_button: Rectangle,
    //coalesce_text: Asset<Image>,
}

impl AllocationMenu {
    fn new(font_size_x: f32, font_size_y: f32) -> Result<Self> {
        let font_num_map = Asset::new(Font::load("mononoki-Regular.ttf")
            .and_then(move |font| {
                let style = FontStyle::new(font_size_y, Color::BLACK);
                result(font.render("0123456789", &style))
            }));

        let free_asset = Asset::new(Font::load("mononoki-Regular.ttf")
            .and_then(move |font| {
                let style = FontStyle::new(36.0, Color::BLACK);
                result(font.render("free", &style))
            }));

        Ok(AllocationMenu {
            free_button: Rectangle::new((0, 0), (SBRK_MENU_PX, SBRK_MENU_PX)),
            free_text: free_asset,
            font_size: Vector::new(font_size_x, font_size_y),
            font_num_map: font_num_map,
        })
    }
    fn draw(&mut self, window: &mut Window, size: i32, y_offset: f32) -> Result<()> {
        let center_x = self.free_button.x()/2.0 + 5.0 + 50.0;
        let center_y = y_offset + self.free_button.y()/2.0;
        let free_rect = self.free_button.with_center((center_x, center_y));

        let mut y_off = y_offset;

        window.draw(&free_rect, Col(Color::CYAN));
        self.free_text.execute(|image| {
            window.draw(&image.area().with_center(
                    (center_x, center_y)), Img(&image));
            Ok(())
        })?;

        y_off += 2.0*self.free_button.y() + 5.0;
        let mut x = self.font_size.x;
        let mut y = self.font_size.y;
        self.font_num_map.execute(|image| {
            let subimg = &image.subimage(Rectangle::new((0, 0), (x, y)));
            window.draw(&subimg.area().with_center((6.0, y_off)), Img(&subimg));
            Ok(())
        })?;

        Ok(())
    }
}

struct MallocState {
    allocations: Vec<(Rectangle, bool)>,
    alloc_menu: AllocationMenu,
    sbrk_obj: SbrkDescriptor,
    display_menu: Option<usize>,
}

static TOTAL_MEMORY: i32 = 4*1024;
static PX_PER_BYTE: i32 = 1;
static SBRK_MENU_PX: i32 = 100;
static MEM_GAP: i32 = 5;

impl MallocState {
    fn handle_click(
            &mut self, _event: &Event, window: &mut Window) -> Result<()> {
       let mouse_pos = window.mouse().pos();
       if mouse_pos.overlaps_rectangle(&self.sbrk_obj.sbrk_rect) {
           self.sbrk_obj.selected = true;
           return Ok(());
       }

       for (i, alloc) in (&self.allocations).iter().enumerate() {
           let rect = alloc.0;
           if mouse_pos.overlaps_rectangle(&rect) {
               self.display_menu = Some(i);
               break;
           }
       }

       Ok(())
    }

    fn handle_release(
            &mut self, _event: &Event, _window: &mut Window) -> Result<()> {
        if !self.sbrk_obj.selected {
            return Ok(());
        }

        let curr_bytes =
            (self.sbrk_obj.sbrk_rect.x()/(PX_PER_BYTE as f32)) as i32;

        if curr_bytes > (self.sbrk_obj.end_of_heap_bytes + MEM_GAP) {
            let new_bytes = curr_bytes - self.sbrk_obj.end_of_heap_bytes;
            self.allocations.push((Rectangle::new(
                        (self.sbrk_obj.end_of_heap_bytes, 0),
                        (new_bytes - MEM_GAP, SBRK_MENU_PX)), false));
            self.sbrk_obj.end_of_heap_bytes = curr_bytes;
        } else {
            self.sbrk_obj.sbrk_rect = Rectangle::new(
                (self.sbrk_obj.end_of_heap_bytes, 0),
                (SBRK_MENU_PX, SBRK_MENU_PX));
        }

        self.sbrk_obj.old_mouse_pos = None;
        self.sbrk_obj.selected = false;
        Ok(())
    }

    fn handle_mouse_moved(
            &mut self, pos: &Vector, _window: &mut Window) -> Result<()> {
        if !self.sbrk_obj.selected {
            return Ok(());
        }

        match self.sbrk_obj.old_mouse_pos {
            Some(old_mouse_pos) => {
                let diff = (pos.x - old_mouse_pos.x)/(PX_PER_BYTE as f32);
                let old_x = self.sbrk_obj.sbrk_rect.x();
                self.sbrk_obj.sbrk_rect = Rectangle::new((old_x + diff, 0),
                    (SBRK_MENU_PX, SBRK_MENU_PX));

                self.sbrk_obj.old_mouse_pos = Some(*pos);
            }
            _ => {
                self.sbrk_obj.old_mouse_pos = Some(*pos);
            }
        }
        Ok(())
    }
}

impl State for MallocState {
    fn new() -> Result<Self> {
        let sbrk_asset = Asset::new(Font::load("mononoki-Regular.ttf")
            .and_then(|font| {
                let style = FontStyle::new(36.0, Color::BLACK);
                result(font.render("SBRK", &style))
            }));

        let sbrk_obj = SbrkDescriptor {
            end_of_heap_bytes: 0,
            sbrk: sbrk_asset,
            sbrk_rect: Rectangle::new((0,0), (SBRK_MENU_PX, SBRK_MENU_PX)),
            selected: false,
            old_mouse_pos: None,
        };

        let alloc_menu = AllocationMenu::new(12.0, 24.0)?;

        Ok(MallocState {
            allocations: vec![],
            alloc_menu: alloc_menu,
            sbrk_obj: sbrk_obj,
            display_menu: None,
        })
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {

        match event {
            Event::MouseButton(button, state) => {
                if button == &MouseButton::Left {
                    if state == &ButtonState::Pressed {
                        return self.handle_click(event, window);
                    } else if state == &ButtonState::Released {
                        return self.handle_release(event, window);
                    }
                }
            }
            Event::MouseMoved(pos) => {
                return self.handle_mouse_moved(pos, window);
            }
            _=> {}
        }
        Ok(())
    }
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;
        window.draw(&self.sbrk_obj.sbrk_rect, Col(Color::CYAN));
        let text_offset = (SBRK_MENU_PX as f32)/2.0;
        let text_x = self.sbrk_obj.sbrk_rect.x() + text_offset;
        let text_y = text_offset;
        self.sbrk_obj.sbrk.execute(|image| {
            window.draw(&image.area().with_center((text_x, text_y)), Img(&image));
            Ok(())
        })?;

        for alloc in (&self.allocations).iter() {
            let rect = alloc.0;
            if alloc.1 {
                window.draw(&rect, Col(Color::RED));
            } else {
                window.draw(&rect, Col(Color::BLUE));
            }
        }

        match self.display_menu {
            Some(i) => {
                // TODO take in y_offset while creating AllocationMenu so that we can detect clicks
                // on free/other buttons when added
                self.alloc_menu.draw(window,
                        (self.allocations[i].0.width()/(PX_PER_BYTE as f32) + MEM_GAP as f32) as i32, 200.0);
            }
            _ => {}
        }

        Ok(())
    }
}

fn main() {
    run::<MallocState>(
        "Malloc Visualization",
        Vector::new(TOTAL_MEMORY*PX_PER_BYTE + SBRK_MENU_PX, 800),
        Settings::default()
    );
}
