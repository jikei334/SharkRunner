use bevy::prelude::*;

use crate::utils::Rng;


// タイルの大きさ
const TILE: f32 = 4.0;


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

    pub fn pick_women_spots(&self, women_count: usize, rng: &mut Rng) -> Vec<Vec2> {
        let mut spots = Vec::new();
        let mut tries = 0;
        while spots.len() < women_count && tries < 1000 {
            tries += 1;
            let r = 2 + rng.range(self.tile_rows() - 7);
            let c = 1 + rng.range(self.tile_cols() - 2);
            if !self.walls[r][c] && !spots.contains(&(r, c)) {
                spots.push((r, c));
            }
        }
        spots.into_iter().map(|(r, c)| self.tile_to_world(c, r)).collect()
    }
}
