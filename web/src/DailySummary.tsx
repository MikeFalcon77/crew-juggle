import { Check, Clock3, Sparkles } from 'lucide-react';
import { canMark } from './DayView';
import { getCrewZone, scheduledLabel } from './time';
import type { Day } from './types';
export default function DailySummary({ day, zone = getCrewZone() }: { day: Day; zone?: string }) {
  const items = day.items.filter((i) => i.status !== 'cancelled');
  const done = items.filter((i) => i.status === 'done').length;
  const remaining = items.length - done;
  const clock = new Intl.DateTimeFormat('en-GB', {
    timeZone: getCrewZone(),
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(day.server_now));
  const pending = items
    .filter((i) => i.status === 'pending' && canMark(i, items))
    .sort((a, b) => a.routine.time.localeCompare(b.routine.time));
  const next = pending.find((i) => i.routine.time >= clock) || pending[0];
  const allDone = remaining === 0 && items.length > 0;
  return (
    <section className={`daily-summary ${allDone ? 'all-done' : ''}`}>
      <span className="summary-icon">{allDone ? <Check size={25} /> : <Sparkles size={25} />}</span>
      <div className="summary-copy">
        <strong>{allDone ? 'Everything is checked off' : 'One list. The whole crew.'}</strong>
        <p>
          {items.length
            ? `${done} of ${items.length} done${remaining ? ` · ${remaining} to go` : '. Nice teamwork.'}`
            : 'Nothing planned for this day'}
        </p>
        {day.date === day.today && next && (
          <span className="next-task">
            <Clock3 size={12} />
            {next.routine.time >= clock ? 'Up next' : 'Waiting'}:{' '}
            {scheduledLabel(
              next.routine.time,
              day.scheduled_times[next.routine.time],
              zone,
              day.date,
            )}{' '}
            · {next.routine.name} · {next.routine.assignee}
          </span>
        )}
      </div>
      <div className="summary-meter">
        <span>{items.length ? Math.round((done / items.length) * 100) : 0}%</span>
        <div>
          <i style={{ width: `${items.length ? (done / items.length) * 100 : 0}%` }} />
        </div>
      </div>
    </section>
  );
}
