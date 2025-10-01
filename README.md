# Lumo
WebAssemblyに直接コンパイルする静的型付けプログラミング言語

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/archy-none/lumo)

## 概要

Lumoは、独自のバックエンドを開発する事によりLLVMなどの既存のコンパイラ基盤を使用しない、直接WATを効率的に出力する事に特化して設計されたプログラミング言語です。分かりやすいシンプルな文法で初心者でも扱いやすく、かつマクロや構造体, 演算子のオーバーロードなど近代的な言語機能が備わっており、静的型付けとメモリ安全性, JavaScript環境とのシームレスな統合を特徴としています。標準ライブラリはJavaScriptで記述され、Lumoの型はJavaScriptオブジェクトとFFIによって相互変換する事ができます。

## 機能紹介

### JavaScript多相関数での演算子のオーバーロード
```
Lumo REPL
> import append([any], [any]): [any]
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
import to_str(num): str;
import to_num(str): num;
import print(str): void;

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
import arrlen([any]): int;

type LinkList = @{ car: int, cdr: LinkList };

overload append = LinkList + LinkList;
overload from_array = [int]: LinkList;

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
let from_array(values: [int]) = {
    let list = node(values[0]);
    let length = values.arrlen();
    let index = 1;
    while index < length loop {
        let list + node(values[index]);
        let index + 1
    };
    list
};

let a = node(100);
let b = [1, 2, 3]: LinkList;
a.clone().append(b) + a
```

型推論サマリーの出力 (`lumo example/list.lm --summary`)
```
# Type Inference Summary
Functions:
 - arrlen(0: [LinkList]): int
 - node(value: int): LinkList
 - append(self: LinkList, other: LinkList): LinkList
 - clone(self: LinkList): LinkList
 - from_array(values: [int]): LinkList
Overloads:
 - append: LinkList + LinkList
 - from_array: [int] : LinkList
Variables:
 - a: LinkList
 - b: LinkList
Globals:
Aliases:
 - LinkList: @{ car: int, cdr: LinkList }
Macros:
Returns: LinkList
```
