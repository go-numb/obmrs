# obmrs for rust [This is a project under review. since: 2024/10/07]

参加者として、取引所配信のOrderbookを受信し、保持するための構造体を作成します。
この構造体は、OrderBoardという名前で、Orderbookのbidsとasksを価格ソートされたマップとして保持及び入出力します。
価格に対する総体を保存するものであり、価格に対する注文それぞれを保存するものではありません。

## Futures
- [x] new
- [x] converter: price as any, size as any to Result<Book, _>
- [x] push, extend
- [x] best: best_ask, best_bidを取得します
- [x] wall(size): 指定サイズ以上のBookを取得します
- [x] trim_inside_best_book(best_ask, best_bid): best値の内側を削除します
_

## via golang client
[go-obm @go-numb](https://github.com/go-numb/go-obm)

## Author

[@_numbP](https://twitter.com/_numbP)

## License

[MIT](https://github.com/go-numb/obmrs/blob/master/LICENSE)