# Lumo
WebAssemblyに直接コンパイルする静的型付けプログラミング言語

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/archy-none/lumo)

## 概要

LumoはWebAssembly(WASM)にコンパイルすることに特化して設計された自由なプログラミング言語です。分かりやすいシンプルな文法で初心者でも扱いやすく、かつマクロや構造体、演算子のオーバーロードなど近代的な言語機能が備わっており、静的型付けとメモリ安全性、JavaScript環境とのシームレスな統合を特徴としています。

## 機能紹介

### JavaScript多相関数での演算子のオーバーロード
```
Lumo REPL
> load append(a: [any], b: [any]): [any]
> overload append = [any] + [any]
> [1, 2] + [3]
[ 1, 2, 3 ]
> ["one", "two"] + ["three"]
[ 'one', 'two', 'three' ]
```

### マクロ定義とコンパイル時型エラー処理
```
Lumo REPL
> macro inc(n) = { try n + 1 catch n + 1.0 }
> inc(3)
4
> inc(3.14)
4.14
```

## プログラム例

Lumoでは、定番のアルゴリズムも以下のように簡潔に記述することが出来ます。

### FizzBuzz出力
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

overload append = LinkList + LinkList;

let a = node(1);
let b = node(2).append(node(3));
a + b + b.clone()
```

型推論サマリーの出力 (`lumo example/list.lm --summary`)
```
# Type Inference Summary
Functions:
 - node(value: int): LinkList
 - append(self: LinkList, other: LinkList): LinkList
 - clone(self: LinkList): LinkList
Overloads:
 - append: LinkList + LinkList
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
