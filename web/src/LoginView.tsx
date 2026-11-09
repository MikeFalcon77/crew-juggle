import { ArrowRight, ShieldCheck, Sparkles } from 'lucide-react';
import type { DemoUser, SignIn } from './session';

/** Sign in with Google, or, on a local demo server, by picking a character. */
export default function LoginView({
  mode,
  demoUsers,
  googleReady,
  busy,
  error,
  onGoogle,
  onDemo,
}: {
  mode: SignIn;
  demoUsers: DemoUser[];
  googleReady: boolean;
  busy: boolean;
  error: string;
  onGoogle: () => void;
  onDemo: (key: string) => void;
}) {
  const crews = [...new Set(demoUsers.map((u) => u.crew))];
  return (
    <div className="login-page">
      <div className="login-decoration decoration-one" />
      <div className="login-decoration decoration-two" />
      <div className="login-card">
        <div className="brand">
          <span className="brand-symbol">
            <img src="/icon.svg" alt="" width="45" height="45" />
          </span>
          <span>
            crew-juggle<small>ROUTINES + SHOPPING</small>
          </span>
        </div>
        <span className="eyebrow">YOUR CREW, IN SYNC</span>
        <h1>
          Chores on repeat.
          <br />
          Receipts that read themselves.
        </h1>
        <p>One shared list for your flat, team or club. Everyone sees what's done.</p>
        {mode === 'google' ? (
          <button className="google-button" disabled={!googleReady || busy} onClick={onGoogle}>
            {busy ? <span className="spinner small" /> : <span className="google-g">G</span>}
            Sign in with Google
            <ArrowRight size={18} />
          </button>
        ) : (
          <div className="demo-picker">
            <span className="demo-badge">
              <Sparkles size={14} /> Demo · pick a character
            </span>
            {crews.map((crew) => (
              <section key={crew} aria-label={crew}>
                <h2>{crew}</h2>
                <div className="demo-characters">
                  {demoUsers
                    .filter((u) => u.crew === crew)
                    .map((user) => (
                      <button
                        key={user.key}
                        className={`demo-character demo-${user.key}`}
                        disabled={busy}
                        onClick={() => onDemo(user.key)}
                      >
                        <img src={`/characters/${user.key}.svg`} alt="" width="56" height="56" />
                        <strong>{user.name}</strong>
                        <small>{user.tagline}</small>
                      </button>
                    ))}
                </div>
              </section>
            ))}
            <p className="demo-note">
              Demo data resets when the server restarts. Mochi the cat lives in Flat 4B, has no
              account, and still has a job.
            </p>
          </div>
        )}
        {error && (
          <div className="notice error" role="alert">
            {error}
          </div>
        )}
        {mode === 'google' && !googleReady && (
          <div className="notice error">Google sign-in is not configured on this server.</div>
        )}
        <div className="login-private">
          <ShieldCheck size={15} />
          Only your crew sees your crew's stuff
        </div>
      </div>
      <footer className="login-footer">Open source · built with Rust and Constructor Gears</footer>
    </div>
  );
}
