use core::marker::PhantomData;
#[allow(unused)]
use micromath::F32Ext as _;

use embedded_graphics::prelude::{PixelColor, Point, Size, DrawTarget};
use embedded_graphics::Drawable as DrawableGraphics;
use embedded_graphics::primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Circle, Rectangle, Primitive, Triangle};
use crate::sprite::Sprite;
use crate::util::{make_point_f32_rounded, make_size_f32_rounded, make_circle_center_radius, rectangle_union_all, prepare_sprite_buffer};
use crate::{BasicPaletteContext, ExpressionContext, Expression};
use crate::component::Component;
use crate::palette::{Palette, BasicPaletteKey};
use super::mouth::MouthContext;

pub struct DrawableBubbleMark<Color: PixelColor> {
    style: PrimitiveStyle<Color>,
    main_circle: Circle,
    small_circle: Circle,
}

impl<Color: PixelColor> DrawableBubbleMark<Color> {
    pub fn new(geometry: &EffectGeometry, offset: f32, color: Color) -> Self {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .stroke_width(1)
            .fill_color(color)
            .build();
        let x = geometry.position.x;
        let y = geometry.position.y;
        let r = geometry.size;
        let r = r as f32 + ((r as f32) * 0.2 * (offset as f32)).floor();
        let r_small = (r / 4.0).round();
        Self {
            style,
            main_circle: Circle::new(make_point_f32_rounded(x as f32 - r, y as f32 - r), (r * 2.0) as u32),
            small_circle: Circle::new(make_point_f32_rounded(x as f32 - r_small, y as f32 - r_small), (r_small * 2.0) as u32),
        }
    }
    pub fn bounding_box(geometry: &EffectGeometry) -> Rectangle {
        let x = geometry.position.x as f32;
        let y = geometry.position.y as f32;
        let r = geometry.size as f32;
        let r = (r + r * 0.2).floor();
        let r_small = (r / 4.0).round();

        let left = x - r_small * 2.0;
        let top = y - r_small * 2.0;
        let width = (r * 3.0 + 1.0).ceil() as u32;
        let height = (r * 3.0 + 1.0).ceil() as u32;
        let left = left.floor() as i32;
        let top = top.floor() as i32;
        Rectangle::new(
            Point::new(left, top),
            Size::new(width, height),
        )
    }
}

impl<Color: PixelColor> DrawableGraphics for DrawableBubbleMark<Color> {
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        self.main_circle.into_styled(self.style.clone()).draw(target)?;
        self.small_circle.into_styled(self.style.clone()).draw(target)?;
        Ok(())
    }
}

pub struct DrawableSweatMark<Color: PixelColor> {
    style: PrimitiveStyle<Color>,
    circle: Circle,
    triangle: Triangle,
}

impl<Color: PixelColor> DrawableSweatMark<Color> {
    pub fn new(geometry: &EffectGeometry,  offset: f32, color: Color) -> Self {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .stroke_width(1)
            .fill_color(color)
            .build();
        let x = geometry.position.x;
        let y = geometry.position.y;
        let r = geometry.size;
        let y = y + (offset * 5.0).floor() as i32;
        let r = r as f32 + ((r as f32) * 0.2 * (offset as f32)).floor();
        let a = 1.7320508 * r / 2.0;
        Self {
            style,
            circle: Circle::new(make_point_f32_rounded(x as f32 - r, y as f32 - r), (r * 2.0).round() as u32),
            triangle: Triangle::new(
                make_point_f32_rounded(x as f32, y as f32 - r * 2.0),
                make_point_f32_rounded(x as f32 - a, y as f32 - r * 0.5),
                make_point_f32_rounded(x as f32 + a, y as f32 - r * 0.5),
            ),
        }
    }
    pub fn bounding_box(geometry: &EffectGeometry) -> Rectangle {
        let x = geometry.position.x as f32;
        let y = geometry.position.y as f32;
        let r = geometry.size as f32;
        let r = (r + r * 0.2).floor();
        let a = 1.7320508 * r / 2.0;

        let left = x - r;
        let top = y;
        let width = (r * 2.0 + 1.0).ceil() as u32;
        let height = (r * 3.0 + 5.0 + 1.0).ceil() as u32;
        let left = left.floor() as i32;
        let top = top.floor() as i32;
        Rectangle::new(
            Point::new(left, top),
            Size::new(width, height),
        )
    }
}

impl<Color: PixelColor> DrawableGraphics for DrawableSweatMark<Color> {
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        self.circle.into_styled(self.style).draw(target)?;
        self.triangle.into_styled(self.style).draw(target)?;
        Ok(())
    }
}


pub struct DrawableChillMark<Color: PixelColor> {
    style: PrimitiveStyle<Color>,
    rect0: Rectangle,
    rect1: Rectangle,
    rect2: Rectangle,
}

impl<Color: PixelColor> DrawableChillMark<Color> {
    pub fn new(geometry: &EffectGeometry,  offset: f32, color: Color) -> Self {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .stroke_width(1)
            .fill_color(color)
            .build();
        let x = geometry.position.x;
        let y = geometry.position.y;
        let r = geometry.size;
        let h = r as f32 + (r as f32 * 0.2 * offset).abs();
        let h_div_2 = (h / 2.0).round();

        Self {
            style,
            rect0: Rectangle::new(make_point_f32_rounded(x as f32 - h_div_2, y as f32), Size::new(3, h_div_2 as u32)),
            rect1: Rectangle::new(Point::new(x, y), Size::new(3, (h * 3.0 / 4.0) as u32)),
            rect2: Rectangle::new(make_point_f32_rounded(x as f32 + h_div_2, y as f32), Size::new(3, h as u32)),
        }
    }
    pub fn bounding_box(geometry: &EffectGeometry) -> Rectangle {
        let x = geometry.position.x as f32;
        let y = geometry.position.y as f32;
        let r = geometry.size as f32;
        let h = r + r * 0.2;
        let h_div_2 = (h / 2.0).round();

        let left = x - h_div_2;
        let top = y;
        let width = (h + 3.0 + 1.0).ceil() as u32;
        let height = (h * 3.0 / 4.0 + 1.0).ceil() as u32;
        let left = left.floor() as i32;
        let top = top.floor() as i32;
        Rectangle::new(
            Point::new(left, top),
            Size::new(width, height),
        )
    }
}

impl<Color: PixelColor> DrawableGraphics for DrawableChillMark<Color> {
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        self.rect0.into_styled(self.style).draw(target)?;
        self.rect1.into_styled(self.style).draw(target)?;
        self.rect2.into_styled(self.style).draw(target)?;
        Ok(())
    }
}

pub struct DrawableAngerMark<Color: PixelColor> {
    style: PrimitiveStyle<Color>,
    rect0: Rectangle,
    rect1: Rectangle,
    rect2: Rectangle,
    rect3: Rectangle,
}

impl<Color: PixelColor> DrawableAngerMark<Color> {
    pub fn new(geometry: &EffectGeometry, offset: f32, color: Color) -> Self {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .stroke_width(1)
            .fill_color(color)
            .build();
        
        let r = geometry.size;
        let r = r as f32 + (r as f32 * 0.4 * offset);
        let x = geometry.position.x as f32;
        let y = geometry.position.y as f32;
        Self {
            style,
            rect0: Rectangle::new(make_point_f32_rounded(x - r / 3.0, y - r), make_size_f32_rounded(r * 2.0 / 3.0, r * 2.0)),
            rect1: Rectangle::new(make_point_f32_rounded(x - r, y - r / 3.0), make_size_f32_rounded(r * 2.0, r * 2.0 / 3.0)),
            rect2: Rectangle::new(make_point_f32_rounded(x - r / 3.0 + 2.0, y - r), make_size_f32_rounded(r * 2.0 / 3.0 - 4.0, r * 2.0)),
            rect3: Rectangle::new(make_point_f32_rounded(x - r, y - r / 3.0 + 2.0), make_size_f32_rounded(r * 2.0 / 3.0, r * 2.0 / 3.0 - 4.0)),
        }
    }
    pub fn bounding_box(geometry: &EffectGeometry) -> Rectangle {
        let r = geometry.size as f32;
        let r = r + r * 0.4;
        let x = geometry.position.x as f32;
        let y = geometry.position.y as f32;
        let left = x - r;
        let top = y - r;
        let width = (r * 2.0 + 1.0).ceil() as u32;
        let height = (r * 2.0 + 1.0).ceil() as u32;
        let left = left.floor() as i32;
        let top = top.floor() as i32;
        Rectangle::new(
            Point::new(left, top),
            Size::new(width, height),
        )
    }
}

impl<Color: PixelColor> DrawableGraphics for DrawableAngerMark<Color> {
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        self.rect0.into_styled(self.style).draw(target)?;
        self.rect1.into_styled(self.style).draw(target)?;
        self.rect2.into_styled(self.style).draw(target)?;
        self.rect3.into_styled(self.style).draw(target)?;
        Ok(())
    }
}

pub struct DrawableHeartMark<Color: PixelColor> {
    style: PrimitiveStyle<Color>,
    circle0: Circle,
    circle1: Circle,
    triangle0: Triangle,
    triangle1: Triangle,
}

impl<Color: PixelColor> DrawableHeartMark<Color> {
    pub fn new(geometry: &EffectGeometry, offset: f32, color: Color) -> Self {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .stroke_width(1)
            .fill_color(color)
            .build();
        let r = geometry.size;
        let r = r as f32 + (r as f32 * 0.4 * offset);
        let x = geometry.position.x as f32;
        let y = geometry.position.y as f32;
        let a = r * 1.41421356 / 4.0;
        Self {
            style,
            circle0: make_circle_center_radius(x - r / 2.0, y, r / 2.0),
            circle1: make_circle_center_radius(x + r / 2.0, y, r / 2.0),
            triangle0: Triangle::new(
                make_point_f32_rounded(x, y),
                make_point_f32_rounded(x - r / 2.0 - a, y + a),
                make_point_f32_rounded(x + r / 2.0 + a, y + a),
            ),
            triangle1: Triangle::new(
                make_point_f32_rounded(x, y + r / 2.0 + 2.0 * a),
                make_point_f32_rounded(x - r / 2.0 - a, y + a),
                make_point_f32_rounded(x + r / 2.0 + a, y + a),
            ),
        }
    }
    pub fn bounding_box(geometry: &EffectGeometry) -> Rectangle {
        let r = geometry.size as f32;
        let r = r + (r * 0.4);
        let x = geometry.position.x as f32;
        let y = geometry.position.y as f32;
        let a = r * 1.41421356 / 4.0;
        let left = x - r / 2.0 - a;
        let right = x + r / 2.0 + a;
        let top = y - r / 2.0;
        let bottom = y + r / 2.0 + 2.0 * a;
        let width = (right - left).ceil();
        let height = (bottom - top).ceil();
        Rectangle::new(
            Point::new(left.floor() as i32, top.floor() as i32),
            Size::new(width as u32, height as u32),
        )
    }
}

impl<Color: PixelColor> DrawableGraphics for DrawableHeartMark<Color> {
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        self.circle0.into_styled(self.style).draw(target)?;
        self.circle1.into_styled(self.style).draw(target)?;
        self.triangle0.into_styled(self.style).draw(target)?;
        self.triangle1.into_styled(self.style).draw(target)?;
        Ok(())
    }
}

pub struct EffectGeometry {
    pub position: Point, 
    pub size: u32,
}

pub struct Effect<'a, Context: MouthContext<'a> + BasicPaletteContext<'a> + ExpressionContext> {
    sweat_geometry: EffectGeometry,
    anger_geometry: EffectGeometry,
    heart_geometry: EffectGeometry,
    chill_geometry: EffectGeometry,
    bubble_geometries: [EffectGeometry; 2],
    context: PhantomData<&'a Context>,
}

impl<'a, Context: MouthContext<'a> + BasicPaletteContext<'a> + ExpressionContext> Effect<'a, Context> {
    pub fn new() -> Self {
        Self {
            sweat_geometry: EffectGeometry { position: Point::new(290, 110), size: 7 },
            anger_geometry: EffectGeometry { position: Point::new(280, 50), size: 12 },
            heart_geometry: EffectGeometry { position: Point::new(280, 50), size: 12 },
            chill_geometry: EffectGeometry { position: Point::new(270, 0), size: 30 },
            bubble_geometries: [
                EffectGeometry { position: Point::new(290, 40), size: 10 },
                EffectGeometry { position: Point::new(270, 52), size: 6 },
            ],
            context: PhantomData{},
        }
    }
}

pub enum DrawableEffectMark<Color: PixelColor> {
    Sweat(DrawableSweatMark<Color>),
    Anger(DrawableAngerMark<Color>),
    Heart(DrawableHeartMark<Color>),
    Chill(DrawableChillMark<Color>),
    Bubbles((DrawableBubbleMark<Color>, DrawableBubbleMark<Color>)),
}
pub struct DrawableEffect<Color: PixelColor> {
    background_color: Color,
    mark: Option<DrawableEffectMark<Color>>,
    bounding_box: Rectangle,
}

impl<Color: PixelColor + Into<Color::Raw> + From<Color::Raw>> DrawableGraphics for DrawableEffect<Color> {
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        
        let mut buffer = prepare_sprite_buffer::<Color>(self.bounding_box);
        let mut sprite = Sprite::<Color>::new_unaligned(&mut buffer, self.bounding_box).unwrap();
        sprite.clear(self.background_color);
        match &self.mark {
            Some(DrawableEffectMark::Sweat(mark)) => { mark.draw(&mut sprite); },
            Some(DrawableEffectMark::Anger(mark)) => { mark.draw(&mut sprite); },
            Some(DrawableEffectMark::Heart(mark)) => { mark.draw(&mut sprite); },
            Some(DrawableEffectMark::Chill(mark)) => { mark.draw(&mut sprite); },
            Some(DrawableEffectMark::Bubbles((mark0, mark1))) => {
                mark0.draw(&mut sprite);
                mark1.draw(&mut sprite);
            },
            None => {},
        }
        sprite.draw(target)?;
        Ok(())
    }
}

impl<'a, Context: MouthContext<'a> + BasicPaletteContext<'a> + ExpressionContext> Component<'a> for Effect<'a, Context> 
    where Context::Color: From<<Context::Color as PixelColor>::Raw> + Into<<Context::Color as PixelColor>::Raw> 
{
    type Context = Context;
    type Drawable = DrawableEffect<<Context as BasicPaletteContext<'a>>::Color>;
    fn render(&self, bounding_rect: embedded_graphics::primitives::Rectangle, context: &'a Self::Context) -> Self::Drawable {
        let foreground_color = context.get_basic_palette().get_color(&BasicPaletteKey::Primary);
        let background_color = context.get_basic_palette().get_color(&BasicPaletteKey::Background);
        
        let bounding_box = rectangle_union_all(&[
            DrawableSweatMark::<Context::Color>::bounding_box(&self.sweat_geometry),
            DrawableAngerMark::<Context::Color>::bounding_box(&self.anger_geometry),
            DrawableHeartMark::<Context::Color>::bounding_box(&self.heart_geometry),
            DrawableChillMark::<Context::Color>::bounding_box(&self.chill_geometry),
            DrawableBubbleMark::<Context::Color>::bounding_box(&self.bubble_geometries[0]),
            DrawableBubbleMark::<Context::Color>::bounding_box(&self.bubble_geometries[1]),
        ]).unwrap();

        let offset = context.breath();
        let expression = context.expression();
        let drawable_effect = match expression {
            Expression::Doubt => Some(DrawableEffectMark::Sweat(DrawableSweatMark::new(&self.sweat_geometry, offset, foreground_color))),
            Expression::Angry => Some(DrawableEffectMark::Anger(DrawableAngerMark::new(&self.anger_geometry, offset, foreground_color))),
            Expression::Happy => Some(DrawableEffectMark::Heart(DrawableHeartMark::new(&self.heart_geometry, offset, foreground_color))),
            Expression::Sad => Some(DrawableEffectMark::Chill(DrawableChillMark::new(&self.chill_geometry, offset, foreground_color))),
            Expression::Sleepy => Some(
                DrawableEffectMark::Bubbles((
                    DrawableBubbleMark::new(&self.bubble_geometries[0], offset, foreground_color),
                    DrawableBubbleMark::new(&self.bubble_geometries[1], offset, foreground_color),
                ),
            )),
            _ => None,
        };
        Self::Drawable {
            background_color,
            mark: drawable_effect,
            bounding_box,
        }
    }
}