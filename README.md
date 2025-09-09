# Lumo
WebAssemblyに直接コンパイルする静的型付けプログラミング言語

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/archy-none/lumo)

## 概要

LumoはWebAssembly(WASM)にコンパイルすることに特化して設計された自由なプログラミング言語です。分かりやすいシンプルな文法で初心者でも扱いやすく、かつマクロや構造体, 演算子のオーバーロードなど近代的な言語機能が備わっています。静的型付けと型推論, メモリ安全性, JavaScript環境とのシームレスな統合を特徴としています。

## 特徴

### 静的型付け
- **型チェック**: 型の整合性に関する問題はコンパイル時に検出され、実行時に落ちません
- **型推論**: コンパイラが自動的に型を推論し、ボイラープレートコードを削減します
- **メモリ安全**: nullは必ず型付きで、nullチェック演算子`?`でバグを防ぎます
- **スコープ**: ブロックに入る毎に新しいスコープが生成され、名前汚染を防ぎます

### Rustライクな構文
- **馴染みやすい構文**: 構文はRustとOCaml, TypeScriptなどに影響を受けています
- **LET文**: 変数や関数の定義や再代入には一貫して`let`キーワードを使用しています
- **マクロ**: コンパイル時に型に縛られずスコープをキャプチャしたままコードを共通化できます
- **静的例外処理**: 型エラーで分岐する`try-catch`で、複数の型に対応したコードが書けます

### WebAssembly統合
- **高速実行**: LLVMを介さず独自のバックエンドで効率的なバイトコードを生成します
- **JavaScript相互運用**: FFIによるJavaScriptオブジェクトとのシームレスな変換
- **Web・Node.js対応**: フロントエンド・バックエンド両環境のランタイムで動作します
- **仮想DOM**: 仮想DOMサポート付きの組み込みUIフレームワークでWebアプリを簡単に作れます

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
~~ 配列と構造体 ~~
let numbers = [1, 2, 3, 4, 5];
let person = @{ name: "Alice", age: 12 };

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
type LinkList = @{ car: int, cdr: LinkList };

let node(value: int) = memcpy(@{ car: value, cdr: LinkList! });
let append(self: LinkList, other: LinkList) = {
    let current = self;
    while current.cdr? loop {
        let current = current.cdr
    };
    let current.cdr = other;
    self
};
let clone(self: LinkList) = {
    let object = self.memcpy();
    if object.cdr? then {
        let object.cdr = clone(self.cdr)
    };
    object
};

let a = node(1);
let b = node(2).append(node(3));
a.append(b).append(b.clone())
```

型推論サマリーの出力 (`lumo example/list.lm --summary`)
```
# Type Inference Summary
Functions:
 - node(value: int): LinkList
 - append(self: LinkList, other: LinkList): LinkList
 - clone(self: LinkList): LinkList
Variables:
 - a: LinkList
 - b: LinkList
Globals:
Aliases:
 - LinkList: @{ car: int, cdr: LinkList }
Macros:
Returns: LinkList
```

その他のサンプルは `example/` ディレクトリにあります。

---

*Lumo - WebAssembly時代の自由なプログラミング言語*
