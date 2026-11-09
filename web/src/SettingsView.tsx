import { Settings, BookOpen, Code2 } from 'lucide-react';
import TimeSettingsView from './TimeSettingsView';
import NotificationSettings, { type NotificationSettingsProps } from './NotificationSettings';
import type { CrewPreferences } from './types';
import type { AppRoute } from './routes';
import { signInMode } from './session';

type Props = {
  data?: CrewPreferences;
  preferencesError?: boolean;
  myUid: string;
  disabled: boolean;
  onError: (message: string) => void;
  section?: string;
  onNavigate: (r: AppRoute) => void;
  notifications: NotificationSettingsProps;
  deviceId?: string | null;
};

export default function SettingsView(props: Props) {
  const {
    section = 'notifications',
    onNavigate,
    disabled,
    data,
    myUid,
    onError,
    notifications,
    deviceId = null,
  } = props;
  const sections = [
    ['notifications', 'Notifications'],
    ['time', 'Time zone'],
    ['about', 'About'],
  ];
  return (
    <>
      <div className="section-heading">
        <div>
          <span className="eyebrow">SETTINGS</span>
          <h1>Settings</h1>
          <p>Notifications, your time zone, and what this app is made of.</p>
        </div>
        <Settings size={28} />
      </div>
      <nav className="track-tabs settings-tabs" aria-label="Settings sections">
        {sections.map(([id, label]) => (
          <button
            key={id}
            aria-current={section === id ? 'page' : undefined}
            className={section === id ? 'active' : ''}
            onClick={() => onNavigate({ page: 'settings', section: id })}
          >
            {label}
          </button>
        ))}
      </nav>
      {section === 'notifications' && (
        <div className="settings-grid">
          <NotificationSettings {...notifications} />
          <section className="settings-card">
            <h2>Send a test</h2>
            <p>Checks that this device really receives pushes.</p>
            <button
              className="secondary"
              disabled={!deviceId || disabled || notifications.busy}
              onClick={() => notifications.onPush('test')}
            >
              Send a test push
            </button>
          </section>
        </div>
      )}
      {section === 'time' &&
        (data ? (
          <TimeSettingsView data={data} myUid={myUid} disabled={disabled} onError={onError} />
        ) : (
          <div className="notice" role="status">
            {props.preferencesError ? "Couldn't load your time zone." : 'Loading…'}
          </div>
        ))}
      {section === 'about' && (
        <div className="settings-grid">
          <section className="settings-card">
            <h2>crew-juggle</h2>
            <p>
              Shared routines and shopping for your crew. An open-source example app built on
              Constructor Gears: every screen talks to a gear that owns its own data.
            </p>
            <p>
              Signed in with:{' '}
              <strong>{signInMode() === 'demo' ? 'demo character' : 'Google'}</strong>
              {signInMode() === 'demo' && ' · demo data resets when the server restarts'}
            </p>
          </section>
          <section className="settings-card">
            <h2>Look under the hood</h2>
            <p>Every gear's API, generated from the routes it declares.</p>
            <a className="secondary link-button" href="/docs" target="_blank" rel="noreferrer">
              <BookOpen size={16} /> Open API docs
            </a>
            <a
              className="text-button link-button"
              href="https://github.com/MikeFalcon77/crew-juggle"
              target="_blank"
              rel="noreferrer"
            >
              <Code2 size={16} /> Source code
            </a>
          </section>
        </div>
      )}
    </>
  );
}
