# SVG 処理

guidebook は `book.json` で制御される 2 つの SVG 処理モードを提供します。

## インライン SVG の外部化

HTML 内のインライン `<svg>` 要素を個別の `.svg` ファイルに抽出：

```json
{
    "externalize_svg": true
}
```

インライン SVG は抽出されたファイルを指す `<img>` タグに置換されます。アイコン SVG（`fill="currentColor"` を持つもの）は動的な色の動作を保持するためインラインのまま維持されます。

## SVG ファイルのインライン化

外部 SVG ファイルを HTML に直接埋め込み：

```json
{
    "inline_svg": true
}
```

外部の `<img src="*.svg">` 参照が SVG コンテンツのインラインに置換されます。アイコン SVG は `<img>` タグのまま維持されます。

## 使い分け

| オプション | ユースケース |
|-----------|-------------|
| `externalize_svg` | HTML サイズを削減、SVG のブラウザキャッシュを有効化 |
| `inline_svg` | 追加の HTTP リクエストを排除、SVG の CSS スタイリングを有効化 |
