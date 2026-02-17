# 数式（KaTeX）

KaTeX を使用して数学の公式をレンダリングします。

## 有効化

`book.json` で `math: true` を設定：

```json
{
    "math": true
}
```

## インライン数式

`$...$` でインライン数式：

```markdown
有名な方程式 $E = mc^2$ です。
```

## ディスプレイ数式

`$$...$$` でディスプレイ（ブロック）数式：

```markdown
$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
```

## KaTeX リファレンス

サポートされる関数と構文については [KaTeX ドキュメント](https://katex.org/docs/supported) を参照してください。
