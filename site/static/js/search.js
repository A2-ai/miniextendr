function debounce(fn, delay) {
    let timer;
    return function() {
        clearTimeout(timer);
        timer = setTimeout(() => fn.apply(this, arguments), delay);
    };
}

function initSearch() {
    const input = document.getElementById("search");
    if (!input) return;

    const resultsContainer = document.querySelector(".search-results");
    const resultsItems = document.querySelector(".search-results__items");
    let index;

    fetch(new URL("search_index.en.json", window.location.origin + window.location.pathname.replace(/[^/]*$/, "")))
        .then(r => r.json())
        .then(data => {
            index = elasticlunr.Index.load(data);
        })
        .catch(() => {});

    const doSearch = debounce(function() {
        const query = input.value.trim();
        if (!query || !index) {
            resultsContainer.classList.remove("active");
            return;
        }

        const results = index.search(query, { expand: true });
        resultsItems.innerHTML = "";

        if (results.length === 0) {
            resultsItems.innerHTML = "<p style='padding:0.5rem 0.75rem;color:#565f89'>No results</p>";
        } else {
            results.slice(0, 10).forEach(r => {
                const doc = index.documentStore.getDoc(r.ref);
                const a = document.createElement("a");
                a.href = r.ref;
                a.textContent = doc.title || r.ref;
                resultsItems.appendChild(a);
            });
        }
        resultsContainer.classList.add("active");
    }, 200);

    input.addEventListener("input", doSearch);
    input.addEventListener("focus", doSearch);
    document.addEventListener("click", function(e) {
        if (!e.target.closest(".search-container")) {
            resultsContainer.classList.remove("active");
        }
    });
}

document.addEventListener("DOMContentLoaded", initSearch);
