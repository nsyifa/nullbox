import type { ErrorCode } from "../api/generated/ErrorCode";
import type { Severity } from "../api/generated/Severity";
import type { TicketStatus } from "../api/generated/TicketStatus";

/**
 * English copy. This object defines the `Copy` type: every other dictionary
 * (ja.ts, and later Pirate, Corporate Speak or "Serious Mode") must match it.
 * Interpolation is done with functions, e.g. `openCount(3)`.
 */
const en = {
  app: {
    name: "Nullbox",
    subtitle: "Support, but make it void.",
    nav: { submit: "Report a problem", inbox: "Worker inbox" },
    language: "Language",
    footer: "No tickets were harmed in the making of this inbox.",
  },

  status: {
    open: "Freshly Hatched",
    in_progress: "Being Stared At",
    waiting_on_customer: "Ball's in Your Court",
    resolved: "Yeeted",
    closed: "Sent to the Shadow Realm",
  } satisfies Record<TicketStatus, string>,

  severity: {
    p1: "Everything is on fire",
    p2: "Rage-typing",
    p3: "My cat is now in IT",
    p4: "Mildly inconvenienced",
  } satisfies Record<Severity, string>,

  /** Keys come from the backend (`nullbox_domain::ANON_ANIMALS`). */
  animals: {
    platypus: "Platypus",
    capybara: "Capybara",
    axolotl: "Axolotl",
    narwhal: "Narwhal",
    quokka: "Quokka",
    pangolin: "Pangolin",
    tardigrade: "Tardigrade",
    red_panda: "Red Panda",
    otter: "Otter",
    tanuki: "Tanuki",
    alpaca: "Alpaca",
    sloth: "Sloth",
  } as Record<string, string>,
  anonName: (animal: string, number: number) => `Anonymous ${animal} #${number}`,

  ticket: {
    number: (n: number) => `#${n}`,
    notFound: "Not Found",
    notFoundHint: "Ticket #404 exists. We just can't find it. Classic.",
    unassigned: "Nobody (yet)",
    severity: "Severity",
    status: "Status",
    assignee: "Assignee",
    customer: "From",
    opened: (when: string) => `opened ${when}`,
    aging: {
      fresh: "Fresh",
      yellowed: "Starting to yellow",
      cobwebs: "Cobwebs detected",
      fossil: "Officially a fossil",
    },
  },

  customer: {
    title: "What broke?",
    intro: "Tell us everything. We read every ticket. Eventually. Probably.",
    subject: "Subject",
    subjectPlaceholder: "The printer is making eye contact again",
    body: "What happened?",
    bodyPlaceholder: "Steps to reproduce, error messages, the sound it made…",
    severity: "How bad is it?",
    name: "Your name (optional)",
    email: "Email (optional)",
    anonymousHint: "Skip both and you'll get a dignified anonymous alias.",
    submit: "Send it into the void",
    submitting: "Launching…",
    createdTitle: "Ticket received!",
    createdBody: (n: number) => `Ticket #${n} is now officially somebody else's problem.`,
    secretLinkTitle: "Your secret link",
    secretLinkWarning:
      "This link is the only way back to your ticket. Bookmark it. We can't resend it: we don't even know who you are.",
    copyLink: "Copy link",
    copied: "Copied!",
    openTicket: "Open my ticket",
    missingToken:
      "This link is missing its secret part. Use the full link you got when you submitted the ticket.",
    threadTitle: "Conversation",
    reply: "Add a reply",
    replyPlaceholder: "Any updates? New symptoms? Fresh smoke?",
    send: "Send reply",
    closedNotice: "This ticket was sent to the Shadow Realm. Please open a new one.",
    sendHint: "Ctrl + Enter to send",
  },

  worker: {
    inboxTitle: "Inbox",
    filter: "Show",
    filterActive: "Everything still alive",
    search: "Search subjects",
    searchPlaceholder: "Filter by subject…",
    noMatches: "Nothing matches. Suspicious.",
    you: "You are",
    actorPlaceholder: "Your name",
    actorHint: "Not a login. Anyone can claim to be anyone. (Login is one of your upgrades.)",
    back: "Back to inbox",
    reply: "Reply to customer",
    internalNote: "Internal note",
    internalBadge: "Internal",
    replyPlaceholder: "Write something reassuring…",
    notePlaceholder: "Only workers can see this. Vent responsibly.",
    send: "Send",
    yeet: "Yeet it",
    yeetHint: "Marks the ticket as resolved.",
    yeeted: "Yeeted into the sun.",
    assigneePlaceholder: "Name or empty",
    assign: "Assign",
    assignToMe: "Take it",
  },

  beast: {
    title: "Backlog Beast",
    moods: {
      asleep: "Asleep. Do not wake it.",
      peckish: "Peckish. It has noticed you.",
      grumpy: "Grumpy. Snacks recommended.",
      kaiju: "KAIJU MODE. Evacuate the city.",
    },
    openCount: (n: string) => `${n} open`,
    oldest: (age: string) => `Oldest: ${age}`,
  },

  zero: {
    celebration: "INBOX ZERO!",
    void: "Please enjoy this complimentary void.",
  },

  thread: {
    empty: "No replies yet. The silence is deafening.",
    statusChanged: (from: string, to: string) => `Status: ${from} → ${to}`,
    itsAlive: "IT'S ALIVE! The customer replied and the ticket rose from the dead.",
    severityChanged: (from: string, to: string) => `Severity: ${from} → ${to}`,
    assigned: (to: string) => `Assigned to ${to}`,
    unassigned: "Unassigned. Free as a bird.",
    system: "Nullbox",
    you: "You",
  },

  errors: {
    not_found: "We looked everywhere. Even under the couch. It's not there.",
    missing_ticket_token: "This page needs your secret link.",
    missing_actor: "Pick a name first (top right).",
    forbidden: "You shall not pass.",
    validation_failed: "Something in the form doesn't look right. Please check the fields.",
    invalid_status_transition: "That status change isn't allowed. Time only flows one way.",
    ticket_closed: "This ticket is closed for good.",
    service_unavailable: "The server is taking a nap. Try again in a moment.",
    internal: "Something broke on our side. It works on our machine, though.",
    im_a_teapot: "I'm a teapot. Short and stout.",
    network_error: "Can't reach the server. Is it plugged in?",
  } satisfies Record<ErrorCode | "network_error", string>,

  loading: [
    "Bribing the database…",
    "Reticulating splines…",
    "Asking the hamsters to run faster…",
    "Consulting the rubber duck…",
    "Warming up the tubes…",
  ],

  eggs: {
    sudo: "This incident will be reported.",
    capsLock: "Priority: Loud",
    enterprise: {
      title: "Enterprise Mode™",
      intro: "Before you continue, please complete these mandatory fields.",
      fields: [
        "Synergy alignment quotient",
        "Paradigm shift reference ID",
        "Stakeholder's favorite font",
        "Fax number (required)",
        "Number of meetings this could have been an email",
      ],
      submit: "Submit for approval",
      synergizing: "Synergizing…",
      exit: "Return to reality",
    },
  },

  time: {
    unknown: "—",
  },
};

export type Copy = typeof en;
export default en;
