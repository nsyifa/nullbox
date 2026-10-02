import type { Copy } from "./en";

/**
 * 日本語コピー。`satisfies Copy` なので、キーが足りないとコンパイルエラーになる。
 * ジョークは翻訳ではなく日本語で書くこと。
 */
const ja = {
  app: {
    name: "Nullbox",
    subtitle: "ヌルボックス ― サポート、ただし虚無。",
    nav: { submit: "問題を報告", inbox: "対応者の受信箱" },
    language: "言語",
    footer: "この受信箱の制作において、チケットは一枚も傷ついておりません。",
  },

  status: {
    open: "生まれたて",
    in_progress: "見つめられ中",
    waiting_on_customer: "ボールはそちらに",
    resolved: "ぶん投げ済み",
    closed: "闇の世界へ送還",
  },

  severity: {
    p1: "全部燃えてる",
    p2: "怒りのタイピング中",
    p3: "猫がIT部門に就職",
    p4: "ちょっと不便",
  },

  animals: {
    platypus: "カモノハシ",
    capybara: "カピバラ",
    axolotl: "ウーパールーパー",
    narwhal: "イッカク",
    quokka: "クオッカ",
    pangolin: "センザンコウ",
    tardigrade: "クマムシ",
    red_panda: "レッサーパンダ",
    otter: "カワウソ",
    tanuki: "タヌキ",
    alpaca: "アルパカ",
    sloth: "ナマケモノ",
  },
  anonName: (animal: string, number: number) => `匿名の${animal} #${number}`,

  ticket: {
    number: (n: number) => `#${n}`,
    notFound: "見つかりません",
    notFoundHint: "チケット #404 は存在します。ただ、見つからないだけです。お約束。",
    unassigned: "まだ誰も",
    severity: "深刻度",
    status: "ステータス",
    assignee: "担当者",
    customer: "差出人",
    opened: (when: string) => `${when}に作成`,
    aging: {
      fresh: "新鮮",
      yellowed: "黄ばみ始め",
      cobwebs: "クモの巣発生",
      fossil: "もはや化石",
    },
  },

  customer: {
    title: "何が壊れましたか？",
    intro: "全部教えてください。チケットは必ず読みます。いつか。たぶん。",
    subject: "件名",
    subjectPlaceholder: "プリンターがまたこっちを見ている",
    body: "何が起きましたか？",
    bodyPlaceholder: "再現手順、エラーメッセージ、そのとき鳴った音など…",
    severity: "どれくらいヤバい？",
    name: "お名前（任意）",
    email: "メールアドレス（任意）",
    anonymousHint: "どちらも空欄なら、気品ある匿名ネームを差し上げます。",
    submit: "虚無に送信する",
    submitting: "発射中…",
    createdTitle: "チケットを受け付けました！",
    createdBody: (n: number) => `チケット #${n} は正式に誰か他の人の問題になりました。`,
    secretLinkTitle: "あなた専用の秘密リンク",
    secretLinkWarning:
      "このリンクがチケットに戻る唯一の方法です。ブックマークしてください。再送はできません。そもそもあなたが誰なのか知らないので。",
    copyLink: "リンクをコピー",
    copied: "コピーしました！",
    openTicket: "チケットを開く",
    missingToken:
      "このリンクには秘密の部分が欠けています。送信時に表示された完全なリンクを使ってください。",
    threadTitle: "やりとり",
    reply: "返信を追加",
    replyPlaceholder: "進展はありましたか？新しい症状は？新たな煙は？",
    send: "返信する",
    closedNotice: "このチケットは闇の世界へ送還されました。新しいチケットを作成してください。",
    sendHint: "Ctrl + Enter で送信",
  },

  worker: {
    inboxTitle: "受信箱",
    filter: "表示",
    filterActive: "まだ生きているもの全部",
    search: "件名で検索",
    searchPlaceholder: "件名で絞り込み…",
    noMatches: "該当なし。怪しい。",
    you: "あなたは",
    actorPlaceholder: "お名前",
    actorHint:
      "ログインではありません。誰でも誰にでもなれます。（ログインはアップグレードのひとつです）",
    back: "受信箱に戻る",
    reply: "お客様に返信",
    internalNote: "社内メモ",
    internalBadge: "社内",
    replyPlaceholder: "安心させる一言をどうぞ…",
    notePlaceholder: "対応者にしか見えません。愚痴はほどほどに。",
    send: "送信",
    yeet: "ぶん投げる",
    yeetHint: "チケットを解決済みにします。",
    yeeted: "太陽に向かってぶん投げました。",
    assigneePlaceholder: "名前（空欄で解除）",
    assign: "割り当て",
    assignToMe: "引き受ける",
  },

  beast: {
    title: "バックログの獣",
    moods: {
      asleep: "おやすみ中。起こさないで。",
      peckish: "小腹が空いてきた。こちらに気づいている。",
      grumpy: "ご機嫌ななめ。おやつ推奨。",
      kaiju: "怪獣モード。直ちに避難してください。",
    },
    openCount: (n: string) => `未対応 ${n} 件`,
    oldest: (age: string) => `最古: ${age}`,
  },

  zero: {
    celebration: "受信箱ゼロ達成！",
    void: "無料の虚無をお楽しみください。",
  },

  thread: {
    empty: "まだ返信はありません。静寂が耳に痛い。",
    statusChanged: (from: string, to: string) => `ステータス: ${from} → ${to}`,
    itsAlive: "生き返った！お客様の返信でチケットが墓場から蘇りました。",
    severityChanged: (from: string, to: string) => `深刻度: ${from} → ${to}`,
    assigned: (to: string) => `${to} さんが担当になりました`,
    unassigned: "担当者なし。自由の身。",
    system: "Nullbox",
    you: "あなた",
  },

  errors: {
    not_found: "隅々まで探しました。ソファの下まで。ありません。",
    missing_ticket_token: "このページには秘密リンクが必要です。",
    missing_actor: "まず名前を選んでください（右上）。",
    forbidden: "ここを通すわけにはいかない。",
    validation_failed: "入力内容に問題があるようです。各項目をご確認ください。",
    invalid_status_transition: "そのステータス変更はできません。時は一方向にしか流れません。",
    ticket_closed: "このチケットは完全に閉じられています。",
    service_unavailable: "サーバーが昼寝中です。少ししてからもう一度お試しください。",
    internal: "こちら側で何かが壊れました。手元の環境では動くんですけどね。",
    im_a_teapot: "私はティーポットです。",
    network_error: "サーバーに接続できません。コンセント、抜けてませんか？",
  },

  loading: [
    "データベースに袖の下を渡しています…",
    "ハムスターにもっと速く走るよう交渉中…",
    "ラバーダックに相談中…",
    "サーバーのご機嫌をうかがっています…",
    "再起動すれば直る気がしています…",
  ],

  eggs: {
    sudo: "この件は報告されます。",
    capsLock: "優先度: 大声",
    enterprise: {
      title: "エンタープライズモード™",
      intro: "続行する前に、以下の必須項目をすべてご記入ください。",
      fields: [
        "シナジー整合係数",
        "パラダイムシフト管理番号",
        "決裁者の好きなフォント",
        "FAX番号（必須）",
        "メールで済んだはずの会議の回数",
      ],
      submit: "稟議に回す",
      synergizing: "シナジー創出中…",
      exit: "現実に戻る",
    },
  },

  time: {
    unknown: "—",
  },
} satisfies Copy;

export default ja;
