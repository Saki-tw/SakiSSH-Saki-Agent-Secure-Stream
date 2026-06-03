# SakiAgentSSH ビルドガイド (Building Guide)

[🇹🇼 繁體中文](BUILDING_zh-TW.md) | [🇯🇵 日本語](BUILDING_ja.md) | [🇺🇸 English](BUILDING.md)

これは、ソースコードからこの小さな橋を組み立てるためのガイドです。
雨の降る午後、ターミナルで `cargo build` と打ち込むと、文字列たちが少しずつ形を変えて、遠くの機械と繋がるための神経になっていく……そんな静かな時間を一緒に過ごしましょう。

## 必要なもの (Prerequisites)

この橋を架ける前に、あなたの機械に以下の道具が揃っているか確認してくださいね。

1.  **Rust Toolchain** (1.75+)
    - すべての始まりです。`rustup` を使って優しくインストールしてあげてください。
    - Linux / macOS: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    - Windows: [rustup.rs](https://rustup.rs/) から `rustup-init.exe` を受け取ってください。
2.  **Protocol Buffers Compiler (`protoc`)**
    - `tonic` という魔法が、`proto/sakissh.proto` の約束をコードに翻訳するために必要です。
    - macOS: `brew install protobuf`
    - Windows: `choco install protoc` または Scoop で。
3.  **macOS GUI のための道具**
    - もし、マカロンパープルと忘れな草のブルーの窓（GUI App）を作りたいなら。
    - Xcode 16+
    - XcodeGen: `brew install xcodegen`

## コア CLI のコンパイル (Rust Core)

GUIを持たない、純粋で静かなコマンドラインツールの作り方です。重たい処理を任せる遠くのサーバーには、これが一番似合います。

```bash
# まず、プロジェクトの入り口に立ちます
cd SakiAgentSSH

# Daemon（遠くで待つ守護者）を組み立てます
cd saki-ssh-daemon
cargo build --release
# 完成したものは target/release/sakisshd に静かに置かれます

# 少し戻って、Client（指示を届ける使者）を組み立てます
cd ../saki-ssh-client
cargo build --release
# こちらは target/release/sakissh にあります
```

## macOS GUI アプリのコンパイル (SwiftUI)

私たちは、SakiAgentSSH に優しいUIの服を着せました。これらを組み立てるには、`xcodegen` を使って設計図（`.xcodeproj`）を魔法のように描き出します。

```bash
# プロジェクトのルートにいることを確認してくださいね

# --------------------------
# 1. Daemon App の組み立て
# --------------------------
cd SakiAgentSSH-Daemon
# Xcode の設計図を描きます
xcodegen generate
# コマンドラインから優しくビルドします
xcodebuild build -configuration Release -scheme SakiAgentSSHDaemon
# 出来上がった .app は build/Release/ の中で待っています

# --------------------------
# 2. Client App の組み立て
# --------------------------
cd ../SakiAgentSSH-Client
xcodegen generate
xcodebuild build -configuration Release -scheme SakiAgentSSHClient
```

> **私からのお願い**：`.xcodeproj` ファイルは、決して Git の記憶（リポジトリ）に残さないでください。競合という悲しい出来事を避けるため、いつでも `project.yml` から `xcodegen` で新しく生み出すようにしましょう。

## もしも迷子になったら (Troubleshooting)

- **`protoc` が見つからない**：`cargo build` で赤くて痛そうなエラーが出たときは、たいてい `protoc` が迷子になっています。環境変数（PATH）をもう一度見直してあげてください。
- **macOS App が開かない**：手動で `.app` を動かしたときは、Appleの厳しい門番に止められているかもしれません。`xattr -dr com.apple.quarantine /Applications/SakiAgentSSHDaemon.app` で、そっと門を開けてあげましょう。

--
*「コンパイルを通る詩。どうか、エラーなく世界が繋がりますように。」*