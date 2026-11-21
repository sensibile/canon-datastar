// canon.js: client behaviour for canon-datastar overlays (Tooltip, InfoTip, Popover, Dialog,
// DropdownMenu, TabList, Toast). Ports the interaction logic of Astryx (packages/core/src),
// MIT, Meta Platforms, Inc.
//
// Load it before Datastar so Datastar expressions can call window.canon:
//   <script type="module" src="/static/canon.js"></script>
//   <script type="module" src="/static/datastar.js"></script>
//
// The markup comes from the Go and Rust renderers. Native features do most of the work:
// the popover attribute (top layer, light dismiss, Escape, focus return) and <dialog>.
// This file adds positioning, keyboard models, tooltips, toast timers and the window.canon API.
// It wires markup through event delegation and a MutationObserver, so elements patched in by
// Datastar SSE work without any setup.

const doc = document;
const GAP = 4;
const PAD = 8;

// ---------------------------------------------------------------------------------------------
// Positioning: keeps each open layer next to its anchor while the page scrolls or resizes.

/** @type {Map<HTMLElement, Element>} */
const anchored = new Map();

function place(layer, anchor) {
  if (!anchor.isConnected) return;
  const pref = (layer._canonPlacement ??= layer.dataset.placement || 'below');
  const align = layer.dataset.alignment || 'center';
  const rtl = getComputedStyle(anchor).direction === 'rtl';
  const a = anchor.getBoundingClientRect();
  if (layer.hasAttribute('data-match-width')) layer.style.minWidth = `${a.width}px`;
  const w = layer.offsetWidth;
  const h = layer.offsetHeight;
  const vw = doc.documentElement.clientWidth;
  const vh = doc.documentElement.clientHeight;

  // Physical side: top, bottom, left, right.
  const physical = (p) =>
    p === 'above' ? 'top' : p === 'below' ? 'bottom' : (p === 'start') !== rtl ? 'left' : 'right';
  const opposite = { top: 'bottom', bottom: 'top', left: 'right', right: 'left' };
  const fits = {
    top: a.top - GAP - h >= PAD,
    bottom: a.bottom + GAP + h <= vh - PAD,
    left: a.left - GAP - w >= PAD,
    right: a.right + GAP + w <= vw - PAD,
  };
  let side = physical(pref);
  if (!fits[side] && fits[opposite[side]]) side = opposite[side];

  let x;
  let y;
  if (side === 'top' || side === 'bottom') {
    y = side === 'bottom' ? a.bottom + GAP : a.top - GAP - h;
    const startEdge = rtl ? a.right - w : a.left;
    const endEdge = rtl ? a.left : a.right - w;
    x = align === 'start' ? startEdge : align === 'end' ? endEdge : a.left + (a.width - w) / 2;
  } else {
    x = side === 'right' ? a.right + GAP : a.left - GAP - w;
    y = align === 'start' ? a.top : align === 'end' ? a.bottom - h : a.top + (a.height - h) / 2;
  }
  x = Math.min(Math.max(x, PAD), Math.max(PAD, vw - PAD - w));
  y = Math.min(Math.max(y, PAD), Math.max(PAD, vh - PAD - h));
  layer.style.left = `${Math.round(x)}px`;
  layer.style.top = `${Math.round(y)}px`;
  const logical = { top: 'above', bottom: 'below', left: rtl ? 'end' : 'start', right: rtl ? 'start' : 'end' };
  layer.dataset.placement = logical[side];
}

function track(layer, anchor) {
  if (!anchor) return;
  anchored.set(layer, anchor);
  place(layer, anchor);
}

let frame = 0;
function reposition() {
  if (frame) return;
  frame = requestAnimationFrame(() => {
    frame = 0;
    for (const [layer, anchor] of anchored) place(layer, anchor);
  });
}
addEventListener('scroll', reposition, { capture: true, passive: true });
addEventListener('resize', reposition);
const resizeObserver = new ResizeObserver(reposition);

/** The element a layer is positioned against. */
function anchorFor(layer) {
  if (layer._canonAnchor?.isConnected) return layer._canonAnchor;
  const id = CSS.escape(layer.id);
  const tip = doc.querySelector(`[data-tooltip="${id}"]`);
  if (tip) return tip.firstElementChild ?? tip;
  return (
    doc.querySelector(`[popovertarget="${id}"]`) ??
    doc.querySelector(`[data-popover="${id}"]`) ??
    null
  );
}

// ---------------------------------------------------------------------------------------------
// Focus helpers.

const FOCUSABLE = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled]):not([type="hidden"])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
  '[contenteditable="true"]',
].join(',');

function focusables(root, skipAriaDisabled = false) {
  return Array.from(root.querySelectorAll(FOCUSABLE)).filter(
    (el) =>
      el.tabIndex >= 0 &&
      !el.matches(':disabled') &&
      !el.closest('[inert]') &&
      !(skipAriaDisabled && el.getAttribute('aria-disabled') === 'true') &&
      el.getClientRects().length > 0 &&
      getComputedStyle(el).visibility === 'visible',
  );
}

function focusFirst(root) {
  const target = root.querySelector('[autofocus], [data-autofocus]') ?? focusables(root, true)[0];
  if (target) {
    target.focus({ preventScroll: true });
    return;
  }
  if (!root.hasAttribute('tabindex')) root.tabIndex = -1;
  root.focus({ preventScroll: true });
}

/** Keeps Tab inside a modal popover. */
function trapTab(e, root) {
  const items = focusables(root);
  if (items.length === 0) {
    e.preventDefault();
    return;
  }
  const first = items[0];
  const last = items[items.length - 1];
  const active = doc.activeElement;
  const inside = root.contains(active);
  if (inside && active !== root && !items.includes(active)) {
    // Focus sits on something no longer tabbable (an item of a menu this Tab just closed):
    // wrap only when no tabbable element is left in that direction.
    const F = Node.DOCUMENT_POSITION_FOLLOWING;
    const P = Node.DOCUMENT_POSITION_PRECEDING;
    const more = e.shiftKey
      ? items.some((el) => active.compareDocumentPosition(el) & P)
      : items.some((el) => active.compareDocumentPosition(el) & F && !active.contains(el));
    if (!more) {
      e.preventDefault();
      (e.shiftKey ? last : first).focus();
    }
    return;
  }
  if (e.shiftKey && (!inside || active === first || active === root)) {
    e.preventDefault();
    last.focus();
  } else if (!e.shiftKey && (!inside || active === last)) {
    e.preventDefault();
    first.focus();
  }
}

/** Whether `node` is in a modal dialog or layer opened above `layer` (not one that contains it). */
function inForeground(node, layer) {
  const fg = node.closest('dialog:modal, .astryx-layer');
  if (fg == null || fg === layer || fg.contains(layer)) return false;
  // Two open layers: only the one opened later is in front.
  const a = stack.indexOf(fg);
  const b = stack.indexOf(layer);
  return a < 0 || b < 0 || a > b;
}

// ---------------------------------------------------------------------------------------------
// Open layers, innermost last.

/** Open popovers and menus, innermost last. Tooltips aren't layers you interact with, so
 * they stay out of it and never suspend a modal popover's focus trap. @type {HTMLElement[]} */
const stack = [];
const isTop = (el) => stack[stack.length - 1] === el;

function safeShow(layer) {
  try {
    if (!layer.matches(':popover-open')) layer.showPopover();
  } catch {
    /* unsupported or disconnected */
  }
}
function safeHide(layer) {
  try {
    if (layer.matches(':popover-open')) layer.hidePopover();
  } catch {
    /* already closed */
  }
}

function setExpanded(layer, open) {
  const id = CSS.escape(layer.id);
  const triggers = new Set(doc.querySelectorAll(`[popovertarget="${id}"]`));
  // A Popover trigger that isn't a <button> has no popovertarget.
  const anchor = doc.querySelector(`[data-popover="${id}"]`);
  const resolved = anchor && popoverTrigger(anchor);
  if (resolved) triggers.add(resolved);
  for (const t of triggers) t.setAttribute('aria-expanded', String(open));
}

// Where focus goes when a menu opens: first or last item (keyboard) or the menu (pointer).
let menuFocus = 'menu';

doc.addEventListener(
  'beforetoggle',
  (e) => {
    const layer = e.target;
    if (!(layer instanceof HTMLElement) || !layer.classList.contains('astryx-layer')) return;
    if (e.newState === 'open') {
      // Position before the first paint.
      requestAnimationFrame(() => track(layer, anchorFor(layer)));
    }
  },
  true,
);

doc.addEventListener(
  'toggle',
  (e) => {
    const layer = e.target;
    if (!(layer instanceof HTMLElement)) return;
    if (layer.classList.contains('astryx-toast-viewport')) return;
    if (!layer.classList.contains('astryx-layer')) return;
    const open = e.newState === 'open';
    const i = stack.indexOf(layer);
    if (i >= 0) stack.splice(i, 1);
    if (open) {
      if (!layer.classList.contains('astryx-tooltip')) stack.push(layer);
      track(layer, anchorFor(layer));
      resizeObserver.observe(layer);
    } else {
      anchored.delete(layer);
      resizeObserver.unobserve(layer);
      layer._canonClosedAt = performance.now();
    }
    setExpanded(layer, open);
    if (!open) return;
    if (layer.classList.contains('astryx-dropdown-menu-surface')) {
      const which = menuFocus;
      menuFocus = 'menu';
      requestAnimationFrame(() => focusMenu(layer, which));
    } else if (layer.classList.contains('astryx-popover') && layer.hasAttribute('data-autofocus-open')) {
      requestAnimationFrame(() => focusFirst(layer));
    }
  },
  true,
);

// ---------------------------------------------------------------------------------------------
// Tooltip and InfoTip.

const HOVER_BRIDGE = 100;
/** @type {WeakMap<Element, {show?: number, hide?: number, focus?: boolean, pinned?: boolean}>} */
const tipState = new WeakMap();
const stateOf = (w) => {
  let s = tipState.get(w);
  if (!s) tipState.set(w, (s = {}));
  return s;
};
const tipLayer = (wrapper) => doc.getElementById(wrapper.dataset.tooltip);
const tipWrapper = (layer) => doc.querySelector(`[data-tooltip="${CSS.escape(layer.id)}"]`);

function clearTip(s) {
  clearTimeout(s.show);
  clearTimeout(s.hide);
}
function showTip(wrapper, ms) {
  const s = stateOf(wrapper);
  clearTip(s);
  const layer = tipLayer(wrapper);
  if (!layer) return;
  s.show = setTimeout(() => {
    layer._canonAnchor = wrapper.firstElementChild ?? wrapper;
    safeShow(layer);
  }, ms);
}
function hideTip(wrapper, ms) {
  const s = stateOf(wrapper);
  // A pinned InfoTip stays; don't cancel the show it is waiting for either.
  if (s.pinned) return;
  clearTip(s);
  const layer = tipLayer(wrapper);
  if (!layer) return;
  if (ms <= 0) safeHide(layer);
  else s.hide = setTimeout(() => safeHide(layer), ms);
}
function unpin(wrapper) {
  const s = stateOf(wrapper);
  s.pinned = false;
  wrapper.querySelector('.astryx-info-tip')?.setAttribute('aria-expanded', 'false');
  hideTip(wrapper, 0);
}

doc.addEventListener('pointerover', (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;
  const layer = t.closest('.astryx-tooltip');
  if (layer) {
    const w = tipWrapper(layer);
    if (w) clearTip(stateOf(w));
    return;
  }
  const wrapper = t.closest('[data-tooltip]');
  if (!wrapper || e.pointerType === 'touch') return;
  if (wrapper.contains(e.relatedTarget)) return;
  showTip(wrapper, Number(wrapper.dataset.delay ?? 200));
});

doc.addEventListener('pointerout', (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;
  const owner = t.closest('[data-tooltip]') ?? (t.closest('.astryx-tooltip') && tipWrapper(t.closest('.astryx-tooltip')));
  if (!owner) return;
  const next = e.relatedTarget;
  const layer = tipLayer(owner);
  if (owner.contains(next) || layer?.contains(next)) return;
  const s = stateOf(owner);
  if (s.focus || s.pinned) {
    // Leaving with the pointer keeps a focus-opened or pinned tooltip.
    clearTimeout(s.hide);
    return;
  }
  const ms = Number(owner.dataset.hideDelay ?? 0);
  hideTip(owner, ms > 0 ? ms : HOVER_BRIDGE);
});

doc.addEventListener('focusin', (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;
  const wrapper = t.closest('[data-tooltip]');
  if (!wrapper) return;
  const mode = wrapper.dataset.focusTrigger ?? 'auto';
  if (mode === 'never') return;
  if (mode === 'auto' && !t.matches(':focus-visible')) return;
  stateOf(wrapper).focus = true;
  showTip(wrapper, 0);
});

doc.addEventListener('focusout', (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;
  const wrapper = t.closest('[data-tooltip]');
  if (wrapper && !wrapper.contains(e.relatedTarget)) {
    stateOf(wrapper).focus = false;
    hideTip(wrapper, 0);
  }
  // A non-modal Popover closes when focus leaves it for somewhere other than its trigger.
  const layer = t.closest('.astryx-popover');
  const next = e.relatedTarget;
  if (
    layer &&
    next instanceof Element &&
    !layer.hasAttribute('data-modal') &&
    !layer.hasAttribute('data-no-light-dismiss') &&
    !layer.contains(next) &&
    !anchorFor(layer)?.contains(next) &&
    !inForeground(next, layer)
  ) {
    safeHide(layer);
  }
});

doc.addEventListener(
  'pointerdown',
  (e) => {
    const t = e.target;
    if (!(t instanceof Element)) return;
    // Pressing a tooltip trigger dismisses its tooltip (an InfoTip toggles on click instead).
    const wrapper = t.closest('[data-tooltip]');
    if (wrapper && !wrapper.querySelector('.astryx-info-tip')) {
      clearTip(stateOf(wrapper));
      const layer = tipLayer(wrapper);
      if (layer) safeHide(layer);
    }
    // Pinned InfoTips close on a press outside the button and the tooltip.
    for (const w of doc.querySelectorAll('[data-tooltip]')) {
      if (stateOf(w).pinned && !w.contains(t) && !tipLayer(w)?.contains(t)) unpin(w);
    }
    // Manual popovers with light dismiss (auto popovers get it natively).
    for (const layer of [...stack].reverse()) {
      if (!layer.matches('.astryx-popover[popover="manual"]:not([data-no-light-dismiss])')) continue;
      if (layer.contains(t) || anchorFor(layer)?.contains(t) || inForeground(t, layer)) continue;
      safeHide(layer);
    }
  },
  true,
);

// ---------------------------------------------------------------------------------------------
// Clicks: InfoTip pinning, Popover triggers, menu items, close buttons, tabs, toasts.

doc.addEventListener(
  'click',
  (e) => {
    const t = e.target;
    if (!(t instanceof Element)) return;

    // Disabled menu items and tabs do nothing (Datastar handlers are dropped server-side too).
    const disabled = t.closest('.astryx-dropdown-menu-item[aria-disabled="true"], .astryx-tab[aria-disabled="true"]');
    if (disabled) {
      e.preventDefault();
      e.stopImmediatePropagation();
      return;
    }

    // A menu trigger opened by keyboard (Enter/Space give detail 0) focuses the first item.
    const menuTrigger = t.closest('[aria-haspopup="menu"][popovertarget]');
    if (menuTrigger) menuFocus = e.detail === 0 ? 'first' : 'menu';

    const info = t.closest('.astryx-info-tip');
    if (info) {
      e.preventDefault();
      const wrapper = info.closest('[data-tooltip]');
      if (!wrapper) return;
      const s = stateOf(wrapper);
      if (s.pinned) unpin(wrapper);
      else {
        s.pinned = true;
        info.setAttribute('aria-expanded', 'true');
        showTip(wrapper, 0);
      }
    }
  },
  true,
);

doc.addEventListener('click', (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;

  // A Popover trigger that isn't a <button> (popovertarget only works on buttons).
  const anchor = t.closest('[data-popover]');
  if (anchor) {
    const trigger = popoverTrigger(anchor);
    const layer = doc.getElementById(anchor.dataset.popover);
    if (layer && trigger && trigger.tagName !== 'BUTTON' && trigger.contains(t)) {
      if (trigger.getAttribute('aria-disabled') === 'true') return;
      // The pointerdown that started this click may have just light-dismissed it.
      if (performance.now() - (layer._canonClosedAt ?? 0) < 300) return;
      if (layer.matches(':popover-open')) safeHide(layer);
      else safeShow(layer);
    }
  }

  const closer = t.closest('[data-popover-close]');
  if (closer) {
    const layer = closer.closest('[popover]');
    if (layer) {
      safeHide(layer);
      anchorFor(layer)?.focus({ preventScroll: true });
    }
  }

  const dialogCloser = t.closest('[data-dialog-close]');
  if (dialogCloser) dialogCloser.closest('dialog')?.close();

  const item = t.closest('.astryx-dropdown-menu-item');
  if (item && !e.defaultPrevented && !item.hasAttribute('data-keep-open')) {
    const surface = item.closest('.astryx-dropdown-menu-surface');
    if (surface) {
      const trigger = anchorFor(surface);
      safeHide(surface);
      trigger?.focus({ preventScroll: true });
    }
  }

  const dismiss = t.closest('[data-toast-dismiss]');
  if (dismiss) {
    const toast = dismiss.closest('.astryx-toast');
    if (toast) dismissToast(toast, 'manual');
  }
});

// ---------------------------------------------------------------------------------------------
// Keyboard.

let typeahead = '';
let typeaheadTimer = 0;

function menuItems(menu) {
  // Skip disabled items and ones hidden (e.g. by data-show), which can't take focus.
  return Array.from(menu.querySelectorAll('[role^="menuitem"]')).filter(
    (el) => el.getAttribute('aria-disabled') !== 'true' && el.getClientRects().length > 0,
  );
}
function focusMenu(surface, which) {
  const menu = surface.querySelector('[role="menu"]');
  if (!menu) return;
  const list = menuItems(menu);
  if (which === 'menu' || list.length === 0) menu.focus({ preventScroll: true });
  else (which === 'first' ? list[0] : list[list.length - 1]).focus({ preventScroll: true });
}

function onMenuKey(e, menu) {
  const list = menuItems(menu);
  const i = list.indexOf(e.target);
  const go = (n) => list[(n + list.length) % list.length]?.focus({ preventScroll: true });
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault();
      go(i < 0 ? 0 : i + 1);
      return;
    case 'ArrowUp':
      e.preventDefault();
      go(i < 0 ? list.length - 1 : i - 1);
      return;
    case 'Home':
      e.preventDefault();
      go(0);
      return;
    case 'End':
      e.preventDefault();
      go(list.length - 1);
      return;
    case 'Tab': {
      const surface = menu.closest('[popover]');
      if (surface) {
        // Leave the stack now (toggle fires later), so an enclosing modal's Tab trap,
        // checked further down in this same keydown, sees itself on top again.
        const at = stack.indexOf(surface);
        if (at >= 0) stack.splice(at, 1);
        safeHide(surface);
      }
      return;
    }
  }
  // Links don't activate on Space natively; menu items should.
  if (e.key === ' ' && typeahead === '' && e.target instanceof HTMLAnchorElement) {
    e.preventDefault();
    e.target.click();
    return;
  }
  // Space activates the focused item; it only extends a typeahead already in progress.
  if (e.key === ' ' && typeahead === '') return;
  // A space inside a search extends it rather than activating the focused item.
  if (e.key === ' ') e.preventDefault();
  if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
    clearTimeout(typeaheadTimer);
    typeahead += e.key.toLowerCase();
    typeaheadTimer = setTimeout(() => (typeahead = ''), 500);
    const start = i < 0 ? 0 : typeahead.length === 1 ? i + 1 : i;
    for (let k = 0; k < list.length; k++) {
      const item = list[(start + k) % list.length];
      if (item.textContent?.trim().toLowerCase().startsWith(typeahead)) {
        item.focus({ preventScroll: true });
        break;
      }
    }
  }
}

function onTabsKey(e, root) {
  const isTabList = root.querySelector('[role="tablist"]') != null;
  const stops = Array.from(root.querySelectorAll('[data-tab-value]')).filter(
    (el) => el.getAttribute('aria-disabled') !== 'true',
  );
  const i = stops.indexOf(e.target);
  if (i < 0) return;
  const rtl = getComputedStyle(root).direction === 'rtl';
  let next;
  if (e.key === 'ArrowRight') next = rtl ? i - 1 : i + 1;
  else if (e.key === 'ArrowLeft') next = rtl ? i + 1 : i - 1;
  else if (!isTabList && e.key === 'ArrowDown') next = i + 1;
  else if (!isTabList && e.key === 'ArrowUp') next = i - 1;
  else if (e.key === 'Home') next = 0;
  else if (e.key === 'End') next = stops.length - 1;
  else return;
  e.preventDefault();
  const target = stops[(next + stops.length) % stops.length];
  target.focus();
  target.scrollIntoView({ block: 'nearest', inline: 'nearest' });
}

doc.addEventListener('keydown', (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;

  if (e.key === 'Escape') {
    // Tooltips close on Escape, pinned InfoTips too.
    for (const w of doc.querySelectorAll('[data-tooltip]')) {
      const layer = tipLayer(w);
      if (stateOf(w).pinned) unpin(w);
      else if (layer?.matches(':popover-open')) hideTip(w, 0);
    }
    // Manual popovers that allow Escape (auto popovers get it natively).
    const top = stack[stack.length - 1];
    // Not when focus is in a modal dialog opened above it: that dialog gets the Escape.
    if (
      top?.matches('.astryx-popover[popover="manual"]:not([data-no-escape-dismiss])') &&
      !inForeground(t, top)
    ) {
      e.preventDefault();
      safeHide(top);
      anchorFor(top)?.focus({ preventScroll: true });
    }
  }

  // Menu trigger: ArrowDown opens on the first item, ArrowUp on the last.
  const menuTrigger = t.closest('[aria-haspopup="menu"][popovertarget]');
  if (menuTrigger && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
    const surface = doc.getElementById(menuTrigger.getAttribute('popovertarget'));
    if (surface && !menuTrigger.matches(':disabled') && menuTrigger.ariaBusy !== 'true') {
      e.preventDefault();
      const which = e.key === 'ArrowDown' ? 'first' : 'last';
      if (surface.matches(':popover-open')) focusMenu(surface, which);
      else {
        menuFocus = which;
        safeShow(surface);
      }
    }
    return;
  }

  // A Popover trigger that isn't a <button> opens on Enter and Space.
  const anchor = t.closest('[data-popover]');
  if (anchor && (e.key === 'Enter' || e.key === ' ')) {
    const trigger = popoverTrigger(anchor);
    if (trigger === t && trigger.tagName !== 'BUTTON') {
      e.preventDefault();
      trigger.click();
    }
  }

  const menu = t.closest('[role="menu"].astryx-dropdown-menu');
  if (menu) onMenuKey(e, menu);

  const layer = t.closest('.astryx-popover[data-modal]');
  if (layer && e.key === 'Tab' && isTop(layer)) trapTab(e, layer);

  const tabs = t.closest('.astryx-tab-list');
  if (tabs) onTabsKey(e, tabs);
});

// Hover moves focus in a menu, so keyboard and pointer share one highlighted row.
doc.addEventListener('pointermove', (e) => {
  if (e.pointerType === 'touch' || !(e.target instanceof Element)) return;
  const item = e.target.closest('.astryx-dropdown-menu-item');
  if (item && item.getAttribute('aria-disabled') !== 'true' && doc.activeElement !== item) {
    item.focus({ preventScroll: true });
  }
});

// ---------------------------------------------------------------------------------------------
// Dialog: purpose-based dismissal (backdrop press for "info"; Escape blocked for "required").

doc.addEventListener(
  'cancel',
  (e) => {
    const d = e.target;
    if (d instanceof HTMLDialogElement && d.dataset.purpose === 'required') e.preventDefault();
  },
  true,
);

let downOnBackdrop = null;
function outsideDialog(e) {
  const d = e.target;
  if (!(d instanceof HTMLDialogElement) || !d.classList.contains('astryx-dialog')) return null;
  const r = d.getBoundingClientRect();
  const out = e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom;
  return out ? d : null;
}
doc.addEventListener('pointerdown', (e) => {
  downOnBackdrop = outsideDialog(e);
});
doc.addEventListener('pointerup', (e) => {
  const d = outsideDialog(e);
  if (d && d === downOnBackdrop && d.dataset.purpose === 'info') d.close();
  downOnBackdrop = null;
});

// ---------------------------------------------------------------------------------------------
// Toasts.

const toastTimers = new WeakMap();

/** Starts a toast's auto-hide timer, or restarts it when a patch changed data-auto-hide. */
/** Start a toast's auto-hide timer once; restart it when its config changes or on `restart`
 * (an overwrite shows new content, so it gets the full duration again). */
function startToast(toast, restart = false) {
  const config = toast.dataset.autoHide ?? '';
  // What the toast says: a server patch (morphed in place) that changes it is a new message.
  const content = `${toast.dataset.type}|${toast.querySelector('.astryx-toast-content')?.innerHTML}|${
    toast.querySelector('.astryx-toast-end')?.textContent ?? ''
  }`;
  let state = toastTimers.get(toast);
  if (state) {
    if (state.config === config && state.content === content && !restart) return;
    clearTimeout(state.timer);
    state.timer = 0;
    state.config = config;
    state.content = content;
    state.remaining = Number(config);
    state.run();
    return;
  }
  state = { config, content, remaining: Number(config), startedAt: 0, timer: 0, pauses: new Set() };
  toastTimers.set(toast, state);
  const run = () => {
    if (!state.remaining || state.pauses.size > 0 || state.timer) return;
    state.startedAt = Date.now();
    state.timer = setTimeout(() => dismissToast(toast, 'auto'), state.remaining);
  };
  state.run = run;
  const pause = (source) => {
    state.pauses.add(source);
    if (!state.timer) return;
    clearTimeout(state.timer);
    state.timer = 0;
    state.remaining = Math.max(0, state.remaining - (Date.now() - state.startedAt));
  };
  const resume = (source) => {
    state.pauses.delete(source);
    run();
  };
  toast.addEventListener('mouseenter', () => pause('pointer'));
  toast.addEventListener('mouseleave', () => resume('pointer'));
  toast.addEventListener('focusin', () => pause('focus'));
  toast.addEventListener('focusout', (e) => {
    if (!toast.contains(e.relatedTarget)) resume('focus');
  });
  run();
}

function dismissToast(toast, reason) {
  const row = toast.closest('.astryx-toast-row');
  if (!row || row.hasAttribute('data-exiting')) return;
  clearTimeout(toastTimers.get(toast)?.timer);
  row.setAttribute('data-exiting', '');
  toast.dispatchEvent(new CustomEvent('astryx-toast-hide', { bubbles: true, detail: { reason } }));
  setTimeout(() => row.remove(), 300);
}

function syncViewport(viewport) {
  const rows = Array.from(viewport.querySelectorAll(':scope > .astryx-toast-row:not([data-exiting])'));
  const max = Number(viewport.dataset.maxVisible || 5);
  for (const row of rows.slice(0, Math.max(0, rows.length - max))) {
    const toast = row.querySelector('.astryx-toast');
    if (toast) dismissToast(toast, 'auto');
  }
  for (const toast of viewport.querySelectorAll('.astryx-toast')) startToast(toast);
  if (viewport.querySelector('.astryx-toast-row')) safeShow(viewport);
  else safeHide(viewport);
}

// A modal dialog makes everything outside it inert, so while one is open the toast viewport
// moves inside the topmost modal (dismiss buttons stay usable and the live region stays
// exposed), and moves back to its original place, marked by a comment, when the modal closes.
// Moving it also re-raises it above the modal in the top layer.
const toastHomes = new Map(); // viewport -> placeholder comment

// Modal dialogs in the order they opened (document order says nothing about which is on top).
let modalOrder = [];

function trackModal(d) {
  modalOrder = modalOrder.filter((m) => m !== d && m.isConnected && m.matches(':modal'));
  if (d.isConnected && d.matches(':modal')) modalOrder.push(d);
}

function topModal() {
  modalOrder = modalOrder.filter((m) => m.isConnected && m.matches(':modal'));
  for (const m of doc.querySelectorAll('dialog:modal')) if (!modalOrder.includes(m)) modalOrder.unshift(m);
  return modalOrder.at(-1) ?? null;
}

function homeToasts() {
  const modal = topModal();
  for (const v of doc.querySelectorAll('.astryx-toast-viewport')) {
    if (!toastHomes.has(v)) {
      const mark = doc.createComment('astryx-toast-viewport');
      v.before(mark);
      toastHomes.set(v, mark);
    }
    placeViewport(v, modal);
  }
}

function placeViewport(v, modal) {
  const mark = toastHomes.get(v);
  if (modal) {
    if (v.parentNode !== modal) modal.append(v);
  } else if (mark.isConnected) {
    if (mark.nextSibling !== v) mark.after(v);
  } else if (!v.isConnected) {
    doc.body.append(v);
  }
  // A move closes the popover; re-showing also puts it last (on top) in the top layer.
  if (v.matches(':popover-open')) safeHide(v);
  syncViewport(v);
}

/** Put back a viewport that a patch of its host dialog removed. */
function rescueToasts() {
  for (const [v, mark] of toastHomes) {
    if (v.isConnected) continue;
    if (mark.isConnected) placeViewport(v, topModal());
    else toastHomes.delete(v);
  }
}

function applyToast(t, opts) {
  const type = opts.type ?? 'info';
  t.dataset.type = type;
  t.setAttribute('role', type === 'error' ? 'alert' : 'status');
  t.setAttribute('aria-live', type === 'error' ? 'assertive' : 'polite');
  const autoHide = opts.autoHide ?? type !== 'error';
  if (autoHide) t.dataset.autoHide = String(opts.duration ?? 5000);
  else delete t.dataset.autoHide;
  t.querySelector('.astryx-toast-content').textContent = opts.body;
}

/**
 * Show a toast from the client. Uses the viewport's <template>, so it has the same markup as a
 * server-rendered Toast. Returns a function that dismisses it.
 * @param {{body: string, type?: 'info'|'error', autoHide?: boolean, duration?: number, id?: string, collision?: 'overwrite'|'ignore', viewport?: string}} opts
 */
function toast(opts) {
  const viewport = doc.getElementById(opts.viewport ?? 'astryx-toasts');
  const tpl = viewport?.querySelector('template.astryx-toast-template');
  if (!viewport || !tpl) throw new Error('canon.toast: render a ToastViewport first');
  if (opts.id) {
    const existing = doc.getElementById(opts.id);
    if (existing && !existing.hasAttribute('data-exiting')) {
      const t = existing.querySelector('.astryx-toast');
      if (opts.collision !== 'ignore' && t) {
        // Overwrite: new body, type and timing; startToast restarts the timer if it changed.
        applyToast(t, opts);
        startToast(t, true);
      }
      return () => t && dismissToast(t, 'manual');
    }
  }
  const row = tpl.content.firstElementChild.cloneNode(true);
  if (opts.id) row.id = opts.id;
  const t = row.querySelector('.astryx-toast');
  applyToast(t, opts);
  viewport.append(row);
  syncViewport(viewport);
  return () => dismissToast(t, 'manual');
}

// ---------------------------------------------------------------------------------------------
// Wiring markup as it appears (initial page and Datastar patches).

function popoverTrigger(anchor) {
  return anchor.matches('button, [role="button"]')
    ? anchor
    : anchor.querySelector('button, [role="button"]');
}

function addID(el, attr, id) {
  const ids = (el.getAttribute(attr) ?? '').split(/\s+/).filter(Boolean);
  if (!ids.includes(id)) el.setAttribute(attr, [...ids, id].join(' '));
}

function scan() {
  for (const w of doc.querySelectorAll('[data-tooltip]')) {
    const first = w.firstElementChild;
    if (first) addID(first, 'aria-describedby', w.dataset.tooltip);
    else {
      addID(w, 'aria-describedby', w.dataset.tooltip);
      // A plain-text trigger is a focus stop so keyboard users can reach its tooltip.
      if (w.dataset.focusTrigger !== 'never' && !w.hasAttribute('tabindex')) w.tabIndex = 0;
    }
  }
  for (const anchor of doc.querySelectorAll('[data-popover]')) {
    const trigger = popoverTrigger(anchor);
    const layer = doc.getElementById(anchor.dataset.popover);
    if (!trigger || !layer) continue;
    layer._canonAnchor = trigger;
    if (trigger.tagName === 'BUTTON' && trigger.getAttribute('popovertarget') !== layer.id) {
      trigger.setAttribute('popovertarget', layer.id);
    }
    if (!trigger.hasAttribute('aria-haspopup')) trigger.setAttribute('aria-haspopup', 'dialog');
    if (!trigger.hasAttribute('aria-controls')) trigger.setAttribute('aria-controls', layer.id);
    trigger.setAttribute('aria-expanded', String(layer.matches(':popover-open')));
  }
  for (const d of doc.querySelectorAll('dialog.astryx-dialog')) {
    if (d.hasAttribute('aria-label') || d.hasAttribute('aria-labelledby')) continue;
    const title = d.querySelector('.astryx-dialog-title');
    if (!title) continue;
    if (!title.id) title.id = `${d.id || 'astryx-dialog'}-title`;
    d.setAttribute('aria-labelledby', title.id);
  }
  for (const v of doc.querySelectorAll('.astryx-toast-viewport')) syncViewport(v);
}

let scanQueued = false;
function queueScan() {
  if (scanQueued) return;
  scanQueued = true;
  queueMicrotask(() => {
    scanQueued = false;
    scan();
  });
}

/** Forget layers a patch removed while open; a detached node gets no toggle event. */
function prune() {
  for (const layer of anchored.keys()) {
    if (layer.isConnected) continue;
    anchored.delete(layer);
    resizeObserver.unobserve(layer);
  }
  for (let i = stack.length - 1; i >= 0; i--) if (!stack[i].isConnected) stack.splice(i, 1);
}

new MutationObserver((records) => {
  if (records.some((r) => r.removedNodes.length > 0)) {
    prune();
    rescueToasts();
  }
  let raise = false;
  for (const r of records) {
    if (r.type === 'attributes' && r.attributeName === 'open' && r.target instanceof HTMLDialogElement) {
      trackModal(r.target);
      raise = true;
    }
  }
  if (raise) homeToasts();
  // Text edits matter only inside toasts (a morphed toast message restarts its timer).
  if (records.some((r) => r.type !== 'characterData' || r.target.parentElement?.closest('.astryx-toast'))) {
    queueScan();
  }
}).observe(doc.documentElement, {
  childList: true,
  subtree: true,
  characterData: true,
  attributes: true,
  attributeFilter: [
    'open',
    'popovertarget',
    'aria-describedby',
    'data-tooltip',
    'data-popover',
    'data-auto-hide',
    'data-type',
  ],
});
if (doc.readyState === 'loading') doc.addEventListener('DOMContentLoaded', scan, { once: true });
else scan();

// ---------------------------------------------------------------------------------------------
// API for Datastar expressions, e.g.
//   data-effect="canon.dialog(el, $confirmOpen)" data-on:close="$confirmOpen = false"

const resolve = (target) => (typeof target === 'string' ? doc.getElementById(target) : target);

const canon = {
  /** Open or close a Dialog (element or id). */
  dialog(target, open = true) {
    const d = resolve(target);
    if (!(d instanceof HTMLDialogElement)) return;
    if (open && !d.open) {
      d.showModal();
      trackModal(d);
      homeToasts();
    } else if (!open && d.open) d.close();
  },
  /** Open or close a Popover, DropdownMenu surface or Tooltip layer (element or id). */
  popover(target, open = true) {
    const p = resolve(target);
    if (!p) return;
    if (open) safeShow(p);
    else safeHide(p);
  },
  toast,
  /** Re-run wiring now (it also runs on every DOM change). */
  scan,
};
window.canon = canon;
export default canon;
