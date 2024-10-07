use rust_decimal::{prelude::*, Decimal};

use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use rand::Rng;

use rayon::prelude::*;

/// OrderBoard
/// OrderBoardは、Orderbookのbidsとasksをソートされたマップとして保持及び入出力します。
#[derive(Debug, Clone)]
pub struct OrderBoard {
    pub asks: Books,
    pub bids: Books,
}

#[derive(Debug, Clone)]
enum Side {
    Ask,
    Bid,
}

impl OrderBoard {
    /// max_number_of_books: 保持する最大の注文数
    /// digits: 価格をunique key as i64に変換するための桁数
    /// 例: 扱う価格(price)1.2345で、浮動小数点深度(digits)が4 -> Books keyは12345となる
    pub fn new(max_number_of_books: usize, digits: u64) -> Self {
        Self {
            asks: Books::new(Side::Ask, max_number_of_books, digits),
            bids: Books::new(Side::Bid, max_number_of_books, digits),
        }
    }

    /// bestbid, bestaskの内側を削除する
    /// bestbid, bestask価格であるbookは保持されます
    pub fn trim_inside_best_book(&mut self, best_ask: Decimal, best_bid: Decimal) {
        self.asks.trim_inside_best_book(best_ask);
        self.bids.trim_inside_best_book(best_bid);
    }

    /// bestbid, bestaskのBookを取得する
    pub fn best(&self) -> (Option<&Book>, Option<&Book>) {
        (self.asks.best(), self.bids.best())
    }

    // 指定したsize以上のBookを取得
    pub fn wall(&self, size: Decimal) -> (Option<&Book>, Option<&Book>) {
        (self.asks.wall(size), self.bids.wall(size))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PriceKey(i64);

impl PriceKey {
    fn add(&self, n: i64) -> Self {
        PriceKey(self.0 + n)
    }
}

#[derive(Debug, Clone)]
pub struct Books {
    side: Side,
    max_number_of_books: usize,
    /// BTreeMap<i64, Book>
    /// price * 10^digits = books key<i64>
    digits: u64,
    /// key: price string, value: Book
    pub books: BTreeMap<PriceKey, Book>,
}

impl Books {
    fn new(side: Side, max_number_of_books: usize, digits: u64) -> Self {
        Self {
            side,
            max_number_of_books,
            digits,
            books: BTreeMap::new(),
        }
    }

    pub fn push(&mut self, book: Book) {
        if book.size.is_zero() {
            self.books.remove(&book.to_key(self.digits));
            return;
        };
        self.books.insert(book.to_key(self.digits), book);
    }

    pub fn extend(&mut self, books: Vec<Book>) {
        books.into_iter().for_each(|book| self.push(book));
    }

    /// 末尾を削除
    /// bidの場合は価格最下位を削除
    /// askの場合は価格最上位を削除
    pub fn restrict(&mut self) {
        if self.books.len() > self.max_number_of_books {
            match self.side {
                Side::Bid => {
                    self.books.pop_first();
                }
                Side::Ask => {
                    self.books.pop_last();
                }
            }
        }
    }

    /// 最前列を取得
    /// bidの場合は価格最上位を取得
    /// askの場合は価格最下位を取得
    pub fn best(&self) -> Option<&Book> {
        match self.side {
            Side::Bid => self.books.values().next_back(),
            Side::Ask => self.books.values().next(),
        }
    }

    /// priceより内側を削除
    /// bidの場合はpriceより高い価格のものを削除
    /// askの場合はpriceより低い価格のものを削除
    pub fn trim_inside_best_book(&mut self, price: Decimal) {
        let key = PriceKey(to_key(price, self.digits).unwrap());
        match self.side {
            Side::Bid => {
                // priceより高い価格のものを削除
                // keyより低いものがBooksに残り、key以上のものが削除される
                self.books.split_off(&key);
            }
            Side::Ask => {
                // priceより低い価格のものを削除
                // key+1にし、key以上のものをBooksに代入し、keyより低いものが削除される
                self.books = self.books.split_off(&key.add(1));
            }
        }
    }

    /// 指定したsize以上のBookを取得
    pub fn wall(&self, size: Decimal) -> Option<&Book> {
        match self.side {
            Side::Bid => self.books.values().rev().find(|book| book.size > size),
            Side::Ask => self.books.values().find(|book| book.size > size),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Book {
    pub price: Decimal,
    pub size: Decimal,
}

impl Book {
    fn to_key(&self, digits: u64) -> PriceKey {
        match to_key(self.price, digits) {
            Ok(key) => PriceKey(key),
            Err(e) => {
                eprintln!("{}", e);
                PriceKey(0)
            }
        }
    }
}

fn to_key(price: Decimal, digits: u64) -> Result<i64, &'static str> {
    if digits == 0 {
        return price.floor().to_i64().ok_or("Conversion to i64 failed");
    }

    let iprice = price * Decimal::new(10, 0).powu(digits);
    iprice.to_i64().ok_or("Conversion to i64 failed")
}

// Convertibleトレイトの定義
pub trait Convertible: Sized {
    fn convert_to_decimal(self) -> Result<Decimal, &'static str>;
}

// 各型の定義に対して、変換を定義する
impl Convertible for u64 {
    fn convert_to_decimal(self) -> Result<Decimal, &'static str> {
        Decimal::from_u64(self).ok_or("Failed to convert u64 to Decimal")
    }
}

impl Convertible for i32 {
    fn convert_to_decimal(self) -> Result<Decimal, &'static str> {
        Decimal::from_i32(self).ok_or("Failed to convert i32 to Decimal")
    }
}

impl Convertible for i64 {
    fn convert_to_decimal(self) -> Result<Decimal, &'static str> {
        Ok(Decimal::from(self))
    }
}

impl Convertible for f32 {
    fn convert_to_decimal(self) -> Result<Decimal, &'static str> {
        Decimal::from_f32(self).ok_or("Failed to convert f32 to Decimal")
    }
}

impl Convertible for f64 {
    fn convert_to_decimal(self) -> Result<Decimal, &'static str> {
        Decimal::from_f64(self).ok_or("Failed to convert f64 to Decimal")
    }
}

impl Convertible for &str {
    fn convert_to_decimal(self) -> Result<Decimal, &'static str> {
        Decimal::from_str(self).map_err(|_| "Failed to convert str to Decimal")
    }
}

impl Convertible for String {
    fn convert_to_decimal(self) -> Result<Decimal, &'static str> {
        self.as_str().convert_to_decimal()
    }
}

impl Convertible for Decimal {
    fn convert_to_decimal(self) -> Result<Decimal, &'static str> {
        Ok(self)
    }
}

/// `converter()`はさまざまな型を`Book`構造体に変換します。
/// 許容される型:
/// - `i64`: 64ビット符号付き整数
/// - `i32`: 32ビット符号付き整数
/// - `u64`: 64ビット符号なし整数
/// - `f64`: 64ビット浮動小数点
/// - `f32`: 32ビット浮動小数点
/// - `&str`: 文字列参照
/// - `Decimal`: 小数値型
///
/// 変換に失敗した場合は`Err`を返します。
/// # Example
/// ```
/// use obmrs::models::converter;
/// let book = converter(100, 50).unwrap();
/// ```
pub fn converter<P: Convertible, S: Convertible>(price: P, size: S) -> Result<Book, &'static str> {
    let price = price.convert_to_decimal()?;
    let size = size.convert_to_decimal()?;

    Ok(Book { price, size })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_key() {
        let mut rng = rand::thread_rng();
        let count = 10;
        let set_place: u32 = 6;

        for _ in 0..count {
            let number = rng.gen_range(0..1000000000000000);

            let price = Decimal::new(number, set_place);
            assert_eq!(price.scale(), set_place);

            let float = price.to_f64().unwrap();
            assert_eq!(float, price.to_f64().unwrap());
            println!("{:?}", price);

            let book = Book {
                price,
                size: Decimal::new(1, 0),
            };
            let key = book.to_key(set_place as u64);
            assert_eq!(key, PriceKey(number));
        }
    }

    #[test]
    // bidのOrderbookを作成し、ソートを確かめる
    fn test_bid() {
        let set_place: u32 = 6;
        let mut order_board = OrderBoard::new(5, set_place as u64);

        let mut rng = rand::thread_rng();
        let count = 100;
        let max = Decimal::new(100000, 0);

        for _ in 0..count {
            let number = rng.gen_range(0..max.to_i64().unwrap());
            let volume = rng.gen_range(0..100);

            let price = Decimal::new(number, set_place);
            let book = Book {
                price,
                size: Decimal::new(volume, 1),
            };
            order_board.bids.push(book);
        }

        for _ in 0..count {
            let number = rng.gen_range(0..max.to_i64().unwrap());
            let volume = rng.gen_range(0..10);

            let price = Decimal::new(number, set_place);
            let book = Book {
                price,
                size: Decimal::new(volume, 1),
            };
            order_board.asks.push(book);
        }

        let mut last = Decimal::new(0, 0);
        for (key, book) in order_board.bids.books.iter() {
            assert!(last <= book.price);
            last = book.price;
            println!("bid: {:?} - {:?}, {}", key, book.price, book.size);
        }

        let mut last = Decimal::new(0, 0);
        for (key, book) in order_board.asks.books.iter() {
            assert!(last <= book.price);
            last = book.price;
            println!("ask: {:?} - {:?}, {}", key, book.price, book.size);
        }

        let wall = order_board.bids.wall(Decimal::new(50, 1)).unwrap();
        println!("wall: {}, {}", wall.price, wall.size);
    }

    // trim_innerの検証
    #[test]
    fn test_trim_inner() {
        // make orderboard
        let set_place: u32 = 6;
        let mut order_board = OrderBoard::new(50, set_place as u64);

        let mut rng = rand::thread_rng();
        let count = 100;

        for _ in 0..count {
            let number = rng.gen_range(0..1000);
            let volume = rng.gen_range(0..100);

            let price = Decimal::new(number, set_place);
            let book = Book {
                price,
                size: Decimal::new(volume, 1),
            };
            order_board.bids.push(book);
        }

        // bid配列からランダムなものをピックアップ
        let number = rng.gen_range(0..order_board.bids.books.len());
        let book = order_board.bids.books.values().nth(number).unwrap();

        for (key, book) in order_board.bids.books.iter() {
            println!("bid: {:?} - {:?}, {}", key, book.price, book.size);
        }

        println!("trim_inside_best_book: {:?}", book.price);

        // trim_inside_best_book
        order_board.bids.trim_inside_best_book(book.price);
        for (key, book) in order_board.bids.books.iter() {
            println!("bid: {:?} - {:?}, {}", key, book.price, book.size);
        }
    }

    // 変換ロジックをテスト
    #[test]
    fn test_converter() {
        let price = 1;
        let size = 1;
        let book = converter(price, size).unwrap();
        assert_eq!(book.price, Decimal::from(1));
        assert_eq!(book.size, Decimal::from(1));

        let price = 1.0;
        let size = 1.0;
        let book = converter(price, size).unwrap();
        assert_eq!(book.price, Decimal::from_f64(1.0).unwrap());
        assert_eq!(book.size, Decimal::from_f64(1.0).unwrap());

        let price = "1.0";
        let size = "1.0";
        let book = converter(price, size).unwrap();
        assert_eq!(book.price, Decimal::from_str("1.0").unwrap());
        assert_eq!(book.size, Decimal::from_str("1.0").unwrap());

        let price = Decimal::from(1);
        let size = Decimal::from(1);
        let book = converter(price, size).unwrap();
        assert_eq!(book.price, Decimal::from(1));
        assert_eq!(book.size, Decimal::from(1));
    }
}
