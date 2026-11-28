use bevy::prelude::*;

use std::collections::VecDeque;

use crate::utils::Rng;


// タイルの大きさ
const TILE: f32 = 4.0;
// 開始タイルから、この距離以内にはトラップも女性も置かない
const START_SAFE_DISTANCE: usize = 2;
// 女性をマス内に置くとき、マスの端から空ける余白
const WOMAN_MARGIN: f32 = 0.6;


// true が壁
// type Maze = [[bool; TILE_COLS]; TILE_ROWS];

/// 周囲のマスのリスト
fn get_sorround(
    x: usize,
    y: usize,
    h: usize,
    w: usize,
) -> Vec<(usize, usize)> {
    let mut ret = Vec::new();

    if 0 < x {
        ret.push((x - 1, y));
    }
    if x + 1 < w {
        ret.push((x + 1, y));
    }
    if 0 < y {
        ret.push((x, y - 1));
    }
    if y + 1 < h {
        ret.push((x, y + 1));
    }

    ret
}

/// セル (cx, cy) から開いている辺の数
fn open_edges(walls: &Vec<Vec<bool>>, cx: usize, cy: usize) -> usize {
    let (r, c) = (2 * cy + 1, 2 * cx + 1);
    [(r - 1, c), (r + 1, c), (r, c - 1), (r, c + 1)]
        .iter()
        .filter(|&&(rr, cc)| !walls[rr][cc])
        .count()
}

/// 行き止まりのセルに、壁を 1 枚壊して別の通路につなげる
fn remove_dead_end(
    h: usize,
    w: usize,
    rng: &mut Rng,
    walls: &mut Vec<Vec<bool>>,
) {
    for cy in 0..h {
        for cx in 0..w {
            if open_edges(walls, cx, cy) != 1 {
                continue;
            }
            let (r, c) = (2 * cy + 1, 2 * cx + 1);

            let mut candidates = Vec::new();
            if cy > 0 && walls[r - 1][c] {
                candidates.push((r - 1, c));
            }
            if cy + 1 < h && walls[r + 1][c] {
                candidates.push((r + 1, c));
            }
            if cx > 0 && walls[r][c-1] {
                candidates.push((r, c - 1));
            }
            if cx + 1 < w && walls[r][c + 1] {
                candidates.push((r, c + 1));
            }

            if !candidates.is_empty() {
                let (wr, wc) = candidates[rng.range(candidates.len())];
                walls[wr][wc] = false;
            }
        }
    }
}

#[derive(Resource)]
pub struct Maze {
    h: usize,
    w: usize,
    walls: Vec<Vec<bool>>,
    traps: Vec<(usize, usize)>,
}

impl Maze {
    pub fn new(
        h: usize,
        w: usize,
        extra_openings: usize,
        rng: &mut Rng,
    ) -> Self {
        let tile_cols = 2 * w + 1;
        let tile_rows = 2 * h + 1;
        let mut walls = vec![vec![true; tile_cols]; tile_rows];
        let mut visited = vec![vec![false; w]; h];

        let start = (w / 2, h - 1);
        visited[start.1][start.0] = true;
        walls[2 * start.1 + 1][2 * start.0 + 1] = false;
        let mut stack = vec![start];

        while let Some(&(cx, cy)) = stack.last() {
            let mut next = Vec::new();
            for (x, y) in get_sorround(cx, cy, h, w) {
                if visited[y][x] {
                    continue;
                }
                next.push((x, y));
            }

            if next.is_empty() {
                stack.pop();
                continue;
            }

            let (nx, ny) = next[rng.range(next.len())];
            visited[ny][nx] = true;
            walls[cy + ny + 1][cx + nx + 1] = false;
            walls[2 * ny + 1][2 * nx + 1] = false;
            stack.push((nx, ny));
        }

        for _ in 0..extra_openings {
            let r = 1 + rng.range(tile_rows - 2);
            let c = 1 + rng.range(tile_cols - 2);
            if (r + c) % 2 == 1 {
                walls[r][c] = false;
            }
        }

        remove_dead_end(h, w, rng, &mut walls);

        for c in 1..tile_cols - 1 {
            walls[1][c] = false;
        }

        Self {
            h,
            w,
            walls,
            traps: Vec::new(),
        }
    }

    pub fn tile(&self) -> f32 {
        TILE
    }

    pub fn tile_cols(&self) -> usize {
        2 * self.w + 1
    }

    pub fn tile_rows(&self) -> usize {
        2 * self.h + 1
    }

    pub fn tile_to_world(&self, col: usize, row: usize) -> Vec2 {
        Vec2::new(
            (col as f32 - (self.tile_cols() as f32 - 1.0) / 2.0) * TILE,
            -(self.tile_rows() as f32 - 2.0 - row as f32) * TILE,
        )
    }

    pub fn goal_z(&self) -> f32 {
        self.tile_to_world(0, 1).y
    }

    pub fn is_wall(&self, x: usize, y: usize) -> bool {
        self.walls[y][x]
    }

    fn maze_center(&self) -> Vec3 {
        Vec3::new(
            0.0,
            0.0,
            (
                self.tile_to_world(0, 0).y + self.tile_to_world(0, self.tile_rows() - 1).y
            ) / 2.0,
        )
    }

    pub fn is_trap(&self, col: usize, row: usize) -> bool {
        self.traps.contains(&(col, row))
    }

    pub fn trap_positions(&self) -> Vec<Vec2> {
        self.traps.iter().map(|&(c, r)| self.tile_to_world(c, r)).collect()
    }

    pub fn pick_women_spots(&self, women_count: usize, rng: &mut Rng) -> Vec<Vec2> {
        let mut spots = Vec::new();
        let mut tries = 0;
        while spots.len() < women_count && tries < 1000 {
            tries += 1;
            let r = 2 + rng.range(self.tile_rows() - 7);
            let c = 1 + rng.range(self.tile_cols() - 2);
            if self.walls[r][c]
                || self.is_trap(c, r)
                || self.is_near_start(r, c)
                || spots.contains(&(c, r))
            {
                continue;
            }
            spots.push((r, c));
        }

        let max_offset = TILE / 2.0 - WOMAN_MARGIN;
        spots
            .into_iter()
            .map(|(r, c)| {
                let center = self.tile_to_world(c, r);
                let dx = (rng.range(1001) as f32 / 500.0 - 1.0) * max_offset;
                let dz = (rng.range(1001) as f32 / 500.0 - 1.0) * max_offset;
                center + Vec2::new(dx, dz)
            })
        .collect()
    }

    pub fn world_to_tile(&self, p: Vec2) -> Option<(usize, usize)> {
        let p0 = self.tile_to_world(0, 0);
        let dc = self.tile_to_world(1, 0) - p0;
        let dr = self.tile_to_world(0, 1) - p0;
        let c = ((p - p0).dot(dc) / dc.length_squared()).round();
        let r = ((p - p0).dot(dr) / dr.length_squared()).round();
        if c < 0.0 || r < 0.0 || c >= self.tile_cols() as f32 || r >= self.tile_rows() as f32 {
            return None;
        }
        Some((c as usize, r as usize))
    }

    pub fn shortest_path(&self, start: (usize, usize)) -> Option<Vec<Vec2>> {
        let cols = self.tile_cols();
        let rows = self.tile_rows();
        let idx = |c: usize, r: usize| r * cols + c;

        let mut seen = vec![false; cols * rows];
        let mut prev: Vec<Option<usize>> = vec![None; cols * rows];
        let mut queue = VecDeque::new();
        seen[idx(start.0, start.1)] = true;
        queue.push_back(start);

        let goal_z = self.goal_z();
        let half = self.tile() / 2.0;
        let mut target: Option<(usize, usize)> = None;
        let mut best = start;

        while let Some((c, r)) = queue.pop_front() {
            let z = self.tile_to_world(c, r).y;
            if z < self.tile_to_world(best.0, best.1).y {
                best = (c, r);
            }
            if z <= goal_z + half {
                target = Some((c, r));
                break;
            }
            for (dc, dr) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                let nc = c as i32 + dc;
                let nr = r as i32 + dr;
                if nc < 0 || nr < 0 || nc >= cols as i32 || nr >= rows as i32 {
                    continue;
                }
                let (nc, nr) = (nc as usize, nr as usize);
                if self.is_wall(nc, nr) || seen[idx(nc, nr)] {
                    continue;
                }
                seen[idx(nc, nr)] = true;
                prev[idx(nc, nr)] = Some(idx(c, r));
                queue.push_back((nc, nr));
            }
        }

        let (tc, tr) = target.unwrap_or(best);
        let mut path = vec![self.tile_to_world(tc, tr)];
        let mut cur = idx(tc, tr);
        while let Some(p) = prev[cur] {
            path.push(self.tile_to_world(p % cols, p / cols));
            cur = p;
        }
        path.reverse();
        Some(path)
    }

    pub fn place_traps(&mut self, count: usize, rng: &mut Rng) {
        let cols = self.tile_cols();
        let rows = self.tile_rows();
        let idx = |c: usize, r: usize| r * cols + c;

        let mut placables = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                if self.walls[r][c] || self.is_trap(c, r) || self.is_near_start(c, r) {
                    continue;
                }
                placables.push((c, r));
            }
        }

        for _ in 0..count {
            if placables.is_empty() {
                break;
            }
            let index = rng.range(placables.len());
            let (c, r) = placables.remove(index);
            self.traps.push((c, r));
        }
    }

    /// プレイヤーの開始タイル
    fn start_tile(&self) -> (usize, usize) {
        (2 * (self.w / 2) + 1, 2 * (self.h - 1) + 1)
    }

    /// 開始位置に近いか
    fn is_near_start(&self, col: usize, row: usize) -> bool {
        let (sc, sr) = self.start_tile();
        col.abs_diff(sc) + row.abs_diff(sr) <= START_SAFE_DISTANCE
    }
}
