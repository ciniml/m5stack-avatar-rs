use core::marker::PhantomData;

use embedded_graphics::{Drawable, draw_target::DrawTarget, primitives::Rectangle, prelude::{PixelColor, Point, Size, Dimensions, RawData, IntoStorage}, Pixel};

pub struct Sprite<'a, Color: PixelColor> {
    buffer: &'a mut [Color],
    geometry: Rectangle,
    _color: PhantomData<Color>,
}

impl<'a, Color> Sprite<'a, Color> 
    where Color: PixelColor
{
    pub fn new(buffer: &'a mut [Color], geometry: Rectangle) -> Result<Self, ()> {
        let required_length = (geometry.size.width * geometry.size.height) as usize;
        if buffer.len() < required_length {
            return Err(())
        }
        Ok(Self {
            buffer,
            geometry,
            _color: PhantomData{},
        })
    }
    pub const fn unaligned_buffer_size(width: u32, height: u32) -> usize {
        let required_length = (width * height) as usize;
        let required_bytes = required_length * core::mem::size_of::<<<Color as PixelColor>::Raw as RawData>::Storage>();
        let alignment = core::mem::align_of::<<<Color as PixelColor>::Raw as RawData>::Storage>();
        required_bytes + alignment - 1
    }
    pub fn new_unaligned(buffer: &'a mut[u8], geometry: Rectangle) -> Result<Self, ()> {
        let required_length = (geometry.size.width * geometry.size.height) as usize;
        let required_bytes = required_length * core::mem::size_of::<Color>();
        let buffer = unsafe {
            let ptr = buffer.as_mut_ptr();
            let alignment = core::mem::align_of::<Color>();
            let offset = ptr.align_offset(alignment);
            if offset ==  usize::MAX {
                return Err(());
            }
            let ptr_aligned = ptr.add(offset);
            if buffer.len() < required_bytes + offset {
                return Err(());
            }
            let ptr_color = ptr_aligned as *mut Color;
            core::slice::from_raw_parts_mut(ptr_color, required_length)
        };
        Self::new(buffer, geometry)
    }
}


impl<'a, Color: PixelColor + Into<<Color as PixelColor>::Raw>> Dimensions for Sprite<'a, Color> {
    fn bounding_box(&self) -> Rectangle {
        self.geometry
    }
}
impl<'a, Color: PixelColor + Into<<Color as PixelColor>::Raw>> DrawTarget for Sprite<'a, Color> {
    type Color = Color;
    type Error = core::convert::Infallible;
    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let valid_area = self.geometry.intersection(area);
        let bottom_right = if let Some(bottom_right) = valid_area.bottom_right() {
            bottom_right
        } else {
            return Ok(())
        };
        let top_left = valid_area.top_left;
        for y in top_left.y..=bottom_right.y {
            let line_start = (y - self.geometry.top_left.y) as u32 * self.geometry.size.width;
            for x in top_left.x..=bottom_right.x {
                let index = line_start + (x - self.geometry.top_left.x) as u32;
                self.buffer[index as usize] =  color;
            }
        }
        Ok(())
    }
    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Self::Color>, {
                let valid_area = self.geometry.intersection(area);
        let bottom_right = if let Some(bottom_right) = valid_area.bottom_right() {
            bottom_right
        } else {
            return Ok(())
        };
        let top_left = valid_area.top_left;
        let mut iter = colors.into_iter();
        'outer: for y in area.top_left.y..=bottom_right.y{
            if y < top_left.y {
                for _x in 0..area.size.width {
                    if iter.next().is_none() {
                        break 'outer;
                    }
                }
                continue;
            }
            let line_start = (y - self.geometry.top_left.y) as u32 * self.geometry.size.width;
            for _x in area.top_left.x..top_left.x {
                iter.next();
            }
            for x in top_left.x..=bottom_right.x {
                let index = line_start + (x - self.geometry.top_left.x) as u32;
                if let Some(color) = iter.next() {
                    self.buffer[index as usize] = color;
                } else {
                    break 'outer;
                }
            }
            for _x in bottom_right.x + 1..=area.top_left.x + area.size.width as i32 {
                iter.next();
            }
        }
        Ok(())
    }
    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let geometry = self.geometry;
        self.fill_solid(&geometry, color)
    }
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>> {
        for Pixel(location, color) in pixels {
            if !self.geometry.contains(location) {
                continue;
            }
            let offset = location - self.geometry.top_left;
            let line_start = offset.y as u32 * self.geometry.size.width;
            let index = line_start + offset.x as u32;
            self.buffer[index as usize] = color;
        }
        Ok(())
    }
}

impl<'a, Color: PixelColor + From<Color::Raw>> Drawable for Sprite<'a, Color> 
{
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: DrawTarget<Color = Self::Color> {
        target.fill_contiguous(&self.geometry, self.buffer.as_ref().into_iter().map(|c| c.clone()))?;
        Ok(())
    }
}