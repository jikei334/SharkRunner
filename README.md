# SharkRunner

A golf-like drag-and-release game set in a randomly generated dungeon maze, seen from a diagonal 3D camera.
Made with **Rust + Bevy 0.19** for [itch.io Game Off 2026](https://itch.io/jam/game-off-2026) (theme: **SHIFT**).

**Play in the browser (desktop & smartphone):** https://jikei334.itch.io/sharkrunner

<!-- Add a screenshot or GIF here: ![screenshot](docs/screenshot.png) -->

## The idea (theme: SHIFT)

You can shift between two forms whenever you are standing still:

| | Human | Shark |
|---|---|---|
| Speed | Slow | Fast |
| Aim | Precise | Rough |
| Hit radius | Small | Big |
| Touching a woman | She is startled, and police drop from the sky to ambush you | You eat her (she screams) |
| Touching a police trap | You are attacked and lose | You are attacked and lose |

The police traps are placed on the maze, sometimes right on the route to the exit.
A human can squeeze past one with careful aim, but the shark's big hit radius makes it much harder to avoid.
So you have to decide where to shift: shark for speed, human for precision.

## How to play

- **Drag and release** anywhere on the screen to launch yourself, like golf. The farther you pull, the faster you go.
- **Space** or the on-screen **TO SHARK / TO HUMAN** button shifts your form (only while standing still).
- Hitting a wall hard costs HP. If HP reaches zero, you lose.
- A guide arrow shows the shortest path to the exit. It does not avoid police traps, so the route may pass right through one.
- Reach the yellow goal line at the top of the maze with as few strokes as possible.
- After clearing, the next maze is generated randomly. After losing, you retry the same maze.

## Build and run

Requirements: Rust (stable), the wasm target, and [Trunk](https://trunkrs.dev/).

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk

# dev server (http://localhost:8080)
trunk serve

# release build for itch.io (relative paths are required)
trunk build --release --public-url ./
```

Zip the contents of `dist/` and upload it to itch.io as an HTML5 game.

To test on a phone on the same network, serve on all interfaces: `trunk serve --address 0.0.0.0`.

## Project layout

```
src/main.rs     game states, UI, camera, input, level setup
src/player.rs   player, forms, animation
src/maze.rs     random maze generation, traps, women spots, shortest path
src/wall.rs     walls and wall collision
src/police.rs   police traps, attack and ambush
src/woman.rs    women characters and their reactions
src/sfx.rs      BGM and sound effects
src/utils.rs    seeded random number generator
assets/         models (.glb) and audio (.ogg)
```

## Credits

- Engine: [Bevy](https://bevyengine.org/)
- Code, design: jikei334
- 3D models: claude
- BGM and sound effects: claude & jikei334

## License

See [LICENSE](LICENSE).

---

## 日本語

ランダム生成されたダンジョン迷路で遊ぶ、ゴルフ風の「引っ張って離す」ゲームです。
Game Off 2026（テーマ: SHIFT）向けに、Rust + Bevy 0.19 で作りました。

- **人間**: 遅いが正確で、当たり判定が小さい。水着の女性に触れると驚かせてしまい、空から警官が降ってきて襲われます。
- **サメ**: 速いが当たり判定が大きい。女性に触れると食べます。
- **警官のトラップ**: 迷路に置かれていて、ルート上にあることもあります。人間でもサメでも、触れると襲われて負けです。サメは判定が大きいので、避けるのが難しくなります。
- **操作**: 画面を引っ張って離すと発射。スペースキー、または画面のボタンで変身します（止まっているときだけ）。
- **目的**: 壁にぶつかりすぎないようにして、出口まで、できるだけ少ない打数で到達します。ガイドの矢印は最短ルートを示しますが、警官のトラップは避けません。
