use super::canvas_state::SubShape;

/// 获取复合图形类型的子组件定义
/// 返回 (Vec<SubShape>, 总宽度, 总高度)
pub fn get_composite_shapes(shape_type: &str) -> Option<(Vec<SubShape>, f64, f64)> {
    match shape_type {
        "house" => Some(house_shapes()),
        "sun" => Some(sun_shapes()),
        "tree" => Some(tree_shapes()),
        "smiley" => Some(smiley_shapes()),
        "star" => Some(star_shapes()),
        _ => None,
    }
}

/// 判断 shape_type 字符串是否为复合图形
pub fn is_composite(shape_type: &str) -> bool {
    matches!(shape_type, "house" | "sun" | "tree" | "smiley" | "star")
}

/// 判断 shape_type 字符串是否为基本几何图形
pub fn is_basic_shape(shape_type: &str) -> bool {
    matches!(shape_type, "circle" | "rectangle" | "triangle" | "line" | "dot")
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

/// 房子 = 三角形屋顶 + 矩形屋身 + 矩形门 + 矩形窗
fn house_shapes() -> (Vec<SubShape>, f64, f64) {
    let shapes = vec![
        // 屋顶（三角形）
        sub(
            "triangle", 0.0, 0.0, 140.0, 60.0, "#ff9800", "#e65100", 2.0, None,
        ),
        // 屋身（矩形）
        sub(
            "rect", 0.0, 60.0, 140.0, 90.0, "#e8f5e9", "#2e7d32", 2.0, None,
        ),
        // 门（矩形）
        sub(
            "rect", 50.0, 100.0, 40.0, 50.0, "#795548", "#4e342e", 1.5, None,
        ),
        // 窗（矩形）
        sub(
            "rect", 10.0, 75.0, 30.0, 30.0, "#bbdefb", "#1565c0", 1.5, None,
        ),
    ];
    (shapes, 140.0, 150.0)
}

/// 太阳 = 圆形 + 8 条放射线
fn sun_shapes() -> (Vec<SubShape>, f64, f64) {
    let cx = 60.0;
    let cy = 60.0;
    let r = 40.0;
    let ray_len = 20.0;
    let mut shapes = vec![sub(
        "circle",
        20.0,
        20.0,
        80.0,
        80.0,
        "#ffc107",
        "#f57f17",
        2.0,
        Some(40.0),
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
        shapes.push(sub(
            "line", lx, ly, lw, lh, "#ffc107", "#f57f17", 3.0, None,
        ));
    }
    (shapes, 120.0, 120.0)
}

/// 树 = 绿色圆形树冠 + 棕色矩形树干
fn tree_shapes() -> (Vec<SubShape>, f64, f64) {
    let shapes = vec![
        // 树冠（圆形）
        sub(
            "circle",
            0.0,
            0.0,
            100.0,
            90.0,
            "#4caf50",
            "#2e7d32",
            2.0,
            Some(45.0),
        ),
        // 树干（矩形）
        sub(
            "rect", 35.0, 80.0, 30.0, 70.0, "#795548", "#4e342e", 2.0, None,
        ),
    ];
    (shapes, 100.0, 150.0)
}

/// 笑脸 = 黄色圆脸 + 两个黑眼 + 弧形嘴巴
fn smiley_shapes() -> (Vec<SubShape>, f64, f64) {
    let shapes = vec![
        // 脸（圆形）
        sub(
            "circle",
            0.0,
            0.0,
            100.0,
            100.0,
            "#ffeb3b",
            "#f9a825",
            2.0,
            Some(50.0),
        ),
        // 左眼（小圆点）
        sub(
            "circle",
            28.0,
            30.0,
            14.0,
            14.0,
            "#333333",
            "#333333",
            0.0,
            Some(7.0),
        ),
        // 右眼（小圆点）
        sub(
            "circle",
            58.0,
            30.0,
            14.0,
            14.0,
            "#333333",
            "#333333",
            0.0,
            Some(7.0),
        ),
        // 嘴巴（弧线 — 前端用弧线路径绘制）
        sub(
            "arc", 32.0, 55.0, 36.0, 20.0, "transparent", "#333333", 2.0, None,
        ),
    ];
    (shapes, 100.0, 100.0)
}

/// 五角星（前端用 star_polygon 渲染）
fn star_shapes() -> (Vec<SubShape>, f64, f64) {
    let shapes = vec![sub(
        "star_polygon",
        0.0,
        0.0,
        120.0,
        120.0,
        "#ffd600",
        "#f57f17",
        2.0,
        None,
    )];
    (shapes, 120.0, 120.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_house_has_four_sub_shapes() {
        let (shapes, w, h) = house_shapes();
        assert_eq!(shapes.len(), 4);
        assert!(w > 0.0 && h > 0.0);
    }

    #[test]
    fn test_sun_has_nine_sub_shapes() {
        let (shapes, _w, _h) = sun_shapes();
        assert_eq!(shapes.len(), 9);
    }

    #[test]
    fn test_tree_has_two_sub_shapes() {
        let (shapes, _w, _h) = tree_shapes();
        assert_eq!(shapes.len(), 2);
    }

    #[test]
    fn test_smiley_has_four_sub_shapes() {
        let (shapes, _w, _h) = smiley_shapes();
        assert_eq!(shapes.len(), 4);
    }

    #[test]
    fn test_star_has_one_polygon() {
        let (shapes, _w, _h) = star_shapes();
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].shape_type, "star_polygon");
    }

    #[test]
    fn test_is_composite() {
        assert!(is_composite("house"));
        assert!(is_composite("star"));
        assert!(!is_composite("circle"));
        assert!(!is_composite("process"));
    }

    #[test]
    fn test_is_basic_shape() {
        assert!(is_basic_shape("circle"));
        assert!(is_basic_shape("triangle"));
        assert!(!is_basic_shape("house"));
        assert!(!is_basic_shape("process"));
    }

    #[test]
    fn test_unknown_returns_none() {
        assert!(get_composite_shapes("unknown").is_none());
        assert!(get_composite_shapes("process").is_none());
    }
}
