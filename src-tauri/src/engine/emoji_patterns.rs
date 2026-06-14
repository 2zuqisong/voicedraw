//! 像素表情图案库 — 16×16 聊天表情小人
//!
//! 每个表情返回 Vec<(行, 列, hex颜色)>，可直接合并到像素画布。
//! 坐标系：左上角 (0,0)，右下角 (15,15)。

/// 生成圆形内所有点（填充用）
fn circle_cells(cx: f64, cy: f64, r: f64) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let r2 = r * r;
    let r0 = r as i32;
    for dy in -r0..=r0 {
        for dx in -r0..=r0 {
            if (dx as f64) * (dx as f64) + (dy as f64) * (dy as f64) <= r2 {
                out.push((cy as i32 + dy, cx as i32 + dx));
            }
        }
    }
    out
}

/// 圆形轮廓（只有边界）
fn circle_outline(cx: f64, cy: f64, r: f64) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let r2 = r * r;
    let inner_r2 = (r - 1.5).max(0.0) * (r - 1.5).max(0.0);
    let r0 = r as i32;
    for dy in -r0..=r0 {
        for dx in -r0..=r0 {
            let d2 = (dx as f64) * (dx as f64) + (dy as f64) * (dy as f64);
            if d2 <= r2 && d2 > inner_r2 {
                out.push((cy as i32 + dy, cx as i32 + dx));
            }
        }
    }
    out
}

/// 生成一个表情图案：返回 (row, col, color) 列表
type EmojiCells = Vec<(i32, i32, &'static str)>;

pub fn get_emoji(name: &str) -> Option<(u32, u32, EmojiCells)> {
    let cells: EmojiCells = match name {
        // 😊 笑脸 — 经典黄色笑脸
        "smile" | "happy" => build_smile(),
        // 😂 笑哭 — 带眼泪的笑
        "laugh" | "cry_laugh" => build_laugh(),
        // 😍 爱心眼 — 心形眼睛
        "heart_eyes" | "love" => build_heart_eyes(),
        // 😡 生气 — 红色愤怒脸
        "angry" | "mad" => build_angry(),
        // 😢 哭泣 — 大滴眼泪
        "cry" | "sad" => build_cry(),
        // 😎 酷 — 墨镜
        "cool" | "sunglasses" => build_cool(),
        // 😱 惊讶 — 张大嘴
        "shock" | "surprised" | "fear" => build_shock(),
        // 😜 眨眼 — 单眼闭 + 吐舌
        "wink" | "tongue" => build_wink(),
        _ => return None,
    };
    Some((16, 16, cells))
}

/// 所有支持的 emoji 名称列表
pub fn emoji_names() -> &'static [&'static str] {
    &["smile", "laugh", "heart_eyes", "angry", "cry", "cool", "shock", "wink"]
}

// ── 具体表情定义 ────────────────────────────────────────────────────

const YELLOW: &str = "#fcc419";
const DARK: &str = "#1a1a1a";
const WHITE: &str = "#ffffff";
const RED: &str = "#e03131";
const BLUE: &str = "#228be6";
const PINK: &str = "#f783ac";

/// 笑脸 😊
fn build_smile() -> EmojiCells {
    let mut cells = Vec::new();
    // 黄色圆脸（半径 7.5，中心 7.5,7.5）
    for (r, c) in circle_cells(7.5, 7.5, 7.5) {
        cells.push((r, c, YELLOW));
    }
    // 深色轮廓
    for (r, c) in circle_outline(7.5, 7.5, 7.5) {
        cells.push((r, c, DARK));
    }
    // 左眼 — 小圆点
    cells.push((6, 5, DARK));
    cells.push((6, 6, DARK));
    // 右眼 — 小圆点
    cells.push((6, 9, DARK));
    cells.push((6, 10, DARK));
    // 微笑弧线嘴（一行一行画）
    for c in 6..=10 { cells.push((10, c, DARK)); }
    for c in 5..=11 { cells.push((9, c, YELLOW)); cells.push((10, c, DARK)); } // 会被覆盖，重做
    // 重置：弧线嘴
    for &(r, c, col) in &[
        (9,5,DARK),(9,11,DARK),
        (10,6,DARK),(10,7,DARK),(10,8,DARK),(10,9,DARK),(10,10,DARK),
    ] {
        cells.push((r, c, col));
    }
    cells
}

/// 笑哭 😂
fn build_laugh() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 眯眼（弧线笑眼）
    for c in 4..=7 { cells.push((5, c, DARK)); }
    for c in 9..=12 { cells.push((5, c, DARK)); }
    // 大笑嘴（半椭圆）
    for (r, c) in circle_cells(7.5, 10.5, 3.0) {
        if r >= 9 { cells.push((r, c, DARK)); }
    }
    for (r, c) in circle_cells(7.5, 9.5, 2.0) {
        if r >= 8 { cells.push((r, c, RED)); } // 红色舌头/口腔
    }
    // 蓝色眼泪（从眼睛流出）
    cells.push((3, 3, BLUE));
    cells.push((4, 3, BLUE));
    cells.push((4, 4, BLUE));
    cells.push((3, 12, BLUE));
    cells.push((4, 12, BLUE));
    cells.push((4, 11, BLUE));
    cells
}

/// 爱心眼 😍
fn build_heart_eyes() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 左爱心眼（小❤️）
    for &(r, c) in &[(5,4),(5,5),(5,6),(5,7),
               (6,3),(6,4),(6,5),(6,6),(6,7),(6,8),
               (7,3),(7,4),(7,5),(7,6),(7,7),(7,8),
               (8,4),(8,5),(8,6),(8,7)] {
        cells.push((r, c, RED));
    }
    // 右爱心眼（对称）
    for &(r, c) in &[(5,9),(5,10),(5,11),(5,12),
               (6,8),(6,9),(6,10),(6,11),(6,12),(6,13),
               (7,8),(7,9),(7,10),(7,11),(7,12),(7,13),
               (8,9),(8,10),(8,11),(8,12)] {
        cells.push((r, c, RED));
    }
    // 微笑
    for &(r, c, col) in &[(10,6,DARK),(10,7,DARK),(10,8,DARK),(10,9,DARK),(10,10,DARK),(9,5,DARK),(9,11,DARK)] {
        cells.push((r, c, col));
    }
    // 红晕
    for (r, c) in circle_cells(4.5, 3.5, 1.8) {
        cells.push((r, c, PINK));
    }
    for (r, c) in circle_cells(4.5, 11.5, 1.8) {
        cells.push((r, c, PINK));
    }
    cells
}

/// 生气 😡
fn build_angry() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, RED)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // V 形眉毛（左）
    for p in &[(4,4),(3,5),(4,6),(3,7)] { cells.push((p.0, p.1, DARK)); }
    // V 形眉毛（右）
    for p in &[(3,9),(4,10),(3,11),(4,12)] { cells.push((p.0, p.1, DARK)); }
    // 小圆眼
    cells.push((6, 5, WHITE)); cells.push((6, 6, WHITE)); cells.push((7, 5, WHITE)); cells.push((7, 6, WHITE));
    cells.push((6, 9, WHITE)); cells.push((6, 10, WHITE)); cells.push((7, 9, WHITE)); cells.push((7, 10, WHITE));
    // 眼珠
    cells.push((6, 5, DARK)); cells.push((6, 10, DARK));
    // 皱眉嘴
    for &(r, c, col) in &[(10,5,DARK),(10,6,DARK),(10,7,DARK),(11,4,DARK),(11,5,DARK),(11,6,DARK),(11,7,DARK),(11,8,DARK),(10,9,DARK),(10,10,DARK),(10,11,DARK)] {
        cells.push((r, c, col));
    }
    cells
}

/// 哭泣 😢
fn build_cry() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 大圆眼
    for (r, c) in circle_cells(5.5, 5.5, 1.5) { cells.push((r, c, WHITE)); }
    for (r, c) in circle_outline(5.5, 5.5, 1.5) { cells.push((r, c, DARK)); }
    for (r, c) in circle_cells(5.5, 9.5, 1.5) { cells.push((r, c, WHITE)); }
    for (r, c) in circle_outline(5.5, 9.5, 1.5) { cells.push((r, c, DARK)); }
    // 眼珠
    cells.push((5, 5, DARK)); cells.push((5, 9, DARK));
    // 伤心嘴
    for &(r, c, col) in &[(11,6,DARK),(10,7,DARK),(10,8,DARK),(10,9,DARK),(11,10,DARK)] {
        cells.push((r, c, col));
    }
    // 蓝色大眼泪
    for (r, c) in circle_cells(2.0, 3.0, 1.5) { cells.push((r, c, BLUE)); }
    for (r, c) in circle_cells(2.0, 12.0, 1.5) { cells.push((r, c, BLUE)); }
    cells.push((3, 3, BLUE)); cells.push((3, 12, BLUE));
    cells.push((4, 2, BLUE)); cells.push((4, 13, BLUE));
    cells
}

/// 酷 😎
fn build_cool() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 黑色墨镜（矩形横条）
    for r in 4..=7 {
        for c in 3..=12 { cells.push((r, c, DARK)); }
    }
    // 墨镜反光
    for r in 4..=5 { for c in 4..=6 { cells.push((r, c, WHITE)); }}
    // 微笑
    for &(r, c, col) in &[(10,6,DARK),(10,7,DARK),(10,8,DARK),(10,9,DARK),(10,10,DARK),(9,5,DARK),(9,11,DARK)] {
        cells.push((r, c, col));
    }
    cells
}

/// 惊讶 😱
fn build_shock() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 超大眼睛
    for (r, c) in circle_cells(5.0, 4.5, 2.5) { cells.push((r, c, WHITE)); }
    for (r, c) in circle_outline(5.0, 4.5, 2.5) { cells.push((r, c, DARK)); }
    for (r, c) in circle_cells(5.0, 10.5, 2.5) { cells.push((r, c, WHITE)); }
    for (r, c) in circle_outline(5.0, 10.5, 2.5) { cells.push((r, c, DARK)); }
    // 眼珠
    for (r, c) in circle_cells(5.0, 4.5, 1.0) { cells.push((r, c, DARK)); }
    for (r, c) in circle_cells(5.0, 10.5, 1.0) { cells.push((r, c, DARK)); }
    // O 形嘴
    for (r, c) in circle_cells(10.5, 7.5, 2.5) { cells.push((r, c, DARK)); }
    for (r, c) in circle_cells(10.5, 7.5, 1.5) { cells.push((r, c, RED)); }
    cells
}

/// 眨眼 😜
fn build_wink() -> EmojiCells {
    let mut cells = Vec::new();
    for (r, c) in circle_cells(7.5, 7.5, 7.5) { cells.push((r, c, YELLOW)); }
    for (r, c) in circle_outline(7.5, 7.5, 7.5) { cells.push((r, c, DARK)); }
    // 左眼闭（一条横线）
    for c in 4..=7 { cells.push((6, c, DARK)); }
    // 右眼正常（小圆点 + 稍微向上）
    cells.push((5, 10, DARK)); cells.push((5, 11, DARK));
    // 吐舌嘴
    for (r, c) in circle_cells(7.5, 10.5, 3.0) {
        if r >= 8 && r < 11 { cells.push((r, c, DARK)); }
    }
    for (r, c) in circle_cells(7.5, 9.5, 2.0) {
        if r >= 8 { cells.push((r, c, RED)); }
    }
    // 舌头伸出
    for c in 7..=9 { cells.push((12, c, RED)); }
    for c in 6..=10 { cells.push((13, c, RED)); }
    cells.push((14, 7, RED)); cells.push((14, 9, RED));
    cells
}
