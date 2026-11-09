import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, within } from '@testing-library/react';
import { DayView } from './DayView';
import { setCrewZone } from './time';
import { flatDay } from './testFixtures';

afterEach(cleanup);

function mount(extra: Partial<Parameters<typeof DayView>[0]> = {}) {
  setCrewZone('Europe/Berlin');
  const onMark = vi.fn();
  render(
    <DayView
      day={flatDay()}
      myUid="demo-pip"
      filter="all"
      readonly={false}
      saving={null}
      disabled={false}
      onMark={onMark}
      {...extra}
    />,
  );
  return onMark;
}

describe('the day list', () => {
  it('groups each person by time of day, with Mochi too', () => {
    mount();
    const zed = screen.getByRole('heading', { name: 'Zed', level: 2 }).closest('section')!;
    expect(within(zed).getByRole('region', { name: 'Morning, Zed' })).toBeTruthy();
    expect(screen.getByRole('heading', { name: 'Mochi', level: 2 })).toBeTruthy();
    expect(screen.getByText('Robot parts are not dishes.')).toBeTruthy();
  });

  it('locks a second step until its first step is done', () => {
    const onMark = mount();
    const hang = screen.getByRole('button', { name: 'Check off: Hang the laundry, Zed, 11:00' });
    expect((hang as HTMLButtonElement).disabled).toBe(true);
    expect(within(hang).getByText('After Start the laundry')).toBeTruthy();
    fireEvent.click(
      screen.getByRole('button', { name: 'Check off: Start the laundry, Zed, 11:00' }),
    );
    expect(onMark).toHaveBeenCalledWith(expect.objectContaining({ id: '2026-11-10/wash' }));
  });

  it('says who checked a task off, calling you "You"', () => {
    mount();
    const mine = screen.getByRole('group', { name: 'Done: Breakfast for Mochi, Pip, 08:00' });
    expect(within(mine).getByText(/You/)).toBeTruthy();
    const theirs = screen.getByRole('group', {
      name: 'Done: Supervise kitchen cleaning 😼, Mochi, 21:00',
    });
    expect(within(theirs).getByText(/Zed/)).toBeTruthy();
  });

  it('shows history read-only', () => {
    mount({ readonly: true });
    expect(screen.queryByRole('button', { name: /^Undo:/ })).toBeNull();
    expect(screen.queryByRole('button', { name: /^Edit:/ })).toBeNull();
  });
});
