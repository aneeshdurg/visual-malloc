
extern crate quicksilver;

use quicksilver::{
    Future, Result,
    combinators::result,
    geom::{Shape, Rectangle, Vector, Transform},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image},
    input::{MouseButton, ButtonState, Key},
    lifecycle::{Asset, Event, Settings, State, Window, run}
};

struct Block {
    rect: Rectangle,
    allocated: bool,
    space_used: i32,
}

struct SbrkDescriptor {
    end_of_heap_bytes: i32,
    sbrk: Asset<Image>,
    sbrk_rect: Rectangle,
    selected: bool,
    old_mouse_pos: Option<Vector>,
}

struct AllocationMenu {
    y_offset: f32,
    free_button: Rectangle,
    free_text: Asset<Image>,

    coalesce_left_button: Rectangle,
    coalesce_left_text: Asset<Image>,
    coalesce_right_button: Rectangle,
    coalesce_right_text: Asset<Image>,

    split_button: Rectangle,
    split_text: Asset<Image>,
    split_selected: bool,

    allocate_button: Rectangle,
    allocate_text: Asset<Image>,
    allocate_selected: bool,

    get_input: bool,
    input: i32,
    input_prompt: Asset<Image>,

    font_size: Vector,
    font_num_map: Asset<Image>,
}

impl AllocationMenu {
    fn new(font_size_x: f32, font_size_y: f32, y_offset: f32) -> Result<Self> {
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

        let allocate_asset = Asset::new(Font::load("mononoki-Regular.ttf")
            .and_then(move |font| {
                let style = FontStyle::new(36.0, Color::BLACK);
                result(font.render("allocate", &style))
            }));

        let input_asset = Asset::new(Font::load("mononoki-Regular.ttf")
            .and_then(move |font| {
                let style = FontStyle::new(36.0, Color::BLACK);
                result(font.render("Input number of bytes: ", &style))
            }));

        let coalesce_right_asset = Asset::new(Font::load("mononoki-Regular.ttf")
            .and_then(move |font| {
                let style = FontStyle::new(36.0, Color::BLACK);
                result(font.render("coalesce-r", &style))
            }));

        let coalesce_left_asset = Asset::new(Font::load("mononoki-Regular.ttf")
            .and_then(move |font| {
                let style = FontStyle::new(36.0, Color::BLACK);
                result(font.render("coalesce-l", &style))
            }));

        let split_asset = Asset::new(Font::load("mononoki-Regular.ttf")
            .and_then(move |font| {
                let style = FontStyle::new(36.0, Color::BLACK);
                result(font.render("split", &style))
            }));

        let center_x = (SBRK_MENU_PX as f32) + 5.0;
        let center_y = y_offset + (SBRK_MENU_PX as f32)/2.0;

        Ok(AllocationMenu {
            y_offset: y_offset,
            free_button: Rectangle::new((0, 0), (2*SBRK_MENU_PX, SBRK_MENU_PX))
                .with_center((center_x, center_y)),
            free_text: free_asset,

            allocate_button: Rectangle::new((0, 0), (2*SBRK_MENU_PX, SBRK_MENU_PX))
                .with_center((center_x, center_y)),
            allocate_text: allocate_asset,
            allocate_selected: false,

            coalesce_left_button: Rectangle::new((0, 0), (2*SBRK_MENU_PX, SBRK_MENU_PX))
                .with_center((center_x, center_y + 2.0 * (SBRK_MENU_PX as f32))),
            coalesce_left_text: coalesce_left_asset,
            coalesce_right_button: Rectangle::new((0, 0), (2*SBRK_MENU_PX, SBRK_MENU_PX))
                .with_center((center_x + 2.0 * (SBRK_MENU_PX as f32), center_y + 2.0 * (SBRK_MENU_PX as f32))),
            coalesce_right_text: coalesce_right_asset,

            split_button: Rectangle::new((0, 0), (2*SBRK_MENU_PX, SBRK_MENU_PX))
                .with_center((center_x, center_y + 4.0 * (SBRK_MENU_PX as f32))),
            split_text: split_asset,
            split_selected: false,


            get_input: false,
            input: 0,
            input_prompt: input_asset,

            font_size: Vector::new(font_size_x, font_size_y),
            font_num_map: font_num_map,
        })
    }

    fn begin_input(&mut self) {
        self.get_input = true;
        self.input = 0;
    }

    fn end_input(&mut self) {
        self.get_input = false;
    }

    fn draw_free_button(&mut self, window: &mut Window) -> Result<()> {
        let free_rect = self.free_button;

        window.draw(&free_rect, Col(Color::CYAN));
        self.free_text.execute(|image| {
            window.draw(&image.area().with_center(free_rect.center()), Img(&image));
            Ok(())
        })?;

        Ok(())
    }

    fn draw_allocate_button(&mut self, window: &mut Window) -> Result<()> {
        let allocate_rect = self.allocate_button;

        window.draw(&allocate_rect, Col(Color::CYAN));
        self.allocate_text.execute(|image| {
            window.draw(&image.area().with_center(allocate_rect.center()), Img(&image));
            Ok(())
        })?;

        Ok(())
    }

    fn draw_input_box(&mut self, window: &mut Window) -> Result<()> {
        let y_offset = self.y_offset;
        self.input_prompt.execute(|image| {
            window.draw(&image.area().with_center(
                    ((MEM_GAP as f32) + image.area().width()/2.0, y_offset)), Img(&image));
            Ok(())
        })?;

        let x = self.font_size.x;
        let y = self.font_size.y;
        let input = self.input.to_string();
        self.font_num_map.execute(|image| {
            for (i, c) in input.chars().enumerate() {
                let index = ((c as i32) - ('0' as i32)) as f32;
                let subimg = &image.subimage(
                    Rectangle::new((index*x, 0), (x, y)));
                let x_off = (i as f32)*x + x/2.0;
                window.draw(
                    &subimg.area().with_center((x_off, y_offset + 50.0)), Img(&subimg));
            }
            Ok(())
        })?;

        Ok(())
    }

    fn draw_coalesce_menu(&mut self, window: &mut Window) -> Result<()> {
        let coalesce_l_rect = self.coalesce_left_button;
        let coalesce_r_rect = self.coalesce_right_button;

        window.draw(&coalesce_l_rect, Col(Color::CYAN));
        self.coalesce_left_text.execute(|image| {
            window.draw(&image.area().with_center(coalesce_l_rect.center()), Img(&image));
            Ok(())
        })?;


        window.draw(&coalesce_r_rect, Col(Color::CYAN));
        self.coalesce_right_text.execute(|image| {
            window.draw(&image.area().with_center(coalesce_r_rect.center()), Img(&image));
            Ok(())
        })?;

        Ok(())
    }

    fn draw_split_button(&mut self, window: &mut Window) -> Result<()> {
        let split_rect = self.split_button;

        window.draw(&split_rect, Col(Color::CYAN));
        self.split_text.execute(|image| {
            window.draw(&image.area().with_center(split_rect.center()), Img(&image));
            Ok(())
        })?;

        Ok(())
    }

    fn draw(&mut self, window: &mut Window, block: &mut Block) -> Result<()> {

        // Split selected may modify MallocState.allocations so it's processed in
        // MallocState's draw
        if self.allocate_selected {
            if !self.get_input {
                if self.input <= (block.rect.width() as i32 + MEM_GAP) {
                    block.allocated = true;
                    block.space_used = self.input;
                }
                self.allocate_selected = false;
            }
        }

        if self.get_input {
            return self.draw_input_box(window);
        }

        let mut y_off = self.y_offset;

        if block.allocated {
            self.draw_free_button(window)?;
        } else {
            self.draw_allocate_button(window)?;
            self.draw_coalesce_menu(window)?;
        }
        self.draw_split_button(window)?;

        y_off += self.free_button.height() + 50.0;
        let x = self.font_size.x;
        let y = self.font_size.y;
        let blk_size =
            (block.rect.width()/(PX_PER_BYTE as f32) + MEM_GAP as f32) as i32;

        let size_str: String = blk_size.to_string();
        self.font_num_map.execute(|image| {
            for (i, c) in size_str.chars().enumerate() {
                let index = ((c as i32) - ('0' as i32)) as f32;
                let subimg = &image.subimage(
                    Rectangle::new((index*x, 0), (x, y)));
                let x_off = (i as f32)*x + x/2.0;
                window.draw(
                    &subimg.area().with_center((x_off, y_off)), Img(&subimg));
            }
            Ok(())
        })?;

        let used_str = block.space_used.to_string();
        let x_offset = (size_str.chars().count() as f32) * x + 2.0 * x;
        self.font_num_map.execute(|image| {
            for (i, c) in used_str.chars().enumerate() {
                let index = ((c as i32) - ('0' as i32)) as f32;
                let subimg = &image.subimage(
                    Rectangle::new((index*x, 0), (x, y)));
                let x_off = (i as f32)*x + x/2.0;
                window.draw(
                    &subimg.area().with_center((x_offset + x_off, y_off)), Img(&subimg));
            }
            Ok(())
        })?;

        Ok(())
    }
}

struct MallocState {
    allocations: Vec<Block>,
    alloc_menu: AllocationMenu,
    sbrk_obj: SbrkDescriptor,
    display_menu: Option<usize>,
}

static TOTAL_MEMORY: i32 = 4*1024;
static PX_PER_BYTE: i32 = 1;
static SBRK_MENU_PX: i32 = 100;
static MEM_GAP: i32 = 5;

impl MallocState {
    fn split(&mut self, idx: usize, bytes: i32) {
        if bytes < MEM_GAP + 1 {
            return;
        }
        let alloc = self.allocations[idx].rect;
        let space_used = self.allocations[idx].space_used;
        let allocated = self.allocations[idx].allocated;
        if allocated {
            if bytes > (alloc.width() as i32 - space_used + MEM_GAP) {
                return;
            }
        } else {
            if bytes > (alloc.width() as i32 + MEM_GAP) {
                return;
            }
        }

        let new_x = alloc.x() as i32 + alloc.width() as i32 + MEM_GAP - bytes;
        self.allocations.insert(idx+1, Block {
            rect: Rectangle::new((new_x, 0), (bytes - MEM_GAP, SBRK_MENU_PX)),
            allocated: false,
            space_used: 0,
        });
        self.allocations[idx].rect = Rectangle::new(
            (alloc.x(), 0), (alloc.width() as i32 - bytes, SBRK_MENU_PX));
    }

    fn coalesce(&mut self, idx1: i64, idx2: i64) -> bool{
        let i1 = idx1 as usize;
        let i2 = idx2 as usize;

        if idx1 < 0 {
            return false;
        } else if idx2 > (self.allocations.len() as i64 - 1) {
            return false;
        } else if self.allocations[i1].allocated || self.allocations[i2].allocated {
            return false;
        }

        let rect1 = self.allocations[i1].rect;
        let rect2 = self.allocations[i2].rect;

        self.allocations[i1].rect = Rectangle::new(
            (rect1.x(), 0),
            (rect1.width() + (MEM_GAP as f32) + rect2.width(), rect1.height()));
        self.allocations.remove(i2);
        return true;
    }

    fn handle_click(
            &mut self, _event: &Event, window: &mut Window) -> Result<()> {
       let mouse_pos = window.mouse().pos();
       if mouse_pos.overlaps_rectangle(&self.sbrk_obj.sbrk_rect) {
           self.sbrk_obj.selected = true;
           return Ok(());
       } else if mouse_pos.overlaps_rectangle(&self.alloc_menu.allocate_button) {
           match self.display_menu {
               Some(i) => {
                   //self.allocations[i].1 = !self.allocations[i].1;
                   if self.allocations[i].allocated {
                       self.allocations[i].allocated = false;
                       self.allocations[i].space_used = 0;
                   } else {
                       self.alloc_menu.allocate_selected = true;
                       self.alloc_menu.begin_input();
                   }
               }
               _ => {}
           }
       } else if mouse_pos.overlaps_rectangle(&self.alloc_menu.coalesce_left_button) {
           match self.display_menu {
               Some(i) => {
                   if !self.allocations[i].allocated {
                       if self.coalesce(i as i64 - 1, i as i64) {
                           self.display_menu = None;
                       }
                   } else { }
               }
               _ => {}
           }
       } else if mouse_pos.overlaps_rectangle(&self.alloc_menu.coalesce_right_button) {
           match self.display_menu {
               Some(i) => {
                    if !self.allocations[i].allocated {
                       if self.coalesce(i as i64, i as i64 +1) {
                           self.display_menu = None;
                       }
                   } else { }
               }
               _ => {}
           }
       } else if mouse_pos.overlaps_rectangle(&self.alloc_menu.split_button) {
            match self.display_menu {
               Some(i) => {
                   self.alloc_menu.split_selected = true;
                   self.alloc_menu.begin_input();
               }
               _ => {}
           }
       }

       if self.alloc_menu.allocate_selected  || self.alloc_menu.split_selected {
           return Ok(());
       }

       for (i, alloc) in (&self.allocations).iter().enumerate() {
           let rect = alloc.rect;
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
            self.allocations.push(
                Block {
                    rect: Rectangle::new(
                        (self.sbrk_obj.end_of_heap_bytes, 0),
                        (new_bytes - MEM_GAP, SBRK_MENU_PX)),
                    allocated: false,
                    space_used: 0,
                });
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

    fn handle_keypress(&mut self, c: char) -> Result<()> {
        if !self.alloc_menu.get_input {
            return Ok(());
        }

        if (c as i32) <= ('9' as i32) && (c as i32) >= ('0' as i32) {
            // TODO handle overflow!
            self.alloc_menu.input *= 10;
            self.alloc_menu.input += (c as i32) - ('0' as i32);
        } else if c == ' ' {
            self.alloc_menu.input /= 10;
        } else if c == '\n' {
            self.alloc_menu.end_input();
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

        let alloc_menu = AllocationMenu::new(
            12.0, 24.0, (SBRK_MENU_PX as f32)*2.0)?;

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
            Event::Key(Key::Return, ButtonState::Pressed) => {
                return self.handle_keypress('\n');
            }
            Event::Key(Key::Back, ButtonState::Pressed) => {
                return self.handle_keypress(' ');
            }
            Event::Typed(c) => {
                return self.handle_keypress(*c);
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
            let rect = alloc.rect;
            if alloc.allocated {
                window.draw(&rect, Col(Color::RED));
            } else {
                window.draw(&rect, Col(Color::BLUE));
            }
        }

        match self.display_menu {
            Some(i) => {
                let block_rect = self.allocations[i].rect;
                let mut color = Color::BLUE.with_red(0.5);
                if self.allocations[i].allocated {
                    color = Color::RED.with_blue(0.5);
                }
                window.draw_ex(&block_rect, Col(color), Transform::IDENTITY, 0);

                if self.alloc_menu.split_selected {
                    if !self.alloc_menu.get_input {
                        self.split(i, self.alloc_menu.input);
                        self.alloc_menu.split_selected = false;
                    }
                }
                self.alloc_menu.draw(window, &mut self.allocations[i])?;
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
