use super::canvas_state::SubShape;

/// 从主色推导配色方案
struct ColorScheme {
    primary: String,
    dark: String,
    light: String,
    accent: String,
    neutral: String,
}

/// 解析 #rrggbb → (r, g, b)
fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let s = hex.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some((r, g, b))
}

fn to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn darken(hex: &str, factor: f64) -> String {
    if let Some((r, g, b)) = parse_hex(hex) {
        to_hex(
            (r as f64 * factor).round() as u8,
            (g as f64 * factor).round() as u8,
            (b as f64 * factor).round() as u8,
        )
    } else {
        hex.to_string()
    }
}

fn lighten(hex: &str, factor: f64) -> String {
    if let Some((r, g, b)) = parse_hex(hex) {
        to_hex(
            (r as f64 + (255.0 - r as f64) * factor).round() as u8,
            (g as f64 + (255.0 - g as f64) * factor).round() as u8,
            (b as f64 + (255.0 - b as f64) * factor).round() as u8,
        )
    } else {
        hex.to_string()
    }
}

fn derive_color_scheme(fill: &str) -> ColorScheme {
    let primary = fill.to_string();
    let dark = darken(fill, 0.6);
    let light = lighten(fill, 0.5);
    // accent: shift towards complementary by swapping channels
    let accent = if let Some((r, g, b)) = parse_hex(fill) {
        to_hex(g, b, r)
    } else {
        fill.to_string()
    };
    let neutral = if let Some((r, g, b)) = parse_hex(fill) {
        let avg = ((r as u16 + g as u16 + b as u16) / 3) as u8;
        to_hex(avg, avg, avg)
    } else {
        "#9e9e9e".to_string()
    };
    ColorScheme {
        primary,
        dark,
        light,
        accent,
        neutral,
    }
}

/// 获取复合图形类型的子组件定义
/// fill: 可选的主色，用于推导子组件配色；None 时使用默认配色
pub fn get_composite_shapes(
    shape_type: &str,
    fill: Option<&str>,
) -> Option<(Vec<SubShape>, f64, f64)> {
    match shape_type {
        "house" => Some(house_shapes(fill)),
        "sun" => Some(sun_shapes(fill)),
        "tree" => Some(tree_shapes(fill)),
        "smiley" => Some(smiley_shapes(fill)),
        "star" => Some(star_shapes(fill)),
        "cake" => Some(cake_shapes(fill)),
        "gift" => Some(gift_shapes(fill)),
        "balloon" => Some(balloon_shapes(fill)),
        "candle" => Some(candle_shapes(fill)),
        "heart" => Some(heart_shapes(fill)),
        "flower" => Some(flower_shapes(fill)),
        "arrow_shape" => Some(arrow_shape_shapes(fill)),
        "speech_bubble" => Some(speech_bubble_shapes(fill)),
        "cloud" => Some(cloud_shapes(fill)),
        "lightning" => Some(lightning_shapes(fill)),
        _ => None,
    }
}

/// 判断 shape_type 字符串是否为复合图形
pub fn is_composite(shape_type: &str) -> bool {
    matches!(
        shape_type,
        "house"
            | "sun"
            | "tree"
            | "smiley"
            | "star"
            | "cake"
            | "gift"
            | "balloon"
            | "candle"
            | "heart"
            | "flower"
            | "arrow_shape"
            | "speech_bubble"
            | "cloud"
            | "lightning"
    )
}

/// 判断 shape_type 字符串是否为基本几何图形
pub fn is_basic_shape(shape_type: &str) -> bool {
    matches!(
        shape_type,
        "circle" | "rectangle" | "triangle" | "line" | "dot"
    )
}

fn sub(
    shape_type: &str,
    rel_x: f64,
    rel_y: f64,
    width: f64,
    height: f64,
    fill: &str,
    stroke: &str,
    stroke_width: f64,
    radius: Option<f64>,
) -> SubShape {
    SubShape {
        shape_type: shape_type.to_string(),
        rel_x,
        rel_y,
        width,
        height,
        fill: fill.to_string(),
        stroke: stroke.to_string(),
        stroke_width,
        radius,
    }
}

// ══════════════════════════════════════════════════════════════
// Existing 5 composite shapes (updated with color scheme support)
// ══════════════════════════════════════════════════════════════

/// 房子 = 三角形屋顶 + 矩形屋身 + 矩形门 + 矩形窗
fn house_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let roof_fill = cs.as_ref().map(|c| c.accent.as_str()).unwrap_or("#ff9800");
    let roof_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#e65100");
    let body_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#e8f5e9");
    let body_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#2e7d32");
    let door_fill: String = cs
        .as_ref()
        .map(|c| c.neutral.clone())
        .unwrap_or_else(|| "#795548".into());
    let door_stroke: String = cs
        .as_ref()
        .map(|c| darken(&c.neutral, 0.7))
        .unwrap_or_else(|| "#4e342e".into());
    let win_fill = cs.as_ref().map(|c| c.light.as_str()).unwrap_or("#bbdefb");
    let win_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#1565c0");

    let shapes = vec![
        sub("triangle", 0.0, 0.0, 140.0, 60.0, roof_fill, roof_stroke, 2.0, None),
        sub("rect", 0.0, 60.0, 140.0, 90.0, body_fill, body_stroke, 2.0, None),
        sub("rect", 50.0, 100.0, 40.0, 50.0, &door_fill, &door_stroke, 1.5, None),
        sub("rect", 10.0, 75.0, 30.0, 30.0, win_fill, win_stroke, 1.5, None),
    ];
    (shapes, 140.0, 150.0)
}

/// 太阳 = 圆形 + 8 条放射线
fn sun_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let body_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#ffc107");
    let body_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#f57f17");
    let ray_fill = body_fill;
    let ray_stroke = body_stroke;

    let cx = 60.0;
    let cy = 60.0;
    let r = 40.0;
    let ray_len = 20.0;
    let mut shapes = vec![sub(
        "circle", 20.0, 20.0, 80.0, 80.0, body_fill, body_stroke, 2.0, Some(40.0),
    )];
    for i in 0..8 {
        let angle = (i as f64) * std::f64::consts::PI / 4.0;
        let x1 = cx + r * angle.cos();
        let y1 = cy + r * angle.sin();
        let x2 = cx + (r + ray_len) * angle.cos();
        let y2 = cy + (r + ray_len) * angle.sin();
        let lx = x1.min(x2);
        let ly = y1.min(y2);
        let lw = (x2 - x1).abs().max(1.0);
        let lh = (y2 - y1).abs().max(1.0);
        shapes.push(sub("line", lx, ly, lw, lh, ray_fill, ray_stroke, 3.0, None));
    }
    (shapes, 120.0, 120.0)
}

/// 树 = 绿色圆形树冠 + 棕色矩形树干
fn tree_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let crown_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#4caf50");
    let crown_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#2e7d32");
    let trunk_fill: String = cs
        .as_ref()
        .map(|c| c.neutral.clone())
        .unwrap_or_else(|| "#795548".into());
    let trunk_stroke: String = cs
        .as_ref()
        .map(|c| darken(&c.neutral, 0.7))
        .unwrap_or_else(|| "#4e342e".into());

    let shapes = vec![
        sub("circle", 0.0, 0.0, 100.0, 90.0, crown_fill, crown_stroke, 2.0, Some(45.0)),
        sub("rect", 35.0, 80.0, 30.0, 70.0, &trunk_fill, &trunk_stroke, 2.0, None),
    ];
    (shapes, 100.0, 150.0)
}

/// 笑脸 = 黄色圆脸 + 两个黑眼 + 弧形嘴巴
fn smiley_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let face_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#ffeb3b");
    let face_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#f9a825");
    let eye_fill = cs.as_ref().map(|_| "#333333").unwrap_or("#333333");

    let shapes = vec![
        sub("circle", 0.0, 0.0, 100.0, 100.0, face_fill, face_stroke, 2.0, Some(50.0)),
        sub("circle", 28.0, 30.0, 14.0, 14.0, eye_fill, eye_fill, 0.0, Some(7.0)),
        sub("circle", 58.0, 30.0, 14.0, 14.0, eye_fill, eye_fill, 0.0, Some(7.0)),
        sub("arc", 32.0, 55.0, 36.0, 20.0, "transparent", "#333333", 2.0, None),
    ];
    (shapes, 100.0, 100.0)
}

/// 五角星
fn star_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let s_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#ffd600");
    let s_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#f57f17");

    let shapes = vec![sub(
        "star_polygon", 0.0, 0.0, 120.0, 120.0, s_fill, s_stroke, 2.0, None,
    )];
    (shapes, 120.0, 120.0)
}

// ══════════════════════════════════════════════════════════════
// 10 new composite shapes
// ══════════════════════════════════════════════════════════════

/// 蛋糕 = 矩形底座 + 矩形上层 + 3 根蜡烛 + 3 个火焰
fn cake_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let body_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#ffab91");
    let body_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#bf360c");
    let layer_fill = cs.as_ref().map(|c| c.light.as_str()).unwrap_or("#ffccbc");
    let candle_fill = cs.as_ref().map(|c| c.accent.as_str()).unwrap_or("#f48fb1");
    let flame_fill = "#ffeb3b";
    let flame_stroke = "#f9a825";

    let mut shapes = vec![
        // 底座
        sub("rect", 15.0, 60.0, 120.0, 80.0, body_fill, body_stroke, 2.0, None),
        // 上层
        sub("rect", 30.0, 20.0, 90.0, 50.0, layer_fill, body_stroke, 2.0, None),
    ];
    // 3 根蜡烛 + 火焰
    for i in 0..3 {
        let cx = 35.0 + i as f64 * 40.0;
        shapes.push(sub(
            "rect", cx, 0.0, 10.0, 28.0, candle_fill,
            &darken(candle_fill, 0.7), 1.0, None,
        ));
        shapes.push(sub(
            "circle", cx - 2.0, -8.0, 14.0, 14.0, flame_fill, flame_stroke, 1.0, Some(6.0),
        ));
    }
    (shapes, 150.0, 140.0)
}

/// 礼物盒 = 矩形盒身 + 矩形盒盖 + 竖丝带 + 横丝带
fn gift_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let box_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#e53935");
    let box_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#b71c1c");
    let lid_fill = cs.as_ref().map(|c| c.accent.as_str()).unwrap_or("#ef5350");
    let ribbon_fill = cs.as_ref().map(|c| c.light.as_str()).unwrap_or("#ffeb3b");

    let shapes = vec![
        // 盒身
        sub("rect", 10.0, 40.0, 100.0, 90.0, box_fill, box_stroke, 2.0, None),
        // 盒盖
        sub("rect", 5.0, 28.0, 110.0, 20.0, lid_fill, box_stroke, 2.0, None),
        // 竖丝带
        sub("rect", 55.0, 28.0, 10.0, 102.0, ribbon_fill, &darken(ribbon_fill, 0.7), 1.0, None),
        // 横丝带
        sub("rect", 10.0, 70.0, 100.0, 8.0, ribbon_fill, &darken(ribbon_fill, 0.7), 1.0, None),
    ];
    (shapes, 120.0, 130.0)
}

/// 气球 = 椭圆球体 + 三角形绳结 + 线条绳子
fn balloon_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let body_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#42a5f5");
    let body_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#1565c0");
    let string_color = cs.as_ref().map(|c| c.neutral.as_str()).unwrap_or("#757575");

    let shapes = vec![
        // 球体 (ellipse approximated as circle)
        sub("circle", 5.0, 0.0, 70.0, 85.0, body_fill, body_stroke, 2.0, Some(35.0)),
        // 绳结 (小三角)
        sub("triangle", 32.0, 78.0, 16.0, 12.0, body_stroke, body_stroke, 1.0, None),
        // 绳子
        sub("line", 40.0, 90.0, 0.0, 30.0, string_color, string_color, 1.5, None),
    ];
    (shapes, 80.0, 120.0)
}

/// 蜡烛 = 矩形蜡烛身 + 椭圆火焰
fn candle_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let body_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#fff9c4");
    let body_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#f9a825");
    let flame_fill = "#ff9800";
    let flame_stroke = "#e65100";

    let shapes = vec![
        sub("rect", 8.0, 30.0, 24.0, 70.0, body_fill, body_stroke, 1.5, None),
        sub("circle", 12.0, 8.0, 16.0, 28.0, flame_fill, flame_stroke, 1.0, Some(10.0)),
    ];
    (shapes, 40.0, 100.0)
}

/// 爱心 = 2 个圆形（上瓣）+ 1 个三角形（下尖）
fn heart_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let h_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#e53935");
    let h_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#b71c1c");

    let shapes = vec![
        // 左瓣
        sub("circle", 0.0, 0.0, 60.0, 60.0, h_fill, h_stroke, 2.0, Some(30.0)),
        // 右瓣
        sub("circle", 60.0, 0.0, 60.0, 60.0, h_fill, h_stroke, 2.0, Some(30.0)),
        // 下尖 (triangle pointing down)
        sub("triangle", 0.0, 42.0, 120.0, 68.0, h_fill, h_stroke, 2.0, None),
    ];
    (shapes, 120.0, 110.0)
}

/// 花朵 = 5 个椭圆花瓣 + 圆形花心
fn flower_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let petal_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#f48fb1");
    let petal_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#ad1457");
    let center_fill = cs.as_ref().map(|c| c.accent.as_str()).unwrap_or("#ffeb3b");
    let center_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#f9a825");

    let mut shapes = Vec::new();
    let cx = 60.0;
    let cy = 60.0;
    for i in 0..5 {
        let angle = (i as f64) * std::f64::consts::TAU / 5.0 - std::f64::consts::FRAC_PI_2;
        let px = cx + 24.0 * angle.cos() - 16.0;
        let py = cy + 24.0 * angle.sin() - 22.0;
        shapes.push(sub(
            "circle", px, py, 32.0, 44.0, petal_fill, petal_stroke, 1.5, Some(18.0),
        ));
    }
    // 花心
    shapes.push(sub(
        "circle", 35.0, 35.0, 50.0, 50.0, center_fill, center_stroke, 2.0, Some(25.0),
    ));
    (shapes, 120.0, 120.0)
}

/// 大箭头 = 矩形箭身 + 三角形箭头
fn arrow_shape_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let a_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#42a5f5");
    let a_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#1565c0");

    let shapes = vec![
        // 箭身
        sub("rect", 0.0, 12.0, 100.0, 16.0, a_fill, a_stroke, 2.0, None),
        // 箭头
        sub("triangle", 90.0, 0.0, 50.0, 40.0, a_fill, a_stroke, 2.0, None),
    ];
    (shapes, 140.0, 40.0)
}

/// 对话气泡 = 圆角矩形 + 三角形指向标
fn speech_bubble_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let b_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#ffffff");
    let b_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#333333");

    let shapes = vec![
        // 主体圆角矩形
        sub("rect", 0.0, 0.0, 160.0, 80.0, b_fill, b_stroke, 2.0, None),
        // 指向三角形
        sub("triangle", 20.0, 75.0, 24.0, 25.0, b_fill, b_stroke, 2.0, None),
    ];
    (shapes, 160.0, 100.0)
}

/// 云朵 = 5 个圆形叠加
fn cloud_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let c_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#eceff1");
    let c_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#90a4ae");

    let shapes = vec![
        sub("circle", 10.0, 30.0, 50.0, 50.0, c_fill, c_stroke, 1.5, Some(25.0)),
        sub("circle", 40.0, 15.0, 60.0, 60.0, c_fill, c_stroke, 1.5, Some(30.0)),
        sub("circle", 80.0, 25.0, 50.0, 50.0, c_fill, c_stroke, 1.5, Some(25.0)),
        sub("circle", 110.0, 35.0, 40.0, 40.0, c_fill, c_stroke, 1.5, Some(20.0)),
        sub("circle", 30.0, 40.0, 50.0, 50.0, c_fill, c_stroke, 1.5, Some(25.0)),
    ];
    (shapes, 160.0, 90.0)
}

/// 闪电 = 锯齿多边形（用多个三角形拼接）
fn lightning_shapes(fill: Option<&str>) -> (Vec<SubShape>, f64, f64) {
    let cs = fill.map(derive_color_scheme);
    let l_fill = cs.as_ref().map(|c| c.primary.as_str()).unwrap_or("#ffd600");
    let l_stroke = cs.as_ref().map(|c| c.dark.as_str()).unwrap_or("#f57f17");

    // 闪电用一个大三角形近似
    let shapes = vec![
        sub("triangle", 10.0, 0.0, 20.0, 60.0, l_fill, l_stroke, 2.0, None),
        sub("triangle", 10.0, 45.0, 20.0, 55.0, l_fill, l_stroke, 2.0, None),
    ];
    (shapes, 40.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_house_has_four_sub_shapes() {
        let (shapes, w, h) = house_shapes(None);
        assert_eq!(shapes.len(), 4);
        assert!(w > 0.0 && h > 0.0);
    }

    #[test]
    fn test_sun_has_nine_sub_shapes() {
        let (shapes, _w, _h) = sun_shapes(None);
        assert_eq!(shapes.len(), 9);
    }

    #[test]
    fn test_tree_has_two_sub_shapes() {
        let (shapes, _w, _h) = tree_shapes(None);
        assert_eq!(shapes.len(), 2);
    }

    #[test]
    fn test_smiley_has_four_sub_shapes() {
        let (shapes, _w, _h) = smiley_shapes(None);
        assert_eq!(shapes.len(), 4);
    }

    #[test]
    fn test_star_has_one_polygon() {
        let (shapes, _w, _h) = star_shapes(None);
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].shape_type, "star_polygon");
    }

    #[test]
    fn test_new_shapes_present() {
        assert!(get_composite_shapes("cake", None).is_some());
        assert!(get_composite_shapes("gift", None).is_some());
        assert!(get_composite_shapes("balloon", None).is_some());
        assert!(get_composite_shapes("candle", None).is_some());
        assert!(get_composite_shapes("heart", None).is_some());
        assert!(get_composite_shapes("flower", None).is_some());
        assert!(get_composite_shapes("arrow_shape", None).is_some());
        assert!(get_composite_shapes("speech_bubble", None).is_some());
        assert!(get_composite_shapes("cloud", None).is_some());
        assert!(get_composite_shapes("lightning", None).is_some());
    }

    #[test]
    fn test_is_composite_includes_all() {
        assert!(is_composite("house"));
        assert!(is_composite("star"));
        assert!(is_composite("cake"));
        assert!(is_composite("heart"));
        assert!(is_composite("cloud"));
        assert!(!is_composite("circle"));
        assert!(!is_composite("process"));
    }

    #[test]
    fn test_is_basic_shape() {
        assert!(is_basic_shape("circle"));
        assert!(is_basic_shape("triangle"));
        assert!(!is_basic_shape("house"));
        assert!(!is_basic_shape("cake"));
    }

    #[test]
    fn test_unknown_returns_none() {
        assert!(get_composite_shapes("unknown", None).is_none());
        assert!(get_composite_shapes("process", None).is_none());
    }

    #[test]
    fn test_color_scheme_with_fill() {
        let (shapes, _, _) = house_shapes(Some("#e53935"));
        // With red fill, the body should be red-ish (primary)
        let body = &shapes[1];
        assert_eq!(body.shape_type, "rect");
        assert!(body.fill.contains("e5")); // primary #e53935
    }

    #[test]
    fn test_color_scheme_without_fill_uses_defaults() {
        let (shapes, _, _) = house_shapes(None);
        let body = &shapes[1];
        assert_eq!(body.fill, "#e8f5e9"); // default green body
    }
}
