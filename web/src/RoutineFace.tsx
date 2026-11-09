import type { Mood } from './types';
export const moods = [
  { id: 'happy' as Mood, label: 'Happy' },
  { id: 'neutral' as Mood, label: 'Chill' },
  { id: 'grumpy' as Mood, label: 'Grumpy (nobody likes this one)' },
];
export default function RoutineFace({
  mood = 'happy',
  celebrating = false,
}: {
  mood?: Mood;
  celebrating?: boolean;
}) {
  return (
    <span
      className={`routine-face mood-${mood} ${celebrating ? 'face-celebrate' : ''}`}
      role="img"
      aria-label={moods.find((m) => m.id === mood)!.label}
    >
      <svg viewBox="0 0 48 48" aria-hidden="true">
        <path
          className="face-body"
          d="M24 4C11 4 5 12 5 25c0 12 7 19 19 19s19-7 19-19C43 12 37 4 24 4Z"
        />
        <path
          d="M12 13c2-3 5-5 9-5"
          stroke="white"
          strokeWidth="3"
          strokeLinecap="round"
          opacity=".65"
        />
        <circle cx="12" cy="28" r="3" fill="#ec9b9b" opacity=".5" />
        <circle cx="36" cy="28" r="3" fill="#ec9b9b" opacity=".5" />
        {mood === 'happy' ? (
          <>
            <path d="m14 21 3-2 3 2m8 0 3-2 3 2" />
            <path d="M17 28q7 10 14 0" />
          </>
        ) : mood === 'neutral' ? (
          <>
            <circle cx="17" cy="22" r="1.5" />
            <circle cx="31" cy="22" r="1.5" />
            <path d="M20 31h8" />
          </>
        ) : (
          <>
            <path d="m14 17 6 3m8 0 6-3" />
            <circle cx="17" cy="24" r="1.5" />
            <circle cx="31" cy="24" r="1.5" />
            <path d="M19 33q5-5 10 0" />
          </>
        )}
      </svg>
    </span>
  );
}
