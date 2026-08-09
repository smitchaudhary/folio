'use strict';

/* ---------- constants ---------- */

const TYPES = {
  blog_post: 'blog', video: 'vid.', podcast: 'pod.', news: 'news',
  thread: 'thrd', academic_paper: 'acad', other: 'oth.',
};
const STATUS_EMOJI = { todo: '📝', doing: '⏳', done: '✅' };
const VIEWS = ['inbox', 'archive', 'all'];

/* ---------- state ---------- */

const state = {
  view: 'inbox',
  items: [],
  config: { max_items: 30, archive_on_overflow: 'abort' },
  q: '',
  chooser: null, // item id whose status menu is open
  editing: null, // item being edited, or null for add mode
};

const $ = (id) => document.getElementById(id);
const el = (tag, cls, text) => {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text !== undefined) e.textContent = text;
  return e;
};

/* ---------- api ---------- */

async function api(path, opts = {}) {
  let res;
  try {
    res = await fetch(path, opts);
  } catch {
    throw { code: 'unreachable', message: 'cannot reach the folio server. is it running?' };
  }
  const text = await res.text();
  let data = null;
  if (text) {
    try { data = JSON.parse(text); } catch { /* non-JSON body */ }
  }
  if (!res.ok) {
    if (data && data.error) throw { code: data.error.code, message: data.error.message };
    throw { code: 'http_' + res.status, message: 'server error (' + res.status + ')' };
  }
  return data;
}

const json = (method, body) => ({
  method,
  headers: { 'Content-Type': 'application/json' },
  body: body === undefined ? undefined : JSON.stringify(body),
});

/* ---------- data ---------- */

async function load() {
  $('list').innerHTML = '';
  $('list').appendChild(el('div', 'empty', 'Loading…'));
  try {
    const [items, config] = await Promise.all([
      api('/api/items?view=' + state.view),
      api('/api/config'),
    ]);
    state.items = items;
    state.config = config;
    hideBanner();
  } catch (e) {
    state.items = [];
    showBanner(e.message, false);
  }
  render();
}

function inboxCount() {
  return state.items.filter((i) => i.status !== 'done').length;
}

/* ---------- render ---------- */

function visibleItems() {
  const q = state.q.trim().toLowerCase();
  return state.items.filter((it) => {
    const inView = state.view === 'all' ? true : (state.view === 'archive' ? it.status === 'done' : it.status !== 'done');
    if (!inView) return false;
    if (!q) return true;
    return (it.name + ' ' + it.author + ' ' + it.link + ' ' + TYPES[it.type]).toLowerCase().includes(q);
  });
}

function render() {
  // tabs
  document.querySelectorAll('.tab').forEach((t) => t.classList.toggle('on', t.dataset.view === state.view));

  // cap bar
  const n = inboxCount();
  const max = state.config.max_items || 30;
  const pct = Math.min(100, (n / max) * 100);
  const bar = $('bar');
  bar.style.width = pct + '%';
  bar.className = n >= max ? 'full' : (n >= max * 0.8 ? 'hot' : '');
  $('count').textContent = `inbox ${n} / ${max}`;

  // overflow notice
  const ov = $('overflow');
  if (n >= max) {
    const strat = state.config.archive_on_overflow;
    ov.hidden = false;
    ov.textContent = strat === 'abort'
      ? `Inbox is full (${max}). Read or delete something first — that is the rule.`
      : `Inbox is full (${max}). Adding a new item will auto-archive the oldest ${strat === 'todo' ? 'todo' : 'item'}.`;
  } else {
    ov.hidden = true;
  }
  $('btn-add').disabled = n >= max && state.config.archive_on_overflow === 'abort';

  // rows
  const list = $('list');
  list.innerHTML = '';
  const vis = visibleItems();
  $('footcount').textContent = `${vis.length} shown · ${state.items.length} total`;

  if (!vis.length) {
    const empty = el('div', 'empty',
      state.q ? `Nothing matches “${state.q}”.`
        : state.view === 'inbox' ? 'Inbox is empty. Add something worth reading.'
        : state.view === 'archive' ? 'Nothing archived yet.'
        : 'Nothing here yet.');
    if (!state.q && state.view === 'inbox' && state.items.length === 0 && !document.querySelector('#list .empty')) {
      const sub = el('div', 'sub', 'Press n or + Add to save your first item.');
      empty.appendChild(sub);
    }
    list.appendChild(empty);
    return;
  }
  vis.forEach((it) => list.appendChild(rowFor(it)));
}

function rowFor(it) {
  const row = el('div', 'row');
  row.dataset.id = it.id;

  const st = el('button', 'st', STATUS_EMOJI[it.status]);
  st.title = 'choose status';
  st.onclick = () => { state.chooser = state.chooser === it.id ? null : it.id; render(); };
  row.appendChild(st);

  const main = el('div', 'main');
  const name = el('div', 'name');
  if (it.link) {
    const a = el('a', null, it.name);
    a.href = it.link;
    a.target = '_blank';
    a.rel = 'noopener';
    name.appendChild(a);
  } else {
    name.textContent = it.name;
  }
  main.appendChild(name);
  if (it.note) main.appendChild(el('div', 'note-line', it.note));
  const date = it.status === 'done' && it.finished_at ? it.finished_at.slice(0, 10) : it.added_at.slice(0, 10);
  main.appendChild(el('div', 'meta', `${TYPES[it.type]} · ${it.status === 'done' ? 'done ' + date : date} · ${it.author || '—'}`));
  row.appendChild(main);

  const chip = el('span', 'chip' + (it.kind === 'reference' ? ' ref' : ''), it.kind === 'reference' ? '🔖 ref' : TYPES[it.type]);
  row.appendChild(chip);

  const acts = el('div', 'acts');
  const eb = el('button', 'a', '✎'); eb.title = 'edit'; eb.onclick = () => openEdit(it.id);
  const rb = el('button', 'a', '🔖'); rb.title = 'toggle reference'; rb.onclick = () => toggleRef(it.id);
  const db = el('button', 'a danger', 'delete'); db.onclick = () => deleteItem(it.id);
  acts.append(eb, rb, db);
  row.appendChild(acts);

  if (state.chooser === it.id) row.appendChild(statusMenu(it));
  return row;
}

function statusMenu(it) {
  const menu = el('div', 'statusmenu');
  ['todo', 'doing', 'done'].forEach((s) => {
    const opt = el('button', 'opt' + (it.status === s ? ' on' : ''), STATUS_EMOJI[s] + ' ' + s.charAt(0).toUpperCase() + s.slice(1));
    opt.onclick = () => chooseStatus(it.id, s);
    menu.appendChild(opt);
  });
  return menu;
}

/* ---------- actions ---------- */

async function chooseStatus(id, status) {
  state.chooser = null;
  try {
    await api(`/api/items/${id}/status`, json('POST', { status }));
    await load();
  } catch (e) {
    showBanner(e.message, false);
  }
}

async function toggleRef(id) {
  try {
    await api(`/api/items/${id}/kind`, { method: 'POST' });
    await load();
  } catch (e) {
    showBanner(e.message, false);
  }
}

async function deleteItem(id) {
  const it = state.items.find((x) => x.id === id);
  if (!it) return;
  if (!confirm(`Delete “${it.name}”?`)) return;
  try {
    await api(`/api/items/${id}`, { method: 'DELETE' });
    await load();
  } catch (e) {
    showBanner(e.message, false);
  }
}

/* ---------- add / edit modal ---------- */

function openAdd() {
  state.editing = null;
  $('modal-title').textContent = 'Add to inbox';
  $('btn-save').textContent = 'Add';
  $('f-name').value = '';
  $('f-type').value = 'blog_post';
  $('f-author').value = '';
  $('f-link').value = '';
  $('f-note').value = '';
  hideFormError();
  $('modal').hidden = false;
  $('f-name').focus();
}

function openEdit(id) {
  const it = state.items.find((x) => x.id === id);
  if (!it) return;
  state.editing = it;
  $('modal-title').textContent = 'Edit item';
  $('btn-save').textContent = 'Save';
  $('f-name').value = it.name;
  $('f-type').value = it.type;
  $('f-author').value = it.author;
  $('f-link').value = it.link;
  $('f-note').value = it.note;
  hideFormError();
  $('modal').hidden = false;
  $('f-name').focus();
}

function closeModal() {
  $('modal').hidden = true;
  state.editing = null;
  state.chooser = null;
}

function showFormError(msg) {
  const e = $('form-error');
  e.textContent = msg;
  e.hidden = false;
}

function hideFormError() {
  $('form-error').hidden = true;
}

async function submitForm() {
  const name = $('f-name').value.trim();
  if (!name) {
    showFormError('Name is required.');
    return;
  }
  const body = {
    name,
    type: $('f-type').value,
    author: $('f-author').value.trim() || null,
    link: $('f-link').value.trim() || null,
    note: $('f-note').value.trim() || null,
  };
  try {
    if (state.editing) {
      await api(`/api/items/${state.editing.id}`, json('PATCH', body));
    } else {
      await api('/api/items', json('POST', body));
    }
    closeModal();
    await load();
  } catch (e) {
    if (e.code === 'inbox_full') {
      showFormError('Inbox is full. Read or delete something first — that is the rule.');
    } else {
      showFormError(e.message);
    }
  }
}

/* ---------- config ---------- */

function openConfig() {
  $('cfg-max').value = state.config.max_items;
  $('cfg-strategy').value = state.config.archive_on_overflow;
  $('config').hidden = !$('config').hidden;
}

async function saveConfig() {
  const max_items = Math.max(1, parseInt($('cfg-max').value, 10) || 30);
  const archive_on_overflow = $('cfg-strategy').value;
  try {
    state.config = await api('/api/config', json('PUT', { max_items, archive_on_overflow }));
    $('config').hidden = true;
    render();
    showBanner('Config saved.', true);
  } catch (e) {
    showBanner(e.message, false);
  }
}

/* ---------- banner ---------- */

let bannerTimer = null;

function showBanner(msg, ok) {
  const b = $('banner');
  b.textContent = '';
  b.className = 'banner' + (ok ? ' ok' : '');
  b.appendChild(document.createTextNode(msg));
  const x = el('button', null, '✕');
  x.onclick = hideBanner;
  b.appendChild(x);
  b.hidden = false;
  if (ok) {
    clearTimeout(bannerTimer);
    bannerTimer = setTimeout(hideBanner, 2500);
  }
}

function hideBanner() {
  $('banner').hidden = true;
  clearTimeout(bannerTimer);
}

/* ---------- routing, keyboard, wiring ---------- */

function route() {
  const h = location.hash.replace(/^#\/?/, '');
  state.view = VIEWS.includes(h) ? h : 'inbox';
  history.replaceState(null, '', '#/' + state.view);
  load();
}

function toggleDark() {
  document.documentElement.classList.toggle('dark');
  localStorage.setItem('folio-dark', document.documentElement.classList.contains('dark') ? '1' : '0');
}

document.addEventListener('DOMContentLoaded', () => {
  // dark mode: stored choice, else system preference
  const stored = localStorage.getItem('folio-dark');
  if (stored === '1' || (stored === null && matchMedia('(prefers-color-scheme: dark)').matches)) {
    document.documentElement.classList.add('dark');
  }

  $('tabs').addEventListener('click', (e) => {
    const t = e.target.closest('.tab');
    if (t) {
      state.view = t.dataset.view;
      history.pushState(null, '', '#/' + state.view);
      load();
    }
  });

  $('search').addEventListener('input', (e) => { state.q = e.target.value; render(); });

  $('btn-add').addEventListener('click', openAdd);
  $('btn-config').addEventListener('click', openConfig);
  $('btn-dark').addEventListener('click', toggleDark);
  $('btn-cancel').addEventListener('click', closeModal);
  $('form').addEventListener('submit', (e) => { e.preventDefault(); submitForm(); });
  $('btn-config-save').addEventListener('click', saveConfig);
  $('modal').addEventListener('click', (e) => { if (e.target === $('modal')) closeModal(); });

  document.addEventListener('click', (e) => {
    if (state.chooser && !e.target.closest('.statusmenu') && !e.target.closest('.st')) {
      state.chooser = null;
      render();
    }
  });

  document.addEventListener('keydown', (e) => {
    const typing = e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT';
    if (e.key === 'Escape') {
      if (!$('modal').hidden) { closeModal(); return; }
      if (!$('config').hidden) { $('config').hidden = true; return; }
      if (state.chooser) { state.chooser = null; render(); return; }
      $('search').value = '';
      state.q = '';
      render();
      return;
    }
    if (typing) return;
    if (e.key === '/') { e.preventDefault(); $('search').focus(); }
    if (e.key.toLowerCase() === 'n') openAdd();
  });

  window.addEventListener('hashchange', route);
  route();
});
