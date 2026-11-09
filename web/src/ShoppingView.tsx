import { useEffect, useRef, useState } from 'react';
import {
  Plus,
  ArrowRight,
  ArrowLeft,
  Camera,
  BookOpen,
  ListChecks,
  CheckCheck,
  CheckCircle2,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Clock3,
  ReceiptText,
  RotateCcw,
  Search,
  Sparkles,
  ShieldCheck,
  Wifi,
  WifiOff,
  X,
  AlertCircle,
  BarChart3,
  LoaderCircle,
  UsersRound,
} from 'lucide-react';
import {
  CATEGORIES,
  currencyScale,
  dateLabel,
  findProduct,
  money,
  monthLabel,
  normalize,
  parseList,
  suggestions,
  summarize,
  lastSixMonths,
  shoppingMonths,
  type CategoryId,
  type ListItem,
  type Product,
  type Receipt,
} from './shoppingModels';
import {
  ProductIcon,
  Modal,
  ReceiptRow,
  Donut,
  DailyChart,
  ReceiptDialog,
  CatalogDialog,
} from './ShoppingComponents';
import { useShopping } from './shoppingStore';
import { ShoppingCamera } from './ShoppingCamera';
import type { AppRoute } from './routes';
import './shopping.css';

export default function ShoppingView({
  route,
  onNavigate,
}: {
  route: AppRoute;
  onNavigate: (route: AppRoute) => void;
}) {
  const store = useShopping();
  const { list, catalog, receipts, purchases, today } = store.data;
  const { setList, setCatalog } = store;
  const tab = (['list', 'history', 'analytics'].includes(route.tab || '') ? route.tab : 'list') as
    'list' | 'history' | 'analytics';
  const setTab = (tab: 'list' | 'history' | 'analytics') =>
    onNavigate({ ...route, page: 'shopping', tab });
  const currentMonth = today.slice(0, 7);
  const months = lastSixMonths(today);
  const storedPeriods = shoppingMonths(receipts, today);
  const [name, setName] = useState('');
  const [quantity, setQuantity] = useState('');
  const [focused, setFocused] = useState(false);
  const [activeSuggestion, setActiveSuggestion] = useState(-1);
  const [bulk, setBulk] = useState(false);
  const [bulkText, setBulkText] = useState('');
  const [scan, setScan] = useState(false);
  const [selected, setSelected] = useState<string | null>(null);
  const [catalogOpen, setCatalogOpen] = useState(false);
  const [catalogProduct, setCatalogProduct] = useState<string | null>(null);
  const [category, setCategory] = useState<CategoryId | null>(null);
  const [notice, setNotice] = useState<{ text: string; receiptId?: string } | null>(null);
  const [selectedHistoryMonth, setHistoryMonth] = useState<string | null>(null);
  const historyMonth = selectedHistoryMonth ?? currentMonth;
  const [historyQuery, setHistoryQuery] = useState('');
  const [historyCurrency, setHistoryCurrency] = useState('all');
  const [selectedMonth, setMonth] = useState<string | null>(null);
  const month = selectedMonth ?? currentMonth;
  const periods = [
    ...new Set([
      ...storedPeriods,
      ...[selectedHistoryMonth, selectedMonth].filter((period): period is string =>
        Boolean(period && period !== 'all'),
      ),
    ]),
  ].sort();
  const [currency, setCurrency] = useState('EUR');
  const nameInput = useRef<HTMLInputElement>(null);
  const pending = list.filter((i) => !i.done);
  const completed = list.filter((i) => i.done);
  const offline = !store.online;
  const currentSummary = summarize(purchases, currentMonth, 'EUR');
  const summary = summarize(purchases, month, currency);
  const availableCurrencies = [
    ...new Set([
      'EUR',
      ...purchases.filter((r) => r.date.startsWith(month)).map((r) => r.currency),
    ]),
  ];
  const options = suggestions(name, catalog);
  const showSuggestions = focused && name.trim().length > 0 && options.length > 0;
  const currencies = [...new Set(purchases.map((r) => r.currency))];
  const historyReceipts = receipts
    .filter(
      (r) =>
        (historyMonth === 'all' || r.date.startsWith(historyMonth)) &&
        (historyCurrency === 'all' || r.currency === historyCurrency) &&
        (!historyQuery.trim() ||
          normalize(
            [r.merchant, ...r.items.flatMap((i) => [i.name, i.rawName])].join(' '),
          ).includes(normalize(historyQuery))),
    )
    .sort((a, b) => b.date.localeCompare(a.date));
  const receiptDates = [...new Set(historyReceipts.map((r) => r.date))];
  const lastReceipt = [...receipts]
    .filter((r) => r.status === 'saved')
    .sort((a, b) => b.date.localeCompare(a.date))[0];
  const frequent = [...catalog]
    .filter((p) => purchases.some((r) => r.items.some((i) => i.productId === p.id)))
    .sort(
      (a, b) =>
        purchases.flatMap((r) => r.items).filter((i) => i.productId === b.id).length -
        purchases.flatMap((r) => r.items).filter((i) => i.productId === a.id).length,
    )
    .slice(0, 6);
  const selectedReceipt = receipts.find((r) => r.id === selected);
  const reviews = receipts.filter((r) => r.status === 'review');
  const otherCurrencySummary = [
    ...new Set(
      purchases
        .filter((r) => r.date.startsWith(currentMonth) && r.currency !== 'EUR')
        .map((r) => r.currency),
    ),
  ]
    .map((c) => money(summarize(purchases, currentMonth, c).total, c))
    .join(' · ');
  useEffect(() => {
    if (selected) void store.loadPhotos(selected);
  }, [selected, selectedReceipt?.status, selectedReceipt?.revision, store.online]);
  useEffect(() => {
    if (!notice) return;
    const id = setTimeout(() => setNotice(null), 6500);
    return () => clearTimeout(id);
  }, [notice]);
  async function addItems(items: ListItem[]) {
    try {
      await store.addItems(items);
      setNotice({
        text: offline
          ? 'List saved on this device. It syncs when you are back online.'
          : 'Added to the list',
      });
    } catch (e) {
      setNotice({ text: e instanceof Error ? e.message : "Couldn't add those" });
    }
  }
  function addOne(p?: Product) {
    const text = p?.name || name.trim();
    if (!text) return;
    void addItems(
      parseList(text, catalog).map((i) => ({ ...i, quantity: quantity.trim() || i.quantity })),
    );
    setName('');
    setQuantity('');
    setActiveSuggestion(-1);
    nameInput.current?.focus();
  }
  function addProduct(p: Product) {
    void addItems([
      {
        id: crypto.randomUUID(),
        productId: p.id,
        name: p.name,
        quantity: '1',
        category: p.category,
        done: false,
        author: 'You',
      },
    ]);
  }
  function openReceipt(id: string) {
    setCatalogOpen(false);
    setCategory(null);
    setSelected(id);
  }
  function repeatReceipt(receipt: Receipt) {
    void addItems(
      receipt.items.map((i) => ({
        id: crypto.randomUUID(),
        productId: i.productId,
        name: catalog.find((p) => p.id === i.productId)?.name || i.name,
        quantity: `${i.quantity} ${i.unit}`,
        category: i.category,
        done: false,
        author: 'You',
      })),
    );
    setSelected(null);
    setTab('list');
  }
  async function upload(photos: string[]) {
    await store.uploadPhotos(photos);
    setScan(false);
    setNotice({
      text: offline
        ? 'Photo saved on this device. It uploads when you are back online.'
        : 'Receipt queued. Scanning in the background.',
    });
  }
  async function importText() {
    try {
      await store.importText(bulkText);
      setBulk(false);
      setNotice({ text: "List saved. We'll read it when you are back online." });
    } catch (e) {
      setNotice({ text: e instanceof Error ? e.message : "Couldn't save the list" });
    }
  }
  if (!store.loaded)
    return (
      <div className="sp-app sp-live">
        <p className="loading">
          <LoaderCircle size={20} /> Loading shopping…
        </p>
      </div>
    );
  return (
    <div
      className="sp-app sp-live"
      inert={Boolean(bulk || scan || selectedReceipt || catalogOpen || category)}
    >
      <div className="sp-page-heading">
        <div>
          <div className="sp-eyebrow">TEAMWORK</div>
          <h1>
            Shopping<span className="sp-title-dot">.</span>
          </h1>
          <p>From “we need” to “where did the money go”.</p>
        </div>
        <button
          className="sp-primary sp-camera-main"
          onClick={() => {
            setScan(true);
          }}
        >
          <Camera size={19} /> Scan a receipt
        </button>
      </div>
      <div className="sp-toolbar">
        <nav className="sp-tabs" aria-label="Shopping sections">
          {(
            [
              ['list', 'List', ListChecks],
              ['history', 'History', Clock3],
              ['analytics', 'Spending', BarChart3],
            ] as const
          ).map(([id, label, Icon]) => (
            <button
              key={id}
              className={tab === id ? 'active' : ''}
              aria-current={tab === id ? 'page' : undefined}
              onClick={() => setTab(id)}
            >
              <Icon size={17} />
              {label}
              {id === 'list' && <span className="sp-tab-count">{pending.length}</span>}
            </button>
          ))}
        </nav>
        <span className="sp-sync-status">
          {store.syncing ? (
            <LoaderCircle size={13} className="sp-spin" />
          ) : (
            <CheckCheck size={13} />
          )}{' '}
          {store.pending.length ? `Waiting to sync: ${store.pending.length}` : 'Shared crew list'}
        </span>
      </div>
      {offline && (
        <div className="sp-banner sp-offline" role="status">
          <WifiOff size={19} />
          <div>
            <strong>Offline</strong>
            <span>
              The list and photos are saved on this device and sync while the app is open once you
              are online.
            </span>
          </div>
          <button onClick={() => void store.sync()}>
            <RotateCcw size={15} /> Retry
          </button>
        </div>
      )}
      {store.error && (
        <div className="sp-banner sp-review-banner" role="alert">
          <AlertCircle size={19} />
          <span>{store.error}</span>
          <button onClick={() => void store.sync()}>Retry</button>
        </div>
      )}
      {store.pending
        .filter((p) => p.blocked)
        .map((p) => (
          <div className="sp-banner sp-review-banner" key={p.id}>
            <AlertCircle size={19} />
            <span>
              <strong>This change needs attention</strong>
              {p.blocked}
            </span>
            <button onClick={() => void store.retryPending(p.id)}>Apply my change</button>
            <button onClick={() => void store.discardPending(p.id)}>Cancel</button>
          </div>
        ))}
      {store.data.imports.map((job) => (
        <div className="sp-banner sp-offline" key={job.id}>
          <ListChecks size={19} />
          <span>
            <strong>
              {job.status === 'failed' ? "Couldn't read the list" : 'Reading your list'}
            </strong>
            {job.text}
            {job.error && <small>{job.error}</small>}
          </span>
          {job.status === 'failed' && (
            <button
              onClick={() => {
                setBulkText(job.text);
                setBulk(true);
              }}
            >
              Edit text
            </button>
          )}
        </div>
      ))}
      {reviews.length > 0 && (
        <button className="sp-banner sp-review-banner" onClick={() => openReceipt(reviews[0].id)}>
          <AlertCircle size={19} />
          <span>
            <strong>Receipts to check · {reviews.length}</strong>Until the total is fixed, it is not
            counted.
          </span>
          <ArrowRight size={18} />
        </button>
      )}
      {tab === 'list' && (
        <div className="sp-list-layout">
          <section className="sp-card sp-shopping-list" aria-labelledby="sp-list-title">
            <div className="sp-card-heading">
              <div>
                <h2 id="sp-list-title">Shopping for the crew</h2>
                <p>One list for the whole crew</p>
              </div>
              <div className="sp-crew-avatars" aria-label="Shared crew list">
                <span>
                  <UsersRound size={16} />
                </span>
              </div>
            </div>
            <div className="sp-add-area">
              <form
                className="sp-add-form"
                onSubmit={(event) => {
                  event.preventDefault();
                  addOne(
                    activeSuggestion >= 0 && showSuggestions
                      ? options[activeSuggestion]
                      : undefined,
                  );
                }}
              >
                <Plus size={20} className="sp-add-icon" />
                <input
                  ref={nameInput}
                  role="combobox"
                  aria-label="What do we need?"
                  aria-expanded={showSuggestions}
                  aria-autocomplete="list"
                  aria-controls="sp-suggestions"
                  aria-activedescendant={
                    showSuggestions && activeSuggestion >= 0
                      ? `sp-suggestion-${activeSuggestion}`
                      : undefined
                  }
                  placeholder="What do we need?"
                  value={name}
                  onFocus={() => setFocused(true)}
                  onBlur={() => setFocused(false)}
                  onChange={(event) => {
                    setName(event.target.value);
                    setActiveSuggestion(-1);
                  }}
                  onKeyDown={(event) => {
                    if (event.key === 'ArrowDown' && showSuggestions) {
                      event.preventDefault();
                      setActiveSuggestion((old) => (old + 1) % options.length);
                    }
                    if (event.key === 'ArrowUp' && showSuggestions) {
                      event.preventDefault();
                      setActiveSuggestion((old) => (old <= 0 ? options.length - 1 : old - 1));
                    }
                    if (event.key === 'Escape') {
                      setFocused(false);
                      setActiveSuggestion(-1);
                    }
                  }}
                />
                <input
                  className="sp-quantity-input"
                  aria-label="Quantity"
                  placeholder="Qty"
                  value={quantity}
                  onChange={(event) => setQuantity(event.target.value)}
                  maxLength={24}
                />
                <button
                  type="submit"
                  className="sp-add-submit"
                  aria-label="Add to the list"
                  disabled={!name.trim()}
                >
                  <ArrowRight size={19} />
                </button>
              </form>
              {showSuggestions && (
                <ul
                  className="sp-autocomplete"
                  id="sp-suggestions"
                  role="listbox"
                  aria-label="Catalog suggestions"
                >
                  {options.map((p, index) => (
                    <li
                      key={p.id}
                      id={`sp-suggestion-${index}`}
                      role="option"
                      aria-selected={activeSuggestion === index}
                    >
                      <button
                        className={activeSuggestion === index ? 'selected' : ''}
                        onMouseDown={(event) => event.preventDefault()}
                        onClick={() => addOne(p)}
                      >
                        <span className="sp-product-icon">
                          <ProductIcon id={p.id} category={p.category} />
                        </span>
                        <span>
                          <strong>{p.name}</strong>
                          <small>{p.variant}</small>
                        </span>
                        <Plus size={16} />
                      </button>
                    </li>
                  ))}
                </ul>
              )}
              <div className="sp-input-tools">
                <button
                  onClick={() => {
                    setBulkText('');
                    setBulk(true);
                  }}
                >
                  <ListChecks size={14} /> Paste a whole list
                </button>
                <button
                  onClick={() => {
                    setCatalogProduct(findProduct(name, catalog)?.id || null);
                    setCatalogOpen(true);
                  }}
                >
                  <BookOpen size={14} /> Catalog
                </button>
              </div>
            </div>
            {(Object.keys(CATEGORIES) as CategoryId[]).map((id) => {
              const items = pending.filter((i) => i.category === id);
              if (!items.length) return null;
              return (
                <div className="sp-list-group" key={id}>
                  <div className="sp-group-label">
                    {CATEGORIES[id].name}
                    <span>{items.length}</span>
                  </div>
                  {items.map((item) => (
                    <div className="sp-list-row" key={item.id}>
                      <input
                        className="sp-checkbox"
                        type="checkbox"
                        aria-label={`Check off: ${item.name}`}
                        checked={false}
                        onChange={() =>
                          setList((all) =>
                            all.map((i) => (i.id === item.id ? { ...i, done: true } : i)),
                          )
                        }
                      />
                      <span className="sp-product-icon">
                        <ProductIcon id={item.productId} category={item.category} />
                      </span>
                      <div className="sp-list-item-name">
                        <strong>{item.name}</strong>
                        <small>
                          {item.author === 'You' ? 'Added by you' : `Added by ${item.author}`}
                        </small>
                      </div>
                      <span className="sp-item-quantity">{item.quantity}</span>
                      <button
                        className="sp-icon-button sp-remove-item"
                        aria-label={`Remove: ${item.name}`}
                        onClick={() => setList((all) => all.filter((i) => i.id !== item.id))}
                      >
                        <X size={16} />
                      </button>
                    </div>
                  ))}
                </div>
              );
            })}
            {!pending.length && (
              <div className="sp-list-finished">
                <CheckCheck size={30} />
                <h3>{list.length ? 'All bought' : 'What do we need?'}</h3>
                <p>
                  {list.length
                    ? 'Nothing left. Add new things above.'
                    : 'Add the first item above, or just snap a receipt.'}
                </p>
              </div>
            )}
            {completed.length > 0 && (
              <details className="sp-completed" open>
                <summary>
                  <CheckCheck size={16} /> Bought <span>{completed.length}</span>
                  <ChevronDown size={15} />
                </summary>
                <div>
                  {completed.map((item) => (
                    <div className="sp-list-row sp-done" key={item.id}>
                      <input
                        className="sp-checkbox"
                        type="checkbox"
                        aria-label={`Back to the list: ${item.name}`}
                        checked
                        onChange={() =>
                          setList((all) =>
                            all.map((i) => (i.id === item.id ? { ...i, done: false } : i)),
                          )
                        }
                      />
                      <span className="sp-list-item-name">
                        <strong>{item.name}</strong>
                      </span>
                      <span className="sp-item-quantity">{item.quantity}</span>
                      <button
                        className="sp-icon-button"
                        aria-label={`Undo check-off: ${item.name}`}
                        onClick={() =>
                          setList((all) =>
                            all.map((i) => (i.id === item.id ? { ...i, done: false } : i)),
                          )
                        }
                      >
                        <RotateCcw size={15} />
                      </button>
                    </div>
                  ))}
                </div>
              </details>
            )}
            <div className="sp-list-foot">
              <ShieldCheck size={15} />
              <span>
                {offline ? 'Changes waiting to sync' : 'Everyone in your crew sees this list'}
              </span>
            </div>
          </section>

          <aside className="sp-sidebar">
            <section className="sp-month-peek">
              <span className="sp-kicker">
                {monthLabel(currentMonth).toLocaleUpperCase('en-US')}
              </span>
              <div className="sp-month-total">{money(currentSummary.total)}</div>
              <p>{currentSummary.receipts.length} receipts · sorted by category</p>
              <div className="sp-mini-stack" aria-label="Spending this month">
                {currentSummary.byCategory.map((c) => (
                  <span
                    key={c.id}
                    style={{
                      width: `${(Math.abs(c.amount) / Math.max(currentSummary.turnover, 1)) * 100}%`,
                      background: c.color,
                    }}
                    title={`${c.name}: ${money(c.amount)}`}
                  />
                ))}
              </div>
              <div className="sp-mini-legend">
                {currentSummary.byCategory.slice(0, 3).map((c) => (
                  <span key={c.id}>
                    <i style={{ background: c.color }} />
                    {c.name}
                  </span>
                ))}
              </div>
              <button
                className="sp-text-link"
                onClick={() => {
                  setTab('analytics');
                  setMonth(currentMonth);
                  setCurrency('EUR');
                }}
              >
                See spending <ArrowRight size={15} />
              </button>
              {otherCurrencySummary && (
                <span className="sp-separate-currency">
                  Other currencies: {otherCurrencySummary}
                </span>
              )}
            </section>
            <section className="sp-frequent">
              <div className="sp-section-line">
                <h3>You usually buy</h3>
                <Clock3 size={16} />
              </div>
              <p>Faster than typing it again</p>
              <div className="sp-quick-products">
                {frequent.map((p) => (
                  <button key={p.id} onClick={() => addProduct(p)}>
                    <ProductIcon id={p.id} category={p.category} />
                    <span>{p.name}</span>
                    <Plus size={13} />
                  </button>
                ))}
              </div>
            </section>
            {lastReceipt && (
              <section className="sp-last-receipt">
                <div className="sp-section-line">
                  <span className="sp-kicker">LAST BOUGHT</span>
                  <CheckCircle2 size={16} />
                </div>
                <button
                  className="sp-last-receipt-link"
                  onClick={() => openReceipt(lastReceipt.id)}
                >
                  <span className="sp-merchant-icon">
                    <ReceiptText size={24} />
                  </span>
                  <span>
                    <strong>{lastReceipt.merchant}</strong>
                    <small>
                      {dateLabel(lastReceipt.date)} · {lastReceipt.items.length} items
                    </small>
                  </span>
                  <strong>{money(lastReceipt.total, lastReceipt.currency)}</strong>
                </button>
                <button className="sp-text-link" onClick={() => repeatReceipt(lastReceipt)}>
                  Buy again <RotateCcw size={14} />
                </button>
              </section>
            )}
            <div className="sp-quiet-tip">
              <Camera size={20} />
              <p>
                Don't tick things off one by one.
                <br />
                <strong>Snap the receipt and the list sorts itself out.</strong>
              </p>
            </div>
          </aside>
        </div>
      )}

      {tab === 'history' && (
        <section className="sp-history" aria-label="Purchase history">
          <div className="sp-section-heading">
            <div>
              <h2>Everything you bought</h2>
              <p>Receipts, items and prices, nothing to remember.</p>
            </div>
            <span className="sp-record-count">{historyReceipts.length} receipts</span>
          </div>
          <div className="sp-history-filters">
            <label className="sp-search">
              <Search size={18} />
              <input
                aria-label="Search stores or items"
                placeholder="Store, item or receipt text"
                value={historyQuery}
                onChange={(e) => setHistoryQuery(e.target.value)}
              />
              {historyQuery && (
                <button
                  className="sp-icon-button"
                  aria-label="Clear search"
                  onClick={() => setHistoryQuery('')}
                >
                  <X size={16} />
                </button>
              )}
            </label>
            <select
              aria-label="History period"
              value={historyMonth}
              onChange={(e) => setHistoryMonth(e.target.value)}
            >
              <option value="all">All time</option>
              {[...periods].reverse().map((m) => (
                <option key={m} value={m}>
                  {monthLabel(m)}
                </option>
              ))}
            </select>
            <select
              aria-label="History currency"
              value={historyCurrency}
              onChange={(e) => setHistoryCurrency(e.target.value)}
            >
              <option value="all">All currencies</option>
              {currencies.map((c) => (
                <option key={c}>{c}</option>
              ))}
            </select>
          </div>
          {receiptDates.map((date) => (
            <div className="sp-history-day" key={date}>
              <div className="sp-history-date">
                <h3>
                  {date === today
                    ? `Today, ${dateLabel(today)}`
                    : `${dateLabel(date)}${date.slice(0, 4) !== today.slice(0, 4) ? ', ' + date.slice(0, 4) : ''}`}
                </h3>
                <span>
                  {[
                    ...new Set(
                      historyReceipts
                        .filter((r) => r.date === date && r.status === 'saved')
                        .map((r) => r.currency),
                    ),
                  ]
                    .map((c) =>
                      money(
                        historyReceipts
                          .filter(
                            (r) => r.date === date && r.status === 'saved' && r.currency === c,
                          )
                          .reduce((sum, r) => sum + r.total, 0),
                        c,
                      ),
                    )
                    .join(' · ')}
                </span>
              </div>
              <div className="sp-card sp-receipt-list">
                {historyReceipts
                  .filter((r) => r.date === date)
                  .map((r) => (
                    <ReceiptRow key={r.id} receipt={r} onClick={() => openReceipt(r.id)} />
                  ))}
              </div>
            </div>
          ))}
          {!historyReceipts.length && (
            <div className="sp-search-empty">
              <Search size={28} />
              <h3>Nothing like that</h3>
              <p>Try another item or pick “All time”.</p>
              <button
                className="sp-secondary"
                onClick={() => {
                  setHistoryQuery('');
                  setHistoryMonth('all');
                  setHistoryCurrency('all');
                }}
              >
                Reset filters
              </button>
            </div>
          )}
        </section>
      )}

      {tab === 'analytics' && (
        <section className="sp-analytics" aria-label="Spending">
          <div className="sp-section-heading">
            <div>
              <h2>Where the money goes</h2>
              <p>Saved receipts only. No guessing, no mixed currencies.</p>
            </div>
            <div className="sp-period-controls">
              <button
                className="sp-icon-button"
                aria-label="Previous month"
                disabled={periods.indexOf(month) <= 0}
                onClick={() => {
                  const next = periods[periods.indexOf(month) - 1];
                  if (!next) return;
                  setMonth(next);
                  if (!receipts.some((r) => r.date.startsWith(next) && r.currency === currency))
                    setCurrency('EUR');
                }}
              >
                <ChevronLeft size={19} />
              </button>
              <select
                aria-label="Period"
                className="sp-month-select"
                value={month}
                onChange={(e) => {
                  setMonth(e.target.value);
                  if (
                    !purchases.some(
                      (r) => r.date.startsWith(e.target.value) && r.currency === currency,
                    )
                  )
                    setCurrency('EUR');
                }}
              >
                {[...periods].reverse().map((m) => (
                  <option key={m} value={m}>
                    {monthLabel(m)}
                  </option>
                ))}
              </select>
              <button
                className="sp-icon-button"
                aria-label="Next month"
                disabled={periods.indexOf(month) >= periods.length - 1}
                onClick={() => {
                  const next = periods[periods.indexOf(month) + 1];
                  if (!next) return;
                  setMonth(next);
                  if (!receipts.some((r) => r.date.startsWith(next) && r.currency === currency))
                    setCurrency('EUR');
                }}
              >
                <ChevronRight size={19} />
              </button>
              <select
                aria-label="Currency"
                value={currency}
                onChange={(e) => setCurrency(e.target.value)}
              >
                {availableCurrencies.map((c) => (
                  <option key={c}>{c}</option>
                ))}
              </select>
            </div>
          </div>
          <div className="sp-analytics-stats">
            <div>
              <span>This month</span>
              <strong data-testid="analytics-total">{money(summary.total, currency)}</strong>
              <small>{summary.receipts.length} saved receipts</small>
            </div>
            <div>
              <span>Average receipt</span>
              <strong>
                {money(
                  summary.receipts.length ? Math.round(summary.total / summary.receipts.length) : 0,
                  currency,
                )}
              </strong>
              <small>In this currency</small>
            </div>
            <div>
              <span>Top category</span>
              <strong className="sp-stat-category">{summary.byCategory[0]?.name || '—'}</strong>
              <small>
                {summary.byCategory[0]
                  ? `${Math.round((summary.byCategory[0].amount / summary.total) * 100)}% of all spending`
                  : 'No spending yet'}
              </small>
            </div>
          </div>
          <div className="sp-analytics-grid">
            <section className="sp-card sp-category-chart">
              <div className="sp-card-heading">
                <div>
                  <h3>By category</h3>
                  <p>Tap a category to see its purchases</p>
                </div>
                <span className="sp-icon-tile">
                  <BarChart3 size={18} />
                </span>
              </div>
              <div className="sp-donut-layout">
                <Donut summary={summary} currency={currency} onSelect={setCategory} />
                <div className="sp-category-legend">
                  {summary.byCategory.map((c) => (
                    <button key={c.id} onClick={() => setCategory(c.id)}>
                      <i style={{ background: c.color }} />
                      <span>
                        {c.name}
                        <small>
                          {Math.round((Math.abs(c.amount) / Math.max(summary.turnover, 1)) * 100)}%
                        </small>
                      </span>
                      <strong>{money(c.amount, currency)}</strong>
                      <ChevronRight size={14} />
                    </button>
                  ))}
                </div>
              </div>
              <p className="sp-chart-caption">
                Purchase ledger · {monthLabel(month)} · {currency}
                {summary.hasRefunds ? ' · refunds count as negative' : ''}
              </p>
            </section>
            <section className="sp-card sp-six-months">
              <div className="sp-card-heading">
                <div>
                  <h3>Last six months</h3>
                  <p>Spending by month, {currency}</p>
                </div>
              </div>
              <div
                className="sp-month-bars"
                role="group"
                aria-label={`Six months of spending in ${currency}`}
              >
                {months.map((m) => {
                  const amount = summarize(purchases, m, currency).total;
                  const max = Math.max(
                    ...months.map((entry) => Math.abs(summarize(purchases, entry, currency).total)),
                    1,
                  );
                  return (
                    <button
                      key={m}
                      className={month === m ? 'selected' : ''}
                      aria-label={`${monthLabel(m)}: ${money(amount, currency)}`}
                      onClick={() => {
                        setMonth(m);
                        if (
                          !receipts.some(
                            (r) =>
                              r.status === 'saved' &&
                              r.date.startsWith(m) &&
                              r.currency === currency,
                          )
                        )
                          setCurrency('EUR');
                      }}
                    >
                      <span className="sp-bar-amount">
                        {Math.round(amount / currencyScale(currency))}
                      </span>
                      <span className="sp-bar-track">
                        <i style={{ height: `${(Math.abs(amount) / max) * 100}%` }} />
                      </span>
                      <span className="sp-bar-month">{monthLabel(m, true).replace('.', '')}</span>
                    </button>
                  );
                })}
              </div>
              <p className="sp-chart-caption">
                Purchase ledger · {monthLabel(months[0], true)} – {monthLabel(months[5], true)} ·
                amounts in {currency}
              </p>
            </section>
            <section className="sp-card sp-daily-chart">
              <div className="sp-card-heading">
                <div>
                  <h3>Through the month</h3>
                  <p>Which days and how much</p>
                </div>
                <span className="sp-muted">{currency}</span>
              </div>
              <DailyChart days={summary.days} currency={currency} />
              <p className="sp-chart-caption">
                Purchase ledger · {monthLabel(month)} · each bar is one day
              </p>
            </section>
          </div>
          {(currency !== 'EUR' || availableCurrencies.length > 1) && (
            <div className="sp-currency-note">
              <ShieldCheck size={17} />
              <p>
                Currencies are counted separately.{' '}
                {currency === 'EUR'
                  ? 'Purchases in other currencies are not in the EUR total. Pick a currency above.'
                  : 'Only spending in this currency, no conversion.'}
              </p>
            </div>
          )}
        </section>
      )}

      {bulk && (
        <Modal title="Paste a list" onClose={() => setBulk(false)}>
          <div className="sp-modal-body">
            <p className="sp-muted">
              Write it however you like: commas, new lines or a normal message. We'll pick out items
              and quantities.
            </p>
            <label className="sp-field">
              What do we need?
              <textarea
                value={bulkText}
                onChange={(e) => setBulkText(e.target.value)}
                rows={7}
                placeholder="Oat milk 2, cat food and something for movie night"
              />
            </label>
          </div>
          <div className="sp-modal-actions">
            <button className="sp-secondary" onClick={() => setBulk(false)}>
              Cancel
            </button>
            <button
              className="sp-primary"
              disabled={!bulkText.trim()}
              onClick={() => void importText()}
            >
              Add to the list
            </button>
          </div>
        </Modal>
      )}
      {scan && (
        <ShoppingCamera offline={offline} onClose={() => setScan(false)} onSubmit={upload} />
      )}
      {selectedReceipt && (
        <ReceiptDialog
          key={selectedReceipt.id + ':' + selectedReceipt.status}
          demo={false}
          today={today}
          online={store.online}
          receipt={selectedReceipt}
          catalog={catalog}
          onClose={() => setSelected(null)}
          onSave={async (receipt, confirm, allowIncomplete) => {
            const saved = await store.editReceipt(receipt, confirm, allowIncomplete);
            setNotice({
              text:
                saved?.status === 'review'
                  ? 'Fixes saved. The receipt still needs a look.'
                  : 'Fix saved in the purchase ledger',
            });
          }}
          onRepeat={() => repeatReceipt(selectedReceipt)}
          onRetry={() => store.retryReceipt(selectedReceipt)}
          onDelete={async () => {
            await store.deleteReceipt(selectedReceipt);
            setSelected(null);
            setNotice({ text: 'Purchase cancelled. The ledger keeps the reversal.' });
          }}
        />
      )}
      {catalogOpen && (
        <CatalogDialog
          catalog={catalog}
          rules={store.data.rules || []}
          onRuleSave={store.saveRule}
          onRuleDelete={store.deleteRule}
          receipts={purchases}
          initialProduct={catalogProduct}
          onClose={() => setCatalogOpen(false)}
          onAdd={addProduct}
          onReceipt={openReceipt}
          onUpdate={(updated) =>
            setCatalog((all) => all.map((p) => (p.id === updated.id ? updated : p)))
          }
        />
      )}
      {category && (
        <Modal title={CATEGORIES[category].name} wide onClose={() => setCategory(null)}>
          <div className="sp-modal-body">
            <div className="sp-breakdown-total">
              <span>{monthLabel(month)}</span>
              <strong>
                {money(summary.byCategory.find((c) => c.id === category)?.amount || 0, currency)}
              </strong>
            </div>
            {CATEGORIES[category].sub.map((sub) => {
              const entries = summary.receipts.flatMap((r) =>
                r.items
                  .filter((i) => i.category === category && i.subcategory === sub)
                  .map((item) => ({ receipt: r, item })),
              );
              if (!entries.length) return null;
              return (
                <section className="sp-subcategory" key={sub}>
                  <div className="sp-section-line">
                    <h3>{sub}</h3>
                    <strong>
                      {money(
                        entries.reduce((sum, e) => sum + e.item.amount, 0),
                        currency,
                      )}
                    </strong>
                  </div>
                  {entries.map(({ receipt, item }) => (
                    <button key={receipt.id + item.id} onClick={() => openReceipt(receipt.id)}>
                      <span>
                        <strong>{item.name}</strong>
                        <small>
                          {item.rawName} · {receipt.merchant} · {dateLabel(receipt.date)}
                        </small>
                      </span>
                      <span>{money(item.amount, currency)}</span>
                      <ChevronRight size={15} />
                    </button>
                  ))}
                </section>
              );
            })}
          </div>
        </Modal>
      )}
      {notice && !bulk && !scan && !selectedReceipt && !catalogOpen && !category && (
        <div className="sp-toast" role="status">
          <CheckCircle2 size={19} />
          <span>{notice.text}</span>
          <button aria-label="Dismiss" onClick={() => setNotice(null)}>
            <X size={16} />
          </button>
        </div>
      )}
    </div>
  );
}
