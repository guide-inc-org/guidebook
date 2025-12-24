// Collapsible chapters functionality with event delegation

(function() {
    'use strict';

    var sidebar = document.querySelector('.book-summary');
    if (!sidebar) return;

    // Use event delegation so it works after SPA navigation
    sidebar.addEventListener('click', function(e) {
        var link = e.target.closest('a');
        var titleSpan = e.target.closest('.chapter-title');
        var chapter = e.target.closest('.chapter.expandable');

        if (!chapter) return;

        var articles = chapter.querySelector('.articles');
        if (!articles) return;

        // If clicked on chapter-title (no link), toggle expand
        if (titleSpan && !link) {
            e.preventDefault();
            e.stopImmediatePropagation();
            chapter.classList.toggle('expanded');
            return;
        }

        // If clicked on link, check if it's on the arrow area (left 25px)
        if (link) {
            var rect = link.getBoundingClientRect();
            var clickX = e.clientX - rect.left;

            if (clickX < 25) {
                e.preventDefault();
                e.stopImmediatePropagation();
                chapter.classList.toggle('expanded');
            }
        }
    });
})();
