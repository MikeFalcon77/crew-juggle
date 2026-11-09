import { useQuery } from '@tanstack/react-query';
import { api, apiAt } from './api';
import { todayCards, type ShoppingGlance } from './today';
import type { Day, Person, AvatarMeta } from './types';

/** Today pulls from two gears: routines (the day) and shopping (the list). */
export function useTodayTracks({
  day,
  routinesLoading,
  routinesError,
  online,
}: {
  day?: Day;
  routinesLoading: boolean;
  routinesError: boolean;
  online: boolean;
}) {
  const directory = useQuery({
    queryKey: ['people'],
    queryFn: () => api<{ people: Person[]; avatars?: Record<string, AvatarMeta> }>('/people'),
    enabled: !day,
  });
  const shopping = useQuery({
    queryKey: ['shopping-glance'],
    queryFn: () => apiAt<ShoppingGlance>('/juggle-shopping/v1/state'),
    refetchInterval: 15000,
  });
  const today = day?.today || '';
  const messages: string[] = [];
  if (routinesLoading) messages.push('Loading routines…');
  if (routinesError || !online)
    messages.push(
      `Routines: ${day ? 'showing the last saved copy' : 'unavailable'}. Check-offs are paused.`,
    );
  if (shopping.isError) messages.push('Shopping is unavailable right now.');
  const cards = todayCards({ day, shopping: shopping.data, today });
  return {
    today,
    people: day?.people || directory.data?.people || [],
    avatars: day?.avatars || directory.data?.avatars,
    cards,
    messages,
    incomplete: routinesLoading || routinesError || !online || shopping.isPending,
  };
}
