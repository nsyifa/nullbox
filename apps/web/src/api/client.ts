/**
 * The single API client: the only module that calls `fetch`.
 *
 * - Base URL from `VITE_API_BASE_URL`, so the same bundle works behind any
 *   origin (Tauri, a CDN, a different domain). No cookies, no same-origin assumptions.
 * - Auth headers are added in `authHeaders()`: that's where a bearer token goes later.
 */
import { getActorName } from "./actor";
import type { BacklogStats } from "./generated/BacklogStats";
import type { CreateTicketRequest } from "./generated/CreateTicketRequest";
import type { CreateTicketResponse } from "./generated/CreateTicketResponse";
import type { ErrorBody } from "./generated/ErrorBody";
import type { ErrorCode } from "./generated/ErrorCode";
import type { Message } from "./generated/Message";
import type { Ticket } from "./generated/Ticket";
import type { TicketStatus } from "./generated/TicketStatus";
import type { TicketThread } from "./generated/TicketThread";
import type { UpdateTicketRequest } from "./generated/UpdateTicketRequest";
import type { WorkerMessageRequest } from "./generated/WorkerMessageRequest";

export const API_BASE_URL: string = (
  import.meta.env.VITE_API_BASE_URL ?? "http://localhost:3000"
).replace(/\/$/, "");

export type ClientErrorCode = ErrorCode | "network_error";

export class ApiError extends Error {
  constructor(
    readonly code: ClientErrorCode,
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

/** Worker identity. Today an untrusted name header; tomorrow `Authorization: Bearer …`. */
function authHeaders(): Record<string, string> {
  // Percent-encoded so non-ASCII names (e.g. 山田) survive HTTP headers.
  return { "X-Actor-Name": encodeURIComponent(getActorName()) };
}

function ticketTokenHeaders(token: string): Record<string, string> {
  return { "X-Ticket-Token": token };
}

async function request<T>(
  method: "GET" | "POST" | "PATCH",
  path: string,
  options: { body?: unknown; headers?: Record<string, string> } = {},
): Promise<T> {
  const headers: Record<string, string> = { Accept: "application/json", ...options.headers };
  if (options.body !== undefined) headers["Content-Type"] = "application/json";

  let res: Response;
  try {
    res = await fetch(`${API_BASE_URL}/api${path}`, {
      method,
      headers,
      body: options.body === undefined ? undefined : JSON.stringify(options.body),
    });
  } catch (err) {
    throw new ApiError("network_error", 0, String(err));
  }

  if (!res.ok) {
    const error = (await res.json().catch(() => null)) as ErrorBody | null;
    throw new ApiError(error?.code ?? "internal", res.status, error?.message ?? res.statusText);
  }
  return (await res.json()) as T;
}

export const api = {
  // customers
  createTicket: (body: CreateTicketRequest) =>
    request<CreateTicketResponse>("POST", "/tickets", { body }),
  getCustomerTicket: (id: string, token: string) =>
    request<TicketThread>("GET", `/public/tickets/${id}`, { headers: ticketTokenHeaders(token) }),
  replyAsCustomer: (id: string, token: string, body: string) =>
    request<Message>("POST", `/public/tickets/${id}/messages`, {
      body: { body },
      headers: ticketTokenHeaders(token),
    }),

  // workers
  listTickets: (status?: TicketStatus) =>
    request<Ticket[]>("GET", status ? `/tickets?status=${status}` : "/tickets", {
      headers: authHeaders(),
    }),
  getTicket: (id: string) =>
    request<TicketThread>("GET", `/tickets/${id}`, { headers: authHeaders() }),
  addMessage: (id: string, body: WorkerMessageRequest) =>
    request<Message>("POST", `/tickets/${id}/messages`, { body, headers: authHeaders() }),
  updateTicket: (id: string, body: UpdateTicketRequest) =>
    request<Ticket>("PATCH", `/tickets/${id}`, { body, headers: authHeaders() }),
  /** Yeet = close (status becomes `resolved`). */
  yeetTicket: (id: string) =>
    request<Ticket>("POST", `/tickets/${id}/yeet`, { headers: authHeaders() }),
  backlogStats: () => request<BacklogStats>("GET", "/stats/backlog", { headers: authHeaders() }),
};
