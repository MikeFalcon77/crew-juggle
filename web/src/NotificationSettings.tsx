import { Bell, Check, Volume2, VolumeX } from 'lucide-react';
import { pushSupported } from './push';
export type NotificationSettingsProps = {
  pushId: string | null;
  busy: boolean;
  online: boolean;
  sound: boolean;
  onPush: (action: 'enable' | 'disable' | 'test') => void;
  onSound: () => void;
};
export default function NotificationSettings({
  pushId,
  busy,
  online,
  sound,
  onPush,
  onSound,
}: NotificationSettingsProps) {
  return (
    <section className="settings-card">
      <h2>Notifications</h2>
      <p>
        Each device asks for permission on its own. You hear about your tasks and the ones of crew
        members without an account.
      </p>
      <div className="notification-actions">
        {pushId ? (
          <>
            <span className="push-status">
              <Check size={15} />
              Notifications on
            </span>
            <button
              className="text-button"
              disabled={busy || !online}
              onClick={() => onPush('disable')}
            >
              Turn off
            </button>
          </>
        ) : (
          <button
            className="primary"
            disabled={busy || !online || !pushSupported()}
            onClick={() => onPush('enable')}
          >
            <Bell size={16} />
            {busy ? 'Setting up…' : 'Turn on notifications'}
          </button>
        )}
        <button className="sound-button" aria-pressed={sound} onClick={onSound}>
          {sound ? <Volume2 size={17} /> : <VolumeX size={17} />}Sound {sound ? 'on' : 'off'}
        </button>
      </div>
      <p>
        On iPhone, add the app to your Home Screen first. Use “Send a test” to check this device.
      </p>
    </section>
  );
}
