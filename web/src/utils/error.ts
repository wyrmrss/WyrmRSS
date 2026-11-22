/** API failure carrying the status, the server's curated `error` message and its optional `code`. */
export class ApiError extends Error {
  status: number;
  code?: string;

  constructor(status: number, message: string, code?: string) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

export async function throwApiError(res: Response): Promise<never> {
  const body = (await res.json().catch(() => undefined)) as { error?: string; code?: string } | undefined;
  // A write blocked in demo mode (see DemoBanner).
  if (body?.code === "demo_restricted") window.dispatchEvent(new Event("wyrm:demo-restricted"));
  throw new ApiError(res.status, body?.error ?? `${res.status} ${res.statusText}`, body?.code);
}
