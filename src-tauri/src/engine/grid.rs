use super::canvas_state::*;
use std::collections::HashMap;

pub struct GridConfig {
    pub grid_size: f64,
    pub origin_x: f64,
    pub origin_y: f64,
}

impl GridConfig {
    pub fn default() -> Self {
        Self {
            grid_size: 20.0,
            origin_x: 40.0,
            origin_y: 24.0,
        }
    }

    /// 网格坐标 → 像素坐标
    pub fn grid_to_pixel(&self, gx: f64, gy: f64) -> (f64, f64) {
        (
            self.origin_x + gx * self.grid_size,
            self.origin_y + gy * self.grid_size,
        )
    }

    /// 像素坐标 → 网格坐标
    pub fn pixel_to_grid(&self, px: f64, py: f64) -> (f64, f64) {
        (
            (px - self.origin_x) / self.grid_size,
            (py - self.origin_y) / self.grid_size,
        )
    }

    /// 扫描画布，返回第一个空白网格锚点
    /// 以 5 格为步长从左到右、从上到下扫描
    pub fn find_empty_anchor(
        &self,
        nodes: &HashMap<String, DiagramNode>,
    ) -> (f64, f64) {
        let step = 5.0; // 扫描步长
        let max_cols = ((1200.0 - self.origin_x) / (step * self.grid_size)).ceil() as i32;
        let max_rows = ((800.0 - self.origin_y) / (step * self.grid_size)).ceil() as i32;

        for row in 0..max_rows {
            for col in 0..max_cols {
                let gx = col as f64 * step;
                let gy = row as f64 * step;
                let (px, py) = self.grid_to_pixel(gx, gy);

                // 检查是否有节点与此区域重叠
                let margin = 40.0;
                let occupied = nodes.values().any(|node| {
                    let nx = node.position.x;
                    let ny = node.position.y;
                    let nw = node.size.width;
                    let nh = node.size.height;
                    px + margin < nx + nw
                        && px + 60.0 > nx - margin
                        && py + margin < ny + nh
                        && py + 60.0 > ny - margin
                });

                if !occupied {
                    return (gx, gy);
                }
            }
        }
        // 画布满了就返回原点
        (0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_to_pixel_origin() {
        let cfg = GridConfig::default();
        let (px, py) = cfg.grid_to_pixel(0.0, 0.0);
        assert_eq!(px, 40.0);
        assert_eq!(py, 24.0);
    }

    #[test]
    fn test_grid_to_pixel_positive() {
        let cfg = GridConfig::default();
        let (px, py) = cfg.grid_to_pixel(3.0, 2.0);
        assert_eq!(px, 100.0); // 40 + 3*20
        assert_eq!(py, 64.0); // 24 + 2*20
    }

    #[test]
    fn test_pixel_to_grid() {
        let cfg = GridConfig::default();
        let (gx, gy) = cfg.pixel_to_grid(100.0, 64.0);
        assert_eq!(gx, 3.0);
        assert_eq!(gy, 2.0);
    }

    #[test]
    fn test_find_empty_anchor_when_canvas_empty() {
        let cfg = GridConfig::default();
        let nodes = HashMap::new();
        let (gx, gy) = cfg.find_empty_anchor(&nodes);
        assert_eq!(gx, 0.0);
        assert_eq!(gy, 0.0);
    }

    #[test]
    fn test_find_empty_anchor_avoids_occupied() {
        let cfg = GridConfig::default();
        let mut nodes = HashMap::new();
        // Place a node at grid (0,0), spanning ~8x3 grid units
        nodes.insert(
            "n1".into(),
            DiagramNode {
                id: "n1".into(),
                node_type: NodeType::Process,
                shape_type: None,
                label: "占位".into(),
                position: Position { x: 40.0, y: 24.0 },
                size: Size {
                    width: 160.0,
                    height: 60.0,
                },
                style: NodeStyle::default(),
                sub_shapes: None,
            },
        );
        let (gx, gy) = cfg.find_empty_anchor(&nodes);
        // First row is occupied at (0,0), so next empty should be further right or below
        // 160px width = 8 grid units, so column >= 9 or row >= 3
        assert!(
            gx >= 8.0 || gy >= 3.0,
            "Expected empty anchor away from occupied node, got ({}, {})",
            gx,
            gy
        );
    }
}
