import { useEffect, useRef, useState, type ReactNode } from 'react';
import { createPortal } from 'react-dom';
import {
  Apple,
  ArrowLeft,
  ArrowRight,
  ArrowUpRight,
  BarChart3,
  BookOpen,
  Camera,
  Check,
  CheckCheck,
  CheckCircle2,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Clock3,
  Coffee,
  Egg,
  History,
  ImagePlus,
  Leaf,
  ListChecks,
  LoaderCircle,
  Milk,
  MoreHorizontal,
  Package,
  Pencil,
  HeartPulse,
  Plus,
  ReceiptText,
  RotateCcw,
  Search,
  ShieldCheck,
  ShoppingBag,
  Sparkles,
  Utensils,
  Wifi,
  WifiOff,
  X,
  AlertCircle,
  PawPrint,
  Cpu,
  type LucideIcon,
} from 'lucide-react';
import {
  CATEGORIES,
  currencyScale,
  decimalAmount,
  dateLabel,
  crewDate,
  findProduct,
  money,
  monthLabel,
  normalize,
  parseMoney,
  photoExpired,
  type CategoryId,
  type CorrectionRule,
  type Product,
  type Receipt,
  type ReceiptItem,
  summarize,
} from './shoppingModels';
import { demoPhoto } from './shoppingDemoPhoto';
const categoryIcons: Record<CategoryId, LucideIcon> = {
  food: Apple,
  restaurants: Utensils,
  home: Package,
  pets: PawPrint,
  hobbies: Cpu,
  clothes: ShoppingBag,
  transport: ArrowUpRight,
  other: ReceiptText,
  health: HeartPulse,
  leisure: Sparkles,
  services: Package,
};
const productIcons: Record<string, LucideIcon> = {
  milk: Milk,
  eggs: Egg,
  apples: Apple,
  coffee: Coffee,
  bread: Leaf,
};
export function ProductIcon({ id, category = 'food' }: { id?: string; category?: CategoryId }) {
  const Icon = (id && productIcons[id]) || categoryIcons[category];
  return <Icon size={21} strokeWidth={1.65} />;
}
export function Modal({
  title,
  children,
  onClose,
  wide = false,
  full = false,
}: {
  title: string;
  children: ReactNode;
  onClose: () => void;
  wide?: boolean;
  full?: boolean;
}) {
  const root = useRef<HTMLDivElement>(null);
  const returnFocus = useRef(document.activeElement as HTMLElement | null);
  const close = useRef(onClose);
  close.current = onClose;
  useEffect(() => {
    const previous = returnFocus.current;
    const overflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    root.current?.focus();
    const key = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        close.current();
      }
      if (event.key !== 'Tab') return;
      const controls = [
        ...(root.current?.querySelectorAll<HTMLElement>(
          'button:not(:disabled), input:not(:disabled), select, textarea, [href], [tabindex="0"]',
        ) || []),
      ].filter((el) => el.getClientRects().length);
      const first = controls[0];
      const last = controls.at(-1);
      if (!first) {
        event.preventDefault();
        return;
      }
      if (
        event.shiftKey &&
        (document.activeElement === first || document.activeElement === root.current)
      ) {
        event.preventDefault();
        last?.focus();
      } else if (
        !event.shiftKey &&
        (document.activeElement === last || document.activeElement === root.current)
      ) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener('keydown', key);
    return () => {
      document.body.style.overflow = overflow;
      document.removeEventListener('keydown', key);
      queueMicrotask(() => previous?.focus());
    };
  }, []);
  return createPortal(
    <div
      className={`sp-overlay ${full ? 'sp-overlay-full' : ''}`}
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <div
        ref={root}
        tabIndex={-1}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        className={`sp-modal ${wide ? 'sp-modal-wide' : ''} ${full ? 'sp-modal-full' : ''}`}
      >
        <div className="sp-modal-head">
          <h2>{title}</h2>
          <button className="sp-icon-button" onClick={onClose} aria-label="Close">
            <X size={21} />
          </button>
        </div>
        {children}
      </div>
    </div>,
    document.body,
  );
}

export function ReceiptRow({ receipt: r, onClick }: { receipt: Receipt; onClick: () => void }) {
  const rootCategory = r.items[0]?.category || 'other';
  const Icon = ['queued', 'processing', 'failed'].includes(r.status)
    ? Clock3
    : categoryIcons[rootCategory];
  return (
    <button className="sp-receipt-row" onClick={onClick}>
      <span className={`sp-merchant-icon sp-category-${rootCategory}`}>
        <Icon size={23} />
      </span>
      <span className="sp-receipt-row-info">
        <strong>{r.merchant}</strong>
        <span>
          {['queued', 'processing', 'failed'].includes(r.status)
            ? r.status === 'failed'
              ? "Couldn't read it"
              : r.status === 'processing'
                ? 'Reading the receipt'
                : 'Photo waiting for the scanner'
            : r.items
                .map((i) => i.name)
                .slice(0, 3)
                .join(', ') + (r.items.length > 3 ? '…' : '')}
        </span>
        <small>
          {r.status === 'review' ? (
            <span className="sp-status-review">
              <AlertCircle size={12} /> Needs a look · not counted yet
            </span>
          ) : ['queued', 'processing', 'failed'].includes(r.status) ? (
            <span className="sp-status-queued">Not counted</span>
          ) : (
            <>
              {r.items.length} items <span>·</span>{' '}
              {photoExpired(r.date, r.photoExpiresAt)
                ? 'Photo deleted after 90 days'
                : 'Receipt saved'}
            </>
          )}
        </small>
      </span>
      <span className="sp-receipt-row-total">
        {['queued', 'processing', 'failed'].includes(r.status) ? (
          <Clock3 size={19} />
        ) : (
          <>
            <strong>{money(r.total, r.currency)}</strong>
            <small>
              {[...new Set(r.items.map((i) => CATEGORIES[i.category].name))]
                .slice(0, 2)
                .join(' · ')}
            </small>
          </>
        )}
      </span>
      <ChevronRight size={17} />
    </button>
  );
}

export function Donut({
  summary,
  currency,
  onSelect,
}: {
  summary: ReturnType<typeof summarize>;
  currency: string;
  onSelect: (id: CategoryId) => void;
}) {
  const radius = 83;
  const length = 2 * Math.PI * radius;
  let offset = 0;
  return (
    <svg
      className="sp-donut"
      viewBox="0 0 240 240"
      role="img"
      aria-label={`Spending by category. Total ${money(summary.total, currency)}`}
    >
      <circle cx="120" cy="120" r={radius} fill="none" stroke="var(--line)" strokeWidth="25" />
      {summary.byCategory.map((c) => {
        const segment = (Math.abs(c.amount) / Math.max(summary.turnover, 1)) * length;
        const currentOffset = offset;
        offset += segment;
        return (
          <circle
            key={c.id}
            cx="120"
            cy="120"
            r={radius}
            fill="none"
            stroke={c.color}
            strokeWidth="25"
            strokeDasharray={`${Math.max(segment - 4, 0)} ${length - Math.max(segment - 4, 0)}`}
            strokeDashoffset={-currentOffset}
            transform="rotate(-90 120 120)"
            onClick={() => onSelect(c.id)}
            style={{ cursor: 'pointer' }}
          >
            <title>
              {c.name}: {money(c.amount, currency)}
            </title>
          </circle>
        );
      })}
      <text x="120" y="111" textAnchor="middle" className="sp-donut-label">
        THIS MONTH
      </text>
      <text x="120" y="139" textAnchor="middle" className="sp-donut-value">
        {money(summary.total, currency)}
      </text>
    </svg>
  );
}
export function DailyChart({
  days,
  currency,
}: {
  days: { date: string; amount: number }[];
  currency: string;
}) {
  const max = Math.max(...days.map((d) => Math.abs(d.amount)), currencyScale(currency));
  const top = Math.ceil(max / (10 * currencyScale(currency))) * 10 * currencyScale(currency);
  return (
    <div
      className="sp-daily-plot"
      role="img"
      aria-label={`Spending by day, ${currency}. ${days.map((d) => `${dateLabel(d.date)}: ${money(d.amount, currency)}`).join('; ')}`}
    >
      <span className="sp-daily-axis-title">Spending, {currency}</span>
      <div className="sp-daily-plot-inner">
        <div className="sp-daily-y-axis">
          <span>{top / currencyScale(currency)}</span>
          <span>{top / (2 * currencyScale(currency))}</span>
          <span>0</span>
        </div>
        <div className="sp-daily-series">
          <div className="sp-daily-grid">
            <i />
            <i />
            <i />
          </div>
          {days.map((d) => (
            <div
              className="sp-daily-column"
              key={d.date}
              style={{ left: `${((Number(d.date.slice(-2)) - 0.5) / 31) * 100}%` }}
            >
              <span
                className="sp-daily-stick"
                style={{
                  height: `${(Math.abs(d.amount) / top) * 100}%`,
                  background: d.amount < 0 ? 'var(--peach-ink)' : undefined,
                }}
                title={`${dateLabel(d.date)}: ${money(d.amount, currency)}`}
              />
              <span className="sp-daily-day-label">{Number(d.date.slice(-2))}</span>
            </div>
          ))}
        </div>
      </div>
      <span className="sp-daily-axis-bottom">Day of month</span>
    </div>
  );
}

type DraftItem = ReceiptItem & {
  amountText: string;
  quantityText: string;
  productTypeEdited?: boolean;
};
export function ReceiptDialog({
  receipt,
  catalog,
  onClose,
  onSave,
  onRepeat,
  demo = true,
  online = true,
  onRetry,
  onDelete,
  today = crewDate(),
}: {
  receipt: Receipt;
  catalog: Product[];
  onClose: () => void;
  onSave: (
    receipt: Receipt,
    confirmDuplicate?: boolean,
    allowIncomplete?: boolean,
  ) => void | Promise<void>;
  demo?: boolean;
  online?: boolean;
  onRetry?: () => Promise<void>;
  onDelete?: () => Promise<void>;
  onRepeat: () => void;
  today?: string;
}) {
  const [editing, setEditing] = useState(false);
  const openedRevision = useRef(receipt.revision);
  const [saving, setSaving] = useState(false);
  const [success, setSuccess] = useState('');
  const saveError = useRef<HTMLParagraphElement>(null);
  const allowDraft = !demo && receipt.status === 'review';
  const [confirmDuplicate, setConfirmDuplicate] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [time, setTime] = useState(receipt.time || '');
  const [merchant, setMerchant] = useState(receipt.merchant);
  const [date, setDate] = useState(receipt.dateMissing ? '' : receipt.date);
  const [acceptedDate, setAcceptedDate] = useState<string | null>(null);
  const [currency, setCurrency] = useState(receipt.currency);
  const [totalText, setTotalText] = useState(decimalAmount(receipt.total, receipt.currency));
  const [items, setItems] = useState<DraftItem[]>(
    receipt.items.map((i) => ({
      ...i,
      productName: i.productName || catalog.find((p) => p.id === i.productId)?.name || i.name,
      amountText: decimalAmount(i.amount, receipt.currency),
      quantityText: String(i.quantity),
    })),
  );
  const [expandedPhoto, setExpandedPhoto] = useState(false);
  const [photoIndex, setPhotoIndex] = useState(0);
  const [error, setError] = useState('');
  useEffect(() => {
    if (error) saveError.current?.scrollIntoView?.({ block: 'nearest' });
  }, [error]);
  const expired = photoExpired(receipt.date, receipt.photoExpiresAt);
  const itemSum = receipt.items.reduce((sum, i) => sum + i.amount, 0);
  const draftSum = items.reduce((sum, i) => sum + (parseMoney(i.amountText, currency) || 0), 0);
  const total = parseMoney(totalText, currency);
  const photos = receipt.photos || (demo ? [demoPhoto(receipt)] : []);
  const shownDate = editing ? date : receipt.dateMissing ? '' : receipt.date;
  const differentYear =
    /^\d{4}-\d{2}-\d{2}$/.test(shownDate) && shownDate.slice(0, 4) !== today.slice(0, 4);
  const showDateChoice =
    !demo &&
    (editing || receipt.status === 'review') &&
    (!shownDate || (differentYear && acceptedDate !== shownDate));
  function beginEdit() {
    cancelEdit();
    openedRevision.current = receipt.revision;
    setEditing(true);
  }
  function chooseToday() {
    if (!editing) beginEdit();
    setDate(today);
    setAcceptedDate(null);
  }
  function keepDate() {
    if (!editing) beginEdit();
    setAcceptedDate(shownDate);
  }
  function updateItem(id: string, patch: Partial<DraftItem>) {
    setItems((all) => all.map((i) => (i.id === id ? { ...i, ...patch } : i)));
  }
  function useCorrection() {
    setItems((all) =>
      all.map((i) =>
        i.uncertain
          ? {
              ...i,
              name: 'Apples',
              productId: 'apples',
              category: 'food',
              subcategory: 'Fruit & vegetables',
              uncertain: false,
              amountText: ((i.amount + 170) / 100).toFixed(2),
            }
          : i,
      ),
    );
    setEditing(true);
  }
  async function save() {
    if (!demo && openedRevision.current !== receipt.revision) {
      setError(
        'Someone changed this receipt on another device. Close the editor and open it again to see their changes.',
      );
      return;
    }
    if (
      !merchant.trim() ||
      ((!allowDraft || date !== '') && !/^\d{4}-\d{2}-\d{2}$/.test(date)) ||
      !/^[A-Z]{3}$/.test(currency) ||
      total === null ||
      (!allowDraft && total !== draftSum) ||
      items.some(
        (i) =>
          !i.name.trim() ||
          !i.productName?.trim() ||
          (!allowDraft && i.uncertain) ||
          parseMoney(i.amountText, currency) === null ||
          !Number.isFinite(Number(i.quantityText.replace(',', '.'))) ||
          Number(i.quantityText.replace(',', '.')) <= 0,
      )
    ) {
      setError(
        'Check the names, quantities and amounts. The items must add up to the receipt total.',
      );
      return;
    }
    setSaving(true);
    try {
      const operation = onSave(
        {
          ...receipt,
          revision: openedRevision.current,
          merchant: merchant.trim(),
          date,
          dateMissing: !date.trim(),
          time: time || null,
          currency,
          total,
          status: allowDraft ? 'review' : 'saved',
          items: items.map(({ amountText, quantityText, productTypeEdited: _edited, ...i }) => ({
            ...i,
            name: i.name.trim(),
            rawName: i.rawName.trim(),
            productName: i.productName?.trim(),
            amount: parseMoney(amountText, currency)!,
            quantity: Number(quantityText.replace(',', '.')),
          })),
        },
        confirmDuplicate,
        allowDraft,
      );
      if (operation) await operation;
      openedRevision.current = (openedRevision.current || 0) + 1;
      setEditing(false);
      setError('');
      setSuccess('Fixes saved.');
      setAcceptedDate(date);
    } catch (error) {
      setError(error instanceof Error ? error.message : "Couldn't save the receipt");
    } finally {
      setSaving(false);
    }
  }
  function cancelEdit() {
    setMerchant(receipt.merchant);
    setDate(receipt.dateMissing ? '' : receipt.date);
    setTime(receipt.time || '');
    setCurrency(receipt.currency);
    setTotalText(decimalAmount(receipt.total, receipt.currency));
    setItems(
      receipt.items.map((item) => ({
        ...item,
        productName:
          item.productName || catalog.find((p) => p.id === item.productId)?.name || item.name,
        amountText: decimalAmount(item.amount, receipt.currency),
        quantityText: String(item.quantity),
      })),
    );
    setEditing(false);
    setError('');
    setSuccess('');
    setAcceptedDate(null);
  }
  return (
    <Modal
      title={
        ['queued', 'processing', 'failed'].includes(receipt.status)
          ? 'Receipt in the queue'
          : editing
            ? 'Fix the receipt'
            : `Receipt · ${receipt.merchant}`
      }
      wide
      onClose={onClose}
    >
      <div className="sp-modal-body">
        {['queued', 'processing', 'failed'].includes(receipt.status) ? (
          <div className="sp-queued-detail">
            <Clock3 size={32} />
            <h3>
              {receipt.status === 'failed'
                ? "Couldn't read the receipt"
                : receipt.status === 'processing'
                  ? 'Reading the receipt'
                  : 'Photo saved in the queue'}
            </h3>
            <p>
              Items and the total show up once you are back online. Until then it is not counted.
            </p>
            <p className="sp-muted">
              {demo
                ? 'In this demo, press “Reconnect” on the main screen.'
                : 'The original is saved. The server picks it up as soon as you are back online.'}
            </p>
            {photos.map((url, index) => (
              <img key={index} src={url} alt={`Queued photo ${index + 1}`} />
            ))}
          </div>
        ) : (
          <>
            {showDateChoice && (
              <div className="sp-date-choice" role="alert">
                <strong>
                  {shownDate
                    ? 'The year on the receipt is not this year'
                    : 'Check the purchase date'}
                </strong>
                <p>
                  {shownDate
                    ? `Read as ${dateLabel(shownDate)} ${shownDate.slice(0, 4)}, but it is ${today.slice(0, 4)}. Compare the year with the photo.`
                    : 'The date on the photo is unclear. Enter it or pick today.'}
                </p>
                <div>
                  <button className="sp-primary" disabled={!online || saving} onClick={chooseToday}>
                    Use today's date
                  </button>
                  {shownDate && (
                    <button
                      className="sp-secondary"
                      disabled={!online || saving}
                      onClick={keepDate}
                    >
                      {shownDate === receipt.date ? 'Keep the receipt date' : 'Keep this date'}
                    </button>
                  )}
                </div>
                <small>The date applies when you save the receipt.</small>
              </div>
            )}
            {receipt.status === 'review' && (
              <div className="sp-review-callout">
                <AlertCircle size={21} />
                <div>
                  <strong>{demo ? 'One item needs a look' : 'Check what we read'}</strong>
                  <p>
                    Items: {money(itemSum, receipt.currency)}. Receipt total:{' '}
                    {money(receipt.total, receipt.currency)}.<br />
                    {itemSum !== receipt.total && (
                      <>Difference: {money(receipt.total - itemSum, receipt.currency)}. </>
                    )}
                    While anything needs a look, the receipt is not counted. You can save fixes bit
                    by bit.
                  </p>
                  {demo && (
                    <button className="sp-text-link" onClick={useCorrection}>
                      Fill in the demo fix <ArrowRight size={14} />
                    </button>
                  )}
                </div>
              </div>
            )}
            <div className={`sp-receipt-detail ${expandedPhoto ? 'sp-photo-expanded' : ''}`}>
              <aside className="sp-receipt-photo">
                <div className="sp-section-line">
                  <span className="sp-kicker">ORIGINAL RECEIPT</span>
                  {!expired && (
                    <span className="sp-muted">
                      {photos.length > 1
                        ? `${photoIndex + 1}/${photos.length}`
                        : demo
                          ? 'Demo'
                          : ''}
                    </span>
                  )}
                </div>
                {expired ? (
                  <div className="sp-expired-photo">
                    <ReceiptText size={33} />
                    <strong>Photo deleted</strong>
                    <p>90 days passed. Every item, price and total is still in history.</p>
                  </div>
                ) : (
                  <>
                    {!demo && !photos.length && (
                      <p className="sp-muted">
                        {receipt.photoError ||
                          (online ? 'Loading photo…' : 'The photo shows up when you are online')}
                      </p>
                    )}
                    {photos.length > 0 && (
                      <button
                        className="sp-photo-button"
                        aria-label={expandedPhoto ? 'Zoom out' : 'Zoom in'}
                        onClick={() => setExpandedPhoto(!expandedPhoto)}
                      >
                        <img
                          src={photos[photoIndex]}
                          alt={`Original receipt ${receipt.merchant}, ${dateLabel(receipt.date)}`}
                        />
                      </button>
                    )}
                    {photos.length > 1 && (
                      <div className="sp-photo-pager">
                        <button
                          className="sp-icon-button"
                          disabled={photoIndex === 0}
                          aria-label="Previous photo"
                          onClick={() => setPhotoIndex((i) => i - 1)}
                        >
                          <ChevronLeft size={17} />
                        </button>
                        <span>
                          {photoIndex + 1} of {photos.length}
                        </span>
                        <button
                          className="sp-icon-button"
                          disabled={photoIndex === photos.length - 1}
                          aria-label="Next photo"
                          onClick={() => setPhotoIndex((i) => i + 1)}
                        >
                          <ChevronRight size={17} />
                        </button>
                      </div>
                    )}
                    <small>Kept for 90 days after upload</small>
                  </>
                )}
              </aside>
              <div className="sp-receipt-content">
                {editing ? (
                  <>
                    <div className="sp-edit-meta">
                      <label className="sp-field">
                        Store
                        <input value={merchant} onChange={(e) => setMerchant(e.target.value)} />
                      </label>
                      <label className="sp-field">
                        Date
                        <input type="date" value={date} onChange={(e) => setDate(e.target.value)} />
                      </label>
                      <label className="sp-field">
                        Currency (code)
                        <input
                          value={currency}
                          maxLength={3}
                          onChange={(e) => setCurrency(e.target.value.toUpperCase())}
                          placeholder="EUR"
                        />
                      </label>
                    </div>
                    <label className="sp-field">
                      Time on the receipt (if printed)
                      <input type="time" value={time} onChange={(e) => setTime(e.target.value)} />
                    </label>
                    <p className="sp-fine-print">
                      You can save fixes bit by bit. While the date, amounts or items need a look,
                      the receipt is not counted. Fixed names are remembered for this store's next
                      receipts.
                    </p>
                    <datalist id="sp-product-types">
                      {catalog.map((p) => (
                        <option key={p.id} value={p.name} />
                      ))}
                    </datalist>
                    <div className="sp-edit-items">
                      {items.map((item, index) => (
                        <div
                          className={`sp-edit-item ${item.uncertain ? 'sp-uncertain' : ''}`}
                          key={item.id}
                        >
                          <label className="sp-field">
                            Item {index + 1}
                            <input
                              aria-label={`Item ${index + 1} name`}
                              value={item.name}
                              onChange={(e) => {
                                const p = findProduct(e.target.value, catalog);
                                updateItem(item.id, {
                                  name: e.target.value,
                                  uncertain: false,
                                  ...(p
                                    ? { productId: p.id, productName: p.name }
                                    : !item.productTypeEdited &&
                                        receipt.items.find((original) => original.id === item.id)
                                          ?.uncertain
                                      ? { productId: undefined, productName: e.target.value }
                                      : {}),
                                  ...(p
                                    ? { category: p.category, subcategory: p.subcategory }
                                    : {}),
                                });
                              }}
                            />
                          </label>
                          <div className="sp-edit-item-controls">
                            <label className="sp-field">
                              Quantity
                              <input
                                aria-label={`Item ${index + 1} quantity`}
                                inputMode="decimal"
                                value={item.quantityText}
                                onChange={(e) =>
                                  updateItem(item.id, { quantityText: e.target.value })
                                }
                              />
                            </label>
                            <label className="sp-field">
                              Amount, {currency}
                              <input
                                aria-label={`Item ${index + 1} amount`}
                                inputMode="decimal"
                                value={item.amountText}
                                onChange={(e) =>
                                  updateItem(item.id, { amountText: e.target.value })
                                }
                              />
                            </label>
                          </div>
                          <div className="sp-edit-item-controls">
                            <label className="sp-field">
                              Category
                              <select
                                aria-label={`Item ${index + 1} category`}
                                value={item.category}
                                onChange={(e) => {
                                  const c = e.target.value as CategoryId;
                                  updateItem(item.id, {
                                    category: c,
                                    subcategory: CATEGORIES[c].sub[0],
                                  });
                                }}
                              >
                                {Object.entries(CATEGORIES).map(([id, c]) => (
                                  <option key={id} value={id}>
                                    {c.name}
                                  </option>
                                ))}
                              </select>
                            </label>
                            <label className="sp-field">
                              Subcategory
                              <select
                                aria-label={`Item ${index + 1} subcategory`}
                                value={item.subcategory}
                                onChange={(e) =>
                                  updateItem(item.id, { subcategory: e.target.value })
                                }
                              >
                                {CATEGORIES[item.category].sub.map((s) => (
                                  <option key={s}>{s}</option>
                                ))}
                              </select>
                            </label>
                          </div>
                          {(receipt.items.find((original) => original.id === item.id)?.uncertain ||
                            item.uncertain) && (
                            <label className="sp-item-confirm">
                              <input
                                type="checkbox"
                                aria-label={`Item ${index + 1} checked`}
                                checked={!item.uncertain}
                                onChange={(e) =>
                                  updateItem(item.id, { uncertain: !e.target.checked })
                                }
                              />{' '}
                              Item checked
                            </label>
                          )}
                          <label className="sp-field">
                            As printed on the receipt
                            <input
                              aria-label={`Item ${index + 1} original text`}
                              value={item.rawName}
                              onChange={(e) => updateItem(item.id, { rawName: e.target.value })}
                            />
                          </label>
                          <label className="sp-field">
                            Product type in the catalog
                            <input
                              aria-label={`Item ${index + 1} product type`}
                              list="sp-product-types"
                              value={item.productName || ''}
                              onChange={(e) => {
                                const product = findProduct(e.target.value, catalog);
                                updateItem(item.id, {
                                  productName: e.target.value,
                                  productTypeEdited: true,
                                  uncertain: false,
                                  productId: product?.id,
                                  ...(product
                                    ? {
                                        category: product.category,
                                        subcategory: product.subcategory,
                                      }
                                    : {}),
                                });
                              }}
                            />
                          </label>
                        </div>
                      ))}
                    </div>
                    <div className="sp-edit-total">
                      <span>
                        Items total <strong>{money(draftSum, currency)}</strong>
                      </span>
                      <label className="sp-field">
                        Receipt total, {currency}
                        <input
                          aria-label="Receipt total"
                          inputMode="decimal"
                          value={totalText}
                          onChange={(e) => setTotalText(e.target.value)}
                        />
                      </label>
                    </div>
                    {total !== draftSum && (
                      <p className="sp-error-text">
                        Totals don't match: off by {money((total || 0) - draftSum, currency)}
                      </p>
                    )}
                  </>
                ) : (
                  <>
                    <div className="sp-receipt-meta">
                      <span className="sp-merchant-icon">
                        <ReceiptText size={25} />
                      </span>
                      <div>
                        <h3>{receipt.merchant}</h3>
                        <p>
                          {receipt.dateMissing
                            ? 'No date'
                            : `${dateLabel(receipt.date)} ${receipt.date.slice(0, 4)}`}
                          {receipt.time ? ` · ${receipt.time} on the receipt` : ' · no time'} ·{' '}
                          {receipt.currency}
                        </p>
                      </div>
                      <span
                        className={`sp-badge ${receipt.status === 'review' ? 'sp-badge-review' : ''}`}
                      >
                        {receipt.status === 'review' ? 'Review' : 'Saved'}
                      </span>
                    </div>
                    <div className="sp-detail-items">
                      {receipt.items.map((item) => (
                        <div
                          className={
                            item.uncertain ? 'sp-detail-item sp-uncertain' : 'sp-detail-item'
                          }
                          key={item.id}
                        >
                          <span className="sp-product-icon">
                            <ProductIcon id={item.productId} category={item.category} />
                          </span>
                          <div>
                            <strong>
                              {item.name}
                              {item.uncertain && <AlertCircle size={14} />}
                            </strong>
                            <small>{item.rawName}</small>
                            <span>
                              {item.quantity} {item.unit} ×{' '}
                              {money(Math.round(item.amount / item.quantity), receipt.currency)}{' '}
                              <i>·</i> {CATEGORIES[item.category].name}
                            </span>
                          </div>
                          <strong>{money(item.amount, receipt.currency)}</strong>
                        </div>
                      ))}
                    </div>
                    <div className="sp-detail-total">
                      <span>Total</span>
                      <strong>{money(receipt.total, receipt.currency)}</strong>
                    </div>
                    {Boolean(receipt.matchedIds?.length) && (
                      <div className="sp-match-notice">
                        <CheckCheck size={17} />
                        <span>Checked off on the crew list: {receipt.matchedIds!.length}</span>
                      </div>
                    )}
                  </>
                )}
              </div>
            </div>
          </>
        )}
        {!demo && receipt.issues?.length ? (
          <div className="sp-receipt-issues">
            {receipt.issues.map((issue, index) => (
              <p className="sp-error-text" key={index}>
                {issue}
              </p>
            ))}
          </div>
        ) : null}
        {!demo && receipt.suspectedDuplicate && (
          <label className="sp-duplicate-confirm">
            <input
              type="checkbox"
              checked={confirmDuplicate}
              onChange={(e) => setConfirmDuplicate(e.target.checked)}
            />
            This is a separate purchase, not the same receipt uploaded twice
          </label>
        )}
        {confirmDelete && (
          <div className="sp-review-callout">
            <div>
              <strong>Cancel this purchase?</strong>
              <p>It leaves the totals. The original entry and its reversal stay in the ledger.</p>
              <button className="sp-secondary" onClick={() => setConfirmDelete(false)}>
                Keep it
              </button>
              <button
                className="sp-secondary"
                disabled={saving}
                onClick={async () => {
                  setSaving(true);
                  try {
                    await onDelete?.();
                  } catch (e) {
                    setError(e instanceof Error ? e.message : "Couldn't cancel");
                  } finally {
                    setSaving(false);
                  }
                }}
              >
                Cancel purchase
              </button>
            </div>
          </div>
        )}
        {success && (
          <p role="status" className="sp-save-status">
            {success}
            {receipt.status === 'review'
              ? ' The receipt still needs a look and is not counted yet.'
              : ''}
          </p>
        )}
        {error && (
          <p ref={saveError} className="sp-error-text" role="alert">
            {error}
          </p>
        )}
      </div>
      <div className="sp-modal-actions">
        {!demo && onDelete && (
          <button
            className="sp-secondary"
            disabled={!online || saving}
            onClick={() => setConfirmDelete(true)}
          >
            Cancel purchase
          </button>
        )}
        {!demo && receipt.status === 'failed' && onRetry && (
          <button
            className="sp-primary"
            disabled={!online || saving}
            onClick={async () => {
              setSaving(true);
              try {
                await onRetry();
              } catch (e) {
                setError(e instanceof Error ? e.message : "Couldn't retry");
              } finally {
                setSaving(false);
              }
            }}
          >
            Scan again
          </button>
        )}
        {['queued', 'processing', 'failed'].includes(receipt.status) ? (
          <button className="sp-primary" onClick={onClose}>
            Got it
          </button>
        ) : editing ? (
          <>
            <button className="sp-secondary" disabled={saving} onClick={cancelEdit}>
              Cancel
            </button>
            <button
              className="sp-primary"
              disabled={
                saving ||
                !online ||
                (!allowDraft && Boolean(receipt.suspectedDuplicate && !confirmDuplicate))
              }
              onClick={() => void save()}
            >
              <Check size={17} /> Save changes
            </button>
          </>
        ) : (
          <>
            <button className="sp-secondary" disabled={!online} onClick={beginEdit}>
              <Pencil size={15} /> {receipt.status === 'review' ? 'Check items' : 'Fix'}
            </button>
            {receipt.status === 'saved' && (
              <button className="sp-primary" onClick={onRepeat}>
                <Plus size={16} /> Add to the list again
              </button>
            )}
          </>
        )}
      </div>
    </Modal>
  );
}

export function CatalogDialog({
  catalog,
  receipts,
  initialProduct,
  onClose,
  onAdd,
  onReceipt,
  onUpdate,
  rules = [],
  onRuleSave,
  onRuleDelete,
}: {
  rules?: CorrectionRule[];
  onRuleSave?: (rule: CorrectionRule) => Promise<void>;
  onRuleDelete?: (rule: CorrectionRule) => Promise<void>;
  catalog: Product[];
  receipts: Receipt[];
  initialProduct: string | null;
  onClose: () => void;
  onAdd: (p: Product) => void;
  onReceipt: (id: string) => void;
  onUpdate: (p: Product) => void;
}) {
  const [query, setQuery] = useState('');
  const [selected, setSelected] = useState(initialProduct);
  const p = catalog.find((i) => i.id === selected);
  const purchases = p
    ? receipts
        .filter((r) => r.status === 'saved')
        .flatMap((r) =>
          r.items.filter((i) => i.productId === p.id).map((item) => ({ receipt: r, item })),
        )
        .sort((a, b) => b.receipt.date.localeCompare(a.receipt.date))
    : [];
  const variants = [...new Set(purchases.map(({ item }) => item.rawName))];
  return (
    <Modal title={p ? p.name : 'Shopping catalog'} wide onClose={onClose}>
      <div className="sp-modal-body">
        {p ? (
          <>
            <button className="sp-text-link" onClick={() => setSelected(null)}>
              <ArrowLeft size={15} /> All items
            </button>
            <div className="sp-product-card">
              <span className="sp-product-icon">
                <ProductIcon id={p.id} category={p.category} />
              </span>
              <div>
                <span className="sp-kicker">PRODUCT TYPE</span>
                <h3>{p.name}</h3>
                <p>
                  {CATEGORIES[p.category].name} → {p.subcategory}
                </p>
              </div>
              <button className="sp-primary" onClick={() => onAdd(p)}>
                <Plus size={16} /> To the list
              </button>
            </div>
            <div className="sp-catalog-category">
              <label className="sp-field">
                Category for future purchases
                <select
                  value={p.category}
                  onChange={(e) => {
                    const category = e.target.value as CategoryId;
                    onUpdate({ ...p, category, subcategory: CATEGORIES[category].sub[0] });
                  }}
                >
                  {Object.entries(CATEGORIES).map(([id, c]) => (
                    <option key={id} value={id}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </label>
              <label className="sp-field">
                Subcategory
                <select
                  value={p.subcategory}
                  onChange={(e) => onUpdate({ ...p, subcategory: e.target.value })}
                >
                  {CATEGORIES[p.category].sub.map((s) => (
                    <option key={s}>{s}</option>
                  ))}
                </select>
              </label>
            </div>
            <p className="sp-fine-print">
              Catalog changes never rewrite history. Fix a single purchase in its receipt.
            </p>
            {onRuleSave && onRuleDelete && (
              <CorrectionRules
                rules={rules.filter((r) => r.productId === p.id)}
                catalog={catalog}
                onSave={onRuleSave}
                onDelete={onRuleDelete}
              />
            )}
            <div className="sp-variants">
              <h3>Seen on receipts</h3>
              {(variants.length ? variants : [p.variant]).map((v) => (
                <div key={v}>
                  <Package size={17} />
                  <span>{v}</span>
                  <CheckCircle2 size={15} />
                </div>
              ))}
            </div>
            <div className="sp-purchase-history">
              <div className="sp-section-line">
                <h3>Purchase history</h3>
                <span className="sp-muted">{purchases.length}</span>
              </div>
              {purchases.length ? (
                purchases.map(({ receipt, item }) => (
                  <button key={`${receipt.id}-${item.id}`} onClick={() => onReceipt(receipt.id)}>
                    <span>
                      <strong>{receipt.merchant}</strong>
                      <small>
                        {dateLabel(receipt.date)} · {item.quantity} {item.unit}
                      </small>
                    </span>
                    <span>{money(item.amount, receipt.currency)}</span>
                    <ChevronRight size={15} />
                  </button>
                ))
              ) : (
                <p className="sp-muted">
                  Only on the list so far. The first purchase shows up here after a receipt.
                </p>
              )}
            </div>
          </>
        ) : (
          <>
            <p className="sp-muted">
              Known items, names from packaging and your own entries, all in one place.
            </p>
            <label className="sp-search">
              <Search size={17} />
              <input
                autoFocus
                aria-label="Search the catalog"
                placeholder="For example, milk"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
              />
            </label>
            <div className="sp-catalog-grid">
              {catalog
                .filter((item) =>
                  [item.name, item.variant, ...item.aliases].some((s) =>
                    normalize(s).includes(normalize(query)),
                  ),
                )
                .map((item) => (
                  <button key={item.id} onClick={() => setSelected(item.id)}>
                    <span className="sp-product-icon">
                      <ProductIcon id={item.id} category={item.category} />
                    </span>
                    <span>
                      <strong>{item.name}</strong>
                      <small>{CATEGORIES[item.category].name}</small>
                    </span>
                    <ChevronRight size={16} />
                  </button>
                ))}
            </div>
            {!catalog.some((item) =>
              [item.name, item.variant, ...item.aliases].some((s) =>
                normalize(s).includes(normalize(query)),
              ),
            ) && <p className="sp-muted">Not here yet. Add it to the list and its card appears.</p>}
          </>
        )}
      </div>
    </Modal>
  );
}

function CorrectionRules({
  rules,
  catalog,
  onSave,
  onDelete,
}: {
  rules: CorrectionRule[];
  catalog: Product[];
  onSave: (rule: CorrectionRule) => Promise<void>;
  onDelete: (rule: CorrectionRule) => Promise<void>;
}) {
  const [draft, setDraft] = useState<CorrectionRule | null>(null);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  async function run(action: () => Promise<void>) {
    setBusy(true);
    setError('');
    try {
      await action();
      setDraft(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Couldn't change the rule");
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="sp-correction-rules">
      <h3>Remembered fixes</h3>
      <p className="sp-fine-print">For an exact line at one store. History never changes.</p>
      {error && (
        <p role="alert" className="sp-error-text">
          {error}
        </p>
      )}
      {rules.length === 0 && (
        <p className="sp-muted">Fix an item on a receipt and it is remembered here.</p>
      )}
      {rules.map((r) => (
        <div className="sp-correction-rule" key={r.id}>
          <strong>{r.merchant}</strong>
          <small>{r.originals.join(' / ')}</small>
          <p>
            {r.name} · {CATEGORIES[r.category].name} → {r.subcategory}
          </p>
          <button
            disabled={busy}
            className="sp-text-link"
            onClick={() => {
              setDraft(structuredClone(r));
              setError('');
            }}
          >
            Edit rule
          </button>
          <button
            disabled={busy}
            className="sp-text-link"
            onClick={() => void run(() => onDelete(r))}
          >
            Forget this fix
          </button>
          {draft?.id === r.id && (
            <div className="sp-rule-editor">
              <label className="sp-field">
                Store
                <input
                  value={draft.merchant}
                  onChange={(e) => setDraft({ ...draft, merchant: e.target.value })}
                />
              </label>
              <label className="sp-field">
                Original lines (one per line)
                <textarea
                  rows={3}
                  value={draft.originals.join('\n')}
                  onChange={(e) => setDraft({ ...draft, originals: e.target.value.split('\n') })}
                />
              </label>
              <label className="sp-field">
                Display name
                <input
                  value={draft.name}
                  onChange={(e) => setDraft({ ...draft, name: e.target.value })}
                />
              </label>
              <label className="sp-field">
                Product type
                <input
                  list="sp-rule-products"
                  value={draft.productName}
                  onChange={(e) => setDraft({ ...draft, productName: e.target.value })}
                />
              </label>
              <datalist id="sp-rule-products">
                {catalog.map((p) => (
                  <option key={p.id} value={p.name} />
                ))}
              </datalist>
              <label className="sp-field">
                Category
                <select
                  value={draft.category}
                  onChange={(e) => {
                    const category = e.target.value as CategoryId;
                    setDraft({ ...draft, category, subcategory: CATEGORIES[category].sub[0] });
                  }}
                >
                  {Object.entries(CATEGORIES).map(([id, c]) => (
                    <option key={id} value={id}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </label>
              <label className="sp-field">
                Subcategory
                <select
                  value={draft.subcategory}
                  onChange={(e) => setDraft({ ...draft, subcategory: e.target.value })}
                >
                  {CATEGORIES[draft.category].sub.map((s) => (
                    <option key={s}>{s}</option>
                  ))}
                </select>
              </label>
              <div className="sp-rule-actions">
                <button className="sp-secondary" disabled={busy} onClick={() => setDraft(null)}>
                  Cancel
                </button>
                <button
                  className="sp-primary"
                  disabled={busy}
                  onClick={() =>
                    void run(() =>
                      onSave({
                        ...draft,
                        originals: draft.originals.map((s) => s.trim()).filter(Boolean),
                      }),
                    )
                  }
                >
                  {busy ? 'Saving…' : 'Save rule'}
                </button>
              </div>
            </div>
          )}
        </div>
      ))}
    </section>
  );
}
