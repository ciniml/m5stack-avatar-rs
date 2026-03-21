use core::marker::PhantomData;
#[allow(unused)]
use micromath::F32Ext as _;

use embedded_graphics::prelude::{PixelColor, Point, Size, DrawTarget};
use embedded_graphics::Drawable as DrawableGraphics;
use embedded_graphics::primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Circle, Rectangle, Primitive, Triangle};
use crate::sprite::Sprite;
use crate::util::prepare_sprite_buffer;
use crate::{BasicPaletteContext, ExpressionContext, Expression};
use crate::component::Component;
use crate::palette::{Palette, BasicPaletteKey};

use super::mouth::MouthContext;

pub struct Eye<'a, Context: EyeContext<'a>> {
    radius: f32,
    is_left: bool,
    context: PhantomData<&'a Context>,
}

impl<'a, Context: EyeContext<'a>> Eye<'a, Context> {
    pub fn new(radius: f32, is_left: bool) -> Self {
        Self {
            radius,
            is_left,
            context: PhantomData::default(),
        }
    }
}

pub trait EyeContext<'a>: BasicPaletteContext<'a> +  GazeContext + ExpressionContext + MouthContext<'a> {
    fn open_ratio(&self) -> f32;
    fn set_open_ratio(&mut self, value: f32);
}

pub trait GazeContext {
    fn horizontal(&self) -> f32;
    fn set_horizontal(&mut self, value: f32);
    fn vertical(&self) -> f32;
    fn set_vertical(&mut self, value: f32);
}


pub struct DrawableEye<Color: PixelColor> {
    bounding_box: Rectangle,
    background_color: Color,
    style: PrimitiveStyle<Color>,
    mask_style: PrimitiveStyle<Color>,
    open_eye_main: Option<Circle>,
    open_eye_triangle: Option<Triangle>,
    open_eye_happy_circle: Option<Circle>,
    open_eye_half_mask: Option<Rectangle>,
    close_eye: Option<Rectangle>,
}

impl<Color: PixelColor + Into<Color::Raw> + From<Color::Raw>> DrawableGraphics for DrawableEye<Color> {
    type Color = Color;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where
            D: embedded_graphics::prelude::DrawTarget<Color = Self::Color> {
        let mut buffer = prepare_sprite_buffer::<Color>(self.bounding_box);
        let mut sprite = Sprite::<Color>::new_unaligned(&mut buffer, self.bounding_box).unwrap();
        sprite.clear(self.background_color).ok();
        self.open_eye_main.map_or(Ok(()), |p| p.into_styled(self.style.clone()).draw(&mut sprite)).ok();
        self.open_eye_triangle.map_or(Ok(()), |p| p.into_styled(self.mask_style.clone()).draw(&mut sprite)).ok();
        self.open_eye_happy_circle.map_or(Ok(()), |p| p.into_styled(self.mask_style.clone()).draw(&mut sprite)).ok();
        self.open_eye_half_mask.map_or(Ok(()), |p| p.into_styled(self.mask_style.clone()).draw(&mut sprite)).ok();
        self.close_eye.map_or(Ok(()), |p| p.into_styled(self.style.clone()).draw(&mut sprite)).ok();
        sprite.draw(target)?;
        Ok(())
    }
}

impl <'a, Context: EyeContext<'a>> Component<'a> for Eye<'a, Context> 
    where Context::Color: From<<Context::Color as PixelColor>::Raw> + Into<<Context::Color as PixelColor>::Raw> 
{
    type Context = Context;
    type Drawable = DrawableEye<Context::Color>;
    fn render(&self, bounding_rect: Rectangle, context: &'a Self::Context) -> Self::Drawable {
        let foreground_color = context.get_basic_palette().get_color(&BasicPaletteKey::Primary);
        let background_color = context.get_basic_palette().get_color(&BasicPaletteKey::Background);
        let open_ratio = EyeContext::open_ratio(context);
        let breath_offset = context.breath();
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(foreground_color)
            .stroke_width(1)
            .fill_color(foreground_color)
            .build();
        let mask_style = PrimitiveStyleBuilder::new()
            .stroke_color(background_color)
            .stroke_width(1)
            .fill_color(background_color)
            .build();
        let center = bounding_rect.center();
        let offset_scale = self.radius / 8.0;
        let x = center.x as f32 + breath_offset * 3.0 * offset_scale;
        let y = center.y as f32 + breath_offset * 3.0 * offset_scale;
        let margin = (6.0 * offset_scale) as i32;
        let bounding_box = Rectangle::new(
            center - Point::new(self.radius.ceil() as i32 + margin, self.radius.ceil() as i32 + margin),
            Size::new((self.radius * 2.0 + (margin * 2) as f32).ceil() as u32, (self.radius * 2.0 + (margin * 2) as f32).ceil() as u32),
        );
        let offset_x = context.horizontal() * 3.0 * offset_scale;
        let offset_y = context.vertical() * 3.0 * offset_scale;
        let expression = context.expression();
        if open_ratio > 0.0 {
            let body = Circle::new(Point::new((x + offset_x - self.radius) as i32, (y + offset_y - self.radius) as i32), (self.radius * 2.0) as u32);
            match expression {
                Expression::Angry | Expression::Sad => {
                    let x0 = x + offset_x - self.radius;
                    let y0 = y + offset_y - self.radius;
                    let x1 = x0 + self.radius * 2.0;
                    let y1 = y0;
                    let x2 = if self.is_left ^ (expression == Expression::Angry) { x0 } else {  x1 };
                    let y2 = y0 + self.radius;
                    let triangle = Triangle::new(Point::new(x0 as i32, y0 as i32), Point::new(x1 as i32, y1 as i32), Point::new(x2 as i32, y2 as i32));
                    Self::Drawable {
                        bounding_box,
                        background_color,
                        style,
                        mask_style,
                        open_eye_main: Some(body),
                        open_eye_triangle: Some(triangle),
                        open_eye_happy_circle: None,
                        open_eye_half_mask: None,
                        close_eye: None,
                    }
                },
                Expression::Happy | Expression::Sleepy => {
                    let x0 = x + offset_x - self.radius;
                    let y0 = y + offset_y - self.radius + if expression == Expression::Happy { self.radius } else { 0.0 };
                    let w = self.radius * 2.0 + 4.0 * offset_scale;
                    let h = self.radius + 2.0 * offset_scale;
                    let open_eye_happy_circle = if expression == Expression::Happy {
                        let radius = self.radius / 1.5;
                        Some(Circle::new(Point::new((x + offset_x - radius).round() as i32, (y + offset_y - radius).round() as i32), (radius * 2.0).round() as u32))
                    } else {
                        None
                    };
                    let open_eye_half_mask = Some(Rectangle::new(
                        Point::new(x0 as i32, y0 as i32),
                        Size::new(w as u32, h as u32)
                    ));
                    Self::Drawable {
                        bounding_box,
                        background_color,
                        style,
                        mask_style,
                        open_eye_main: Some(body),
                        open_eye_triangle: None,
                        open_eye_happy_circle,
                        open_eye_half_mask,
                        close_eye: None,
                    }
                },
                _ => {
                    Self::Drawable {
                        bounding_box,
                        background_color,
                        style,
                        mask_style,
                        open_eye_main: Some(body),
                        open_eye_triangle: None,
                        open_eye_happy_circle: None,
                        open_eye_half_mask: None,
                        close_eye: None,
                    }
                }
            }
        } else {
            let h = (4.0 * offset_scale).max(1.0);
            let x1 = x - self.radius + offset_x;
            let y1 = y - h / 2.0 + offset_y;
            let w = self.radius * 2.0;
            let close_eye = Some(Rectangle::new(Point::new(x1 as i32, y1 as i32), Size::new(w as u32, h as u32)));
            Self::Drawable {
                bounding_box,
                background_color,
                style,
                mask_style,
                open_eye_main: None,
                open_eye_triangle: None,
                open_eye_happy_circle: None,
                open_eye_half_mask: None,
                close_eye,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::pixelcolor::BinaryColor;
    use embedded_graphics::prelude::Point;
    use crate::{ArrayPalette, BasicPaletteKey, BasicPaletteContext, ExpressionContext, Expression};
    use crate::components::mouth::MouthContext;

    // テスト用の最小コンテキスト
    struct TestContext {
        gaze_h: f32,
        gaze_v: f32,
        eye_open_ratio: f32,
        mouth_open_ratio: f32,
        breath: f32,
        expression: Expression,
        palette: ArrayPalette<BasicPaletteKey, BinaryColor, { BasicPaletteKey::VARIANT_COUNT }>,
    }

    impl TestContext {
        fn new() -> Self {
            Self {
                gaze_h: 0.0,
                gaze_v: 0.0,
                eye_open_ratio: 1.0,
                mouth_open_ratio: 0.0,
                breath: 0.0,
                expression: Expression::Neutral,
                palette: ArrayPalette::default(),
            }
        }
    }

    impl GazeContext for TestContext {
        fn horizontal(&self) -> f32 { self.gaze_h }
        fn set_horizontal(&mut self, v: f32) { self.gaze_h = v; }
        fn vertical(&self) -> f32 { self.gaze_v }
        fn set_vertical(&mut self, v: f32) { self.gaze_v = v; }
    }

    impl ExpressionContext for TestContext {
        fn expression(&self) -> Expression { self.expression }
    }

    impl<'a> BasicPaletteContext<'a> for TestContext {
        type BasicPalette = ArrayPalette<BasicPaletteKey, BinaryColor, { BasicPaletteKey::VARIANT_COUNT }>;
        type Color = BinaryColor;
        fn get_basic_palette(&self) -> &Self::BasicPalette { &self.palette }
    }

    impl<'a> MouthContext<'a> for TestContext {
        fn open_ratio(&self) -> f32 { self.mouth_open_ratio }
        fn set_open_ratio(&mut self, v: f32) { self.mouth_open_ratio = v; }
        fn breath(&self) -> f32 { self.breath }
        fn set_breath(&mut self, v: f32) { self.breath = v; }
    }

    impl<'a> EyeContext<'a> for TestContext {
        fn open_ratio(&self) -> f32 { self.eye_open_ratio }
        fn set_open_ratio(&mut self, v: f32) { self.eye_open_ratio = v; }
    }

    fn render_eye(radius: f32, center: Point, ctx: &TestContext) -> DrawableEye<BinaryColor> {
        let eye = Eye::<TestContext>::new(radius, false);
        eye.render(Rectangle::new(center, Size::zero()), ctx)
    }

    // 期待する開眼時の目の円の左上座標を計算（eye.rs の実装と同じ式）
    fn expected_open_eye_topleft(center: Point, radius: f32, gaze_h: f32, gaze_v: f32, breath: f32) -> Point {
        let offset_scale = radius / 8.0;
        let x = center.x as f32 + breath * 3.0 * offset_scale;
        let y = center.y as f32 + breath * 3.0 * offset_scale;
        let offset_x = gaze_h * 3.0 * offset_scale;
        let offset_y = gaze_v * 3.0 * offset_scale;
        Point::new(
            (x + offset_x - radius) as i32,
            (y + offset_y - radius) as i32,
        )
    }

    // ---- 開眼テスト ----

    /// 視線・呼吸ゼロのとき、目が指定座標を中心に描画されること
    #[test]
    fn test_open_eye_no_gaze_no_breath() {
        let ctx = TestContext::new();

        // 320x240 (radius=8)
        let c8 = Point::new(230, 96);
        let d8 = render_eye(8.0, c8, &ctx);
        let circle8 = d8.open_eye_main.unwrap();
        assert_eq!(circle8.top_left, expected_open_eye_topleft(c8, 8.0, 0.0, 0.0, 0.0));
        assert_eq!(circle8.diameter, 16, "radius=8: diameter should be 16");

        // 128x128 (radius=3)
        let c3 = Point::new(92, 54);
        let d3 = render_eye(3.0, c3, &ctx);
        let circle3 = d3.open_eye_main.unwrap();
        assert_eq!(circle3.top_left, expected_open_eye_topleft(c3, 3.0, 0.0, 0.0, 0.0));
        assert_eq!(circle3.diameter, 6, "radius=3: diameter should be 6");
    }

    /// 水平視線オフセットが radius に比例してスケールすること
    #[test]
    fn test_gaze_horizontal_scales_with_radius() {
        let mut ctx = TestContext::new();
        ctx.gaze_h = 1.0;

        let c8 = Point::new(100, 100);
        let d8 = render_eye(8.0, c8, &ctx);
        let pos8 = d8.open_eye_main.unwrap().top_left;
        assert_eq!(pos8, expected_open_eye_topleft(c8, 8.0, 1.0, 0.0, 0.0));

        let c3 = Point::new(50, 50);
        let d3 = render_eye(3.0, c3, &ctx);
        let pos3 = d3.open_eye_main.unwrap().top_left;
        assert_eq!(pos3, expected_open_eye_topleft(c3, 3.0, 1.0, 0.0, 0.0));

        // 視線オフセット量が radius に対して同じ比率であることを確認
        // radius=8: shift=3px, 3/8=0.375
        // radius=3: shift=1px (1.125を切り捨て), 1/3≈0.333
        // 整数化による誤差を考慮した許容範囲(1px / min_radius)
        let no_gaze8 = expected_open_eye_topleft(c8, 8.0, 0.0, 0.0, 0.0);
        let no_gaze3 = expected_open_eye_topleft(c3, 3.0, 0.0, 0.0, 0.0);
        let shift8 = (pos8.x - no_gaze8.x) as f32;
        let shift3 = (pos3.x - no_gaze3.x) as f32;
        let ratio8 = shift8 / 8.0;
        let ratio3 = shift3 / 3.0;
        let tolerance = 1.0 / 3.0; // 最小 radius(3) での1px 誤差による最大ずれ
        assert!(
            (ratio8 - ratio3).abs() < tolerance,
            "Gaze ratio mismatch: radius=8 shift={} ({:.3}), radius=3 shift={} ({:.3})",
            shift8, ratio8, shift3, ratio3,
        );
    }

    /// 垂直視線オフセットが radius に比例してスケールすること
    #[test]
    fn test_gaze_vertical_scales_with_radius() {
        let mut ctx = TestContext::new();
        ctx.gaze_v = 1.0;

        let c8 = Point::new(100, 100);
        let pos8 = render_eye(8.0, c8, &ctx).open_eye_main.unwrap().top_left;
        assert_eq!(pos8, expected_open_eye_topleft(c8, 8.0, 0.0, 1.0, 0.0));

        let c3 = Point::new(50, 50);
        let pos3 = render_eye(3.0, c3, &ctx).open_eye_main.unwrap().top_left;
        assert_eq!(pos3, expected_open_eye_topleft(c3, 3.0, 0.0, 1.0, 0.0));
    }

    /// 呼吸オフセットが radius に比例してスケールすること
    #[test]
    fn test_breath_offset_scales_with_radius() {
        let mut ctx = TestContext::new();
        ctx.breath = 1.0;

        let c8 = Point::new(100, 100);
        let pos8 = render_eye(8.0, c8, &ctx).open_eye_main.unwrap().top_left;
        assert_eq!(pos8, expected_open_eye_topleft(c8, 8.0, 0.0, 0.0, 1.0));

        let c3 = Point::new(50, 50);
        let pos3 = render_eye(3.0, c3, &ctx).open_eye_main.unwrap().top_left;
        assert_eq!(pos3, expected_open_eye_topleft(c3, 3.0, 0.0, 0.0, 1.0));
    }

    /// 視線オフセットが radius を超えないこと（旧バグの回帰テスト）
    /// 修正前: radius=3 でも 3px 固定 → 目が自分の半径分移動してしまった
    /// 修正後: radius に比例するため radius=3 では約 1px に収まる
    #[test]
    fn test_gaze_offset_does_not_exceed_radius() {
        let ctx_center = TestContext::new();
        let mut ctx_gaze = TestContext::new();
        ctx_gaze.gaze_h = 1.0;

        for (radius, center) in [(8.0f32, Point::new(100, 100)), (3.0f32, Point::new(50, 50))] {
            let center_x = render_eye(radius, center, &ctx_center)
                .open_eye_main.unwrap().top_left.x;
            let gaze_x = render_eye(radius, center, &ctx_gaze)
                .open_eye_main.unwrap().top_left.x;
            let shift = (gaze_x - center_x).abs() as f32;
            assert!(
                shift < radius,
                "radius={}: gaze shift {}px should be less than radius",
                radius, shift,
            );
        }
    }

    // ---- 閉眼テスト ----

    /// 閉眼の高さが radius に比例すること
    #[test]
    fn test_close_eye_height_proportional_to_radius() {
        let mut ctx = TestContext::new();
        ctx.eye_open_ratio = 0.0;

        // radius=8: h = (4.0 * 1.0).max(1.0) = 4.0 → u32 = 4
        let d8 = render_eye(8.0, Point::new(100, 100), &ctx);
        let close8 = d8.close_eye.unwrap();
        assert_eq!(close8.size.height, 4, "radius=8: expected height=4");
        assert_eq!(close8.size.width, 16, "radius=8: expected width=16");

        // radius=3: h = (4.0 * 0.375).max(1.0) = 1.5 → u32 = 1
        let d3 = render_eye(3.0, Point::new(50, 50), &ctx);
        let close3 = d3.close_eye.unwrap();
        assert_eq!(close3.size.height, 1, "radius=3: expected height=1");
        assert_eq!(close3.size.width, 6, "radius=3: expected width=6");
    }

    /// 閉眼の矩形が目の中心に垂直方向で揃っていること
    #[test]
    fn test_close_eye_vertically_centered() {
        let mut ctx = TestContext::new();
        ctx.eye_open_ratio = 0.0;

        for (radius, center) in [(8.0f32, Point::new(100, 100)), (3.0f32, Point::new(50, 50))] {
            let d = render_eye(radius, center, &ctx);
            let close = d.close_eye.unwrap();
            let h = (4.0 * radius / 8.0).max(1.0);
            // y1 = center.y - h/2.0 (breath=0, gaze=0)
            let expected_y = (center.y as f32 - h / 2.0) as i32;
            assert_eq!(
                close.top_left.y, expected_y,
                "radius={}: close eye top_left.y should be {}",
                radius, expected_y,
            );
        }
    }
}