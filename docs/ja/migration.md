# HonKit からの移行

guidebook は HonKit のドロップイン置き換えとして設計されています。

## クイック移行

1. guidebook をインストール：
   ```bash
   curl -fsSL https://raw.githubusercontent.com/guide-inc-org/guidebook/main/install.sh | sh
   ```

2. コマンドを置き換え：
   ```bash
   # 以前（HonKit）
   npx honkit build
   npx honkit serve

   # 以後（guidebook）
   guidebook build
   guidebook serve
   ```

これだけです！設定の変更は不要です。

## 互換性

### サポート対象

- `book.json` 設定
- `SUMMARY.md` フォーマット
- `LANGS.md` 多言語対応
- Markdown レンダリング
- カスタムスタイル
- ほとんどのプラグイン（collapsible-chapters、back-to-top-button）

### 未サポート

- PDF/EPUB エクスポート（Web のみ）
- JavaScript プラグイン（guidebook は組み込み Rust 実装を使用）
- GitBook レガシーフォーマット

## 移行のメリット

| 機能 | HonKit | guidebook |
|------|--------|-----------|
| インストール | Node.js + npm | 単一バイナリ |
| ビルド速度 | 秒単位 | ミリ秒単位 |
| 依存関係 | 多数 | なし |
| 自己更新 | 手動 | `guidebook update` |

## CI/CD の移行

### 以前（HonKit）

```yaml
- uses: actions/setup-node@v4
- run: npm install -g honkit
- run: honkit build
```

### 以後（guidebook）

```yaml
- run: |
    curl -sL https://github.com/guide-inc-org/guidebook/releases/latest/download/guidebook-linux-x86_64.tar.gz | tar xz
    ./guidebook build
```
