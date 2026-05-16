// Sobr.ia extension — client Native Messaging vers `sobria-bridge` (C27.5).
//
// Le bridge natif (`com.sobria.bridge`) est un binaire local installé par
// l'app Sobr.ia desktop. WebExtensions length-prefix le JSON automatiquement
// (uint32 LE + payload), `chrome.runtime.connectNative` retourne un `Port`.
//
// Protocole côté bridge (cf. crates/sobria-bridge/src/main.rs) :
//   Pair{ code }             → { ok: true, secret, pairingId, fingerprint } | { ok: false, error }
//   Estimate{ secret, payload } → { ok: true } | { ok: false, error }
//   Revoke{ secret }         → { ok: true } | { ok: false, error }
//   Ping                     → { pong: true }
//
// Si l'app n'est pas lancée mais que le manifest natif existe, le bridge
// pousse vers `~/.sobria/spool/incoming.jsonl`. Si le bridge n'est pas
// installé du tout, `chrome.runtime.connectNative` lève → on attrape proprement.

import type { Estimate } from '../lib/types.js';

const NATIVE_APP_NAME = 'com.sobria.bridge';
const REQUEST_TIMEOUT_MS = 5000;

/** Requête envoyée au bridge — discriminée par `type`. */
type BridgeRequest =
  | { type: 'ping'; reqId: string }
  | { type: 'pair'; reqId: string; code: string }
  | {
      type: 'estimate';
      reqId: string;
      secret: string;
      payload: {
        estimate: Estimate;
        host: 'chatgpt' | 'claude' | 'le-chat';
        modelDisplayName: string;
        ts: string;
      };
    }
  | { type: 'revoke'; reqId: string; secret: string };

/** Réponse du bridge — toutes portent `reqId` pour le matching. */
type BridgeResponse = {
  readonly reqId: string;
  readonly ok: boolean;
  readonly error?: string;
  readonly secret?: string;
  readonly pairingId?: string;
  readonly fingerprint?: string;
  readonly pong?: boolean;
};

/** Client BridgeClient minimal façade. Re-connecte automatiquement. */
export class BridgeClient {
  private port: chrome.runtime.Port | null = null;
  private pending = new Map<string, (r: BridgeResponse) => void>();
  private reqCounter = 0;

  /**
   * Tente d'ouvrir un port natif. Retourne `true` si l'app desktop répond
   * à un Ping, `false` sinon (bridge non installé, app éteinte, etc.).
   */
  async connect(): Promise<boolean> {
    if (!chrome.runtime?.connectNative) return false;
    try {
      this.port = chrome.runtime.connectNative(NATIVE_APP_NAME);
      this.port.onMessage.addListener(this.onMessage.bind(this));
      this.port.onDisconnect.addListener(() => this.handleDisconnect());
    } catch (err) {
      console.warn('[sobria bridge] connectNative throws:', err);
      this.port = null;
      return false;
    }
    // Ping de santé pour confirmer que le bridge répond.
    try {
      const res = await this.send({ type: 'ping', reqId: this.nextReqId() });
      return res.pong === true;
    } catch {
      return false;
    }
  }

  /** Tente un pairing avec le code 6 chiffres saisi par l'utilisateur. */
  async pair(code: string): Promise<BridgeResponse> {
    return this.send({ type: 'pair', reqId: this.nextReqId(), code });
  }

  /** Pousse une estimation au bridge (qui forwarde à l'app ou spool). */
  async sendEstimate(
    secret: string,
    payload: {
      estimate: Estimate;
      host: 'chatgpt' | 'claude' | 'le-chat';
      modelDisplayName: string;
      ts: string;
    }
  ): Promise<BridgeResponse> {
    return this.send({ type: 'estimate', reqId: this.nextReqId(), secret, payload });
  }

  /** Révoque le secret côté app. L'extension nettoie son storage en parallèle. */
  async revoke(secret: string): Promise<BridgeResponse> {
    return this.send({ type: 'revoke', reqId: this.nextReqId(), secret });
  }

  /** Ferme le port natif (cleanup). */
  disconnect(): void {
    if (this.port) {
      try {
        this.port.disconnect();
      } catch {
        /* déjà fermé */
      }
      this.port = null;
    }
    this.pending.clear();
  }

  // ─── Internes ────────────────────────────────────────────────────────────

  private nextReqId(): string {
    this.reqCounter += 1;
    return `r${Date.now().toString(36)}-${this.reqCounter}`;
  }

  private send(req: BridgeRequest): Promise<BridgeResponse> {
    const port = this.port;
    if (!port) {
      return Promise.reject(new Error('Bridge non connecté'));
    }
    return new Promise<BridgeResponse>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(req.reqId);
        reject(new Error(`Bridge timeout (${REQUEST_TIMEOUT_MS} ms)`));
      }, REQUEST_TIMEOUT_MS);

      this.pending.set(req.reqId, (response) => {
        clearTimeout(timer);
        resolve(response);
      });

      try {
        port.postMessage(req);
      } catch (err) {
        clearTimeout(timer);
        this.pending.delete(req.reqId);
        reject(err);
      }
    });
  }

  private onMessage(msg: unknown): void {
    if (typeof msg !== 'object' || msg === null) return;
    const response = msg as BridgeResponse;
    if (typeof response.reqId !== 'string') return;
    const handler = this.pending.get(response.reqId);
    if (handler) {
      this.pending.delete(response.reqId);
      handler(response);
    }
  }

  private handleDisconnect(): void {
    if (chrome.runtime.lastError) {
      console.warn('[sobria bridge] disconnect:', chrome.runtime.lastError.message);
    }
    this.port = null;
    // Reject toutes les requêtes en cours.
    for (const [, handler] of this.pending) {
      handler({ reqId: '', ok: false, error: 'Bridge déconnecté' });
    }
    this.pending.clear();
  }
}
