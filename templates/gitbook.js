// GitBook-compatible JavaScript

(function() {
    'use strict';

    // TOC toggle functionality
    var tocToggle = document.querySelector('.toc-toggle');
    var book = document.querySelector('.book');
    var pageToc = document.querySelector('.page-toc');

    function isMobileToc() {
        return window.innerWidth <= 768;
    }

    if (tocToggle && book && pageToc) {
        // Restore TOC state from localStorage (desktop only)
        if (!isMobileToc()) {
            var tocHidden = localStorage.getItem('guidebook-toc-hidden') === 'true';
            if (tocHidden) {
                book.classList.add('toc-hidden');
            }
        }

        tocToggle.addEventListener('click', function() {
            if (!isMobileToc()) {
                book.classList.toggle('toc-hidden');
                var isHidden = book.classList.contains('toc-hidden');
                localStorage.setItem('guidebook-toc-hidden', isHidden);
            }
        });

        // Handle resize
        window.addEventListener('resize', function() {
            if (isMobileToc()) {
                // On mobile, TOC is always hidden via CSS
            } else {
                // On desktop, restore saved state
                var tocHidden = localStorage.getItem('guidebook-toc-hidden') === 'true';
                if (tocHidden) {
                    book.classList.add('toc-hidden');
                } else {
                    book.classList.remove('toc-hidden');
                }
            }
        });
    }

    // TOC scroll spy - highlight current section
    function setupTocScrollSpy() {
        var tocLinks = document.querySelectorAll('.page-toc .toc-list a');
        if (tocLinks.length === 0) return;

        var headings = [];
        tocLinks.forEach(function(link) {
            var href = link.getAttribute('href');
            if (href && href.startsWith('#')) {
                var id = href.substring(1);
                try {
                    id = decodeURIComponent(id);
                } catch (e) {}
                var heading = document.getElementById(id);
                if (heading) {
                    headings.push({ element: heading, link: link });
                }
            }
        });

        if (headings.length === 0) return;

        function updateActiveLink() {
            var scrollTop = window.scrollY + 100; // Offset for fixed header
            var activeIndex = 0;

            for (var i = 0; i < headings.length; i++) {
                if (headings[i].element.offsetTop <= scrollTop) {
                    activeIndex = i;
                }
            }

            tocLinks.forEach(function(link) {
                link.parentElement.classList.remove('active');
            });
            headings[activeIndex].link.parentElement.classList.add('active');
        }

        window.addEventListener('scroll', updateActiveLink);
        updateActiveLink(); // Initial call
    }

    setupTocScrollSpy();

    // TOC link click handler - prevent base href issue
    function setupTocLinks() {
        var pageToc = document.querySelector('.page-toc');
        if (!pageToc) return;

        pageToc.addEventListener('click', function(e) {
            var link = e.target.closest('a');
            if (!link) return;

            var href = link.getAttribute('href');
            if (!href || !href.startsWith('#')) return;

            e.preventDefault();

            var id = href.substring(1);
            try {
                id = decodeURIComponent(id);
            } catch (ex) {}

            var target = document.getElementById(id);
            if (target) {
                rememberScroll();
                scrollHeadingIntoView(target, 'smooth');
                history.pushState(null, '', href);
            }
        });
    }

    setupTocLinks();

    // Back to top button
    var backToTop = document.querySelector('.back-to-top');
    if (backToTop) {
        window.addEventListener('scroll', function() {
            if (window.scrollY > 300) {
                backToTop.classList.add('visible');
            } else {
                backToTop.classList.remove('visible');
            }
        });

        backToTop.addEventListener('click', function(e) {
            e.preventDefault();
            window.scrollTo({ top: 0, behavior: 'smooth' });
        });
    }

    // Sidebar toggle
    var sidebarToggle = document.querySelector('.sidebar-toggle');
    var book = document.querySelector('.book');
    var bookSummary = document.querySelector('.book-summary');

    function isMobile() {
        return window.innerWidth <= 768;
    }

    var wasMobile = isMobile();

    if (sidebarToggle && book && bookSummary) {
        // Restore sidebar state from localStorage (desktop only)
        if (!isMobile()) {
            var sidebarHidden = localStorage.getItem('guidebook-sidebar-hidden') === 'true';
            if (sidebarHidden) {
                book.classList.add('sidebar-hidden');
            }
        }

        sidebarToggle.addEventListener('click', function() {
            if (isMobile()) {
                // Mobile: toggle .open on sidebar
                bookSummary.classList.toggle('open');
            } else {
                // Desktop: toggle .sidebar-hidden on book
                book.classList.add('sidebar-toggling');
                book.classList.toggle('sidebar-hidden');
                var isHidden = book.classList.contains('sidebar-hidden');
                localStorage.setItem('guidebook-sidebar-hidden', isHidden);
                setTimeout(function() {
                    book.classList.remove('sidebar-toggling');
                }, 350);
            }
        });

        // Close sidebar when clicking outside on mobile
        document.addEventListener('click', function(e) {
            if (isMobile() && bookSummary.classList.contains('open')) {
                if (!bookSummary.contains(e.target) && !sidebarToggle.contains(e.target)) {
                    bookSummary.classList.remove('open');
                }
            }
        });

        // Handle resize: switch between mobile and desktop modes
        window.addEventListener('resize', function() {
            var nowMobile = isMobile();
            if (wasMobile !== nowMobile) {
                if (nowMobile) {
                    // Switched to mobile: reset desktop state, close sidebar
                    book.classList.remove('sidebar-hidden');
                    book.classList.remove('sidebar-toggling');
                    bookSummary.classList.remove('open');
                } else {
                    // Switched to desktop: reset mobile state, restore desktop state
                    bookSummary.classList.remove('open');
                    var sidebarHidden = localStorage.getItem('guidebook-sidebar-hidden') === 'true';
                    if (sidebarHidden) {
                        book.classList.add('sidebar-hidden');
                    } else {
                        book.classList.remove('sidebar-hidden');
                    }
                }
                wasMobile = nowMobile;
            }
        });
    }

    // Smooth scroll for in-page anchor links.
    // Delegated on document so links in content replaced by SPA navigation are
    // covered too; bound per element they were left to the browser's own
    // fragment scroll, which lands the heading under a fixed header bar.
    document.addEventListener('click', function(e) {
        var anchor = e.target.closest ? e.target.closest('a[href*="#"]') : null;
        if (!anchor) return;

        // Handled by their own listeners: page TOC (setupTocLinks) and the
        // heading anchor icons. Running two handlers pushes two history
        // entries per click, so Back lands on a duplicate entry.
        if (anchor.closest('.page-toc') || anchor.classList.contains('heading-anchor')) return;

        // Let modifier clicks use the browser default (open in new tab, ...)
        if (e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return;

        var url;
        try {
            url = new URL(anchor.getAttribute('href'), location.href);
        } catch (ex) {
            return;
        }

        // Only same-page anchors: a link to another page must navigate normally
        if (url.pathname !== location.pathname || url.search !== location.search) return;
        if (!url.hash) return;

        var hash = url.hash.substring(1);
        // Decode URL-encoded anchor (e.g., %E3%83%87%E3%82%B6%E3%82%A4%E3%83%B3 -> デザイン)
        try {
            hash = decodeURIComponent(hash);
        } catch (ex) {
            // If decoding fails, use as-is
        }

        var target = document.getElementById(hash);
        if (target) {
            e.preventDefault();
            rememberScroll();
            scrollHeadingIntoView(target, 'smooth');
            // Update URL hash without triggering navigation
            history.pushState(null, '', '#' + encodeURIComponent(hash));
        }
    });

    // Heading anchors: hovering a heading reveals a link icon.
    // Clicking it puts #id in the URL and copies the full link for sharing,
    // so opening that link scrolls straight to the heading.
    var HEADING_ANCHOR_ICON = '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M7.775 3.275a.75.75 0 0 0 1.06 1.06l1.25-1.25a2 2 0 1 1 2.83 2.83l-2.5 2.5a2 2 0 0 1-2.83 0 .75.75 0 0 0-1.06 1.06 3.5 3.5 0 0 0 4.95 0l2.5-2.5a3.5 3.5 0 0 0-4.95-4.95l-1.25 1.25Zm-4.69 9.64a2 2 0 0 1 0-2.83l2.5-2.5a2 2 0 0 1 2.83 0 .75.75 0 0 0 1.06-1.06 3.5 3.5 0 0 0-4.95 0l-2.5 2.5a3.5 3.5 0 0 0 4.95 4.95l1.25-1.25a.75.75 0 0 0-1.06-1.06l-1.25 1.25a2 2 0 0 1-2.83 0Z"></path></svg>';

    function setupHeadingAnchors() {
        var section = document.querySelector('.markdown-section');
        if (!section) return;

        section.querySelectorAll('h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]').forEach(function(heading) {
            if (heading.querySelector('.heading-anchor')) return;

            var anchor = document.createElement('a');
            anchor.className = 'heading-anchor';
            anchor.href = '#' + encodeURIComponent(heading.id);
            // aria-label only: a title attribute would make the browser's native
            // tooltip appear ~1s after hover and overlap the "Link copied!" bubble
            anchor.setAttribute('aria-label', 'Copy link to this section');
            anchor.innerHTML = HEADING_ANCHOR_ICON;
            heading.appendChild(anchor);
        });
    }

    // Copy text to clipboard, falling back to execCommand on insecure origins
    // (navigator.clipboard is unavailable over plain http except on localhost)
    function copyTextToClipboard(text) {
        if (navigator.clipboard && window.isSecureContext) {
            return navigator.clipboard.writeText(text);
        }

        return new Promise(function(resolve, reject) {
            var textarea = document.createElement('textarea');
            textarea.value = text;
            textarea.setAttribute('readonly', '');
            textarea.style.position = 'fixed';
            textarea.style.top = '-1000px';
            textarea.style.opacity = '0';
            document.body.appendChild(textarea);
            textarea.select();
            var ok = false;
            try {
                ok = document.execCommand('copy');
            } catch (ex) {
                ok = false;
            }
            document.body.removeChild(textarea);
            ok ? resolve() : reject(new Error('copy failed'));
        });
    }

    // Gap kept above a heading scrolled into view, matching the CSS
    // scroll-margin-top used when the browser does the scrolling
    var HEADING_SCROLL_GAP = 24;

    // Fixed/sticky chrome guidebook renders itself. The bar we probe for lives
    // outside the generated markup, so ours must never be mistaken for it: at
    // narrow widths .sidebar-toggle (fixed, top 10px) sits over the content
    // column and chains onto the real bar, overstating its height by its own.
    var OWN_FIXED_UI = '.book-summary, .page-toc, .page-nav, .sidebar-toggle, ' +
        '.toc-toggle, .fontsettings-toolbar, .back-to-top';

    // The nearest fixed/sticky ancestor of el, or null
    function fixedLayerOf(el) {
        for (; el && el !== document.body; el = el.parentElement) {
            var position = getComputedStyle(el).position;
            if (position === 'fixed' || position === 'sticky') return el;
        }
        return null;
    }

    // True when the point sits inside a fixed/sticky layer (a site header bar,
    // a floating toolbar, ...) that would paint over the bubble
    function isCoveredByFixedLayer(x, y) {
        return !!fixedLayerOf(document.elementFromPoint(x, y));
    }

    // True when a fixed/sticky layer from outside the generated markup paints at
    // this point. The whole stack is inspected, not just the topmost element:
    // chrome of ours drawn over the bar must not hide it and cut the measurement
    // short either.
    function isCoveredBySiteLayer(x, y) {
        var stack = document.elementsFromPoint(x, y);
        for (var i = 0; i < stack.length; i++) {
            var layer = fixedLayerOf(stack[i]);
            if (layer && !layer.closest(OWN_FIXED_UI)) return true;
        }
        return false;
    }

    // Height of a bar pinned to the top of the viewport (a site header wrapped
    // around the book, ...) measured at x, so headings don't scroll underneath it.
    // Probed instead of configured: the bar lives outside the generated markup.
    function fixedTopInset(x) {
        if (!isCoveredBySiteLayer(x, 0)) return 0;

        // Coarse scan for the first uncovered point, then narrow to 1px: a step
        // wider than that lands a few pixels off depending on where the probe
        // falls relative to the bar's border
        var covered = 0;
        var uncovered = -1;
        for (var y = 8; y <= 240; y += 8) {
            if (!isCoveredBySiteLayer(x, y)) {
                uncovered = y;
                break;
            }
            covered = y;
        }
        if (uncovered === -1) return covered;

        while (uncovered - covered > 1) {
            var mid = (covered + uncovered) >> 1;
            if (isCoveredBySiteLayer(x, mid)) {
                covered = mid;
            } else {
                uncovered = mid;
            }
        }
        return uncovered;
    }

    // Scroll a heading just below any fixed top bar, with room for the bubble
    function scrollHeadingIntoView(heading, behavior) {
        var rect = heading.getBoundingClientRect();
        var probeX = rect.left + Math.min(rect.width, 40) / 2;
        var top = window.scrollY + rect.top - fixedTopInset(probeX) - HEADING_SCROLL_GAP;
        window.scrollTo({ top: Math.max(0, top), behavior: behavior || 'auto' });
    }

    // The bubble sits above the icon, but flips below when that space is clipped
    // by the viewport or covered by a fixed header
    function positionAnchorTip(anchor) {
        anchor.classList.remove('tip-below');

        var rect = anchor.getBoundingClientRect();
        var tipHeight = 25; // 12px line + 5px padding top/bottom + 3px gap
        var probeY = rect.top - tipHeight / 2;

        if (probeY < 0 || isCoveredByFixedLayer(rect.left + rect.width / 2, probeY)) {
            anchor.classList.add('tip-below');
        }
    }

    function flashAnchorTip(anchor, message) {
        anchor.setAttribute('data-tip', message);
        positionAnchorTip(anchor);
        anchor.classList.add('copied');

        // The same click smooth-scrolls the heading, so what sits above the icon
        // keeps changing: re-check every frame while the bubble is visible
        var track = function() {
            if (!anchor.classList.contains('copied')) return;
            positionAnchorTip(anchor);
            requestAnimationFrame(track);
        };
        requestAnimationFrame(track);

        if (anchor._tipTimer) clearTimeout(anchor._tipTimer);
        anchor._tipTimer = setTimeout(function() {
            anchor.classList.remove('copied');
            anchor.classList.remove('tip-below');
            anchor.removeAttribute('data-tip');
        }, 1500);
    }

    // Delegated on document so it keeps working after SPA navigation replaces content
    document.addEventListener('click', function(e) {
        var anchor = e.target.closest ? e.target.closest('.heading-anchor') : null;
        if (!anchor) return;

        // Let modifier clicks fall through to browser default (open in new tab, etc.)
        if (e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return;

        var heading = anchor.closest('h1, h2, h3, h4, h5, h6');
        if (!heading || !heading.id) return;

        e.preventDefault();

        var hash = '#' + encodeURIComponent(heading.id);
        rememberScroll();
        history.pushState(null, '', location.pathname + location.search + hash);
        scrollHeadingIntoView(heading, 'smooth');

        copyTextToClipboard(location.href)
            .then(function() {
                flashAnchorTip(anchor, 'Link copied!');
            })
            .catch(function() {
                flashAnchorTip(anchor, 'Press Ctrl/Cmd+C to copy');
            });
    });

    // SPA-like navigation for sidebar links
    // Get base URL for resolving relative links (e.g., /jp/)
    function getBaseUrl() {
        var base = document.querySelector('base');
        if (base && base.href) {
            return base.href;
        }
        return window.location.href.replace(/[^/]*$/, '');
    }

    // Prevent rapid navigation
    var isNavigating = false;

    // Convert hrefs to absolute URLs and set target="_blank" for external pages
    function normalizeLinks(container) {
        if (!container) return;

        container.querySelectorAll('a[href]').forEach(function(link) {
            var href = link.getAttribute('href');
            if (!href || href.startsWith('#')) return;

            // Convert relative href to absolute URL
            var absoluteUrl;
            if (href.startsWith('http')) {
                absoluteUrl = href;
            } else if (href.startsWith('/')) {
                absoluteUrl = new URL(href, window.location.origin).href;
            } else {
                absoluteUrl = new URL(href, getBaseUrl()).href;
            }
            link.setAttribute('href', absoluteUrl);

            // Determine if link should open in new tab
            var isSameOrigin = absoluteUrl.startsWith(window.location.origin);
            if (!isSameOrigin) {
                // Different domain - always open in new tab
                link.setAttribute('target', '_blank');
            } else if (!absoluteUrl.endsWith('.html') && !absoluteUrl.includes('.html#')) {
                // Same domain but not .html - open in new tab (e.g., Swagger UI at /api-docs/)
                link.setAttribute('target', '_blank');
            }
            // Same domain + .html - no target="_blank", SPA navigation will handle
        });
    }

    // Convert sidebar hrefs to absolute URLs on initial load
    // This ensures right-click > "Open in new tab" works correctly after SPA navigation
    // Also set target="_blank" for external pages (non-.html links like Swagger UI)
    function normalizeSidebarHrefs() {
        normalizeLinks(document.querySelector('.book-summary'));
    }

    // Normalize links in main content area
    // Called on page load and after SPA navigation
    function normalizeContentLinks() {
        normalizeLinks(document.querySelector('.markdown-section'));
    }

    function setupSpaNavigation() {
        var sidebar = document.querySelector('.book-summary');
        if (!sidebar) return;

        // Normalize hrefs to absolute URLs for correct browser-native behavior
        normalizeSidebarHrefs();

        sidebar.addEventListener('click', function(e) {
            var link = e.target.closest('a');
            if (!link) return;

            // Note: expandable items with children are handled by collapsible.js
            // - Arrow click: collapsible.js calls stopImmediatePropagation(), so this handler won't run
            // - Text click: collapsible.js returns without stopping, so this handler runs for SPA navigation

            var href = link.getAttribute('href');
            if (!href || href.startsWith('#')) return;

            // Allow modifier key clicks to use browser default behavior (open in new tab)
            if (e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) {
                return;
            }

            // Skip external links and links that open in new tab
            if (href.startsWith('http') && !href.startsWith(window.location.origin)) {
                return;
            }
            if (link.getAttribute('target') === '_blank') {
                return;
            }

            e.preventDefault();
            if (isNavigating) return;

            // For search results, pass null to trigger sidebar scroll to active item
            // Also hide search results after navigation
            var isSearchResult = link.classList.contains('search-result-item');
            if (isSearchResult) {
                var searchResults = document.querySelector('.search-results');
                if (searchResults) {
                    searchResults.classList.remove('visible');
                }
            }

            loadPage(href, isSearchResult ? null : link);
        });
    }

    // Setup page navigation (prev/next buttons)
    function setupPageNavigation() {
        document.querySelectorAll('.page-nav').forEach(function(nav) {
            nav.addEventListener('click', function(e) {
                // Allow modifier key clicks to use browser default behavior (open in new tab)
                if (e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) {
                    return;
                }

                e.preventDefault();
                if (isNavigating) return;

                var href = this.getAttribute('href');
                if (!href) return;

                loadPage(href, null);
            });
        });
    }

    function loadPage(url, clickedLink, restoreState) {
        if (isNavigating) return;
        isNavigating = true;

        // Add loading state
        document.body.classList.add('loading');

        var absoluteUrl = new URL(url, getBaseUrl()).href;

        // Extract hash from URL if present
        var hashIndex = url.indexOf('#');
        var hash = hashIndex !== -1 ? url.substring(hashIndex + 1) : null;

        fetch(absoluteUrl)
            .then(function(response) {
                if (!response.ok) throw new Error('Page not found');
                return response.text();
            })
            .then(function(html) {
                var parser = new DOMParser();
                var doc = parser.parseFromString(html, 'text/html');

                // Update content
                var newContent = doc.querySelector('.markdown-section');
                var currentContent = document.querySelector('.markdown-section');

                // If the new page doesn't have .markdown-section (e.g., Swagger UI, external page),
                // open in a new tab so user can return to the documentation
                if (!newContent) {
                    isNavigating = false;
                    document.body.classList.remove('loading');
                    window.open(absoluteUrl, '_blank');
                    return;
                }

                if (currentContent) {
                    currentContent.innerHTML = newContent.innerHTML;
                }

                // Update title
                var newTitle = doc.querySelector('title');
                if (newTitle) {
                    document.title = newTitle.textContent;
                }

                // Update active state in sidebar (don't replace HTML to preserve expanded state)
                document.querySelectorAll('.book-summary .chapter.active').forEach(function(ch) {
                    ch.classList.remove('active');
                });

                // Find and mark new active item
                // Use absoluteUrl for matching since sidebar links are normalized to absolute URLs
                var newActiveHref = absoluteUrl.split('#')[0];
                document.querySelectorAll('.book-summary .chapter a').forEach(function(link) {
                    var href = link.getAttribute('href');
                    if (href === newActiveHref) {
                        var chapter = link.closest('.chapter');
                        if (chapter) {
                            chapter.classList.add('active');
                            // Expand parent chapters
                            var parent = chapter.parentElement;
                            while (parent) {
                                if (parent.classList && parent.classList.contains('chapter')) {
                                    parent.classList.add('expanded');
                                }
                                parent = parent.parentElement;
                            }
                        }
                    }
                });

                // Update URL (use absolute URL to avoid relative path issues with SPA navigation)
                history.pushState(null, '', absoluteUrl);
                currentPage = pageKey(absoluteUrl);

                // Scroll to hash anchor or top
                if (hash) {
                    try {
                        var decodedHash = decodeURIComponent(hash);
                        var target = document.getElementById(decodedHash);
                        if (target) {
                            setTimeout(function() {
                                scrollHeadingIntoView(target);
                            }, 50);
                        } else {
                            window.scrollTo(0, 0);
                        }
                    } catch (ex) {
                        window.scrollTo(0, 0);
                    }
                } else if (restoreState) {
                    // Back/forward to a page without an anchor: put the reader
                    // back where they were, not at the top. Re-asserted once the
                    // page is fully laid out, since highlight.js/mermaid grow it
                    // afterwards and an early scroll would be clamped short
                    restoreScroll(restoreState);
                    setTimeout(function() {
                        restoreScroll(restoreState);
                    }, 250);
                } else {
                    window.scrollTo(0, 0);
                }

                // Re-init mermaid if present
                if (typeof mermaid !== 'undefined') {
                    mermaid.init(undefined, '.markdown-section .mermaid');
                }

                // Re-apply syntax highlighting
                if (typeof hljs !== 'undefined') {
                    document.querySelectorAll('.markdown-section pre code').forEach(function(block) {
                        hljs.highlightElement(block);
                    });
                }

                // Re-apply font settings (theme styles for tables/headings)
                if (window.guidebookFontsettings && window.guidebookFontsettings.reapply) {
                    window.guidebookFontsettings.reapply();
                }

                // Update TOC from new page
                var newToc = doc.querySelector('.page-toc');
                var currentToc = document.querySelector('.page-toc');
                var newTocToggle = doc.querySelector('.toc-toggle');
                var currentTocToggle = document.querySelector('.toc-toggle');

                if (currentToc) currentToc.remove();
                if (currentTocToggle) currentTocToggle.remove();

                if (newToc) {
                    var bookBody = document.querySelector('.book-body');
                    if (bookBody) {
                        var tocClone = newToc.cloneNode(true);
                        bookBody.insertBefore(tocClone, bookBody.querySelector('.body-inner'));
                        if (newTocToggle) {
                            var toggleClone = newTocToggle.cloneNode(true);
                            bookBody.insertBefore(toggleClone, tocClone);
                            // Re-setup toggle handler
                            toggleClone.addEventListener('click', function() {
                                if (window.innerWidth > 768) {
                                    document.querySelector('.book').classList.toggle('toc-hidden');
                                    var isHidden = document.querySelector('.book').classList.contains('toc-hidden');
                                    localStorage.setItem('guidebook-toc-hidden', isHidden);
                                }
                            });
                        }
                        // Re-setup scroll spy and TOC links
                        setupTocScrollSpy();
                        setupTocLinks();
                    }
                }

                // Update prev/next navigation buttons
                var newPrev = doc.querySelector('.page-nav.prev');
                var newNext = doc.querySelector('.page-nav.next');
                var currentPrev = document.querySelector('.page-nav.prev');
                var currentNext = document.querySelector('.page-nav.next');

                if (currentPrev) currentPrev.remove();
                if (currentNext) currentNext.remove();

                var bodyInner = document.querySelector('.body-inner');
                if (bodyInner) {
                    if (newPrev) {
                        var prevClone = newPrev.cloneNode(true);
                        bodyInner.insertBefore(prevClone, bodyInner.firstChild);
                    }
                    if (newNext) {
                        var nextClone = newNext.cloneNode(true);
                        bodyInner.insertBefore(nextClone, bodyInner.querySelector('.page-wrapper'));
                    }
                    // Re-setup page navigation for new buttons
                    setupPageNavigation();
                }

                // Normalize links in updated content (set target="_blank" for external pages)
                normalizeContentLinks();

                // Re-add heading anchor icons to the new content
                setupHeadingAnchors();

                // Scroll sidebar to show active item only for page navigation (prev/next buttons)
                // Not for sidebar clicks - user already knows where they clicked
                if (!clickedLink) {
                    setTimeout(function() {
                        scrollSidebarToActive();
                    }, 100);
                }

                // Reset navigation state
                isNavigating = false;
                document.body.classList.remove('loading');
            })
            .catch(function(err) {
                console.error('Navigation error:', err);
                isNavigating = false;
                document.body.classList.remove('loading');
                window.location.href = url;
            });
    }

    // Headings are scrolled clear of any fixed header, so the browser's own
    // restoration must not race us on back/forward: it alternates between
    // restoring the saved offset and doing a plain fragment scroll (which knows
    // nothing about the header). We remember the offset per history entry instead.
    if ('scrollRestoration' in history) {
        history.scrollRestoration = 'manual';
    }

    var rememberScrollTimer = null;

    // Our offset merged into whatever the current entry already holds. The book
    // can share its document (and so its history entry) with a host application
    // — Guidebook Cloud wraps it in a page of its own — and replacing the state
    // outright would drop what that host put there.
    function stateWithScroll(state) {
        var next = {};
        if (state && typeof state === 'object') {
            Object.keys(state).forEach(function(key) { next[key] = state[key]; });
        }
        next.scrollY = window.scrollY;
        return next;
    }

    // Store the reader's position in the current history entry.
    // Writes are coalesced and skipped for tiny moves: browsers rate-limit
    // history writes (Safari throws after ~100 in 30s)
    function rememberScroll() {
        var state = history.state;
        if (state && Math.abs(state.scrollY - window.scrollY) < 16) return;
        try {
            history.replaceState(stateWithScroll(state), '');
        } catch (ex) {
            // Rate-limited: the next scroll tick will try again
        }
    }

    function restoreScroll(state) {
        window.scrollTo(0, state && typeof state.scrollY === 'number' ? state.scrollY : 0);
    }

    window.addEventListener('scroll', function() {
        if (rememberScrollTimer) return;
        rememberScrollTimer = setTimeout(function() {
            rememberScrollTimer = null;
            rememberScroll();
        }, 500);
    });

    // Scroll to the element a location.hash points at (no-op without a hash)
    function scrollToHash(hash, behavior) {
        if (!hash) return;

        var id = hash.charAt(0) === '#' ? hash.substring(1) : hash;
        // Decode URL-encoded anchors (e.g. %E5%A4%89%E6%95%B0 -> 変数)
        try {
            id = decodeURIComponent(id);
        } catch (ex) {
            // If decoding fails, use as-is
        }

        var target = document.getElementById(id);
        if (target) scrollHeadingIntoView(target, behavior);
    }

    // Page currently rendered in the content area, so back/forward can tell a
    // hash-only history entry (anchor click) from a real page navigation.
    // Includes the query string: two URLs differing only there are distinct pages.
    function pageKey(url) {
        var loc = url ? new URL(url) : location;
        return loc.pathname + loc.search;
    }

    var currentPage = pageKey();

    // Handle browser back/forward
    window.addEventListener('popstate', function(e) {
        // Same page, only the #anchor changed: scroll instead of refetching
        if (pageKey() === currentPage) {
            if (location.hash) {
                scrollToHash(location.hash);
            } else {
                // Back to an entry without an anchor: return to where the reader was
                restoreScroll(e.state);
            }
            return;
        }
        loadPage(location.pathname + location.search + location.hash, null, e.state);
    });

    setupSpaNavigation();
    setupPageNavigation();
    normalizeContentLinks();
    setupHeadingAnchors();

    // Handle initial page load with hash anchor
    function scrollToHashOnLoad() {
        // Use setTimeout to ensure layout is complete after all resources load
        setTimeout(function() {
            if (window.location.hash) {
                scrollToHash(window.location.hash);
            } else if (history.state && typeof history.state.scrollY === 'number') {
                // Reload without an anchor: scrollRestoration is 'manual', so the
                // position remembered for this entry is restored here instead.
                // Re-asserted like loadPage() does, since highlight.js/mermaid
                // grow the page afterwards and an early scroll would be clamped
                var state = history.state;
                restoreScroll(state);
                setTimeout(function() {
                    restoreScroll(state);
                }, 250);
            }
        }, 100);
    }

    // Use 'load' event to ensure all resources (images, CSS) are loaded
    if (document.readyState === 'complete') {
        scrollToHashOnLoad();
    } else {
        window.addEventListener('load', scrollToHashOnLoad);
    }

    // Initialize syntax highlighting on page load
    if (typeof hljs !== 'undefined') {
        hljs.highlightAll();
    }

    // Scroll sidebar to show active item centered
    function scrollSidebarToActive() {
        var sidebar = document.querySelector('.book-summary');
        var activeItem = document.querySelector('.book-summary .chapter.active');

        if (!sidebar || !activeItem) return;

        // Get the active item's link element for more precise positioning
        var activeLink = activeItem.querySelector('a') || activeItem;

        // Calculate position to center the active item in the sidebar
        var sidebarRect = sidebar.getBoundingClientRect();
        var activeRect = activeLink.getBoundingClientRect();

        // Calculate the offset needed to center the active item
        var sidebarScrollTop = sidebar.scrollTop;
        var activeOffsetTop = activeRect.top - sidebarRect.top + sidebarScrollTop;
        var sidebarHeight = sidebar.clientHeight;
        var activeHeight = activeRect.height;

        // Scroll so that active item is centered (minus half sidebar height, plus half item height)
        var targetScrollTop = activeOffsetTop - (sidebarHeight / 2) + (activeHeight / 2);

        // Clamp to valid scroll range
        var maxScroll = sidebar.scrollHeight - sidebarHeight;
        targetScrollTop = Math.max(0, Math.min(targetScrollTop, maxScroll));

        sidebar.scrollTop = targetScrollTop;
    }

    // Scroll sidebar on initial page load
    if (document.readyState === 'complete') {
        scrollSidebarToActive();
    } else {
        window.addEventListener('load', scrollSidebarToActive);
    }

    // Expose for use after SPA navigation
    window.scrollSidebarToActive = scrollSidebarToActive;

})();
