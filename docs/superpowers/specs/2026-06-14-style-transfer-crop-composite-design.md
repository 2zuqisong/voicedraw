# Style Transfer Crop & Composite Fix

> **Status:** Design approved → ready for implementation plan
> **Date:** 2026-06-14

## Problem

When user says "把房子变成梵高风格", the entire canvas (including background) is sent to DashScope API. The diffusion model sees a tiny house (<1% of canvas) against a large background, so it stylizes the entire image instead of just the house.

## Root Cause

```text
❌ Current: canvas.toDataURL() → full canvas PNG → DashScope → full stylized image
✅ Desired:  crop house region → small PNG → DashScope → paste back at original position
```

## Solution: Crop & Composite

Two distinct modes based on `node_ids`:

| Mode | `node_ids` | Input to API | Output handling |
|------|-----------|-------------|----------------|
| Canvas | `[]` | Full canvas PNG | Replace all objects |
| Node | `["house_1"]` | Cropped 200×150 PNG (no bg) | Paste back at bounding box |

### Data Flow (Node Mode)

```
canvas.toDataURL({ multiplier: 2 })
         │
         ▼
cropCanvasRegion(bounds) → 200×150 PNG (house only)
         │
         ▼
invoke('apply_style_transfer', { image_base64_small, prompt })
         │
         ▼
DashScope API stylization_all
         │
         ▼
fabric.FabricImage.fromURL(result)
         │
         ▼
Paste at bounds.left, bounds.top (scale to fit)
Original objects removed. Background & other objects untouched.
```

## File Changes

### 1. CanvasView.tsx — crop function + dispatch

- New: `cropCanvasRegion(canvas, bounds) → Promise<string>` — extracts bounded region from canvas
- Modified: Style transfer effect — choose mode based on `node_ids.length`
  - Empty → full canvas (existing behavior)
  - Non-empty → `cropCanvasRegion()` then send cropped image

### 2. system_prompt.md — scope constraint prompts

Node mode prompt suffix:
```
仅对选中对象进行风格转换。禁止修改未选中区域。禁止改变画布背景。禁止新增任何元素。严格保持对象轮廓和比例不变。
```

### 3. CanvasView.tsx — multiplier bump

- Change `multiplier: 1` → `multiplier: 2` for better API quality

## Not Changing

- Rust `style_transfer.rs` — unchanged (receives base64, returns base64)
- `tool_defs.rs` — `apply_style` tool unchanged
- `scheduler.rs` — unchanged
- `store/index.ts` — unchanged
- `types.ts` — unchanged

## Implementation Order

1. Add `cropCanvasRegion()` to CanvasView.tsx
2. Modify style transfer effect: node mode uses cropped image
3. Update system_prompt.md with scope constraint prompts
4. Bump multiplier to 2
5. Verify: tsc + cargo check + vite build
6. Commit
