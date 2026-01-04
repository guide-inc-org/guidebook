# Search

guidebook includes built-in full-text search.

## How It Works

A search index is generated during build. Users can search using the search box in the sidebar.

## Features

- Full-text search across all pages
- Instant results as you type
- Keyboard navigation (arrow keys, Enter)
- Highlights matching content

## Search Index

The search index is stored in `search_index.json` in the output directory.

For large books, the initial build may take a few seconds to generate the index.

## Hot Reload Note

During development (`guidebook serve`), the search index is not regenerated on every change to improve rebuild speed. Restart the server to update the search index.
