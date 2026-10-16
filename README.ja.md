# Nullbox

[English](README.md) | 日本語

> ヌルボックス ― サポート、ただし虚無。

わざと小さく、わざとふざけて作ったチケット管理システムです。お客様が問題を報告し、対応者が返信し、
**バックログの獣**に見守られながら、全員で**受信箱ゼロ**を目指します。

これがハッカソンの出発点です。製品としては動きますが、エンタープライズ向けの機能は**何もありません**。
認証も、CI/CDも、オブザーバビリティも、バックアップもありません。それを足すのがみなさんの仕事です。
どの機能にも、手を入れる場所がコードにはっきり用意してあります（[拡張ポイント](#拡張ポイント)を参照）。

## クイックスタート

必要なもの: Rust（バージョンは `rust-toolchain.toml` で固定）、Node 22（`.nvmrc`）、Corepack 経由の pnpm
（`corepack enable`）。

```sh
# API は http://localhost:3000（起動時に nullbox.db を作成し、マイグレーションを実行します）
cp .env.example .env
cargo run -p nullbox-api

# Web アプリは http://localhost:5173（API とは別オリジンで通信します。開発用プロキシはありません）
pnpm install
cp apps/web/.env.example apps/web/.env
pnpm dev
```

<http://localhost:5173> でチケットを作成でき、<http://localhost:5173/inbox> が対応者の受信箱です。

### Dev Container

必要なものを自分でインストールする代わりに、リポジトリを [Dev Container](https://containers.dev/) で開くこともできます
（VS Code: **Dev Containers: Reopen in Container**。Docker が必要です）。Rust、Node、pnpm、GitHub CLI が入っていて、
依存関係のインストールと2つの `.env` ファイルの作成まで済ませます。あとは `cargo run -p nullbox-api` と `pnpm dev` で
API と Web アプリを起動してください。ポート 3000 と 5173 は手元のマシンに転送されます。

`target/` と `node_modules/` は Docker ボリュームに置かれるので、手元のマシンからは見えません。Rust のバージョンを
変えるときは `rust-toolchain.toml` だけを変更すれば十分です。`devcontainer.json` のバージョンは、ダウンロードを
省くためのものです。Docker 自体は含まれていません。必要なら `docker-outside-of-docker` フィーチャーを追加してください。

GitHub Codespaces は、デスクトップ版の VS Code で開けば動きます。ブラウザ版では、アプリから API に届きません。
そこでの `localhost` は手元のマシンを指すためです。

## チェック

```sh
cargo test                 # ドメインと HTTP のテスト。apps/web/src/api/generated も再生成します
cargo clippy --all-targets
cargo fmt --check
pnpm test                  # Vitest + Testing Library
pnpm lint                  # Biome
pnpm build                 # 型チェックと、apps/web/dist への本番ビルド
```

今のところ、これらを強制する仕組みはありません。それもアップグレードのひとつです。

## 構成

```
apps/web                     React + Vite の SPA（お客様用ポータルは /、対応者の受信箱は /inbox）
  src/api/client.ts          fetch を呼ぶ唯一のモジュール
  src/api/generated/         ts-rs が Rust から生成する TS の型（コミット済み）
  src/copy/                  en.ts、ja.ts と CopyProvider
  src/router.tsx             すべてのルート（Tauri では createHashRouter に切り替え）
crates/domain                純粋な型とルール: ステータス、遷移、獣の機嫌。IO なし
crates/api                   axum サーバー、サービス、sqlx リポジトリ
migrations/                  SQL マイグレーション。バイナリに埋め込まれ、起動時に実行されます
.sqlx/                       sqlx のオフライン用クエリキャッシュ。クローン直後でも DB なしでビルドできます
```

## API

すべて JSON で、`/api` の下にあります。エラーは必ず `{ "code": ErrorCode, "message": string }` の形です。
`code` は変わらない値で、TypeScript と共有しています。`message` は英語で、開発者向けです。

| メソッド | パス | 備考 |
|---|---|---|
| POST | `/tickets` | 作成。`{ ticket, access_token, access_link }` を返します |
| GET | `/public/tickets/{id}` | お客様用の表示。ヘッダー `X-Ticket-Token` が必要 |
| POST | `/public/tickets/{id}/messages` | お客様の返信。解決済みのチケットは再オープンされます |
| GET | `/tickets?status=` | 対応者の受信箱。status なしなら、対応中のチケットすべて |
| GET | `/tickets/{id}` | 対応者用の表示。社内メモを含みます |
| POST | `/tickets/{id}/messages` | `{ body, internal? }` |
| PATCH | `/tickets/{id}` | `{ status?, severity?, assignee_name? }`（`""` で担当を解除） |
| POST | `/tickets/{id}/yeet` | クローズ: ステータスを `resolved`（「ぶん投げ済み」）にします |
| GET | `/stats/backlog` | `{ open_count, beast_mood, oldest_open_age_secs }` |
| GET | `/admin/export` | すべてのチケット。CEO のスプレッドシート用。ヘッダー `X-Admin-Token` が必要 |
| GET | `/healthz` | 死活確認 |
| GET | `/teapot` | 418。RFC 2324 の定めるとおり |

対応者用のエンドポイントは**保護されていません**。`X-Actor-Name` は、クライアントが名乗ったとおりの値になります。

## 設定

API は環境変数または `.env` から読みます（`.env.example` と `crates/api/src/config.rs` を参照）:
`BIND_ADDR`、`DATABASE_URL`、`CORS_ALLOWED_ORIGINS`、`PUBLIC_WEB_URL`、`ADMIN_TOKEN`、`RUST_LOG`。

Web はビルド時に読みます: `VITE_API_BASE_URL`。API のオリジンで、`/api` は付けません。未設定の場合、バンドルは
`http://localhost:3000` を呼びます。これが正しいのは手元のマシンだけです。デプロイするときは、サイトのアドレスを
設定してください。1つの Web サーバーが Web アプリと `/api` の両方を配信するなら、空の値にします。

## 拡張ポイント

| 追加したいもの | ここから始める |
|---|---|
| ログイン: パスワード、SSO、セッション | `crates/api/src/http/extract.rs` の `Actor` エクストラクター（`AppState` を受け取ります）、`apps/web/src/api/client.ts` の `authHeaders()`。別オリジンで Cookie を使うなら、`crates/api/src/http/mod.rs` の `cors()` を参照 |
| ロール / 権限 | `crates/api/src/policy.rs` の `policy::can`（今は常に `true`） |
| 監査ログ、Webhook、メール、メトリクス | `crates/api/src/events.rs` の `EventSink`（今はログに出すだけ）。拒否したリクエストは `Actor` エクストラクターで |
| レート制限、トレースのエクスポーター、CORS の強化 | `crates/api/src/http/mod.rs` の `middleware()` |
| Postgres、マルチテナント、バックアップ | `crates/api/src/repo/mod.rs` の `TicketRepository` |
| OpenTelemetry | すべてのログは `tracing` を通ります。`main.rs` にレイヤーを追加してください |
| 決済 | `crates/api/src/http/mod.rs` で、ほかのルートの隣に新しいルートを追加。ページは `apps/web/src/router.tsx`、キーは `Config` に |
| AI エージェント向けの MCP サーバー | `router()` の `/api` の nest の隣に、`/mcp` のサービスを追加。ツールからは、対応者用のハンドラーと同じように `TicketService` を呼びます |
| 他の言語、まじめモード | `apps/web/src/copy/en.ts` の隣に、`Copy` 型の辞書を追加 |
| Tauri のデスクトップ / モバイルアプリ | `VITE_API_BASE_URL` と、`apps/web/src/router.tsx` の `createHashRouter` |

## 決めたこと

- **受信箱ゼロ**とは、`open`、`in_progress`、`waiting_on_customer` のチケットが1件もない状態です。解決済みと
  クローズ済みのチケットは、受信箱のステータスフィルターから引き続き見られます。
- **ぶん投げる（Yeet）** = 対応者から見たクローズです。チケットは `resolved`（「ぶん投げ済み」）になります。
  お客様が返信すると `open` に戻ります（「生き返った！」）。`closed`（「闇の世界へ送還」）が最終状態です。
- **秘密リンクをなくした場合**: MVP の既知の制限です。取り戻す方法はありません（メールがあれば解決します）。
- **名前**: どちらの言語でも「Nullbox」。日本語のサブタイトルに「ヌルボックス」が入ります。
- **ID とタイムスタンプ**はテキストで保存し（uuid v7、RFC 3339 の UTC）、Rust 側で生成します。そのため、スキーマは
  型を変えるだけで Postgres に移せます。チケット番号はリポジトリ層で採番します。
- `X-Actor-Name` はパーセントエンコードするので、ASCII 以外の名前（山田）も HTTP ヘッダーを通れます。

## データベースを変更するとき

クエリはコンパイル時に `.sqlx/` と照合されます。クエリやマイグレーションを変更したら:

```sh
cargo install sqlx-cli --no-default-features --features sqlite   # 初回だけ
export DATABASE_URL=sqlite://nullbox.db
sqlx database create && sqlx migrate run
cargo sqlx prepare --workspace     # .sqlx を更新して、コミットします
```

## 伝承

はじめに受信箱があった。受信箱は形なく、そして満杯であった。

返信のないチケットは、すべて**バックログの獣**の餌になります。ゼロ件のとき、獣は
`crates/domain/src/beast.rs` のどこかで丸くなって眠っています。数件たまると小腹が空いてきて、こちらを
見つめ始めます。6件を超えるとご機嫌ななめになり、湯気を立て始めます。20件を超えると怪獣になり、
もう誰も安全ではありません。知られている唯一の治療法は、チケットに返信することです。

対応者はチケットを閉じません。**ぶん投げ**ます。ぶん投げたチケットのほとんどは、そのまま消えます。
しかし、戻ってくるものもあります。お客様が返信すると、チケットは墓場から蘇るのです。本当に終わった
チケットは**闇の世界**へ送られ、そこから戻るものはありません（`TicketStatus::can_transition_to` を参照）。

チケット #404 は存在します。見つけた者は、まだ誰もいません。

↑ ↑ ↓ ↓ ← → ← → B A と入力すると、アプリがエンタープライズソフトウェアに変わるという噂があります。
そのフォームを埋めた人たちは、今も承認を待っています。

サーバーがパニックすると、`EXISTENTIAL` というターゲットでログを出します。DNS のせいではありません。
DNS のせいだったことなど一度もありません。（DNS のせいでした: `ApiError::ProbablyDns` を参照。）

### イースターエッグ

- コナミコマンド: エンタープライズモード™。理不尽な必須項目と、永遠に終わらない「シナジー創出中…」
- チケット #404 は「見つかりません」と表示されます
- 受信箱の検索に `sudo` と入力: 「この件は報告されます。」
- 返信中に Caps Lock がオン: 「優先度: 大声」（見た目だけです。正直に言うと）
- ランダムなローディングメッセージ。例: 「データベースに袖の下を渡しています…」
- `GET /api/teapot`

## ライセンス

[The Unlicense](LICENSE): パブリックドメインです。ご自由にどうぞ。
