// vibelight: the outlines as one page that works from file:.
//
// Cursor's browser drops every link click that leads to a file: URL, so the
// HTML folder holds one page, index.html, and this script. Each outline's
// page lives in a data file, <page>.page.js, that the script loads with a
// script tag, which is no navigation, and shows in a tab. The address names
// the shown page as #<page>/<id>, a change of the fragment, which is allowed,
// so Back, Forward, and a reload work. search.js lists every declaration
// with an id, for the search box.
(function () {
  'use strict';

  var INDEX = 'index';
  var MARK = 'vibelight-target';
  var TABS_KEY = 'vibelight.tabs';

  var pages = Object.create(null);
  var search = null;
  var searchWaiting = [];
  var tabs = [];
  var active = -1;

  // Each data file calls this when it loads.
  window.vibelightPage = function (stem, title, content) {
    pages[stem] = { title: title, content: content };
  };

  // search.js calls this when it loads.
  window.vibelightSearch = function (index) {
    search = index;
    var waiting = searchWaiting;
    searchWaiting = [];
    waiting.forEach(function (done) {
      done();
    });
  };

  function script(src, done, fail) {
    var tag = document.createElement('script');
    tag.src = src;
    tag.onload = function () {
      tag.remove();
      done();
    };
    tag.onerror = function () {
      tag.remove();
      fail();
    };
    document.head.appendChild(tag);
  }

  function load(stem, done, fail) {
    if (pages[stem]) {
      done();
      return;
    }
    script(
      encodeURIComponent(stem) + '.page.js',
      function () {
        if (pages[stem]) {
          done();
        } else {
          fail();
        }
      },
      fail
    );
  }

  function loadSearch(done) {
    if (search) {
      done();
      return;
    }
    searchWaiting.push(done);
    if (searchWaiting.length === 1) {
      script('search.js', function () {}, function () {
        searchWaiting = [];
      });
    }
  }

  // The page and the id an address names: #<page>/<id>, or the index.
  function parse(hash) {
    hash = hash.replace(/^#/, '');
    try {
      hash = decodeURIComponent(hash);
    } catch (error) {
      // A malformed escape stays as written.
    }
    var slash = hash.indexOf('/');
    if (!hash) {
      return { stem: INDEX, id: '' };
    }
    if (slash < 0) {
      return { stem: hash, id: '' };
    }
    return { stem: hash.slice(0, slash) || INDEX, id: hash.slice(slash + 1) };
  }

  function address(stem, id) {
    return stem === INDEX && !id ? '' : stem + '/' + (id || '');
  }

  // The app's frame: the bar with its tabs and search, and the views.
  var bar = document.createElement('header');
  bar.className = 'vl-bar';
  bar.innerHTML =
    '<a class="vl-home" href="#">Outlines</a>' +
    '<div class="vl-tabs" role="tablist"></div>' +
    '<div class="vl-search"><input type="search" placeholder="Search" ' +
    'aria-label="Search declarations" autocomplete="off" spellcheck="false">' +
    '<kbd>Ctrl K</kbd><ol class="vl-results" hidden></ol></div>';
  var tabStrip = bar.querySelector('.vl-tabs');
  var input = bar.querySelector('input');
  var results = bar.querySelector('.vl-results');
  var views = document.createElement('div');
  views.className = 'vl-views';
  var toast = document.createElement('div');
  toast.className = 'vl-toast';
  toast.hidden = true;
  document.body.textContent = '';
  document.body.appendChild(bar);
  document.body.appendChild(views);
  document.body.appendChild(toast);

  function save() {
    try {
      sessionStorage.setItem(
        TABS_KEY,
        JSON.stringify({
          tabs: tabs.map(function (tab) {
            return { stem: tab.stem, id: tab.id };
          }),
          active: active
        })
      );
    } catch (error) {
      // Storage is optional: without it a reload keeps only the shown page.
    }
  }

  function restore() {
    try {
      var saved = JSON.parse(sessionStorage.getItem(TABS_KEY) || 'null');
      if (saved && saved.tabs && saved.tabs.length) {
        return saved;
      }
    } catch (error) {
      // Nothing saved, or storage is off.
    }
    return null;
  }

  function newTab(stem, id) {
    var view = document.createElement('div');
    view.className = 'vl-view';
    view.hidden = true;
    views.appendChild(view);
    var tab = { stem: stem, id: id, view: view, shown: null, scroll: 0 };
    tabs.push(tab);
    return tabs.length - 1;
  }

  function drawTabs() {
    tabStrip.textContent = '';
    tabs.forEach(function (tab, i) {
      var button = document.createElement('div');
      button.className = 'vl-tab' + (i === active ? ' on' : '');
      button.setAttribute('role', 'tab');
      button.dataset.index = String(i);
      var label = document.createElement('span');
      label.textContent = pages[tab.stem] ? pages[tab.stem].title : tab.stem;
      button.appendChild(label);
      if (tabs.length > 1) {
        var close = document.createElement('span');
        close.className = 'vl-close';
        close.textContent = '\u00d7';
        close.title = 'Close tab';
        button.appendChild(close);
      }
      tabStrip.appendChild(button);
    });
  }

  function mark(tab) {
    var old = tab.view.querySelector('.' + MARK);
    if (old) {
      old.classList.remove(MARK);
    }
    var el = tab.id ? tab.view.querySelector('[id="' + tab.id.replace(/"/g, '\\"') + '"]') : null;
    if (el) {
      el.classList.add(MARK);
      el.scrollIntoView();
      window.scrollBy(0, -bar.offsetHeight - 8);
    } else {
      window.scrollTo(0, 0);
    }
  }

  // Shows the tab's page in its view, loading the page first if it must.
  function fill(tab, scroll) {
    load(
      tab.stem,
      function () {
        if (tab.shown !== tab.stem) {
          var page = pages[tab.stem];
          tab.view.innerHTML = page.content;
          tab.shown = tab.stem;
          drawTabs();
        }
        if (tabs[active] !== tab) {
          return;
        }
        document.title = pages[tab.stem].title;
        if (scroll === undefined) {
          mark(tab);
        } else {
          window.scrollTo(0, scroll);
        }
      },
      function () {
        tab.shown = null;
        tab.view.innerHTML = '';
        var note = document.createElement('p');
        note.className = 'vl-missing';
        note.textContent = 'vibelight: cannot load ' + tab.stem + '.page.js beside this page.';
        tab.view.appendChild(note);
      }
    );
  }

  function activate(i, scroll) {
    if (tabs[active] && active !== i) {
      tabs[active].scroll = window.scrollY;
      tabs[active].view.hidden = true;
    }
    active = i;
    var tab = tabs[i];
    tab.view.hidden = false;
    drawTabs();
    history.replaceState(null, '', '#' + address(tab.stem, tab.id));
    save();
    fill(tab, scroll);
  }

  // Shows a page in the active tab, as a step Back can undo.
  function go(stem, id) {
    var next = '#' + address(stem, id);
    if (location.hash === next || (next === '#' && !location.hash)) {
      route();
    } else {
      location.hash = next;
    }
  }

  function open(stem, id) {
    var i = newTab(stem, id);
    activate(i);
  }

  function close(i) {
    var tab = tabs[i];
    tab.view.remove();
    tabs.splice(i, 1);
    if (!tabs.length) {
      active = -1;
      open(INDEX, '');
      return;
    }
    if (i === active) {
      active = -1;
      activate(Math.min(i, tabs.length - 1), tabs[Math.min(i, tabs.length - 1)].scroll);
    } else {
      if (i < active) {
        active -= 1;
      }
      drawTabs();
      save();
    }
  }

  // Follows the address in the active tab.
  function route() {
    var t = parse(location.hash);
    if (active < 0) {
      open(t.stem, t.id);
      return;
    }
    var tab = tabs[active];
    tab.stem = t.stem;
    tab.id = t.id;
    save();
    fill(tab);
  }

  function showToast(text) {
    toast.textContent = text;
    toast.hidden = false;
    clearTimeout(showToast.timer);
    showToast.timer = setTimeout(function () {
      toast.hidden = true;
    }, 2400);
  }

  // A source link names a file beside the repository, which no browser
  // inside an editor opens, so a click copies its path and line instead.
  function copySource(href) {
    var match = /^((?:\.\.\/)*)(.*?)(?:#L(\d+))?$/.exec(href);
    var path = match[2] + (match[3] ? ':' + match[3] : '');
    var done = function () {
      showToast('Copied ' + path);
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(path).then(done, function () {
        fallbackCopy(path);
        done();
      });
    } else {
      fallbackCopy(path);
      done();
    }
  }

  function fallbackCopy(text) {
    var area = document.createElement('textarea');
    area.value = text;
    document.body.appendChild(area);
    area.select();
    try {
      document.execCommand('copy');
    } catch (error) {
      // Nothing more to try.
    }
    area.remove();
  }

  document.addEventListener('click', function (e) {
    var tabButton = e.target.closest ? e.target.closest('.vl-tab') : null;
    if (tabButton) {
      var index = Number(tabButton.dataset.index);
      if (e.target.classList.contains('vl-close')) {
        close(index);
      } else if (index !== active) {
        activate(index, tabs[index].scroll);
      }
      return;
    }
    if (e.defaultPrevented || e.button !== 0 || e.altKey || e.shiftKey) {
      return;
    }
    var a = e.target.closest ? e.target.closest('a[href]') : null;
    if (!a || a.target) {
      return;
    }
    var href = a.getAttribute('href');
    var other = /^([^\/#?:]+)\.html(?:#(.*))?$/.exec(href);
    var here = /^#(.*)$/.exec(href);
    var newTabWanted = e.ctrlKey || e.metaKey;
    if (other || here) {
      e.preventDefault();
      var stem = other ? other[1] : here[1] ? tabs[active].stem : INDEX;
      var id = other ? other[2] || '' : here[1];
      if (a.classList.contains('vl-home')) {
        stem = INDEX;
        id = '';
      }
      if (newTabWanted) {
        open(stem, id);
      } else {
        go(stem, id);
      }
      return;
    }
    if (/\.rs(#L\d+)?$/.test(href) && !/^[a-z]+:/i.test(href)) {
      e.preventDefault();
      copySource(href);
    }
  });

  // A middle click closes a tab, and opens a link in a new tab.
  document.addEventListener('auxclick', function (e) {
    if (e.button !== 1) {
      return;
    }
    var tabButton = e.target.closest ? e.target.closest('.vl-tab') : null;
    if (tabButton && tabs.length > 1) {
      e.preventDefault();
      close(Number(tabButton.dataset.index));
      return;
    }
    var a = e.target.closest ? e.target.closest('a[href]') : null;
    if (!a) {
      return;
    }
    var other = /^([^\/#?:]+)\.html(?:#(.*))?$/.exec(a.getAttribute('href'));
    var here = /^#(.+)$/.exec(a.getAttribute('href'));
    if (other || here) {
      e.preventDefault();
      open(other ? other[1] : tabs[active].stem, other ? other[2] || '' : here[1]);
    }
  });

  window.addEventListener('hashchange', route);

  // Search: every declaration with an id, by its name and its path.
  var found = [];
  var chosen = 0;

  // An id reads <kind>.<path>.<name>; a facade's API heading reads
  // api.<path>.<name>, its path inside the facade.
  function describe(id, page) {
    var parts = id.replace(/-\d+$/, '').split('.');
    var kind = parts.shift();
    var name = parts.pop() || '';
    if (kind === 'api') {
      return { kind: 'api', name: name, path: [page].concat(parts).join('::'), api: true };
    }
    return { kind: kind, name: name, path: parts.join('::'), api: false };
  }

  // Exact names first, then names that start with the query, then paths
  // that hold it; a facade's API before the crates behind it.
  function find(query) {
    var q = query.trim().toLowerCase();
    if (!q || !search) {
      return [];
    }
    var ranks = [[], [], []];
    for (var i = 0; i < search.ids.length; i += 1) {
      var entry = search.ids[i];
      var page = search.pages[entry[0]];
      var d = describe(entry[1], page);
      var name = d.name.toLowerCase();
      var rank = name === q ? 0 : name.indexOf(q) === 0 ? 1 : -1;
      if (rank < 0 && (d.path + '::' + name).toLowerCase().indexOf(q) >= 0) {
        rank = 2;
      }
      if (rank >= 0) {
        ranks[rank].push({ page: page, id: entry[1], d: d });
      }
    }
    var api = function (a, b) {
      return (b.d.api ? 1 : 0) - (a.d.api ? 1 : 0);
    };
    return ranks[0].sort(api).concat(ranks[1].sort(api), ranks[2].sort(api)).slice(0, 40);
  }

  function drawResults() {
    results.textContent = '';
    found.forEach(function (hit, i) {
      var li = document.createElement('li');
      li.className = i === chosen ? 'on' : '';
      li.dataset.index = String(i);
      li.innerHTML =
        '<span class="vl-kind"></span><span class="vl-name"></span><span class="vl-path"></span>';
      li.children[0].textContent = hit.d.kind;
      li.children[1].textContent = hit.d.name;
      li.children[2].textContent = hit.d.path;
      results.appendChild(li);
    });
    results.hidden = !found.length;
  }

  function pick(i, inNewTab) {
    var hit = found[i];
    if (!hit) {
      return;
    }
    input.value = '';
    found = [];
    drawResults();
    input.blur();
    if (inNewTab) {
      open(hit.page, hit.id);
    } else {
      go(hit.page, hit.id);
    }
  }

  input.addEventListener('focus', function () {
    loadSearch(function () {
      found = find(input.value);
      drawResults();
    });
  });
  input.addEventListener('input', function () {
    loadSearch(function () {
      found = find(input.value);
      chosen = 0;
      drawResults();
    });
  });
  input.addEventListener('keydown', function (e) {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (found.length) {
        chosen = (chosen + (e.key === 'ArrowDown' ? 1 : found.length - 1)) % found.length;
        drawResults();
      }
    } else if (e.key === 'Enter') {
      e.preventDefault();
      pick(chosen, e.ctrlKey || e.metaKey);
    } else if (e.key === 'Escape') {
      input.value = '';
      found = [];
      drawResults();
      input.blur();
    }
  });
  input.addEventListener('blur', function () {
    setTimeout(function () {
      results.hidden = true;
    }, 150);
  });
  results.addEventListener('mousedown', function (e) {
    var li = e.target.closest('li');
    if (li) {
      e.preventDefault();
      pick(Number(li.dataset.index), e.ctrlKey || e.metaKey);
    }
  });
  document.addEventListener('keydown', function (e) {
    var typing = /^(INPUT|TEXTAREA)$/.test(document.activeElement && document.activeElement.tagName);
    if ((e.key === 'k' && (e.ctrlKey || e.metaKey)) || (e.key === '/' && !typing)) {
      e.preventDefault();
      input.focus();
      input.select();
    }
  });

  // Start: the saved tabs, then the page the address names.
  var saved = restore();
  if (saved) {
    saved.tabs.forEach(function (tab) {
      newTab(tab.stem, tab.id || '');
    });
    var want = parse(location.hash);
    var i = Math.min(Math.max(saved.active, 0), tabs.length - 1);
    tabs[i].stem = want.stem;
    tabs[i].id = want.id;
    activate(i);
  } else {
    route();
  }
})();
