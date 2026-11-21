// canon.js: client behaviour for canon-datastar components: overlays (Tooltip, InfoTip, Popover,
// Dialog, DropdownMenu, TabList, Toast) and content (Collapsible, Banner, Avatar, Token,
// MetadataList, Thumbnail, Kbd, Timestamp). Ports the interaction logic of Astryx (packages/core/src),
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
  // The preferred side is the server's data-placement; ours (written below after a flip) is
  // remembered, so a patch that changes the attribute is told apart and becomes the new preference.
  if (layer._canonPlacement === undefined || layer.dataset.placement !== layer._canonWrote) {
    layer._canonPlacement = layer.dataset.placement || 'below';
  }
  const pref = layer._canonPlacement;
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
  layer._canonWrote = logical[side];
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
  // The preferred target only if it can take focus (not hidden or disabled), else the first
  // focusable, else the surface itself.
  const usable = focusables(root, true);
  const preferred = Array.from(root.querySelectorAll('[autofocus], [data-autofocus]')).find(
    (el) => usable.includes(el) || (el.tabIndex === -1 && el.getClientRects().length > 0 && !el.closest('[inert]')),
  );
  const target = preferred ?? usable[0];
  if (target) {
    target.focus({ preventScroll: true });
    if (doc.activeElement === target) return;
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
/** Whether `el` is the innermost open modal popover; non-modal layers above it (a child
 * popover or menu) don't suspend its focus trap. */
const isTopModal = (el) => stack.findLast((l) => l.matches('[data-modal]')) === el;

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
  // Skip disabled tabs and ones hidden (e.g. by data-show), which can't take focus.
  const stops = Array.from(root.querySelectorAll('[data-tab-value]')).filter(
    (el) => el.getAttribute('aria-disabled') !== 'true' && el.getClientRects().length > 0,
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
  // A modal dialog opened inside the popover owns Tab (natively) while it's open.
  const dialog = t.closest('dialog:modal');
  const dialogInside = dialog != null && layer?.contains(dialog);
  if (layer && e.key === 'Tab' && isTopModal(layer) && !dialogInside) trapTab(e, layer);

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
// Collapsible and Banner: expand and collapse, with a bubbling "canon-toggle" event.

function disclosure(root) {
  if (root.matches('.astryx-collapsible')) {
    return {
      trigger: root.querySelector(':scope > .astryx-collapsible-trigger'),
      content: root.querySelector(':scope > .astryx-collapsible-content'),
    };
  }
  return {
    trigger: root.querySelector(':scope > .astryx-banner .astryx-banner-toggle'),
    content: root.querySelector(':scope > .astryx-banner-content'),
  };
}

/** Opens or closes a Collapsible or Banner body; in a "single" group, closes the others. */
function setOpen(root, open) {
  const { trigger, content } = disclosure(root);
  if (!trigger || !content) return;
  if ((trigger.getAttribute('aria-expanded') === 'true') === open) return;
  // Focus inside content that's about to hide goes back to the trigger.
  if (!open && content.contains(doc.activeElement)) trigger.focus({ preventScroll: true });
  trigger.setAttribute('aria-expanded', String(open));
  content.hidden = !open;
  if (trigger.matches('.astryx-banner-toggle')) trigger.setAttribute('aria-label', open ? 'Collapse' : 'Expand');
  root.dispatchEvent(new CustomEvent('canon-toggle', { bubbles: true, detail: { open } }));
  const group = root.parentElement;
  if (open && root.dataset.value && group?.matches('.astryx-collapsible-group[data-type="single"]')) {
    for (const other of group.querySelectorAll(':scope > .astryx-collapsible[data-value]')) {
      if (other !== root) setOpen(other, false);
    }
  }
}

// Where focus was before it entered a Banner, so dismissing can send it back.
function rememberOrigin(candidate, frame) {
  if (candidate instanceof HTMLElement && candidate !== doc.body && !frame.contains(candidate)) {
    frame._canonOrigin = candidate;
  }
}
doc.addEventListener(
  'focusin',
  (e) => {
    const frame = e.target instanceof Element && e.target.closest('.astryx-banner-frame');
    if (frame) rememberOrigin(e.relatedTarget, frame);
  },
  true,
);
doc.addEventListener(
  'pointerdown',
  (e) => {
    const frame = e.target instanceof Element && e.target.closest('.astryx-banner-frame');
    if (frame) rememberOrigin(doc.activeElement, frame);
  },
  true,
);

// ---------------------------------------------------------------------------------------------
// Avatar and Thumbnail images that fail to load.

function imageFailed(img) {
  if (img.matches('.astryx-avatar-image')) {
    const next = img.dataset.fallbackSrc;
    if (next) {
      img.removeAttribute('data-fallback-src');
      img.src = next;
      return;
    }
    img.hidden = true;
    const fallback = img.parentElement?.querySelector(':scope > .astryx-avatar-fallback');
    if (fallback) fallback.hidden = false;
  } else if (img.matches('.astryx-thumbnail-image')) {
    img.hidden = true;
    const frame = img.closest('.astryx-thumbnail-frame');
    const border = frame?.querySelector(':scope > .astryx-thumbnail-inset-border');
    if (border) border.hidden = true;
    const placeholder = frame?.querySelector('.astryx-thumbnail-placeholder');
    if (placeholder) placeholder.hidden = false;
  }
}
doc.addEventListener(
  'error',
  (e) => {
    if (e.target instanceof HTMLImageElement) imageFailed(e.target);
  },
  true,
);
const IMAGES = '.astryx-avatar-image:not([hidden]), .astryx-thumbnail-image:not([hidden])';
/** Images that failed before this module loaded (or were patched in already broken). */
function checkImages() {
  for (const img of doc.querySelectorAll(IMAGES)) {
    if (img.complete && img.naturalWidth === 0 && img.getAttribute('src')) imageFailed(img);
  }
}

// ---------------------------------------------------------------------------------------------
// Clicks: Collapsible, Banner toggle and dismiss, Token body, MetadataList "Show more".

doc.addEventListener('click', (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;

  const trigger = t.closest('.astryx-collapsible-trigger, .astryx-banner-toggle');
  if (trigger && trigger.getAttribute('aria-disabled') !== 'true') {
    const root = trigger.closest('.astryx-collapsible, .astryx-banner-frame');
    if (root) setOpen(root, trigger.getAttribute('aria-expanded') !== 'true');
  }

  const dismiss = t.closest('[data-banner-dismiss]');
  if (dismiss) {
    const frame = dismiss.closest('.astryx-banner-frame');
    if (frame) {
      const origin = frame._canonOrigin;
      frame.hidden = true;
      frame.dispatchEvent(new CustomEvent('canon-dismiss', { bubbles: true }));
      if (origin?.isConnected) origin.focus({ preventScroll: true });
    }
  }

  // Clicking a Token's body (not a control inside it) activates its label link or button.
  const owner = t.closest('.astryx-token[data-owner]');
  if (owner && !owner.hasAttribute('data-disabled')) {
    const nested = t.closest('a[href], button, input, select, textarea, summary, label, [role="button"], [role="link"], [tabindex]');
    const inner = owner.querySelector(':scope > .astryx-token-inner');
    if (inner && (!nested || !owner.contains(nested) || nested === owner)) {
      // Keep the modifiers so Ctrl/Cmd/Shift-click keep their link meaning.
      const { ctrlKey, metaKey, shiftKey, altKey, button } = e;
      inner.dispatchEvent(
        new MouseEvent('click', { bubbles: true, cancelable: true, ctrlKey, metaKey, shiftKey, altKey, button }),
      );
    }
  }

  const more = t.closest('.astryx-metadata-list-toggle');
  if (more) {
    const list = doc.getElementById(more.getAttribute('aria-controls') ?? '');
    const max = Number(list?.dataset.max ?? 0);
    if (list && max > 0) {
      const showAll = more.getAttribute('aria-expanded') !== 'true';
      [...list.querySelectorAll(':scope > [data-metadata-item]')].forEach((item, i) => {
        const hide = !showAll && i >= max;
        item.hidden = hide;
        const value = item.nextElementSibling;
        if (item.tagName === 'DT' && value?.tagName === 'DD') value.hidden = hide;
      });
      more.setAttribute('aria-expanded', String(showAll));
      more.textContent = showAll ? 'Show less' : 'Show more';
    }
  }
});

// ---------------------------------------------------------------------------------------------
// AvatarGroup: interactive avatars are one tab stop, moved with arrows, Home and End.

doc.addEventListener('focusin', (e) => {
  const item = e.target instanceof Element && e.target.closest('[data-avatar-item]');
  const group = item && item.closest('.astryx-avatar-group');
  if (!group) return;
  for (const el of group.querySelectorAll('[data-avatar-item]')) el.tabIndex = el === item ? 0 : -1;
});
doc.addEventListener('keydown', (e) => {
  const item = e.target instanceof Element && e.target.closest('[data-avatar-item]');
  const group = item && item.closest('.astryx-avatar-group');
  if (!group || e.altKey || e.ctrlKey || e.metaKey) return;
  const list = [...group.querySelectorAll('[data-avatar-item]')];
  const i = list.indexOf(item);
  const rtl = getComputedStyle(group).direction === 'rtl';
  let next;
  if (e.key === 'ArrowRight' || e.key === 'ArrowDown') next = rtl && e.key === 'ArrowRight' ? i - 1 : i + 1;
  else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') next = rtl && e.key === 'ArrowLeft' ? i + 1 : i - 1;
  else if (e.key === 'Home') next = 0;
  else if (e.key === 'End') next = list.length - 1;
  else return;
  e.preventDefault();
  list[Math.max(0, Math.min(list.length - 1, next))].focus();
});

// ---------------------------------------------------------------------------------------------
// Kbd: Apple platforms show ⌘ ⌃ ⌥ ⇧ (the server renders the Ctrl/Alt/Shift form).

const isApple = (() => {
  const named = navigator.userAgentData?.platform?.trim();
  if (named) return /mac|ios|iphone|ipad/i.test(named);
  return /Mac|iPhone|iPad|iPod/.test(navigator.platform || navigator.userAgent);
})();
const APPLE_KEYS = { mod: '⌘', meta: '⌘', ctrl: '⌃', alt: '⌥', shift: '⇧' };

function fixKbd(kbd) {
  const keys = (kbd.dataset.keys ?? '').split('+');
  const caps = kbd.querySelectorAll(':scope > .astryx-kbd-key');
  keys.forEach((k, i) => {
    if (APPLE_KEYS[k] && caps[i] && caps[i].textContent !== APPLE_KEYS[k]) caps[i].textContent = APPLE_KEYS[k];
  });
  const label = keys.map((k) => (k === 'mod' ? 'Command' : null));
  if (label.some(Boolean)) {
    const parts = (kbd.getAttribute('aria-label') ?? '').split(' + ');
    const next = parts.map((p, i) => label[i] ?? p).join(' + ');
    if (next !== kbd.getAttribute('aria-label')) kbd.setAttribute('aria-label', next);
  }
}

// ---------------------------------------------------------------------------------------------
// Timestamp: Intl formatting in the viewer's zone. Port of Astryx Timestamp (formatRelativeTime,
// formatInstant, tooltipEntries), MIT, Meta Platforms, Inc. Gregorian calendar only.

const MINUTE = 60;
const HOUR = 3600;
const DAY = 86400;
const MONTH = 30 * DAY;
const YEAR = 365 * DAY;
const FUTURE_SKEW_TOLERANCE = 30;

function formatRelative(date, now, locale, style) {
  const diff = Math.round((now.getTime() - date.getTime()) / 1000);
  const abs = Math.abs(diff);
  const rtf = (numeric) => new Intl.RelativeTimeFormat(locale, { numeric, style });
  if (abs < 10 || (diff < 0 && abs <= FUTURE_SKEW_TOLERANCE)) return rtf('auto').format(0, 'second');
  let count;
  let unit;
  if (abs < MINUTE) [count, unit] = [abs, 'second'];
  else if (abs < HOUR) [count, unit] = [Math.floor(abs / MINUTE), 'minute'];
  else if (abs < DAY) [count, unit] = [Math.floor(abs / HOUR), 'hour'];
  else if (abs < MONTH) [count, unit] = [Math.floor(abs / DAY), 'day'];
  else if (abs < YEAR) [count, unit] = [Math.floor(abs / MONTH), 'month'];
  else [count, unit] = [Math.floor(abs / YEAR), 'year'];
  const value = diff < 0 ? count : -count;
  return rtf(style === 'long' && unit === 'day' && value === -1 ? 'auto' : 'always').format(value, unit);
}

const TIME_OPTIONS = { hour: 'numeric', minute: '2-digit' };
const INSTANT_OPTIONS = {
  date: { year: 'numeric', month: 'short', day: 'numeric' },
  date_long: { year: 'numeric', month: 'long', day: 'numeric' },
  date_weekday: { weekday: 'short', month: 'short', day: 'numeric', year: 'numeric' },
  date_time: { year: 'numeric', month: 'short', day: 'numeric', ...TIME_OPTIONS },
  time: TIME_OPTIONS,
  full: { year: 'numeric', month: 'long', day: 'numeric', hour: 'numeric', minute: '2-digit', second: '2-digit' },
};
const pad = (n) => String(n).padStart(2, '0');

function wallClock(date, timeZone) {
  if (!timeZone) {
    return {
      year: date.getFullYear(),
      month: date.getMonth() + 1,
      day: date.getDate(),
      hour: date.getHours(),
      minute: date.getMinutes(),
      second: date.getSeconds(),
    };
  }
  const parts = new Intl.DateTimeFormat('en-US', {
    timeZone,
    hourCycle: 'h23',
    year: 'numeric',
    month: 'numeric',
    day: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
    second: 'numeric',
  }).formatToParts(date);
  const get = (type) => Number(parts.find((p) => p.type === type)?.value ?? 0);
  return { year: get('year'), month: get('month'), day: get('day'), hour: get('hour'), minute: get('minute'), second: get('second') };
}

function formatZoneName(date, locale, timeZone) {
  return (
    new Intl.DateTimeFormat(locale, { timeZoneName: 'long', ...(timeZone ? { timeZone } : {}) })
      .formatToParts(date)
      .find((p) => p.type === 'timeZoneName')?.value ?? ''
  );
}

function formatInstant(date, format, locale, { timeZone, isTimezoneShown = false, timeZoneNameStyle = 'short' } = {}) {
  const zone = timeZone ? { timeZone } : {};
  const fmt = (options) => new Intl.DateTimeFormat(locale, { ...options, ...zone }).format(date);
  switch (format) {
    case 'full':
      return fmt({ ...INSTANT_OPTIONS.full, timeZoneName: timeZoneNameStyle });
    case 'date_time':
    case 'time':
      return fmt({ ...INSTANT_OPTIONS[format], ...(isTimezoneShown ? { timeZoneName: 'short' } : {}) });
    case 'system_date': {
      const w = wallClock(date, timeZone);
      return `${w.year}-${pad(w.month)}-${pad(w.day)}`;
    }
    case 'system_date_time': {
      const w = wallClock(date, timeZone);
      return `${w.year}-${pad(w.month)}-${pad(w.day)} ${pad(w.hour)}:${pad(w.minute)}:${pad(w.second)}`;
    }
    case 'system_time': {
      const w = wallClock(date, timeZone);
      return `${pad(w.hour)}:${pad(w.minute)}:${pad(w.second)}`;
    }
    case 'unix_seconds':
      return String(Math.floor(date.getTime() / 1000));
    default:
      return fmt(INSTANT_OPTIONS[format] ?? INSTANT_OPTIONS.date_time);
  }
}

/** Unknown zones and "local" resolve to the viewer's zone (undefined). */
function resolveZone(id) {
  if (!id || id.toLowerCase() === 'local') return undefined;
  try {
    new Intl.DateTimeFormat('en-US', { timeZone: id });
    return id;
  } catch {
    console.warn(`[Timestamp] unknown time zone ${JSON.stringify(id)}. Falling back to the viewer's time zone.`);
    return undefined;
  }
}

const isRelative = (f) => f === 'relative' || f === 'relative_short';
/** Live relative timestamps and when each is next due. */
const liveTimes = new Map();

// Focus on the time (or a control it names) holds live refreshes, so a screen reader isn't
// re-reading a changing label.
const NAMED_FROM_CONTENT =
  'button,summary,a[href],[role=button],[role=link],[role=tab],[role^=menuitem],[role=option],[role=radio],[role=checkbox],[role=switch],[role=treeitem]';
function holdsRefresh(time) {
  const f = doc.activeElement;
  return (
    f === time ||
    (f != null && f.contains(time) && f.matches(NAMED_FROM_CONTENT) && !f.matches('[aria-label],[aria-labelledby]')) ||
    (f != null && time.closest('label')?.control === f)
  );
}

function renderTime(time, force = false) {
  const d = time.dataset;
  const date = new Date(time.getAttribute('datetime') ?? '');
  if (Number.isNaN(date.getTime())) return;
  const sig = [time.getAttribute('datetime'), d.format, d.locale, d.timezone, d.threshold, d.timezoneShown, d.live].join('|');
  if (!force && time._canonSig === sig && time.textContent === time._canonText) return;
  time._canonSig = sig;

  const now = new Date();
  const locale = d.locale || 'en-US';
  const format = d.format || 'auto';
  const threshold = Number(d.threshold ?? 7 * DAY);
  const diff = Math.round((now.getTime() - date.getTime()) / 1000);
  const effective = format === 'auto' ? (Math.abs(diff) <= threshold ? 'relative' : 'date_time') : format;
  const timeZone = resolveZone(d.timezone);
  const text = isRelative(effective)
    ? formatRelative(date, now, locale, effective === 'relative' ? 'long' : 'narrow')
    : formatInstant(date, effective, locale, { timeZone, isTimezoneShown: d.timezoneShown !== undefined });
  if (time.textContent !== text) time.textContent = text;
  time._canonText = text;
  const host = time.closest('.astryx-timestamp');
  if (host && host.dataset.timestampFormat !== effective) host.dataset.timestampFormat = effective;

  // The spoken label adds the full date to a relative time (unless the page set its own label).
  const current = time.getAttribute('aria-label');
  if (current === null || current === time._canonLabel) {
    const label = isRelative(effective) ? `${text}, ${formatInstant(date, 'full', locale, { timeZoneNameStyle: 'long' })}` : null;
    if (label === null) time.removeAttribute('aria-label');
    else if (label !== current) time.setAttribute('aria-label', label);
    time._canonLabel = label;
  }

  // Details tooltip: the default one shows only for relative times.
  const wrapper = time.parentElement?.matches('.astryx-tooltip-trigger') ? time.parentElement : null;
  const tipId = wrapper ? (wrapper.dataset.tooltip ?? wrapper._canonTip) : undefined;
  const details = tipId ? doc.getElementById(tipId)?.querySelector('.astryx-timestamp-details') : null;
  if (wrapper && details) {
    wrapper._canonTip = tipId;
    const isDefault = details.hasAttribute('data-default');
    const on = !isDefault || isRelative(effective);
    if (on && !wrapper.dataset.tooltip) wrapper.dataset.tooltip = tipId;
    if (!on && wrapper.dataset.tooltip) wrapper.removeAttribute('data-tooltip');
    if (on) time.tabIndex = 0;
    else time.removeAttribute('tabindex');
    const rows = [...details.querySelectorAll(':scope > .astryx-timestamp-row')];
    // The default line is in the viewer's zone; an entry without a zone takes the timestamp's.
    const zones = rows.map((r) =>
      isDefault ? undefined : r.dataset.timezone === undefined ? timeZone : resolveZone(r.dataset.timezone),
    );
    const keys = new Set(zones.map((z) => (z === undefined ? 'local' : z.toLowerCase())));
    if (timeZone !== undefined) keys.add(timeZone.toLowerCase());
    rows.forEach((row, i) => {
      const fmt = row.dataset.format || 'full';
      const zone = zones[i];
      const showZone = fmt === 'full' || ((fmt === 'date_time' || fmt === 'time') && (keys.size > 1 || zone !== undefined));
      const value = row.querySelector('.astryx-timestamp-value');
      const v = formatInstant(date, fmt, locale, { timeZone: zone, isTimezoneShown: showZone });
      if (value && value.textContent !== v) value.textContent = v;
      const name = row.querySelector('[data-zone-name]');
      const z = formatZoneName(date, locale, zone);
      if (name && name.textContent !== z) name.textContent = z;
    });
  }

  if (d.live !== undefined && isRelative(effective)) {
    const abs = Math.abs(diff);
    const every = abs < MINUTE ? 1000 : abs < HOUR ? 30_000 : abs < DAY ? 60_000 : 300_000;
    liveTimes.set(time, Date.now() + every);
  } else liveTimes.delete(time);
}

setInterval(() => {
  const now = Date.now();
  for (const [time, due] of liveTimes) {
    if (!time.isConnected) liveTimes.delete(time);
    else if (now >= due && !holdsRefresh(time)) renderTime(time, true);
  }
}, 1000);
// Catch up as soon as focus leaves a held time.
doc.addEventListener('focusout', () => {
  queueMicrotask(() => {
    for (const [time, due] of liveTimes) if (Date.now() >= due && !holdsRefresh(time)) renderTime(time, true);
  });
});

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
  checkImages();
  if (isApple) for (const k of doc.querySelectorAll('.astryx-kbd[data-keys]')) fixKbd(k);
  for (const t of doc.querySelectorAll('time.astryx-timestamp-time')) renderTime(t);
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
  // Text edits matter only inside toasts (a morphed toast message restarts its timer) and
  // timestamps (a morph puts the server's fallback text back).
  if (records.some((r) => r.type !== 'characterData' || r.target.parentElement?.closest('.astryx-toast, .astryx-timestamp-time'))) {
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
    'datetime',
    'src',
    'data-keys',
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
  /** Expand or collapse a Collapsible or Banner (element or id). */
  collapse(target, open = true) {
    const root = resolve(target)?.closest('.astryx-collapsible, .astryx-banner-frame');
    if (root) setOpen(root, Boolean(open));
  },
  toast,
  /** Re-run wiring now (it also runs on every DOM change). */
  scan,
};
window.canon = canon;
export default canon;
