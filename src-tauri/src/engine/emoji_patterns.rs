//! 像素表情图案库 — 16×16 聊天表情小人
//!
//! circle_cells / circle_outline 参数顺序：row, col, radius
//! 返回 Vec<(row, col)>，可直接写入像素画布。

type EmojiCells = Vec<(i32, i32, &'static str)>;

/// 圆形内所有点（row, col, radius）
fn circle_cells(row: f64, col: f64, r: f64) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let r2 = r * r;
    let r0 = r as i32;
    for dy in -r0..=r0 {
        for dx in -r0..=r0 {
            if (dx as f64) * (dx as f64) + (dy as f64) * (dy as f64) <= r2 {
                out.push((row as i32 + dy, col as i32 + dx));
            }
        }
    }
    out
}

/// 圆形轮廓（row, col, radius）
fn circle_outline(row: f64, col: f64, r: f64) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let r2 = r * r;
    let inner_r2 = (r - 1.5).max(0.0) * (r - 1.5).max(0.0);
    let r0 = r as i32;
    for dy in -r0..=r0 {
        for dx in -r0..=r0 {
            let d2 = (dx as f64) * (dx as f64) + (dy as f64) * (dy as f64);
            if d2 <= r2 && d2 > inner_r2 {
                out.push((row as i32 + dy, col as i32 + dx));
            }
        }
    }
    out
}

// ── 颜色 ──
const YELLOW: &str = "#fcc419";
const DARK: &str = "#1a1a1a";
const WHITE: &str = "#ffffff";
const RED: &str = "#e03131";
const BLUE: &str = "#228be6";
const PINK: &str = "#f783ac";

// ── API ──

pub fn get_emoji(name: &str) -> Option<(u32, u32, EmojiCells)> {
    let cells = match name {
        "smile" | "happy" => build_smile(),
        "laugh" | "cry_laugh" => build_laugh(),
        "heart_eyes" | "love" => build_heart_eyes(),
        "angry" | "mad" => build_angry(),
        "cry" | "sad" => build_cry(),
        "cool" | "sunglasses" => build_cool(),
        "shock" | "surprised" | "fear" => build_shock(),
        "wink" | "tongue" => build_wink(),
        _ => return None,
    };
    Some((16, 16, cells))
}

pub fn emoji_names() -> &'static [&'static str] {
    &["smile", "laugh", "heart_eyes", "angry", "cry", "cool", "shock", "wink"]
}

// ── 各表情 — 约定：先画底层（脸），再画上层（眼/嘴），HashMap insert 后写覆盖前写 ──

/// 😊 笑脸
fn build_smile() -> EmojiCells {
    let mut cells = Vec::new();
    // 黄脸
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 眼
    cells.push((6, 5, DARK)); cells.push((6, 6, DARK));
    cells.push((6, 9, DARK)); cells.push((6, 10, DARK));
    // 微笑弧线
    for &(r, c, col) in &[(9,5,DARK),(9,11,DARK),(10,6,DARK),(10,7,DARK),(10,8,DARK),(10,9,DARK),(10,10,DARK)] {
        cells.push((r, c, col));
    }
    cells
}

/// 😂 笑哭
fn build_laugh() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 眯眼
    for c in 4..=7 { cells.push((5, c, DARK)); }
    for c in 9..=12 { cells.push((5, c, DARK)); }
    // 大笑嘴
    for (r, c) in circle_cells(10.5, 7.5, 3.0) { if r >= 9 { cells.push((r, c, DARK)); } }
    for (r, c) in circle_cells(9.5, 7.5, 2.0) { if r >= 8 { cells.push((r, c, RED)); } }
    // 蓝色眼泪
    for &(r, c) in &[(3,3),(4,3),(4,4),(3,12),(4,12),(4,11)] { cells.push((r, c, BLUE)); }
    cells
}

/// 😍 爱心眼
fn build_heart_eyes() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 左❤️眼
    for &(r, c) in &[(5,4),(5,5),(5,6),(5,7),(6,3),(6,4),(6,5),(6,6),(6,7),(6,8),(7,3),(7,4),(7,5),(7,6),(7,7),(7,8),(8,4),(8,5),(8,6),(8,7)] {
        cells.push((r, c, RED));
    }
    // 右❤️眼
    for &(r, c) in &[(5,9),(5,10),(5,11),(5,12),(6,8),(6,9),(6,10),(6,11),(6,12),(6,13),(7,8),(7,9),(7,10),(7,11),(7,12),(7,13),(8,9),(8,10),(8,11),(8,12)] {
        cells.push((r, c, RED));
    }
    // 微笑
    for &(r, c, col) in &[(9,5,DARK),(9,11,DARK),(10,6,DARK),(10,7,DARK),(10,8,DARK),(10,9,DARK),(10,10,DARK)] {
        cells.push((r, c, col));
    }
    // 粉晕（左脸颊 row~4.5 col~3.5 / 右脸颊 row~4.5 col~11.5）
    for (r, c) in circle_cells(4.5, 3.5, 1.8) { cells.push((r, c, PINK)); }
    for (r, c) in circle_cells(4.5, 11.5, 1.8) { cells.push((r, c, PINK)); }
    cells
}

/// 😡 生气
fn build_angry() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, RED)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // V眉
    for &(r, c) in &[(4,4),(3,5),(4,6),(3,7),(3,9),(4,10),(3,11),(4,12)] { cells.push((r, c, DARK)); }
    // 眼白 + 眼珠
    for &(r, c) in &[(6,5),(6,6),(7,5),(7,6),(6,9),(6,10),(7,9),(7,10)] { cells.push((r, c, WHITE)); }
    cells.push((6, 5, DARK)); cells.push((6, 10, DARK));
    // 嘴
    for &(r, c, col) in &[(10,5,DARK),(10,6,DARK),(10,7,DARK),(10,9,DARK),(10,10,DARK),(10,11,DARK),(11,4,DARK),(11,5,DARK),(11,6,DARK),(11,7,DARK),(11,8,DARK)] {
        cells.push((r, c, col));
    }
    cells
}

/// 😢 哭泣
fn build_cry() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 大眼
    for (r, c) in circle_cells(5.5, 5.5, 1.5) { cells.push((r, c, WHITE)); }
    for (r, c) in circle_outline(5.5, 5.5, 1.5) { cells.push((r, c, DARK)); }
    for (r, c) in circle_cells(5.5, 9.5, 1.5) { cells.push((r, c, WHITE)); }
    for (r, c) in circle_outline(5.5, 9.5, 1.5) { cells.push((r, c, DARK)); }
    cells.push((5, 5, DARK)); cells.push((5, 9, DARK)); // 眼珠
    // 嘴
    for &(r, c, col) in &[(10,7,DARK),(10,8,DARK),(10,9,DARK),(11,6,DARK),(11,10,DARK)] { cells.push((r, c, col)); }
    // 眼泪
    for (r, c) in circle_cells(3.0, 2.0, 1.5) { cells.push((r, c, BLUE)); }
    for (r, c) in circle_cells(3.0, 13.0, 1.5) { cells.push((r, c, BLUE)); }
    for &(r, c) in &[(3,3),(3,12),(4,2),(4,13)] { cells.push((r, c, BLUE)); }
    cells
}

/// 😎 酷
fn build_cool() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 墨镜横条
    for r in 4..=7 { for c in 3..=12 { cells.push((r, c, DARK)); } }
    // 反光
    for r in 4..=5 { for c in 4..=6 { cells.push((r, c, WHITE)); } }
    // 微笑
    for &(r, c, col) in &[(9,5,DARK),(9,11,DARK),(10,6,DARK),(10,7,DARK),(10,8,DARK),(10,9,DARK),(10,10,DARK)] {
        cells.push((r, c, col));
    }
    cells
}

/// 😱 惊讶
fn build_shock() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 左大眼
    for (r, c) in circle_cells(5.0, 4.5, 2.5) { cells.push((r, c, WHITE)); }
    for (r, c) in circle_outline(5.0, 4.5, 2.5) { cells.push((r, c, DARK)); }
    // 右大眼
    for (r, c) in circle_cells(5.0, 10.5, 2.5) { cells.push((r, c, WHITE)); }
    for (r, c) in circle_outline(5.0, 10.5, 2.5) { cells.push((r, c, DARK)); }
    // 眼珠
    for (r, c) in circle_cells(5.0, 4.5, 1.0) { cells.push((r, c, DARK)); }
    for (r, c) in circle_cells(5.0, 10.5, 1.0) { cells.push((r, c, DARK)); }
    // O嘴
    for (r, c) in circle_cells(10.5, 7.5, 2.5) { cells.push((r, c, DARK)); }
    for (r, c) in circle_cells(10.5, 7.5, 1.5) { cells.push((r, c, RED)); }
    cells
}

/// 😜 眨眼吐舌
fn build_wink() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 左眼闭
    for c in 4..=7 { cells.push((6, c, DARK)); }
    // 右眼睁
    cells.push((5, 10, DARK)); cells.push((5, 11, DARK));
    // 嘴
    for (r, c) in circle_cells(10.5, 7.5, 3.0) { if r >= 8 && r < 11 { cells.push((r, c, DARK)); } }
    for (r, c) in circle_cells(9.5, 7.5, 2.0) { if r >= 8 { cells.push((r, c, RED)); } }
    // 吐舌
    for c in 7..=9 { cells.push((12, c, RED)); }
    for c in 6..=10 { cells.push((13, c, RED)); }
    cells.push((14, 7, RED)); cells.push((14, 9, RED));
    cells
}
