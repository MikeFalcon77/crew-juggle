import { token } from './session';
export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
    public code?: string,
  ) {
    super(message);
  }
}
/** Calls the routines gear: `api('/day')` is `GET /juggle-routines/v1/day`. */
export async function api<T>(
  path: string,
  method = 'GET',
  body?: unknown,
  force = false,
): Promise<T> {
  return apiAt<T>(`/juggle-routines/v1${path}`, method, body, force);
}
/** Calls any gear by its full versioned path, such as `/juggle-crews/v1/me`. */
export async function apiAt<T>(
  path: string,
  method = 'GET',
  body?: unknown,
  force = false,
): Promise<T> {
  if (!path.startsWith('/') || path.startsWith('//') || path.includes('\\'))
    throw new ApiError(400, 'Invalid request path');
  const jwt = await token(force);
  if (!jwt) throw new ApiError(401, 'Sign in first');
  let response: Response;
  const multipart = body instanceof FormData;
  try {
    response = await fetch(path, {
      method,
      headers: {
        Authorization: `Bearer ${jwt}`,
        ...(body === undefined || multipart ? {} : { 'Content-Type': 'application/json' }),
      },
      body: body === undefined ? undefined : multipart ? body : JSON.stringify(body),
      cache: 'no-store',
      signal: AbortSignal.timeout(
        multipart && path.startsWith('/juggle-shopping/') ? 90000 : 15000,
      ),
    });
  } catch {
    throw new ApiError(0, "Can't reach the server. Nothing was saved.");
  }
  if (response.status === 401 && !force) return apiAt<T>(path, method, body, true);
  if (!response.ok) {
    // Every gear answers errors the same way (RFC 9457): title, detail, error_code.
    const error = await response.json().catch(() => ({}));
    throw new ApiError(
      response.status,
      error.detail ||
        error.title ||
        (response.status === 401
          ? 'This account cannot sign in here'
          : "Couldn't save. Refresh and try again."),
      error.error_code || error.code,
    );
  }
  return response.json() as Promise<T>;
}
export async function avatarBlob(id: string, signal: AbortSignal, force = false): Promise<Blob> {
  const jwt = await token(force);
  if (!jwt) throw new ApiError(401, 'Sign in first');
  const response = await fetch(`/juggle-routines/v1/people/${encodeURIComponent(id)}/avatar`, {
    headers: { Authorization: `Bearer ${jwt}` },
    cache: 'no-store',
    signal,
  });
  if (response.status === 401 && !force) return avatarBlob(id, signal, true);
  if (!response.ok) throw new ApiError(response.status, "Couldn't load the photo");
  return response.blob();
}
