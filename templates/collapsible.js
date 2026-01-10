// Collapsible chapters functionality with localStorage persistence

(function() {
    'use strict';

    var STORAGE_KEY = 'guidebook-expanded-chapters';
    var sidebar = document.querySelector('.book-summary');
    if (!sidebar) return;

    // Get stored expanded state
    function getExpandedState() {
        try {
            var stored = localStorage.getItem(STORAGE_KEY);
            return stored ? JSON.parse(stored) : {};
        } catch (e) {
            return {};
        }
    }

    // Save expanded state
    function saveExpandedState(state) {
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
        } catch (e) {}
    }

    // Get unique identifier for a chapter (based on its link href or title)
    function getChapterId(chapter) {
        var link = chapter.querySelector(':scope > a');
        if (link) {
            return link.getAttribute('href');
        }
        var title = chapter.querySelector(':scope > .chapter-title');
        if (title) {
            return 'title:' + title.textContent.trim();
        }
        return null;
    }

    // Restore expanded state from localStorage
    function restoreExpandedState() {
        var state = getExpandedState();
        var chapters = sidebar.querySelectorAll('.chapter.expandable');

        chapters.forEach(function(chapter) {
            var id = getChapterId(chapter);
            if (id && state[id] !== undefined) {
                if (state[id]) {
                    chapter.classList.add('expanded');
                } else {
                    // Only close if not in the active path
                    if (!chapter.classList.contains('active') &&
                        !chapter.querySelector('.chapter.active')) {
                        chapter.classList.remove('expanded');
                    }
                }
            }
        });
    }

    // Save current expanded state to localStorage
    function saveCurrentState() {
        var state = {};
        var chapters = sidebar.querySelectorAll('.chapter.expandable');

        chapters.forEach(function(chapter) {
            var id = getChapterId(chapter);
            if (id) {
                state[id] = chapter.classList.contains('expanded');
            }
        });

        saveExpandedState(state);
    }

    // Restore state on page load
    restoreExpandedState();

    // Toggle icon click area width (includes padding-left where arrow icon is displayed)
    var TOGGLE_ICON_WIDTH = 70;

    // Use event delegation so it works after SPA navigation
    sidebar.addEventListener('click', function(e) {
        var link = e.target.closest('a');
        var titleSpan = e.target.closest('.chapter-title');

        // Get the direct parent chapter of the clicked element
        var directChapter = (link || titleSpan)?.closest('.chapter');
        if (!directChapter) return;

        // Only handle if the direct chapter is expandable
        if (!directChapter.classList.contains('expandable')) return;

        var articles = directChapter.querySelector(':scope > .articles');
        if (!articles) return;

        // If clicked on chapter-title (no link), toggle expand
        if (titleSpan && !link) {
            e.preventDefault();
            e.stopImmediatePropagation();
            directChapter.classList.toggle('expanded');
            saveCurrentState();
            return;
        }

        // If clicked on link with children - check click position
        // Left side (arrow icon area) = toggle, Right side (text) = navigate
        if (link) {
            var linkRect = link.getBoundingClientRect();
            var clickX = e.clientX - linkRect.left;

            // If clicked on the left arrow area, toggle expand/collapse
            if (clickX < TOGGLE_ICON_WIDTH) {
                e.preventDefault();
                e.stopImmediatePropagation();
                directChapter.classList.toggle('expanded');
                saveCurrentState();
                return;
            }
            // Otherwise, allow normal link navigation (do nothing, let browser handle)
            return;
        }
    });
})();
