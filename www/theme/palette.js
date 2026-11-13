/* Ctrl-K quick search.
 *
 * Identity and documentation site
 * (`.cursor/plans/voltaic-identity-and-site.plan.md`) wave 7.
 *
 * **This builds no index.** mdBook already emits one, and exposes it as
 * `window.search` with `doc_urls[]` and `index.documentStore.docs[ref]`
 * carrying `{title, breadcrumbs, body}`. A second index would be a second
 * thing to keep in step with the book, so this reads mdBook's.
 *
 * It does its own *ranking*, though, because a command palette and a search
 * page want different things: the palette is used to jump to a heading whose
 * name you half-remember, so a title prefix must beat a body hit by a wide
 * margin, and a subsequence match on the title ("expop" -> "Expressions and
 * operators") has to work at all. elasticlunr's BM25 over bodies is the wrong
 * instrument for that, and skipping it also means no coupling to its API.
 *
 * mdBook content-hashes its asset filenames (`searchindex-3fa690f8.js`), so the
 * index is never referenced by path — `window.path_to_searchindex_js`, which
 * mdBook writes into the page, is.
 *
 * `/` still opens mdBook's own search bar. This is additive.
 */
(function () {
	"use strict";

	var MAX = 24;
	var box,
		input,
		list,
		empty,
		palette,
		items = [],
		selected = 0,
		loading = false;

	function el(tag, attrs, kids) {
		var n = document.createElement(tag);
		Object.keys(attrs || {}).forEach(function (k) {
			if (k === "text") n.textContent = attrs[k];
			else n.setAttribute(k, attrs[k]);
		});
		(kids || []).forEach(function (c) {
			n.appendChild(c);
		});
		return n;
	}

	function build() {
		input = el("input", {
			id: "vlt-input",
			type: "text",
			autocomplete: "off",
			spellcheck: "false",
			placeholder: "Search the book…",
			"aria-label": "Search the book",
			"aria-controls": "vlt-results",
			"aria-autocomplete": "list",
			role: "combobox",
			"aria-expanded": "true",
		});

		// The pile, at three plates, as the field's mark.
		var icon = document.createElementNS("http://www.w3.org/2000/svg", "svg");
		icon.setAttribute("viewBox", "0 0 64 64");
		icon.setAttribute("aria-hidden", "true");
		[
			[29, 8, 6, 5, 1.5, "m1"],
			[10, 16, 44, 10, 3, "m1"],
			[13, 29, 38, 10, 3, "m2"],
			[10, 42, 44, 10, 3, "m1"],
		].forEach(function (r) {
			var rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
			rect.setAttribute("x", r[0]);
			rect.setAttribute("y", r[1]);
			rect.setAttribute("width", r[2]);
			rect.setAttribute("height", r[3]);
			rect.setAttribute("rx", r[4]);
			rect.setAttribute(
				"fill",
				r[5] === "m1" ? "var(--voltaic-copper)" : "var(--voltaic-zinc)",
			);
			icon.appendChild(rect);
		});

		list = el("ul", {
			id: "vlt-results",
			role: "listbox",
			"aria-label": "Results",
		});
		empty = el("div", { class: "vlt-empty", text: "Type to search." });

		box = el("div", { class: "vlt-box", role: "document" }, [
			el("div", { class: "vlt-field" }, [
				icon,
				input,
				el("span", { class: "vlt-kbd", text: "ESC" }),
			]),
			list,
			empty,
			el("div", { class: "vlt-foot" }, [
				el("span", { text: "↑↓ move" }),
				el("span", { text: "⏎ open" }),
				el("span", { text: "esc close" }),
				el("span", { text: "/ full search" }),
			]),
		]);

		palette = el(
			"div",
			{
				id: "vlt-palette",
				role: "dialog",
				"aria-modal": "true",
				"aria-label": "Quick search",
				hidden: "hidden",
			},
			[box],
		);

		document.body.appendChild(palette);

		palette.addEventListener("mousedown", function (e) {
			if (!box.contains(e.target)) close();
		});
		input.addEventListener("input", function () {
			run(input.value);
		});
		input.addEventListener("keydown", onKey);
	}

	/* mdBook loads the index lazily, so the first open may have to fetch it. */
	function ensureIndex() {
		if (window.search && window.search.index) return Promise.resolve(true);
		if (loading) return loading;
		var path = window.path_to_searchindex_js;
		if (!path) return Promise.resolve(false);
		loading = new Promise(function (resolve) {
			var s = document.createElement("script");
			s.src = path;
			s.onload = function () {
				resolve(!!(window.search && window.search.index));
			};
			s.onerror = function () {
				resolve(false);
			};
			document.head.appendChild(s);
		});
		return loading;
	}

	function docs() {
		var s = window.search;
		if (!s || !s.index || !s.index.documentStore) return [];
		var store = s.index.documentStore.docs;
		var urls = s.doc_urls || [];
		return Object.keys(store)
			.map(function (ref) {
				var d = store[ref];
				return {
					url: urls[Number(ref)] || "",
					title: d.title || "",
					crumbs: (d.breadcrumbs || "").replace(/\s*»\s*/g, " › "),
					body: d.body || "",
				};
			})
			.filter(function (d) {
				return d.url;
			});
	}

	/* Is `q` a subsequence of `s`? "expop" matches "expressions and operators". */
	function subseq(q, s) {
		var i = 0;
		for (var j = 0; j < s.length && i < q.length; j++) {
			if (s[j] === q[i]) i++;
		}
		return i === q.length;
	}

	function score(d, q) {
		var t = d.title.toLowerCase();
		var c = d.crumbs.toLowerCase();
		var b = d.body.toLowerCase();
		if (t === q) return 1000;
		if (t.indexOf(q) === 0) return 700 - t.length;
		var ti = t.indexOf(q);
		if (ti > 0) return 500 - ti;
		if (c.indexOf(q) >= 0) return 300;
		if (subseq(q, t)) return 220 - t.length;
		var bi = b.indexOf(q);
		if (bi >= 0) return 120 - Math.min(bi, 100) / 10;
		return -1;
	}

	function mark(text, q) {
		var i = text.toLowerCase().indexOf(q);
		if (i < 0) return [document.createTextNode(text)];
		return [
			document.createTextNode(text.slice(0, i)),
			el("mark", { text: text.slice(i, i + q.length) }),
			document.createTextNode(text.slice(i + q.length)),
		];
	}

	function run(raw) {
		var q = (raw || "").trim().toLowerCase();
		list.textContent = "";
		items = [];
		if (!q) {
			empty.textContent = "Type to search.";
			empty.hidden = false;
			return;
		}
		var scored = [];
		docs().forEach(function (d) {
			var s = score(d, q);
			if (s > 0) scored.push({ d: d, s: s });
		});
		scored.sort(function (a, b) {
			return b.s - a.s;
		});
		scored = scored.slice(0, MAX);

		if (!scored.length) {
			empty.textContent = 'Nothing matches "' + raw.trim() + '".';
			empty.hidden = false;
			return;
		}
		empty.hidden = true;

		scored.forEach(function (hit, n) {
			var a = el("a", { href: hit.d.url, tabindex: "-1" });
			var title = el("span", { class: "vlt-title" });
			mark(hit.d.title, q).forEach(function (p) {
				title.appendChild(p);
			});
			a.appendChild(title);
			a.appendChild(el("span", { class: "vlt-crumb", text: hit.d.crumbs }));
			var li = el(
				"li",
				{
					role: "option",
					id: "vlt-opt-" + n,
					"aria-selected": n === 0 ? "true" : "false",
				},
				[a],
			);
			li.addEventListener("mouseenter", function () {
				select(n);
			});
			list.appendChild(li);
			items.push(li);
		});
		selected = 0;
		sync();
	}

	function select(n) {
		if (!items.length) return;
		selected = (n + items.length) % items.length;
		sync();
	}

	function sync() {
		items.forEach(function (li, n) {
			li.setAttribute("aria-selected", n === selected ? "true" : "false");
		});
		var cur = items[selected];
		if (cur) {
			input.setAttribute("aria-activedescendant", cur.id);
			cur.scrollIntoView({ block: "nearest" });
		}
	}

	function onKey(e) {
		if (e.key === "Escape") {
			e.preventDefault();
			close();
			return;
		}
		if (e.key === "ArrowDown") {
			e.preventDefault();
			select(selected + 1);
			return;
		}
		if (e.key === "ArrowUp") {
			e.preventDefault();
			select(selected - 1);
			return;
		}
		if (e.key === "Enter") {
			e.preventDefault();
			var cur = items[selected];
			var a = cur && cur.querySelector("a");
			if (a) window.location.assign(a.getAttribute("href"));
			return;
		}
		// The dialog is modal, so Tab must not escape it. One focusable element,
		// so Tab simply stays put rather than cycling through the page behind.
		if (e.key === "Tab") {
			e.preventDefault();
		}
	}

	var lastFocus = null;

	function open() {
		if (!palette) build();
		lastFocus = document.activeElement;
		palette.hidden = false;
		document.documentElement.style.overflow = "hidden";
		input.value = "";
		run("");
		input.focus();
		ensureIndex().then(function (ok) {
			if (!ok) empty.textContent = "The search index could not be loaded.";
		});
	}

	function close() {
		if (!palette || palette.hidden) return;
		palette.hidden = true;
		document.documentElement.style.overflow = "";
		if (lastFocus && lastFocus.focus) lastFocus.focus();
	}

	document.addEventListener("keydown", function (e) {
		if ((e.ctrlKey || e.metaKey) && (e.key === "k" || e.key === "K")) {
			e.preventDefault();
			if (palette && !palette.hidden) close();
			else open();
		}
	});

	if (document.readyState === "loading") {
		document.addEventListener("DOMContentLoaded", build);
	} else {
		build();
	}
})();
