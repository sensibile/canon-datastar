// NumberInput's parsing and commit rules. Ported from Astryx (packages/core/src/NumberInput/
// numberParser.ts, numberInputCommit.ts, internal/unicodeDigits.ts), MIT, Meta Platforms, Inc.
// Reads numbers the way people paste them: locale grouping and decimal separators, any digit
// script, currency symbols, accounting negatives, exponents. Text no reading fits is null.

const DECIMAL_DIGIT = new RegExp('\\p{Nd}', 'u');
/** Zero-width joiners, bidi marks and isolates, and the BOM Excel prepends. */
const INVISIBLE_CHARS = /[​-‍‎‏؜⁦-⁩﻿]/g;

function digitValue(char) {
  const codePoint = char.codePointAt(0);
  if (codePoint == null || !DECIMAL_DIGIT.test(char)) return null;
  for (let zero = codePoint; codePoint - zero < 10; zero--) {
    if (!DECIMAL_DIGIT.test(String.fromCodePoint(zero - 1))) return codePoint - zero;
  }
  return null;
}

function toAsciiDigits(text) {
  let out = '';
  for (const char of text) {
    if (!DECIMAL_DIGIT.test(char)) {
      out += char;
      continue;
    }
    const value = digitValue(char);
    if (value == null) return null;
    out += String(value);
  }
  return out;
}

const symbolsByLocale = new Map();

function localeSymbols(locale) {
  const key = locale ?? '';
  let symbols = symbolsByLocale.get(key);
  if (symbols === undefined) {
    let format;
    try {
      format = new Intl.NumberFormat(locale);
    } catch {
      format = new Intl.NumberFormat('en');
    }
    const parts = format.formatToParts(12345.6);
    const groupSizes = format
      .formatToParts(11111111111)
      .filter((part) => part.type === 'integer')
      .map((part) => part.value.length);
    const hasGrouping = groupSizes.length > 1;
    symbols = {
      group: (parts.find((p) => p.type === 'group')?.value ?? ',').normalize('NFKC'),
      decimal: parts.find((p) => p.type === 'decimal')?.value ?? '.',
      primaryGroupSize: hasGrouping ? groupSizes[groupSizes.length - 1] : 3,
      secondaryGroupSize: hasGrouping ? groupSizes[groupSizes.length - 2] : 3,
    };
    symbolsByLocale.set(key, symbols);
  }
  return symbols;
}

const CURRENCY = new RegExp('\\p{Sc}', 'u');
const MINUS_SIGNS = '−‒–-';
const SEPARATOR_CHARS = ",.' ’٫٬";
const NUMBER_BODY = new RegExp(`^[0-9${SEPARATOR_CHARS}]+$`);
const MACHINE_NUMBER = /^(?:\d+\.?\d*|\.\d+)$/;

function readUnder(core, group, decimal, primaryGroupSize, secondaryGroupSize, minGroups) {
  let intText = core;
  let fracText = null;
  if (decimal !== '' && core.includes(decimal)) {
    const at = core.indexOf(decimal);
    if (core.includes(decimal, at + decimal.length)) return null;
    intText = core.slice(0, at);
    fracText = core.slice(at + decimal.length);
    if (!/^\d*$/.test(fracText)) return null;
  }
  let digits;
  if (group !== '' && intText.includes(group)) {
    const chunks = intText.split(group);
    if (chunks.length - 1 < minGroups || !chunks.every((chunk) => /^\d+$/.test(chunk))) return null;
    const first = chunks[0];
    const last = chunks[chunks.length - 1];
    if (last.length !== primaryGroupSize || first.length > secondaryGroupSize) return null;
    for (let i = 1; i < chunks.length - 1; i++) if (chunks[i].length !== secondaryGroupSize) return null;
    digits = chunks.join('');
  } else {
    if (!/^\d*$/.test(intText)) return null;
    digits = intText;
  }
  if (digits === '' && (fracText == null || fracText === '')) return null;
  return fracText == null ? digits : `${digits}.${fracText}`;
}

const isGroupOnlySeparator = (separator) => /^[\s'’]$/.test(separator);

function toPlainNumber(core, locale) {
  if (!NUMBER_BODY.test(core)) return null;
  const { group, decimal, primaryGroupSize, secondaryGroupSize } = localeSymbols(locale);
  const inLocale = readUnder(core, group, decimal, primaryGroupSize, secondaryGroupSize, 1);
  if (inLocale != null) return inLocale;
  if (MACHINE_NUMBER.test(core)) return core;
  const separators = [...new Set(core.replace(/\d/g, ''))];
  if (separators.length === 0 || separators.length > 2) return null;
  for (const candidate of separators) {
    const occurrences = core.split(candidate).length - 1;
    const minGroups = isGroupOnlySeparator(candidate) ? 1 : 2;
    if (occurrences < minGroups) continue;
    const other = separators.find((s) => s !== candidate) ?? '';
    if (other !== '' && core.split(other).length - 1 !== 1) continue;
    const read = readUnder(core, candidate, other, 3, 3, minGroups);
    if (read != null) return read;
  }
  return null;
}

function stripCurrency(text) {
  let out = text.trim();
  if (CURRENCY.test(out.charAt(0))) out = out.slice(1).trim();
  if (CURRENCY.test(out.charAt(out.length - 1))) out = out.slice(0, -1).trim();
  return out;
}

/** Read a number out of text the way a person in `locale` writes one, or null. */
export function parseLocaleNumber(text, locale) {
  const normalized = toAsciiDigits(text.normalize('NFKC').replace(INVISIBLE_CHARS, ''));
  if (normalized == null) return null;
  let rest = normalized.trim();
  let sign = 1;
  const accounting = /^\((.*)\)$/s.exec(rest);
  if (accounting) {
    sign = -1;
    rest = accounting[1].trim();
  }
  rest = stripCurrency(rest);
  let hasWrittenSign = false;
  const leadingSign = new RegExp(`^([+${MINUS_SIGNS}])\\s*`).exec(rest);
  if (leadingSign) {
    hasWrittenSign = true;
    sign = leadingSign[1] === '+' ? sign : -sign;
    rest = rest.slice(leadingSign[0].length);
  } else {
    const trailingSign = new RegExp(`\\s*([+${MINUS_SIGNS}])$`).exec(rest);
    if (trailingSign) {
      hasWrittenSign = true;
      sign = trailingSign[1] === '+' ? sign : -sign;
      rest = rest.slice(0, rest.length - trailingSign[0].length);
    }
  }
  if (accounting && hasWrittenSign) return null;
  rest = stripCurrency(rest);
  let exponent = 0;
  const exponentMatch = /\s*[eE]\s*([+-]?\d+)$/.exec(rest);
  if (exponentMatch) {
    exponent = Number(exponentMatch[1]);
    rest = rest.slice(0, rest.length - exponentMatch[0].length);
  }
  const core = rest.trim();
  if (core === '') return null;
  const plain = toPlainNumber(core, locale);
  if (plain == null) return null;
  const magnitude = Number(plain);
  if (!Number.isFinite(magnitude)) return null;
  const scaled = exponent === 0 ? magnitude : Number(`${plain}e${exponent}`);
  if (!Number.isFinite(scaled)) return null;
  const result = sign * scaled;
  return Object.is(result, -0) ? 0 : result;
}

/** The text shown while focused: the plain number with the locale's decimal separator. */
export function formatEditableNumber(value, locale) {
  const { decimal } = localeSymbols(locale);
  const raw = String(value);
  return decimal === '.' ? raw : raw.replace('.', decimal);
}

function parseNumericInput(input, options) {
  const trimmed = input.trim();
  if (trimmed === '') return null;
  const value = parseLocaleNumber(trimmed, options.locale);
  if (value == null || !Number.isFinite(value)) return null;
  if (options.isIntegerOnly && !Number.isInteger(value)) return null;
  return value;
}

/** A value inside the range, or null. */
export function parseNumberInput(input, options) {
  const value = parseNumericInput(input, options);
  if (value === null) return null;
  if (options.min != null && value < options.min) return null;
  if (options.max != null && value > options.max) return null;
  return value;
}

function commitRange(options) {
  const min = options.min == null ? null : options.isIntegerOnly ? Math.ceil(options.min) : options.min;
  const max = options.max == null ? null : options.isIntegerOnly ? Math.floor(options.max) : options.max;
  return { min, max };
}

/** Why a draft cannot commit ({kind: 'unreadable'|'fractional'|'outOfRange'}), or null. */
export function getNumberInputRejection(input, options) {
  const trimmed = input.trim();
  if (trimmed === '') return null;
  const value = parseLocaleNumber(trimmed, options.locale);
  if (value == null || !Number.isFinite(value)) return { kind: 'unreadable' };
  if (options.isIntegerOnly && !Number.isInteger(value)) return { kind: 'fractional', value };
  const { min, max } = commitRange(options);
  if (min != null && max != null && min > max) return { kind: 'outOfRange', value };
  return null;
}

/** {type: 'commit', value, didClamp} | {type: 'clear'} | {type: 'revert'}. */
export function resolveNumberInputCommit(input, options) {
  if (input.trim() === '') return options.hasClear ? { type: 'clear' } : { type: 'revert' };
  const value = parseNumericInput(input, options);
  if (value === null) return { type: 'revert' };
  const { min, max } = commitRange(options);
  if (min != null && max != null && min > max) return { type: 'revert' };
  const committed = min != null && value < min ? min : max != null && value > max ? max : value;
  return { type: 'commit', value: committed, didClamp: committed !== value };
}
