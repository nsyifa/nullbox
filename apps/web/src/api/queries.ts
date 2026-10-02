/** TanStack Query hooks on top of the API client. */
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./client";
import type { TicketStatus } from "./generated/TicketStatus";
import type { UpdateTicketRequest } from "./generated/UpdateTicketRequest";
import type { WorkerMessageRequest } from "./generated/WorkerMessageRequest";

export const keys = {
  backlog: ["backlog"] as const,
  inbox: (status?: TicketStatus) => ["tickets", "list", status ?? "active"] as const,
  ticket: (id: string) => ["tickets", "detail", id] as const,
  customerTicket: (id: string) => ["public-ticket", id] as const,
};

export function useBacklogStats() {
  return useQuery({ queryKey: keys.backlog, queryFn: api.backlogStats, refetchInterval: 15_000 });
}

export function useInbox(status?: TicketStatus) {
  return useQuery({
    queryKey: keys.inbox(status),
    queryFn: () => api.listTickets(status),
    refetchInterval: 15_000,
  });
}

export function useWorkerTicket(id: string) {
  return useQuery({ queryKey: keys.ticket(id), queryFn: () => api.getTicket(id) });
}

export function useCustomerTicket(id: string, token: string | null) {
  return useQuery({
    queryKey: keys.customerTicket(id),
    queryFn: () => api.getCustomerTicket(id, token ?? ""),
    enabled: Boolean(token),
  });
}

/** After any worker change, the inbox, the ticket and the Beast may all be stale. */
function useInvalidateTickets() {
  const queryClient = useQueryClient();
  return () => {
    queryClient.invalidateQueries({ queryKey: ["tickets"] });
    queryClient.invalidateQueries({ queryKey: keys.backlog });
  };
}

export function useCreateTicket() {
  return useMutation({ mutationFn: api.createTicket });
}

export function useCustomerReply(id: string, token: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (body: string) => api.replyAsCustomer(id, token, body),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: keys.customerTicket(id) }),
  });
}

export function useAddMessage(id: string) {
  const invalidate = useInvalidateTickets();
  return useMutation({
    mutationFn: (body: WorkerMessageRequest) => api.addMessage(id, body),
    onSuccess: invalidate,
  });
}

export function useUpdateTicket(id: string) {
  const invalidate = useInvalidateTickets();
  return useMutation({
    mutationFn: (body: UpdateTicketRequest) => api.updateTicket(id, body),
    onSuccess: invalidate,
  });
}

export function useYeetTicket() {
  const invalidate = useInvalidateTickets();
  return useMutation({ mutationFn: api.yeetTicket, onSuccess: invalidate });
}
