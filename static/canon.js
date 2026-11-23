// canon.js: client behaviour for canon-datastar components: overlays (Tooltip, InfoTip, Popover,
// Dialog, DropdownMenu, TabList, Toast), content (Collapsible, Banner, Avatar, Token,
// MetadataList, Thumbnail, Kbd, Timestamp) and forms (TextArea, Item, CheckboxList, RadioList,
// NumberInput, Slider, FileInput, Selector). It imports ./number-parser.js, so serve both. Ports the interaction logic of Astryx (packages/core/src),
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

import { formatEditableNumber, getNumberInputRejection, parseNumberInput, resolveNumberInputCommit } from './number-parser.js';

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
// Forms.

const segmenter = typeof Intl.Segmenter === 'function' ? new Intl.Segmenter(undefined, { granularity: 'grapheme' }) : null;
/** User-perceived characters, so an emoji counts as one (Astryx utils/characters). */
function characterCount(str) {
  if (str === '') return 0;
  if (!segmenter) return [...str].length;
  let n = 0;
  for (const _ of segmenter.segment(str)) n++;
  return n;
}

const plural = (n) => `${n.toLocaleString()} ${n === 1 ? 'character' : 'characters'}`;

/** Refresh a TextArea's counter; with announce, speak once on entering "near" or "over". */
function countTextArea(ta, announce = false) {
  const max = Number(ta.dataset.maxLength);
  const box = ta.closest('.astryx-text-area');
  const counter = box?.querySelector(':scope > .astryx-text-area-counter');
  const count = counter?.querySelector('.astryx-text-area-count');
  if (!max || !count) return;
  const n = characterCount(ta.value);
  const over = n > max;
  const label = `${n.toLocaleString()}/${max.toLocaleString()}`;
  if (count.textContent !== label) count.textContent = label;
  if (counter.hasAttribute('data-over') !== over) counter.toggleAttribute('data-over', over);
  const invalid = over || box.dataset.status === 'error';
  if (invalid) ta.setAttribute('aria-invalid', 'true');
  else ta.removeAttribute('aria-invalid');
  if (!announce) return;
  const zone = over ? 'over' : n >= max * 0.8 ? 'near' : 'under';
  if (zone === ta._canonZone) return;
  ta._canonZone = zone;
  const field = box.parentElement?.closest('.astryx-field') ?? box.parentElement;
  if (zone === 'over') {
    const live = field?.querySelector('.astryx-text-area-assertive');
    if (live) live.textContent = `${plural(n - max)} over the limit`;
  } else if (zone === 'near') {
    const live = field?.querySelector('.astryx-text-area-polite');
    if (live) live.textContent = `${plural(max - n)} remaining`;
  }
}

doc.addEventListener('input', (e) => {
  const t = e.target;
  if (t instanceof HTMLTextAreaElement && t.matches('.astryx-text-area-control[data-max-length]')) countTextArea(t, true);
});

// Clicking the padding around a TextArea focuses it.
doc.addEventListener('click', (e) => {
  const box = e.target instanceof Element && e.target.matches('.astryx-text-area') ? e.target : null;
  if (!box || box.hasAttribute('data-disabled')) return;
  const ta = box.querySelector('.astryx-text-area-control');
  if (ta && !ta.matches('[aria-busy="true"]')) ta.focus();
});

// Item rows that delegate: a click on the row's empty space clicks the control in it.
doc.addEventListener('click', (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;
  const row = t.closest('.astryx-item[data-delegate]');
  if (!row || row.hasAttribute('data-disabled')) return;
  const control = row.querySelector(':scope > .astryx-item-start input, :scope > .astryx-item-start button');
  if (!control || control.contains(t) || t.closest('button, a, input, select, textarea, label')) return;
  control.click();
  control.focus({ preventScroll: true });
});

/** A required CheckboxList: only the first enabled box is required, and only while none is checked. */
function syncRequired(group) {
  const boxes = [...group.querySelectorAll('input[type="checkbox"]')];
  const empty = !boxes.some((b) => b.checked);
  const target =
    group.hasAttribute('data-required') && empty
      ? boxes.find((b) => !b.disabled && !b.hasAttribute('aria-readonly'))
      : undefined;
  for (const b of boxes) if (b.required !== (b === target)) b.required = b === target;
}
doc.addEventListener('change', (e) => {
  const group = e.target instanceof Element ? e.target.closest('.astryx-checkbox-list-group') : null;
  if (group) syncRequired(group);
});
/** Put a hidden input back to its wiring-time value; true (and marked) when that changed it. */
function restoreDefault(h) {
  if (h._canonDefault === undefined || nativeValue.get.call(h) === h._canonDefault) return false;
  nativeValue.set.call(h, h._canonDefault);
  h._canonRestored = true;
  return true;
}
/** Tell data-bind about a restored value. */
function notify(h) {
  if (!h._canonRestored) return;
  h._canonRestored = false;
  h.dispatchEvent(new Event('input', { bubbles: true }));
  h.dispatchEvent(new Event('change', { bubbles: true }));
}

// A form reset restores controls without events; resync once the browser is done.
doc.addEventListener('reset', (e) => {
  const form = e.target;
  if (!(form instanceof HTMLFormElement)) return;
  // A <select multiple> resets itself; remember the values to tell data-bind if they change.
  const multiBefore = new Map();
  for (const ctl of doc.querySelectorAll('.astryx-selector-control[data-multiple]')) {
    if (selParts(ctl).hidden?.form === form) multiBefore.set(ctl, selValues(ctl).join('\u0000'));
  }
  setTimeout(() => {
    if (e.defaultPrevented) return; // a listener cancelled the reset
    for (const el of form.elements) {
      if (el instanceof HTMLTextAreaElement && el.matches('.astryx-text-area-control[data-max-length]')) countTextArea(el);
    }
    for (const g of doc.querySelectorAll('.astryx-checkbox-list-group')) {
      if (g.querySelector('input[type="checkbox"]')?.form === form) syncRequired(g);
    }
    // Hidden inputs have no separate default (setting .value rewrites the attribute), so the
    // values recorded at wiring time are restored by hand.
    for (const ctl of doc.querySelectorAll('.astryx-number-input')) {
      const { hidden } = numberParts(ctl);
      if (hidden?.form !== form || hidden._canonDefault === undefined) continue;
      endNumberDraft(ctl);
      restoreDefault(hidden);
      renderNumber(ctl);
      notify(hidden);
    }
    for (const ctl of doc.querySelectorAll('.astryx-selector-control')) {
      const { hidden } = selParts(ctl);
      if (hidden?.form !== form) continue;
      if (multiBefore.has(ctl)) {
        if (multiBefore.get(ctl) !== selValues(ctl).join('\u0000')) selChanged(ctl);
        else renderSelector(ctl);
        continue;
      }
      restoreDefault(hidden);
      renderSelector(ctl);
      notify(hidden);
    }
    for (const ctl of doc.querySelectorAll('.astryx-slider-control')) {
      const hiddens = sliderHiddens(ctl);
      if (hiddens[0]?.form !== form) continue;
      const changed = hiddens.filter(restoreDefault);
      renderSlider(ctl);
      for (const h of changed) notify(h);
    }
    for (const box of doc.querySelectorAll('.astryx-file-input')) {
      const input = fileNative(box);
      if (input?.form !== form) continue;
      const had = box.hasAttribute('data-has-files');
      renderFiles(box, null);
      // data-bind reads files on change only; let it see the emptied selection.
      if (had && input.files.length === 0) input.dispatchEvent(new Event('change', { bubbles: true }));
    }
  }, 0);
});

// RadioList: with nothing checked, focus entering the group lands on the first radio (the
// last when tabbing backwards), and Tab from a radio leaves the group in one step.
doc.addEventListener('focusin', (e) => {
  const t = e.target;
  const group = t instanceof HTMLInputElement && t.type === 'radio' ? t.closest('.astryx-radio-list') : null;
  if (!group) return;
  const radios = [...group.querySelectorAll('input[type="radio"]:not([disabled])')].filter((r) => r.name === t.name);
  if (radios.some((r) => r.checked)) return;
  if (e.relatedTarget instanceof Node && group.contains(e.relatedTarget)) return;
  const index = radios.indexOf(t);
  if (index === -1) return;
  const intended = index === radios.length - 1 ? radios[radios.length - 1] : radios[0];
  if (intended !== t) intended.focus();
});
doc.addEventListener('keydown', (e) => {
  if (e.key !== 'Tab' || e.ctrlKey || e.metaKey) return;
  const from = e.target;
  const group = from instanceof HTMLInputElement && from.type === 'radio' ? from.closest('.astryx-radio-list') : null;
  if (!group) return;
  const others = [...group.querySelectorAll('input[type="radio"]')].filter((r) => r !== from && r.name === from.name);
  const saved = others.map((r) => [r, r.getAttribute('tabindex')]);
  for (const r of others) r.tabIndex = -1;
  setTimeout(() => {
    for (const [r, t] of saved) {
      if (t == null) r.removeAttribute('tabindex');
      else r.setAttribute('tabindex', t);
    }
  }, 0);
});

// NumberInput: typing is a draft that commits on blur or Enter. The committed number lives in
// the hidden .astryx-number-input-value input (where data-bind goes).

/** Set an id-list attribute only when it changes (aria-describedby edits trigger a rescan). */
function setIDs(el, attr, ids) {
  const next = ids.join(' ');
  if ((el.getAttribute(attr) ?? '') === next) return;
  if (next) el.setAttribute(attr, next);
  else el.removeAttribute(attr);
}

const nativeValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value');
const numberParts = (ctl) => ({
  field: ctl.querySelector(':scope > .astryx-number-input-field'),
  hidden: ctl.querySelector(':scope > .astryx-number-input-value'),
});
const optNumber = (s) => (s === undefined || s === '' ? null : Number(s));

function numberState(ctl) {
  if (!ctl._canonNumber) {
    ctl._canonNumber = { pending: null, focused: false, rejection: null, kept: false, holdDraft: null, hold: null, clickAfterHold: false };
  }
  return ctl._canonNumber;
}
const numberRules = (ctl) => ({
  min: optNumber(ctl.dataset.min),
  max: optNumber(ctl.dataset.max),
  isIntegerOnly: ctl.hasAttribute('data-integer'),
  locale: ctl.dataset.locale || undefined,
});
const numberInactive = (ctl) => ctl.hasAttribute('data-disabled') || ctl.hasAttribute('data-readonly');

function committedNumber(ctl) {
  const { hidden } = numberParts(ctl);
  const raw = hidden ? nativeValue.get.call(hidden) : '';
  if (raw === '') return null;
  const n = Number(raw);
  return Number.isFinite(n) ? n : null;
}

function formatNumber(ctl, value) {
  const options = ctl.dataset.format;
  if (!options) return null;
  try {
    return new Intl.NumberFormat(ctl.dataset.locale || undefined, JSON.parse(options)).format(value);
  } catch {
    return null;
  }
}

function decimalPlaces(n) {
  const [coefficient, exponentText] = String(n).toLowerCase().split('e');
  const fraction = coefficient.split('.')[1]?.length ?? 0;
  return Math.max(0, fraction - (exponentText == null ? 0 : Number(exponentText)));
}

function steppedValue(ctl, current, direction) {
  const { min, max, isIntegerOnly } = numberRules(ctl);
  const step = optNumber(ctl.dataset.step);
  const effective = step == null || !Number.isFinite(step) || step <= 0 || (isIntegerOnly && !Number.isInteger(step)) ? 1 : step;
  const base = min != null && (!isIntegerOnly || Number.isInteger(min)) ? min : 0;
  let next;
  if (current == null) {
    next = direction === 1 ? (min ?? 0) : (max ?? 0);
    if (isIntegerOnly) next = direction === 1 ? Math.ceil(next) : Math.floor(next);
  } else {
    const position = (current - base) / effective;
    const tolerance = Number.EPSILON * Math.max(1, Math.abs(position)) * 4;
    const nextPosition = direction === 1 ? Math.floor(position + tolerance) + 1 : Math.ceil(position - tolerance) - 1;
    next = base + nextPosition * effective;
  }
  next = Number(next.toFixed(Math.min(100, Math.max(decimalPlaces(effective), decimalPlaces(base)))));
  if (min != null) next = Math.max(min, next);
  if (max != null) next = Math.min(max, next);
  if (!Number.isFinite(next) || (isIntegerOnly && !Number.isInteger(next))) return current;
  return Object.is(next, -0) ? 0 : next;
}

function valueForStepping(ctl) {
  const s = numberState(ctl);
  const value = committedNumber(ctl);
  if (s.pending === null) return value;
  if (s.pending.trim() === '') return null;
  return parseNumberInput(s.pending, numberRules(ctl)) ?? value;
}

/** Bring the visible text, ARIA, clear button, steppers and error state up to date. */
function renderNumber(ctl) {
  const { field } = numberParts(ctl);
  if (!field) return;
  const s = numberState(ctl);
  const locale = ctl.dataset.locale || undefined;
  const value = committedNumber(ctl);
  const formatted = value == null ? '' : (formatNumber(ctl, value) ?? String(value));
  const text =
    s.holdDraft !== null
      ? formatEditableNumber(s.holdDraft, locale)
      : s.pending !== null
        ? s.pending
        : value == null
          ? ''
          : s.focused
            ? formatEditableNumber(value, locale)
            : formatted;
  if (field.value !== text) field.value = text;
  if (value == null) field.removeAttribute('aria-valuenow');
  else field.setAttribute('aria-valuenow', String(value));
  if (value != null && ctl.dataset.format && formatted !== String(value)) field.setAttribute('aria-valuetext', formatted);
  else field.removeAttribute('aria-valuetext');
  const clear = ctl.querySelector(':scope > [data-number-input-clear]');
  if (clear) clear.hidden = value == null;
  const current = s.holdDraft ?? valueForStepping(ctl);
  for (const b of ctl.querySelectorAll(':scope > .astryx-number-input-steppers > .astryx-number-input-stepper')) {
    const dir = b.dataset.direction === 'increment' ? 1 : -1;
    const off = numberInactive(ctl) || steppedValue(ctl, current, dir) === current;
    if (off) b.setAttribute('aria-disabled', 'true');
    else b.removeAttribute('aria-disabled');
  }
  // A rejected draft on screen (hidden while disabled or holding a stepper).
  const shown = !ctl.hasAttribute('data-disabled') && s.holdDraft === null && s.rejection !== null;
  ctl.toggleAttribute('data-entry-error', shown);
  const callerError = ctl.dataset.status === 'error';
  if (shown || callerError) field.setAttribute('aria-invalid', 'true');
  else field.removeAttribute('aria-invalid');
  // A rejected draft also blocks explicit form submission, not just Enter.
  const validity = shown ? 'Invalid number' : '';
  if (field.validationMessage !== validity) field.setCustomValidity(validity);
  const fieldEl = ctl.closest('.astryx-field');
  const entry = fieldEl && !ctl.hasAttribute('data-astryx-input-group-item') ? doc.getElementById(`${field.id}-entry`) : null;
  if (entry) {
    entry.hidden = !shown;
    const own = doc.getElementById(`${field.id}-status`);
    if (own && own !== entry) own.hidden = shown;
    const ids = (field.getAttribute('aria-describedby') ?? '').split(/\s+/).filter((x) => x && x !== entry.id);
    const at = own ? ids.indexOf(own.id) : -1;
    if (shown) ids.splice(at === -1 ? ids.length : at, at === -1 ? 0 : 1, entry.id);
    else if (own && !ids.includes(own.id) && own.textContent) ids.push(own.id);
    setIDs(field, 'aria-describedby', ids);
  }
  const alert = ctl.querySelector(':scope > .astryx-number-input-alert');
  const alertText = shown && !ctl.hasAttribute('data-error-message') && !entry ? 'Invalid number' : '';
  if (alert && alert.textContent !== alertText) alert.textContent = alertText;
}

function emitNumber(ctl, next) {
  const { hidden } = numberParts(ctl);
  if (!hidden) return;
  nativeValue.set.call(hidden, next == null ? '' : String(next));
  renderNumber(ctl);
  hidden.dispatchEvent(new Event('input', { bubbles: true }));
  hidden.dispatchEvent(new Event('change', { bubbles: true }));
}

function endNumberDraft(ctl) {
  const s = numberState(ctl);
  s.pending = null;
  s.rejection = null;
  s.kept = false;
}

/** Commits the draft; false when it was rejected and stays on screen. */
function commitNumber(ctl, trigger) {
  const s = numberState(ctl);
  if (s.pending === null) return true;
  if (ctl.hasAttribute('data-disabled')) {
    endNumberDraft(ctl);
    return true;
  }
  const rules = numberRules(ctl);
  const rejection = getNumberInputRejection(s.pending, rules);
  if (rejection != null) {
    s.rejection = rejection;
    if (trigger === 'blur') s.kept = true;
    renderNumber(ctl);
    return false;
  }
  s.rejection = null;
  s.kept = false;
  const hasClear = ctl.hasAttribute('data-clear');
  const decision = resolveNumberInputCommit(s.pending, { ...rules, hasClear });
  if (trigger === 'blur' || decision.type === 'revert' || (decision.type === 'commit' && decision.didClamp)) s.pending = null;
  const value = committedNumber(ctl);
  if (decision.type === 'clear') {
    if (hasClear && value != null) return emitNumber(ctl, null), true;
  } else if (decision.type === 'commit' && decision.value !== value) {
    return emitNumber(ctl, decision.value), true;
  }
  renderNumber(ctl);
  return true;
}

function stepNumber(ctl, direction) {
  if (numberInactive(ctl)) return;
  const next = steppedValue(ctl, valueForStepping(ctl), direction);
  if (next == null) return;
  endNumberDraft(ctl);
  if (next !== committedNumber(ctl)) emitNumber(ctl, next);
  else renderNumber(ctl);
}

const numberControl = (t) => (t instanceof Element ? t.closest('.astryx-number-input') : null);
const isNumberField = (t) => t instanceof HTMLInputElement && t.matches('.astryx-number-input-field');

doc.addEventListener('input', (e) => {
  if (!isNumberField(e.target)) return;
  const ctl = numberControl(e.target);
  if (numberInactive(ctl)) return;
  const s = numberState(ctl);
  s.pending = e.target.value;
  s.kept = false;
  if (s.rejection !== null && getNumberInputRejection(s.pending, numberRules(ctl)) === null) s.rejection = null;
  // Steppers follow the draft, since stepping starts from it.
  renderNumber(ctl);
});
doc.addEventListener('focusin', (e) => {
  if (!isNumberField(e.target)) return;
  const input = e.target;
  const ctl = numberControl(input);
  const s = numberState(ctl);
  const value = committedNumber(ctl);
  const editable = s.pending ?? (value == null ? '' : formatEditableNumber(value, ctl.dataset.locale || undefined));
  const reselect =
    input.value.length > 0 && input.value !== editable && input.selectionStart === 0 && input.selectionEnd === input.value.length;
  s.focused = true;
  renderNumber(ctl);
  if (reselect) queueMicrotask(() => input.setSelectionRange(0, input.value.length));
});
doc.addEventListener('focusout', (e) => {
  if (!isNumberField(e.target)) return;
  const ctl = numberControl(e.target);
  commitNumber(ctl, 'blur');
  numberState(ctl).focused = false;
  renderNumber(ctl);
});
doc.addEventListener('keydown', (e) => {
  if (!isNumberField(e.target)) return;
  const ctl = numberControl(e.target);
  if (ctl.hasAttribute('data-disabled')) {
    if (e.key === 'Enter') e.preventDefault();
    return;
  }
  if (e.isComposing || e.defaultPrevented) return;
  const modified = e.altKey || e.ctrlKey || e.metaKey || e.shiftKey;
  if (!modified && (e.key === 'ArrowUp' || e.key === 'ArrowDown')) {
    e.preventDefault();
    stepNumber(ctl, e.key === 'ArrowUp' ? 1 : -1);
  } else if (e.key === 'Enter' && !commitNumber(ctl, 'Enter')) {
    // A rejected draft must not reach the form through implicit submission.
    e.preventDefault();
  }
});
function numberWheel(e) {
  const input = e.currentTarget;
  const ctl = numberControl(input);
  if (!ctl || ctl.hasAttribute('data-no-wheel') || doc.activeElement !== input || numberInactive(ctl)) return;
  if (e.deltaY === 0 || e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
  e.preventDefault();
  e.stopPropagation();
  stepNumber(ctl, e.deltaY < 0 ? 1 : -1);
}

// Steppers: press and hold repeats, accelerating, and commits on release.
const HOLD = { delayMs: 500, firstIntervalMs: 250, acceleration: 0.9, minIntervalMs: 50 };
const stepperOf = (t) => (t instanceof Element ? t.closest('.astryx-number-input-stepper') : null);
const stepDir = (b) => (b.dataset.direction === 'increment' ? 1 : -1);
function cancelHold(ctl) {
  const s = numberState(ctl);
  if (!s.hold) return;
  if (s.hold.timer != null) clearTimeout(s.hold.timer);
  s.hold = null;
  s.holdDraft = null;
  renderNumber(ctl);
}
function endHold(ctl, end, pointerId) {
  const s = numberState(ctl);
  const h = s.hold;
  if (!h || h.pointerId !== pointerId) return;
  s.hold = null;
  if (h.timer != null) clearTimeout(h.timer);
  if (h.held == null) return;
  s.clickAfterHold = end === 'release';
  s.holdDraft = null;
  if (numberInactive(ctl)) return renderNumber(ctl);
  endNumberDraft(ctl);
  if (h.held !== committedNumber(ctl)) emitNumber(ctl, h.held);
  else renderNumber(ctl);
}
doc.addEventListener('pointerdown', (e) => {
  const b = stepperOf(e.target);
  if (!b) return;
  const ctl = numberControl(b);
  const s = numberState(ctl);
  e.preventDefault();
  s.clickAfterHold = false;
  cancelHold(ctl);
  if (e.button !== 0 || e.pointerType === 'touch' || b.hasAttribute('aria-disabled') || numberInactive(ctl)) return;
  const dir = stepDir(b);
  const first = steppedValue(ctl, valueForStepping(ctl), dir);
  if (first == null) return;
  b.setPointerCapture?.(e.pointerId);
  const h = { pointerId: e.pointerId, timer: null, held: null, intervalMs: HOLD.firstIntervalMs };
  const repeat = () => {
    const next = h.held == null ? first : steppedValue(ctl, h.held, dir);
    if (next == null || next === h.held) {
      h.timer = null;
      return;
    }
    h.held = next;
    s.holdDraft = next;
    renderNumber(ctl);
    h.timer = setTimeout(repeat, h.intervalMs);
    h.intervalMs = Math.max(HOLD.minIntervalMs, h.intervalMs * HOLD.acceleration);
  };
  h.timer = setTimeout(repeat, HOLD.delayMs);
  s.hold = h;
});
for (const [type, end] of [
  ['pointerup', 'release'],
  ['pointercancel', 'cancel'],
  ['lostpointercapture', 'cancel'],
]) {
  doc.addEventListener(
    type,
    (e) => {
      const b = stepperOf(e.target);
      if (b) endHold(numberControl(b), end, e.pointerId);
    },
    true,
  );
}
doc.addEventListener('contextmenu', (e) => {
  const b = stepperOf(e.target);
  if (!b) return;
  const ctl = numberControl(b);
  const s = numberState(ctl);
  if (s.hold) {
    cancelHold(ctl);
    s.clickAfterHold = true;
  }
});
doc.addEventListener('click', (e) => {
  const t = e.target;
  const b = stepperOf(t);
  if (b) {
    const ctl = numberControl(b);
    const s = numberState(ctl);
    const afterHold = s.clickAfterHold;
    s.clickAfterHold = false;
    numberParts(ctl).field?.focus();
    if (afterHold && e.detail > 0) return;
    if (b.hasAttribute('aria-disabled')) return;
    stepNumber(ctl, stepDir(b));
    return;
  }
  const clear = t instanceof Element ? t.closest('.astryx-number-input > [data-number-input-clear]') : null;
  if (clear) {
    const ctl = numberControl(clear);
    endNumberDraft(ctl);
    if (ctl.hasAttribute('data-clear')) emitNumber(ctl, null);
    else renderNumber(ctl);
    numberParts(ctl).field?.focus({ preventScroll: e.detail !== 0 });
    return;
  }
  // Clicking the padding around a text or number field focuses it.
  if (t instanceof Element && t.matches('.astryx-number-input, .astryx-text-input') && !t.hasAttribute('data-disabled')) {
    t.querySelector(':scope > input:not([type="hidden"])')?.focus();
  }
});
// Keep the clear button from taking focus away from the field.
doc.addEventListener('mousedown', (e) => {
  if (e.target instanceof Element && e.target.closest('.astryx-number-input > [data-number-input-clear]')) e.preventDefault();
});

/** Wire a NumberInput once: wheel stepping, and a hidden value Datastar (or a script) can set. */
function wireNumber(ctl) {
  const { field, hidden } = numberParts(ctl);
  if (!field || !hidden) return;
  if (!field._canonWheel) {
    field._canonWheel = true;
    field.addEventListener('wheel', numberWheel, { passive: false });
  }
  hidden._canonDefault ??= nativeValue.get.call(hidden);
  hookValue(hidden, () => {
    const s = numberState(ctl);
    // A new value from outside replaces a rejected or kept draft.
    if (s.rejection !== null || s.kept) endNumberDraft(ctl);
    renderNumber(ctl);
  });
  renderNumber(ctl);
}

// Slider: values live in the hidden .astryx-slider-value inputs (one per thumb). Moving fires
// input on them; the end of a drag or key press fires change.

const THUMB = 20;
const sliderOf = (t) => (t instanceof Element ? t.closest('.astryx-slider-control') : null);
const sliderHiddens = (ctl) => [...ctl.parentElement.querySelectorAll(':scope > .astryx-slider-value')];
const sliderThumbs = (ctl) => [...ctl.querySelectorAll(':scope > .astryx-slider-travel > .astryx-slider-thumb')];
function sliderConfig(ctl) {
  const d = ctl.dataset;
  const min = Number(d.min ?? 0);
  const max = Number(d.max ?? 100);
  const step = Number(d.step ?? 1);
  return { min, max, step, gap: Number(d.minGap ?? 0) * step, range: sliderHiddens(ctl).length > 1 };
}
const clampTo = (v, lo, hi) => Math.min(Math.max(v, lo), hi);
function sliderValues(ctl) {
  const { min, max } = sliderConfig(ctl);
  return sliderHiddens(ctl).map((h) => {
    const n = Number(nativeValue.get.call(h));
    return clampTo(Number.isFinite(n) ? n : min, min, max);
  });
}
function decimalPrecision(n) {
  if (Math.abs(n) < 1) {
    const parts = n.toExponential().split('e-');
    if (parts.length === 2) return (parts[0].split('.')[1]?.length ?? 0) + parseInt(parts[1], 10);
  }
  return String(n).split('.')[1]?.length ?? 0;
}
function snapToStep(ctl, v) {
  const { min, step } = sliderConfig(ctl);
  if (step <= 0) return v;
  const snapped = min + Math.round((v - min) / step) * step;
  return Number(snapped.toFixed(Math.min(Math.max(decimalPrecision(min), decimalPrecision(step)), 20)));
}
function sliderConstrained(ctl, index, raw, snap = true, from = sliderValues(ctl)) {
  const { min, max, gap, range } = sliderConfig(ctl);
  const v = clampTo(snap ? snapToStep(ctl, raw) : raw, min, max);
  if (!range) return [v];
  const next = [...from];
  next[index] = v;
  if (index === 0) next[0] = Math.min(next[0], next[1] - gap);
  else next[1] = Math.max(next[1], next[0] + gap);
  return [clampTo(next[0], min, max), clampTo(next[1], min, max)];
}
const sliderText = (ctl, v) => formatNumber(ctl, v) ?? String(v);

function renderSlider(ctl) {
  const { min, max, gap, range } = sliderConfig(ctl);
  const values = sliderValues(ctl);
  const percent = (v) => (max === min ? 0 : ((v - min) / (max - min)) * 100);
  const thumbs = sliderThumbs(ctl);
  const formatted = ctl.dataset.format !== undefined;
  thumbs.forEach((thumb, i) => {
    const v = values[i];
    if (v === undefined) return;
    const progress = String(percent(v) / 100);
    thumb.style.setProperty('--_slider-progress', progress);
    thumb.setAttribute('aria-valuenow', String(v));
    if (range) {
      if (i === 1) thumb.setAttribute('aria-valuemin', String(clampTo(values[0] + gap, min, max)));
      else thumb.setAttribute('aria-valuemax', String(clampTo(values[1] - gap, min, max)));
    }
    if (formatted) thumb.setAttribute('aria-valuetext', sliderText(ctl, v));
    const anchor = thumb.previousElementSibling?.matches('.astryx-slider-anchor') ? thumb.previousElementSibling : null;
    if (anchor) {
      anchor.style.setProperty('--_slider-progress', progress);
      const tip = anchor.firstElementChild;
      const t = sliderText(ctl, v);
      if (tip && tip.textContent !== t) tip.textContent = t;
    }
  });
  const start = range ? percent(values[0]) / 100 : 0;
  const end = percent(values[values.length - 1]) / 100;
  for (const el of ctl.querySelectorAll('.astryx-slider-fill-mover, .astryx-slider-fill')) {
    el.style.setProperty('--_slider-fill-start', String(start));
    if (el.matches('.astryx-slider-fill')) el.style.setProperty('--_slider-fill-end', String(end));
  }
  for (const mark of ctl.querySelectorAll('.astryx-slider-mark')) {
    const m = Number(mark.dataset.markValue);
    mark.toggleAttribute('data-filled', range ? m >= values[0] && m <= values[1] : m <= values[0]);
  }
  const text = ctl.parentElement.querySelector(':scope > .astryx-slider-text');
  if (text) {
    const t = range ? `${sliderText(ctl, values[0])} – ${sliderText(ctl, values[1])}` : sliderText(ctl, values[0]);
    if (text.textContent !== t) text.textContent = t;
  }
}

function emitSlider(ctl, next, end = false) {
  const hiddens = sliderHiddens(ctl);
  const before = sliderValues(ctl);
  hiddens.forEach((h, i) => nativeValue.set.call(h, String(next[i])));
  renderSlider(ctl);
  hiddens.forEach((h, i) => {
    if (next[i] !== before[i]) h.dispatchEvent(new Event('input', { bubbles: true }));
  });
  if (end) for (const h of hiddens) h.dispatchEvent(new Event('change', { bubbles: true }));
}
function endSlider(ctl) {
  for (const h of sliderHiddens(ctl)) h.dispatchEvent(new Event('change', { bubbles: true }));
}

function sliderFromPosition(ctl, x, y) {
  const { min, max } = sliderConfig(ctl);
  const rect = ctl.getBoundingClientRect();
  const fraction = (offset, size) => {
    const travel = size - THUMB;
    return travel > 0 ? (offset - THUMB / 2) / travel : size > 0 ? offset / size : 0;
  };
  const rtl = getComputedStyle(ctl).direction === 'rtl';
  let p =
    ctl.dataset.orientation !== 'vertical'
      ? fraction(rtl ? rect.right - x : x - rect.left, rect.width)
      : 1 - fraction(y - rect.top, rect.height);
  p = clampTo(p, 0, 1);
  return clampTo(snapToStep(ctl, min + p * (max - min)), min, max);
}

const sliderDrags = new WeakMap();
function sliderUpdate(ctl, index, raw, snap = true, from) {
  if (ctl.hasAttribute('data-disabled')) return;
  emitSlider(ctl, sliderConstrained(ctl, index, raw, snap, from));
}
function setDragging(ctl, index) {
  sliderThumbs(ctl).forEach((t, i) => t.toggleAttribute('data-pressed', i === index));
  ctl.toggleAttribute('data-dragging', index !== null);
}

doc.addEventListener('pointerdown', (e) => {
  const ctl = sliderOf(e.target);
  if (!ctl || ctl.hasAttribute('data-disabled') || e.button !== 0 || e.ctrlKey) return;
  e.preventDefault();
  if (sliderDrags.has(ctl)) return;
  const values = sliderValues(ctl);
  const { range } = sliderConfig(ctl);
  const thumbs = sliderThumbs(ctl);
  const pressed = e.target.closest('[role="slider"]');
  const markEl = e.target.closest('[data-mark-value]');
  const newVal = markEl ? Number(markEl.dataset.markValue) : sliderFromPosition(ctl, e.clientX, e.clientY);
  const pressedIndex = pressed ? thumbs.indexOf(pressed) : -1;
  const coincident = range && values[0] === values[1];
  const closest = !range ? 0 : Math.abs(newVal - values[0]) <= Math.abs(newVal - values[1]) ? 0 : 1;
  let index = pressedIndex >= 0 ? pressedIndex : closest;
  // Two thumbs on one spot: the first move decides which one is dragged.
  const parks = coincident && (pressedIndex >= 0 || newVal === values[0]);
  if (parks && pressedIndex < 0) index = 1;
  if (coincident && !parks) index = newVal > values[0] ? 1 : 0;
  const drag = { pointer: e.pointerId, index, press: parks ? sliderFromPosition(ctl, e.clientX, e.clientY) : null, origin: parks };
  sliderDrags.set(ctl, drag);
  setDragging(ctl, index);
  // Pressing a thumb doesn't move it; pressing the track jumps the nearest thumb there.
  if (pressedIndex < 0) sliderUpdate(ctl, index, parks ? values[index] : newVal);
  thumbs[index]?.focus();
  ctl.setPointerCapture?.(e.pointerId);
});
doc.addEventListener('pointermove', (e) => {
  const ctl = sliderOf(e.target);
  const drag = ctl && sliderDrags.get(ctl);
  if (!drag || e.pointerId !== drag.pointer || ctl.hasAttribute('data-disabled')) return;
  const newVal = sliderFromPosition(ctl, e.clientX, e.clientY);
  const handTo = (index) => {
    if (index === drag.index) return;
    drag.index = index;
    setDragging(ctl, index);
    sliderThumbs(ctl)[index]?.focus();
  };
  if (drag.press !== null) {
    if (newVal === drag.press) return;
    const pressValue = drag.press;
    drag.press = null;
    handTo(newVal > pressValue ? 1 : 0);
  } else if (drag.origin) {
    const values = sliderValues(ctl);
    const dragging = drag.index;
    const other = dragging === 0 ? 1 : 0;
    if ((dragging === 0 && newVal > values[1]) || (dragging === 1 && newVal < values[0])) {
      const from = [...values];
      from[dragging] = values[other];
      handTo(other);
      sliderUpdate(ctl, other, newVal, true, from);
      return;
    }
  }
  sliderUpdate(ctl, drag.index, newVal);
});
for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) {
  doc.addEventListener(
    type,
    (e) => {
      const ctl = sliderOf(e.target);
      const drag = ctl && sliderDrags.get(ctl);
      if (!drag || e.pointerId !== drag.pointer) return;
      sliderDrags.delete(ctl);
      setDragging(ctl, null);
      endSlider(ctl);
    },
    true,
  );
}

/** The `count`-th grid point from `current` in `direction`. */
function gridStep(ctl, current, direction, count) {
  const { min, step } = sliderConfig(ctl);
  if (step <= 0) return current + direction * count;
  const position = (current - min) / step;
  const tolerance = Number.EPSILON * Math.max(1, Math.abs(position)) * 4;
  const base = direction === 1 ? Math.floor(position + tolerance) : Math.ceil(position - tolerance);
  return min + (base + direction * count) * step;
}
doc.addEventListener('keydown', (e) => {
  const thumb = e.target instanceof Element && e.target.matches('.astryx-slider-thumb') ? e.target : null;
  const ctl = thumb && sliderOf(thumb);
  if (!ctl || ctl.hasAttribute('data-disabled') || e.defaultPrevented) return;
  const index = sliderThumbs(ctl).indexOf(thumb);
  const current = sliderValues(ctl)[index];
  const { min, max } = sliderConfig(ctl);
  const rtl = getComputedStyle(ctl).direction === 'rtl';
  const towardMax = ctl.dataset.orientation !== 'vertical' && rtl ? 'ArrowLeft' : 'ArrowRight';
  const towardMin = towardMax === 'ArrowLeft' ? 'ArrowRight' : 'ArrowLeft';
  let next;
  switch (e.key) {
    case towardMax:
    case 'ArrowUp':
      next = gridStep(ctl, current, 1, 1);
      break;
    case towardMin:
    case 'ArrowDown':
      next = gridStep(ctl, current, -1, 1);
      break;
    case 'PageUp':
      next = gridStep(ctl, current, 1, 10);
      break;
    case 'PageDown':
      next = gridStep(ctl, current, -1, 10);
      break;
    case 'Home':
      next = min;
      break;
    case 'End':
      next = max;
      break;
    default:
      return;
  }
  e.preventDefault();
  emitSlider(ctl, sliderConstrained(ctl, index, next, e.key !== 'Home' && e.key !== 'End'), true);
});

/**
 * Hook a control's property so a value set from outside (Datastar's data-bind, a script)
 * calls `render`. Such writes fire no event and no mutation, so nothing else would notice.
 */
function hookProperty(el, name, render) {
  if (Object.hasOwn(el, name)) return;
  let proto = Object.getPrototypeOf(el);
  while (proto && !Object.hasOwn(proto, name)) proto = Object.getPrototypeOf(proto);
  const native = proto && Object.getOwnPropertyDescriptor(proto, name);
  if (!native?.set) return;
  Object.defineProperty(el, name, {
    configurable: true,
    get: () => native.get.call(el),
    set: (v) => {
      const before = native.get.call(el);
      native.set.call(el, v);
      if (native.get.call(el) !== before) render();
    },
  });
}
const hookValue = (input, render) => hookProperty(input, 'value', render);
function wireSlider(ctl) {
  for (const h of sliderHiddens(ctl)) {
    h._canonDefault ??= nativeValue.get.call(h);
    hookValue(h, () => renderSlider(ctl));
  }
  if (ctl.dataset.format !== undefined || !ctl._canonRendered) {
    ctl._canonRendered = true;
    renderSlider(ctl);
  }
}

// FileInput: the hidden trigger button is the keyboard and screen reader control, the box a
// mouse and drop target. Accepted files stay on the native input, so forms and data-bind see them.

const fileBox = (t) => (t instanceof Element ? t.closest('.astryx-file-input') : null);
const fileNative = (box) => box.querySelector(':scope > .astryx-file-input-native');
const fileBlocked = (box) => box.hasAttribute('data-disabled') || box.hasAttribute('data-busy');
const listFormat = new Intl.ListFormat('en', { type: 'conjunction' });
const sizeFormat = new Intl.NumberFormat('en', { maximumFractionDigits: 1, minimumFractionDigits: 1, useGrouping: false });
function formatFileSize(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${sizeFormat.format(bytes / 1024)} KB`;
  return `${sizeFormat.format(bytes / (1024 * 1024))} MB`;
}

function validateFiles(box, list) {
  const input = fileNative(box);
  const accept = input?.getAttribute('accept');
  const maxSize = optNumber(box.dataset.maxSize);
  const maxFiles = optNumber(box.dataset.maxFiles);
  const errors = [];
  let valid = list;
  if (accept) {
    const types = accept.split(',').map((s) => s.trim().toLowerCase());
    valid = valid.filter((file) => {
      const ok = types.some((type) =>
        type.startsWith('.')
          ? file.name.toLowerCase().endsWith(type)
          : type.endsWith('/*')
            ? file.type.startsWith(type.slice(0, -1))
            : file.type.toLowerCase() === type,
      );
      if (!ok) errors.push(`"${file.name}" is not an accepted file type`);
      return ok;
    });
  }
  if (maxSize != null) {
    valid = valid.filter((file) => {
      if (file.size > maxSize) {
        errors.push(`"${file.name}" exceeds ${formatFileSize(maxSize)} limit`);
        return false;
      }
      return true;
    });
  }
  if (input?.multiple && maxFiles != null && valid.length > maxFiles) {
    errors.push(`Maximum ${maxFiles} files allowed`);
    valid = valid.slice(0, maxFiles);
  }
  if (!input?.multiple) valid = valid.slice(0, 1);
  return { valid, errors };
}

/** Show the native input's files, and a validation error (null clears it). */
function renderFiles(box, error) {
  const input = fileNative(box);
  if (!input) return;
  const files = [...(input.files ?? [])];
  const names = files.map((f) => f.name).join(', ');
  box.toggleAttribute('data-has-files', files.length > 0);
  const name = box.querySelector(':scope > .astryx-file-input-name');
  if (name && name.textContent !== names) name.textContent = names;
  const trigger = box.querySelector('.astryx-file-input-trigger');
  const label = box.dataset.label ?? '';
  trigger?.setAttribute('aria-label', files.length ? `${label}, ${names}` : label);
  if (error === undefined) return;
  box.toggleAttribute('data-invalid', error !== null);
  const status = doc.getElementById(`${input.id}-error`);
  if (status) {
    status.hidden = error === null;
    const target = status.querySelector('.astryx-field-status-content > span:last-child') ?? status;
    target.textContent = error ?? '';
    if (trigger) {
      const ids = (trigger.getAttribute('aria-describedby') ?? '').split(/\s+/).filter((x) => x && x !== status.id);
      if (error !== null) ids.push(status.id);
      setIDs(trigger, 'aria-describedby', ids);
    }
  }
}

function setFiles(input, files) {
  try {
    const dt = new DataTransfer();
    for (const f of files) dt.items.add(f);
    input.files = dt.files;
  } catch {
    input.value = '';
  }
}

/** Validate a new selection onto the native input; true when the input's files changed. */
function takeFiles(box, list) {
  const input = fileNative(box);
  if (!input || fileBlocked(box)) return;
  const { valid, errors } = validateFiles(box, list);
  setFiles(input, valid);
  renderFiles(box, errors.length > 0 ? listFormat.format(errors) : null);
  const live = box.parentElement?.querySelector(':scope > .astryx-file-input-live');
  if (live && errors.length === 0 && valid.length > 0) {
    live.textContent = valid.length === 1 ? `1 file selected: ${valid[0].name}` : `${valid.length} files selected`;
  }
}

// Capture phase: validate before data-bind (or any change listener on the input) reads it.
doc.addEventListener(
  'change',
  (e) => {
    const input = e.target;
    if (!(input instanceof HTMLInputElement) || !input.matches('.astryx-file-input-native')) return;
    if (input._canonTaken) return;
    takeFiles(fileBox(input), [...(input.files ?? [])]);
  },
  true,
);
doc.addEventListener('click', (e) => {
  const t = e.target;
  const box = fileBox(t);
  if (!box) return;
  if (t.closest('[data-file-input-clear]')) {
    e.stopPropagation();
    const input = fileNative(box);
    if (!input) return;
    input.value = '';
    renderFiles(box, null);
    input._canonTaken = true;
    input.dispatchEvent(new Event('change', { bubbles: true }));
    input._canonTaken = false;
    const trigger = box.querySelector('.astryx-file-input-trigger');
    if (e.detail === 0) trigger?.focus();
    else requestAnimationFrame(() => trigger?.focus({ preventScroll: true }));
    return;
  }
  if (t.matches('.astryx-file-input-native')) {
    if (fileBlocked(box)) e.preventDefault();
    return;
  }
  if (fileBlocked(box)) return;
  if (t.closest('button') && !t.closest('.astryx-file-input-trigger')) return;
  fileNative(box)?.click();
});
doc.addEventListener('mousedown', (e) => {
  if (e.target instanceof Element && e.target.closest('.astryx-file-input [data-file-input-clear]')) e.preventDefault();
});
for (const type of ['dragenter', 'dragover']) {
  doc.addEventListener(type, (e) => {
    const box = fileBox(e.target);
    if (!box || box.dataset.mode !== 'dropzone') return;
    e.preventDefault();
    e.stopPropagation();
    if (fileBlocked(box)) {
      if (e.dataTransfer) e.dataTransfer.dropEffect = 'none';
    } else if (!box.hasAttribute('data-drag-over')) {
      box.setAttribute('data-drag-over', '');
      const ph = box.querySelector(':scope > .astryx-file-input-placeholder');
      if (ph) {
        ph._canonText ??= ph.textContent;
        ph.textContent = 'Drop files here';
      }
    }
  });
}
function leaveDrop(box) {
  box.removeAttribute('data-drag-over');
  const ph = box.querySelector(':scope > .astryx-file-input-placeholder');
  if (ph?._canonText != null) ph.textContent = ph._canonText;
}
doc.addEventListener('dragleave', (e) => {
  const box = fileBox(e.target);
  if (!box || box.dataset.mode !== 'dropzone') return;
  e.preventDefault();
  if (e.relatedTarget instanceof Node && box.contains(e.relatedTarget)) return;
  leaveDrop(box);
});
doc.addEventListener('drop', (e) => {
  const box = fileBox(e.target);
  if (!box || box.dataset.mode !== 'dropzone') return;
  e.preventDefault();
  e.stopPropagation();
  leaveDrop(box);
  if (fileBlocked(box)) return;
  const list = [...(e.dataTransfer?.files ?? [])];
  if (list.length === 0) return;
  takeFiles(box, list);
  const input = fileNative(box);
  input._canonTaken = true;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  input.dispatchEvent(new Event('change', { bubbles: true }));
  input._canonTaken = false;
});

// ---------------------------------------------------------------------------------------------
// Selector: a combobox button over a popover listbox. The value lives in the hidden
// .astryx-selector-value input (where data-bind goes); rows carry data-value. Rows are
// reached with aria-activedescendant, so focus stays on the trigger (or the search field).
// MultiSelector shares the control ([data-multiple]): its values are the selected options of a
// hidden <select multiple>, rows toggle without closing the list, and lead rows ([data-create],
// [data-select-all]) sit above the options.

const selCtl = (t) => (t instanceof Element ? t.closest('.astryx-selector-control') : null);
function selParts(ctl) {
  const trigger = ctl.querySelector(':scope > .astryx-selector-trigger');
  const id = trigger?.id ?? '';
  const popup = id ? doc.getElementById(`${id}-popup`) : null;
  return {
    trigger,
    hidden: ctl.querySelector(':scope > :is(.astryx-selector-value, .astryx-multi-selector-value)'),
    popup,
    listbox: popup?.querySelector('[role="listbox"]') ?? null,
    search: popup?.querySelector('.astryx-selector-search') ?? null,
    live: id ? doc.getElementById(`${id}-live`) : null,
  };
}
function selState(ctl) {
  ctl._canonSel ??= { active: null, keyboard: false, buffer: '', timer: 0 };
  return ctl._canonSel;
}
const selInactive = (ctl) => ctl.hasAttribute('data-disabled') || ctl.hasAttribute('data-readonly');
const selRows = (ctl) => [...(selParts(ctl).listbox?.querySelectorAll('[role="option"]') ?? [])];
const selMulti = (ctl) => ctl.hasAttribute('data-multiple');
const isLeadRow = (r) => r.hasAttribute('data-select-all') || r.hasAttribute('data-create');
const selOptionRows = (ctl) => selRows(ctl).filter((r) => !isLeadRow(r));
/** The MultiSelector's selected values, in option order. */
const selValues = (ctl) => [...(selParts(ctl).hidden?.options ?? [])].filter((o) => o.selected).map((o) => o.value);
/** Set or remove an attribute, touching the DOM only on change (scan() re-renders on mutations). */
function attrTo(el, name, value) {
  if (value == null) {
    if (el.hasAttribute(name)) el.removeAttribute(name);
  } else if (el.getAttribute(name) !== value) el.setAttribute(name, value);
}
const selVisible = (ctl) => selRows(ctl).filter((r) => !r.hidden);
const rowDisabled = (r) => r.getAttribute('aria-disabled') === 'true';
const rowLabel = (r) => r.querySelector('.astryx-selector-option-label')?.textContent ?? r.dataset.value ?? '';
const selValue = (ctl) => {
  const { hidden } = selParts(ctl);
  return hidden ? nativeValue.get.call(hidden) : '';
};
const selIsOpen = (ctl) => selParts(ctl).popup?.matches(':popover-open') ?? false;

/** Bring the trigger text, the selected row and the clear button up to date with the value. */
function renderSelector(ctl) {
  if (selMulti(ctl)) return renderMultiSelector(ctl);
  const { trigger } = selParts(ctl);
  if (!trigger) return;
  const value = selValue(ctl);
  let picked = null;
  for (const r of selRows(ctl)) {
    const on = value !== '' && r.dataset.value === value;
    if (on && !picked) picked = r;
    const next = String(on);
    if (r.getAttribute('aria-selected') !== next) r.setAttribute('aria-selected', next);
  }
  const text = trigger.querySelector('.astryx-selector-value-text');
  if (text) {
    const label = picked ? rowLabel(picked) : (text.dataset.placeholderText ?? '');
    if (text.textContent !== label) text.textContent = label;
  }
  const iconSlot = trigger.querySelector('.astryx-selector-value-icon');
  if (iconSlot) {
    // Only touch the DOM on change: scan() runs on mutations and calls this.
    const icon = picked?.dataset.icon ?? '';
    if ((iconSlot.dataset.icon ?? '') !== icon) {
      const source = picked?.querySelector(':scope > .astryx-selector-option-content > .astryx-icon');
      iconSlot.replaceChildren(...(source ? [source.cloneNode(true)] : []));
      iconSlot.dataset.icon = icon;
    }
    if (iconSlot.hidden !== (icon === '')) iconSlot.hidden = icon === '';
  }
  if (ctl.hasAttribute('data-placeholder') === !!picked) ctl.toggleAttribute('data-placeholder', !picked);
  const clear = ctl.querySelector(':scope > [data-selector-clear]');
  if (clear && clear.hidden !== (value === '')) clear.hidden = value === '';
}

/** The trigger text (or badges), the checked rows, "Select all" and the clear button. */
function renderMultiSelector(ctl) {
  const { trigger, hidden } = selParts(ctl);
  if (!trigger || !hidden) return;
  const picked = [...hidden.options].filter((o) => o.selected);
  const on = new Set(picked.map((o) => o.value));
  const rows = selOptionRows(ctl);
  for (const r of rows) attrTo(r, 'aria-selected', String(on.has(r.dataset.value ?? '')));
  const all = selParts(ctl).listbox?.querySelector('[data-select-all]');
  if (all) {
    // Over the enabled options the search leaves shown.
    const enabled = rows.filter((r) => !r.hidden && !rowDisabled(r));
    const count = enabled.filter((r) => on.has(r.dataset.value ?? '')).length;
    const state = enabled.length > 0 && count === enabled.length ? 'checked' : count > 0 ? 'indeterminate' : 'unchecked';
    attrTo(all, 'data-state', state);
    attrTo(all, 'aria-selected', String(state === 'checked'));
    attrTo(all, 'aria-label', state === 'indeterminate' ? `${rowLabel(all)}, partially selected` : null);
  }
  const text = trigger.querySelector('.astryx-selector-value-text');
  if (text) {
    const labels = picked.map((o) => o.textContent || o.value);
    const display = text.dataset.display ?? 'count';
    const badges = display === 'badges' && labels.length > 0;
    let label = text.dataset.placeholderText ?? '';
    if (labels.length && display === 'labels') {
      const rest = labels.length - 3;
      label = labels.slice(0, 3).join(', ') + (rest > 0 ? `, +${rest}` : '');
    } else if (labels.length && !badges) label = `${labels.length} selected`;
    const key = badges ? `\u0000${labels.join('\u0000')}` : label;
    if ((text._canonKey ?? (text.hasAttribute('data-badges') ? null : text.textContent)) !== key) {
      text._canonKey = key;
      text.toggleAttribute('data-badges', badges);
      if (!badges) text.textContent = label;
      else {
        const max = Number(text.dataset.maxBadges) || 3;
        const nodes = labels.slice(0, max).map((l) => {
          const b = doc.createElement('span');
          b.className = 'astryx-badge';
          b.dataset.variant = 'neutral';
          b.setAttribute('data-has-label', '');
          b.title = l;
          const inner = doc.createElement('span');
          inner.className = 'astryx-badge-label';
          inner.dir = 'auto';
          inner.textContent = l;
          b.append(inner);
          return b;
        });
        if (labels.length > max) {
          const more = doc.createElement('span');
          more.className = 'astryx-multi-selector-overflow';
          more.textContent = `+${labels.length - max}`;
          nodes.push(more);
        }
        text.replaceChildren(...nodes);
      }
    }
  }
  if (ctl.hasAttribute('data-placeholder') !== (picked.length === 0)) ctl.toggleAttribute('data-placeholder', picked.length === 0);
  const clear = ctl.querySelector(':scope > [data-selector-clear]');
  if (clear && clear.hidden !== (picked.length === 0)) clear.hidden = picked.length === 0;
}

function selHighlight(ctl, row, keyboard) {
  const s = selState(ctl);
  const { trigger, search } = selParts(ctl);
  if (s.active && s.active !== row) {
    s.active.removeAttribute('data-highlighted');
    s.active.removeAttribute('data-keyboard');
  }
  s.active = row;
  if (keyboard !== undefined) s.keyboard = keyboard;
  if (row) {
    row.setAttribute('data-highlighted', '');
    row.toggleAttribute('data-keyboard', s.keyboard);
    row.scrollIntoView?.({ block: 'nearest' });
  }
  const owner = search ?? trigger;
  if (!owner || owner.getAttribute('role') !== 'combobox') return;
  if (row) owner.setAttribute('aria-activedescendant', row.id);
  else owner.removeAttribute('aria-activedescendant');
}

/** The row to highlight when the list opens: the selected one, else the first (or last) enabled. */
function selOpeningRow(ctl, fromEnd) {
  const visible = selVisible(ctl).filter((r) => !rowDisabled(r));
  if (selMulti(ctl)) return (fromEnd ? visible.at(-1) : visible[0]) ?? null;
  const value = selValue(ctl);
  const picked = value === '' ? null : visible.find((r) => r.dataset.value === value);
  return picked ?? (fromEnd ? visible.at(-1) : visible[0]) ?? null;
}

function openSelector(ctl, highlight = 'start') {
  if (selInactive(ctl)) return false;
  const { popup, search } = selParts(ctl);
  if (!popup) return false;
  popup._canonAnchor = ctl;
  if (selMulti(ctl) && !popup.matches(':popover-open')) selReorder(ctl);
  safeShow(popup);
  if (!popup.matches(':popover-open')) return false;
  if (search) {
    requestAnimationFrame(() => {
      search.focus({ preventScroll: true });
      search.setSelectionRange(search.value.length, search.value.length);
    });
  } else if (highlight !== 'none') {
    selHighlight(ctl, selOpeningRow(ctl, highlight === 'end'));
  }
  return true;
}

function closeSelector(ctl, restoreFocus = false) {
  const { popup, trigger } = selParts(ctl);
  if (!popup) return;
  if (popup.matches(':popover-open')) {
    if (popup.contains(doc.activeElement)) restoreFocus = true;
    // Cleared now, not in the (later) toggle event, so a message sent right after closing stays.
    const { live } = selParts(ctl);
    if (live) live.textContent = '';
    safeHide(popup);
  }
  if (restoreFocus) trigger?.focus({ preventScroll: true });
}

/** Reset the list state after the popup closes, however it closed. */
function selClosed(ctl) {
  const s = selState(ctl);
  const { search } = selParts(ctl);
  selHighlight(ctl, null);
  s.buffer = '';
  if (search && search.value !== '') {
    search.value = '';
    selFilter(ctl, '');
  }
}

function selCommit(ctl, value) {
  const { hidden } = selParts(ctl);
  if (!hidden || selInactive(ctl)) return;
  if (nativeValue.get.call(hidden) === value) return;
  nativeValue.set.call(hidden, value);
  renderSelector(ctl);
  hidden.dispatchEvent(new Event('input', { bubbles: true }));
  hidden.dispatchEvent(new Event('change', { bubbles: true }));
}

function selPick(ctl, row) {
  if (!row || rowDisabled(row)) return;
  if (selMulti(ctl)) return selToggle(ctl, row);
  selCommit(ctl, row.dataset.value ?? '');
  closeSelector(ctl, true);
}

/** Move the selected options to the top of each run of rows (and each section), keeping the
 * rest in option order. Runs only as the list opens, so rows don't jump while toggling. */
function selReorder(ctl) {
  const { listbox, hidden } = selParts(ctl);
  if (!listbox || !hidden) return;
  const on = new Set(selValues(ctl));
  const index = (r) => Number(r.id.match(/-option-(\d+)$/)?.[1] ?? 0);
  const sortRun = (parent, run) => {
    if (run.length < 2) return;
    const sorted = [...run].sort((a, b) => on.has(b.dataset.value) - on.has(a.dataset.value) || index(a) - index(b));
    if (sorted.every((r, i) => r === run[i])) return;
    const after = run.at(-1).nextSibling;
    for (const r of sorted) parent.insertBefore(r, after);
  };
  const runs = (parent) => {
    let run = [];
    for (const child of [...parent.children]) {
      if (child.getAttribute('role') === 'option' && !isLeadRow(child)) run.push(child);
      else {
        sortRun(parent, run);
        run = [];
        if (child.classList.contains('astryx-selector-section')) runs(child);
      }
    }
    sortRun(parent, run);
  };
  runs(listbox);
}

/** Tell data-bind (and listeners) that the MultiSelector's select changed. */
function selChanged(ctl) {
  const { hidden } = selParts(ctl);
  renderMultiSelector(ctl);
  hidden?.dispatchEvent(new Event('input', { bubbles: true }));
  hidden?.dispatchEvent(new Event('change', { bubbles: true }));
}

function selAnnounce(ctl, message) {
  const { live } = selParts(ctl);
  if (live) live.textContent = message;
}

function selAnnounceCount(ctl) {
  const rows = selOptionRows(ctl);
  const on = new Set(selValues(ctl));
  const count = rows.filter((r) => on.has(r.dataset.value ?? '')).length;
  const total = rows.length;
  selAnnounce(ctl, count === 0 ? 'Selection cleared' : total > 0 && count === total ? 'All selected' : `${count} of ${total} selected`);
}

/** Set the MultiSelector's selection (an array of values) and notify. */
function selSetValues(ctl, values) {
  const { hidden } = selParts(ctl);
  if (!hidden || selInactive(ctl)) return;
  const want = new Set(values);
  let changed = false;
  for (const o of hidden.options) {
    if (o.selected !== want.has(o.value)) {
      o.selected = want.has(o.value);
      changed = true;
    }
  }
  if (changed) selChanged(ctl);
}

/** Toggle a row of a MultiSelector: an option, "Select all" or "Create". The list stays open. */
function selToggle(ctl, row) {
  const { hidden, search } = selParts(ctl);
  if (!hidden || selInactive(ctl)) return;
  const values = selValues(ctl);
  if (row.hasAttribute('data-create')) {
    const query = row.dataset.query ?? '';
    if (!query) return;
    let opt = [...hidden.options].find((o) => o.value === query);
    if (!opt) {
      opt = new Option(query, query);
      hidden.append(opt);
      hookProperty(opt, 'selected', () => renderMultiSelector(ctl));
    }
    if (!opt.selected) {
      opt.selected = true;
      selChanged(ctl);
    }
    ctl.dispatchEvent(new CustomEvent('astryx-create', { bubbles: true, detail: { query } }));
    if (search) {
      search.value = '';
      search.parentElement?.querySelector('.astryx-selector-search-clear')?.toggleAttribute('hidden', true);
      selFilter(ctl, '');
    }
    selAnnounce(ctl, `${query} created`);
    return;
  }
  if (row.hasAttribute('data-select-all')) {
    const shown = selOptionRows(ctl).filter((r) => !r.hidden && !rowDisabled(r)).map((r) => r.dataset.value ?? '');
    const allOn = row.dataset.state === 'checked';
    const next = allOn ? values.filter((v) => !shown.includes(v)) : [...values, ...shown.filter((v) => !values.includes(v))];
    selSetValues(ctl, next);
  } else {
    const v = row.dataset.value ?? '';
    selSetValues(ctl, values.includes(v) ? values.filter((x) => x !== v) : [...values, v]);
  }
  selAnnounceCount(ctl);
}

/** Show the rows whose label contains the query; dividers hide while searching. */
function selFilter(ctl, query) {
  const { listbox, live } = selParts(ctl);
  if (!listbox) return;
  const q = query.toLowerCase();
  let shown = 0;
  for (const r of listbox.querySelectorAll('[role="option"]')) {
    if (isLeadRow(r)) continue;
    const match = !q || rowLabel(r).toLowerCase().includes(q);
    r.hidden = !match;
    if (match) shown++;
  }
  // MultiSelector: offer the query as a new value when no option has that label.
  const create = listbox.querySelector(':scope > [data-create]');
  let creating = false;
  if (create) {
    const wanted = query.trim();
    creating = wanted !== '' && !ctl.querySelector('[aria-busy="true"]') &&
      !selOptionRows(ctl).some((r) => rowLabel(r).toLowerCase() === wanted.toLowerCase());
    create.hidden = !creating;
    create.dataset.query = creating ? wanted : '';
    const label = create.querySelector('.astryx-selector-option-label');
    if (label) label.textContent = creating ? `Create "${wanted}"` : '';
  }
  const all = listbox.querySelector(':scope > [data-select-all]');
  if (all) all.hidden = shown === 0;
  for (const d of listbox.querySelectorAll(':scope > .astryx-selector-divider')) d.hidden = q !== '';
  for (const sec of listbox.querySelectorAll(':scope > .astryx-selector-section')) {
    sec.hidden = !sec.querySelector('[role="option"]:not([hidden])');
  }
  const empty = listbox.querySelector(':scope > .astryx-selector-empty-state');
  if (empty) {
    empty.hidden = shown > 0 || creating;
    const text = q ? listbox.dataset.emptySearchText : listbox.dataset.emptyText;
    if (text != null && empty.textContent !== text) empty.textContent = text;
  }
  if (selMulti(ctl)) renderMultiSelector(ctl);
  selHighlight(ctl, null);
  if (live) live.textContent = q && shown > 0 && !ctl.querySelector('[aria-busy="true"]') ? `${shown} ${shown === 1 ? 'result' : 'results'}` : '';
}

/** Label-prefix typeahead: repeating one character cycles through the labels starting with it. */
function selTypeahead(ctl, key, from) {
  const s = selState(ctl);
  clearTimeout(s.timer);
  s.timer = setTimeout(() => (s.buffer = ''), 500);
  const lower = key.toLowerCase();
  const cycling = s.buffer.length > 0 && [...s.buffer].every((c) => c === lower);
  s.buffer += lower;
  const needle = cycling ? lower : s.buffer;
  const rows = selRows(ctl);
  const current = from ? rows.indexOf(from) : -1;
  const start = cycling || s.buffer.length === 1 ? current + 1 : Math.max(current, 0);
  for (let k = 0; k < rows.length; k++) {
    const r = rows[(((start + k) % rows.length) + rows.length) % rows.length];
    if (!rowDisabled(r) && rowLabel(r).toLowerCase().startsWith(needle)) return r;
  }
  return null;
}

/** Listbox keys shared by the trigger and the search field; true when handled. */
function selNavigate(ctl, e) {
  const open = selIsOpen(ctl);
  const s = selState(ctl);
  const enabled = selVisible(ctl).filter((r) => !rowDisabled(r));
  const hasSearch = selParts(ctl).search != null;
  switch (e.key) {
    case 'ArrowDown':
    case 'ArrowUp': {
      e.preventDefault();
      const down = e.key === 'ArrowDown';
      if (!open) {
        openSelector(ctl, down ? 'start' : 'end');
        return true;
      }
      const i = s.active ? enabled.indexOf(s.active) : -1;
      const next = i < 0 ? (down ? enabled[0] : enabled.at(-1)) : enabled[Math.min(Math.max(i + (down ? 1 : -1), 0), enabled.length - 1)];
      if (next) selHighlight(ctl, next, true);
      return true;
    }
    case 'Home':
    case 'PageUp':
    case 'End':
    case 'PageDown': {
      if (!open) return false;
      e.preventDefault();
      const next = e.key === 'Home' || e.key === 'PageUp' ? enabled[0] : enabled.at(-1);
      if (next) selHighlight(ctl, next, true);
      return true;
    }
    case ' ':
      if (hasSearch) return false;
    // falls through
    case 'Enter':
      e.preventDefault();
      if (open && s.active) selPick(ctl, s.active);
      else if (!open) openSelector(ctl);
      return true;
    case 'Escape':
      if (!open) return false;
      e.preventDefault();
      e.stopPropagation();
      closeSelector(ctl, true);
      return true;
    case 'Tab':
      if (open) closeSelector(ctl);
      return false;
  }
  return false;
}

const printableKey = (e) => e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey;

doc.addEventListener('keydown', (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;
  if (t.matches('.astryx-selector-trigger')) {
    const ctl = selCtl(t);
    if (!ctl || e.defaultPrevented || selInactive(ctl) || e.isComposing) return;
    const s = selState(ctl);
    s.keyboard = true;
    const open = selIsOpen(ctl);
    const hasSearch = selParts(ctl).search != null;
    if (!hasSearch && printableKey(e) && (e.key !== ' ' || s.buffer !== '')) {
      e.preventDefault();
      const multi = selMulti(ctl);
      const value = multi ? '' : selValue(ctl);
      const from = open ? s.active : selRows(ctl).find((r) => value !== '' && r.dataset.value === value);
      const match = selTypeahead(ctl, e.key, from);
      if (!match) return;
      // A MultiSelector's typeahead only moves the highlight; it never toggles.
      if (multi && !open) {
        if (openSelector(ctl, 'none')) selHighlight(ctl, match, true);
      } else if (open) selHighlight(ctl, match, true);
      else if (match.dataset.value !== value) {
        selCommit(ctl, match.dataset.value ?? '');
        const { live } = selParts(ctl);
        if (live) live.textContent = rowLabel(match);
      }
      return;
    }
    if (selNavigate(ctl, e)) return;
    if ((e.key === 'Delete' || e.key === 'Backspace') && !hasSearch && !open && ctl.querySelector(':scope > [data-selector-clear]')) {
      s.buffer = '';
      if (selMulti(ctl)) {
        if (selValues(ctl).length === 0) return;
        e.preventDefault();
        selSetValues(ctl, []);
        selAnnounce(ctl, 'Selection cleared');
      } else if (selValue(ctl) !== '') {
        e.preventDefault();
        selCommit(ctl, '');
      }
      return;
    }
    if (hasSearch && printableKey(e)) {
      // Typing on the trigger opens the list and seeds the search with that character.
      e.preventDefault();
      const { search } = selParts(ctl);
      if (open || openSelector(ctl, 'none')) {
        search.value += e.key;
        selFilter(ctl, search.value);
      }
    }
  } else if (t.matches('.astryx-selector-search')) {
    const ctl = selFromPopup(t);
    if (!ctl || e.isComposing) return;
    const s = selState(ctl);
    s.keyboard = true;
    const create = selParts(ctl).listbox?.querySelector(':scope > [data-create]:not([hidden])');
    if (e.key === 'Enter' && !s.active && create) {
      // Enter with nothing highlighted creates the typed value.
      e.preventDefault();
      selToggle(ctl, create);
      return;
    }
    if (['ArrowDown', 'ArrowUp', 'PageUp', 'PageDown', 'Enter', 'Escape'].includes(e.key)) selNavigate(ctl, e);
    else if (e.key === 'Tab' && (e.shiftKey || t.value === '')) closeSelector(ctl, e.shiftKey);
  } else if (t.matches('.astryx-selector-search-clear')) {
    const ctl = selFromPopup(t);
    if (!ctl) return;
    if (e.key === 'Escape' || (e.key === 'Tab' && !e.shiftKey)) {
      if (e.key === 'Escape') e.preventDefault();
      closeSelector(ctl, e.key === 'Escape');
    }
  }
});

/** The control a popup (or something inside it) belongs to. */
function selFromPopup(t) {
  const popup = t.closest('.astryx-selector-popup');
  const trigger = popup && doc.getElementById(popup.id.replace(/-popup$/, ''));
  return trigger ? selCtl(trigger) : null;
}

doc.addEventListener('input', (e) => {
  const t = e.target;
  if (!(t instanceof HTMLInputElement) || !t.matches('.astryx-selector-search')) return;
  const ctl = selFromPopup(t);
  if (!ctl) return;
  selFilter(ctl, t.value);
  const clear = t.parentElement?.querySelector('.astryx-selector-search-clear');
  if (clear) clear.hidden = t.value === '';
});

// A press on a row, the clear button or the search clear must not move focus off the combobox.
doc.addEventListener('mousedown', (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;
  if (t.closest('.astryx-selector-listbox, .astryx-selector-control > [data-selector-clear]')) e.preventDefault();
});

doc.addEventListener('click', (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;
  const row = t.closest('.astryx-selector-listbox [role="option"]');
  if (row) {
    const ctl = selFromPopup(row);
    if (ctl) selPick(ctl, row);
    return;
  }
  const searchClear = t.closest('.astryx-selector-search-clear');
  if (searchClear) {
    const ctl = selFromPopup(searchClear);
    const { search } = ctl ? selParts(ctl) : {};
    if (!search) return;
    search.value = '';
    searchClear.hidden = true;
    selFilter(ctl, '');
    search.focus();
    return;
  }
  // The popup sits inside the control, but presses in it are not presses on the control.
  if (t.closest('.astryx-selector-popup')) return;
  const ctl = selCtl(t);
  if (!ctl) return;
  const { trigger, popup } = selParts(ctl);
  if (t.closest('[data-selector-clear]')) {
    selState(ctl).buffer = '';
    if (selMulti(ctl)) {
      selSetValues(ctl, []);
      selAnnounce(ctl, 'Selection cleared');
    } else selCommit(ctl, '');
    trigger?.focus({ preventScroll: e.detail !== 0 });
    return;
  }
  if (selInactive(ctl) || !popup) return;
  selState(ctl).keyboard = e.detail === 0;
  // The press that light-dismissed the list must not reopen it.
  if (selIsOpen(ctl)) closeSelector(ctl);
  else if (performance.now() - (popup._canonClosedAt ?? 0) > 300) openSelector(ctl);
  trigger?.focus({ preventScroll: true });
});

doc.addEventListener('mousemove', (e) => {
  const row = e.target instanceof Element ? e.target.closest('.astryx-selector-listbox [role="option"]') : null;
  if (!row || rowDisabled(row)) return;
  const ctl = selFromPopup(row);
  if (ctl && selState(ctl).active !== row) selHighlight(ctl, row, false);
  else if (ctl && selState(ctl).keyboard) selHighlight(ctl, row, false);
});

doc.addEventListener(
  'toggle',
  (e) => {
    const popup = e.target;
    if (!(popup instanceof HTMLElement) || !popup.classList.contains('astryx-selector-popup')) return;
    const ctl = selFromPopup(popup);
    if (!ctl) return;
    const open = e.newState === 'open';
    ctl.toggleAttribute('data-open', open);
    ctl.querySelector(':scope > .astryx-selector-indicator-icon')?.setAttribute('data-state', open ? 'expanded' : 'collapsed');
    const { trigger, search } = selParts(ctl);
    trigger?.setAttribute('aria-expanded', String(open));
    if (open) return;
    selClosed(ctl);
    // Escape or a light dismiss with focus inside the list sends focus back to the trigger.
    if (doc.activeElement === doc.body || doc.activeElement === search) trigger?.focus({ preventScroll: true });
  },
  true,
);

/** Wire a Selector once: a hidden value Datastar (or a script) can set, and isDefaultOpen. */
function wireSelector(ctl) {
  const { hidden } = selParts(ctl);
  if (!hidden) return;
  if (hidden instanceof HTMLSelectElement) {
    // data-bind on a <select multiple> writes each option's .selected.
    for (const o of hidden.options) hookProperty(o, 'selected', () => renderMultiSelector(ctl));
  } else {
    hidden._canonDefault ??= nativeValue.get.call(hidden);
    hookValue(hidden, () => renderSelector(ctl));
  }
  renderSelector(ctl);
  if (ctl.hasAttribute('data-default-open') && !ctl._canonDefaultOpened) {
    ctl._canonDefaultOpened = true;
    requestAnimationFrame(() => openSelector(ctl));
  }
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
  checkImages();
  if (isApple) for (const k of doc.querySelectorAll('.astryx-kbd[data-keys]')) fixKbd(k);
  for (const t of doc.querySelectorAll('time.astryx-timestamp-time')) renderTime(t);
  for (const ta of doc.querySelectorAll('.astryx-text-area-control[data-max-length]')) {
    hookValue(ta, () => countTextArea(ta));
    countTextArea(ta);
  }
  for (const g of doc.querySelectorAll('.astryx-checkbox-list-group')) {
    for (const b of g.querySelectorAll('input[type="checkbox"]')) hookProperty(b, 'checked', () => syncRequired(g));
    syncRequired(g);
  }
  for (const c of doc.querySelectorAll('.astryx-selector-control')) wireSelector(c);
  for (const n of doc.querySelectorAll('.astryx-number-input')) wireNumber(n);
  for (const s of doc.querySelectorAll('.astryx-slider-control')) wireSlider(s);
  for (const f of doc.querySelectorAll('.astryx-file-input')) renderFiles(f);
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
