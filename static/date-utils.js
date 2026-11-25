// date-utils.js: Gregorian calendar dates for canon.js (Calendar now; DateInput next). Ports
// Astryx utils/plainDate.ts, utils/calendarSystem.ts (Gregorian), Calendar/hooks,
// Calendar/dayCellUtils.ts, Calendar/getInitialFocusDate.ts and Calendar/getWeekdayHeaderNames.ts,
// MIT, Meta Platforms, Inc. A date is {year, month (1-12), day}; strings are ISO YYYY-MM-DD.
// Day arithmetic runs on UTC midnights, so time zones and DST never add or drop a day.

const MS_PER_DAY = 86400000;

/** Date.UTC that keeps years 0-99 as given. */
function utcTime(year, monthIndex, day) {
  const d = new Date(0);
  d.setUTCFullYear(year, monthIndex, day);
  d.setUTCHours(0, 0, 0, 0);
  return d.getTime();
}

export function daysInMonth(year, month) {
  if (month === 2) return (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0 ? 29 : 28;
  return [0, 31, 0, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month];
}

/** Parses YYYY-MM-DD; null when it isn't a real date. */
export function fromISO(str) {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(str ?? '');
  if (!m) return null;
  const [year, month, day] = [Number(m[1]), Number(m[2]), Number(m[3])];
  if (year < 1 || month < 1 || month > 12 || day < 1 || day > daysInMonth(year, month)) return null;
  return { year, month, day };
}

export const toISO = (pd) =>
  `${String(pd.year).padStart(4, '0')}-${String(pd.month).padStart(2, '0')}-${String(pd.day).padStart(2, '0')}`;

export function today() {
  const d = new Date();
  return { year: d.getFullYear(), month: d.getMonth() + 1, day: d.getDate() };
}

export function addDays(pd, n) {
  const d = new Date(utcTime(pd.year, pd.month - 1, pd.day + n));
  return { year: d.getUTCFullYear(), month: d.getUTCMonth() + 1, day: d.getUTCDate() };
}

/** Month arithmetic with the day clamped: Jan 31 + 1 month is Feb 28/29. */
export function addMonths(pd, n) {
  const total = pd.year * 12 + (pd.month - 1) + n;
  const year = Math.floor(total / 12);
  const month = (((total % 12) + 12) % 12) + 1;
  return { year, month, day: Math.min(pd.day, daysInMonth(year, month)) };
}

export const dayOfWeek = (pd) => new Date(utcTime(pd.year, pd.month - 1, pd.day)).getUTCDay();
export const diffDays = (a, b) =>
  Math.round((utcTime(b.year, b.month - 1, b.day) - utcTime(a.year, a.month - 1, a.day)) / MS_PER_DAY);
export const compare = (a, b) => a.year - b.year || a.month - b.month || a.day - b.day;
export const isBefore = (a, b) => compare(a, b) < 0;
export const isAfter = (a, b) => compare(a, b) > 0;
export const isEqual = (a, b) => compare(a, b) === 0;
export const isInRange = (pd, start, end) => compare(pd, start) >= 0 && compare(pd, end) <= 0;

/** ISO 8601 week number: the week's Thursday's day of the year, in weeks. */
export function weekNumber(pd) {
  const utc = utcTime(pd.year, pd.month - 1, pd.day);
  const dayNum = new Date(utc).getUTCDay() || 7;
  const thursday = utc + (4 - dayNum) * MS_PER_DAY;
  const yearStart = utcTime(new Date(thursday).getUTCFullYear(), 0, 1);
  return Math.ceil(((thursday - yearStart) / MS_PER_DAY + 1) / 7);
}

// Months: {start, end, length}.
export function monthOf(pd) {
  const length = daysInMonth(pd.year, pd.month);
  const start = { year: pd.year, month: pd.month, day: 1 };
  return { start, end: { ...start, day: length }, length };
}
export const addMonthsToMonth = (month, n) => (n === 0 ? month : monthOf(addMonths(month.start, n)));
export const isInMonth = (pd, month) => isInRange(pd, month.start, month.end);

// Formatting. Date fields alone format UTC midnight in UTC, so the host zone never matters.
const formats = new Map();
function formatter(locale, options) {
  const key = JSON.stringify([locale, options]);
  let f = formats.get(key);
  if (!f) {
    try {
      f = new Intl.DateTimeFormat(locale, { ...options, calendar: 'gregory', timeZone: 'UTC' });
    } catch {
      f = new Intl.DateTimeFormat('en-US', { ...options, calendar: 'gregory', timeZone: 'UTC' });
    }
    formats.set(key, f);
  }
  return f;
}
export const formatDate = (pd, options, locale) => formatter(locale, options).format(utcTime(pd.year, pd.month - 1, pd.day));

/** "Wednesday, May 21, 2026" */
export const DATE_WITH_WEEKDAY = { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' };

const segmenter = typeof Intl.Segmenter === 'function' ? new Intl.Segmenter(undefined, { granularity: 'grapheme' }) : null;
const graphemes = (s) => (segmenter ? [...segmenter.segment(s)].map((x) => x.segment) : [...s]);
// Languages whose CLDR stand-alone month names start with a capital.
const CAPITAL_MONTH_LANGUAGES = new Set(['es', 'fr', 'it', 'nl', 'pl', 'ru', 'uk']);
const languageOf = (locale) => {
  try {
    return new Intl.Locale(locale).language;
  } catch {
    return '';
  }
};

/** A stand-alone month and year label, such as "January 2026" or "Janvier 2026". */
export function formatMonthYear(month, locale) {
  const label = formatDate(month.start, { year: 'numeric', month: 'long' }, locale);
  if (!CAPITAL_MONTH_LANGUAGES.has(languageOf(locale))) return label;
  const first = graphemes(label)[0] ?? '';
  return first.toLocaleUpperCase(locale) + label.slice(first.length);
}

const numberFormats = new Map();
/** A day or week number in the locale's digits, with no grouping. */
export function formatNumber(n, locale) {
  let f = numberFormats.get(locale);
  if (!f) {
    try {
      f = new Intl.NumberFormat(locale, { useGrouping: false });
    } catch {
      f = new Intl.NumberFormat('en-US', { useGrouping: false });
    }
    numberFormats.set(locale, f);
  }
  return f.format(n);
}

/** The locale's word for the week-of-year field ("week", "Woche"). */
export function weekFieldName(locale) {
  try {
    const name = new Intl.DisplayNames(locale, { type: 'dateTimeField' }).of('weekOfYear');
    if (name) return name;
  } catch {
    /* fall back */
  }
  return 'week';
}

// CLDR's short weekday names (Unicode CLDR 48.2.0, from Astryx's generated table), Sunday first.
const WEEKDAY_HEADERS = {
  af: ['So.', 'Ma.', 'Di.', 'Wo.', 'Do.', 'Vr.', 'Sa.'],
  ar: ['أحد', 'إثنين', 'ثلاثاء', 'أربعاء', 'خميس', 'جمعة', 'سبت'],
  'ar-SA': ['أحد', 'إثنين', 'ثلاثاء', 'أربعاء', 'خميس', 'جمعة', 'سبت'],
  ca: ['dg.', 'dl.', 'dt.', 'dc.', 'dj.', 'dv.', 'ds.'],
  cs: ['ne', 'po', 'út', 'st', 'čt', 'pá', 'so'],
  da: ['sø.', 'ma.', 'ti.', 'on.', 'to.', 'fr.', 'lø.'],
  de: ['So', 'Mo', 'Di', 'Mi', 'Do', 'Fr', 'Sa'],
  el: ['Κυρ', 'Δευ', 'Τρί', 'Τετ', 'Πέμ', 'Παρ', 'Σάβ'],
  en: ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'],
  es: ['dom', 'lun', 'mar', 'mié', 'jue', 'vie', 'sáb'],
  fi: ['su', 'ma', 'ti', 'ke', 'to', 'pe', 'la'],
  fr: ['dim.', 'lun.', 'mar.', 'mer.', 'jeu.', 'ven.', 'sam.'],
  he: ['א׳', 'ב׳', 'ג׳', 'ד׳', 'ה׳', 'ו׳', 'ש׳'],
  hu: ['V', 'H', 'K', 'Sze', 'Cs', 'P', 'Szo'],
  it: ['dom', 'lun', 'mar', 'mer', 'gio', 'ven', 'sab'],
  ja: ['日', '月', '火', '水', '木', '金', '土'],
  ko: ['일', '월', '화', '수', '목', '금', '토'],
  nl: ['zo', 'ma', 'di', 'wo', 'do', 'vr', 'za'],
  no: ['søn.', 'man.', 'tir.', 'ons.', 'tor.', 'fre.', 'lør.'],
  pl: ['nie', 'pon', 'wto', 'śro', 'czw', 'pią', 'sob'],
  pt: ['dom.', 'seg.', 'ter.', 'qua.', 'qui.', 'sex.', 'sáb.'],
  'pt-PT': ['dom.', 'seg.', 'ter.', 'qua.', 'qui.', 'sex.', 'sáb.'],
  ro: ['dum.', 'lun.', 'mar.', 'mie.', 'joi', 'vin.', 'sâm.'],
  ru: ['вс', 'пн', 'вт', 'ср', 'чт', 'пт', 'сб'],
  sr: ['нед', 'пон', 'уто', 'сре', 'чет', 'пет', 'суб'],
  sv: ['sön', 'mån', 'tis', 'ons', 'tors', 'fre', 'lör'],
  tr: ['Paz', 'Pzt', 'Sal', 'Çar', 'Per', 'Cum', 'Cmt'],
  uk: ['нд', 'пн', 'вт', 'ср', 'чт', 'пт', 'сб'],
  vi: ['CN', 'T2', 'T3', 'T4', 'T5', 'T6', 'T7'],
  zh: ['周日', '周一', '周二', '周三', '周四', '周五', '周六'],
  'zh-TW': ['週日', '週一', '週二', '週三', '週四', '週五', '週六'],
};

/** Sunday-first header names: the CLDR table, else Intl's short names when they fit, else narrow. */
function sundayFirstNames(locale) {
  let parsed;
  try {
    parsed = new Intl.Locale(locale);
  } catch {
    return WEEKDAY_HEADERS.en;
  }
  const table = WEEKDAY_HEADERS[parsed.baseName] ?? WEEKDAY_HEADERS[parsed.language];
  if (table) return table;
  const names = (width) => {
    const f = formatter(parsed.baseName, { weekday: width });
    // A language the platform lacks resolves to the host's default locale.
    if (new Intl.Locale(f.resolvedOptions().locale).language !== parsed.language) return null;
    return Array.from({ length: 7 }, (_, d) => formatDate({ year: 1970, month: 1, day: 4 + d }, { weekday: width }, parsed.baseName));
  };
  const short = names('short');
  if (short == null) return WEEKDAY_HEADERS.en;
  if (short.every((n) => graphemes(n).length <= 4)) return short;
  return names('narrow') ?? WEEKDAY_HEADERS.en;
}

/** The seven weekday header names in column order for a week starting on weekStartsOn. */
export function weekdayHeaders(locale, weekStartsOn) {
  const names = sundayFirstNames(locale);
  return Array.from({ length: 7 }, (_, i) => names[(weekStartsOn + i) % 7]);
}

/** A month's grid days: six rows, or only the rows it needs with variableRows. */
export function monthWeeks(month, weekStartsOn, variableRows) {
  let lead = dayOfWeek(month.start) - weekStartsOn;
  if (lead < 0) lead += 7;
  const rows = variableRows ? Math.ceil((month.length + lead) / 7) : 6;
  const weeks = [];
  for (let r = 0; r < rows; r++) {
    const week = [];
    for (let c = 0; c < 7; c++) {
      const offset = r * 7 + c - lead;
      const date = addDays(month.start, offset);
      week.push({ date, iso: toISO(date), isOutside: offset < 0 || offset >= month.length });
    }
    weeks.push(week);
  }
  return weeks;
}

/**
 * The date whose month the calendar opens on: focusDate, else the value (shifted so a second
 * pane isn't wholly past max), else today clamped into min/max.
 */
export function initialFocusDate({ focusDate, value, min, max, months, today: now }) {
  if (focusDate) return focusDate;
  const lastPane = Math.max(0, months - 1);
  if (value) return keepLastPaneWithinMax(value, min, max, lastPane);
  if (min && isBefore(now, min)) return min;
  if (max && isAfter(now, max)) {
    const shifted = addMonthsToMonth(monthOf(max), -lastPane).start;
    if (min && isBefore(shifted, monthOf(min).start)) return min;
    return shifted;
  }
  return keepLastPaneWithinMax(now, min, max, lastPane);
}

function keepLastPaneWithinMax(date, min, max, lastPane) {
  if (max == null || lastPane === 0) return date;
  const month = monthOf(date).start;
  const maxMonth = monthOf(max);
  const latestFirst = addMonthsToMonth(maxMonth, -lastPane).start;
  if (!isAfter(month, latestFirst) || isAfter(month, maxMonth.start)) return date;
  if (min == null) return latestFirst;
  const minMonth = monthOf(min).start;
  if (!isBefore(latestFirst, minMonth)) return latestFirst;
  return isAfter(month, minMonth) ? min : date;
}
