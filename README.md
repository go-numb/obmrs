# obmrs for Rust 

[This is a project under review. Since: 2024/10/07]  

`obmrs`は、Rustで記述された仮想通貨や株式市場向けの**OrderBook Management System**です。参加者は取引所から送られるOrderbook（注文板）を受信し、効率的かつ整理された方法で保持するための構造体を活用できます。  

## 概要

このライブラリは、`OrderBoard`という構造体を提供し、Orderbookの買い注文（bids）と売り注文（asks）を**価格でソートされたマップ**として保存し、効率的な入出力およびデータ保持を行います。

`OrderBoard`は、価格ごとにまとめた`Books`構造体のデータを保存します。これは、各価格に紐づく複数の注文ではなく、各価格に対する総量を記録します。

`converter()`関数を用いることで、さまざまな型（`i64`, `f64`, `String`, `Decimal`など）から`Book`（注文量と価格のペア）構造体への変換をシンプルに行うことができます。

## 特徴

- **高速なデータ処理**: 価格と注文サイズを効率的に扱うため、`BTreeMap`を使用しています。これにより、データの検索・挿入・削除が高速に行えます。
- **柔軟な型対応**: `converter`機能により、複数の異なるデータ型（整数や小数、文字列など）を簡単に`Book`構造体に変換可能です。
- **ベストプライスの取得**: 買い注文と売り注文のうち、最良の注文（最高の買い、最低の売り）を素早く取得できます。
- **サイズフィルタリング機能**: `wall(size)`により、指定したサイズ以上の注文をフィルタリングすることができます。
- **内側注文のトリム機能**: `trim_inside_best_book`メソッドで、注文板の内側にある注文をトリムし、最適化された状態に維持できます。これにより、不要なデータが削除され、メモリ効率を向上させます。

## インストール

`Cargo.toml`に以下の依存関係を追加してください:

```
$ cargo add obmrs
```

```toml
[dependencies]
obmrs = "0.1.0" 
```



## 使用方法

以下のシンプルなコードサンプルを通じて、基本的な操作の流れを確認できます。

### 1. **OrderBoardの作成**

`OrderBoard`構造体を生成し、初期化を行います。`bids`は買い注文、`asks`は売り注文です。

```rust
use obmrs::{OrderBoard, Book};
use rust_decimal::Decimal;

fn main() {
    let max_length = 100; // 最大100件
    let digits = 6; // 価格の精度は小数点以下6桁
    let mut orderboard = OrderBoard::new(max_length, digits);
}
```

### 2. **Orderの追加と変換**

異なるデータ型をサポートする`converter()`を使って、`Book`構造体に変換し、`OrderBoard`に追加します。

```rust
use obmrs::{converter, Books, Book};
use rust_decimal::Decimal;

fn main() {
    let mut ob = OrderBoard::new(100, 6);

    // 様々な型からBookへ変換
    let book1 = converter(100i64, 50i64).unwrap(); // price: 100, size: 50のBook
    let book2 = converter(100.5f64, 200f64).unwrap(); // price: 100.5, size: 200のBook

    // OrderBoardへ追加
    ob.asks.push(book1);
    ob.bids.push(book2);
}
```

### 3. **ベストプライスの取得**

`OrderBoard`から現在の最良の売り注文（ask）と最良の買い注文（bid）を取得します。

```rust
fn main() {
    let mut orderbook = OrderBoard::new(100, 6);

    // 追加した後、最良のaskとbidを取得
    let (bestask, bestbid) = orderbook.best();
    
    if let Some(ask) = bestask {
        println!("Best Ask: {}, Size: {}", ask.price, ask.size);
    }
    
    if let Some(bid) = bestbid {
        println!("Best Bid: {}, Size: {}", bid.price, bid.size);
    }
}
```

### 4. **サイズフィルタリング（壁注文の取得）**

指定されたサイズ以上の注文を取得したい場合は、`Books::wall(size)`メソッドを使用します。

```rust
fn main() {
    let mut ob = OrderBoard::new(100, 6);
    
    // 例えば、サイズ50以上の注文を取得する
    if let Some(wall) = ob.bids.wall(Decimal::new(50, 0)) {
        println!("Wall Bid: Price: {}, Size: {}", wall.price, wall.size);
    }
}
```

### 5. **[option] 内側注文の削除**

`trim_inside_best_book`メソッドを使って、現在のベストASKとベストBIDより内側にある注文を削除します。この機能により不要な注文が削除され、効率化が図れます。  
size:0が返ってくる取引所情報では不要です。size:0がpushされた場合、当クレートはその価格情報を削除します。

```rust
fn main() {
    let mut ob = OrderBoard::new(100, 6);
    
    // 各ベストの内側（スプレッド内）にある注文を削除
    let bestbid = 99.9;
    let bestask = 100.1;
    ob.trim_inside_best_book(bestask, bestbid);
}
```

## 機能詳細

- `new`: 新しい`OrderBoard`を作成します。最大保持数と精度を引数に取ります。
- `converter`: 様々な型のデータ（`i64`, `f64`, `String`, `Decimal`など）を`Book`に変換します。
- `push`: 単一の`Book`を`Books`に追加します。
- `extend`: 複数の`Book`を一度に`Books`構造に追加します。
- `best`: 最良の売り・買い注文（best_ask, best_bid）を取得します。
- `wall(size)`: 指定したサイズ以上の注文（壁）を取得します。
- `trim_inside_best_book(best_ask, best_bid)`: ベストASKとベストBIDの内側にある注文を削除します。

## via Golang Client

Go言語でこのライブラリを扱いたい場合は、以下にあるクライアントライブラリを参考にしてください。

[go-obm @go-numb](https://github.com/go-numb/go-obm)

## Author

[@_numbP](https://twitter.com/_numbP)

## License

[MIT License](https://github.com/go-numb/obmrs/blob/master/LICENSE)  
このプロジェクトはMITライセンスに基づいて提供されており、自由に使用・改変・再配布が可能です。
