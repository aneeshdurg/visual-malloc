use crate::constants::*;

use quicksilver::{
    Future, Result,
    combinators::result,
    geom::{Shape, Rectangle, Vector},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image},
    lifecycle::{Asset, Window}
};

pub struct Block {
    pub rect: Rectangle,
    pub allocated: bool,
    pub space_used: i32,
}

pub struct SbrkDescriptor {
    pub end_of_heap_bytes: i32,
    pub sbrk: Asset<Image>,
    pub sbrk_rect: Rectangle,
    pub selected: bool,
    pub old_mouse_pos: Option<Vector>,
}


pub struct AllocationMenu {
    y_offset: f32,
    pub free_button: Rectangle,
    pub free_text: Asset<Image>,

    pub coalesce_left_button: Rectangle,
    pub coalesce_left_text: Asset<Image>,
    pub coalesce_right_button: Rectangle,
    pub coalesce_right_text: Asset<Image>,

    pub split_button: Rectangle,
    pub split_text: Asset<Image>,

    pub allocate_button: Rectangle,
    pub allocate_text: Asset<Image>,

    font_size: Vector,
    font_num_map: Asset<Image>,
}

impl AllocationMenu {
    pub fn new(font_size_x: f32, font_size_y: f32, y_offset: f32) -> Result<Self> {
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

            coalesce_left_button: Rectangle::new((0, 0), (2*SBRK_MENU_PX, SBRK_MENU_PX))
                .with_center((center_x, center_y + 2.0 * (SBRK_MENU_PX as f32))),
            coalesce_left_text: coalesce_left_asset,
            coalesce_right_button: Rectangle::new((0, 0), (2*SBRK_MENU_PX, SBRK_MENU_PX))
                .with_center((center_x + 2.0 * (SBRK_MENU_PX as f32), center_y + 2.0 * (SBRK_MENU_PX as f32))),
            coalesce_right_text: coalesce_right_asset,

            split_button: Rectangle::new((0, 0), (2*SBRK_MENU_PX, SBRK_MENU_PX))
                .with_center((center_x, center_y + 4.0 * (SBRK_MENU_PX as f32))),
            split_text: split_asset,

            font_size: Vector::new(font_size_x, font_size_y),
            font_num_map: font_num_map,
        })
    }

    pub fn draw_free_button(&mut self, window: &mut Window) -> Result<()> {
        let free_rect = self.free_button;

        window.draw(&free_rect, Col(Color::CYAN));
        self.free_text.execute(|image| {
            window.draw(&image.area().with_center(free_rect.center()), Img(&image));
            Ok(())
        })?;

        Ok(())
    }

    pub fn draw_allocate_button(&mut self, window: &mut Window) -> Result<()> {
        let allocate_rect = self.allocate_button;

        window.draw(&allocate_rect, Col(Color::CYAN));
        self.allocate_text.execute(|image| {
            window.draw(&image.area().with_center(allocate_rect.center()), Img(&image));
            Ok(())
        })?;

        Ok(())
    }


    pub fn draw_coalesce_menu(&mut self, window: &mut Window) -> Result<()> {
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

    pub fn draw_split_button(&mut self, window: &mut Window) -> Result<()> {
        let split_rect = self.split_button;

        window.draw(&split_rect, Col(Color::CYAN));
        self.split_text.execute(|image| {
            window.draw(&image.area().with_center(split_rect.center()), Img(&image));
            Ok(())
        })?;

        Ok(())
    }

    pub fn draw(&mut self, window: &mut Window, block: &mut Block) -> Result<()> {

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
