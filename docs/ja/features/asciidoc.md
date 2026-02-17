# AsciiDoc サポート

guidebook は Markdown と並行して AsciiDoc（`.adoc` および `.asciidoc`）ファイルをサポートします。

## 使い方

`SUMMARY.md` で Markdown ファイルと同様に `.adoc` ファイルを参照：

```markdown
# 目次

* [はじめに](README.md)
* [AsciiDoc チャプター](chapter.adoc)
```

## サポートされる構文

guidebook の内蔵レンダラーで AsciiDoc をレンダリングします：

- 見出し（`= タイトル`、`== セクション` 等）
- 段落とテキスト整形（`*太字*`、`_イタリック_`、`` `コード` ``）
- リスト（順序付き・順序なし）
- コードブロック
- リンクとクロスリファレンス
- テーブル
- 注釈（NOTE、TIP、WARNING、IMPORTANT、CAUTION）

## フォーマットの混在

同じブック内で Markdown と AsciiDoc ファイルを自由に混在できます。各ファイルは拡張子に基づいて適切なパーサーでレンダリングされます。
