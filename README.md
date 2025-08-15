# Lumo
WebAssemblyに直接コンパイルする静的型付けプログラミング言語

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/TTaichi/lumo)

## 目次
- [概要](#概要)
- [特徴](#特徴)
- [インストール](#インストール)
- [使用方法](#使用方法)
- [言語構文](#言語構文)
- [サンプルコード](#サンプルコード)
- [アーキテクチャ](#アーキテクチャ)
- [開発](#開発)
- [貢献](#貢献)
- [ライセンス](#ライセンス)

## 概要

LumoはWebAssembly (WASM)にコンパイルすることに特化して設計された自由なプログラミング言語です。Rustの安全性とパフォーマンスを、OCamlのような関数型プログラミング言語の表現力と組み合わせています。静的型付けと型推論、メモリ安全性、JavaScript環境とのシームレスな統合を特徴としています。

## 特徴

### 静的型付け
- **型チェック**: 型の整合性に関する問題はコンパイル時に検出され、実行時に落ちません
- **型推論**: コンパイラが自動的に型を推論し、ボイラープレートコードを削減します
- **メモリ安全**: nullは必ず型付きで、nullチェック演算子`?`でバグを防ぎます
- **スコープ**: ブロックに入る毎に新しいスコープが生成され、名前汚染を防ぎます

### Rust風構文
- **馴染みやすい構文**: 構文はRustとOCaml, TypeScriptなどに影響を受けています
- **`let`文**: 変数や関数の定義や再代入には一貫して`let`キーワードを使用しています
- **マクロ**: コンパイル時に型に縛られずスコープをキャプチャしたままコードを共通化できます
- **静的例外処理**: 型エラーで分岐する`try-catch`で、マクロをもっと便利に汎用的に

### WebAssembly統合
- **高速実行**: LLVMを介さず独自のバックエンドで効率的なバイトコードを生成します
- **JavaScript相互運用**: FFIによるJavaScriptオブジェクトとのシームレスな変換
- **Web・Node.js対応**: フロントエンド・バックエンド両環境のランタイムで動作します
- **仮想DOM**: 仮想DOMサポート付きの組み込みUIフレームワークでWebアプリを簡単に作れます

## インストール

### 前提条件
- Rust（最新安定版）
- Node.js（v16以上）
- wasm-pack

### ソースからのビルド
```bash
git clone https://github.com/archy-none/lumo.git
cargo install --path ./lumo/app
```

### WebAssemblyバインディングのビルド
```bash
# ビルドスクリプトを実行
./build_wasm.sh

# または手動で:
cd wasm
wasm-pack build --target nodejs
wasm-pack build --target web
```

## 使用方法

### コマンドラインインターフェース
```bash
# Lumoファイルをコンパイル
lumo example/fizzbuzz.lm

# 型推論サマリーを表示
lumo example/fizzbuzz.lm --summary

# Node.jsランタイムでコンパイル・実行
node run.mjs example/fizzbuzz.lm
```

### REPLモード
```bash
node repl.mjs
```

### Web統合
```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import { lumo } from 'https://archy-none.github.io/lumo/runtime/web.mjs';

        const code = `
            load alert(message: str): void;
            alert("Hello, WebAssembly from Lumo!")
        `;
        lumo(code);
    </script>
</head>
</html>
```

## 言語構文

### 変数と関数
```lumo
~~ 変数宣言 ~~
let x = 5;

~~ 関数定義 ~~
let fact(n: int): int = {
    if n == 0
        then 1 ~~ 再帰 ~~
        else n * fact(n - 1)
};

~~ 関数呼び出し ~~
fact(x) == x.fact()
```

### 制御フロー
```lumo
~~ 繰り返し ~~
while i < 10 loop {
    ~~ 条件分岐 ~~
    if i % 2 == 0 then {
        ~~ フォーマット文字列 ~~
        let message = f"{i} is an even number";
        print(message)
    };
    let i + 1
}
```

### オブジェクト
```lumo
let numbers = [1, 2, 3, 4, 5]; ~~ 配列 ~~
let person = @{ name: "Alice", age: 12 }; ~~ 構造体 ~~

~~ 内部値の操作 ~~
let numbers[-1] * 10;
let person.name = "Bob";

~~ 列挙型 ~~
type Status = ( Success | Error | Pending );
let current = Status#Pending
```

### マクロ
```lumo
~~ マクロ定義 ~~
macro inc(n) = {
    ~~ 静的例外処理 ~~
    try n + 1 catch n + 1.0
};

~~ 使用例 ~~
inc(5): num + inc(3.14)
```

### 演算子のオーバーロード
```lumo
~~ JavaScript関数の読み込み ~~
load repeat(text: str, count: int): str;

~~ 関数を特定の型の演算に適用 ~~
overload repeat = str * int;
"Hey " * 10
```

## サンプルコード

### FizzBuzz
```lumo
load to_str(n: num): str;
load print(n: str): void;

let fizzbuzz(n: int) = {
    if n % 15 == 0 then "FizzBuzz"
    else if n % 3 == 0 then "Fizz"
    else if n % 5 == 0 then "Buzz"
    else n: str
};

let i = 1;
while i <= 100 loop {
    i.fizzbuzz().print();
    let i + 1
}
```

### リンクリスト
```lumo
type LinkList = @{ value: int, next: LinkList };

let car(self: LinkList) = self.value;
let cdr(self: LinkList) = self.next;
let node(value: int) = memcpy(@{ value: value, next: LinkList! });
let append(self: LinkList, other: LinkList) = {
    let current = self;
    while current.next? loop {
        let current = current.next
    };
    let current.next = other;
    self
};

let a = node(1);
let b = node(2).append(node(3));
b.append(a)
```

その他のサンプルは `example/` ディレクトリにあります。

## アーキテクチャ

### プロジェクト構造
```
lumo/
├── core/           # 核となる言語実装
│   ├── src/
│   │   ├── lexer.rs    # トークン化
│   │   ├── expr.rs     # 式の解析
│   │   ├── stmt.rs     # 文の解析
│   │   ├── type.rs     # 型システム
│   │   └── value.rs    # 値の型
│   └── Cargo.toml
├── app/            # コマンドラインインターフェース
│   ├── src/
│   │   └── main.rs
│   └── Cargo.toml
├── wasm/           # WebAssemblyバインディング
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
├── docs/           # ドキュメントとランタイム
│   ├── runtime/    # JavaScriptランタイム
│   └── wasm/       # 生成されたWASMバインディング
├── example/        # サンプルプログラム
└── build_wasm.sh   # ビルドスクリプト
```

### コンパイルパイプライン
1. **字句解析**: ソースコードがトークンストリームにトークン化されます
2. **構文解析**: トークンが抽象構文木（AST）に解析されます
3. **型チェック**: 型推論による静的型解析
4. **コード生成**: ASTがWebAssembly Text format（WAT）にコンパイルされます
5. **WebAssembly**: WATがバイナリWebAssembly形式にコンパイルされます

### ランタイム環境
- **Node.jsランタイム**: ファイルシステムアクセス付きのフル機能ランタイム
- **Webランタイム**: DOM統合付きのブラウザ互換ランタイム
- **標準ライブラリ**: math、OS、random、datetime、time操作のための組み込みモジュール

## 開発

### 開発環境のセットアップ
```bash
# Rustをインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# wasm-packをインストール
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# クローンとビルド
git clone <repository-url>
cd lumo/main
cargo build
```

### テストの実行
```bash
# Rustテストを実行
cargo test

# サンプルのテスト
node run.mjs example/fizzbuzz.lm
node run.mjs example/app.lm
```

### ドキュメントのビルド
```bash
# Rustドキュメントを生成
cargo doc --open

# WebAssemblyバインディングをビルド
./build_wasm.sh
```

## 貢献

貢献大歓迎！以下の手順に従ってください：

1. リポジトリをフォーク
2. 機能ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更を実装
4. 変更に対するテストを追加
5. すべてのテストが通ることを確認 (`cargo test`)
6. 変更をコミット (`git commit -m 'Add amazing feature'`)
7. ブランチにプッシュ (`git push origin feature/amazing-feature`)
8. プルリクエストを作成

### コードスタイル
- Rustの命名規則に従う
- コードフォーマットには `rustfmt` を使用
- パブリックAPIにはドキュメントを追加
- 新機能にはテストを含める

### 問題の報告
バグの報告や機能要求にはGitHubのissueを使用してください。以下を含めてください：
- Lumoのバージョン
- オペレーティングシステム
- 最小限の再現ケース
- 期待される動作と実際の動作

## ライセンス

このプロジェクトはMITライセンスの下でライセンスされています - 詳細は [LICENSE](LICENSE) ファイルをご覧ください。

## リンク

- [ドキュメント](docs/index.html)
- [サンプル](example/)
- [DeepWiki](https://deepwiki.com/archy-none/lumo)

---

*Lumo - WebAssembly時代の自由なプログラミング言語*
