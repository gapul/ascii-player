了解いたしました。開発主体であるLLMが直接解釈し、コード生成を実行可能な形式で技術仕様書を再構築します。曖昧さを排除し、構造化され、具体的な指示に焦点を当てたドキュメントを作成します。

-----

### **LLM向け実装指示書: `nix-ascii-player`**

**目標:** 指定された動画ファイルを、ユーザーの`dotfiles`環境 に完全に統合された、カラー・レスポンシブ対応のASCIIアニメーションとして再生するRust製CLIアプリケーションを生成せよ。

-----

### **フェーズ1: プロジェクト定義と環境設定**

**TASK:** プロジェクトのコア属性を定義し、Nixによる開発環境をセットアップする。

```yaml
# Project Definition
project_name: "nix-ascii-player"
description: "A responsive, color-enabled ASCII video player for the terminal, optimized for a specific Nix-based dotfiles environment."
language: "Rust"
target_os: "macOS"
target_platform: "nix-darwin"
target_terminal: "WezTerm"
primary_dependencies:
  - { name: "ffmpeg", reason: "Video decoding" }
  - { name: "rustc", reason: "Compilation" }
  - { name: "cargo", reason: "Build system & package management" }
key_features:
  - "Video to ASCII conversion"
  - "24-bit True Color support"
  - "Terminal resize (responsive) handling"
  - "GPU-accelerated terminal background transparency support"
  - "Seamless integration with the user's Nix environment"
```

**指示:**

1.  `dotfiles`リポジトリ内に、`nix/test-project` の構成を参考に、`nix-ascii-player`という名前で新しいRustプロジェクトを作成せよ。`Cargo.toml`を生成すること。

2.  `flake.nix`（またはそれに準ずるNix設定ファイル）に、以下の`devShell`を定義するコードを追加せよ。これにより`nix develop`で開発環境が構築される。

    ```nix
    # In your flake.nix devShells section
    nix-ascii-player = pkgs.mkShell {
      name = "nix-ascii-player-dev";
      buildInputs = with pkgs; [
        # Rust toolchain
        rustc
        cargo
        rust-analyzer

        # Core dependency for video processing
        ffmpeg

        # Build dependencies
        pkg-config
        openssl
      ];
      # Environment variables for Rust development
      RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
    };
    ```

-----

### **フェーズ2: コアロジック実装 (Rust)**

**TASK:** Rustでアプリケーションのコア機能を実装する。以下のモジュールとAPIシグネチャに従うこと。

#### **モジュール: `cli`**

  * **クレート:** `clap` (version 4.x, with `derive` feature)

  * **ファイル:** `src/cli.rs`

  * **実装:** 以下の仕様でCLI引数をパースする構造体`Cli`を定義せよ。

    ```rust
    // src/cli.rs
    use std::path::PathBuf;
    use clap::Parser;

    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    pub struct Cli {
        /// Path to the video file to play
        #[arg(required = true)]
        pub file_path: PathBuf,

        /// Loop the video playback
        #[arg(short, long)]
        pub loop_playback: bool,

        /// Set playback speed factor
        #[arg(short, long, default_value_t = 1.0)]
        pub speed: f64,

        /// Enable transparent background by not drawing background colors
        #[arg(short, long)]
        pub transparent: bool,

        /// Enable alpha channel support with a specific threshold (0-255)
        #[arg(short, long, value_name = "THRESHOLD")]
        pub alpha_threshold: Option<u8>,
    }
    ```

#### **モジュール: `decoder`**

  * **クレート:** `ffmpeg-next`

  * **ファイル:** `src/decoder.rs`

  * **実装:** 動画ファイルをデコードし、フレームをRGB形式で抽出する機能を実装せよ。

    ```rust
    // src/decoder.rs
    use ffmpeg_next as ffmpeg;
    use std::path::Path;

    pub struct FrameIterator {
        // ... internal state
    }

    impl Iterator for FrameIterator {
        type Item = ffmpeg::frame::Video;
        // ...
    }

    pub fn load_video(path: &Path) -> Result<FrameIterator, ffmpeg::Error> {
        // Implementation here
    }
    ```

#### **モジュール: `converter`**

  * **ファイル:** `src/converter.rs`

  * **実装:** 1つのビデオフレームを、指定された解像度のASCII表現に変換するロジックを実装せよ。

    ```rust
    // src/converter.rs
    // Define a struct to hold the result of the conversion
    pub struct AsciiFrame {
        pub characters: Vec<char>,
        pub fg_colors: Vec<(u8, u8, u8)>,
        pub width: u16,
        pub height: u16,
    }

    const ASCII_RAMP: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

    pub fn frame_to_ascii(
        frame: &ffmpeg::frame::Video,
        terminal_width: u16,
        terminal_height: u16,
    ) -> AsciiFrame {
        // 1. Resize the frame to match terminal dimensions (respecting aspect ratio).
        // 2. For each pixel of the resized frame:
        //    a. Calculate luminance: L = 0.2126*R + 0.7152*G + 0.0722*B
        //    b. Map luminance (0-255) to an index in ASCII_RAMP.
        //    c. Store the character and the original RGB value.
        // 3. Return an AsciiFrame instance.
    }
    ```

#### **モジュール: `renderer`**

  * **クレート:** `crossterm`

  * **ファイル:** `src/renderer.rs`

  * **実装:** `AsciiFrame`をターミナルに描画する機能を実装せよ。

    ```rust
    // src/renderer.rs
    use crossterm::{execute, style::{Color, Print, SetForegroundColor}, cursor};
    use std::io::stdout;
    use crate::converter::AsciiFrame;

    pub fn render_frame(frame: &AsciiFrame, transparent_mode: bool) {
        let mut stdout = stdout();
        execute!(stdout, cursor::MoveTo(0, 0)).unwrap();

        for y in 0..frame.height {
            for x in 0..frame.width {
                let index = (y as usize * frame.width as usize) + x as usize;
                let char_to_print = frame.characters[index];
                let (r, g, b) = frame.fg_colors[index];

                // Set foreground color using True Color
                execute!(
                    stdout,
                    SetForegroundColor(Color::Rgb { r, g, b }),
                    Print(char_to_print)
                ).unwrap();
            }
            execute!(stdout, Print('\n')).unwrap();
        }
    }
    ```

    **注意:** `transparent_mode`が`true`の場合、背景色を設定するANSIコードを出力してはならない。

-----

### **フェーズ3: アプリケーション統合と実行ループ**

**TASK:** `main.rs`を完成させ、全てのモジュールを統合し、実行ループを構築する。

**ファイル:** `src/main.rs`

**実装指示:**

1.  `cli::Cli::parse()`を使用してコマンドライン引数を解析せよ。
2.  `crossterm::terminal` を使用してターミナルをrawモードに設定し、カーソルを非表示にせよ。
3.  メインループを構築せよ:
      * `crossterm::terminal::size()`で現在のターミナルサイズを取得せよ。
      * `decoder`でビデオフレームを取得せよ。
      * `converter`でフレームを`AsciiFrame`に変換せよ。
      * `renderer`で`AsciiFrame`を描画せよ。
      * 動画のFPSと`--speed`引数に基づいて適切な`sleep`を挿入し、再生速度を制御せよ。
      * `crossterm::event::poll`でキー入力を非同期にチェックし、'q'またはCtrl+Cで終了するようにせよ。
4.  レスポンシブ対応:
      * `crossterm::event::read()`を使用して`Event::Resize(width, height)`を待ち受け、ループの先頭でターミナルサイズを更新せよ。
5.  アプリケーション終了時、必ずターミナルの状態（rawモード、カーソル表示など）を復元すること。

-----

### **フェーズ4: Dotfilesエコシステムとの連携**

**TASK:** アプリケーションを`dotfiles`に完全に統合する。

#### **Nixパッケージ化**

  * **指示:** `flake.nix`内に、本アプリケーションをビルドするためのNixパッケージ定義を記述せよ。

    ```nix
    # In your flake.nix packages section
    nix-ascii-player = pkgs.rustPlatform.buildRustPackage {
      pname = "nix-ascii-player";
      version = "0.1.0";
      src = ./path/to/nix-ascii-player; # Adjust path
      cargoLock.lockFile = ./path/to/nix-ascii-player/Cargo.lock; # Adjust path
      nativeBuildInputs = [ pkgs.pkg-config ];
      buildInputs = [ pkgs.ffmpeg pkgs.openssl ];
    };
    ```

#### **SketchyBar連携**

  * **指示:** アプリケーションに `--sketchybar-item <ITEM_NAME>` オプションを追加せよ。
  * このオプションが指定された場合、再生開始時に以下のコマンドをサブプロセスとして実行するロジックを実装せよ。
      * `sketchybar --set <ITEM_NAME> label="▶ {filename}"`
  * 再生終了時には以下を実行せよ。
      * `sketchybar --set <ITEM_NAME> label=""`
  * この機能は、`configs/wm/sketchybar/plugins/`内のスクリプト群 と同様の思想で実装すること。

-----

### **フェーズ5: 検証プロトコル**

**TASK:** 実装が仕様通りであることを確認するためのテストを定義する。

  * **ユニットテスト:**
      * `converter`モジュールに対し、1x1の黒ピクセル (`#000000`) と白ピクセル (`#FFFFFF`) のフレームを入力し、それぞれ`ASCII_RAMP`の最初と最後の文字が出力されることを検証するテストを記述せよ。
  * **統合テスト:**
      * `tests/`ディレクトリを作成し、短いサンプル動画 (`sample.mp4`) を配置せよ。
      * その動画を再生し、3フレーム描画した後に正常終了するテストケースを作成せよ。パニックが発生しないことを確認する。
  * **手動テスト:**
    1.  `nix develop`でシェルに入る。
    2.  `cargo run -- <video_path>` でアプリケーションを実行する。
    3.  WezTermウィンドウのサイズを変更し、描画が追従することを確認する。
    4.  `--transparent`オプションを付与し、`wezterm.lua` の背景設定が透けて見えることを確認する。
    5.  'q'キーで正常に終了できることを確認する。